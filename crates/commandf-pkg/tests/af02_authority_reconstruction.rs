#[path = "../../../tools/af02-verifier/src/authority.rs"]
mod authority;
#[path = "../../../tools/af02-verifier/src/canonical.rs"]
mod canonical;
#[path = "../../../tools/af02-verifier/src/retained.rs"]
mod retained;

use std::fs;
use std::path::PathBuf;

use authority::{project_assurance_ruleset, project_authority, project_cf06, Cf06Source};
use canonical::{canonical_json_bytes, git_blob_sha1_hex, parse_json_no_duplicates};
use retained::{
    locator_plan, project_retained, validate_and_parse, verify_artifacts, verify_workflow_run,
};
use serde_json::Value;

const MAIN_SHA: &str = "54b9772a3b86464da6f395f8ba8371f364c9bb38";
const MAIN_TREE: &str = "4ac26d8de419a0bec0faba8e14ded1763cfe30b3";
const RETAINED_SOURCES_BLOB: &str = "f9c0bc16ac742238c93ff77a85486cd1db5dbcf3";
const RETAINED_SCHEMA_BLOB: &str = "7d0daced343fd15d797cc0d4d53e9d63aac790c5";

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

fn assert_canonical_contract_objects() {
    assert_eq!(git_blob_sha1_hex(RETAINED_SOURCES), RETAINED_SOURCES_BLOB);
    assert_eq!(git_blob_sha1_hex(RETAINED_SCHEMA), RETAINED_SCHEMA_BLOB);
}

