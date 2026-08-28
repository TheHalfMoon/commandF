#[path = "../../../tools/af02-verifier/src/authority.rs"]
mod authority;
#[path = "../../../tools/af02-verifier/src/canonical.rs"]
mod canonical;
#[path = "../../../tools/af02-verifier/src/retained.rs"]
mod retained;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use authority::{project_assurance_ruleset, project_authority, project_cf06, Cf06Source};
use canonical::{canonical_json_bytes, git_blob_sha1_hex, parse_json_no_duplicates};
use retained::{
    locator_plan, project_retained, validate_and_parse, verify_artifacts, verify_workflow_run,
};
use serde_json::{json, Value};

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

const AF01_CLOSEOUT_PATH: &str = "specs/015-af-01-trusted-development-baseline/closeout.md";
const AF01_CLOSEOUT_BLOB: &str = "ac01a88ff7c1a4f4771dd16c5a61afe6e2566ce6";

const ASSURANCE_RULESET: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/assurance-ruleset.json");
const REVIEW_RULESET: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/review-ruleset.json");
const RETAINED_RUN: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/cf10-run.json");
const RETAINED_ARTIFACTS: &[u8] =
    include_bytes!("../../../tools/af02-verifier/tests/fixtures/cf10-artifacts.json");

static GIT_FETCH_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn pinned_commit_available(root: &Path, revision: &str) -> bool {
    let commit = format!("{revision}^{{commit}}");
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "-e", &commit])
        .status()
        .expect("run git cat-file commit probe")
        .success()
}

fn ensure_pinned_commit_available(root: &Path, revision: &str) {
    if pinned_commit_available(root, revision) {
        return;
    }

    let fetch_lock = GIT_FETCH_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = fetch_lock.lock().expect("Git fetch mutex poisoned");
    if pinned_commit_available(root, revision) {
        return;
    }

    let remote = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["remote", "get-url", "origin"])
        .output()
        .expect("read Git origin URL");
    assert!(
        remote.status.success(),
        "git remote get-url origin failed: {}",
        String::from_utf8_lossy(&remote.stderr)
    );
    let origin = String::from_utf8(remote.stdout)
        .expect("Git origin URL UTF-8")
        .trim()
        .trim_end_matches('/')
        .trim_end_matches(".git")
        .to_owned();
    assert!(
        origin == "https://github.com/TheHalfMoon/commandF"
            || origin == "git@github.com:TheHalfMoon/commandF",
        "refusing to fetch authority objects from non-canonical origin {origin}"
    );

    let fetched = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "fetch",
            "--no-tags",
            "--no-write-fetch-head",
            "--depth=1",
            "origin",
            revision,
        ])
        .output()
        .expect("fetch pinned authority commit");
    assert!(
        fetched.status.success(),
        "git fetch failed for pinned authority commit {revision}: {}",
        String::from_utf8_lossy(&fetched.stderr)
    );

    let commit = format!("{revision}^{{commit}}");
    let resolved = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--verify", &commit])
        .output()
        .expect("verify fetched authority commit");
    assert!(
        resolved.status.success(),
        "git rev-parse failed for fetched authority commit {revision}: {}",
        String::from_utf8_lossy(&resolved.stderr)
    );
    assert_eq!(
        String::from_utf8(resolved.stdout)
            .expect("fetched authority commit UTF-8")
            .trim(),
        revision,
        "fetched authority commit identity drifted"
    );
}

fn github_content_object_bytes(revision: &str, path: &str, expected_blob: &str) -> Vec<u8> {
    assert!(
        revision.len() == 40
            && revision
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "refusing non-immutable GitHub revision {revision}"
    );
    assert!(
        expected_blob.len() == 40
            && expected_blob
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "invalid expected Git blob identity {expected_blob}"
    );
    assert!(
        !path.starts_with('/')
            && !path
                .split('/')
                .any(|segment| segment.is_empty() || segment == "." || segment == "..")
            && path.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'.' | b'_' | b'-')
            }),
        "refusing unsafe GitHub authority path {path}"
    );

    let url =
        format!("https://api.github.com/repos/TheHalfMoon/commandF/contents/{path}?ref={revision}");
    let response = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--connect-timeout",
            "10",
            "--max-time",
            "30",
            "--header",
            "Accept: application/vnd.github.raw+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            "--header",
            "User-Agent: commandF-af02-authority-reconstruction",
            &url,
        ])
        .output()
        .expect("fetch immutable GitHub authority object with curl");
    assert!(
        response.status.success(),
        "immutable GitHub authority request failed for {revision}:{path}: {}",
        String::from_utf8_lossy(&response.stderr)
    );
    assert_eq!(
        git_blob_sha1_hex(&response.stdout),
        expected_blob,
        "immutable GitHub authority bytes do not reproduce expected blob identity for {revision}:{path}"
    );
    response.stdout
}

