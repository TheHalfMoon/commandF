use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::canonical::{git_blob_sha1_hex, parse_json_no_duplicates, sha256_hex};

const POLICY_SCHEMA_ID: &str = "commandf.af02-required-check-policy/v1";
const POLICY_SCHEMA_URL: &str =
    "https://commandf.dev/schemas/af02-required-check-policy-v1.schema.json";
const POLICY_SCHEMA_GIT_BLOB_SHA: &str = "3036ba3cb5debfff3f1f888bc59a8be7e0ce7ff1";
const PROVENANCE_SCHEMA_ID: &str = "commandf.af02-required-check-provenance/v1";
const PROVENANCE_SCHEMA_URL: &str =
    "https://commandf.dev/schemas/af02-required-check-provenance-v1.schema.json";
const PROVENANCE_SCHEMA_GIT_BLOB_SHA: &str = "1daaeefdac79ad02b9e8d49bc18fe8bdd47ad355";
const REPOSITORY: &str = "TheHalfMoon/commandF";
const APP_ID: u64 = 15_368;
const APP_SLUG: &str = "github-actions";
const APP_OWNER: &str = "github";
const EVENT: &str = "pull_request";
const SUCCESS: &str = "success";

#[derive(Debug, Error)]
pub enum GitHubProvenanceError {
    #[error("required-check JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("required-check schema violation: {0}")]
    Schema(String),
    #[error("required-check provenance violation: {0}")]
    Contract(String),
    #[error("trusted GitHub resolver failure: {0}")]
    Resolver(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCheckPolicy {
    pub schema: String,
    pub repository: String,
    pub app: RequiredCheckApp,
    pub event: String,
    pub checks: Vec<RequiredCheckPolicyEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCheckApp {
    pub id: u64,
    pub slug: String,
    pub owner: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCheckPolicyEntry {
    pub context: String,
    pub workflow_id: u64,
    pub workflow_path: String,
    pub workflow_blob_sha: String,
    pub job_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCheckProvenance {
    pub schema: String,
    pub repository: String,
    pub head_sha: String,
    pub base_sha: String,
    pub policy_sha256: String,
    pub checks: RequiredCheckEvidenceSet,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCheckEvidenceSet {
    #[serde(rename = "assurance-proof")]
    pub assurance_proof: RequiredCheckEvidence,
    pub rust: RequiredCheckEvidence,
    pub scorecard: RequiredCheckEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCheckEvidence {
    pub integration_id: u64,
    pub app_slug: String,
    pub app_owner: String,
    pub check_run_ref: String,
    pub check_suite_ref: String,
    pub workflow_id: u64,
    pub workflow_run_ref: String,
    pub run_attempt: u64,
    pub workflow_path: String,
    pub workflow_blob_sha_at_canonical_base: String,
    pub job_name: String,
    pub job_ref: String,
    pub event: String,
    pub conclusion: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckRunSnapshot {
    pub repository: String,
    pub id: u64,
    pub name: String,
    pub head_sha: String,
    pub conclusion: String,
    pub integration_id: u64,
    pub app_slug: String,
    pub app_owner: String,
    pub check_suite_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowRunSnapshot {
    pub repository: String,
    pub id: u64,
    pub workflow_id: u64,
    pub workflow_path: String,
    pub head_sha: String,
    pub base_sha: String,
    pub event: String,
    pub run_attempt: u64,
    pub check_suite_id: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobSnapshot {
    pub repository: String,
    pub id: u64,
    pub workflow_run_id: u64,
    pub check_run_id: u64,
    pub head_sha: String,
    pub name: String,
    pub conclusion: String,
}

pub trait GitHubProvenanceResolver {
    fn check_runs(
        &self,
        repository: &str,
        head_sha: &str,
        context: &str,
    ) -> Result<Vec<CheckRunSnapshot>, GitHubProvenanceError>;

    fn workflow_run(
        &self,
        repository: &str,
        workflow_run_id: u64,
    ) -> Result<Option<WorkflowRunSnapshot>, GitHubProvenanceError>;

    fn job(
        &self,
        repository: &str,
        job_id: u64,
    ) -> Result<Option<JobSnapshot>, GitHubProvenanceError>;

    fn canonical_workflow_blob(
        &self,
        repository: &str,
        base_sha: &str,
        workflow_path: &str,
    ) -> Result<Option<String>, GitHubProvenanceError>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VerifiedRequiredChecks {
    pub schema: String,
    pub repository: String,
    pub head_sha: String,
    pub base_sha: String,
    pub checks: BTreeMap<String, VerifiedRequiredCheck>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct VerifiedRequiredCheck {
    pub check_run_id: u64,
    pub check_suite_id: u64,
    pub workflow_run_id: u64,
    pub job_id: u64,
}

pub fn parse_required_check_policy(
    instance_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<RequiredCheckPolicy, GitHubProvenanceError> {
    verify_schema_bytes(
        schema_bytes,
        POLICY_SCHEMA_URL,
        POLICY_SCHEMA_GIT_BLOB_SHA,
        "required-check policy",
    )?;
    let value = parse_json_no_duplicates(instance_bytes)?;
    let policy: RequiredCheckPolicy = serde_json::from_value(value)?;
    validate_policy(&policy)?;
    Ok(policy)
}

pub fn parse_required_check_provenance(
    instance_bytes: &[u8],
    schema_bytes: &[u8],
) -> Result<RequiredCheckProvenance, GitHubProvenanceError> {
    verify_schema_bytes(
        schema_bytes,
        PROVENANCE_SCHEMA_URL,
        PROVENANCE_SCHEMA_GIT_BLOB_SHA,
        "required-check provenance",
    )?;
    let value = parse_json_no_duplicates(instance_bytes)?;
    let provenance: RequiredCheckProvenance = serde_json::from_value(value)?;
    validate_provenance_shape(&provenance)?;
    Ok(provenance)
}

pub fn verify_required_checks_parsed<R: GitHubProvenanceResolver>(
    policy: &RequiredCheckPolicy,
    policy_bytes: &[u8],
    provenance: &RequiredCheckProvenance,
    resolver: &R,
) -> Result<VerifiedRequiredChecks, GitHubProvenanceError> {
    validate_policy(policy)?;
    validate_provenance_shape(provenance)?;

    if provenance.repository != policy.repository {
        return contract_error(format!(
            "provenance repository {} differs from policy repository {}",
            provenance.repository, policy.repository
        ));
    }
    let expected_policy_digest = sha256_hex(policy_bytes);
    if provenance.policy_sha256 != expected_policy_digest {
        return contract_error("required-check provenance policy_sha256 does not bind exact policy bytes");
    }

    let bindings = [
        (
            "assurance-proof",
            &policy.checks[0],
            &provenance.checks.assurance_proof,
        ),
        ("rust", &policy.checks[1], &provenance.checks.rust),
        ("scorecard", &policy.checks[2], &provenance.checks.scorecard),
    ];

    let mut check_run_ids = BTreeSet::new();
    let mut check_suite_ids = BTreeSet::new();
    let mut workflow_run_ids = BTreeSet::new();
    let mut job_ids = BTreeSet::new();
    let mut verified = BTreeMap::new();

    for (context, policy_entry, evidence) in bindings {
        let ids = verify_one(
            context,
            policy,
            policy_entry,
            provenance,
            evidence,
            resolver,
        )?;
        if !check_run_ids.insert(ids.check_run_id) {
            return contract_error(format!("duplicate check-run id {}", ids.check_run_id));
        }
        if !check_suite_ids.insert(ids.check_suite_id) {
            return contract_error(format!("duplicate check-suite id {}", ids.check_suite_id));
        }
        if !workflow_run_ids.insert(ids.workflow_run_id) {
            return contract_error(format!("duplicate workflow-run id {}", ids.workflow_run_id));
        }
        if !job_ids.insert(ids.job_id) {
            return contract_error(format!("duplicate job id {}", ids.job_id));
        }
        verified.insert(context.to_owned(), ids);
    }

    Ok(VerifiedRequiredChecks {
        schema: "commandf.af02-required-check-verification/v1".to_owned(),
        repository: provenance.repository.clone(),
        head_sha: provenance.head_sha.clone(),
        base_sha: provenance.base_sha.clone(),
        checks: verified,
    })
}

fn verify_one<R: GitHubProvenanceResolver>(
    context: &str,
    policy: &RequiredCheckPolicy,
    policy_entry: &RequiredCheckPolicyEntry,
    provenance: &RequiredCheckProvenance,
    evidence: &RequiredCheckEvidence,
    resolver: &R,
) -> Result<VerifiedRequiredCheck, GitHubProvenanceError> {
    if policy_entry.context != context {
        return contract_error(format!(
            "policy context {} is not in frozen position for {context}",
            policy_entry.context
        ));
    }
    if evidence.integration_id != policy.app.id
        || evidence.app_slug != policy.app.slug
        || evidence.app_owner != policy.app.owner
    {
        return contract_error(format!("{context} evidence does not bind the frozen GitHub app"));
    }
    if evidence.workflow_id != policy_entry.workflow_id
        || evidence.workflow_path != policy_entry.workflow_path
        || evidence.workflow_blob_sha_at_canonical_base != policy_entry.workflow_blob_sha
        || evidence.job_name != policy_entry.job_name
        || evidence.event != policy.event
        || evidence.conclusion != SUCCESS
    {
        return contract_error(format!("{context} evidence differs from required-check policy"));
    }
    if evidence.run_attempt == 0 {
        return contract_error(format!("{context} run_attempt must be positive"));
    }

    let check_run_id = parse_context_ref(context, &evidence.check_run_ref, "check_run_ref")?;
    let check_suite_id =
        parse_context_ref(context, &evidence.check_suite_ref, "check_suite_ref")?;
    let workflow_run_id =
        parse_context_ref(context, &evidence.workflow_run_ref, "workflow_run_ref")?;
    let job_id = parse_context_ref(context, &evidence.job_ref, "job_ref")?;

    let candidates = resolver.check_runs(&policy.repository, &provenance.head_sha, context)?;
    let matching = candidates
        .iter()
        .filter(|candidate| candidate.name == context)
        .collect::<Vec<_>>();
    if matching.len() != 1 {
        return contract_error(format!(
            "{context} must resolve to exactly one matching check-run; found {}",
            matching.len()
        ));
    }
    let check_run = matching[0];
    if check_run.id != check_run_id
        || check_run.repository != policy.repository
        || check_run.head_sha != provenance.head_sha
        || check_run.conclusion != SUCCESS
        || check_run.integration_id != policy.app.id
        || check_run.app_slug != policy.app.slug
        || check_run.app_owner != policy.app.owner
        || check_run.check_suite_id != check_suite_id
    {
        return contract_error(format!("{context} check-run API identity mismatch"));
    }

    let workflow_run = resolver
        .workflow_run(&policy.repository, workflow_run_id)?
        .ok_or_else(|| {
            GitHubProvenanceError::Contract(format!(
                "{context} workflow-run {workflow_run_id} is absent"
            ))
        })?;
    if workflow_run.repository != policy.repository
        || workflow_run.id != workflow_run_id
        || workflow_run.workflow_id != policy_entry.workflow_id
        || workflow_run.workflow_path != policy_entry.workflow_path
        || workflow_run.head_sha != provenance.head_sha
        || workflow_run.base_sha != provenance.base_sha
        || workflow_run.event != policy.event
        || workflow_run.run_attempt != evidence.run_attempt
        || workflow_run.check_suite_id != check_suite_id
    {
        return contract_error(format!("{context} workflow-run API identity mismatch"));
    }

    let job = resolver.job(&policy.repository, job_id)?.ok_or_else(|| {
        GitHubProvenanceError::Contract(format!("{context} job {job_id} is absent"))
    })?;
    if job.repository != policy.repository
        || job.id != job_id
        || job.workflow_run_id != workflow_run_id
        || job.check_run_id != check_run_id
        || job.head_sha != provenance.head_sha
        || job.name != policy_entry.job_name
        || job.conclusion != SUCCESS
    {
        return contract_error(format!("{context} job API identity mismatch"));
    }

    let canonical_blob = resolver
        .canonical_workflow_blob(
            &policy.repository,
            &provenance.base_sha,
            &policy_entry.workflow_path,
        )?
        .ok_or_else(|| {
            GitHubProvenanceError::Contract(format!(
                "{context} canonical-base workflow path is absent"
            ))
        })?;
    if canonical_blob != policy_entry.workflow_blob_sha
        || canonical_blob != evidence.workflow_blob_sha_at_canonical_base
    {
        return contract_error(format!(
            "{context} canonical-base workflow blob does not match frozen policy"
        ));
    }

    Ok(VerifiedRequiredCheck {
        check_run_id,
        check_suite_id,
        workflow_run_id,
        job_id,
    })
}

fn validate_policy(policy: &RequiredCheckPolicy) -> Result<(), GitHubProvenanceError> {
    if policy.schema != POLICY_SCHEMA_ID
        || policy.repository != REPOSITORY
        || policy.app.id != APP_ID
        || policy.app.slug != APP_SLUG
        || policy.app.owner != APP_OWNER
        || policy.event != EVENT
    {
        return contract_error("required-check policy top-level frozen identity mismatch");
    }
    if policy.checks.len() != 3 {
        return contract_error("required-check policy must contain exactly three checks");
    }
    let expected = [
        (
            "assurance-proof",
            343_599_979,
            ".github/workflows/af01-assurance-proof.yml",
            "f41045416803fac9ed22aeacf0e38c0fc2a6289f",
            "assurance-proof",
        ),
        (
            "rust",
            333_259_855,
            ".github/workflows/ci.yml",
            "41f33c1aa0f458363cc92bbc206df4fe203b32ef",
            "rust",
        ),
        (
            "scorecard",
            343_592_713,
            ".github/workflows/af01-scorecard.yml",
            "69aa7d808d7b13b3a7cac21a71a09b55c432794b",
            "scorecard",
        ),
    ];
    for (entry, expected) in policy.checks.iter().zip(expected) {
        if entry.context != expected.0
            || entry.workflow_id != expected.1
            || entry.workflow_path != expected.2
            || entry.workflow_blob_sha != expected.3
            || entry.job_name != expected.4
        {
            return contract_error(format!(
                "required-check policy entry {} differs from planning-frozen identity",
                entry.context
            ));
        }
    }
    Ok(())
}

fn validate_provenance_shape(
    provenance: &RequiredCheckProvenance,
) -> Result<(), GitHubProvenanceError> {
    if provenance.schema != PROVENANCE_SCHEMA_ID || provenance.repository != REPOSITORY {
        return contract_error("required-check provenance top-level identity mismatch");
    }
    validate_git_sha(&provenance.head_sha, "head_sha")?;
    validate_git_sha(&provenance.base_sha, "base_sha")?;
    validate_sha256(&provenance.policy_sha256, "policy_sha256")?;

    for (context, evidence) in [
        ("assurance-proof", &provenance.checks.assurance_proof),
        ("rust", &provenance.checks.rust),
        ("scorecard", &provenance.checks.scorecard),
    ] {
        if evidence.integration_id != APP_ID
            || evidence.app_slug != APP_SLUG
            || evidence.app_owner != APP_OWNER
            || evidence.event != EVENT
            || evidence.conclusion != SUCCESS
        {
            return contract_error(format!("{context} provenance fixed identity mismatch"));
        }
        validate_git_sha(
            &evidence.workflow_blob_sha_at_canonical_base,
            "workflow_blob_sha_at_canonical_base",
        )?;
        if evidence.run_attempt == 0 {
            return contract_error(format!("{context} run_attempt must be positive"));
        }
        parse_context_ref(context, &evidence.check_run_ref, "check_run_ref")?;
        parse_context_ref(context, &evidence.check_suite_ref, "check_suite_ref")?;
        parse_context_ref(context, &evidence.workflow_run_ref, "workflow_run_ref")?;
        parse_context_ref(context, &evidence.job_ref, "job_ref")?;
    }
    Ok(())
}

fn verify_schema_bytes(
    schema_bytes: &[u8],
    expected_url: &str,
    expected_blob_sha: &str,
    label: &str,
) -> Result<(), GitHubProvenanceError> {
    if git_blob_sha1_hex(schema_bytes) != expected_blob_sha {
        return schema_error(format!(
            "{label} schema bytes do not match the planning-frozen Git blob"
        ));
    }
    let schema = parse_json_no_duplicates(schema_bytes)?;
    if schema.get("$id").and_then(Value::as_str) != Some(expected_url) {
        return schema_error(format!("unexpected planning-frozen {label} schema id"));
    }
    Ok(())
}

fn parse_context_ref(
    context: &str,
    value: &str,
    label: &str,
) -> Result<u64, GitHubProvenanceError> {
    let prefix = format!("{context}:");
    let numeric = value.strip_prefix(&prefix).ok_or_else(|| {
        GitHubProvenanceError::Contract(format!(
            "{context} {label} must use context-prefixed GitHub identity"
        ))
    })?;
    if numeric.is_empty() || numeric.starts_with('0') || !numeric.bytes().all(|byte| byte.is_ascii_digit()) {
        return contract_error(format!("{context} {label} has invalid numeric identity"));
    }
    let id = numeric.parse::<u64>().map_err(|_| {
        GitHubProvenanceError::Contract(format!("{context} {label} numeric identity overflows u64"))
    })?;
    if id == 0 {
        return contract_error(format!("{context} {label} must be positive"));
    }
    Ok(id)
}

fn validate_git_sha(value: &str, label: &str) -> Result<(), GitHubProvenanceError> {
    if value.len() != 40 || !value.bytes().all(is_lower_hex) {
        return contract_error(format!(
            "{label} must be 40 lowercase hexadecimal characters"
        ));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> Result<(), GitHubProvenanceError> {
    if value.len() != 64 || !value.bytes().all(is_lower_hex) {
        return contract_error(format!(
            "{label} must be 64 lowercase hexadecimal characters"
        ));
    }
    Ok(())
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn schema_error<T>(message: impl Into<String>) -> Result<T, GitHubProvenanceError> {
    Err(GitHubProvenanceError::Schema(message.into()))
}

fn contract_error<T>(message: impl Into<String>) -> Result<T, GitHubProvenanceError> {
    Err(GitHubProvenanceError::Contract(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY_BYTES: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/required-check-policy.json"
    );
    const POLICY_SCHEMA_BYTES: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/schemas/af02-required-check-policy-v1.schema.json"
    );
    const PROVENANCE_SCHEMA_BYTES: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/schemas/af02-required-check-provenance-v1.schema.json"
    );
    const HEAD: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const BASE: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[derive(Default)]
    struct TestResolver {
        check_runs: BTreeMap<String, Vec<CheckRunSnapshot>>,
        workflow_runs: BTreeMap<u64, WorkflowRunSnapshot>,
        jobs: BTreeMap<u64, JobSnapshot>,
        blobs: BTreeMap<String, String>,
    }

    impl GitHubProvenanceResolver for TestResolver {
        fn check_runs(
            &self,
            _repository: &str,
            _head_sha: &str,
            context: &str,
        ) -> Result<Vec<CheckRunSnapshot>, GitHubProvenanceError> {
            Ok(self.check_runs.get(context).cloned().unwrap_or_default())
        }

        fn workflow_run(
            &self,
            _repository: &str,
            workflow_run_id: u64,
        ) -> Result<Option<WorkflowRunSnapshot>, GitHubProvenanceError> {
            Ok(self.workflow_runs.get(&workflow_run_id).cloned())
        }

        fn job(
            &self,
            _repository: &str,
            job_id: u64,
        ) -> Result<Option<JobSnapshot>, GitHubProvenanceError> {
            Ok(self.jobs.get(&job_id).cloned())
        }

        fn canonical_workflow_blob(
            &self,
            _repository: &str,
            _base_sha: &str,
            workflow_path: &str,
        ) -> Result<Option<String>, GitHubProvenanceError> {
            Ok(self.blobs.get(workflow_path).cloned())
        }
    }

    fn valid_policy() -> RequiredCheckPolicy {
        parse_required_check_policy(POLICY_BYTES, POLICY_SCHEMA_BYTES).unwrap()
    }

    fn evidence(context: &str, entry: &RequiredCheckPolicyEntry, seed: u64) -> RequiredCheckEvidence {
        RequiredCheckEvidence {
            integration_id: APP_ID,
            app_slug: APP_SLUG.to_owned(),
            app_owner: APP_OWNER.to_owned(),
            check_run_ref: format!("{context}:{}", seed + 1),
            check_suite_ref: format!("{context}:{}", seed + 2),
            workflow_id: entry.workflow_id,
            workflow_run_ref: format!("{context}:{}", seed + 3),
            run_attempt: 1,
            workflow_path: entry.workflow_path.clone(),
            workflow_blob_sha_at_canonical_base: entry.workflow_blob_sha.clone(),
            job_name: entry.job_name.clone(),
            job_ref: format!("{context}:{}", seed + 4),
            event: EVENT.to_owned(),
            conclusion: SUCCESS.to_owned(),
        }
    }

    fn valid_provenance(policy: &RequiredCheckPolicy) -> RequiredCheckProvenance {
        RequiredCheckProvenance {
            schema: PROVENANCE_SCHEMA_ID.to_owned(),
            repository: REPOSITORY.to_owned(),
            head_sha: HEAD.to_owned(),
            base_sha: BASE.to_owned(),
            policy_sha256: sha256_hex(POLICY_BYTES),
            checks: RequiredCheckEvidenceSet {
                assurance_proof: evidence("assurance-proof", &policy.checks[0], 100),
                rust: evidence("rust", &policy.checks[1], 200),
                scorecard: evidence("scorecard", &policy.checks[2], 300),
            },
        }
    }

    fn resolver(policy: &RequiredCheckPolicy, provenance: &RequiredCheckProvenance) -> TestResolver {
        let mut resolver = TestResolver::default();
        for (context, entry, evidence) in [
            ("assurance-proof", &policy.checks[0], &provenance.checks.assurance_proof),
            ("rust", &policy.checks[1], &provenance.checks.rust),
            ("scorecard", &policy.checks[2], &provenance.checks.scorecard),
        ] {
            let check_run_id = parse_context_ref(context, &evidence.check_run_ref, "check").unwrap();
            let check_suite_id = parse_context_ref(context, &evidence.check_suite_ref, "suite").unwrap();
            let workflow_run_id = parse_context_ref(context, &evidence.workflow_run_ref, "run").unwrap();
            let job_id = parse_context_ref(context, &evidence.job_ref, "job").unwrap();
            resolver.check_runs.insert(
                context.to_owned(),
                vec![CheckRunSnapshot {
                    repository: REPOSITORY.to_owned(),
                    id: check_run_id,
                    name: context.to_owned(),
                    head_sha: HEAD.to_owned(),
                    conclusion: SUCCESS.to_owned(),
                    integration_id: APP_ID,
                    app_slug: APP_SLUG.to_owned(),
                    app_owner: APP_OWNER.to_owned(),
                    check_suite_id,
                }],
            );
            resolver.workflow_runs.insert(
                workflow_run_id,
                WorkflowRunSnapshot {
                    repository: REPOSITORY.to_owned(),
                    id: workflow_run_id,
                    workflow_id: entry.workflow_id,
                    workflow_path: entry.workflow_path.clone(),
                    head_sha: HEAD.to_owned(),
                    base_sha: BASE.to_owned(),
                    event: EVENT.to_owned(),
                    run_attempt: 1,
                    check_suite_id,
                },
            );
            resolver.jobs.insert(
                job_id,
                JobSnapshot {
                    repository: REPOSITORY.to_owned(),
                    id: job_id,
                    workflow_run_id,
                    check_run_id,
                    head_sha: HEAD.to_owned(),
                    name: entry.job_name.clone(),
                    conclusion: SUCCESS.to_owned(),
                },
            );
            resolver
                .blobs
                .insert(entry.workflow_path.clone(), entry.workflow_blob_sha.clone());
        }
        resolver
    }

    #[test]
    fn accepts_exact_cross_bound_github_provenance() {
        let policy = valid_policy();
        let provenance = valid_provenance(&policy);
        let resolver = resolver(&policy, &provenance);

        let verified = verify_required_checks_parsed(
            &policy,
            POLICY_BYTES,
            &provenance,
            &resolver,
        )
        .unwrap();

        assert_eq!(verified.head_sha, HEAD);
        assert_eq!(verified.checks.len(), 3);
    }

    #[test]
    fn rejects_wrong_app_even_with_correct_app_id() {
        let policy = valid_policy();
        let mut provenance = valid_provenance(&policy);
        provenance.checks.rust.app_slug = "forged-actions".to_owned();
        let resolver = resolver(&policy, &provenance);
        let error = verify_required_checks_parsed(&policy, POLICY_BYTES, &provenance, &resolver)
            .unwrap_err();
        assert!(error.to_string().contains("fixed identity mismatch"));
    }

    #[test]
    fn rejects_wrong_repository() {
        let policy = valid_policy();
        let mut provenance = valid_provenance(&policy);
        provenance.repository = "TheHalfMoon/other".to_owned();
        let resolver = resolver(&policy, &provenance);
        let error = verify_required_checks_parsed(&policy, POLICY_BYTES, &provenance, &resolver)
            .unwrap_err();
        assert!(error.to_string().contains("top-level identity mismatch"));
    }

    #[test]
    fn rejects_wrong_exact_head_from_github() {
        let policy = valid_policy();
        let provenance = valid_provenance(&policy);
        let mut resolver = resolver(&policy, &provenance);
        resolver.check_runs.get_mut("rust").unwrap()[0].head_sha = BASE.to_owned();
        let error = verify_required_checks_parsed(&policy, POLICY_BYTES, &provenance, &resolver)
            .unwrap_err();
        assert!(error.to_string().contains("check-run API identity mismatch"));
    }

    #[test]
    fn rejects_wrong_canonical_base_workflow_blob() {
        let policy = valid_policy();
        let provenance = valid_provenance(&policy);
        let mut resolver = resolver(&policy, &provenance);
        resolver.blobs.insert(
            ".github/workflows/ci.yml".to_owned(),
            "cccccccccccccccccccccccccccccccccccccccc".to_owned(),
        );
        let error = verify_required_checks_parsed(&policy, POLICY_BYTES, &provenance, &resolver)
            .unwrap_err();
        assert!(error.to_string().contains("canonical-base workflow blob"));
    }

    #[test]
    fn rejects_wrong_job_binding() {
        let policy = valid_policy();
        let provenance = valid_provenance(&policy);
        let mut resolver = resolver(&policy, &provenance);
        let job_id = parse_context_ref("rust", &provenance.checks.rust.job_ref, "job").unwrap();
        resolver.jobs.get_mut(&job_id).unwrap().name = "different-job".to_owned();
        let error = verify_required_checks_parsed(&policy, POLICY_BYTES, &provenance, &resolver)
            .unwrap_err();
        assert!(error.to_string().contains("job API identity mismatch"));
    }

    #[test]
    fn rejects_duplicate_workflow_run_ids_across_contexts() {
        let policy = valid_policy();
        let mut provenance = valid_provenance(&policy);
        provenance.checks.scorecard.workflow_run_ref = "scorecard:203".to_owned();
        let resolver = resolver(&policy, &provenance);
        let error = verify_required_checks_parsed(&policy, POLICY_BYTES, &provenance, &resolver)
            .unwrap_err();
        assert!(error.to_string().contains("workflow-run API identity mismatch") || error.to_string().contains("duplicate workflow-run id"));
    }

    #[test]
    fn rejects_duplicate_matching_check_runs() {
        let policy = valid_policy();
        let provenance = valid_provenance(&policy);
        let mut resolver = resolver(&policy, &provenance);
        let duplicate = resolver.check_runs["rust"][0].clone();
        resolver.check_runs.get_mut("rust").unwrap().push(duplicate);
        let error = verify_required_checks_parsed(&policy, POLICY_BYTES, &provenance, &resolver)
            .unwrap_err();
        assert!(error.to_string().contains("exactly one matching check-run"));
    }

    #[test]
    fn rejects_policy_digest_mismatch() {
        let policy = valid_policy();
        let mut provenance = valid_provenance(&policy);
        provenance.policy_sha256 = "d".repeat(64);
        let resolver = resolver(&policy, &provenance);
        let error = verify_required_checks_parsed(&policy, POLICY_BYTES, &provenance, &resolver)
            .unwrap_err();
        assert!(error.to_string().contains("policy_sha256"));
    }
}
