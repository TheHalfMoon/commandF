use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use syn::visit::{self, Visit};
use syn::{
    Expr, ExprAssign, ExprCall, ExprMethodCall, ExprStruct, Item, ItemMod, Local, Macro, Pat,
    Path as SynPath, UseTree,
};
use thiserror::Error;

use crate::canonical::parse_json_no_duplicates;

const SURFACE_SCHEMA: &str = "commandf.af02-surface-policy/v1";
const SCANNER_TOOL_ID: &str = "syn-af02-scanner";
const SOURCE_ROOTS: [&str; 2] = ["crates/**/src/**/*.rs", "tools/**/src/**/*.rs"];
const BOUNDARY_CATEGORIES: [&str; 6] = [
    "ARCHIVE_OR_COMPRESSION",
    "CACHE_OR_PERSISTENCE",
    "FILESYSTEM",
    "NETWORK_OR_ACQUISITION",
    "SERDE_OR_TEXT_PARSE",
    "SUBPROCESS",
];

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("surface policy JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("surface policy violation: {0}")]
    Policy(String),
    #[error("Rust source parse failed for {path}: {source}")]
    RustParse { path: String, source: syn::Error },
    #[error("repository source path is invalid: {0}")]
    SourcePath(String),
    #[error("repository source is not a regular file: {0}")]
    NonRegularSource(String),
    #[error("I/O error for {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("git ls-files failed: {0}")]
    Git(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfacePolicy {
    pub schema: String,
    pub lineage: PolicyLineage,
    pub source_sha: String,
    pub source_tree: String,
    pub source_roots: Vec<String>,
    pub exclusion_policy_sha256: String,
    pub scanner_tool_id: String,
    pub boundary_categories: Vec<String>,
    pub matchers: Vec<Matcher>,
    pub critical_surfaces: Vec<CriticalSurface>,
    pub known_boundary_witnesses: Vec<BoundaryWitness>,
    pub finding_exclusions: Vec<FindingExclusion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyLineage {
    pub mode: PolicyLineageMode,
    pub canonical_base_sha: String,
    pub canonical_base_tree: String,
    pub policy_path: String,
    pub predecessor_blob_sha: Option<String>,
    pub predecessor_sha256: Option<String>,
    pub change_is_policy_only: bool,
    pub dependent_evidence_allowed_in_same_candidate: bool,
    pub comparison_rule: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyLineageMode {
    Bootstrap,
    Rebase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matcher {
    pub matcher_id: String,
    pub category: String,
    pub kind: MatcherKind,
    pub callee_or_method: String,
    pub import_roots: Vec<String>,
    pub receiver_constructor_or_null: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MatcherKind {
    PathCall,
    MethodCall,
    MacroToken,
    TypeConstructor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriticalSurface {
    pub surface_id: String,
    pub category: String,
    pub matcher_ids: Vec<String>,
    pub source_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryWitness {
    pub witness_id: String,
    pub category: String,
    pub matcher_id: String,
    pub source_path: String,
    pub source_blob_sha: String,
    pub symbol_or_span: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindingExclusion {
    pub exclusion_id: String,
    pub matcher_id: String,
    pub source_path: String,
    pub reason: String,
    pub introduced_policy_sha: String,
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingCertainty {
    Definite,
    Uncertain,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Finding {
    pub source_path: String,
    pub syntax_ordinal: u64,
    pub matcher_id: String,
    pub category: String,
    pub certainty: FindingCertainty,
}

pub fn parse_surface_policy(bytes: &[u8]) -> Result<SurfacePolicy, SurfaceError> {
    let value = parse_json_no_duplicates(bytes)?;
    let policy: SurfacePolicy = serde_json::from_value(value)?;
    validate_surface_policy(&policy)?;
    Ok(policy)
}

pub fn discover_tracked_rust_sources(repo_root: &Path) -> Result<Vec<SourceFile>, SurfaceError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["ls-files", "-z", "--", "crates", "tools"])
        .output()
        .map_err(|source| SurfaceError::Io {
            path: repo_root.display().to_string(),
            source,
        })?;
    if !output.status.success() {
        return Err(SurfaceError::Git(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }

    let mut paths = Vec::new();
    for raw in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|raw| !raw.is_empty())
    {
        let path = std::str::from_utf8(raw)
            .map_err(|_| SurfaceError::SourcePath("git returned a non-UTF-8 path".to_owned()))?;
        if is_surface_source_path(path) {
            validate_repo_path(path)?;
            paths.push(path.to_owned());
        }
    }
    paths.sort();
    paths.dedup();

    let mut sources = Vec::with_capacity(paths.len());
    for path in paths {
        let absolute = repo_root.join(&path);
        let metadata = fs::symlink_metadata(&absolute).map_err(|source| SurfaceError::Io {
            path: path.clone(),
            source,
        })?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            return Err(SurfaceError::NonRegularSource(path));
        }
        let bytes = fs::read(&absolute).map_err(|source| SurfaceError::Io {
            path: path.clone(),
            source,
        })?;
        sources.push(SourceFile { path, bytes });
    }
    Ok(sources)
}

pub fn scan_surface(
    policy: &SurfacePolicy,
    sources: &[SourceFile],
) -> Result<Vec<Finding>, SurfaceError> {
    validate_surface_policy(policy)?;
    let mut ordered = sources.to_vec();
    ordered.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));

    let mut findings = Vec::new();
    for source in ordered {
        validate_repo_path(&source.path)?;
        if !is_surface_source_path(&source.path) {
            return Err(SurfaceError::SourcePath(format!(
                "{} is outside the closed Rust source roots",
                source.path
            )));
        }
        let text = std::str::from_utf8(&source.bytes)
            .map_err(|_| SurfaceError::SourcePath(format!("{} is not UTF-8", source.path)))?;
        let file = syn::parse_file(text).map_err(|source_error| SurfaceError::RustParse {
            path: source.path.clone(),
            source: source_error,
        })?;
        let mut ordinal = 0_u64;
        let module_path = module_path_from_repo_path(&source.path)?;
        scan_module_items(
            policy,
            &source.path,
            &file.items,
            &module_path,
            &mut ordinal,
            &mut findings,
        );
    }

    findings.sort_by(|left, right| {
        left.source_path
            .as_bytes()
            .cmp(right.source_path.as_bytes())
            .then(left.syntax_ordinal.cmp(&right.syntax_ordinal))
            .then(left.matcher_id.as_bytes().cmp(right.matcher_id.as_bytes()))
    });
    let mut identities = BTreeSet::new();
    for finding in &findings {
        let key = (
            finding.source_path.as_str(),
            finding.syntax_ordinal,
            finding.matcher_id.as_str(),
        );
        if !identities.insert(key) {
            return Err(SurfaceError::Policy(format!(
                "duplicate finding identity {}:{}:{}",
                finding.source_path, finding.syntax_ordinal, finding.matcher_id
            )));
        }
    }
    Ok(findings)
}

fn validate_surface_policy(policy: &SurfacePolicy) -> Result<(), SurfaceError> {
    if policy.schema != SURFACE_SCHEMA {
        return policy_error("unexpected surface policy schema");
    }
    if policy.scanner_tool_id != SCANNER_TOOL_ID {
        return policy_error("unexpected scanner_tool_id");
    }
    if policy.source_roots != SOURCE_ROOTS {
        return policy_error("source_roots do not equal the closed source roots");
    }
    if policy.boundary_categories != BOUNDARY_CATEGORIES {
        return policy_error("boundary_categories do not equal the closed category order");
    }
    validate_hex(&policy.source_sha, 40, "source_sha")?;
    validate_hex(&policy.source_tree, 40, "source_tree")?;
    validate_hex(
        &policy.exclusion_policy_sha256,
        64,
        "exclusion_policy_sha256",
    )?;
    validate_lineage(&policy.lineage)?;

    let allowed_categories: BTreeSet<&str> = BOUNDARY_CATEGORIES.into_iter().collect();
    let mut matcher_ids = BTreeSet::new();
    let mut matcher_category = BTreeMap::new();
    let mut matcher_categories = BTreeSet::new();
    for matcher in &policy.matchers {
        validate_id(&matcher.matcher_id, "matcher_id")?;
        validate_category(&matcher.category, &allowed_categories)?;
        if !matcher_ids.insert(matcher.matcher_id.as_str()) {
            return policy_error(format!("duplicate matcher_id {}", matcher.matcher_id));
        }
        matcher_categories.insert(matcher.category.as_str());
        matcher_category.insert(matcher.matcher_id.as_str(), matcher.category.as_str());
        if matcher.callee_or_method.is_empty() || matcher.callee_or_method.len() > 256 {
            return policy_error(format!(
                "matcher {} has invalid callee_or_method",
                matcher.matcher_id
            ));
        }
        ensure_unique_strings(
            matcher.import_roots.iter().map(String::as_str),
            &format!("matcher {} import_roots", matcher.matcher_id),
        )?;
        if matcher
            .import_roots
            .iter()
            .any(|root| root.is_empty() || root.len() > 256)
        {
            return policy_error(format!(
                "matcher {} has invalid import root",
                matcher.matcher_id
            ));
        }
        if matcher.kind != MatcherKind::MethodCall && matcher.receiver_constructor_or_null.is_some()
        {
            return policy_error(format!(
                "matcher {} sets receiver constructor for non-method matcher",
                matcher.matcher_id
            ));
        }
        if matcher
            .receiver_constructor_or_null
            .as_deref()
            .is_some_and(|value| value.is_empty() || value.len() > 256)
        {
            return policy_error(format!(
                "matcher {} has invalid receiver constructor",
                matcher.matcher_id
            ));
        }
    }
    ensure_all_categories("matchers", &matcher_categories)?;

    let mut surface_ids = BTreeSet::new();
    let mut surface_categories = BTreeSet::new();
    for surface in &policy.critical_surfaces {
        validate_id(&surface.surface_id, "surface_id")?;
        validate_category(&surface.category, &allowed_categories)?;
        if !surface_ids.insert(surface.surface_id.as_str()) {
            return policy_error(format!("duplicate surface_id {}", surface.surface_id));
        }
        surface_categories.insert(surface.category.as_str());
        if surface.matcher_ids.is_empty() || surface.source_paths.is_empty() {
            return policy_error(format!(
                "surface {} has an empty membership",
                surface.surface_id
            ));
        }
        ensure_unique_strings(
            surface.matcher_ids.iter().map(String::as_str),
            &format!("surface {} matcher_ids", surface.surface_id),
        )?;
        ensure_unique_strings(
            surface.source_paths.iter().map(String::as_str),
            &format!("surface {} source_paths", surface.surface_id),
        )?;
        for matcher_id in &surface.matcher_ids {
            let category = matcher_category.get(matcher_id.as_str()).ok_or_else(|| {
                SurfaceError::Policy(format!(
                    "surface {} references unknown matcher {}",
                    surface.surface_id, matcher_id
                ))
            })?;
            if *category != surface.category {
                return policy_error(format!(
                    "surface {} category disagrees with matcher {}",
                    surface.surface_id, matcher_id
                ));
            }
        }
        for path in &surface.source_paths {
            validate_repo_path(path)?;
        }
    }
    ensure_all_categories("critical_surfaces", &surface_categories)?;

    let mut witness_ids = BTreeSet::new();
    let mut witness_categories = BTreeSet::new();
    for witness in &policy.known_boundary_witnesses {
        validate_id(&witness.witness_id, "witness_id")?;
        validate_category(&witness.category, &allowed_categories)?;
        if !witness_ids.insert(witness.witness_id.as_str()) {
            return policy_error(format!("duplicate witness_id {}", witness.witness_id));
        }
        witness_categories.insert(witness.category.as_str());
        let category = matcher_category
            .get(witness.matcher_id.as_str())
            .ok_or_else(|| {
                SurfaceError::Policy(format!(
                    "witness {} references unknown matcher {}",
                    witness.witness_id, witness.matcher_id
                ))
            })?;
        if *category != witness.category {
            return policy_error(format!(
                "witness {} category disagrees with matcher {}",
                witness.witness_id, witness.matcher_id
            ));
        }
        validate_repo_path(&witness.source_path)?;
        validate_hex(&witness.source_blob_sha, 40, "source_blob_sha")?;
        if witness.symbol_or_span.is_empty() || witness.symbol_or_span.len() > 512 {
            return policy_error(format!(
                "witness {} has invalid symbol_or_span",
                witness.witness_id
            ));
        }
    }
    ensure_all_categories("known_boundary_witnesses", &witness_categories)?;

    let mut exclusion_ids = BTreeSet::new();
    for exclusion in &policy.finding_exclusions {
        if !is_exclusion_id(&exclusion.exclusion_id) {
            return policy_error(format!(
                "invalid finding exclusion id {}",
                exclusion.exclusion_id
            ));
        }
        if !exclusion_ids.insert(exclusion.exclusion_id.as_str()) {
            return policy_error(format!(
                "duplicate finding exclusion id {}",
                exclusion.exclusion_id
            ));
        }
        if !matcher_ids.contains(exclusion.matcher_id.as_str()) {
            return policy_error(format!(
                "finding exclusion {} references unknown matcher {}",
                exclusion.exclusion_id, exclusion.matcher_id
            ));
        }
        validate_repo_path(&exclusion.source_path)?;
        validate_hex(
            &exclusion.introduced_policy_sha,
            40,
            "introduced_policy_sha",
        )?;
        if exclusion.reason.len() < 20 || exclusion.reason.len() > 2048 {
            return policy_error(format!(
                "finding exclusion {} has invalid reason length",
                exclusion.exclusion_id
            ));
        }
    }
    Ok(())
}

fn validate_lineage(lineage: &PolicyLineage) -> Result<(), SurfaceError> {
    validate_hex(&lineage.canonical_base_sha, 40, "canonical_base_sha")?;
    validate_hex(&lineage.canonical_base_tree, 40, "canonical_base_tree")?;
    if lineage.policy_path != "specs/016-af-02-adversarial-test-strength/surface-policy.json" {
        return policy_error("lineage policy_path is not the closed surface policy path");
    }
    if !lineage.change_is_policy_only {
        return policy_error("surface policy lineage must be policy-only");
    }
    if lineage.dependent_evidence_allowed_in_same_candidate {
        return policy_error(
            "surface policy lineage cannot allow same-candidate dependent evidence",
        );
    }
    if lineage.comparison_rule != "BASE_CONTROLLED_PREDECESSOR_OR_SINGLE_BOOTSTRAP" {
        return policy_error("unexpected surface policy comparison_rule");
    }
    match lineage.mode {
        PolicyLineageMode::Bootstrap => {
            if lineage.predecessor_blob_sha.is_some() || lineage.predecessor_sha256.is_some() {
                return policy_error("BOOTSTRAP lineage cannot carry predecessor identity");
            }
        }
        PolicyLineageMode::Rebase => {
            let blob = lineage.predecessor_blob_sha.as_deref().ok_or_else(|| {
                SurfaceError::Policy("REBASE lineage missing predecessor blob".into())
            })?;
            let digest = lineage.predecessor_sha256.as_deref().ok_or_else(|| {
                SurfaceError::Policy("REBASE lineage missing predecessor digest".into())
            })?;
            validate_hex(blob, 40, "predecessor_blob_sha")?;
            validate_hex(digest, 64, "predecessor_sha256")?;
        }
    }
    Ok(())
}

fn scan_module_items(
    policy: &SurfacePolicy,
    source_path: &str,
    items: &[Item],
    module_path: &[String],
    ordinal: &mut u64,
    findings: &mut Vec<Finding>,
) {
    let imports = collect_imports(items, module_path);
    let local_items = collect_local_items(items);
    for item in items {
        if let Item::Mod(module) = item {
            if let Some((_, nested)) = &module.content {
                let mut nested_path = module_path.to_vec();
                nested_path.push(module.ident.to_string());
                scan_module_items(policy, source_path, nested, &nested_path, ordinal, findings);
            }
            continue;
        }
        let mut visitor = ScannerVisitor {
            policy,
            source_path,
            module_path,
            imports: &imports,
            local_items: &local_items,
            ordinal,
            findings,
            bindings: vec![BTreeMap::new()],
        };
        visitor.visit_item(item);
    }
}

#[derive(Debug, Default)]
struct ImportTable {
    aliases: BTreeMap<String, Vec<String>>,
    globs: Vec<Vec<String>>,
}

fn collect_imports(items: &[Item], module_path: &[String]) -> ImportTable {
    let mut table = ImportTable::default();
    for item in items {
        if let Item::Use(item_use) = item {
            expand_use_tree(&item_use.tree, Vec::new(), module_path, &mut table);
        }
    }
    table.globs.sort();
    table.globs.dedup();
    table
}

fn collect_local_items(items: &[Item]) -> BTreeSet<String> {
    items
        .iter()
        .filter_map(|item| match item {
            Item::Const(item) => Some(item.ident.to_string()),
            Item::Enum(item) => Some(item.ident.to_string()),
            Item::Fn(item) => Some(item.sig.ident.to_string()),
            Item::Mod(item) => Some(item.ident.to_string()),
            Item::Static(item) => Some(item.ident.to_string()),
            Item::Struct(item) => Some(item.ident.to_string()),
            Item::Trait(item) => Some(item.ident.to_string()),
            Item::TraitAlias(item) => Some(item.ident.to_string()),
            Item::Type(item) => Some(item.ident.to_string()),
            Item::Union(item) => Some(item.ident.to_string()),
            _ => None,
        })
        .collect()
}

fn expand_use_tree(
    tree: &UseTree,
    mut prefix: Vec<String>,
    module_path: &[String],
    table: &mut ImportTable,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            expand_use_tree(&path.tree, prefix, module_path, table);
        }
        UseTree::Name(name) => {
            let ident = name.ident.to_string();
            if ident == "self" {
                if !prefix.is_empty() {
                    let target = normalize_special_segments(&prefix, module_path);
                    if let Some(alias) = target.last().cloned() {
                        table.aliases.insert(alias, target);
                    }
                }
            } else {
                prefix.push(ident.clone());
                table
                    .aliases
                    .insert(ident, normalize_special_segments(&prefix, module_path));
            }
        }
        UseTree::Rename(rename) => {
            let ident = rename.ident.to_string();
            if ident != "self" {
                prefix.push(ident);
            }
            table.aliases.insert(
                rename.rename.to_string(),
                normalize_special_segments(&prefix, module_path),
            );
        }
        UseTree::Glob(_) => {
            table
                .globs
                .push(normalize_special_segments(&prefix, module_path));
        }
        UseTree::Group(group) => {
            for item in &group.items {
                expand_use_tree(item, prefix.clone(), module_path, table);
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Binding {
    Known(Vec<String>),
    Unknown,
}

struct ScannerVisitor<'a> {
    policy: &'a SurfacePolicy,
    source_path: &'a str,
    module_path: &'a [String],
    imports: &'a ImportTable,
    local_items: &'a BTreeSet<String>,
    ordinal: &'a mut u64,
    findings: &'a mut Vec<Finding>,
    bindings: Vec<BTreeMap<String, Binding>>,
}

impl ScannerVisitor<'_> {
    fn next_ordinal(&mut self) -> u64 {
        let current = *self.ordinal;
        *self.ordinal = current.saturating_add(1);
        current
    }

    fn classify_path(&mut self, ordinal: u64, path: &SynPath, allowed_kinds: &[MatcherKind]) {
        let raw = syn_path_segments(path);
        let resolved =
            resolve_segments_with_locals(&raw, self.module_path, self.imports, self.local_items);
        for matcher in &self.policy.matchers {
            if !allowed_kinds.contains(&matcher.kind) {
                continue;
            }
            if exact_match(&resolved, matcher) {
                self.findings.push(Finding {
                    source_path: self.source_path.to_owned(),
                    syntax_ordinal: ordinal,
                    matcher_id: matcher.matcher_id.clone(),
                    category: matcher.category.clone(),
                    certainty: FindingCertainty::Definite,
                });
            } else if glob_could_resolve(&raw, matcher, self.imports) {
                self.findings.push(Finding {
                    source_path: self.source_path.to_owned(),
                    syntax_ordinal: ordinal,
                    matcher_id: matcher.matcher_id.clone(),
                    category: matcher.category.clone(),
                    certainty: FindingCertainty::Uncertain,
                });
            }
        }
    }

    fn bind_local(&mut self, local: &Local) {
        let Pat::Ident(ident) = &local.pat else {
            return;
        };
        let name = ident.ident.to_string();
        let binding = if ident.mutability.is_none() {
            local
                .init
                .as_ref()
                .and_then(|init| direct_call_path(&init.expr))
                .and_then(|path| {
                    let raw = syn_path_segments(path);
                    let resolved = resolve_segments_with_locals(
                        &raw,
                        self.module_path,
                        self.imports,
                        self.local_items,
                    );
                    if path_has_glob_ambiguity(&raw, self.imports) {
                        None
                    } else {
                        Some(Binding::Known(resolved))
                    }
                })
                .unwrap_or(Binding::Unknown)
        } else {
            Binding::Unknown
        };
        self.bindings
            .last_mut()
            .expect("binding scope is always present")
            .insert(name, binding);
    }

    fn invalidate_assignment(&mut self, assign: &ExprAssign) {
        if let Expr::Path(path) = assign.left.as_ref() {
            if path.path.segments.len() == 1 {
                let name = path.path.segments[0].ident.to_string();
                for scope in self.bindings.iter_mut().rev() {
                    if let Some(binding) = scope.get_mut(&name) {
                        *binding = Binding::Unknown;
                        break;
                    }
                }
            }
        }
    }

    fn classify_method(&mut self, ordinal: u64, call: &ExprMethodCall) {
        let method_name = call.method.to_string();
        for matcher in &self.policy.matchers {
            if matcher.kind != MatcherKind::MethodCall || matcher.callee_or_method != method_name {
                continue;
            }
            let Some(constructor) = matcher.receiver_constructor_or_null.as_deref() else {
                self.findings.push(Finding {
                    source_path: self.source_path.to_owned(),
                    syntax_ordinal: ordinal,
                    matcher_id: matcher.matcher_id.clone(),
                    category: matcher.category.clone(),
                    certainty: FindingCertainty::Definite,
                });
                continue;
            };
            let expected = expected_constructor_paths(matcher, constructor);
            match receiver_ownership(
                &call.receiver,
                &expected,
                self.module_path,
                self.imports,
                self.local_items,
                &self.bindings,
            ) {
                Ownership::Expected => self.findings.push(Finding {
                    source_path: self.source_path.to_owned(),
                    syntax_ordinal: ordinal,
                    matcher_id: matcher.matcher_id.clone(),
                    category: matcher.category.clone(),
                    certainty: FindingCertainty::Definite,
                }),
                Ownership::Unknown => self.findings.push(Finding {
                    source_path: self.source_path.to_owned(),
                    syntax_ordinal: ordinal,
                    matcher_id: matcher.matcher_id.clone(),
                    category: matcher.category.clone(),
                    certainty: FindingCertainty::Uncertain,
                }),
                Ownership::Different => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for ScannerVisitor<'_> {
    fn visit_item_mod(&mut self, _node: &'ast ItemMod) {}

    fn visit_block(&mut self, node: &'ast syn::Block) {
        self.bindings.push(BTreeMap::new());
        visit::visit_block(self, node);
        self.bindings.pop();
    }

    fn visit_local(&mut self, node: &'ast Local) {
        visit::visit_local(self, node);
        self.bind_local(node);
    }

    fn visit_expr_assign(&mut self, node: &'ast ExprAssign) {
        self.invalidate_assignment(node);
        visit::visit_expr_assign(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        let ordinal = self.next_ordinal();
        if let Expr::Path(path) = node.func.as_ref() {
            self.classify_path(
                ordinal,
                &path.path,
                &[MatcherKind::PathCall, MatcherKind::TypeConstructor],
            );
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let ordinal = self.next_ordinal();
        self.classify_method(ordinal, node);
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast ExprStruct) {
        let ordinal = self.next_ordinal();
        self.classify_path(ordinal, &node.path, &[MatcherKind::TypeConstructor]);
        visit::visit_expr_struct(self, node);
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        let ordinal = self.next_ordinal();
        self.classify_path(ordinal, &node.path, &[MatcherKind::MacroToken]);
        visit::visit_macro(self, node);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ownership {
    Expected,
    Different,
    Unknown,
}

fn receiver_ownership(
    receiver: &Expr,
    expected: &[Vec<String>],
    module_path: &[String],
    imports: &ImportTable,
    local_items: &BTreeSet<String>,
    bindings: &[BTreeMap<String, Binding>],
) -> Ownership {
    if let Some(path) = direct_call_path(receiver) {
        let raw = syn_path_segments(path);
        let resolved = resolve_segments_with_locals(&raw, module_path, imports, local_items);
        if expected.contains(&resolved) {
            return Ownership::Expected;
        }
        if glob_matches_any_expected(&raw, expected, imports) {
            return Ownership::Unknown;
        }
        return Ownership::Different;
    }

    if let Expr::Path(path) = receiver {
        if path.path.segments.len() == 1 {
            let name = path.path.segments[0].ident.to_string();
            if let Some(binding) = bindings.iter().rev().find_map(|scope| scope.get(&name)) {
                return match binding {
                    Binding::Known(path) if expected.contains(path) => Ownership::Expected,
                    Binding::Known(_) => Ownership::Different,
                    Binding::Unknown => Ownership::Unknown,
                };
            }
        }
    }
    Ownership::Unknown
}

fn direct_call_path(expr: &Expr) -> Option<&SynPath> {
    let Expr::Call(call) = expr else {
        return None;
    };
    let Expr::Path(path) = call.func.as_ref() else {
        return None;
    };
    Some(&path.path)
}

fn exact_match(resolved: &[String], matcher: &Matcher) -> bool {
    matcher.import_roots.iter().any(|root| {
        let mut expected = split_canonical(root);
        expected.push(matcher.callee_or_method.clone());
        expected == resolved
    })
}

fn expected_constructor_paths(matcher: &Matcher, constructor: &str) -> Vec<Vec<String>> {
    matcher
        .import_roots
        .iter()
        .map(|root| {
            let mut expected = split_canonical(root);
            expected.push(constructor.to_owned());
            expected
        })
        .collect()
}

fn glob_could_resolve(raw: &[String], matcher: &Matcher, imports: &ImportTable) -> bool {
    let expected = matcher
        .import_roots
        .iter()
        .map(|root| {
            let mut value = split_canonical(root);
            value.push(matcher.callee_or_method.clone());
            value
        })
        .collect::<Vec<_>>();
    glob_matches_any_expected(raw, &expected, imports)
}

fn glob_matches_any_expected(
    raw: &[String],
    expected: &[Vec<String>],
    imports: &ImportTable,
) -> bool {
    imports.globs.iter().any(|glob| {
        expected.iter().any(|expected_path| {
            if expected_path.starts_with(glob) {
                expected_path[glob.len()..] == *raw
            } else {
                false
            }
        })
    })
}

fn path_has_glob_ambiguity(raw: &[String], imports: &ImportTable) -> bool {
    let Some(first) = raw.first() else {
        return false;
    };
    if imports.aliases.contains_key(first) || matches!(first.as_str(), "crate" | "self" | "super") {
        return false;
    }
    imports.globs.iter().any(|glob| {
        !glob.is_empty()
            && (raw.len() == 1
                || raw
                    .first()
                    .is_some_and(|segment| glob.last().is_some_and(|last| last != segment)))
    })
}

fn resolve_segments(raw: &[String], module_path: &[String], imports: &ImportTable) -> Vec<String> {
    let mut current = normalize_special_segments(raw, module_path);
    let mut seen = BTreeSet::new();
    for _ in 0..=imports.aliases.len() {
        let Some(first) = current.first().cloned() else {
            break;
        };
        if !seen.insert(first.clone()) {
            break;
        }
        let Some(target) = imports.aliases.get(&first) else {
            break;
        };
        let mut expanded = target.clone();
        expanded.extend(current.into_iter().skip(1));
        current = normalize_special_segments(&expanded, module_path);
    }
    current
}

fn resolve_segments_with_locals(
    raw: &[String],
    module_path: &[String],
    imports: &ImportTable,
    local_items: &BTreeSet<String>,
) -> Vec<String> {
    let resolved = resolve_segments(raw, module_path, imports);
    if resolved != raw {
        return resolved;
    }
    let Some(first) = raw.first() else {
        return resolved;
    };
    if !local_items.contains(first) {
        return resolved;
    }
    let mut canonical = vec!["crate".to_owned()];
    canonical.extend(module_path.iter().cloned());
    canonical.extend(raw.iter().cloned());
    canonical
}

fn normalize_special_segments(raw: &[String], module_path: &[String]) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    match raw[0].as_str() {
        "crate" => {
            let mut out = vec!["crate".to_owned()];
            out.extend(raw.iter().skip(1).cloned());
            out
        }
        "self" => {
            let mut out = vec!["crate".to_owned()];
            out.extend(module_path.iter().cloned());
            out.extend(raw.iter().skip(1).cloned());
            out
        }
        "super" => {
            let mut supers = 0usize;
            while raw.get(supers).is_some_and(|part| part == "super") {
                supers += 1;
            }
            let keep = module_path.len().saturating_sub(supers);
            let mut out = vec!["crate".to_owned()];
            out.extend(module_path[..keep].iter().cloned());
            out.extend(raw.iter().skip(supers).cloned());
            out
        }
        _ => raw.to_vec(),
    }
}

fn syn_path_segments(path: &SynPath) -> Vec<String> {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect()
}

fn split_canonical(value: &str) -> Vec<String> {
    value.split("::").map(str::to_owned).collect()
}

fn module_path_from_repo_path(path: &str) -> Result<Vec<String>, SurfaceError> {
    validate_repo_path(path)?;
    let parts = path.split('/').collect::<Vec<_>>();
    let src = parts
        .iter()
        .position(|part| *part == "src")
        .ok_or_else(|| SurfaceError::SourcePath(format!("{path} has no src component")))?;
    let tail = &parts[src + 1..];
    let Some(file) = tail.last() else {
        return Err(SurfaceError::SourcePath(format!("{path} has no Rust file")));
    };
    let mut module = tail[..tail.len() - 1]
        .iter()
        .map(|part| (*part).to_owned())
        .collect::<Vec<_>>();
    match *file {
        "lib.rs" | "main.rs" | "mod.rs" => {}
        other => module.push(other.trim_end_matches(".rs").to_owned()),
    }
    Ok(module)
}

fn is_surface_source_path(path: &str) -> bool {
    if !(path.starts_with("crates/") || path.starts_with("tools/")) || !path.ends_with(".rs") {
        return false;
    }
    let parts = path.split('/').collect::<Vec<_>>();
    parts
        .iter()
        .position(|part| *part == "src")
        .is_some_and(|src| src + 1 < parts.len())
}

fn validate_repo_path(path: &str) -> Result<(), SurfaceError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains("//")
        || path.contains('\0')
        || path
            .split('/')
            .any(|component| component == "." || component == ".." || component.is_empty())
    {
        return Err(SurfaceError::SourcePath(path.to_owned()));
    }
    Ok(())
}

fn validate_category(category: &str, allowed: &BTreeSet<&str>) -> Result<(), SurfaceError> {
    if !allowed.contains(category) {
        return policy_error(format!("unknown boundary category {category}"));
    }
    Ok(())
}

fn ensure_all_categories(label: &str, categories: &BTreeSet<&str>) -> Result<(), SurfaceError> {
    let expected: BTreeSet<&str> = BOUNDARY_CATEGORIES.into_iter().collect();
    if *categories != expected {
        return policy_error(format!(
            "{label} does not cover all six boundary categories"
        ));
    }
    Ok(())
}

fn ensure_unique_strings<'a>(
    values: impl Iterator<Item = &'a str>,
    label: &str,
) -> Result<(), SurfaceError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return policy_error(format!("{label} contains duplicate value {value}"));
        }
    }
    Ok(())
}

fn validate_id(value: &str, label: &str) -> Result<(), SurfaceError> {
    if value.is_empty()
        || value.len() > 160
        || !value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphanumeric() || (index > 0 && matches!(byte, b'.' | b'_' | b':' | b'-'))
        })
    {
        return policy_error(format!("invalid {label} {value}"));
    }
    Ok(())
}

fn is_exclusion_id(value: &str) -> bool {
    value.len() == 10
        && value.starts_with("SURF-X")
        && value[6..].bytes().all(|byte| byte.is_ascii_digit())
}

fn validate_hex(value: &str, len: usize, label: &str) -> Result<(), SurfaceError> {
    if value.len() != len
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return policy_error(format!("invalid lowercase hex {label}"));
    }
    Ok(())
}

fn policy_error<T>(message: impl Into<String>) -> Result<T, SurfaceError> {
    Err(SurfaceError::Policy(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANONICAL_POLICY: &[u8] =
        include_bytes!("../../../specs/016-af-02-adversarial-test-strength/surface-policy.json");

    fn policy() -> SurfacePolicy {
        parse_surface_policy(CANONICAL_POLICY).expect("canonical surface policy must parse")
    }

    fn source(text: &str) -> SourceFile {
        SourceFile {
            path: "crates/example/src/lib.rs".to_owned(),
            bytes: text.as_bytes().to_vec(),
        }
    }

    fn findings_for(text: &str) -> Vec<Finding> {
        scan_surface(&policy(), &[source(text)]).expect("scan must succeed")
    }

    #[test]
    fn canonical_surface_policy_parses() {
        let policy = policy();
        assert_eq!(policy.scanner_tool_id, "syn-af02-scanner");
        assert_eq!(policy.boundary_categories, BOUNDARY_CATEGORIES);
    }

    #[test]
    fn explicit_alias_resolves_to_canonical_path() {
        let findings = findings_for("use std::fs as io; fn f() { let _ = io::read(\"fixture\"); }");
        assert!(findings.iter().any(|finding| {
            finding.matcher_id == "filesystem-read"
                && finding.certainty == FindingCertainty::Definite
        }));
    }

    #[test]
    fn glob_import_emits_uncertain_finding() {
        let findings = findings_for("use std::fs::*; fn f() { let _ = read(\"fixture\"); }");
        assert!(findings.iter().any(|finding| {
            finding.matcher_id == "filesystem-read"
                && finding.certainty == FindingCertainty::Uncertain
        }));
    }

    #[test]
    fn comments_and_literals_do_not_create_findings() {
        let findings =
            findings_for(r#"fn f() { let _ = "std::fs::read(\"x\")"; /* std::fs::read("x"); */ }"#);
        assert!(!findings
            .iter()
            .any(|finding| finding.matcher_id == "filesystem-read"));
    }

    #[test]
    fn cfg_disabled_syntax_is_still_scanned() {
        let findings = findings_for("#[cfg(any())] fn f() { let _ = std::fs::read(\"fixture\"); }");
        assert!(findings
            .iter()
            .any(|finding| finding.matcher_id == "filesystem-read"));
    }

    #[test]
    fn null_receiver_method_matcher_matches_syntactically() {
        let findings = findings_for("fn f(value: &str) { let _ = value.parse::<u64>(); }");
        assert!(findings.iter().any(|finding| {
            finding.matcher_id == "parser-method-parse"
                && finding.certainty == FindingCertainty::Definite
        }));
    }

    #[test]
    fn constructor_bound_method_proves_direct_and_immutable_receivers() {
        let mut policy = policy();
        policy.matchers.push(Matcher {
            matcher_id: "test-agent-call".to_owned(),
            category: "NETWORK_OR_ACQUISITION".to_owned(),
            kind: MatcherKind::MethodCall,
            callee_or_method: "call".to_owned(),
            import_roots: vec!["ureq::Agent".to_owned()],
            receiver_constructor_or_null: Some("config_builder".to_owned()),
        });
        let findings = scan_surface(
            &policy,
            &[source(
                "use ureq::Agent; fn f() { Agent::config_builder().call(); let agent = Agent::config_builder(); agent.call(); }",
            )],
        )
        .expect("scan must succeed");
        let exact = findings
            .iter()
            .filter(|finding| finding.matcher_id == "test-agent-call")
            .collect::<Vec<_>>();
        assert_eq!(exact.len(), 2);
        assert!(exact
            .iter()
            .all(|finding| finding.certainty == FindingCertainty::Definite));
    }

    #[test]
    fn mutable_constructor_binding_is_uncertain() {
        let mut policy = policy();
        policy.matchers.push(Matcher {
            matcher_id: "test-agent-call".to_owned(),
            category: "NETWORK_OR_ACQUISITION".to_owned(),
            kind: MatcherKind::MethodCall,
            callee_or_method: "call".to_owned(),
            import_roots: vec!["ureq::Agent".to_owned()],
            receiver_constructor_or_null: Some("config_builder".to_owned()),
        });
        let findings = scan_surface(
            &policy,
            &[source(
                "use ureq::Agent; fn f() { let mut agent = Agent::config_builder(); agent.call(); }",
            )],
        )
        .expect("scan must succeed");
        assert!(findings.iter().any(|finding| {
            finding.matcher_id == "test-agent-call"
                && finding.certainty == FindingCertainty::Uncertain
        }));
    }

    #[test]
    fn macro_matcher_is_exact_and_does_not_scan_literal_tokens() {
        let mut policy = policy();
        policy.matchers.push(Matcher {
            matcher_id: "test-select-macro".to_owned(),
            category: "SUBPROCESS".to_owned(),
            kind: MatcherKind::MacroToken,
            callee_or_method: "select".to_owned(),
            import_roots: vec!["tokio".to_owned()],
            receiver_constructor_or_null: None,
        });
        let findings = scan_surface(
            &policy,
            &[source(
                r#"fn f() { tokio::select! { _ = async {} => {} } let _ = "tokio::select!"; }"#,
            )],
        )
        .expect("scan must succeed");
        assert_eq!(
            findings
                .iter()
                .filter(|finding| finding.matcher_id == "test-select-macro")
                .count(),
            1
        );
    }

    #[test]
    fn bare_local_function_resolves_to_module_canonical_path() {
        let findings = scan_surface(
            &policy(),
            &[SourceFile {
                path: "crates/example/src/source_map.rs".to_owned(),
                bytes: br#"fn portable_relative_path(_: &str, _: &str, _: bool) {}
fn f() { portable_relative_path("fixture", "fixture", true); }"#
                    .to_vec(),
            }],
        )
        .expect("scan must succeed");
        assert!(findings.iter().any(|finding| {
            finding.matcher_id == "filesystem-portable-relative-path"
                && finding.certainty == FindingCertainty::Definite
        }));
    }

    #[test]
    fn bare_local_type_resolves_to_crate_root_canonical_path() {
        let findings = scan_surface(
            &policy(),
            &[SourceFile {
                path: "crates/example/src/main.rs".to_owned(),
                bytes:
                    b"struct Cli; impl Cli { fn try_parse() {} } fn main() { Cli::try_parse(); }"
                        .to_vec(),
            }],
        )
        .expect("scan must succeed");
        assert!(findings.iter().any(|finding| {
            finding.matcher_id == "parser-cli-try-parse"
                && finding.certainty == FindingCertainty::Definite
        }));
    }

    #[test]
    fn undeclared_bare_name_is_not_promoted_to_local_canonical_path() {
        let findings = scan_surface(
            &policy(),
            &[SourceFile {
                path: "crates/example/src/source_map.rs".to_owned(),
                bytes: b"fn f() { portable_relative_path(\"fixture\", \"fixture\", true); }"
                    .to_vec(),
            }],
        )
        .expect("scan must succeed");
        assert!(!findings
            .iter()
            .any(|finding| finding.matcher_id == "filesystem-portable-relative-path"));
    }

    #[test]
    fn source_order_is_byte_deterministic() {
        let policy = policy();
        let findings = scan_surface(
            &policy,
            &[
                SourceFile {
                    path: "tools/z/src/lib.rs".to_owned(),
                    bytes: b"fn f(){std::fs::read(\"z\");}".to_vec(),
                },
                SourceFile {
                    path: "crates/a/src/lib.rs".to_owned(),
                    bytes: b"fn f(){std::fs::read(\"a\");}".to_vec(),
                },
            ],
        )
        .expect("scan must succeed");
        assert_eq!(findings[0].source_path, "crates/a/src/lib.rs");
    }
}
