use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const WORKFLOW: &str = ".github/workflows/af02-base-verifier.yml";
const RUNNER: &str = ".github/scripts/run_af02_base_verifier.sh";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("AF-02 verifier must live under tools/ in the repository")
        .to_path_buf()
}

#[test]
fn base_gate_runner_self_test_passes() {
    let root = repo_root();
    let output = Command::new("bash")
        .arg(root.join(RUNNER))
        .arg("--self-test")
        .output()
        .expect("run canonical AF-02 base-gate self-test");
    assert!(
        output.status.success(),
        "base-gate self-test failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("self-test output must be UTF-8");
    let value: serde_json::Value = serde_json::from_str(stdout.trim()).expect("self-test JSON");
    assert_eq!(
        value["schema"],
        "commandf.af02-base-gate-self-test/v1",
        "self-test schema must remain versioned"
    );
    assert_eq!(value["result"], "PASS");
    assert_eq!(value["test_count"], 8);
}

#[test]
fn pull_request_target_workflow_is_base_controlled_and_read_only() {
    let root = repo_root();
    let workflow = fs::read_to_string(root.join(WORKFLOW)).expect("read AF-02 base workflow");

    assert!(workflow.contains("  pull_request_target:\n"));
    assert!(!workflow.contains("  pull_request:\n"));
    assert!(!workflow.contains("    paths:\n"));
    assert_eq!(workflow.matches("uses: actions/checkout@").count(), 2);
    assert_eq!(workflow.matches("persist-credentials: false").count(), 2);
    assert!(workflow.contains("      contents: read\n"));
    assert!(!workflow.contains("pull-requests:"));
    assert!(!workflow.contains("contents: write"));
    assert!(!workflow.contains("checks: write"));
    assert!(!workflow.contains("GITHUB_TOKEN"));
    assert!(!workflow.contains("github.token"));
    assert!(workflow.contains("path: base\n"));
    assert!(workflow.contains("path: candidate\n"));
    assert!(workflow.contains(
        "run: bash \"${GITHUB_WORKSPACE}/base/.github/scripts/run_af02_base_verifier.sh\" \"${GITHUB_WORKSPACE}/base\" \"${GITHUB_WORKSPACE}/candidate\""
    ));
    assert!(!workflow.contains("candidate/.github/scripts/"));
}

#[test]
fn base_gate_runner_has_no_candidate_execution_or_token_escape_hatch() {
    let root = repo_root();
    let runner = fs::read_to_string(root.join(RUNNER)).expect("read AF-02 base runner");

    assert!(runner.starts_with("#!/usr/bin/env bash\nset -euo pipefail\n"));
    assert!(runner.contains("\"candidate_code_executed\": False"));
    assert!(runner.contains("git_head(candidate_root)"));
    assert!(runner.contains("GITHUB_API_ROOT = \"https://api.github.com\""));
    assert!(!runner.contains("GITHUB_TOKEN"));
    assert!(!runner.contains("Authorization"));
    assert!(!runner.contains("cargo run"));
    assert!(!runner.contains("cargo build"));
    assert!(!runner.contains("eval "));
    assert!(!runner.contains("source "));
    assert!(!runner.contains("candidate/.github/scripts/"));
    assert!(!runner.contains("candidate_root / \".github/scripts"));
}
