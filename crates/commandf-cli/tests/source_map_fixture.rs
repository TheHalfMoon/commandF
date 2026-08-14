use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

fn failed_check_report() -> String {
    format!(
        "{{\n  \"schema\": 1,\n  \"policy\": {{\"direction\": \"both\", \"fail_on\": \"breaking\"}},\n  \"decision\": {{\"passed\": false, \"total_findings\": 1, \"selected_findings\": 1, \"breaking_findings\": 1, \"risky_findings\": 0, \"additive_findings\": 0, \"blocking_findings\": 1}},\n  \"compatibility\": {{\"schema\": 1, \"ruleset\": \"cf04-rules-v1\", \"package_name\": \"example.package\", \"before\": {{\"version\": \"1.0.0\", \"archive_sha256\": \"{}\"}}, \"after\": {{\"version\": \"1.1.0\", \"archive_sha256\": \"{}\"}}, \"findings\": [{{\"rule_id\":\"CF04-TEST-001\",\"severity\":\"BREAKING\",\"direction\":\"producer\",\"source_kind\":\"element_field_changed\",\"message\":\"synthetic breaking finding\",\"resource\":{{\"kind\":\"canonical\",\"value\":\"http://example.org/StructureDefinition/example\"}},\"after_filename\":\"StructureDefinition-example.json\"}}]}}\n}}\n",
        "a".repeat(64),
        "b".repeat(64)
    )
}

#[test]
fn committed_sushi_shaped_fixture_maps_and_renders() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cf09");
    let index = fixture_root.join("fsh-index.json");
    let temp = tempdir().unwrap();
    let report = temp.path().join("check.json");
    let mapped = temp.path().join("mapped.json");
    fs::write(&report, failed_check_report()).unwrap();

    let source_map = commandf()
        .args(["source-map", "--input"])
        .arg(&report)
        .arg("--fsh-index")
        .arg(&index)
        .arg("--repo-root")
        .arg(&fixture_root)
        .args(["--fsh-root", "input/fsh", "--output"])
        .arg(&mapped)
        .output()
        .unwrap();
    assert!(source_map.status.success());

    let annotations = commandf()
        .args(["github-annotations", "--input"])
        .arg(&report)
        .arg("--source-map")
        .arg(&mapped)
        .output()
        .unwrap();
    assert!(annotations.status.success());
    let stdout = String::from_utf8(annotations.stdout).unwrap();
    assert!(stdout.starts_with(
        "::error title=commandF CF04-TEST-001,file=input/fsh/example.fsh,line=1,endLine=4::"
    ));
    assert!(stdout.contains("exact rule-line attribution not proven"));
}
