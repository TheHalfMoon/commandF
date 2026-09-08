pub mod base_gate;
pub mod enforcement;
pub mod input_guard;
pub mod semantic;

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use commandf_af02_verifier::authority::{project_authority, Cf06Source};
use commandf_af02_verifier::canonical::{canonical_json_bytes, parse_json_no_duplicates};
use commandf_af02_verifier::corpus::{
    parse_assertion_registry, parse_corpus_manifest, validate_corpus_and_assertions,
    verify_fixture_bytes,
};
use commandf_af02_verifier::resource::{parse_resource_policy, run_bounded};
use commandf_af02_verifier::retained::{
    locator_plan, project_retained, validate_and_parse, verify_artifacts, verify_workflow_run,
};
use commandf_af02_verifier::surface::{
    discover_tracked_rust_sources, parse_surface_policy, scan_surface,
};
use commandf_af02_verifier::surface_proof::{canonical_surface_proof_bytes, prove_surface};
use commandf_af02_verifier::waiver::parse_waiver_policy;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityInput {
    captured_from_main_sha: String,
    captured_from_main_tree: String,
    assurance_ruleset_path: PathBuf,
    review_ruleset_path: PathBuf,
    retained_sources_path: PathBuf,
    retained_schema_path: PathBuf,
    retained_manifest_path: PathBuf,
    retained_donor_path: PathBuf,
    retained_workflow_run_path: PathBuf,
    retained_artifacts_path: PathBuf,
    cf06_oracle_model: SourcePath,
    cf06_donor: SourcePath,
    cf06_workflow: SourcePath,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcePath {
    path: String,
    git_blob_sha: String,
    local_path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProcessEvidenceInput {
    binary_sha256: String,
    expected_binary_sha256: String,
    cargo_lock_blob: String,
    expected_cargo_lock_blob: String,
    unprivileged: bool,
    cgroup_v2: bool,
    wall_timeout_enforced: bool,
    memory_limit_enforced: bool,
    pid_limit_enforced: bool,
    network_none: bool,
    root_read_only: bool,
    stdout_observed: u64,
    stdout_limit: u64,
    stdout_exceeded: bool,
    stderr_observed: u64,
    stderr_limit: u64,
    stderr_exceeded: bool,
    termination: String,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("commandf-af02-verifier: {error}");
        std::process::exit(2);
    }
}

fn write_serialized<T: Serialize>(value: &T) -> Result<(), Box<dyn std::error::Error>> {
    let value = serde_json::to_value(value)?;
    Write::write_all(&mut std::io::stdout().lock(), &canonical_json_bytes(&value)?)?;
    Ok(())
}

fn write_value(value: Value) -> Result<(), Box<dyn std::error::Error>> {
    Write::write_all(&mut std::io::stdout().lock(), &canonical_json_bytes(&value)?)?;
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or("missing entrypoint")?;
    match command.as_str() {
        "project-retained" => {
            let retained_path = PathBuf::from(args.next().ok_or("missing retained authority path")?);
            let schema_path = PathBuf::from(args.next().ok_or("missing retained schema path")?);
            if args.next().is_some() {
                return Err("project-retained accepts exactly two paths".into());
            }
            let retained = validate_and_parse(&fs::read(retained_path)?, &fs::read(schema_path)?)?;
            write_serialized(&locator_plan(&retained)?)?;
        }
        "project-authority" => {
            let input_path = PathBuf::from(args.next().ok_or("missing authority input path")?);
            if args.next().is_some() {
                return Err("project-authority accepts exactly one input path".into());
            }
            let input_value = parse_json_no_duplicates(&fs::read(input_path)?)?;
            let input: AuthorityInput = serde_json::from_value(input_value)?;
            let retained_bytes = fs::read(&input.retained_sources_path)?;
            let retained_schema_bytes = fs::read(&input.retained_schema_path)?;
            let retained = validate_and_parse(&retained_bytes, &retained_schema_bytes)?;

            let run: Value = parse_json_no_duplicates(&fs::read(&input.retained_workflow_run_path)?)?;
            verify_workflow_run(&retained, &run)?;
            let artifacts: Value = parse_json_no_duplicates(&fs::read(&input.retained_artifacts_path)?)?;
            verify_artifacts(&retained, &artifacts)?;

            let retained_projection = project_retained(
                &retained,
                &fs::read(&input.retained_manifest_path)?,
                &fs::read(&input.retained_donor_path)?,
            )?;
            let assurance: Value = parse_json_no_duplicates(&fs::read(&input.assurance_ruleset_path)?)?;
            let review: Value = parse_json_no_duplicates(&fs::read(&input.review_ruleset_path)?)?;
            let oracle_model = fs::read(&input.cf06_oracle_model.local_path)?;
            let donor = fs::read(&input.cf06_donor.local_path)?;
            let workflow = fs::read(&input.cf06_workflow.local_path)?;
            let baseline = project_authority(
                &input.captured_from_main_sha,
                &input.captured_from_main_tree,
                &assurance,
                &review,
                [
                    Cf06Source {
                        path: &input.cf06_oracle_model.path,
                        git_blob_sha: &input.cf06_oracle_model.git_blob_sha,
                        bytes: &oracle_model,
                    },
                    Cf06Source {
                        path: &input.cf06_donor.path,
                        git_blob_sha: &input.cf06_donor.git_blob_sha,
                        bytes: &donor,
                    },
                    Cf06Source {
                        path: &input.cf06_workflow.path,
                        git_blob_sha: &input.cf06_workflow.git_blob_sha,
                        bytes: &workflow,
                    },
                ],
                retained_projection,
            )?;
            write_serialized(&baseline)?;
        }
        "parse-surface-policy" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing surface policy path")?);
            if args.next().is_some() {
                return Err("parse-surface-policy accepts exactly one path".into());
            }
            write_serialized(&parse_surface_policy(&fs::read(policy_path)?)?)?;
        }
        "scan-surface" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing surface policy path")?);
            let repo_root = PathBuf::from(args.next().ok_or("missing repository root")?);
            if args.next().is_some() {
                return Err("scan-surface accepts exactly a policy path and repository root".into());
            }
            let policy = parse_surface_policy(&fs::read(policy_path)?)?;
            let sources = discover_tracked_rust_sources(&repo_root)?;
            write_serialized(&scan_surface(&policy, &sources)?)?;
        }
        "prove-surface" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing surface policy path")?);
            let exclusion_policy_path = PathBuf::from(args.next().ok_or("missing exclusion policy path")?);
            let source_repo_root = PathBuf::from(args.next().ok_or("missing source repository root")?);
            if args.next().is_some() {
                return Err(
                    "prove-surface accepts exactly a surface policy path, exclusion policy path, and source repository root"
                        .into(),
                );
            }
            let evidence = prove_surface(
                &fs::read(policy_path)?,
                &fs::read(exclusion_policy_path)?,
                &source_repo_root,
            )?;
            Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_surface_proof_bytes(&evidence)?,
            )?;
        }
        "parse-resource-policy" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing resource policy path")?);
            if args.next().is_some() {
                return Err("parse-resource-policy accepts exactly one path".into());
            }
            write_serialized(&parse_resource_policy(&fs::read(policy_path)?)?)?;
        }
        "run-bounded" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing resource policy path")?);
            let source_dir = PathBuf::from(args.next().ok_or("missing source directory")?);
            let output_dir = PathBuf::from(args.next().ok_or("missing output directory")?);
            if args.next().as_deref() != Some("--") {
                return Err("run-bounded requires `--` before the bounded command".into());
            }
            let bounded_command = args.collect::<Vec<_>>();
            if bounded_command.is_empty() {
                return Err("run-bounded requires a bounded command".into());
            }
            let policy = parse_resource_policy(&fs::read(policy_path)?)?;
            write_serialized(&run_bounded(&policy, &source_dir, &output_dir, &bounded_command)?)?;
        }
        "parse-corpus" => {
            let corpus_path = PathBuf::from(args.next().ok_or("missing corpus manifest path")?);
            let schema_path = PathBuf::from(args.next().ok_or("missing corpus schema path")?);
            if args.next().is_some() {
                return Err("parse-corpus accepts exactly a corpus path and schema path".into());
            }
            write_serialized(&parse_corpus_manifest(
                &fs::read(corpus_path)?,
                &fs::read(schema_path)?,
            )?)?;
        }
        "parse-assertions" => {
            let assertion_path = PathBuf::from(args.next().ok_or("missing assertion registry path")?);
            if args.next().is_some() {
                return Err("parse-assertions accepts exactly one assertion registry path".into());
            }
            write_serialized(&parse_assertion_registry(&fs::read(assertion_path)?)?)?;
        }
        "validate-corpus-assertions" => {
            let corpus_path = PathBuf::from(args.next().ok_or("missing corpus manifest path")?);
            let schema_path = PathBuf::from(args.next().ok_or("missing corpus schema path")?);
            let assertion_path = PathBuf::from(args.next().ok_or("missing assertion registry path")?);
            let surface_policy_path = PathBuf::from(args.next().ok_or("missing surface policy path")?);
            let repo_root = PathBuf::from(args.next().ok_or("missing repository root")?);
            if args.next().is_some() {
                return Err(
                    "validate-corpus-assertions accepts exactly corpus, corpus schema, assertion registry, surface policy, and repository root paths"
                        .into(),
                );
            }
            let (corpus, assertions) = validate_corpus_and_assertions(
                &fs::read(corpus_path)?,
                &fs::read(schema_path)?,
                &fs::read(assertion_path)?,
                &fs::read(surface_policy_path)?,
            )?;
            for entry in &corpus.entries {
                verify_fixture_bytes(entry, &fs::read(repo_root.join(&entry.fixture_path))?)?;
            }
            write_value(serde_json::json!({
                "assertion_count": assertions.entries.len(),
                "scenario_count": corpus.entries.len(),
                "schema": "commandf.af02-corpus-assertion-validation/v1"
            }))?;
        }
        "parse-waiver-policy" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing waiver policy path")?);
            if args.next().is_some() {
                return Err("parse-waiver-policy accepts exactly one path".into());
            }
            write_serialized(&parse_waiver_policy(&fs::read(policy_path)?)?)?;
        }
        "parse-enforcement-inventory" => {
            let inventory_path = PathBuf::from(args.next().ok_or("missing enforcement inventory path")?);
            let schema_path = PathBuf::from(args.next().ok_or("missing enforcement inventory schema path")?);
            if args.next().is_some() {
                return Err(
                    "parse-enforcement-inventory accepts exactly an inventory path and schema path"
                        .into(),
                );
            }
            write_serialized(&enforcement::parse_frozen_enforcement_inventory(
                &fs::read(inventory_path)?,
                &fs::read(schema_path)?,
            )?)?;
        }
        "guard-inputs" => {
            let candidate_root = PathBuf::from(args.next().ok_or("missing candidate root")?);
            let remaining = args.collect::<Vec<_>>();
            if remaining.is_empty() || remaining.len() % 2 != 0 {
                return Err(
                    "guard-inputs requires candidate root followed by one or more <json|yaml> <relative-path> pairs"
                        .into(),
                );
            }
            let mut inputs = Vec::with_capacity(remaining.len() / 2);
            for pair in remaining.chunks_exact(2) {
                inputs.push(input_guard::CandidateInput {
                    format: input_guard::CandidateFormat::parse(&pair[0])?,
                    relative_path: PathBuf::from(&pair[1]),
                });
            }
            write_serialized(&input_guard::guard_inputs(&candidate_root, &inputs)?)?;
        }
        "verify-semantic-contract" => {
            let contract_path = PathBuf::from(args.next().ok_or("missing semantic contract path")?);
            let schema_path = PathBuf::from(args.next().ok_or("missing semantic contract schema path")?);
            if args.next().is_some() {
                return Err(
                    "verify-semantic-contract accepts exactly a contract path and schema path".into(),
                );
            }
            let coverage = semantic::validate_semantic_contract(
                &fs::read(contract_path)?,
                &fs::read(schema_path)?,
            )?;
            write_value(serde_json::json!({
                "algorithm_count": coverage.algorithm_count,
                "negative_fixture_count": coverage.negative_fixture_count,
                "schema": "commandf.af02-semantic-contract-validation/v1"
            }))?;
        }
        "verify-pr" => {
            let base_root = PathBuf::from(args.next().ok_or("missing canonical-base root")?);
            let candidate_root = PathBuf::from(args.next().ok_or("missing candidate root")?);
            let gate_input = PathBuf::from(args.next().ok_or("missing base-gate input path")?);
            if args.next().is_some() {
                return Err(
                    "verify-pr accepts exactly canonical-base root, candidate root, and base-gate input path"
                        .into(),
                );
            }
            let proof = base_gate::verify_pr(&base_root, &candidate_root, &fs::read(gate_input)?)?;
            write_serialized(&proof)?;
        }
        "validate-input-process-evidence" => {
            let evidence_path = PathBuf::from(args.next().ok_or("missing process-evidence path")?);
            if args.next().is_some() {
                return Err("validate-input-process-evidence accepts exactly one path".into());
            }
            let value = parse_json_no_duplicates(&fs::read(evidence_path)?)?;
            let evidence: ProcessEvidenceInput = serde_json::from_value(value)?;
            semantic::validate_input_process_enforcement(&semantic::ProcessEvidence {
                binary_sha256: evidence.binary_sha256,
                expected_binary_sha256: evidence.expected_binary_sha256,
                cargo_lock_blob: evidence.cargo_lock_blob,
                expected_cargo_lock_blob: evidence.expected_cargo_lock_blob,
                unprivileged: evidence.unprivileged,
                cgroup_v2: evidence.cgroup_v2,
                wall_timeout_enforced: evidence.wall_timeout_enforced,
                memory_limit_enforced: evidence.memory_limit_enforced,
                pid_limit_enforced: evidence.pid_limit_enforced,
                network_none: evidence.network_none,
                root_read_only: evidence.root_read_only,
                stdout_observed: evidence.stdout_observed,
                stdout_limit: evidence.stdout_limit,
                stdout_exceeded: evidence.stdout_exceeded,
                stderr_observed: evidence.stderr_observed,
                stderr_limit: evidence.stderr_limit,
                stderr_exceeded: evidence.stderr_exceeded,
                termination: evidence.termination,
            })?;
            write_value(serde_json::json!({
                "result": "PASS",
                "schema": "commandf.af02-input-process-validation/v1"
            }))?;
        }
        other => return Err(format!("unknown entrypoint {other}").into()),
    }
    Ok(())
}
