#[path = "../../../tools/af02-verifier/src/authority.rs"]
mod authority;
#[path = "../../../tools/af02-verifier/src/canonical.rs"]
mod canonical;
#[path = "../../../tools/af02-verifier/src/retained.rs"]
mod retained;

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use authority::{project_assurance_ruleset, project_authority, project_cf06, Cf06Source};
use canonical::{canonical_json_bytes, git_blob_sha1_hex, parse_json_no_duplicates};
use retained::{
    locator_plan, project_retained, validate_and_parse, verify_artifacts, verify_workflow_run,
};
use serde_json::Value;

const MAIN_SHA: &str = "54b9772a3b86464da6f395f8ba8371f364c9bb38";
const MAIN_TREE: &str = "4ac26d8de419a0bec0faba8e14ded1763cfe30b3";
const RETAINED_HEAD: &str = "5fe10d9859407272acf6649fc3e868d3eb2fbd12";

const RETAINED_SOURCES_PATH: &str =
    "specs/016-af-02-adversarial-test-strength/retained-authority-sources.json";
const RETAINED_SOURCES_BLOB: &str = "f9c0bc16ac742238c93ff77a85486cd1db5dbcf3";
const RETAINED_SCHEMA_PATH: &str = "specs/016-af-02-adversarial-test-strength/schemas/af02-retained-authority-sources-v1.schema.json";
const RETAINED_SCHEMA_BLOB: &str = "7d0daced343fd15d797cc0d4d53e9d63aac790c5";

const ORACLE_MODEL_PATH: &str = "crates/commandf-pkg/src/oracle_model.rs";
const ORACLE_MODEL_BLOB: &str = "9046546a86061961cf3e17f3f1880165625edea8";
const CF06_DONOR_PATH: &str = "donors/hl7-fhir-validator-6.10.2.yaml";
const CF06_DONOR_BLOB: &str = "9add2dad45cb8958c9304d38e29950ed1f769990";
const CF06_WORKFLOW_PATH: &str = ".github/workflows/cf06-oracle.yml";
const CF06_WORKFLOW_BLOB: &str = "664e303983d2ef85aad934cbef2c14d63744e0ee";

const ASSURANCE_RULESET: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/assurance-ruleset.json");
const REVIEW_RULESET: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/review-ruleset.json");
const RETAINED_RUN: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/cf10-run.json");
const RETAINED_ARTIFACTS: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/cf10-artifacts.json");

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn git_object_bytes(revision: &str, path: &str, expected_blob: &str) -> Vec<u8> {
    let root = repository_root();
    let spec = format!("{revision}:{path}");
    let resolved = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", &spec])
        .output()
        .expect("run git rev-parse");
    assert!(
        resolved.status.success(),
        "git rev-parse failed for {spec}: {}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    let observed_blob = String::from_utf8(resolved.stdout)
        .expect("git rev-parse UTF-8")
        .trim()
        .to_owned();
    assert_eq!(
        observed_blob, expected_blob,
        "canonical Git object identity drifted for {spec}"
    );

    let object = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["cat-file", "blob", expected_blob])
        .output()
        .expect("run git cat-file");
    assert!(
        object.status.success(),
        "git cat-file failed for {expected_blob}: {}",
        String::from_utf8_lossy(&object.stderr)
    );
    assert_eq!(
        git_blob_sha1_hex(&object.stdout),
        expected_blob,
        "Git object bytes do not reproduce expected blob identity"
    );
    object.stdout
}

fn canonical_retained_contract() -> (Vec<u8>, Vec<u8>) {
    (
        git_object_bytes(MAIN_SHA, RETAINED_SOURCES_PATH, RETAINED_SOURCES_BLOB),
        git_object_bytes(MAIN_SHA, RETAINED_SCHEMA_PATH, RETAINED_SCHEMA_BLOB),
    )
}

fn canonical_cf06_sources() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    (
        git_object_bytes(MAIN_SHA, ORACLE_MODEL_PATH, ORACLE_MODEL_BLOB),
        git_object_bytes(MAIN_SHA, CF06_DONOR_PATH, CF06_DONOR_BLOB),
        git_object_bytes(MAIN_SHA, CF06_WORKFLOW_PATH, CF06_WORKFLOW_BLOB),
    )
}

