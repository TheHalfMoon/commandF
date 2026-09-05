use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::canonical::{git_blob_sha1_hex, parse_json_no_duplicates, sha256_hex};
use crate::surface::parse_surface_policy;

const CORPUS_SCHEMA_ID: &str = "commandf.af02-corpus/v1";
const CORPUS_SCHEMA_URL: &str = "https://commandf.dev/schemas/af02-corpus-v1.schema.json";
const CORPUS_SCHEMA_GIT_BLOB_SHA: &str = "7ef4591d96adaa507014e4ad2f137cba6462fde2";
const ASSERTION_SCHEMA_ID: &str = "commandf.af02-assertion-registry/v1";
const MAX_FIXTURE_BYTES: u64 = 262_144;
const MAX_TOTAL_BYTES: u64 = 8_388_608;

#[derive(Debug, Error)]
pub enum CorpusError {
    #[error("corpus JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("corpus schema violation: {0}")]
    Schema(String),
    #[error("corpus contract violation: {0}")]
    Contract(String),
    #[error("surface policy validation failed: {0}")]
    Surface(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusManifest {
    pub schema: String,
    pub source_sha: String,
    pub max_fixture_bytes: u64,
    pub max_total_bytes: u64,
    pub entries: Vec<CorpusEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusEntry {
    pub scenario_id: String,
    pub fixture_path: String,
    pub fixture_sha256: String,
    pub byte_length: u64,
    pub provenance_class: ProvenanceClass,
    pub expected_outcome: ExpectedOutcome,
    pub assertion_id: String,
    pub replay_id: String,
    pub discovery_origin: DiscoveryOrigin,
    pub parent_scenario_id_or_null: Option<String>,
    pub minimization_tool_or_null: Option<String>,
    pub contains_phi: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProvenanceClass {
    Synthetic,
    PublicRedistributable,
    GeneratedFromSynthetic,
    OpaqueFuzzArtifactSafe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExpectedOutcome {
    AcceptCanonical,
    RejectInvalid,
    FailClosedLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiscoveryOrigin {
    HandAuthored,
    FuzzDiscovery,
    PropertyCounterexample,
    MutationRegression,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssertionRegistry {
    pub schema: String,
    pub source_sha: String,
    pub corpus_manifest_sha256: String,
    pub surface_policy_sha256: String,
    pub entries: Vec<AssertionEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssertionEntry {
    pub assertion_id: String,
    pub scenario_id: String,
    pub surface_id: String,
    pub runner_kind: RunnerKind,
    pub manifest_path: String,
    pub package_or_binary: String,
    pub cargo_target_or_null: Option<String>,
    pub test_name_or_null: Option<String>,
    pub argv: Vec<String>,
    pub cwd_repo_relative: String,
    pub environment_allowlist: BTreeMap<String, String>,
    pub expected_outcome: ExpectedOutcome,
    pub result_parser_id: String,
    pub source_paths: Vec<String>,
    pub config_sha256s: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunnerKind {
    CargoTest,
    Af02ReplayBinary,
}

pub fn parse_corpus_manifest(
    instance_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<CorpusManifest, CorpusError> {
    if git_blob_sha1_hex(schema_bytes) != CORPUS_SCHEMA_GIT_BLOB_SHA {
        return Err(CorpusError::Schema(
            "corpus schema bytes do not match the planning-frozen Git blob".to_owned(),
        ));
    }
    let schema = parse_json_no_duplicates(schema_bytes)?;
    if schema.get("$id").and_then(Value::as_str) != Some(CORPUS_SCHEMA_URL) {
        return Err(CorpusError::Schema(
            "unexpected planning-frozen corpus schema id".to_owned(),
        ));
    }
    let value = parse_json_no_duplicates(instance_bytes)?;
    let manifest: CorpusManifest = serde_json::from_value(value)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn parse_assertion_registry(bytes: &[u8]) -> Result<AssertionRegistry, CorpusError> {
    let value = parse_json_no_duplicates(bytes)?;
    let registry: AssertionRegistry = serde_json::from_value(value)?;
    validate_registry(&registry)?;
    Ok(registry)
}

pub fn verify_fixture_bytes(entry: &CorpusEntry, bytes: &[u8]) -> Result<(), CorpusError> {
    let actual_bytes = u64::try_from(bytes.len()).map_err(|_| {
        CorpusError::Contract(format!(
            "fixture {} length cannot be represented as u64",
            entry.fixture_path
        ))
    })?;
    if actual_bytes > MAX_FIXTURE_BYTES {
        return Err(CorpusError::Contract(format!(
            "fixture {} exceeds the per-fixture byte limit",
            entry.fixture_path
        )));
    }
    if actual_bytes != entry.byte_length {
        return Err(CorpusError::Contract(format!(
            "fixture {} byte length mismatch: declared {}, observed {}",
            entry.fixture_path, entry.byte_length, actual_bytes
        )));
    }
    let actual_sha256 = sha256_hex(bytes);
    if actual_sha256 != entry.fixture_sha256 {
        return Err(CorpusError::Contract(format!(
            "fixture {} SHA-256 mismatch: declared {}, observed {}",
            entry.fixture_path, entry.fixture_sha256, actual_sha256
        )));
    }
    Ok(())
}

pub fn validate_corpus_and_assertions(
    corpus_bytes: &[u8],
    corpus_schema_bytes: &[u8],
    assertion_registry_bytes: &[u8],
    surface_policy_bytes: &[u8],
) -> Result<(CorpusManifest, AssertionRegistry), CorpusError> {
    let manifest = parse_corpus_manifest(corpus_bytes, corpus_schema_bytes)?;
    let registry = parse_assertion_registry(assertion_registry_bytes)?;
    let surface_policy = parse_surface_policy(surface_policy_bytes)
        .map_err(|error| CorpusError::Surface(error.to_string()))?;

    if registry.source_sha != manifest.source_sha {
        return Err(CorpusError::Contract(
            "assertion registry source_sha does not match corpus source_sha".to_owned(),
        ));
    }
    if registry.corpus_manifest_sha256 != sha256_hex(corpus_bytes) {
        return Err(CorpusError::Contract(
            "assertion registry corpus_manifest_sha256 does not match exact corpus bytes"
                .to_owned(),
        ));
    }
    if registry.surface_policy_sha256 != sha256_hex(surface_policy_bytes) {
        return Err(CorpusError::Contract(
            "assertion registry surface_policy_sha256 does not match exact surface policy bytes"
                .to_owned(),
        ));
    }

    let allowed_surfaces = surface_policy
        .critical_surfaces
        .iter()
        .map(|surface| surface.surface_id.as_str())
        .collect::<BTreeSet<_>>();
    for assertion in &registry.entries {
        if !allowed_surfaces.contains(assertion.surface_id.as_str()) {
            return Err(CorpusError::Contract(format!(
                "assertion {} references unknown critical surface {}",
                assertion.assertion_id, assertion.surface_id
            )));
        }
    }

    if manifest.entries.len() != registry.entries.len() {
        return Err(CorpusError::Contract(format!(
            "corpus/assertion cardinality mismatch: {} scenarios versus {} assertions",
            manifest.entries.len(),
            registry.entries.len()
        )));
    }
    let assertions_by_id = registry
        .entries
        .iter()
        .map(|entry| (entry.assertion_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    if assertions_by_id.len() != registry.entries.len() {
        return Err(CorpusError::Contract("assertion ids are not unique".to_owned()));
    }

    for scenario in &manifest.entries {
        let assertion = assertions_by_id
            .get(scenario.assertion_id.as_str())
            .ok_or_else(|| {
                CorpusError::Contract(format!(
                    "scenario {} has no assertion {}",
                    scenario.scenario_id, scenario.assertion_id
                ))
            })?;
        if assertion.scenario_id != scenario.scenario_id {
            return Err(CorpusError::Contract(format!(
                "assertion {} binds scenario {}, expected {}",
                assertion.assertion_id, assertion.scenario_id, scenario.scenario_id
            )));
        }
        if assertion.expected_outcome != scenario.expected_outcome {
            return Err(CorpusError::Contract(format!(
                "assertion {} expected outcome does not match scenario {}",
                assertion.assertion_id, scenario.scenario_id
            )));
        }
    }

    let scenario_ids = manifest
        .entries
        .iter()
        .map(|entry| entry.scenario_id.as_str())
        .collect::<BTreeSet<_>>();
    for assertion in &registry.entries {
        if !scenario_ids.contains(assertion.scenario_id.as_str()) {
            return Err(CorpusError::Contract(format!(
                "assertion {} is orphaned from corpus scenario {}",
                assertion.assertion_id, assertion.scenario_id
            )));
        }
    }

    Ok((manifest, registry))
}

fn validate_manifest(manifest: &CorpusManifest) -> Result<(), CorpusError> {
    if manifest.schema != CORPUS_SCHEMA_ID {
        return Err(CorpusError::Contract(format!(
            "unexpected corpus schema {}",
            manifest.schema
        )));
    }
    validate_git_sha(&manifest.source_sha, "corpus source_sha")?;
    if manifest.max_fixture_bytes != MAX_FIXTURE_BYTES
        || manifest.max_total_bytes != MAX_TOTAL_BYTES
    {
        return Err(CorpusError::Contract(
            "corpus limits do not match the planning-frozen schema".to_owned(),
        ));
    }

    let mut scenarios = BTreeSet::new();
    let mut assertions = BTreeSet::new();
    let mut replays = BTreeSet::new();
    let mut fixture_paths = BTreeSet::new();
    let mut total_bytes = 0_u64;
    let mut previous_scenario: Option<&str> = None;

    for entry in &manifest.entries {
        validate_id(&entry.scenario_id, "scenario_id")?;
        validate_id(&entry.assertion_id, "assertion_id")?;
        validate_id(&entry.replay_id, "replay_id")?;
        validate_repo_path(&entry.fixture_path, "fixture_path")?;
        validate_sha256(&entry.fixture_sha256, "fixture_sha256")?;
        if entry.byte_length > MAX_FIXTURE_BYTES {
            return Err(CorpusError::Contract(format!(
                "scenario {} exceeds the per-fixture byte limit",
                entry.scenario_id
            )));
        }
        total_bytes = total_bytes.checked_add(entry.byte_length).ok_or_else(|| {
            CorpusError::Contract("corpus byte total overflowed".to_owned())
        })?;
        if total_bytes > MAX_TOTAL_BYTES {
            return Err(CorpusError::Contract(
                "corpus exceeds the aggregate committed byte limit".to_owned(),
            ));
        }
        if entry.contains_phi {
            return Err(CorpusError::Contract(format!(
                "scenario {} is marked as containing PHI",
                entry.scenario_id
            )));
        }
        if let Some(parent) = &entry.parent_scenario_id_or_null {
            validate_id(parent, "parent_scenario_id_or_null")?;
        }
        if let Some(tool) = &entry.minimization_tool_or_null {
            if tool.is_empty() || tool.len() > 160 {
                return Err(CorpusError::Contract(format!(
                    "scenario {} has invalid minimization tool identity",
                    entry.scenario_id
                )));
            }
        }
        if !scenarios.insert(entry.scenario_id.as_str()) {
            return Err(CorpusError::Contract(format!(
                "duplicate scenario id {}",
                entry.scenario_id
            )));
        }
        if !assertions.insert(entry.assertion_id.as_str()) {
            return Err(CorpusError::Contract(format!(
                "duplicate corpus assertion id {}",
                entry.assertion_id
            )));
        }
        if !replays.insert(entry.replay_id.as_str()) {
            return Err(CorpusError::Contract(format!(
                "duplicate replay id {}",
                entry.replay_id
            )));
        }
        if !fixture_paths.insert(entry.fixture_path.as_str()) {
            return Err(CorpusError::Contract(format!(
                "duplicate fixture path {}",
                entry.fixture_path
            )));
        }
        if previous_scenario.is_some_and(|previous| previous >= entry.scenario_id.as_str()) {
            return Err(CorpusError::Contract(
                "corpus entries must be strictly ordered by scenario_id".to_owned(),
            ));
        }
        previous_scenario = Some(entry.scenario_id.as_str());
    }
    Ok(())
}

fn validate_registry(registry: &AssertionRegistry) -> Result<(), CorpusError> {
    if registry.schema != ASSERTION_SCHEMA_ID {
        return Err(CorpusError::Contract(format!(
            "unexpected assertion registry schema {}",
            registry.schema
        )));
    }
    validate_git_sha(&registry.source_sha, "assertion registry source_sha")?;
    validate_sha256(
        &registry.corpus_manifest_sha256,
        "assertion registry corpus_manifest_sha256",
    )?;
    validate_sha256(
        &registry.surface_policy_sha256,
        "assertion registry surface_policy_sha256",
    )?;

    let mut assertion_ids = BTreeSet::new();
    let mut scenario_ids = BTreeSet::new();
    let mut previous_assertion: Option<&str> = None;
    for entry in &registry.entries {
        validate_id(&entry.assertion_id, "assertion_id")?;
        validate_id(&entry.scenario_id, "scenario_id")?;
        validate_id(&entry.surface_id, "surface_id")?;
        validate_repo_path(&entry.manifest_path, "manifest_path")?;
        validate_repo_path(&entry.cwd_repo_relative, "cwd_repo_relative")?;
        validate_id(&entry.result_parser_id, "result_parser_id")?;
        if entry.package_or_binary.is_empty() {
            return Err(CorpusError::Contract(format!(
                "assertion {} has empty package_or_binary",
                entry.assertion_id
            )));
        }
        if entry.argv.is_empty() || entry.argv.iter().any(|arg| arg.len() > 4096) {
            return Err(CorpusError::Contract(format!(
                "assertion {} has invalid argv",
                entry.assertion_id
            )));
        }
        for (key, value) in &entry.environment_allowlist {
            if !valid_environment_key(key) || value.len() > 4096 {
                return Err(CorpusError::Contract(format!(
                    "assertion {} has invalid environment allowlist entry {}",
                    entry.assertion_id, key
                )));
            }
        }
        if entry.source_paths.is_empty() {
            return Err(CorpusError::Contract(format!(
                "assertion {} must bind at least one source path",
                entry.assertion_id
            )));
        }
        validate_sorted_unique_paths(&entry.source_paths, "source_paths")?;
        validate_sorted_unique_sha256s(&entry.config_sha256s, "config_sha256s")?;

        match entry.runner_kind {
            RunnerKind::CargoTest => {
                let cargo_target_missing = entry
                    .cargo_target_or_null
                    .as_deref()
                    .is_none_or(str::is_empty);
                let test_name_missing = entry
                    .test_name_or_null
                    .as_deref()
                    .is_none_or(str::is_empty);
                if cargo_target_missing || test_name_missing {
                    return Err(CorpusError::Contract(format!(
                        "CARGO_TEST assertion {} requires cargo target and test name",
                        entry.assertion_id
                    )));
                }
            }
            RunnerKind::Af02ReplayBinary => {
                if entry.cargo_target_or_null.is_some() || entry.test_name_or_null.is_some() {
                    return Err(CorpusError::Contract(format!(
                        "AF02_REPLAY_BINARY assertion {} requires null Cargo target/test fields",
                        entry.assertion_id
                    )));
                }
            }
        }

        if !assertion_ids.insert(entry.assertion_id.as_str()) {
            return Err(CorpusError::Contract(format!(
                "duplicate assertion id {}",
                entry.assertion_id
            )));
        }
        if !scenario_ids.insert(entry.scenario_id.as_str()) {
            return Err(CorpusError::Contract(format!(
                "multiple assertions bind scenario {}",
                entry.scenario_id
            )));
        }
        if previous_assertion.is_some_and(|previous| previous >= entry.assertion_id.as_str()) {
            return Err(CorpusError::Contract(
                "assertion entries must be strictly ordered by assertion_id".to_owned(),
            ));
        }
        previous_assertion = Some(entry.assertion_id.as_str());
    }
    Ok(())
}

fn validate_git_sha(value: &str, label: &str) -> Result<(), CorpusError> {
    if value.len() != 40 || !value.bytes().all(is_lower_hex) {
        return Err(CorpusError::Contract(format!(
            "{label} must be 40 lowercase hexadecimal characters"
        )));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), CorpusError> {
    if value.len() != 64 || !value.bytes().all(is_lower_hex) {
        return Err(CorpusError::Contract(format!(
            "{label} must be 64 lowercase hexadecimal characters"
        )));
    }
    Ok(())
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn validate_id(value: &str, label: &str) -> Result<(), CorpusError> {
    if value.is_empty() || value.len() > 160 {
        return Err(CorpusError::Contract(format!("invalid {label}")));
    }
    let mut bytes = value.bytes();
    let first = bytes
        .next()
        .ok_or_else(|| CorpusError::Contract(format!("invalid {label}")))?;
    if !first.is_ascii_alphanumeric()
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        return Err(CorpusError::Contract(format!("invalid {label}")));
    }
    Ok(())
}

fn validate_repo_path(value: &str, label: &str) -> Result<(), CorpusError> {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains("//")
        || value.contains('\0')
        || value.split('/').any(|part| matches!(part, "." | ".."))
    {
        return Err(CorpusError::Contract(format!("invalid {label}: {value}")));
    }
    Ok(())
}

fn valid_environment_key(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_uppercase() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_sorted_unique_paths(values: &[String], label: &str) -> Result<(), CorpusError> {
    let mut previous: Option<&str> = None;
    for value in values {
        validate_repo_path(value, label)?;
        if previous.is_some_and(|prior| prior >= value.as_str()) {
            return Err(CorpusError::Contract(format!(
                "{label} must be strictly sorted and unique"
            )));
        }
        previous = Some(value.as_str());
    }
    Ok(())
}

fn validate_sorted_unique_sha256s(values: &[String], label: &str) -> Result<(), CorpusError> {
    let mut previous: Option<&str> = None;
    for value in values {
        validate_sha256(value, label)?;
        if previous.is_some_and(|prior| prior >= value.as_str()) {
            return Err(CorpusError::Contract(format!(
                "{label} must be strictly sorted and unique"
            )));
        }
        previous = Some(value.as_str());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORPUS_SCHEMA: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/schemas/af02-corpus-v1.schema.json"
    );
    const SURFACE_POLICY: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/surface-policy.json"
    );
    const SOURCE_SHA: &str = "94bf4f1a9987f474613e67ddbc182ece8dff5a8d";

    fn corpus(entry: &str) -> Vec<u8> {
        format!(
            "{{\"entries\":[{entry}],\"max_fixture_bytes\":262144,\"max_total_bytes\":8388608,\"schema\":\"commandf.af02-corpus/v1\",\"source_sha\":\"{SOURCE_SHA}\"}}"
        )
        .into_bytes()
    }

    fn scenario(contains_phi: bool, fixture_sha256: &str, byte_length: u64) -> String {
        format!(
            "{{\"assertion_id\":\"A001\",\"byte_length\":{byte_length},\"contains_phi\":{contains_phi},\"discovery_origin\":\"HAND_AUTHORED\",\"expected_outcome\":\"REJECT_INVALID\",\"fixture_path\":\"tests/assurance/corpus/f001.json\",\"fixture_sha256\":\"{fixture_sha256}\",\"minimization_tool_or_null\":null,\"parent_scenario_id_or_null\":null,\"provenance_class\":\"SYNTHETIC\",\"replay_id\":\"R001\",\"scenario_id\":\"S001\"}}"
        )
    }

    fn assertion(corpus_sha: &str, scenario_id: &str) -> Vec<u8> {
        let surface_sha = sha256_hex(SURFACE_POLICY);
        format!(
            "{{\"corpus_manifest_sha256\":\"{corpus_sha}\",\"entries\":[{{\"argv\":[\"cargo\",\"test\"],\"assertion_id\":\"A001\",\"cargo_target_or_null\":\"af02_corpus\",\"config_sha256s\":[],\"cwd_repo_relative\":\"tools\",\"environment_allowlist\":{{}},\"expected_outcome\":\"REJECT_INVALID\",\"manifest_path\":\"crates/commandf-pkg/Cargo.toml\",\"package_or_binary\":\"commandf-pkg\",\"result_parser_id\":\"cargo-test-v1\",\"runner_kind\":\"CARGO_TEST\",\"scenario_id\":\"{scenario_id}\",\"source_paths\":[\"crates/commandf-pkg/src/lock.rs\"],\"surface_id\":\"serde-json-from-slice\",\"test_name_or_null\":\"af02_corpus\"}}],\"schema\":\"commandf.af02-assertion-registry/v1\",\"source_sha\":\"{SOURCE_SHA}\",\"surface_policy_sha256\":\"{surface_sha}\"}}"
        )
        .into_bytes()
    }

    #[test]
    fn accepts_empty_design_freeze() {
        let corpus = corpus("");
        let registry = format!(
            "{{\"corpus_manifest_sha256\":\"{}\",\"entries\":[],\"schema\":\"commandf.af02-assertion-registry/v1\",\"source_sha\":\"{SOURCE_SHA}\",\"surface_policy_sha256\":\"{}\"}}",
            sha256_hex(&corpus),
            sha256_hex(SURFACE_POLICY)
        );
        validate_corpus_and_assertions(
            &corpus,
            CORPUS_SCHEMA,
            registry.as_bytes(),
            SURFACE_POLICY,
        )
        .unwrap();
    }

    #[test]
    fn rejects_phi_even_with_approved_provenance_class() {
        let raw = corpus(&scenario(true, &"0".repeat(64), 2));
        let error = parse_corpus_manifest(&raw, CORPUS_SCHEMA).unwrap_err();
        assert!(error.to_string().contains("containing PHI"));
    }

    #[test]
    fn rejects_orphan_assertion_scenario() {
        let fixture = b"{}";
        let raw = corpus(&scenario(false, &sha256_hex(fixture), 2));
        let registry = assertion(&sha256_hex(&raw), "S999");
        let error = validate_corpus_and_assertions(
            &raw,
            CORPUS_SCHEMA,
            &registry,
            SURFACE_POLICY,
        )
        .unwrap_err();
        assert!(error.to_string().contains("binds scenario"));
    }

    #[test]
    fn rejects_fixture_digest_mismatch() {
        let fixture = b"{}";
        let raw = corpus(&scenario(false, &"0".repeat(64), 2));
        let manifest = parse_corpus_manifest(&raw, CORPUS_SCHEMA).unwrap();
        let error = verify_fixture_bytes(&manifest.entries[0], fixture).unwrap_err();
        assert!(error.to_string().contains("SHA-256 mismatch"));
    }

    #[test]
    fn accepts_exact_fixture_bytes() {
        let fixture = b"{}";
        let raw = corpus(&scenario(false, &sha256_hex(fixture), 2));
        let manifest = parse_corpus_manifest(&raw, CORPUS_SCHEMA).unwrap();
        verify_fixture_bytes(&manifest.entries[0], fixture).unwrap();
    }

    #[test]
    fn rejects_replay_runner_with_cargo_target() {
        let raw = br#"{"corpus_manifest_sha256":"0000000000000000000000000000000000000000000000000000000000000000","entries":[{"argv":["replay"],"assertion_id":"A001","cargo_target_or_null":"not-null","config_sha256s":[],"cwd_repo_relative":"tools","environment_allowlist":{},"expected_outcome":"REJECT_INVALID","manifest_path":"tools/af02-verifier/Cargo.toml","package_or_binary":"commandf-af02-verifier","result_parser_id":"replay-v1","runner_kind":"AF02_REPLAY_BINARY","scenario_id":"S001","source_paths":["tools/af02-verifier/src/main.rs"],"surface_id":"filesystem-read","test_name_or_null":null}],"schema":"commandf.af02-assertion-registry/v1","source_sha":"94bf4f1a9987f474613e67ddbc182ece8dff5a8d","surface_policy_sha256":"0000000000000000000000000000000000000000000000000000000000000000"}"#;
        let error = parse_assertion_registry(raw).unwrap_err();
        assert!(error
            .to_string()
            .contains("requires null Cargo target/test fields"));
    }
}
