use std::fs;
use std::path::PathBuf;

use commandf_af02_verifier::authority::{project_authority, Cf06Source};
use commandf_af02_verifier::canonical::canonical_json_bytes;
use commandf_af02_verifier::retained::{
    locator_plan, project_retained, validate_and_parse, verify_artifacts, verify_workflow_run,
};
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
            let retained_path = PathBuf::from(args.next().ok_or("missing retained authority path")?);
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
            let input: AuthorityInput = serde_json::from_slice(&fs::read(input_path)?)?;
            let retained_bytes = fs::read(&input.retained_sources_path)?;
            let retained_schema_bytes = fs::read(&input.retained_schema_path)?;
            let retained = validate_and_parse(&retained_bytes, &retained_schema_bytes)?;

            let run: Value = serde_json::from_slice(&fs::read(&input.retained_workflow_run_path)?)?;
            verify_workflow_run(&retained, &run)?;
            let artifacts: Value =
                serde_json::from_slice(&fs::read(&input.retained_artifacts_path)?)?;
            verify_artifacts(&retained, &artifacts)?;

            let retained_projection = project_retained(
                &retained,
                &fs::read(&input.retained_manifest_path)?,
                &fs::read(&input.retained_donor_path)?,
            )?;
            let assurance: Value =
                serde_json::from_slice(&fs::read(&input.assurance_ruleset_path)?)?;
            let review: Value = serde_json::from_slice(&fs::read(&input.review_ruleset_path)?)?;

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
