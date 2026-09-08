#[path = "../src/input_guard.rs"]
mod input_guard;
#[path = "../src/base_gate.rs"]
mod base_gate;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use base_gate::{test_gate_input, verify_pr, ChangedFile};

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

fn copy_from_base(candidate: &Path, relative: &str) {
    let base = repo_root();
    let bytes = fs::read(base.join(relative))
        .unwrap_or_else(|error| panic!("read canonical fixture {relative}: {error}"));
    write_candidate(candidate, relative, &bytes);
}

fn changed(status: &str, filename: &str, previous_filename: Option<&str>) -> ChangedFile {
    ChangedFile {
        status: status.to_owned(),
        filename: filename.to_owned(),
        previous_filename: previous_filename.map(str::to_owned),
    }
}

fn verify(candidate: &Path, changes: Vec<ChangedFile>) -> Result<base_gate::BaseGateProof, String> {
    let base = repo_root();
    let input = test_gate_input(&base, changes).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec(&input).expect("serialize trusted test gate input");
    verify_pr(&base, candidate, &bytes).map_err(|error| error.to_string())
}

#[test]
fn canonical_base_identity_records_full_verifier_schema_inventory_and_workflow_topology() {
    let proof = base_gate::prove_base_identity(&repo_root())
        .expect("prove exact canonical-base identities");

    assert_eq!(proof.schema, "commandf.af02-base-identity-proof/v1");
    assert_eq!(proof.base_sha.len(), 40);
    assert_eq!(proof.base_tree.len(), 40);
    assert_eq!(proof.workflow_blob.len(), 40);
    assert_eq!(proof.runner_blob.len(), 40);
    assert_eq!(proof.cargo_manifest_blob.len(), 40);
    assert_eq!(proof.cargo_lock_blob.len(), 40);
    assert_eq!(proof.enforcement_inventory_blob.len(), 40);
    for required in [
        "tools/af02-verifier/src/base_gate.rs",
        "tools/af02-verifier/src/input_guard.rs",
        "tools/af02-verifier/src/main.rs",
        "tools/af02-verifier/src/semantic.rs",
    ] {
        assert_eq!(
            proof.verifier_blobs.get(required).map(String::len),
            Some(40),
            "canonical verifier identity must bind {required}"
        );
    }
    assert!(
        proof.schema_blobs.contains_key(
            "specs/016-af-02-adversarial-test-strength/schemas/af02-enforcement-inventory-v1.schema.json"
        ),
        "frozen enforcement-inventory schema must be bound by Git blob identity"
    );
    assert!(proof.verifier_blobs.values().all(|sha| sha.len() == 40));
    assert!(proof.schema_blobs.values().all(|sha| sha.len() == 40));
}

#[test]
fn authority_triggering_is_universal_across_frozen_surfaces() {
    let cases = [
        ".github/required-checks.json",
        ".github/scripts/run_af02_base_verifier.sh",
        ".github/workflows/ci.yml",
        "donors/af-02-adversarial-testing.yaml",
        "specs/016-af-02-adversarial-test-strength/semantic-contract.json",
        "tools/af02-verifier/src/main.rs",
        "Cargo.lock",
        "crates/commandf-pkg/src/oracle_model.rs",
    ];

    for path in cases {
        let candidate = candidate_root("universal");
        copy_from_base(&candidate, path);
        let proof = verify(&candidate, vec![changed("modified", path, None)])
            .unwrap_or_else(|error| panic!("authority path {path} must classify: {error}"));
        assert_eq!(proof.mode, "AUTHORITY_VERIFICATION_REQUIRED", "{path}");
        assert_eq!(proof.authority_paths, vec![path.to_owned()]);
        assert!(!proof.candidate_code_executed);
        fs::remove_dir_all(candidate).expect("remove isolated candidate root");
    }
}

