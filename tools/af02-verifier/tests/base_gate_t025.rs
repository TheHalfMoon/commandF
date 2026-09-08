#[path = "../src/base_gate.rs"]
mod base_gate;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base_gate::{prove_base_identity, verify_changed_authority};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("AF-02 verifier must live under tools/ in the repository")
        .to_path_buf()
}

fn candidate_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must follow epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "commandf-af02-t025-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create isolated candidate root");
    root
}

fn write_candidate(root: &Path, relative: &str, bytes: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("candidate path parent"))
        .expect("create candidate parent");
    fs::write(path, bytes).expect("write candidate authority fixture");
}

#[test]
fn canonical_base_identity_records_workflow_verifier_schema_and_inventory_blobs() {
    let proof = prove_base_identity(&repo_root()).expect("prove exact canonical-base identities");

    assert_eq!(proof.schema, "commandf.af02-base-identity-proof/v1");
    assert_eq!(proof.base_sha.len(), 40);
    assert_eq!(proof.workflow_blob.len(), 40);
    assert_eq!(proof.enforcement_inventory_blob.len(), 40);
    assert_eq!(proof.verifier_blobs.len(), 3);
    assert_eq!(
        proof
            .verifier_blobs
            .get(".github/scripts/run_af02_base_verifier.sh")
            .map(String::len),
        Some(40)
    );
    assert_eq!(
        proof
            .verifier_blobs
            .get("tools/af02-verifier/src/main.rs")
            .map(String::len),
        Some(40)
    );
    assert_eq!(
        proof
            .verifier_blobs
            .get("tools/af02-verifier/src/base_gate.rs")
            .map(String::len),
        Some(40)
    );
    assert!(
        proof.schema_blobs.contains_key(
            "specs/016-af-02-adversarial-test-strength/schemas/af02-enforcement-inventory-v1.schema.json"
        ),
        "frozen enforcement-inventory schema must be bound by Git blob identity"
    );
    assert!(proof.schema_blobs.values().all(|sha| sha.len() == 40));
}

#[test]
fn authority_triggering_is_universal_across_frozen_surfaces() {
    let base = repo_root();
    let candidate = candidate_root("universal");
    let cases = [
        ".github/required-checks.json",
        ".github/scripts/run_af02_base_verifier.sh",
        ".github/workflows/ci.yml",
        "donors/CF_06-RUSTSEC-0001/manifest.json",
        "specs/016-af-02-adversarial-test-strength/semantic-contract.json",
        "tools/af02-verifier/src/main.rs",
        "tools/af02-verifier/src/replay.rs",
        "Cargo.lock",
        "crates/commandf-pkg/src/oracle_model.rs",
    ];

    for path in cases {
        let proof = verify_changed_authority(&base, &candidate, &[path.to_owned()])
            .unwrap_or_else(|error| panic!("authority path {path} must classify: {error}"));
        assert_eq!(proof.mode, "AUTHORITY_VERIFICATION_REQUIRED", "{path}");
        assert_eq!(proof.authority_paths, vec![path.to_owned()]);
    }

    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn non_authority_change_is_not_applicable() {
    let candidate = candidate_root("not-applicable");
    let proof = verify_changed_authority(&repo_root(), &candidate, &["README.md".to_owned()])
        .expect("non-authority path must classify without weakening the gate");
    assert_eq!(proof.mode, "NOT_APPLICABLE");
    assert!(proof.authority_paths.is_empty());
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn unknown_authority_path_fails_closed() {
    let candidate = candidate_root("unknown");
    let error = verify_changed_authority(
        &repo_root(),
        &candidate,
        &["tools/af02-verifier/src/unplanned_authority.rs".to_owned()],
    )
    .expect_err("unplanned verifier authority must fail closed");
    assert!(error.to_string().contains("unknown AF-02 authority path"));
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn malformed_json_authority_fails_closed_before_semantic_acceptance() {
    let candidate = candidate_root("malformed-json");
    write_candidate(
        &candidate,
        ".github/required-checks.json",
        br#"{"duplicate":1,"duplicate":2}"#,
    );
    let error = verify_changed_authority(
        &repo_root(),
        &candidate,
        &[".github/required-checks.json".to_owned()],
    )
    .expect_err("duplicate-key candidate authority must fail closed");
    assert!(error.to_string().contains("unparseable"));
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn yaml_authority_is_fail_closed_until_hardened_parser_activation() {
    let candidate = candidate_root("yaml");
    write_candidate(
        &candidate,
        ".github/workflows/ci.yml",
        b"name: candidate\non:\n  pull_request:\n",
    );
    let error = verify_changed_authority(
        &repo_root(),
        &candidate,
        &[".github/workflows/ci.yml".to_owned()],
    )
    .expect_err("candidate YAML authority must not bypass the hardened parser boundary");
    assert!(error.to_string().contains("hardened YAML parser"));
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn noncanonical_and_escape_paths_fail_before_classification() {
    let candidate = candidate_root("path-escape");
    for path in ["../escape", "/absolute", "tools//af02-verifier/src/main.rs"] {
        assert!(
            verify_changed_authority(&repo_root(), &candidate, &[path.to_owned()]).is_err(),
            "path {path} must fail closed"
        );
    }
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}
