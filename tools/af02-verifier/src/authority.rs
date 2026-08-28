use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::canonical::{canonical_sha256, sha256_hex, CanonicalError};
use crate::retained::{RetainedError, RetainedProjection};

pub const AUTHORITY_BASELINE_SCHEMA: &str = "commandf.af02-authority-baseline/v2";
pub const ASSURANCE_RULESET_ID: u64 = 21652953;
pub const REVIEW_RULESET_ID: u64 = 21652974;

pub const CF06_PROJECT: &str = "hapifhir/org.hl7.fhir.core";
pub const CF06_RELEASE: &str = "6.10.2";
pub const CF06_SOURCE_COMMIT: &str = "d06577dbc5c62c74a2a8823fbc4830a3024d5b0b";
pub const CF06_VALIDATOR_SHA256: &str =
    "a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6";
pub const CF06_R4_CONTEXT: &str = "hl7.fhir.r4.core@4.0.1";

#[derive(Debug, Error)]
pub enum AuthorityError {
    #[error("canonicalization failed: {0}")]
    Canonical(#[from] CanonicalError),
    #[error("retained authority error: {0}")]
    Retained(#[from] RetainedError),
    #[error("authority mismatch: {0}")]
    Mismatch(String),
    #[error("failed to serialize authority projection: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityBaseline {
    pub schema: String,
    pub captured_from_main_sha: String,
    pub captured_from_main_tree: String,
    pub af01: Af01Baseline,
    pub cf06: Cf06Baseline,
    pub cf10: Cf10Baseline,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Af01Baseline {
    pub assurance: RulesetProjection,
    pub review_governance: RulesetProjection,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RulesetProjection {
    pub ruleset_id: u64,
    pub semantic_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cf06Baseline {
    pub project: String,
    pub release: String,
    pub source_commit: String,
    pub validator_cli_jar_sha256: String,
    pub r4_core_context: String,
    pub projection_sha256: String,
    pub source_files: Vec<SourceFileEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFileEvidence {
    pub path: String,
    pub git_blob_sha: String,
    pub raw_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cf10Baseline {
    pub deltas: Vec<crate::retained::Delta>,
    pub states: Vec<crate::retained::State>,
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
    pub projection_sha256: String,
}

pub struct Cf06Source<'a> {
    pub path: &'a str,
    pub git_blob_sha: &'a str,
    pub bytes: &'a [u8],
}

pub fn project_authority(
    captured_from_main_sha: &str,
    captured_from_main_tree: &str,
    assurance_ruleset: &Value,
    review_ruleset: &Value,
    cf06_sources: [Cf06Source<'_>; 3],
    retained: RetainedProjection,
) -> Result<AuthorityBaseline, AuthorityError> {
    validate_git_sha(captured_from_main_sha, "captured_from_main_sha")?;
    validate_git_sha(captured_from_main_tree, "captured_from_main_tree")?;

    let assurance = project_assurance_ruleset(assurance_ruleset)?;
    let review_governance = project_review_ruleset(review_ruleset)?;
    let cf06 = project_cf06(cf06_sources)?;
    let cf10 = project_cf10(retained)?;

    Ok(AuthorityBaseline {
        schema: AUTHORITY_BASELINE_SCHEMA.to_owned(),
        captured_from_main_sha: captured_from_main_sha.to_owned(),
        captured_from_main_tree: captured_from_main_tree.to_owned(),
        af01: Af01Baseline {
            assurance,
            review_governance,
        },
        cf06,
        cf10,
    })
}

pub fn project_assurance_ruleset(value: &Value) -> Result<RulesetProjection, AuthorityError> {
    validate_ruleset_header(value, ASSURANCE_RULESET_ID)?;
    let bypass = value
        .get("bypass_actors")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("assurance bypass_actors missing".to_owned()))?;
    if !bypass.is_empty() {
        return Err(AuthorityError::Mismatch(
            "assurance ruleset must have no bypass actors".to_owned(),
        ));
    }

    let rules = value
        .get("rules")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("assurance rules missing".to_owned()))?;
    if rules.len() != 3 {
        return Err(AuthorityError::Mismatch(format!(
            "assurance ruleset must contain exactly three rules, observed {}",
            rules.len()
        )));
    }
    let deletion = exactly_one_rule(rules, "deletion")?;
    reject_parameters(deletion, "deletion")?;
    let non_fast_forward = exactly_one_rule(rules, "non_fast_forward")?;
    reject_parameters(non_fast_forward, "non_fast_forward")?;
    let required = exactly_one_rule(rules, "required_status_checks")?;
    let parameters = required
        .get("parameters")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            AuthorityError::Mismatch("required_status_checks parameters missing".to_owned())
        })?;
    if parameters
        .get("strict_required_status_checks_policy")
        .and_then(Value::as_bool)
        != Some(true)
        || parameters
            .get("do_not_enforce_on_create")
            .and_then(Value::as_bool)
            != Some(false)
    {
        return Err(AuthorityError::Mismatch(
            "assurance strict required-check policy drifted".to_owned(),
        ));
    }
    let checks = parameters
        .get("required_status_checks")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("required status checks missing".to_owned()))?;
    if checks.len() != 3 {
        return Err(AuthorityError::Mismatch(format!(
            "required status checks must contain exactly three entries, observed {}",
            checks.len()
        )));
    }

    let mut normalized = Vec::new();
    for check in checks {
        let context = check
            .get("context")
            .and_then(Value::as_str)
            .ok_or_else(|| AuthorityError::Mismatch("required check context missing".to_owned()))?;
        let integration_id = check
            .get("integration_id")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                AuthorityError::Mismatch("required check integration_id missing".to_owned())
            })?;
        if integration_id != 15368 {
            return Err(AuthorityError::Mismatch(format!(
                "required check {context} has unexpected integration {integration_id}"
            )));
        }
        normalized.push((context.to_owned(), integration_id));
    }
    normalized.sort();
    let expected = vec![
        ("assurance-proof".to_owned(), 15368),
        ("rust".to_owned(), 15368),
        ("scorecard".to_owned(), 15368),
    ];
    if normalized != expected {
        return Err(AuthorityError::Mismatch(format!(
            "required check membership drifted: {normalized:?}"
        )));
    }

