use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::canonical::{git_blob_sha1_hex, parse_json_no_duplicates, sha256_hex, CanonicalError};

pub const RETAINED_SCHEMA_ID: &str = "commandf.af02-retained-authority-sources/v1";

#[derive(Debug, Error)]
pub enum RetainedError {
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("canonicalization failed: {0}")]
    Canonical(#[from] CanonicalError),
    #[error("schema contract violation at {path}: {message}")]
    Schema { path: String, message: String },
    #[error("retained authority mismatch: {0}")]
    Mismatch(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedAuthoritySources {
    pub schema: String,
    pub repository: RepositoryIdentity,
    pub planning_base: PlanningBase,
    pub cf10: RetainedCf10,
    pub reconstruction: ReconstructionPolicy,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryIdentity {
    pub owner: String,
    pub name: String,
    pub full_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanningBase {
    pub sha: String,
    pub tree: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedCf10 {
    pub pull_request: PullRequestIdentity,
    pub retained_head: String,
    pub retained_base: String,
    pub manifest: RetainedFile,
    pub donor: RetainedFile,
    pub workflow_run: WorkflowRunIdentity,
    pub artifact: ArtifactIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PullRequestIdentity {
    pub number: u64,
    pub node_id_numeric: u64,
    pub head_ref: String,
    pub base_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedFile {
    pub path: String,
    pub git_blob_sha: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRunIdentity {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub event: String,
    pub workflow_id: u64,
    pub run_number: u64,
    pub run_attempt: u64,
    pub check_suite_id: u64,
    pub head_sha: String,
    pub base_sha: String,
    pub conclusion: String,
    pub pull_request_number: u64,
    pub pull_request_head_sha: String,
    pub pull_request_base_sha: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub id: u64,
    pub name: String,
    pub sha256: String,
    pub workflow_run_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconstructionPolicy {
    pub supplied_urls_are_authority: bool,
    pub api_urls_reconstructed_from_structured_fields: bool,
    pub git_blob_verified_before_parse: bool,
    pub raw_sha256_computed_after_git_identity: bool,
    pub retained_failure_preserved: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LocatorPlan {
    pub pull_request: String,
    pub retained_head_commit: String,
    pub retained_base_commit: String,
    pub manifest_contents: String,
    pub manifest_blob: String,
    pub donor_contents: String,
    pub donor_blob: String,
    pub workflow_run: String,
    pub workflow_run_artifacts: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    pub id: String,
    pub package: String,
    pub before: String,
    pub after: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub state_id: String,
    pub case_id: String,
    pub side: String,
    pub package: String,
    pub version: String,
    pub archive_sha256: String,
    pub archive_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RetainedProjection {
    pub deltas: Vec<Delta>,
    pub states: Vec<State>,
    pub retained_pr: u64,
    pub retained_head: String,
    pub retained_base: String,
    pub retained_run: u64,
    pub retained_run_conclusion: String,
    pub retained_artifact_id: u64,
    pub retained_artifact_name: String,
    pub retained_artifact_sha256: String,
    pub retained_manifest_blob_sha: String,
    pub retained_manifest_sha256: String,
    pub retained_donor_blob_sha: String,
    pub retained_donor_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusManifest {
    schema: u64,
    selection_policy: String,
    cases: Vec<CorpusCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusCase {
    id: String,
    package: String,
    before: CorpusSide,
    after: CorpusSide,
    fhir_version: String,
    publisher: String,
    change_evidence_url: String,
    rights_evidence_url: String,
    rights_mode: String,
    oracle_mode: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusSide {
    version: String,
    archive_sha256: String,
    archive_bytes: u64,
    publication_url: String,
}

pub fn validate_and_parse(
    instance_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<RetainedAuthoritySources, RetainedError> {
    let instance = parse_json_no_duplicates(instance_bytes)?;
    let schema = parse_json_no_duplicates(schema_bytes)?;
    let schema_id = schema
        .get("$id")
        .and_then(Value::as_str)
        .ok_or_else(|| RetainedError::Schema {
            path: "$".to_owned(),
            message: "trusted schema is missing $id".to_owned(),
        })?;
    if schema_id != "https://commandf.dev/schemas/af02-retained-authority-sources-v1.schema.json" {
        return Err(RetainedError::Schema {
            path: "$".to_owned(),
            message: format!("unexpected trusted schema id {schema_id}"),
        });
    }
    validate_schema_node(&instance, &schema, "$")?;
    let parsed: RetainedAuthoritySources = serde_json::from_value(instance)?;
    if parsed.schema != RETAINED_SCHEMA_ID {
        return Err(RetainedError::Mismatch(format!(
            "unexpected retained schema {}",
            parsed.schema
        )));
    }
    Ok(parsed)
}

pub fn locator_plan(retained: &RetainedAuthoritySources) -> Result<LocatorPlan, RetainedError> {
    if retained.repository.full_name
        != format!("{}/{}", retained.repository.owner, retained.repository.name)
    {
        return Err(RetainedError::Mismatch(
            "repository full_name does not match owner/name".to_owned(),
        ));
    }
    if retained.reconstruction.supplied_urls_are_authority
        || !retained
            .reconstruction
            .api_urls_reconstructed_from_structured_fields
        || !retained.reconstruction.git_blob_verified_before_parse
        || !retained
            .reconstruction
            .raw_sha256_computed_after_git_identity
        || !retained.reconstruction.retained_failure_preserved
    {
        return Err(RetainedError::Mismatch(
            "reconstruction policy weakens the closed retained-authority contract".to_owned(),
        ));
    }

    let repo = &retained.repository.full_name;
    let cf10 = &retained.cf10;
    Ok(LocatorPlan {
        pull_request: format!(
            "https://api.github.com/repos/{repo}/pulls/{}",
            cf10.pull_request.number
        ),
        retained_head_commit: format!(
            "https://api.github.com/repos/{repo}/commits/{}",
            cf10.retained_head
        ),
        retained_base_commit: format!(
            "https://api.github.com/repos/{repo}/commits/{}",
            cf10.retained_base
        ),
        manifest_contents: format!(
            "https://api.github.com/repos/{repo}/contents/{}?ref={}",
            cf10.manifest.path, cf10.retained_head
        ),
        manifest_blob: format!(
            "https://api.github.com/repos/{repo}/git/blobs/{}",
            cf10.manifest.git_blob_sha
        ),
        donor_contents: format!(
            "https://api.github.com/repos/{repo}/contents/{}?ref={}",
            cf10.donor.path, cf10.retained_head
        ),
        donor_blob: format!(
            "https://api.github.com/repos/{repo}/git/blobs/{}",
            cf10.donor.git_blob_sha
        ),
        workflow_run: format!(
            "https://api.github.com/repos/{repo}/actions/runs/{}",
            cf10.workflow_run.id
        ),
        workflow_run_artifacts: format!(
            "https://api.github.com/repos/{repo}/actions/runs/{}/artifacts",
            cf10.workflow_run.id
        ),
    })
}

pub fn verify_workflow_run(
    retained: &RetainedAuthoritySources,
    run: &Value,
) -> Result<(), RetainedError> {
    let expected = &retained.cf10.workflow_run;
    expect_u64(run, "id", expected.id)?;
    expect_str(run, "name", &expected.name)?;
    expect_str(run, "path", &expected.path)?;
    expect_str(run, "event", &expected.event)?;
    expect_u64(run, "workflow_id", expected.workflow_id)?;
    expect_u64(run, "run_number", expected.run_number)?;
    expect_u64(run, "run_attempt", expected.run_attempt)?;
    expect_u64(run, "check_suite_id", expected.check_suite_id)?;
    expect_str(run, "head_sha", &expected.head_sha)?;
    expect_str(run, "conclusion", &expected.conclusion)?;

    let pull_requests = run
        .get("pull_requests")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            RetainedError::Mismatch("workflow run pull_requests is missing".to_owned())
        })?;
    let matching: Vec<&Value> = pull_requests
        .iter()
        .filter(|item| {
            item.get("number").and_then(Value::as_u64) == Some(expected.pull_request_number)
        })
        .collect();
    if matching.len() != 1 {
        return Err(RetainedError::Mismatch(format!(
            "workflow run must bind exactly one PR {}, observed {}",
            expected.pull_request_number,
            matching.len()
        )));
    }
    let pr = matching[0];
    let head = pr
        .get("head")
        .ok_or_else(|| RetainedError::Mismatch("workflow run PR head is missing".to_owned()))?;
    let base = pr
        .get("base")
        .ok_or_else(|| RetainedError::Mismatch("workflow run PR base is missing".to_owned()))?;
    expect_str(head, "ref", &retained.cf10.pull_request.head_ref)?;
    expect_str(head, "sha", &expected.pull_request_head_sha)?;
    expect_str(base, "ref", &retained.cf10.pull_request.base_ref)?;
    expect_str(base, "sha", &expected.pull_request_base_sha)?;

    if expected.head_sha != expected.pull_request_head_sha
        || expected.base_sha != expected.pull_request_base_sha
        || expected.head_sha != retained.cf10.retained_head
        || expected.base_sha != retained.cf10.retained_base
    {
        return Err(RetainedError::Mismatch(
            "retained run/head/base cross-binding is inconsistent".to_owned(),
        ));
    }
    Ok(())
}

pub fn verify_artifacts(
    retained: &RetainedAuthoritySources,
    artifacts: &Value,
) -> Result<(), RetainedError> {
    let items = artifacts
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            RetainedError::Mismatch("artifact collection is missing artifacts".to_owned())
        })?;
    let expected = &retained.cf10.artifact;
    let matching: Vec<&Value> = items
        .iter()
        .filter(|item| item.get("id").and_then(Value::as_u64) == Some(expected.id))
        .collect();
    if matching.len() != 1 {
        return Err(RetainedError::Mismatch(format!(
            "expected exactly one artifact {}, observed {}",
            expected.id,
            matching.len()
        )));
    }
    let artifact = matching[0];
    expect_str(artifact, "name", &expected.name)?;
    let digest = artifact
        .get("digest")
        .and_then(Value::as_str)
        .ok_or_else(|| RetainedError::Mismatch("artifact digest is missing".to_owned()))?;
    if digest != format!("sha256:{}", expected.sha256) {
        return Err(RetainedError::Mismatch(format!(
            "artifact digest mismatch: expected sha256:{}, got {digest}",
            expected.sha256
        )));
    }
    let run = artifact
        .get("workflow_run")
        .ok_or_else(|| RetainedError::Mismatch("artifact workflow_run is missing".to_owned()))?;
    expect_u64(run, "id", expected.workflow_run_id)?;
    expect_str(run, "head_sha", &retained.cf10.retained_head)?;
    expect_str(run, "head_branch", &retained.cf10.pull_request.head_ref)?;
    Ok(())
}

pub fn project_retained(
    retained: &RetainedAuthoritySources,
    manifest_bytes: &[u8],
    donor_bytes: &[u8],
) -> Result<RetainedProjection, RetainedError> {
    let manifest_blob = git_blob_sha1_hex(manifest_bytes);
    if manifest_blob != retained.cf10.manifest.git_blob_sha {
        return Err(RetainedError::Mismatch(format!(
            "retained manifest Git blob mismatch: expected {}, got {manifest_blob}",
            retained.cf10.manifest.git_blob_sha
        )));
    }
    let donor_blob = git_blob_sha1_hex(donor_bytes);
    if donor_blob != retained.cf10.donor.git_blob_sha {
        return Err(RetainedError::Mismatch(format!(
            "retained donor Git blob mismatch: expected {}, got {donor_blob}",
            retained.cf10.donor.git_blob_sha
        )));
    }

    let manifest_value = parse_json_no_duplicates(manifest_bytes)?;
    let manifest: CorpusManifest = serde_json::from_value(manifest_value)?;
    if manifest.schema != 1 || manifest.selection_policy != "frozen_pre_result_v1" {
        return Err(RetainedError::Mismatch(
            "retained corpus manifest schema/selection policy drifted".to_owned(),
        ));
    }
    if manifest.cases.len() != 3 {
        return Err(RetainedError::Mismatch(format!(
            "retained corpus must contain exactly three cases, observed {}",
            manifest.cases.len()
        )));
    }

    let expected_ids = ["C001", "C002", "C003"];
    for (case, expected_id) in manifest.cases.iter().zip(expected_ids) {
        if case.id != expected_id {
            return Err(RetainedError::Mismatch(format!(
                "retained corpus order/id mismatch: expected {expected_id}, got {}",
                case.id
            )));
        }
        validate_metadata_only_case(case)?;
    }

    let mut deltas = Vec::with_capacity(3);
    let mut states = Vec::with_capacity(6);
    for case in &manifest.cases {
        deltas.push(Delta {
            id: case.id.clone(),
            package: case.package.clone(),
            before: case.before.version.clone(),
            after: case.after.version.clone(),
        });
        states.push(State {
            state_id: format!("{}-after", case.id),
            case_id: case.id.clone(),
            side: "after".to_owned(),
            package: case.package.clone(),
            version: case.after.version.clone(),
            archive_sha256: case.after.archive_sha256.clone(),
            archive_bytes: case.after.archive_bytes,
        });
        states.push(State {
            state_id: format!("{}-before", case.id),
            case_id: case.id.clone(),
            side: "before".to_owned(),
            package: case.package.clone(),
            version: case.before.version.clone(),
            archive_sha256: case.before.archive_sha256.clone(),
            archive_bytes: case.before.archive_bytes,
        });
    }

    Ok(RetainedProjection {
        deltas,
        states,
        retained_pr: retained.cf10.pull_request.number,
        retained_head: retained.cf10.retained_head.clone(),
        retained_base: retained.cf10.retained_base.clone(),
        retained_run: retained.cf10.workflow_run.id,
        retained_run_conclusion: retained.cf10.workflow_run.conclusion.clone(),
        retained_artifact_id: retained.cf10.artifact.id,
        retained_artifact_name: retained.cf10.artifact.name.clone(),
        retained_artifact_sha256: retained.cf10.artifact.sha256.clone(),
        retained_manifest_blob_sha: retained.cf10.manifest.git_blob_sha.clone(),
        retained_manifest_sha256: sha256_hex(manifest_bytes),
        retained_donor_blob_sha: retained.cf10.donor.git_blob_sha.clone(),
        retained_donor_sha256: sha256_hex(donor_bytes),
    })
}

fn validate_metadata_only_case(case: &CorpusCase) -> Result<(), RetainedError> {
    let allowed = [
        ("C001", "hl7.fhir.us.core", "8.0.1", "9.0.0"),
        ("C002", "hl7.fhir.uv.ips", "1.1.0", "2.0.1"),
        ("C003", "hl7.fhir.us.mcode", "3.0.0", "4.0.0"),
    ];
    let expected = allowed
        .iter()
        .find(|entry| entry.0 == case.id)
        .ok_or_else(|| RetainedError::Mismatch(format!("unknown retained case {}", case.id)))?;
    if case.package != expected.1
        || case.before.version != expected.2
        || case.after.version != expected.3
        || case.fhir_version != "4.0.1"
        || case.publisher != "HL7 International"
        || case.rights_mode != "metadata_only_no_redistribution"
        || case.oracle_mode != "changed_structure_definitions_only"
    {
        return Err(RetainedError::Mismatch(format!(
            "retained case {} semantic identity drifted",
            case.id
        )));
    }
    for value in [
        &case.before.publication_url,
        &case.after.publication_url,
        &case.change_evidence_url,
        &case.rights_evidence_url,
    ] {
        if !value.starts_with("https://hl7.org/") {
            return Err(RetainedError::Mismatch(format!(
                "retained case {} contains unexpected publication authority",
                case.id
            )));
        }
    }
    Ok(())
}

fn validate_schema_node(instance: &Value, schema: &Value, path: &str) -> Result<(), RetainedError> {
    if let Some(expected) = schema.get("const") {
        if instance != expected {
            return Err(RetainedError::Schema {
                path: path.to_owned(),
                message: "value does not equal trusted const".to_owned(),
            });
        }
        return Ok(());
    }

    if let Some(kind) = schema.get("type").and_then(Value::as_str) {
        let matches = match kind {
            "object" => instance.is_object(),
            "array" => instance.is_array(),
            "string" => instance.is_string(),
            "integer" => instance.as_i64().is_some() || instance.as_u64().is_some(),
            "boolean" => instance.is_boolean(),
            "null" => instance.is_null(),
            other => {
                return Err(RetainedError::Schema {
                    path: path.to_owned(),
                    message: format!("unsupported trusted schema type {other}"),
                });
            }
        };
        if !matches {
            return Err(RetainedError::Schema {
                path: path.to_owned(),
                message: format!("expected type {kind}"),
            });
        }
    }

    if let Some(object) = instance.as_object() {
        let properties = schema
            .get("properties")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        if schema.get("additionalProperties") == Some(&Value::Bool(false)) {
            let allowed: BTreeSet<&str> = properties.keys().map(String::as_str).collect();
            for key in object.keys() {
                if !allowed.contains(key.as_str()) {
                    return Err(RetainedError::Schema {
                        path: format!("{path}.{key}"),
                        message: "unknown field".to_owned(),
                    });
                }
            }
        }
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required {
                let key = key.as_str().ok_or_else(|| RetainedError::Schema {
                    path: path.to_owned(),
                    message: "trusted schema required entry is not a string".to_owned(),
                })?;
                if !object.contains_key(key) {
                    return Err(RetainedError::Schema {
                        path: format!("{path}.{key}"),
                        message: "required field is missing".to_owned(),
                    });
                }
            }
        }
        for (key, child) in object {
            if let Some(child_schema) = properties.get(key) {
                validate_schema_node(child, child_schema, &format!("{path}.{key}"))?;
            }
        }
    }
    Ok(())
}

fn expect_str(value: &Value, field: &str, expected: &str) -> Result<(), RetainedError> {
    let observed = value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| RetainedError::Mismatch(format!("{field} is missing or not a string")))?;
    if observed != expected {
        return Err(RetainedError::Mismatch(format!(
            "{field} mismatch: expected {expected}, got {observed}"
        )));
    }
    Ok(())
}

fn expect_u64(value: &Value, field: &str, expected: u64) -> Result<(), RetainedError> {
    let observed = value
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| RetainedError::Mismatch(format!("{field} is missing or not an integer")))?;
    if observed != expected {
        return Err(RetainedError::Mismatch(format!(
            "{field} mismatch: expected {expected}, got {observed}"
        )));
    }
    Ok(())
}
