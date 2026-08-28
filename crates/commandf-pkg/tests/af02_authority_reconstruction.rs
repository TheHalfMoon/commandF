#[path = "../../../tools/af02-verifier/src/authority.rs"]
mod authority;
#[path = "../../../tools/af02-verifier/src/canonical.rs"]
mod canonical;
#[path = "../../../tools/af02-verifier/src/retained.rs"]
mod retained;

use std::fs;
use std::path::PathBuf;

use authority::{project_assurance_ruleset, project_authority, project_cf06, Cf06Source};
use canonical::canonical_json_bytes;
use retained::{project_retained, validate_and_parse, verify_artifacts, verify_workflow_run};
use serde_json::Value;

const MAIN_SHA: &str = "54b9772a3b86464da6f395f8ba8371f364c9bb38";
const MAIN_TREE: &str = "4ac26d8de419a0bec0faba8e14ded1763cfe30b3";

const ORACLE_MODEL: &[u8] = include_bytes!("../src/oracle_model.rs");
const CF06_DONOR: &[u8] = include_bytes!("../../../donors/hl7-fhir-validator-6.10.2.yaml");
const CF06_WORKFLOW: &[u8] = include_bytes!("../../../.github/workflows/cf06-oracle.yml");
const RETAINED_SOURCES: &[u8] = include_bytes!(
    "../../../specs/016-af-02-adversarial-test-strength/retained-authority-sources.json"
);
const RETAINED_SCHEMA: &[u8] = include_bytes!(
    "../../../specs/016-af-02-adversarial-test-strength/schemas/af02-retained-authority-sources-v1.schema.json"
);
const ASSURANCE_RULESET: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/assurance-ruleset.json");
const REVIEW_RULESET: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/review-ruleset.json");
const RETAINED_MANIFEST: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/cf10-corpus.json");
const RETAINED_DONOR: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/cf10-donor.yaml");
const RETAINED_RUN: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/cf10-run.json");
const RETAINED_ARTIFACTS: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/cf10-artifacts.json");

fn build_baseline() -> authority::AuthorityBaseline {
    let retained = validate_and_parse(RETAINED_SOURCES, RETAINED_SCHEMA).unwrap();
    let run: Value = serde_json::from_slice(RETAINED_RUN).unwrap();
    verify_workflow_run(&retained, &run).unwrap();
    let artifacts: Value = serde_json::from_slice(RETAINED_ARTIFACTS).unwrap();
    verify_artifacts(&retained, &artifacts).unwrap();
    let retained_projection =
        project_retained(&retained, RETAINED_MANIFEST, RETAINED_DONOR).unwrap();
    let assurance: Value = serde_json::from_slice(ASSURANCE_RULESET).unwrap();
    let review: Value = serde_json::from_slice(REVIEW_RULESET).unwrap();

    project_authority(
        MAIN_SHA,
        MAIN_TREE,
        &assurance,
        &review,
        [
            Cf06Source {
                path: "crates/commandf-pkg/src/oracle_model.rs",
                git_blob_sha: "9046546a86061961cf3e17f3f1880165625edea8",
                bytes: ORACLE_MODEL,
            },
            Cf06Source {
                path: "donors/hl7-fhir-validator-6.10.2.yaml",
                git_blob_sha: "9add2dad45cb8958c9304d38e29950ed1f769990",
                bytes: CF06_DONOR,
            },
            Cf06Source {
                path: ".github/workflows/cf06-oracle.yml",
                git_blob_sha: "664e303983d2ef85aad934cbef2c14d63744e0ee",
                bytes: CF06_WORKFLOW,
            },
        ],
        retained_projection,
    )
    .unwrap()
}

#[test]
fn retained_schema_rejects_candidate_url_authority() {
    let mut value: Value = serde_json::from_slice(RETAINED_SOURCES).unwrap();
    value.as_object_mut().unwrap().insert(
        "url".to_owned(),
        Value::String("https://example.invalid".to_owned()),
    );
    let bytes = serde_json::to_vec(&value).unwrap();
    let error = validate_and_parse(&bytes, RETAINED_SCHEMA).unwrap_err();
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn retained_run_binding_rejects_wrong_event() {
    let retained = validate_and_parse(RETAINED_SOURCES, RETAINED_SCHEMA).unwrap();
    let mut run: Value = serde_json::from_slice(RETAINED_RUN).unwrap();
    run.as_object_mut()
        .unwrap()
        .insert("event".to_owned(), Value::String("push".to_owned()));
    let error = verify_workflow_run(&retained, &run).unwrap_err();
    assert!(error.to_string().contains("event mismatch"));
}

#[test]
fn retained_artifact_binding_rejects_wrong_digest() {
    let retained = validate_and_parse(RETAINED_SOURCES, RETAINED_SCHEMA).unwrap();
    let mut artifacts: Value = serde_json::from_slice(RETAINED_ARTIFACTS).unwrap();
    artifacts["artifacts"][0]["digest"] = Value::String(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
    );
    let error = verify_artifacts(&retained, &artifacts).unwrap_err();
    assert!(error.to_string().contains("artifact digest mismatch"));
}

#[test]
fn assurance_projection_rejects_wrong_required_check_app() {
    let mut assurance: Value = serde_json::from_slice(ASSURANCE_RULESET).unwrap();
    assurance["rules"][2]["parameters"]["required_status_checks"][0]["integration_id"] =
        Value::from(1);
    let error = project_assurance_ruleset(&assurance).unwrap_err();
    assert!(error.to_string().contains("unexpected integration"));
}

#[test]
fn cf06_projection_rejects_missing_source_pin() {
    let altered = ORACLE_MODEL
        .windows(authority::CF06_SOURCE_COMMIT.len())
        .position(|window| window == authority::CF06_SOURCE_COMMIT.as_bytes())
        .unwrap();
    let mut bytes = ORACLE_MODEL.to_vec();
    bytes[altered] = b'0';

    let error = project_cf06([
        Cf06Source {
            path: "crates/commandf-pkg/src/oracle_model.rs",
            git_blob_sha: "9046546a86061961cf3e17f3f1880165625edea8",
            bytes: &bytes,
        },
        Cf06Source {
            path: "donors/hl7-fhir-validator-6.10.2.yaml",
            git_blob_sha: "9add2dad45cb8958c9304d38e29950ed1f769990",
            bytes: CF06_DONOR,
        },
        Cf06Source {
            path: ".github/workflows/cf06-oracle.yml",
            git_blob_sha: "664e303983d2ef85aad934cbef2c14d63744e0ee",
            bytes: CF06_WORKFLOW,
        },
    ])
    .unwrap_err();
    assert!(error.to_string().contains("does not bind"));
}

#[test]
fn authority_baseline_v2_matches_canonical_snapshot() {
    let baseline = build_baseline();
    let value = serde_json::to_value(&baseline).unwrap();
    let generated = canonical_json_bytes(&value).unwrap();

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../specs/016-af-02-adversarial-test-strength/authority-baseline.json");
    match fs::read(&path) {
        Ok(expected) => assert_eq!(generated, expected),
        Err(error) => panic!(
            "authority baseline snapshot is missing ({error}); AF02_GENERATED_BASELINE={}",
            String::from_utf8(generated).unwrap()
        ),
    }
}