    let semantic = json!({
        "bypass_actors": [],
        "deletion": true,
        "enforcement": "active",
        "non_fast_forward": true,
        "ref_name": {
            "exclude": [],
            "include": ["refs/heads/main"]
        },
        "required_status_checks": normalized
            .iter()
            .map(|(context, integration_id)| json!({
                "context": context,
                "integration_id": integration_id
            }))
            .collect::<Vec<_>>(),
        "strict_required_status_checks_policy": true
    });

    Ok(RulesetProjection {
        ruleset_id: ASSURANCE_RULESET_ID,
        semantic_sha256: canonical_sha256(&semantic)?,
    })
}

pub fn project_review_ruleset(value: &Value) -> Result<RulesetProjection, AuthorityError> {
    validate_ruleset_header(value, REVIEW_RULESET_ID)?;
    let bypass = value
        .get("bypass_actors")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("review bypass_actors missing".to_owned()))?;
    if bypass.len() != 1 {
        return Err(AuthorityError::Mismatch(format!(
            "review governance must contain one bypass actor, observed {}",
            bypass.len()
        )));
    }
    let actor = &bypass[0];
    if actor.get("actor_id").and_then(Value::as_u64) != Some(5)
        || actor.get("actor_type").and_then(Value::as_str) != Some("RepositoryRole")
        || actor.get("bypass_mode").and_then(Value::as_str) != Some("pull_request")
    {
        return Err(AuthorityError::Mismatch(
            "review governance bypass actor drifted".to_owned(),
        ));
    }

    let rules = value
        .get("rules")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("review rules missing".to_owned()))?;
    if rules.len() != 1 {
        return Err(AuthorityError::Mismatch(format!(
            "review governance must contain exactly one rule, observed {}",
            rules.len()
        )));
    }
    let pull_request = exactly_one_rule(rules, "pull_request")?;
    let parameters = pull_request
        .get("parameters")
        .and_then(Value::as_object)
        .ok_or_else(|| AuthorityError::Mismatch("pull_request parameters missing".to_owned()))?;

    expect_u64(parameters, "required_approving_review_count", 1)?;
    expect_bool(parameters, "dismiss_stale_reviews_on_push", true)?;
    expect_bool(parameters, "require_code_owner_review", true)?;
    expect_bool(parameters, "require_last_push_approval", true)?;
    expect_bool(parameters, "required_review_thread_resolution", true)?;
    expect_bool(
        parameters,
        "require_extra_approval_for_unattributed_changes",
        true,
    )?;
    let required_reviewers = parameters
        .get("required_reviewers")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("required_reviewers missing".to_owned()))?;
    if !required_reviewers.is_empty() {
        return Err(AuthorityError::Mismatch(
            "review governance required_reviewers unexpectedly non-empty".to_owned(),
        ));
    }
    let allowed_merge_methods = parameters
        .get("allowed_merge_methods")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("allowed_merge_methods missing".to_owned()))?;
    if allowed_merge_methods.as_slice() != [Value::String("merge".to_owned())] {
        return Err(AuthorityError::Mismatch(
            "review governance must remain merge-only".to_owned(),
        ));
    }

    let semantic = json!({
        "allowed_merge_methods": ["merge"],
        "bypass_actors": [{
            "actor_id": 5,
            "actor_type": "RepositoryRole",
            "bypass_mode": "pull_request"
        }],
        "dismiss_stale_reviews_on_push": true,
        "enforcement": "active",
        "ref_name": {
            "exclude": [],
            "include": ["refs/heads/main"]
        },
        "require_code_owner_review": true,
        "require_extra_approval_for_unattributed_changes": true,
        "require_last_push_approval": true,
        "required_approving_review_count": 1,
        "required_review_thread_resolution": true,
        "required_reviewers": []
    });

    Ok(RulesetProjection {
        ruleset_id: REVIEW_RULESET_ID,
        semantic_sha256: canonical_sha256(&semantic)?,
    })
}

