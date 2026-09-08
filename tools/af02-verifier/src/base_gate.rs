use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use commandf_af02_verifier::canonical::{git_blob_sha1_hex, parse_json_no_duplicates};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::input_guard::{guard_inputs, CandidateFormat, CandidateInput};

pub const WORKFLOW_PATH: &str = ".github/workflows/af02-base-verifier.yml";
pub const RUNNER_PATH: &str = ".github/scripts/run_af02_base_verifier.sh";
pub const INVENTORY_PATH: &str =
    "specs/016-af-02-adversarial-test-strength/enforcement-inventory.json";
pub const INVENTORY_SCHEMA_PATH: &str =
    "specs/016-af-02-adversarial-test-strength/schemas/af02-enforcement-inventory-v1.schema.json";
pub const VERIFIER_SOURCE_PREFIX: &str = "tools/af02-verifier/src/";
pub const SCHEMA_PREFIX: &str = "specs/016-af-02-adversarial-test-strength/schemas/";
pub const VERIFIER_MANIFEST_PATH: &str = "tools/af02-verifier/Cargo.toml";
pub const VERIFIER_LOCK_PATH: &str = "tools/af02-verifier/Cargo.lock";

const GATE_INPUT_SCHEMA: &str = "commandf.af02-base-gate-input/v1";
const GATE_PROOF_SCHEMA: &str = "commandf.af02-base-gate-proof/v1";
const MAX_CHANGED_PATHS: usize = 3000;
const MAX_PATH_BYTES: usize = 4096;
const MAX_AUTHORITY_FILES: usize = 128;
const MAX_AUTHORITY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_SINGLE_AUTHORITY_BYTES: u64 = 1024 * 1024;

const AUTHORITY_EXACT: &[&str] = &[
    ".github/main-review-ruleset.json",
    ".github/main-ruleset.json",
    ".github/required-checks.json",
    ".github/workflow-trust-policy.json",
    "Cargo.lock",
    "Cargo.toml",
    "crates/commandf-pkg/src/oracle_model.rs",
];

const AUTHORITY_PREFIXES: &[&str] = &[
    ".github/scripts/",
    ".github/workflows/",
    "donors/",
    "specs/016-af-02-adversarial-test-strength/",
    "tools/af02-verifier/",
];

const ALLOWED_FILE_STATUSES: &[&str] = &[
    "added",
    "changed",
    "copied",
    "modified",
    "removed",
    "renamed",
    "unchanged",
];