fn build_baseline() -> authority::AuthorityBaseline {
    let (retained_sources, retained_schema) = canonical_retained_contract();
    let retained = validate_and_parse(&retained_sources, &retained_schema).unwrap();
    assert_eq!(retained.cf10.retained_head, RETAINED_HEAD);

    let run = parse_json_no_duplicates(RETAINED_RUN).unwrap();
    verify_workflow_run(&retained, &run).unwrap();
    let artifacts = parse_json_no_duplicates(RETAINED_ARTIFACTS).unwrap();
    verify_artifacts(&retained, &artifacts).unwrap();

    let retained_manifest = git_object_bytes(
        RETAINED_HEAD,
        &retained.cf10.manifest.path,
        &retained.cf10.manifest.git_blob_sha,
    );
    let retained_donor = git_object_bytes(
        RETAINED_HEAD,
        &retained.cf10.donor.path,
        &retained.cf10.donor.git_blob_sha,
    );
    let retained_projection =
        project_retained(&retained, &retained_manifest, &retained_donor).unwrap();

    let assurance = parse_json_no_duplicates(ASSURANCE_RULESET).unwrap();
    let review = parse_json_no_duplicates(REVIEW_RULESET).unwrap();
    let (oracle_model, cf06_donor, cf06_workflow) = canonical_cf06_sources();

    project_authority(
        MAIN_SHA,
        MAIN_TREE,
        &assurance,
        &review,
        [
            Cf06Source {
                path: ORACLE_MODEL_PATH,
                git_blob_sha: ORACLE_MODEL_BLOB,
                bytes: &oracle_model,
            },
            Cf06Source {
                path: CF06_DONOR_PATH,
                git_blob_sha: CF06_DONOR_BLOB,
                bytes: &cf06_donor,
            },
            Cf06Source {
                path: CF06_WORKFLOW_PATH,
                git_blob_sha: CF06_WORKFLOW_BLOB,
                bytes: &cf06_workflow,
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
    let (retained_sources, retained_schema) = canonical_retained_contract();
    let mut value: Value = parse_json_no_duplicates(&retained_sources).unwrap();
    value.as_object_mut().unwrap().insert(
        "url".to_owned(),
        Value::String("https://example.invalid".to_owned()),
    );
    let bytes = serde_json::to_vec(&value).unwrap();
    let error = validate_and_parse(&bytes, &retained_schema).unwrap_err();
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn retained_contract_rejects_duplicate_semantic_keys_before_schema_validation() {
    let (retained_sources, retained_schema) = canonical_retained_contract();
    let mut duplicate = br#"{"schema":"forged","#.to_vec();
    duplicate.extend_from_slice(&retained_sources[1..]);
    let error = validate_and_parse(&duplicate, &retained_schema).unwrap_err();
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
    let (retained_sources, retained_schema) = canonical_retained_contract();
    let retained = validate_and_parse(&retained_sources, &retained_schema).unwrap();
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
    let (retained_sources, retained_schema) = canonical_retained_contract();
    let retained = validate_and_parse(&retained_sources, &retained_schema).unwrap();
    let mut run = parse_json_no_duplicates(RETAINED_RUN).unwrap();
    run.as_object_mut()
        .unwrap()
        .insert("event".to_owned(), Value::String("push".to_owned()));
    let error = verify_workflow_run(&retained, &run).unwrap_err();
    assert!(error.to_string().contains("event mismatch"));
}

#[test]
fn retained_artifact_binding_rejects_wrong_digest() {
    let (retained_sources, retained_schema) = canonical_retained_contract();
    let retained = validate_and_parse(&retained_sources, &retained_schema).unwrap();
    let mut artifacts = parse_json_no_duplicates(RETAINED_ARTIFACTS).unwrap();
    artifacts["artifacts"][0]["digest"] = Value::String(
        "sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
    );
    let error = verify_artifacts(&retained, &artifacts).unwrap_err();
    assert!(error.to_string().contains("artifact digest mismatch"));
}

#[test]
fn retained_projection_rejects_candidate_controlled_manifest_bytes() {
    let (retained_sources, retained_schema) = canonical_retained_contract();
    let retained = validate_and_parse(&retained_sources, &retained_schema).unwrap();
    let mut manifest = git_object_bytes(
        RETAINED_HEAD,
        &retained.cf10.manifest.path,
        &retained.cf10.manifest.git_blob_sha,
    );
    let donor = git_object_bytes(
        RETAINED_HEAD,
        &retained.cf10.donor.path,
        &retained.cf10.donor.git_blob_sha,
    );
    let index = manifest.iter().position(|byte| *byte == b'C').unwrap();
    manifest[index] = b'X';
    let error = project_retained(&retained, &manifest, &donor).unwrap_err();
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
    let (mut oracle_model, cf06_donor, cf06_workflow) = canonical_cf06_sources();
    let altered = oracle_model
        .windows(authority::CF06_SOURCE_COMMIT.len())
        .position(|window| window == authority::CF06_SOURCE_COMMIT.as_bytes())
        .unwrap();
    oracle_model[altered] = b'0';

    let error = project_cf06([
        Cf06Source {
            path: ORACLE_MODEL_PATH,
            git_blob_sha: ORACLE_MODEL_BLOB,
            bytes: &oracle_model,
        },
        Cf06Source {
            path: CF06_DONOR_PATH,
            git_blob_sha: CF06_DONOR_BLOB,
            bytes: &cf06_donor,
        },
        Cf06Source {
            path: CF06_WORKFLOW_PATH,
            git_blob_sha: CF06_WORKFLOW_BLOB,
            bytes: &cf06_workflow,
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