pub fn project_cf06(sources: [Cf06Source<'_>; 3]) -> Result<Cf06Baseline, AuthorityError> {
    let expected_paths = [
        "crates/commandf-pkg/src/oracle_model.rs",
        "donors/hl7-fhir-validator-6.10.2.yaml",
        ".github/workflows/cf06-oracle.yml",
    ];
    for (source, expected_path) in sources.iter().zip(expected_paths) {
        if source.path != expected_path {
            return Err(AuthorityError::Mismatch(format!(
                "CF-06 source order/path mismatch: expected {expected_path}, got {}",
                source.path
            )));
        }
        validate_git_sha(source.git_blob_sha, source.path)?;
    }

    let oracle = std::str::from_utf8(sources[0].bytes)
        .map_err(|_| AuthorityError::Mismatch("oracle_model.rs is not UTF-8".to_owned()))?;
    for required in [
        CF06_PROJECT,
        CF06_RELEASE,
        CF06_SOURCE_COMMIT,
        CF06_VALIDATOR_SHA256,
    ] {
        if !oracle.contains(required) {
            return Err(AuthorityError::Mismatch(format!(
                "oracle_model.rs does not bind {required}"
            )));
        }
    }

    let donor = std::str::from_utf8(sources[1].bytes)
        .map_err(|_| AuthorityError::Mismatch("CF-06 donor is not UTF-8".to_owned()))?;
    for required in [
        "repository: https://github.com/hapifhir/org.hl7.fhir.core",
        "tag: 6.10.2",
        CF06_SOURCE_COMMIT,
        CF06_VALIDATOR_SHA256,
    ] {
        if !donor.contains(required) {
            return Err(AuthorityError::Mismatch(format!(
                "CF-06 donor does not bind {required}"
            )));
        }
    }

    let workflow = std::str::from_utf8(sources[2].bytes)
        .map_err(|_| AuthorityError::Mismatch("CF-06 workflow is not UTF-8".to_owned()))?;
    for required in [
        "name: cf06-oracle",
        CF06_R4_CONTEXT,
        "oracle-proof:",
        "test \"$SELF_SMOKE_RESULT\" = success",
        "test \"$CHANGED_PROFILE_RESULT\" = success",
    ] {
        if !workflow.contains(required) {
            return Err(AuthorityError::Mismatch(format!(
                "CF-06 workflow does not bind {required}"
            )));
        }
    }

    let projection = json!({
        "project": CF06_PROJECT,
        "r4_core_context": CF06_R4_CONTEXT,
        "release": CF06_RELEASE,
        "source_commit": CF06_SOURCE_COMMIT,
        "validator_cli_jar_sha256": CF06_VALIDATOR_SHA256
    });

    Ok(Cf06Baseline {
        project: CF06_PROJECT.to_owned(),
        release: CF06_RELEASE.to_owned(),
        source_commit: CF06_SOURCE_COMMIT.to_owned(),
        validator_cli_jar_sha256: CF06_VALIDATOR_SHA256.to_owned(),
        r4_core_context: CF06_R4_CONTEXT.to_owned(),
        projection_sha256: canonical_sha256(&projection)?,
        source_files: sources
            .into_iter()
            .map(|source| SourceFileEvidence {
                path: source.path.to_owned(),
                git_blob_sha: source.git_blob_sha.to_owned(),
                raw_sha256: sha256_hex(source.bytes),
            })
            .collect(),
    })
}

pub fn project_cf10(retained: RetainedProjection) -> Result<Cf10Baseline, AuthorityError> {
    if retained.retained_run_conclusion != "failure" {
        return Err(AuthorityError::Mismatch(
            "CF-10 retained run must remain failure".to_owned(),
        ));
    }
    let semantic = serde_json::to_value(&retained)?;
    let projection_sha256 = canonical_sha256(&semantic)?;
    Ok(Cf10Baseline {
        deltas: retained.deltas,
        states: retained.states,
        retained_pr: retained.retained_pr,
        retained_head: retained.retained_head,
        retained_base: retained.retained_base,
        retained_run: retained.retained_run,
        retained_run_conclusion: retained.retained_run_conclusion,
        retained_artifact_id: retained.retained_artifact_id,
        retained_artifact_name: retained.retained_artifact_name,
        retained_artifact_sha256: retained.retained_artifact_sha256,
        retained_manifest_blob_sha: retained.retained_manifest_blob_sha,
        retained_manifest_sha256: retained.retained_manifest_sha256,
        retained_donor_blob_sha: retained.retained_donor_blob_sha,
        retained_donor_sha256: retained.retained_donor_sha256,
        projection_sha256,
    })
}

fn validate_ruleset_header(value: &Value, expected_id: u64) -> Result<(), AuthorityError> {
    if value.get("id").and_then(Value::as_u64) != Some(expected_id) {
        return Err(AuthorityError::Mismatch(format!(
            "ruleset id mismatch, expected {expected_id}"
        )));
    }
    if value.get("enforcement").and_then(Value::as_str) != Some("active")
        || value.get("target").and_then(Value::as_str) != Some("branch")
        || value.get("source_type").and_then(Value::as_str) != Some("Repository")
        || value.get("source").and_then(Value::as_str) != Some("TheHalfMoon/commandF")
    {
        return Err(AuthorityError::Mismatch(format!(
            "ruleset {expected_id} header drifted"
        )));
    }
    let ref_name = value
        .get("conditions")
        .and_then(|value| value.get("ref_name"))
        .ok_or_else(|| AuthorityError::Mismatch("ruleset ref_name condition missing".to_owned()))?;
    let include = ref_name
        .get("include")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("ruleset include missing".to_owned()))?;
    let exclude = ref_name
        .get("exclude")
        .and_then(Value::as_array)
        .ok_or_else(|| AuthorityError::Mismatch("ruleset exclude missing".to_owned()))?;
    if include.as_slice() != [Value::String("refs/heads/main".to_owned())] || !exclude.is_empty() {
        return Err(AuthorityError::Mismatch(format!(
            "ruleset {expected_id} ref scope drifted"
        )));
    }
    Ok(())
}

