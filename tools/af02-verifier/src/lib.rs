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
    CheckRunSnapshot, GitHubProvenanceError, JobSnapshot, RequiredCheckApp,
    RequiredCheckEvidence, RequiredCheckEvidenceSet, RequiredCheckPolicy,
    RequiredCheckPolicyEntry, RequiredCheckProvenance, VerifiedRequiredCheck,
    VerifiedRequiredChecks, parse_required_check_policy, parse_required_check_provenance,
};

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

struct ResolverAdapter<'a, R>(&'a R);

impl<R: GitHubProvenanceResolver> github_provenance::GitHubProvenanceResolver
    for ResolverAdapter<'_, R>
{
    fn check_runs(
        &self,
        repository: &str,
        head_sha: &str,
        context: &str,
    ) -> Result<Vec<CheckRunSnapshot>, GitHubProvenanceError> {
        self.0.check_runs(repository, head_sha, context)
    }

    fn workflow_run(
        &self,
        repository: &str,
        workflow_run_id: u64,
    ) -> Result<Option<github_provenance::WorkflowRunSnapshot>, GitHubProvenanceError> {
        let Some(run) = self.0.workflow_run(repository, workflow_run_id)? else {
            return Ok(None);
        };
        if run.conclusion != "success" {
            return Err(GitHubProvenanceError::Contract(format!(
                "workflow-run {} conclusion {} is not success",
                run.id, run.conclusion
            )));
        }
        Ok(Some(github_provenance::WorkflowRunSnapshot {
            repository: run.repository,
            id: run.id,
            workflow_id: run.workflow_id,
            workflow_path: run.workflow_path,
            head_sha: run.head_sha,
            base_sha: run.base_sha,
            event: run.event,
            run_attempt: run.run_attempt,
            check_suite_id: run.check_suite_id,
        }))
    }

    fn job(
        &self,
        repository: &str,
        job_id: u64,
    ) -> Result<Option<JobSnapshot>, GitHubProvenanceError> {
        self.0.job(repository, job_id)
    }

    fn canonical_workflow_blob(
        &self,
        repository: &str,
        base_sha: &str,
        workflow_path: &str,
    ) -> Result<Option<String>, GitHubProvenanceError> {
        self.0
            .canonical_workflow_blob(repository, base_sha, workflow_path)
    }
}

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
    let adapter = ResolverAdapter(resolver);
    github_provenance::verify_required_checks_parsed(
        &policy,
        policy_bytes,
        &provenance,
        &adapter,
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
    use std::collections::BTreeMap;

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

    fn resolver(
        policy: &RequiredCheckPolicy,
        provenance: &RequiredCheckProvenance,
    ) -> TestResolver {
        let mut resolver = TestResolver::default();
        for (context, entry, evidence) in [
            (
                "assurance-proof",
                &policy.checks[0],
                &provenance.checks.assurance_proof,
            ),
            ("rust", &policy.checks[1], &provenance.checks.rust),
            ("scorecard", &policy.checks[2], &provenance.checks.scorecard),
        ] {
            let check_run_id =
                required_check_ref_id(context, &evidence.check_run_ref, "check_run_ref").unwrap();
            let check_suite_id = required_check_ref_id(
                context,
                &evidence.check_suite_ref,
                "check_suite_ref",
            )
            .unwrap();
            let workflow_run_id = required_check_ref_id(
                context,
                &evidence.workflow_run_ref,
                "workflow_run_ref",
            )
            .unwrap();
            let job_id = required_check_ref_id(context, &evidence.job_ref, "job_ref").unwrap();

            resolver.check_runs.insert(
                context.to_owned(),
                vec![CheckRunSnapshot {
                    repository: "TheHalfMoon/commandF".to_owned(),
                    id: check_run_id,
                    name: context.to_owned(),
                    head_sha: HEAD.to_owned(),
                    conclusion: "success".to_owned(),
                    integration_id: 15_368,
                    app_slug: "github-actions".to_owned(),
                    app_owner: "github".to_owned(),
                    check_suite_id,
                }],
            );
            resolver.workflow_runs.insert(
                workflow_run_id,
                WorkflowRunSnapshot {
                    repository: "TheHalfMoon/commandF".to_owned(),
                    id: workflow_run_id,
                    workflow_id: entry.workflow_id,
                    workflow_path: entry.workflow_path.clone(),
                    head_sha: HEAD.to_owned(),
                    base_sha: BASE.to_owned(),
                    event: "pull_request".to_owned(),
                    run_attempt: 1,
                    check_suite_id,
                    conclusion: "success".to_owned(),
                },
            );
            resolver.jobs.insert(
                job_id,
                JobSnapshot {
                    repository: "TheHalfMoon/commandF".to_owned(),
                    id: job_id,
                    workflow_run_id,
                    check_run_id,
                    head_sha: HEAD.to_owned(),
                    name: entry.job_name.clone(),
                    conclusion: "success".to_owned(),
                },
            );
            resolver
                .blobs
                .insert(entry.workflow_path.clone(), entry.workflow_blob_sha.clone());
        }
        resolver
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
    fn accepts_public_verifier_with_successful_workflow_runs() {
        let policy = parse_required_check_policy(POLICY_BYTES, POLICY_SCHEMA_BYTES).unwrap();
        let provenance = valid_provenance(&policy);
        let resolver = resolver(&policy, &provenance);
        let provenance_bytes = serde_json::to_vec(&provenance).unwrap();
        let verified = verify_required_checks(
            POLICY_BYTES,
            POLICY_SCHEMA_BYTES,
            &provenance_bytes,
            PROVENANCE_SCHEMA_BYTES,
            &resolver,
        )
        .unwrap();
        assert_eq!(verified.head_sha, HEAD);
        assert_eq!(verified.checks.len(), 3);
    }

    #[test]
    fn rejects_failed_workflow_run() {
        let policy = parse_required_check_policy(POLICY_BYTES, POLICY_SCHEMA_BYTES).unwrap();
        let provenance = valid_provenance(&policy);
        let mut resolver = resolver(&policy, &provenance);
        resolver.workflow_runs.get_mut(&203).unwrap().conclusion = "failure".to_owned();
        let provenance_bytes = serde_json::to_vec(&provenance).unwrap();
        let error = verify_required_checks(
            POLICY_BYTES,
            POLICY_SCHEMA_BYTES,
            &provenance_bytes,
            PROVENANCE_SCHEMA_BYTES,
            &resolver,
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "required-check provenance violation: workflow-run 203 conclusion failure is not success"
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
