use std::fs;
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
use serde::Deserialize;
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

fn main() {
    if let Err(error) = run() {
        eprintln!("commandf-af02-verifier: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or("missing entrypoint")?;
    match command.as_str() {
        "project-retained" => {
            let retained_path =
                PathBuf::from(args.next().ok_or("missing retained authority path")?);
            let schema_path = PathBuf::from(args.next().ok_or("missing retained schema path")?);
            if args.next().is_some() {
                return Err("project-retained accepts exactly two paths".into());
            }
            let retained_bytes = fs::read(retained_path)?;
            let schema_bytes = fs::read(schema_path)?;
            let retained = validate_and_parse(&retained_bytes, &schema_bytes)?;
            let plan = locator_plan(&retained)?;
            let value = serde_json::to_value(plan)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
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

            let run: Value =
                parse_json_no_duplicates(&fs::read(&input.retained_workflow_run_path)?)?;
            verify_workflow_run(&retained, &run)?;
            let artifacts: Value =
                parse_json_no_duplicates(&fs::read(&input.retained_artifacts_path)?)?;
            verify_artifacts(&retained, &artifacts)?;

            let retained_projection = project_retained(
                &retained,
                &fs::read(&input.retained_manifest_path)?,
                &fs::read(&input.retained_donor_path)?,
            )?;
            let assurance: Value =
                parse_json_no_duplicates(&fs::read(&input.assurance_ruleset_path)?)?;
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
            let value = serde_json::to_value(baseline)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
        }
        "parse-surface-policy" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing surface policy path")?);
            if args.next().is_some() {
                return Err("parse-surface-policy accepts exactly one path".into());
            }
            let policy = parse_surface_policy(&fs::read(policy_path)?)?;
            let value = serde_json::to_value(policy)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
        }
        "scan-surface" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing surface policy path")?);
            let repo_root = PathBuf::from(args.next().ok_or("missing repository root")?);
            if args.next().is_some() {
                return Err(
                    "scan-surface accepts exactly a policy path and repository root".into(),
                );
            }
            let policy = parse_surface_policy(&fs::read(policy_path)?)?;
            let sources = discover_tracked_rust_sources(&repo_root)?;
            let findings = scan_surface(&policy, &sources)?;
            let value = serde_json::to_value(findings)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
        }
        "prove-surface" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing surface policy path")?);
            let exclusion_policy_path =
                PathBuf::from(args.next().ok_or("missing exclusion policy path")?);
            let source_repo_root =
                PathBuf::from(args.next().ok_or("missing source repository root")?);
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
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_surface_proof_bytes(&evidence)?,
            )?;
        }
        "parse-resource-policy" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing resource policy path")?);
            if args.next().is_some() {
                return Err("parse-resource-policy accepts exactly one path".into());
            }
            let policy = parse_resource_policy(&fs::read(policy_path)?)?;
            let value = serde_json::to_value(policy)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
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
            let outcome = run_bounded(&policy, &source_dir, &output_dir, &bounded_command)?;
            let value = serde_json::to_value(outcome)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
        }
        "parse-corpus" => {
            let corpus_path = PathBuf::from(args.next().ok_or("missing corpus manifest path")?);
            let schema_path = PathBuf::from(args.next().ok_or("missing corpus schema path")?);
            if args.next().is_some() {
                return Err("parse-corpus accepts exactly a corpus path and schema path".into());
            }
            let corpus = parse_corpus_manifest(&fs::read(corpus_path)?, &fs::read(schema_path)?)?;
            let value = serde_json::to_value(corpus)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
        }
        "parse-assertions" => {
            let assertion_path =
                PathBuf::from(args.next().ok_or("missing assertion registry path")?);
            if args.next().is_some() {
                return Err("parse-assertions accepts exactly one assertion registry path".into());
            }
            let assertions = parse_assertion_registry(&fs::read(assertion_path)?)?;
            let value = serde_json::to_value(assertions)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
        }
        "validate-corpus-assertions" => {
            let corpus_path = PathBuf::from(args.next().ok_or("missing corpus manifest path")?);
            let schema_path = PathBuf::from(args.next().ok_or("missing corpus schema path")?);
            let assertion_path =
                PathBuf::from(args.next().ok_or("missing assertion registry path")?);
            let surface_policy_path =
                PathBuf::from(args.next().ok_or("missing surface policy path")?);
            let repo_root = PathBuf::from(args.next().ok_or("missing repository root")?);
            if args.next().is_some() {
                return Err(
                    "validate-corpus-assertions accepts exactly corpus, corpus schema, assertion registry, surface policy, and repository root paths"
                        .into(),
                );
            }
            let corpus_bytes = fs::read(corpus_path)?;
            let schema_bytes = fs::read(schema_path)?;
            let assertion_bytes = fs::read(assertion_path)?;
            let surface_policy_bytes = fs::read(surface_policy_path)?;
            let (corpus, assertions) = validate_corpus_and_assertions(
                &corpus_bytes,
                &schema_bytes,
                &assertion_bytes,
                &surface_policy_bytes,
            )?;
            for entry in &corpus.entries {
                let fixture_bytes = fs::read(repo_root.join(&entry.fixture_path))?;
                verify_fixture_bytes(entry, &fixture_bytes)?;
            }
            let value = serde_json::json!({
                "assertion_count": assertions.entries.len(),
                "scenario_count": corpus.entries.len(),
                "schema": "commandf.af02-corpus-assertion-validation/v1"
            });
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
        }
        "parse-waiver-policy" => {
            let policy_path = PathBuf::from(args.next().ok_or("missing waiver policy path")?);
            if args.next().is_some() {
                return Err("parse-waiver-policy accepts exactly one path".into());
            }
            let policy = parse_waiver_policy(&fs::read(policy_path)?)?;
            let value = serde_json::to_value(policy)?;
            std::io::Write::write_all(
                &mut std::io::stdout().lock(),
                &canonical_json_bytes(&value)?,
            )?;
        }
        "verify-pr" => {
            return Err(
                "verify-pr is fail-closed until AF-02 T021-T025 semantic/input/base-gate enforcement is canonical"
                    .into(),
            );
        }
        other => return Err(format!("unknown entrypoint {other}").into()),
    }
    Ok(())
}
