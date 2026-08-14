use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestDir(PathBuf);

impl TestDir {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "commandf-cf08-{label}-{}-{nonce}",
            process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

fn empty_check_report() -> String {
    format!(
        "{{\n  \"schema\": 1,\n  \"policy\": {{\"direction\": \"both\", \"fail_on\": \"breaking\"}},\n  \"decision\": {{\"passed\": true, \"total_findings\": 0, \"selected_findings\": 0, \"breaking_findings\": 0, \"risky_findings\": 0, \"additive_findings\": 0, \"blocking_findings\": 0}},\n  \"compatibility\": {{\"schema\": 1, \"ruleset\": \"cf04-rules-v1\", \"package_name\": \"example.package\", \"before\": {{\"version\": \"1.0.0\", \"archive_sha256\": \"{}\"}}, \"after\": {{\"version\": \"1.0.0\", \"archive_sha256\": \"{}\"}}, \"findings\": []}}\n}}\n",
        "a".repeat(64),
        "a".repeat(64)
    )
}

fn write(path: &Path, content: &[u8]) {
    fs::write(path, content).unwrap();
}

#[test]
fn github_annotations_help_succeeds() {
    let output = commandf()
        .args(["github-annotations", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--input"));
}

#[test]
fn github_annotations_requires_input() {
    let output = commandf().arg("github-annotations").output().unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn github_annotations_empty_valid_report_emits_nothing() {
    let temp = TestDir::new("empty");
    let report = temp.path().join("check.json");
    write(&report, empty_check_report().as_bytes());

    let output = commandf()
        .args(["github-annotations", "--input"])
        .arg(&report)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn github_annotations_policy_failed_report_still_exits_zero() {
    let temp = TestDir::new("policy-failed");
    let report = temp.path().join("check.json");
    let content = empty_check_report()
        .replace("\"passed\": true", "\"passed\": false")
        .replace("\"blocking_findings\": 0", "\"blocking_findings\": 1");
    write(&report, content.as_bytes());

    let output = commandf()
        .args(["github-annotations", "--input"])
        .arg(&report)
        .output()
        .unwrap();
    assert!(output.status.success());
}

#[test]
fn github_annotations_malformed_json_fails_operationally() {
    let temp = TestDir::new("malformed");
    let report = temp.path().join("check.json");
    write(&report, b"{not-json");

    let output = commandf()
        .args(["github-annotations", "--input"])
        .arg(&report)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("JSON"));
}

#[test]
fn github_annotations_oversized_input_fails_before_json_parse() {
    let temp = TestDir::new("oversized");
    let report = temp.path().join("large.json");
    let file = fs::File::create(&report).unwrap();
    file.set_len(64 * 1024 * 1024 + 1).unwrap();

    let output = commandf()
        .args(["github-annotations", "--input"])
        .arg(&report)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("byte limit"));
}
