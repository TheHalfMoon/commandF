use std::process::Command;

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

#[test]
fn check_usage_errors_are_operational_exit_one() {
    let output = commandf()
        .args([
            "check",
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
        .expect("commandf check parse failure must execute");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "stderr: {stderr}");
    assert!(stderr.contains("invalid-threshold"), "stderr: {stderr}");
}
