use std::process::Command;

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

#[test]
fn gate_usage_errors_are_operational_exit_one() {
    let output = commandf()
        .args([
            "gate",
            "example.package",
            "--before-lock",
            "before.lock",
            "--before-cache",
            "before-cache",
            "--after-lock",
            "after.lock",
            "--after-cache",
            "after-cache",
            "--fail-on",
            "invalid-threshold",
        ])
        .output()
        .expect("commandf gate parse failure must execute");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "stderr: {stderr}");
    assert!(stderr.contains("invalid-threshold"), "stderr: {stderr}");
}

#[test]
fn gate_help_remains_success() {
    let output = commandf()
        .args(["gate", "--help"])
        .output()
        .expect("commandf gate help must execute");

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn unrelated_clap_usage_behavior_remains_exit_two() {
    let output = commandf()
        .args(["inspect"])
        .output()
        .expect("commandf unrelated parse failure must execute");

    assert_eq!(output.status.code(), Some(2));
}