fn git_object_bytes(revision: &str, path: &str, expected_blob: &str) -> Vec<u8> {
    let root = repository_root();
    ensure_pinned_commit_available(&root, revision);
    let spec = format!("{revision}:{path}");
    let resolved = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", &spec])
        .output()
        .expect("run git rev-parse");
    if !resolved.status.success() {
        return github_content_object_bytes(revision, path, expected_blob);
    }
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

fn canonical_ruleset_view(url: &str, ruleset_id: u64) -> Value {
    let response = github_api_bytes(url);
    let mut value = parse_json_no_duplicates(&response).unwrap();
    let bypass_is_redacted = matches!(value.get("bypass_actors"), None | Some(Value::Null));
    if !bypass_is_redacted {
        return value;
    }

    // GitHub intentionally withholds bypass_actors from callers without write
    // access to the ruleset. Recover only that redacted field from AF-01's
    // owner-authorized canonical closeout; every non-privileged field remains
    // live API authority. The closeout itself is bound to MAIN_SHA and an exact
    // Git blob identity, so candidate-controlled fixtures cannot supply it.
    let closeout = git_object_bytes(MAIN_SHA, AF01_CLOSEOUT_PATH, AF01_CLOSEOUT_BLOB);
    let closeout = std::str::from_utf8(&closeout).expect("AF-01 closeout must be UTF-8");
    let bypass = match ruleset_id {
        authority::ASSURANCE_RULESET_ID => {
            let owner_evidence = "21652953 commandF main assurance\n  enforcement: active\n  bypass actors: none\n  current user bypass: never";
            assert!(
                closeout.contains(owner_evidence),
                "canonical AF-01 closeout no longer proves assurance bypass authority"
            );
            json!([])
        }
        authority::REVIEW_RULESET_ID => {
            let owner_evidence = "21652974 commandF main review governance\n  enforcement: active\n  merge method: merge\n  approvals: 1\n  code-owner review: required\n  latest-push approval: required\n  stale approvals: dismissed\n  review-thread resolution: required\n  bypass: RepositoryRole actor 5, pull_request only\n  current user bypass: pull_requests_only";
            assert!(
                closeout.contains(owner_evidence),
                "canonical AF-01 closeout no longer proves review bypass authority"
            );
            json!([{
                "actor_id": 5,
                "actor_type": "RepositoryRole",
                "bypass_mode": "pull_request"
            }])
        }
        other => panic!("unexpected AF-01 ruleset id {other}"),
    };

    value
        .as_object_mut()
        .expect("ruleset API response must be an object")
        .insert("bypass_actors".to_owned(), bypass);
    value
}

fn github_api_bytes(url: &str) -> Vec<u8> {
    const CANONICAL_API_PREFIX: &str = "https://api.github.com/repos/TheHalfMoon/commandF/";
    assert!(
        url.starts_with(CANONICAL_API_PREFIX),
        "refusing non-canonical GitHub authority URL {url}"
    );

    let response = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--connect-timeout",
            "10",
            "--max-time",
            "30",
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
            "--header",
            "User-Agent: commandF-af02-authority-reconstruction",
            url,
        ])
        .output()
        .expect("fetch live GitHub authority response with curl");
    assert!(
        response.status.success(),
        "GitHub authority request failed for {url}: {}",
        String::from_utf8_lossy(&response.stderr)
    );
    response.stdout
}

fn build_baseline() -> authority::AuthorityBaseline {
    let (retained_sources, retained_schema) = canonical_retained_contract();
    let retained = validate_and_parse(&retained_sources, &retained_schema).unwrap();
    assert_eq!(retained.cf10.retained_head, RETAINED_HEAD);

    let plan = locator_plan(&retained).unwrap();
    assert_eq!(
        plan.workflow_run,
        "https://api.github.com/repos/TheHalfMoon/commandF/actions/runs/31916124080"
    );
    assert_eq!(
        plan.workflow_run_artifacts,
        "https://api.github.com/repos/TheHalfMoon/commandF/actions/runs/31916124080/artifacts"
    );
    let run_bytes = github_api_bytes(&plan.workflow_run);
    let run = parse_json_no_duplicates(&run_bytes).unwrap();
    verify_workflow_run(&retained, &run).unwrap();
    let artifact_bytes = github_api_bytes(&plan.workflow_run_artifacts);
    let artifacts = parse_json_no_duplicates(&artifact_bytes).unwrap();
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

    assert_eq!(authority::ASSURANCE_RULESET_ID, 21652953);
    assert_eq!(authority::REVIEW_RULESET_ID, 21652974);
    let assurance = canonical_ruleset_view(
        "https://api.github.com/repos/TheHalfMoon/commandF/rulesets/21652953",
        authority::ASSURANCE_RULESET_ID,
    );
    let review = canonical_ruleset_view(
        "https://api.github.com/repos/TheHalfMoon/commandF/rulesets/21652974",
        authority::REVIEW_RULESET_ID,
    );
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
