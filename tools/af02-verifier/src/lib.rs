use std::collections::BTreeSet;

pub mod authority;
pub mod canonical;
pub mod corpus;
mod github_provenance;
pub mod resource;
pub mod retained;
pub mod surface;
pub mod surface_proof;
pub mod waiver;

pub use github_provenance::{
    CheckRunSnapshot, GitHubProvenanceError, GitHubProvenanceResolver, JobSnapshot,
    RequiredCheckApp, RequiredCheckEvidence, RequiredCheckEvidenceSet, RequiredCheckPolicy,
    RequiredCheckPolicyEntry, RequiredCheckProvenance, VerifiedRequiredCheck,
    VerifiedRequiredChecks, WorkflowRunSnapshot, parse_required_check_policy,
    parse_required_check_provenance,
};

pub fn verify_required_checks<R: GitHubProvenanceResolver>(
    policy_bytes: &[u8],
    policy_schema_bytes: &[u8],
    provenance_bytes: &[u8],
    provenance_schema_bytes: &[u8],
    resolver: &R,
) -> Result<VerifiedRequiredChecks, GitHubProvenanceError> {
    let policy = github_provenance::parse_required_check_policy(policy_bytes, policy_schema_bytes)?;
    let provenance = github_provenance::parse_required_check_provenance(
        provenance_bytes,
        provenance_schema_bytes,
    )?;
    verify_required_check_ref_uniqueness(&provenance)?;
    github_provenance::verify_required_checks_parsed(
        &policy,
        policy_bytes,
        &provenance,
        resolver,
    )
}

fn verify_required_check_ref_uniqueness(
    provenance: &RequiredCheckProvenance,
) -> Result<(), GitHubProvenanceError> {
    let mut check_run_ids = BTreeSet::new();
    let mut check_suite_ids = BTreeSet::new();
    let mut workflow_run_ids = BTreeSet::new();
    let mut job_ids = BTreeSet::new();

    for (context, evidence) in [
        ("assurance-proof", &provenance.checks.assurance_proof),
        ("rust", &provenance.checks.rust),
        ("scorecard", &provenance.checks.scorecard),
    ] {
        require_unique_id(
            &mut check_run_ids,
            required_check_ref_id(context, &evidence.check_run_ref, "check_run_ref")?,
            "check-run",
        )?;
        require_unique_id(
            &mut check_suite_ids,
            required_check_ref_id(context, &evidence.check_suite_ref, "check_suite_ref")?,
            "check-suite",
        )?;
        require_unique_id(
            &mut workflow_run_ids,
            required_check_ref_id(context, &evidence.workflow_run_ref, "workflow_run_ref")?,
            "workflow-run",
        )?;
        require_unique_id(
            &mut job_ids,
            required_check_ref_id(context, &evidence.job_ref, "job_ref")?,
            "job",
        )?;
    }

    Ok(())
}

fn require_unique_id(
    ids: &mut BTreeSet<u64>,
    id: u64,
    domain: &str,
) -> Result<(), GitHubProvenanceError> {
    if !ids.insert(id) {
        return Err(GitHubProvenanceError::Contract(format!(
            "duplicate {domain} id {id}"
        )));
    }
    Ok(())
}

fn required_check_ref_id(
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
    if numeric.is_empty()
        || numeric.starts_with('0')
        || !numeric.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(GitHubProvenanceError::Contract(format!(
            "{context} {label} has invalid numeric identity"
        )));
    }
    let id = numeric.parse::<u64>().map_err(|_| {
        GitHubProvenanceError::Contract(format!(
            "{context} {label} numeric identity overflows u64"
        ))
    })?;
    if id == 0 {
        return Err(GitHubProvenanceError::Contract(format!(
            "{context} {label} must be positive"
        )));
    }
    Ok(id)
}

#[cfg(test)]
mod required_check_public_api_tests {
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

    struct NeverResolver;

    impl GitHubProvenanceResolver for NeverResolver {
        fn check_runs(
            &self,
            _repository: &str,
            _head_sha: &str,
            _context: &str,
        ) -> Result<Vec<CheckRunSnapshot>, GitHubProvenanceError> {
            panic!("duplicate-id preflight must run before resolver access")
        }

        fn workflow_run(
            &self,
            _repository: &str,
            _workflow_run_id: u64,
        ) -> Result<Option<WorkflowRunSnapshot>, GitHubProvenanceError> {
            panic!("duplicate-id preflight must run before resolver access")
        }