fn exactly_one_rule<'a>(rules: &'a [Value], kind: &str) -> Result<&'a Value, AuthorityError> {
    let matching: Vec<&Value> = rules
        .iter()
        .filter(|rule| rule.get("type").and_then(Value::as_str) == Some(kind))
        .collect();
    if matching.len() != 1 {
        return Err(AuthorityError::Mismatch(format!(
            "expected exactly one {kind} rule, observed {}",
            matching.len()
        )));
    }
    Ok(matching[0])
}

fn reject_parameters(rule: &Value, kind: &str) -> Result<(), AuthorityError> {
    if rule.get("parameters").is_some() {
        return Err(AuthorityError::Mismatch(format!(
            "{kind} rule unexpectedly has parameters"
        )));
    }
    Ok(())
}

fn expect_bool(
    parameters: &serde_json::Map<String, Value>,
    field: &str,
    expected: bool,
) -> Result<(), AuthorityError> {
    if parameters.get(field).and_then(Value::as_bool) != Some(expected) {
        return Err(AuthorityError::Mismatch(format!(
            "{field} mismatch, expected {expected}"
        )));
    }
    Ok(())
}

fn expect_u64(
    parameters: &serde_json::Map<String, Value>,
    field: &str,
    expected: u64,
) -> Result<(), AuthorityError> {
    if parameters.get(field).and_then(Value::as_u64) != Some(expected) {
        return Err(AuthorityError::Mismatch(format!(
            "{field} mismatch, expected {expected}"
        )));
    }
    Ok(())
}

fn validate_git_sha(value: &str, field: &str) -> Result<(), AuthorityError> {
    if value.len() != 40
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(AuthorityError::Mismatch(format!(
            "{field} is not a lowercase 40-hex Git SHA"
        )));
    }
    Ok(())
}
