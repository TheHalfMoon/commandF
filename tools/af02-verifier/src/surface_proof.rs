use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::canonical::{
    canonical_json_bytes, canonical_sha256, git_blob_sha1_hex, parse_json_no_duplicates,
    sha256_hex, CanonicalError,
};
use crate::surface::{
    discover_tracked_rust_sources, parse_surface_policy, scan_surface, Finding, FindingCertainty,
    SourceFile, SurfaceError, SurfacePolicy,
};

const SURFACE_PROOF_SCHEMA: &str = "commandf.af02-surface-proof/v1";
const EXCLUSION_POLICY_SCHEMA: &str = "commandf.af02-exclusion-policy/v1";
const SCANNER_SOURCE_PATH: &str = "tools/af02-verifier/src/surface.rs";
const SCANNER_LOCK_PATH: &str = "tools/af02-verifier/Cargo.lock";

#[derive(Debug, Error)]
pub enum SurfaceProofError {
    #[error(transparent)]
    Surface(#[from] SurfaceError),
    #[error(transparent)]
    Canonical(#[from] CanonicalError),
    #[error("surface proof I/O error for {path}: {source}")]
    Io {
        path: String,
        source: std::io::Error,
    },
    #[error("surface proof git error: {0}")]
    Git(String),
    #[error("surface proof violation: {0}")]
    Violation(String),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SourceUniverseEntry {
    pub path: String,
    pub git_blob_sha: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ClassifiedFinding {
    pub finding_id: String,
    pub source_path: String,
    pub syntax_ordinal: u64,
    pub matcher_id: String,
    pub category: String,
    pub certainty: FindingCertainty,
    pub disposition: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SurfaceProofEvidence {
    pub schema: &'static str,
    pub source_sha: String,
    pub source_tree: String,
    pub policy_sha256: String,
    pub exclusion_policy_sha256: String,
    pub scanner_source_sha256: String,
    pub scanner_dependency_lock_sha256: String,
    pub source_universe_sha256: String,
    pub raw_findings_sha256: String,
    pub discovery_inventory_sha256: String,
    pub classified_boundary_count: u64,
    pub reviewed_exclusion_count: u64,
    pub stale_entry_count: u64,
    pub unclassified_boundary_count: u64,
    pub source_universe: Vec<SourceUniverseEntry>,
    pub raw_findings: Vec<Finding>,
    pub discovery_inventory: Vec<ClassifiedFinding>,
}

pub fn prove_surface(
    policy_bytes: &[u8],
    exclusion_policy_bytes: &[u8],
    source_repo_root: &Path,
) -> Result<SurfaceProofEvidence, SurfaceProofError> {
    let policy = parse_surface_policy(policy_bytes)?;
    let exclusions = parse_production_source_exclusions(exclusion_policy_bytes)?;
    let exclusion_digest = sha256_hex(exclusion_policy_bytes);
    if exclusion_digest != policy.exclusion_policy_sha256 {
        return violation(format!(
            "exclusion-policy digest mismatch: policy={} actual={exclusion_digest}",
            policy.exclusion_policy_sha256
        ));
    }

    let source_sha = git_text(source_repo_root, &["rev-parse", "HEAD"])?;
    let source_tree = git_text(source_repo_root, &["rev-parse", "HEAD^{tree}"])?;
    if source_sha != policy.source_sha {
        return violation(format!(
            "source HEAD {source_sha} does not equal policy source_sha {}",
            policy.source_sha
        ));
    }
    if source_tree != policy.source_tree {
        return violation(format!(
            "source tree {source_tree} does not equal policy source_tree {}",
            policy.source_tree
        ));
    }

    let all_sources = discover_tracked_rust_sources(source_repo_root)?;
    let all_paths = all_sources
        .iter()
        .map(|source| source.path.as_str())
        .collect::<BTreeSet<_>>();
    for excluded in &exclusions {
        if !all_paths.contains(excluded.as_str()) {
            return violation(format!(
                "stale production source exclusion {excluded} is not in the Git-derived Rust universe"
            ));
        }
    }
    let sources = all_sources
        .into_iter()
        .filter(|source| !exclusions.contains(&source.path))
        .collect::<Vec<_>>();

    let source_universe = derive_source_universe(source_repo_root, &sources)?;
    let source_universe_value = serde_json::to_value(&source_universe)
        .map_err(|error| SurfaceProofError::Violation(error.to_string()))?;
    let source_universe_sha256 = canonical_sha256(&source_universe_value)?;

    let raw_findings = scan_surface(&policy, &sources)?;
    let raw_value = serde_json::to_value(&raw_findings)
        .map_err(|error| SurfaceProofError::Violation(error.to_string()))?;
    let raw_findings_sha256 = canonical_sha256(&raw_value)?;

    let source_blob_by_path = source_universe
        .iter()
        .map(|entry| (entry.path.as_str(), entry.git_blob_sha.as_str()))
        .collect::<BTreeMap<_, _>>();
    let classification = classify_findings(&policy, &raw_findings, &source_blob_by_path)?;
    let discovery_value = serde_json::to_value(&classification.findings)
        .map_err(|error| SurfaceProofError::Violation(error.to_string()))?;
    let discovery_inventory_sha256 = canonical_sha256(&discovery_value)?;

    let scanner_source = read_exact_source(source_repo_root, SCANNER_SOURCE_PATH)?;
    let scanner_lock = read_exact_source(source_repo_root, SCANNER_LOCK_PATH)?;

    Ok(SurfaceProofEvidence {
        schema: SURFACE_PROOF_SCHEMA,
        source_sha,
        source_tree,
        policy_sha256: sha256_hex(policy_bytes),
        exclusion_policy_sha256: exclusion_digest,
        scanner_source_sha256: sha256_hex(&scanner_source),
        scanner_dependency_lock_sha256: sha256_hex(&scanner_lock),
        source_universe_sha256,
        raw_findings_sha256,
        discovery_inventory_sha256,
        classified_boundary_count: classification.findings.len() as u64,
        reviewed_exclusion_count: classification.reviewed_exclusion_count,
        stale_entry_count: 0,
        unclassified_boundary_count: 0,
        source_universe,
        raw_findings,
        discovery_inventory: classification.findings,
    })
}

pub fn canonical_surface_proof_bytes(
    evidence: &SurfaceProofEvidence,
) -> Result<Vec<u8>, SurfaceProofError> {
    let value = serde_json::to_value(evidence)
        .map_err(|error| SurfaceProofError::Violation(error.to_string()))?;
    Ok(canonical_json_bytes(&value)?)
}

#[derive(Debug)]
struct ClassificationResult {
    findings: Vec<ClassifiedFinding>,
    reviewed_exclusion_count: u64,
}

fn classify_findings(
    policy: &SurfacePolicy,
    findings: &[Finding],
    source_blob_by_path: &BTreeMap<&str, &str>,
) -> Result<ClassificationResult, SurfaceProofError> {
    let mut classified = Vec::with_capacity(findings.len());
    let mut used_surface_paths = BTreeSet::<(String, String)>::new();
    let mut used_surface_matchers = BTreeSet::<(String, String)>::new();
    let mut used_exclusions = BTreeSet::<String>::new();
    let mut reviewed_exclusion_count = 0_u64;

    for finding in findings {
        let mut dispositions = Vec::new();
        for surface in &policy.critical_surfaces {
            if surface.category == finding.category
                && surface
                    .matcher_ids
                    .iter()
                    .any(|id| id == &finding.matcher_id)
                && surface
                    .source_paths
                    .iter()
                    .any(|path| path == &finding.source_path)
            {
                dispositions.push(format!("CRITICAL_SURFACE:{}", surface.surface_id));
                used_surface_paths
                    .insert((surface.surface_id.clone(), finding.source_path.clone()));
                used_surface_matchers
                    .insert((surface.surface_id.clone(), finding.matcher_id.clone()));
            }
        }
        for exclusion in &policy.finding_exclusions {
            if exclusion.matcher_id == finding.matcher_id
                && exclusion.source_path == finding.source_path
            {
                dispositions.push(format!("REVIEWED_EXCLUSION:{}", exclusion.exclusion_id));
                used_exclusions.insert(exclusion.exclusion_id.clone());
            }
        }
        if dispositions.len() != 1 {
            return violation(format!(
                "finding {} has {} dispositions: {:?}",
                finding_identity(finding),
                dispositions.len(),
                dispositions
            ));
        }
        if dispositions[0].starts_with("REVIEWED_EXCLUSION:") {
            reviewed_exclusion_count = reviewed_exclusion_count.saturating_add(1);
        }
        classified.push(ClassifiedFinding {
            finding_id: finding_identity(finding),
            source_path: finding.source_path.clone(),
            syntax_ordinal: finding.syntax_ordinal,
            matcher_id: finding.matcher_id.clone(),
            category: finding.category.clone(),
            certainty: finding.certainty,
            disposition: dispositions.remove(0),
        });
    }

    for surface in &policy.critical_surfaces {
        for path in &surface.source_paths {
            if !used_surface_paths.contains(&(surface.surface_id.clone(), path.clone())) {
                return violation(format!(
                    "stale critical-surface source membership {} -> {}",
                    surface.surface_id, path
                ));
            }
        }
        for matcher_id in &surface.matcher_ids {
            if !used_surface_matchers.contains(&(surface.surface_id.clone(), matcher_id.clone())) {
                return violation(format!(
                    "stale critical-surface matcher membership {} -> {}",
                    surface.surface_id, matcher_id
                ));
            }
        }
    }
    for exclusion in &policy.finding_exclusions {
        if !used_exclusions.contains(&exclusion.exclusion_id) {
            return violation(format!(
                "stale reviewed finding exclusion {}",
                exclusion.exclusion_id
            ));
        }
    }

    let finding_keys = findings
        .iter()
        .map(|finding| {
            (
                finding.source_path.as_str(),
                finding.matcher_id.as_str(),
                finding.category.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    for witness in &policy.known_boundary_witnesses {
        let actual_blob = source_blob_by_path
            .get(witness.source_path.as_str())
            .ok_or_else(|| {
                SurfaceProofError::Violation(format!(
                    "known boundary witness {} references a source outside the exact universe",
                    witness.witness_id
                ))
            })?;
        if **actual_blob != witness.source_blob_sha {
            return violation(format!(
                "known boundary witness {} blob mismatch: policy={} actual={}",
                witness.witness_id, witness.source_blob_sha, actual_blob
            ));
        }
        if !finding_keys.contains(&(
            witness.source_path.as_str(),
            witness.matcher_id.as_str(),
            witness.category.as_str(),
        )) {
            return violation(format!(
                "stale known boundary witness {} has no scanner finding",
                witness.witness_id
            ));
        }
    }

    let mut identities = BTreeSet::new();
    for finding in &classified {
        if !identities.insert(finding.finding_id.as_str()) {
            return violation(format!(
                "duplicate classified finding identity {}",
                finding.finding_id
            ));
        }
    }

    Ok(ClassificationResult {
        findings: classified,
        reviewed_exclusion_count,
    })
}

fn derive_source_universe(
    repo_root: &Path,
    sources: &[SourceFile],
) -> Result<Vec<SourceUniverseEntry>, SurfaceProofError> {
    let mut entries = Vec::with_capacity(sources.len());
    for source in sources {
        let expected_blob = git_text(repo_root, &["rev-parse", &format!("HEAD:{}", source.path)])?;
        let actual_blob = git_blob_sha1_hex(&source.bytes);
        if actual_blob != expected_blob {
            return violation(format!(
                "worktree bytes for {} do not equal HEAD blob: expected={} actual={actual_blob}",
                source.path, expected_blob
            ));
        }
        entries.push(SourceUniverseEntry {
            path: source.path.clone(),
            git_blob_sha: expected_blob,
            sha256: sha256_hex(&source.bytes),
            bytes: source.bytes.len() as u64,
        });
    }
    entries.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));
    Ok(entries)
}

fn parse_production_source_exclusions(bytes: &[u8]) -> Result<BTreeSet<String>, SurfaceProofError> {
    let value = parse_json_no_duplicates(bytes)
        .map_err(|error| SurfaceProofError::Violation(error.to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| SurfaceProofError::Violation("exclusion policy must be an object".into()))?;
    if object.get("schema").and_then(Value::as_str) != Some(EXCLUSION_POLICY_SCHEMA) {
        return violation("unexpected exclusion policy schema");
    }
    let raw = object
        .get("production_source_exclusions")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            SurfaceProofError::Violation(
                "exclusion policy production_source_exclusions must be an array".into(),
            )
        })?;
    let mut exclusions = BTreeSet::new();
    for value in raw {
        let path = value.as_str().ok_or_else(|| {
            SurfaceProofError::Violation(
                "production source exclusion entries must be strings".into(),
            )
        })?;
        validate_repo_path(path)?;
        if !exclusions.insert(path.to_owned()) {
            return violation(format!("duplicate production source exclusion {path}"));
        }
    }
    Ok(exclusions)
}

fn validate_repo_path(path: &str) -> Result<(), SurfaceProofError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains("//")
        || path.contains('\0')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return violation(format!("invalid repository-relative path {path}"));
    }
    Ok(())
}

fn read_exact_source(repo_root: &Path, path: &str) -> Result<Vec<u8>, SurfaceProofError> {
    let bytes = fs::read(repo_root.join(path)).map_err(|source| SurfaceProofError::Io {
        path: path.to_owned(),
        source,
    })?;
    let expected_blob = git_text(repo_root, &["rev-parse", &format!("HEAD:{path}")])?;
    let actual_blob = git_blob_sha1_hex(&bytes);
    if expected_blob != actual_blob {
        return violation(format!(
            "worktree bytes for {path} do not equal HEAD blob: expected={expected_blob} actual={actual_blob}"
        ));
    }
    Ok(bytes)
}

fn git_text(repo_root: &Path, args: &[&str]) -> Result<String, SurfaceProofError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .map_err(|source| SurfaceProofError::Io {
            path: repo_root.display().to_string(),
            source,
        })?;
    if !output.status.success() {
        return Err(SurfaceProofError::Git(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn finding_identity(finding: &Finding) -> String {
    format!(
        "{}:{}:{}",
        finding.source_path, finding.syntax_ordinal, finding.matcher_id
    )
}

fn violation<T>(message: impl Into<String>) -> Result<T, SurfaceProofError> {
    Err(SurfaceProofError::Violation(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{CriticalSurface, FindingExclusion};

    const CANONICAL_POLICY: &[u8] =
        include_bytes!("../../../specs/016-af-02-adversarial-test-strength/surface-policy.json");

    fn policy() -> SurfacePolicy {
        parse_surface_policy(CANONICAL_POLICY).expect("canonical surface policy must parse")
    }

    fn finding(path: &str, matcher: &str, category: &str) -> Finding {
        Finding {
            source_path: path.to_owned(),
            syntax_ordinal: 7,
            matcher_id: matcher.to_owned(),
            category: category.to_owned(),
            certainty: FindingCertainty::Definite,
        }
    }

    fn blob_map<'a>(path: &'a str, blob: &'a str) -> BTreeMap<&'a str, &'a str> {
        BTreeMap::from([(path, blob)])
    }

    #[test]
    fn exact_finding_gets_one_critical_surface_disposition() {
        let mut policy = policy();
        let witness = policy.known_boundary_witnesses[0].clone();
        policy.critical_surfaces = vec![CriticalSurface {
            surface_id: "archive-gzip-decode".to_owned(),
            category: witness.category.clone(),
            matcher_ids: vec![witness.matcher_id.clone()],
            source_paths: vec![witness.source_path.clone()],
        }];
        policy.known_boundary_witnesses = vec![witness.clone()];
        policy.finding_exclusions.clear();
        let findings = vec![finding(
            &witness.source_path,
            &witness.matcher_id,
            &witness.category,
        )];
        let blobs = blob_map(&witness.source_path, &witness.source_blob_sha);
        let result =
            classify_findings(&policy, &findings, &blobs).expect("classification must pass");
        assert_eq!(result.findings.len(), 1);
        assert_eq!(
            result.findings[0].disposition,
            "CRITICAL_SURFACE:archive-gzip-decode"
        );
    }

    #[test]
    fn unclassified_finding_fails_closed() {
        let mut policy = policy();
        policy.critical_surfaces.clear();
        policy.known_boundary_witnesses.clear();
        policy.finding_exclusions.clear();
        let findings = vec![finding(
            "crates/example/src/lib.rs",
            "filesystem-read",
            "FILESYSTEM",
        )];
        let error = classify_findings(&policy, &findings, &BTreeMap::new()).unwrap_err();
        assert!(error.to_string().contains("has 0 dispositions"));
    }

    #[test]
    fn multiply_classified_finding_fails_closed() {
        let mut policy = policy();
        policy.known_boundary_witnesses.clear();
        policy.finding_exclusions.clear();
        policy.critical_surfaces = vec![
            CriticalSurface {
                surface_id: "one".to_owned(),
                category: "FILESYSTEM".to_owned(),
                matcher_ids: vec!["filesystem-read".to_owned()],
                source_paths: vec!["crates/example/src/lib.rs".to_owned()],
            },
            CriticalSurface {
                surface_id: "two".to_owned(),
                category: "FILESYSTEM".to_owned(),
                matcher_ids: vec!["filesystem-read".to_owned()],
                source_paths: vec!["crates/example/src/lib.rs".to_owned()],
            },
        ];
        let findings = vec![finding(
            "crates/example/src/lib.rs",
            "filesystem-read",
            "FILESYSTEM",
        )];
        let error = classify_findings(&policy, &findings, &BTreeMap::new()).unwrap_err();
        assert!(error.to_string().contains("has 2 dispositions"));
    }

    #[test]
    fn stale_surface_path_membership_fails_closed() {
        let mut policy = policy();
        policy.known_boundary_witnesses.clear();
        policy.finding_exclusions.clear();
        policy.critical_surfaces = vec![CriticalSurface {
            surface_id: "filesystem-read".to_owned(),
            category: "FILESYSTEM".to_owned(),
            matcher_ids: vec!["filesystem-read".to_owned()],
            source_paths: vec![
                "crates/example/src/lib.rs".to_owned(),
                "crates/example/src/stale.rs".to_owned(),
            ],
        }];
        let findings = vec![finding(
            "crates/example/src/lib.rs",
            "filesystem-read",
            "FILESYSTEM",
        )];
        let error = classify_findings(&policy, &findings, &BTreeMap::new()).unwrap_err();
        assert!(error
            .to_string()
            .contains("stale critical-surface source membership"));
    }

    #[test]
    fn stale_reviewed_exclusion_fails_closed() {
        let mut policy = policy();
        policy.critical_surfaces.clear();
        policy.known_boundary_witnesses.clear();
        policy.finding_exclusions = vec![FindingExclusion {
            exclusion_id: "SURF-X0001".to_owned(),
            matcher_id: "filesystem-read".to_owned(),
            source_path: "crates/example/src/lib.rs".to_owned(),
            reason: "Reviewed exclusion exists only for this negative stale-entry test.".to_owned(),
            introduced_policy_sha: "0".repeat(40),
        }];
        let error = classify_findings(&policy, &[], &BTreeMap::new()).unwrap_err();
        assert!(error
            .to_string()
            .contains("stale reviewed finding exclusion"));
    }

    #[test]
    fn uncertain_finding_still_requires_and_accepts_exact_disposition() {
        let mut policy = policy();
        policy.known_boundary_witnesses.clear();
        policy.finding_exclusions.clear();
        policy.critical_surfaces = vec![CriticalSurface {
            surface_id: "filesystem-read".to_owned(),
            category: "FILESYSTEM".to_owned(),
            matcher_ids: vec!["filesystem-read".to_owned()],
            source_paths: vec!["crates/example/src/lib.rs".to_owned()],
        }];
        let mut uncertain = finding("crates/example/src/lib.rs", "filesystem-read", "FILESYSTEM");
        uncertain.certainty = FindingCertainty::Uncertain;
        let result = classify_findings(&policy, &[uncertain], &BTreeMap::new())
            .expect("uncertain finding with exact disposition must remain classified evidence");
        assert_eq!(result.findings.len(), 1);
    }
}