        fn job(
            &self,
            _repository: &str,
            _job_id: u64,
        ) -> Result<Option<JobSnapshot>, GitHubProvenanceError> {
            panic!("duplicate-id preflight must run before resolver access")
        }

        fn canonical_workflow_blob(
            &self,
            _repository: &str,
            _base_sha: &str,
            _workflow_path: &str,
        ) -> Result<Option<String>, GitHubProvenanceError> {
            panic!("duplicate-id preflight must run before resolver access")
        }
    }

    fn evidence(
        context: &str,
        entry: &RequiredCheckPolicyEntry,
        seed: u64,
    ) -> RequiredCheckEvidence {
        RequiredCheckEvidence {
            integration_id: 15_368,
            app_slug: "github-actions".to_owned(),
            app_owner: "github".to_owned(),
            check_run_ref: format!("{context}:{}", seed + 1),
            check_suite_ref: format!("{context}:{}", seed + 2),
            workflow_id: entry.workflow_id,
            workflow_run_ref: format!("{context}:{}", seed + 3),
            run_attempt: 1,
            workflow_path: entry.workflow_path.clone(),
            workflow_blob_sha_at_canonical_base: entry.workflow_blob_sha.clone(),
            job_name: entry.job_name.clone(),
            job_ref: format!("{context}:{}", seed + 4),
            event: "pull_request".to_owned(),
            conclusion: "success".to_owned(),
        }
    }

    fn valid_provenance(policy: &RequiredCheckPolicy) -> RequiredCheckProvenance {
        RequiredCheckProvenance {
            schema: "commandf.af02-required-check-provenance/v1".to_owned(),
            repository: "TheHalfMoon/commandF".to_owned(),
            head_sha: HEAD.to_owned(),
            base_sha: BASE.to_owned(),
            policy_sha256: crate::canonical::sha256_hex(POLICY_BYTES),
            checks: RequiredCheckEvidenceSet {
                assurance_proof: evidence("assurance-proof", &policy.checks[0], 100),
                rust: evidence("rust", &policy.checks[1], 200),
                scorecard: evidence("scorecard", &policy.checks[2], 300),
            },
        }
    }

    fn assert_duplicate(provenance: RequiredCheckProvenance, expected: &str) {
        let provenance_bytes = serde_json::to_vec(&provenance).unwrap();
        let error = verify_required_checks(
            POLICY_BYTES,
            POLICY_SCHEMA_BYTES,
            &provenance_bytes,
            PROVENANCE_SCHEMA_BYTES,
            &NeverResolver,
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("required-check provenance violation: {expected}")
        );
    }

    #[test]
    fn rejects_duplicate_check_run_id_before_resolver_access() {
        let policy = parse_required_check_policy(POLICY_BYTES, POLICY_SCHEMA_BYTES).unwrap();
        let mut provenance = valid_provenance(&policy);
        provenance.checks.scorecard.check_run_ref = "scorecard:201".to_owned();
        assert_duplicate(provenance, "duplicate check-run id 201");
    }

    #[test]
    fn rejects_duplicate_check_suite_id_before_resolver_access() {
        let policy = parse_required_check_policy(POLICY_BYTES, POLICY_SCHEMA_BYTES).unwrap();
        let mut provenance = valid_provenance(&policy);
        provenance.checks.scorecard.check_suite_ref = "scorecard:202".to_owned();
        assert_duplicate(provenance, "duplicate check-suite id 202");
    }

    #[test]
    fn rejects_duplicate_workflow_run_id_before_resolver_access() {
        let policy = parse_required_check_policy(POLICY_BYTES, POLICY_SCHEMA_BYTES).unwrap();
        let mut provenance = valid_provenance(&policy);
        provenance.checks.scorecard.workflow_run_ref = "scorecard:203".to_owned();
        assert_duplicate(provenance, "duplicate workflow-run id 203");
    }

    #[test]
    fn rejects_duplicate_job_id_before_resolver_access() {
        let policy = parse_required_check_policy(POLICY_BYTES, POLICY_SCHEMA_BYTES).unwrap();
        let mut provenance = valid_provenance(&policy);
        provenance.checks.scorecard.job_ref = "scorecard:204".to_owned();
        assert_duplicate(provenance, "duplicate job id 204");
    }
}