fn build_baseline() -> authority::AuthorityBaseline {
    assert_canonical_contract_objects();
    let retained = validate_and_parse(RETAINED_SOURCES, RETAINED_SCHEMA).unwrap();
    let run = parse_json_no_duplicates(RETAINED_RUN).unwrap();
    verify_workflow_run(&retained, &run).unwrap();
    let artifacts = parse_json_no_duplicates(RETAINED_ARTIFACTS).unwrap();
    verify_artifacts(&retained, &artifacts).unwrap();
    let retained_projection =
        project_retained(&retained, RETAINED_MANIFEST, RETAINED_DONOR).unwrap();
    let assurance = parse_json_no_duplicates(ASSURANCE_RULESET).unwrap();
    let review = parse_json_no_duplicates(REVIEW_RULESET).unwrap();

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

fn duplicate_probe(bytes: &[u8]) -> Vec<u8> {
    assert_eq!(bytes.first(), Some(&b'{'));
    let mut duplicate = br#"{"__duplicate_probe":0,"__duplicate_probe":1,"#.to_vec();
    duplicate.extend_from_slice(&bytes[1..]);
    duplicate
}

#[test]
fn retained_schema_rejects_candidate_url_authority() {
    let mut value: Value = parse_json_no_duplicates(RETAINED_SOURCES).unwrap();
    value.as_object_mut().unwrap().insert(
        "url".to_owned(),
        Value::String("https://example.invalid".to_owned()),
    );
    let bytes = serde_json::to_vec(&value).unwrap();
    let error = validate_and_parse(&bytes, RETAINED_SCHEMA).unwrap_err();
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn retained_contract_rejects_duplicate_semantic_keys_before_schema_validation() {
    let mut duplicate = br#"{"schema":"forged","#.to_vec();
    duplicate.extend_from_slice(&RETAINED_SOURCES[1..]);
    let error = validate_and_parse(&duplicate, RETAINED_SCHEMA).unwrap_err();
    assert!(error
        .to_string()
        .contains("duplicate JSON object key \"schema\""));
}

#[test]
fn authority_api_inputs_reject_duplicate_keys_before_projection() {
    for bytes in [
        ASSURANCE_RULESET,
        REVIEW_RULESET,
        RETAINED_RUN,
        RETAINED_ARTIFACTS,
    ] {
        let error = parse_json_no_duplicates(&duplicate_probe(bytes)).unwrap_err();
        assert!(error
            .to_string()
            .contains("duplicate JSON object key \"__duplicate_probe\""));
    }
}

#[test]
fn retained_locator_plan_reconstructs_frozen_github_urls() {
    assert_canonical_contract_objects();
    let retained = validate_and_parse(RETAINED_SOURCES, RETAINED_SCHEMA).unwrap();
    let plan = locator_plan(&retained).unwrap();

    assert_eq!(
        plan.pull_request,
        "https://api.github.com/repos/TheHalfMoon/commandF/pulls/11"
    );
    assert_eq!(
        plan.retained_head_commit,
        "https://api.github.com/repos/TheHalfMoon/commandF/commits/5fe10d9859407272acf6649fc3e868d3eb2fbd12"
    );
    assert_eq!(
        plan.retained_base_commit,
        "https://api.github.com/repos/TheHalfMoon/commandF/commits/5cb1a4c3445c0ebd86654cfb467a5e008e801c3e"
    );
    assert_eq!(
        plan.manifest_contents,
        "https://api.github.com/repos/TheHalfMoon/commandF/contents/corpus/real-ig/v1/corpus.json?ref=5fe10d9859407272acf6649fc3e868d3eb2fbd12"
    );
    assert_eq!(
        plan.manifest_blob,
        "https://api.github.com/repos/TheHalfMoon/commandF/git/blobs/655949a8a30d67502dffd624a175d2e8e02b1d1f"
    );
    assert_eq!(
        plan.donor_contents,
        "https://api.github.com/repos/TheHalfMoon/commandF/contents/donors/cf-10-real-ig-delta-corpus.yaml?ref=5fe10d9859407272acf6649fc3e868d3eb2fbd12"
    );
    assert_eq!(
        plan.donor_blob,
        "https://api.github.com/repos/TheHalfMoon/commandF/git/blobs/566b46f4e6f467a1ccae3ac810b31956309173b6"
    );
    assert_eq!(
        plan.workflow_run,
        "https://api.github.com/repos/TheHalfMoon/commandF/actions/runs/31916124080"
    );
    assert_eq!(
        plan.workflow_run_artifacts,
        "https://api.github.com/repos/TheHalfMoon/commandF/actions/runs/31916124080/artifacts"
    );
}

#[test]
fn retained_run_binding_rejects_wrong_event() {
    let retained = validate_and_parse(RETAINED_SOURCES, RETAINED_SCHEMA).unwrap();
    let mut run = parse_json_no_duplicates(RETAINED_RUN).unwrap();
    run.as_object_mut()
        .unwrap()
        .insert("event".to_owned(), Value::String("push".to_owned()));
    let error = verify_workflow_run(&retained, &run).unwrap_err();
    assert!(error.to_string().contains("event mismatch"));
}

#[test]
fn retained_artifact_binding_rejects_wrong_digest() {
    let retained = validate_and_parse(RETAINED_SOURCES, RETAINED_SCHEMA).unwrap();
    let mut artifacts = parse_json_no_duplicates(RETAINED_ARTIFACTS).unwrap();
    artifacts["artifacts"][0]["digest"] = Value::String(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
    );
    let error = verify_artifacts(&retained, &artifacts).unwrap_err();
    assert!(error.to_string().contains("artifact digest mismatch"));
}

#[test]
fn retained_projection_rejects_candidate_controlled_manifest_bytes() {
    let retained = validate_and_parse(RETAINED_SOURCES, RETAINED_SCHEMA).unwrap();
    let mut manifest = RETAINED_MANIFEST.to_vec();
    let index = manifest.iter().position(|byte| *byte == b'C').unwrap();
    manifest[index] = b'X';
    let error = project_retained(&retained, &manifest, RETAINED_DONOR).unwrap_err();
    assert!(error
        .to_string()
        .contains("retained manifest Git blob mismatch"));
}

#[test]
fn assurance_projection_rejects_wrong_required_check_app() {
    let mut assurance = parse_json_no_duplicates(ASSURANCE_RULESET).unwrap();
    assurance["rules"][2]["parameters"]["required_status_checks"][0]["integration_id"] =
        Value::from(1);
    let error = project_assurance_ruleset(&assurance).unwrap_err();
    assert!(error.to_string().contains("unexpected integration"));
}

#[test]
fn cf06_projection_rejects_candidate_controlled_source_bytes() {
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
    assert!(error.to_string().contains("Git blob mismatch"));
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
