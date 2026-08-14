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
        let path =
            std::env::temp_dir().join(format!("commandf-cf08-{label}-{}-{nonce}", process::id()));
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

fn failed_check_report() -> String {
    let finding = "{\"rule_id\":\"CF04-TEST-001\",\"severity\":\"BREAKING\",\"direction\":\"producer\",\"source_kind\":\"element_field_changed\",\"message\":\"synthetic breaking finding\",\"resource\":{\"kind\":\"canonical\",\"value\":\"http://example.org/StructureDefinition/example\"},\"after_filename\":\"StructureDefinition-example.json\"}";
    empty_check_report()
        .replace("\"passed\": true", "\"passed\": false")
        .replace("\"total_findings\": 0", "\"total_findings\": 1")
        .replace("\"selected_findings\": 0", "\"selected_findings\": 1")
        .replace("\"breaking_findings\": 0", "\"breaking_findings\": 1")
        .replace("\"blocking_findings\": 0", "\"blocking_findings\": 1")
        .replace("\"findings\": []", &format!("\"findings\": [{finding}]"))
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
    assert!(stdout.contains("--fsh-index"));
    assert!(stdout.contains("--repo-root"));
    assert!(stdout.contains("--fsh-root"));
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
    write(&report, failed_check_report().as_bytes());

    let output = commandf()
        .args(["github-annotations", "--input"])
        .arg(&report)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .starts_with("::error title=commandF CF04-TEST-001::"));
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
    assert!(stderr.starts_with("commandf: "));
    assert!(stderr.contains("line 1 column"));
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

#[test]
fn source_map_cli_maps_sushi_definition_range_and_renderer_uses_it() {
    let temp = TestDir::new("source-map-integration");
    let fsh_root = temp.path().join("input/fsh");
    fs::create_dir_all(&fsh_root).unwrap();
    write(
        &fsh_root.join("example.fsh"),
        b"Profile: Example\nParent: Observation\n* status 1..1\n* code 1..1\n",
    );

    let report = temp.path().join("check.json");
    let index = temp.path().join("fsh-index.json");
    let mapped = temp.path().join("mapped.json");
    write(&report, failed_check_report().as_bytes());
    write(
        &index,
        br#"[
  {
    "outputFile": "StructureDefinition-example.json",
    "fshFile": "example.fsh",
    "fshName": "Example",
    "fshType": "Profile",
    "startLine": 1,
    "endLine": 4
  }
]
"#,
    );

    let source_map = commandf()
        .args(["source-map", "--input"])
        .arg(&report)
        .arg("--fsh-index")
        .arg(&index)
        .arg("--repo-root")
        .arg(temp.path())
        .args(["--fsh-root", "input/fsh", "--output"])
        .arg(&mapped)
        .output()
        .unwrap();
    assert!(source_map.status.success());

    let mapped_text = fs::read_to_string(&mapped).unwrap();
    assert!(mapped_text.contains("\"status\": \"mapped\""));
    assert!(mapped_text.contains("\"file\": \"input/fsh/example.fsh\""));
    assert!(mapped_text.contains("\"line\": 1"));
    assert!(mapped_text.contains("\"end_line\": 4"));

    let annotations = commandf()
        .args(["github-annotations", "--input"])
        .arg(&report)
        .arg("--source-map")
        .arg(&mapped)
        .arg("--fsh-index")
        .arg(&index)
        .arg("--repo-root")
        .arg(temp.path())
        .args(["--fsh-root", "input/fsh"])
        .output()
        .unwrap();
    assert!(annotations.status.success());
    let stdout = String::from_utf8(annotations.stdout).unwrap();
    assert!(stdout.starts_with(
        "::error title=commandF CF04-TEST-001,file=input/fsh/example.fsh,line=1,endLine=4::"
    ));
    assert!(stdout.contains("exact rule-line attribution not proven"));
}

#[test]
fn mapped_projection_requires_current_source_evidence_inputs() {
    let temp = TestDir::new("source-map-proof-inputs");
    let report = temp.path().join("check.json");
    let mapped = temp.path().join("mapped.json");
    write(&report, empty_check_report().as_bytes());
    write(&mapped, b"{}\n");

    let output = commandf()
        .args(["github-annotations", "--input"])
        .arg(&report)
        .arg("--source-map")
        .arg(&mapped)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stderr).unwrap().contains(
        "requires --source-map%2C --fsh-index%2C --repo-root%2C and --fsh-root together"
    ));
}

#[test]
fn malicious_source_map_diagnostic_cannot_inject_workflow_command() {
    let temp = TestDir::new("source-map-diagnostic-injection");
    let fsh_root = temp.path().join("input/fsh");
    fs::create_dir_all(&fsh_root).unwrap();
    write(&fsh_root.join("example.fsh"), b"Profile: Example\n");

    let report = temp.path().join("check.json");
    let index = temp.path().join("fsh-index.json");
    write(&report, failed_check_report().as_bytes());
    write(
        &index,
        br#"[
  {
    "outputFile": "bad/\n::warning title=pwned::injected",
    "fshFile": "example.fsh",
    "fshName": "Example",
    "fshType": "Profile",
    "startLine": 1,
    "endLine": 1
  }
]
"#,
    );

    let output = commandf()
        .args(["source-map", "--input"])
        .arg(&report)
        .arg("--fsh-index")
        .arg(&index)
        .arg("--repo-root")
        .arg(temp.path())
        .args(["--fsh-root", "input/fsh"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 1);
    assert!(!stderr.contains("::warning"));
    assert!(!stderr.contains("\n::warning"));
    assert!(stderr.contains("%0A%3A%3Awarning"));
}