#[derive(Debug, Error)]
pub enum BaseGateError {
    #[error("AF-02 base-gate contract violation: {0}")]
    Contract(String),
    #[error("AF-02 base-gate I/O failure: {0}")]
    Io(String),
    #[error("AF-02 base-gate Git failure: {0}")]
    Git(String),
    #[error("AF-02 base-gate JSON failure: {0}")]
    Json(String),
    #[error("AF-02 candidate input guard failure: {0}")]
    Input(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaseIdentityProof {
    pub schema: String,
    pub base_sha: String,
    pub base_tree: String,
    pub workflow_blob: String,
    pub runner_blob: String,
    pub cargo_manifest_blob: String,
    pub cargo_lock_blob: String,
    pub verifier_blobs: BTreeMap<String, String>,
    pub schema_blobs: BTreeMap<String, String>,
    pub enforcement_inventory_blob: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangedFile {
    pub status: String,
    pub filename: String,
    pub previous_filename: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateInput {
    pub schema: String,
    pub repository: String,
    pub pull_request: u64,
    pub base_sha: String,
    pub base_tree: String,
    pub head_sha: String,
    pub changed_files: Vec<ChangedFile>,
    pub base_identity: BaseIdentityProof,
    pub known_authority_paths: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BaseGateProof {
    pub schema: &'static str,
    pub repository: String,
    pub pull_request: u64,
    pub base_sha: String,
    pub base_tree: String,
    pub head_sha: String,
    pub mode: &'static str,
    pub changed_path_count: usize,
    pub authority_paths: Vec<String>,
    pub candidate_code_executed: bool,
    pub base_identity: BaseIdentityProof,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnforcementInventory {
    schema: String,
    policy_status: String,
    entries: Vec<EnforcementEntry>,
    closure_rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnforcementEntry {
    role: String,
    required_from_stack: String,
    implementation_kind: String,
    planned_path: String,
    entrypoint: String,
}

pub fn verify_pr(
    base_root: &Path,
    candidate_root: &Path,
    input_bytes: &[u8],
) -> Result<BaseGateProof, BaseGateError> {
    let value = parse_json_no_duplicates(input_bytes)
        .map_err(|error| BaseGateError::Json(format!("gate input: {error}")))?;
    let input: GateInput = serde_json::from_value(value)
        .map_err(|error| BaseGateError::Json(format!("gate input shape: {error}")))?;
    validate_gate_header(&input)?;
    verify_base_identity(base_root, &input.base_identity)?;

    let inventory = parse_inventory(base_root)?;
    verify_inventory_schema_freeze(base_root, &inventory)?;
    let known = validate_known_authority_paths(&input.known_authority_paths, &inventory)?;

    if input.changed_files.is_empty() {
        return contract("changed-file set must not be empty");
    }
    if input.changed_files.len() > MAX_CHANGED_PATHS {
        return contract("changed-file set exceeds bounded maximum");
    }

    let mut seen_current = BTreeSet::new();
    let mut authority_paths = BTreeSet::new();
    let mut current_authority = Vec::new();
    let mut existing_base_authority = Vec::new();

    for changed in &input.changed_files {
        if !ALLOWED_FILE_STATUSES.contains(&changed.status.as_str()) {
            return contract(format!("unsupported GitHub file status {}", changed.status));
        }
        let current = normalize_repo_path(&changed.filename)?;
        if !seen_current.insert(current.clone()) {
            return contract(format!("duplicate changed filename {current}"));
        }
        let previous = changed
            .previous_filename
            .as_deref()
            .map(normalize_repo_path)
            .transpose()?;

        let current_protected = is_authority_path(&current);
        let previous_protected = previous.as_deref().is_some_and(is_authority_path);
        if current_protected && !known.contains(&current) {
            return contract(format!("unknown AF-02 authority path {current}"));
        }
        if let Some(previous_path) = previous.as_deref()
            && previous_protected
            && !known.contains(previous_path)
        {
            return contract(format!("unknown prior AF-02 authority path {previous_path}"));
        }

        if changed.status == "renamed" && (current_protected || previous_protected) {
            return contract(format!(
                "AF-02 authority rename is fail-closed until a precanonical verifier strengthening: {} -> {}",
                previous.as_deref().unwrap_or("<missing>"),
                current
            ));
        }
        if changed.status == "removed" && current_protected {
            return contract(format!("AF-02 authority removal is fail-closed: {current}"));
        }

        if current_protected {
            authority_paths.insert(current.clone());
            current_authority.push(current.clone());
            if canonical_base_file_exists(base_root, &current)? {
                existing_base_authority.push(current);
            } else if !matches!(changed.status.as_str(), "added" | "copied") {
                return contract(format!(
                    "future AF-02 authority {current} must enter as an explicit added or copied path"
                ));
            }
        }
        if let Some(previous_path) = previous
            && previous_protected
        {
            authority_paths.insert(previous_path);
        }
    }

    if authority_paths.is_empty() {
        return Ok(BaseGateProof {
            schema: GATE_PROOF_SCHEMA,
            repository: input.repository,
            pull_request: input.pull_request,
            base_sha: input.base_sha,
            base_tree: input.base_tree,
            head_sha: input.head_sha,
            mode: "NOT_APPLICABLE",
            changed_path_count: input.changed_files.len(),
            authority_paths: Vec::new(),
            candidate_code_executed: false,
            base_identity: input.base_identity,
        });
    }

    validate_candidate_authority(candidate_root, &current_authority)?;

    if !existing_base_authority.is_empty() {
        existing_base_authority.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        return contract(format!(
            "canonical-base AF-02 authority is immutable under the A0 gate; dedicated precanonical strengthening is required: {}",
            existing_base_authority.join(",")
        ));
    }

    Ok(BaseGateProof {
        schema: GATE_PROOF_SCHEMA,
        repository: input.repository,
        pull_request: input.pull_request,
        base_sha: input.base_sha,
        base_tree: input.base_tree,
        head_sha: input.head_sha,
        mode: "FUTURE_AUTHORITY_ADDITION_VERIFIED",
        changed_path_count: input.changed_files.len(),
        authority_paths: authority_paths.into_iter().collect(),
        candidate_code_executed: false,
        base_identity: input.base_identity,
    })
}

pub fn prove_base_identity(base_root: &Path) -> Result<BaseIdentityProof, BaseGateError> {
    let base_sha = git_output(base_root, &["rev-parse", "HEAD"])?;
    let base_tree = git_output(base_root, &["rev-parse", "HEAD^{tree}"])?;
    validate_git_sha(&base_sha, "base commit")?;
    validate_git_sha(&base_tree, "base tree")?;

    Ok(BaseIdentityProof {
        schema: "commandf.af02-base-identity-proof/v1".to_owned(),
        base_sha,
        base_tree,
        workflow_blob: git_blob(base_root, WORKFLOW_PATH)?,
        runner_blob: git_blob(base_root, RUNNER_PATH)?,
        cargo_manifest_blob: git_blob(base_root, VERIFIER_MANIFEST_PATH)?,
        cargo_lock_blob: git_blob(base_root, VERIFIER_LOCK_PATH)?,
        verifier_blobs: git_tree_blobs(base_root, VERIFIER_SOURCE_PREFIX)?,
        schema_blobs: git_tree_blobs(base_root, SCHEMA_PREFIX)?,
        enforcement_inventory_blob: git_blob(base_root, INVENTORY_PATH)?,
    })
}

pub fn known_authority_paths(base_root: &Path) -> Result<Vec<String>, BaseGateError> {
    let mut known = AUTHORITY_EXACT
        .iter()
        .map(|path| (*path).to_owned())
        .collect::<BTreeSet<_>>();
    let tracked = git_output(
        base_root,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            "HEAD",
            "--",
            ".github/scripts",
            ".github/workflows",
            "donors",
            "specs/016-af-02-adversarial-test-strength",
            "tools/af02-verifier",
        ],
    )?;
    for line in tracked.lines().filter(|line| !line.is_empty()) {
        known.insert(normalize_repo_path(line)?);
    }
    for entry in parse_inventory(base_root)?.entries {
        if !entry.planned_path.ends_with('/') {
            known.insert(normalize_repo_path(&entry.planned_path)?);
        }
    }
    Ok(known.into_iter().collect())
}

pub fn test_gate_input(
    base_root: &Path,
    changed_files: Vec<ChangedFile>,
) -> Result<GateInput, BaseGateError> {
    let identity = prove_base_identity(base_root)?;
    Ok(GateInput {
        schema: GATE_INPUT_SCHEMA.to_owned(),
        repository: "TheHalfMoon/commandF".to_owned(),
        pull_request: 1,
        base_sha: identity.base_sha.clone(),
        base_tree: identity.base_tree.clone(),
        head_sha: "1111111111111111111111111111111111111111".to_owned(),
        changed_files,
        base_identity: identity,
        known_authority_paths: known_authority_paths(base_root)?,
    })
}

fn validate_gate_header(input: &GateInput) -> Result<(), BaseGateError> {
    if input.schema != GATE_INPUT_SCHEMA {
        return contract("unexpected gate-input schema");
    }
    if input.repository != "TheHalfMoon/commandF" || input.pull_request == 0 {
        return contract("unexpected repository or pull-request identity");
    }
    validate_git_sha(&input.base_sha, "gate base SHA")?;
    validate_git_sha(&input.base_tree, "gate base tree")?;
    validate_git_sha(&input.head_sha, "gate head SHA")?;
    if input.base_sha == input.head_sha {
        return contract("gate base and head SHAs must differ");
    }
    if input.base_identity.base_sha != input.base_sha
        || input.base_identity.base_tree != input.base_tree
    {
        return contract("gate header/base identity disagreement");
    }
    Ok(())
}

fn verify_base_identity(
    base_root: &Path,
    expected: &BaseIdentityProof,
) -> Result<(), BaseGateError> {
    if expected.schema != "commandf.af02-base-identity-proof/v1" {
        return contract("unexpected base-identity proof schema");
    }

    let observed_sha = git_output(base_root, &["rev-parse", "HEAD"])?;
    let observed_tree = git_output(base_root, &["rev-parse", "HEAD^{tree}"])?;
    validate_git_sha(&observed_sha, "observed base commit")?;
    validate_git_sha(&observed_tree, "observed base tree")?;
    if expected.base_sha != observed_sha || expected.base_tree != observed_tree {
        return contract(format!(
            "declared base identity disagrees with locally observed canonical base {observed_sha}/{observed_tree}"
        ));
    }

    compare_file_blob(base_root, WORKFLOW_PATH, &expected.workflow_blob)?;
    compare_file_blob(base_root, RUNNER_PATH, &expected.runner_blob)?;
    compare_file_blob(
        base_root,
        VERIFIER_MANIFEST_PATH,
        &expected.cargo_manifest_blob,
    )?;
    compare_file_blob(base_root, VERIFIER_LOCK_PATH, &expected.cargo_lock_blob)?;
    compare_file_blob(
        base_root,
        INVENTORY_PATH,
        &expected.enforcement_inventory_blob,
    )?;
    compare_tree_blob_map(
        base_root,
        VERIFIER_SOURCE_PREFIX,
        &expected.verifier_blobs,
        "verifier",
    )?;
    compare_tree_blob_map(
        base_root,
        SCHEMA_PREFIX,
        &expected.schema_blobs,
        "schema",
    )?;
    if !expected.schema_blobs.contains_key(INVENTORY_SCHEMA_PATH) {
        return contract("base identity omits enforcement-inventory schema blob");
    }
    Ok(())
}

fn compare_file_blob(
    root: &Path,
    path: &str,
    expected: &str,
) -> Result<(), BaseGateError> {
    validate_git_sha(expected, path)?;
    let full = root.join(path);
    let bytes = fs::read(&full).map_err(|error| io_error(&full, error))?;
    let observed = git_blob_sha1_hex(&bytes);
    if observed != expected {
        return contract(format!("canonical-base blob mismatch for {path}"));
    }
    Ok(())
}

fn compare_tree_blob_map(
    root: &Path,
    prefix: &str,
    expected: &BTreeMap<String, String>,
    label: &str,
) -> Result<(), BaseGateError> {
    if expected.is_empty() {
        return contract(format!("canonical-base {label} blob map is empty"));
    }
    let observed = filesystem_blob_map(root, prefix)?;
    if observed != *expected {
        return contract(format!("canonical-base {label} blob topology or identity mismatch"));
    }
    Ok(())
}

fn filesystem_blob_map(
    root: &Path,
    prefix: &str,
) -> Result<BTreeMap<String, String>, BaseGateError> {
    let start = root.join(prefix.trim_end_matches('/'));
    let mut files = Vec::new();
    collect_regular_files(root, &start, &mut files)?;
    let mut blobs = BTreeMap::new();
    for relative in files {
        let path = normalize_repo_path(&relative)?;
        let bytes = fs::read(root.join(&path)).map_err(|error| io_error(&root.join(&path), error))?;
        blobs.insert(path, git_blob_sha1_hex(&bytes));
    }
    Ok(blobs)
}

fn collect_regular_files(
    root: &Path,
    current: &Path,
    output: &mut Vec<String>,
) -> Result<(), BaseGateError> {
    let metadata = fs::symlink_metadata(current).map_err(|error| io_error(current, error))?;
    if metadata.file_type().is_symlink() {
        return contract(format!(
            "canonical-base identity path is a symlink: {}",
            current.display()
        ));
    }
    if metadata.is_file() {
        let relative = current
            .strip_prefix(root)
            .map_err(|_| BaseGateError::Contract("canonical-base identity path escaped root".to_owned()))?
            .to_string_lossy()
            .replace('\\', "/");
        output.push(relative);
        return Ok(());
    }
    if !metadata.is_dir() {
        return contract(format!(
            "canonical-base identity path has unsupported type: {}",
            current.display()
        ));
    }
    let mut entries = fs::read_dir(current)
        .map_err(|error| io_error(current, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| io_error(current, error))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        collect_regular_files(root, &entry.path(), output)?;
    }
    Ok(())
}

fn parse_inventory(base_root: &Path) -> Result<EnforcementInventory, BaseGateError> {
    let path = base_root.join(INVENTORY_PATH);
    let bytes = fs::read(&path).map_err(|error| io_error(&path, error))?;
    let value = parse_json_no_duplicates(&bytes)
        .map_err(|error| BaseGateError::Json(format!("canonical enforcement inventory: {error}")))?;
    let inventory: EnforcementInventory = serde_json::from_value(value)
        .map_err(|error| BaseGateError::Json(format!("canonical enforcement inventory shape: {error}")))?;
    if inventory.schema != "commandf.af02-enforcement-inventory/v1"
        || inventory.policy_status != "PLANNING_FREEZE"
        || inventory.closure_rule
            != "EXACT_ROLE_SET_NO_DUPLICATES_ACTIVE_PATHS_MUST_RESOLVE_AT_OR_AFTER_REQUIRED_STACK"
    {
        return contract("canonical enforcement inventory header is not frozen authority");
    }
    if inventory.entries.is_empty() {
        return contract("canonical enforcement inventory contains no roles");
    }
    let mut roles = BTreeSet::new();
    for entry in &inventory.entries {
        if entry.role.is_empty()
            || entry.required_from_stack.is_empty()
            || entry.implementation_kind.is_empty()
            || entry.planned_path.is_empty()
            || entry.entrypoint.is_empty()
        {
            return contract("canonical enforcement inventory contains an empty authority field");
        }
        if !roles.insert(entry.role.clone()) {
            return contract(format!("duplicate enforcement role {}", entry.role));
        }
    }
    Ok(inventory)
}

fn verify_inventory_schema_freeze(
    base_root: &Path,
    inventory: &EnforcementInventory,
) -> Result<(), BaseGateError> {
    let schema_path = base_root.join(INVENTORY_SCHEMA_PATH);
    let schema_bytes = fs::read(&schema_path).map_err(|error| io_error(&schema_path, error))?;
    let schema = parse_json_no_duplicates(&schema_bytes)
        .map_err(|error| BaseGateError::Json(format!("enforcement schema: {error}")))?;
    let frozen = schema
        .pointer("/properties/entries/const")
        .cloned()
        .ok_or_else(|| BaseGateError::Contract("enforcement schema omits entries.const".to_owned()))?;
    let frozen_entries: Vec<EnforcementEntry> = serde_json::from_value(frozen)
        .map_err(|error| BaseGateError::Json(format!("enforcement schema entries.const: {error}")))?;
    if frozen_entries != inventory.entries {
        return contract("enforcement inventory differs from schema-frozen role topology");
    }
    Ok(())
}

fn validate_known_authority_paths(
    paths: &[String],
    inventory: &EnforcementInventory,
) -> Result<BTreeSet<String>, BaseGateError> {
    if paths.is_empty() {
        return contract("known authority universe is empty");
    }
    let mut known = BTreeSet::new();
    let mut previous: Option<String> = None;
    for raw in paths {
        let path = normalize_repo_path(raw)?;
        if previous
            .as_deref()
            .is_some_and(|item| item.as_bytes() >= path.as_bytes())
        {
            return contract("known authority universe must be strictly UTF-8-byte sorted and unique");
        }
        previous = Some(path.clone());
        known.insert(path);
    }
    for exact in AUTHORITY_EXACT {
        if !known.contains(*exact) {
            return contract(format!("known authority universe omits {exact}"));
        }
    }
    for entry in &inventory.entries {
        if !entry.planned_path.ends_with('/') && !known.contains(&entry.planned_path) {
            return contract(format!(
                "known authority universe omits frozen implementation path {}",
                entry.planned_path
            ));
        }
    }
    Ok(known)
}

fn validate_candidate_authority(
    candidate_root: &Path,
    authority_paths: &[String],
) -> Result<(), BaseGateError> {
    if authority_paths.len() > MAX_AUTHORITY_FILES {
        return contract("candidate authority file count exceeds verifier-input policy");
    }
    let mut aggregate = 0u64;
    let mut structured = Vec::new();
    for path in authority_paths {
        let size = ensure_regular_contained(candidate_root, path)?;
        if size > MAX_SINGLE_AUTHORITY_BYTES {
            return contract(format!(
                "candidate authority {path} exceeds single-file byte policy"
            ));
        }
        aggregate = aggregate.checked_add(size).ok_or_else(|| {
            BaseGateError::Contract("candidate authority aggregate byte overflow".to_owned())
        })?;
        if aggregate > MAX_AUTHORITY_BYTES {
            return contract("candidate authority aggregate bytes exceed verifier-input policy");
        }
        if path.ends_with(".json") {
            structured.push(CandidateInput {
                format: CandidateFormat::Json,
                relative_path: PathBuf::from(path),
            });
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            structured.push(CandidateInput {
                format: CandidateFormat::Yaml,
                relative_path: PathBuf::from(path),
            });
        }
    }
    if !structured.is_empty() {
        guard_inputs(candidate_root, &structured)
            .map_err(|error| BaseGateError::Input(error.to_string()))?;
        for input in &structured {
            if input.format != CandidateFormat::Json {
                continue;
            }
            let path = candidate_root.join(&input.relative_path);
            let bytes = fs::read(&path).map_err(|error| io_error(&path, error))?;
            parse_json_no_duplicates(&bytes).map_err(|error| {
                BaseGateError::Input(format!(
                    "candidate JSON {} contains a duplicate key or is otherwise non-canonical: {error}",
                    input.relative_path.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn ensure_regular_contained(root: &Path, relative: &str) -> Result<u64, BaseGateError> {
    let mut current = root.to_path_buf();
    let parts = relative.split('/').collect::<Vec<_>>();
    for (index, part) in parts.iter().enumerate() {
        current.push(part);
        let metadata = fs::symlink_metadata(&current).map_err(|error| io_error(&current, error))?;
        if metadata.file_type().is_symlink() {
            return contract(format!(
                "candidate authority contains symlink component {relative}"
            ));
        }
        if index + 1 == parts.len() {
            if !metadata.is_file() {
                return contract(format!(
                    "candidate authority is not a regular file {relative}"
                ));
            }
            return Ok(metadata.len());
        }
        if !metadata.is_dir() {
            return contract(format!(
                "candidate authority parent is not a directory {relative}"
            ));
        }
    }
    contract("candidate authority path has no components")
}

fn canonical_base_file_exists(base_root: &Path, relative: &str) -> Result<bool, BaseGateError> {
    let full = base_root.join(relative);
    match fs::symlink_metadata(&full) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return contract(format!(
                    "canonical-base authority path is not a regular file: {relative}"
                ));
            }
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io_error(&full, error)),
    }
}

fn is_authority_path(path: &str) -> bool {
    AUTHORITY_EXACT.contains(&path)
        || AUTHORITY_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix))
}

fn normalize_repo_path(raw: &str) -> Result<String, BaseGateError> {
    if raw.is_empty()
        || raw.as_bytes().contains(&0)
        || raw.contains('\\')
        || raw.starts_with('/')
        || raw.len() > MAX_PATH_BYTES
    {
        return contract(format!(
            "changed path is not a bounded repository-relative POSIX path: {raw}"
        ));
    }
    let parts = raw.split('/').collect::<Vec<_>>();
    if parts
        .iter()
        .any(|part| part.is_empty() || *part == "." || *part == "..")
    {
        return contract(format!("changed path is not canonical POSIX form: {raw}"));
    }
    Ok(raw.to_owned())
}

fn git_blob(base_root: &Path, path: &str) -> Result<String, BaseGateError> {
    let spec = format!("HEAD:{path}");
    let sha = git_output(base_root, &["rev-parse", &spec])?;
    validate_git_sha(&sha, path)?;
    Ok(sha)
}

fn git_tree_blobs(
    base_root: &Path,
    prefix: &str,
) -> Result<BTreeMap<String, String>, BaseGateError> {
    let output = git_output(base_root, &["ls-tree", "-r", "HEAD", "--", prefix])?;
    let mut blobs = BTreeMap::new();
    for line in output.lines().filter(|line| !line.is_empty()) {
        let (meta, path) = line
            .split_once('\t')
            .ok_or_else(|| BaseGateError::Git("malformed git ls-tree output".to_owned()))?;
        let mut fields = meta.split_whitespace();
        let _mode = fields.next();
        let object_type = fields.next();
        let sha = fields.next();
        if object_type != Some("blob") || fields.next().is_some() {
            return contract(format!("authority {path} is not a Git blob"));
        }
        let sha = sha
            .ok_or_else(|| BaseGateError::Git("git ls-tree omitted blob SHA".to_owned()))?;
        validate_git_sha(sha, path)?;
        blobs.insert(normalize_repo_path(path)?, sha.to_owned());
    }
    if blobs.is_empty() {
        return contract(format!(
            "canonical base has no tracked blobs under {prefix}"
        ));
    }
    Ok(blobs)
}

fn git_output(base_root: &Path, args: &[&str]) -> Result<String, BaseGateError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(base_root)
        .args(args)
        .output()
        .map_err(|error| BaseGateError::Git(format!("cannot execute Git: {error}")))?;
    if !output.status.success() {
        return Err(BaseGateError::Git(format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim_end().to_owned())
        .map_err(|error| BaseGateError::Git(format!("Git output is not UTF-8: {error}")))
}

fn validate_git_sha(value: &str, label: &str) -> Result<(), BaseGateError> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return contract(format!(
            "{label} Git identity is not lowercase 40-hex"
        ));
    }
    Ok(())
}

fn io_error(path: &Path, error: std::io::Error) -> BaseGateError {
    BaseGateError::Io(format!("{}: {error}", path.display()))
}

fn contract<T>(message: impl Into<String>) -> Result<T, BaseGateError> {
    Err(BaseGateError::Contract(message.into()))
}