#[test]
fn frozen_future_verifier_path_is_known_but_never_executed() {
    let candidate = candidate_root("future-role");
    let path = "tools/af02-verifier/src/replay.rs";
    write_candidate(&candidate, path, b"pub fn run_replay() {}\n");
    let proof = verify(&candidate, vec![changed("added", path, None)])
        .expect("frozen future implementation path should be known authority data");
    assert_eq!(proof.mode, "AUTHORITY_VERIFICATION_REQUIRED");
    assert!(!proof.candidate_code_executed);
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn non_authority_change_is_not_applicable() {
    let candidate = candidate_root("not-applicable");
    let proof = verify(&candidate, vec![changed("modified", "README.md", None)])
        .expect("non-authority path must classify without weakening the gate");
    assert_eq!(proof.mode, "NOT_APPLICABLE");
    assert!(proof.authority_paths.is_empty());
    assert!(!proof.candidate_code_executed);
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn unknown_authority_path_fails_closed() {
    let candidate = candidate_root("unknown");
    let error = verify(
        &candidate,
        vec![changed(
            "added",
            "tools/af02-verifier/src/unplanned_authority.rs",
            None,
        )],
    )
    .expect_err("unplanned verifier authority must fail closed");
    assert!(error.contains("unknown AF-02 authority path"));
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn malformed_and_duplicate_key_json_authority_fail_closed() {
    for (label, bytes) in [
        ("malformed", b"{not-json".as_slice()),
        ("duplicate", br#"{"duplicate":1,"duplicate":2}"#.as_slice()),
    ] {
        let candidate = candidate_root(label);
        write_candidate(&candidate, ".github/required-checks.json", bytes);
        let error = verify(
            &candidate,
            vec![changed("modified", ".github/required-checks.json", None)],
        )
        .expect_err("invalid JSON candidate authority must fail closed");
        assert!(error.contains("input guard") || error.contains("candidate"));
        fs::remove_dir_all(candidate).expect("remove isolated candidate root");
    }
}

#[test]
fn canonical_yaml_authority_uses_hardened_parser_instead_of_bypassing_it() {
    let candidate = candidate_root("yaml");
    copy_from_base(&candidate, ".github/workflows/ci.yml");
    let proof = verify(
        &candidate,
        vec![changed("modified", ".github/workflows/ci.yml", None)],
    )
    .expect("canonical-shaped YAML authority should pass hardened input parsing");
    assert_eq!(proof.mode, "AUTHORITY_VERIFICATION_REQUIRED");
    assert!(!proof.candidate_code_executed);
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn t026_skip_attempt_cannot_disable_base_workflow() {
    let workflow = fs::read_to_string(repo_root().join(".github/workflows/af02-base-verifier.yml"))
        .expect("read AF-02 workflow");
    assert!(workflow.contains("  pull_request_target:"));
    assert!(!workflow.contains("  pull_request:\n"));
    assert!(!workflow.contains("    paths:\n"));
    assert!(workflow.contains("Checkout canonical base"));
    assert!(workflow.contains("Checkout candidate as data only"));
    assert!(workflow.contains(
        "run: bash \"${GITHUB_WORKSPACE}/base/.github/scripts/run_af02_base_verifier.sh\" \"${GITHUB_WORKSPACE}/base\" \"${GITHUB_WORKSPACE}/candidate\""
    ));
}

#[test]
fn t026_authority_rename_and_removal_fail_closed() {
    let candidate = candidate_root("rename");
    let rename = verify(
        &candidate,
        vec![changed(
            "renamed",
            "README.md",
            Some(".github/required-checks.json"),
        )],
    )
    .expect_err("renaming authority out of the protected universe must fail closed");
    assert!(rename.contains("authority rename"));

    let removal = verify(
        &candidate,
        vec![changed("removed", ".github/required-checks.json", None)],
    )
    .expect_err("authority removal must fail closed");
    assert!(removal.contains("authority removal"));
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn t026_base_ref_swap_is_rejected_before_candidate_parsing() {
    let base = repo_root();
    let candidate = candidate_root("base-swap");
    let mut input = test_gate_input(
        &base,
        vec![changed("modified", "README.md", None)],
    )
    .expect("build trusted gate input");
    input.base_sha = "2222222222222222222222222222222222222222".to_owned();
    let bytes = serde_json::to_vec(&input).expect("serialize forged gate input");
    let error = verify_pr(&base, &candidate, &bytes)
        .expect_err("base-ref substitution must fail")
        .to_string();
    assert!(error.contains("header/base identity disagreement"));
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn t026_candidate_verifier_substitution_is_data_only_and_has_no_execution_side_effect() {
    let candidate = candidate_root("substitution");
    let marker = candidate.join("candidate-executed.marker");
    let malicious = format!(
        "fn main() {{ std::fs::write({:?}, b\"executed\").unwrap(); }}\n",
        marker
    );
    write_candidate(
        &candidate,
        "tools/af02-verifier/src/main.rs",
        malicious.as_bytes(),
    );
    let proof = verify(
        &candidate,
        vec![changed("modified", "tools/af02-verifier/src/main.rs", None)],
    )
    .expect("candidate verifier substitution is inspected as bytes only");
    assert_eq!(proof.mode, "AUTHORITY_VERIFICATION_REQUIRED");
    assert!(!proof.candidate_code_executed);
    assert!(!marker.exists(), "candidate verifier content must never execute");
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn t026_parser_exhaustion_fails_before_semantic_acceptance() {
    let candidate = candidate_root("oversize");
    let mut oversized = Vec::with_capacity(1024 * 1024 + 32);
    oversized.extend_from_slice(b"{\"value\":\"");
    oversized.resize(1024 * 1024 + 16, b'a');
    oversized.extend_from_slice(b"\"}");
    write_candidate(&candidate, ".github/required-checks.json", &oversized);
    let error = verify(
        &candidate,
        vec![changed("modified", ".github/required-checks.json", None)],
    )
    .expect_err("oversized authority must fail before parser allocation grows unbounded");
    assert!(error.contains("single-file byte policy"));
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");

    let candidate = candidate_root("deep-json");
    let mut deep = String::new();
    for _ in 0..40 {
        deep.push_str("{\"a\":");
    }
    deep.push('0');
    for _ in 0..40 {
        deep.push('}');
    }
    write_candidate(
        &candidate,
        ".github/required-checks.json",
        deep.as_bytes(),
    );
    assert!(
        verify(
            &candidate,
            vec![changed("modified", ".github/required-checks.json", None)],
        )
        .is_err(),
        "excessive JSON depth must fail closed"
    );
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[cfg(unix)]
#[test]
fn t026_symlink_authority_fails_closed() {
    use std::os::unix::fs::symlink;

    let candidate = candidate_root("symlink");
    let outside_root = candidate_root("outside");
    let outside = outside_root.join("authority.json");
    fs::write(&outside, b"{}").expect("write outside authority target");
    let target = candidate.join(".github/required-checks.json");
    fs::create_dir_all(target.parent().expect("symlink parent")).expect("create symlink parent");
    symlink(&outside, &target).expect("create candidate authority symlink");
    let error = verify(
        &candidate,
        vec![changed("modified", ".github/required-checks.json", None)],
    )
    .expect_err("symlink authority must fail closed");
    assert!(error.contains("symlink"));
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
    fs::remove_dir_all(outside_root).expect("remove outside fixture root");
}

#[test]
fn t026_noncanonical_and_escape_paths_fail_before_classification() {
    let candidate = candidate_root("path-escape");
    for path in [
        "../escape",
        "/absolute",
        "tools//af02-verifier/src/main.rs",
        "tools/./af02-verifier/src/main.rs",
        "tools\\af02-verifier\\src\\main.rs",
    ] {
        assert!(
            verify(&candidate, vec![changed("modified", path, None)]).is_err(),
            "path {path} must fail closed"
        );
    }
    fs::remove_dir_all(candidate).expect("remove isolated candidate root");
}

#[test]
fn t026_runtime_parent_is_pinned_offline_read_only_and_invokes_base_binary_only() {
    let root = repo_root();
    let runner = fs::read_to_string(root.join(".github/scripts/run_af02_base_verifier.sh"))
        .expect("read canonical parent runner");
    let workflow = fs::read_to_string(root.join(".github/workflows/af02-base-verifier.yml"))
        .expect("read canonical workflow");

    assert!(runner.contains("--network=none"));
    assert!(runner.contains("--read-only"));
    assert!(runner.contains("--pids-limit=64"));
    assert!(runner.contains("--memory=512m"));
    assert!(runner.contains("--security-opt=no-new-privileges"));
    assert!(runner.contains("/workspace/base/target/af02-verifier/release/commandf-af02-verifier"));
    assert!(runner.contains("\"verify-pr\""));
    assert!(runner.contains("validate-input-process-evidence"));
    assert!(!runner.contains("/workspace/candidate/target/"));
    assert!(!runner.contains("candidate/.github/scripts/"));

    assert!(workflow.contains("docker pull \"docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f\""));
    assert!(workflow.contains("docker run --rm --pull=never"));
    assert!(workflow.contains("--mount \"type=bind,src=${GITHUB_WORKSPACE}/base,dst=/workspace\""));
    assert!(workflow.contains("--workdir /workspace/tools/af02-verifier"));
    assert!(workflow.contains("cargo build --locked --release --target-dir /workspace/target/af02-verifier"));
}
