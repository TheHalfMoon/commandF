use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use commandf_af02_verifier::canonical::parse_json_no_duplicates;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const WORKFLOW_PATH: &str = ".github/workflows/af02-base-verifier.yml";
pub const RUNNER_PATH: &str = ".github/scripts/run_af02_base_verifier.sh";
pub const MAIN_VERIFIER_PATH: &str = "tools/af02-verifier/src/main.rs";
pub const BASE_GATE_PATH: &str = "tools/af02-verifier/src/base_gate.rs";
pub const INVENTORY_PATH: &str =
    "specs/016-af-02-adversarial-test-strength/enforcement-inventory.json";
pub const SCHEMA_PREFIX: &str = "specs/016-af-02-adversarial-test-strength/schemas/";
const MAX_AUTHORITY_BYTES: u64 = 4 * 1024 * 1024;
const MAX_CHANGED_PATHS: usize = 3000;

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
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BaseIdentityProof {
    pub schema: &'static str,
    pub base_sha: String,
    pub workflow_blob: String,
    pub verifier_blobs: BTreeMap<String, String>,
    pub schema_blobs: BTreeMap<String, String>,
    pub enforcement_inventory_blob: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BaseGateProof {
    pub schema: &'static str,
    pub mode: &'static str,
    pub authority_paths: Vec<String>,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnforcementEntry {
    role: String,
    required_from_stack: String,
    implementation_kind: String,
    planned_path: String,
    entrypoint: String,
}

pub fn prove_base_identity(base_root: &Path) -> Result<BaseIdentityProof, BaseGateError> {
    let base_sha = git_output(base_root, &["rev-parse", "HEAD"])?;
    validate_git_sha(&base_sha, "base commit")?;

    let workflow_blob = git_blob(base_root, WORKFLOW_PATH)?;
    let enforcement_inventory_blob = git_blob(base_root, INVENTORY_PATH)?;

    let mut verifier_blobs = BTreeMap::new();
    for path in [RUNNER_PATH, MAIN_VERIFIER_PATH, BASE_GATE_PATH] {
        verifier_blobs.insert(path.to_owned(), git_blob(base_root, path)?);
    }

    let schema_blobs = git_tree_blobs(base_root, SCHEMA_PREFIX)?;
    if schema_blobs.is_empty() {
        return contract("canonical base exposes no AF-02 schema blobs");
    }

    parse_inventory(base_root)?;

    Ok(BaseIdentityProof {
        schema: "commandf.af02-base-identity-proof/v1",
        base_sha,
        workflow_blob,
        verifier_blobs,
        schema_blobs,
        enforcement_inventory_blob,
    })
}

pub fn verify_changed_authority(
    base_root: &Path,
    candidate_root: &Path,
    changed_paths: &[String],
) -> Result<BaseGateProof, BaseGateError> {
    if changed_paths.is_empty() {
        return contract("changed-path set must not be empty");
    }
    if changed_paths.len() > MAX_CHANGED_PATHS {
        return contract("changed-path set exceeds bounded maximum");
    }

    let identity = prove_base_identity(base_root)?;
    let known = known_authority_paths(base_root)?;
    let mut normalized = BTreeSet::new();
    for raw in changed_paths {
        normalized.insert(normalize_repo_path(raw)?);
    }

    let authority = normalized
        .iter()
        .filter(|path| is_authority_path(path))
        .cloned()
        .collect::<Vec<_>>();

    if authority.is_empty() {
        return Ok(BaseGateProof {
            schema: "commandf.af02-base-gate-proof/v1",
            mode: "NOT_APPLICABLE",
            authority_paths: authority,
            base_identity: identity,
        });
    }

    for path in &authority {
        if !known.contains(path) {
            return contract(format!("unknown AF-02 authority path {path}"));
        }
        verify_structured_candidate(candidate_root, path)?;
    }

    Ok(BaseGateProof {
        schema: "commandf.af02-base-gate-proof/v1",
        mode: "AUTHORITY_VERIFICATION_REQUIRED",
        authority_paths: authority,
        base_identity: identity,
    })
}

fn parse_inventory(base_root: &Path) -> Result<EnforcementInventory, BaseGateError> {
    let path = base_root.join(INVENTORY_PATH);
    let bytes = fs::read(&path).map_err(|error| io_error(&path, error))?;
    let value = parse_json_no_duplicates(&bytes)
        .map_err(|error| BaseGateError::Json(format!("canonical enforcement inventory: {error}")))?;
    let inventory: EnforcementInventory = serde_json::from_value(value).map_err(|error| {
        BaseGateError::Json(format!("canonical enforcement inventory shape: {error}"))
    })?;
    if inventory.schema != "commandf.af02-enforcement-inventory/v1"
        || inventory.policy_status != "PLANNING_FREEZE"
        || inventory.closure_rule
            != "EXACT_ROLE_SET_NO_DUPLICATES_ACTIVE_PATHS_MUST_RESOLVE_AT_OR_AFTER_REQUIRED_STACK"
    {
        return contract("canonical enforcement inventory header is not frozen A0 authority");
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

fn known_authority_paths(base_root: &Path) -> Result<BTreeSet<String>, BaseGateError> {
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

    let inventory = parse_inventory(base_root)?;
    for entry in inventory.entries {
        if !entry.planned_path.ends_with('/') {
            known.insert(normalize_repo_path(&entry.planned_path)?);
        }
    }
    Ok(known)
}

fn verify_structured_candidate(candidate_root: &Path, path: &str) -> Result<(), BaseGateError> {
    if !(path.ends_with(".json") || path.ends_with(".yaml") || path.ends_with(".yml")) {
        return Ok(());
    }
    let candidate = candidate_root.join(path);
    let metadata = match fs::symlink_metadata(&candidate) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(io_error(&candidate, error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return contract(format!(
            "structured candidate authority {path} is not a regular file"
        ));
    }
    if metadata.len() > MAX_AUTHORITY_BYTES {
        return contract(format!(
            "structured candidate authority {path} exceeds bounded byte policy"
        ));
    }
    let bytes = fs::read(&candidate).map_err(|error| io_error(&candidate, error))?;
    if path.ends_with(".json") {
        parse_json_no_duplicates(&bytes).map_err(|error| {
            BaseGateError::Json(format!("candidate authority {path} is unparseable: {error}"))
        })?;
    } else {
        std::str::from_utf8(&bytes).map_err(|error| {
            BaseGateError::Contract(format!(
                "candidate YAML authority {path} is not UTF-8 and is fail-closed: {error}"
            ))
        })?;
        return contract(format!(
            "candidate YAML authority {path} requires the hardened YAML parser before acceptance"
        ));
    }
    Ok(())
}

fn is_authority_path(path: &str) -> bool {
    AUTHORITY_EXACT.contains(&path)
        || AUTHORITY_PREFIXES
            .iter()
            .any(|prefix| path.starts_with(prefix))
}

fn normalize_repo_path(raw: &str) -> Result<String, BaseGateError> {
    if raw.is_empty()
        || raw.contains('\0')
        || raw.contains('\\')
        || raw.len() > 4096
        || raw.starts_with('/')
        || raw.split('/').any(|part| part.is_empty() || part == "." || part == "..")
    {
        return contract(format!(
            "changed path is not canonical repository-relative POSIX form: {raw}"
        ));
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
            return contract(format!("schema authority {path} is not a Git blob"));
        }
        let sha =
            sha.ok_or_else(|| BaseGateError::Git("git ls-tree omitted blob SHA".to_owned()))?;
        validate_git_sha(sha, path)?;
        blobs.insert(normalize_repo_path(path)?, sha.to_owned());
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
            "git {args:?} failed: {}",
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
