use std::fs;
use std::path::Path;

use commandf_pkg::{
    build_source_mapped_check_report, evaluate_compatibility_policy,
    source_mapped_check_report_to_github_annotations_bytes, CheckPolicy, CompatibilityDirection,
    CompatibilityFinding, CompatibilityReport, CompatibilitySeverity, PackageEvidence, ResourceKey,
    ResourceKeyKind, SourceMapError, StructuralChangeKind, MAX_SUSHI_INDEX_ENTRIES,
    MAX_SUSHI_INDEX_INPUT_BYTES,
};
use serde_json::json;
use tempfile::tempdir;

fn finding() -> CompatibilityFinding {
    CompatibilityFinding {
        rule_id: "CF04-TEST-001".to_owned(),
        severity: CompatibilitySeverity::Breaking,
        direction: CompatibilityDirection::Producer,
        source_kind: StructuralChangeKind::ElementFieldChanged,
        message: "synthetic breaking finding".to_owned(),
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: "http://example.org/StructureDefinition/example".to_owned(),
        },
        before_filename: Some("StructureDefinition-example.json".to_owned()),
        after_filename: Some("StructureDefinition-example.json".to_owned()),
        view: None,
        element_id: Some("Observation.status".to_owned()),
        field: Some("min".to_owned()),
        before: None,
        after: None,
    }
}

fn report() -> commandf_pkg::CheckReport {
    let compatibility = CompatibilityReport {
        schema: CompatibilityReport::SCHEMA_V1,
        ruleset: CompatibilityReport::RULESET_V1.to_owned(),
        package_name: "example.package".to_owned(),
        before: PackageEvidence {
            version: "1.0.0".to_owned(),
            archive_sha256: "a".repeat(64),
        },
        after: PackageEvidence {
            version: "1.1.0".to_owned(),
            archive_sha256: "b".repeat(64),
        },
        findings: vec![finding()],
    };
    evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap()
}

fn index_bytes(fsh_file: &str) -> Vec<u8> {
    serde_json::to_vec(&json!([{
        "outputFile": "StructureDefinition-example.json",
        "fshFile": fsh_file,
        "fshName": "Example",
        "fshType": "Profile",
        "startLine": 1,
        "endLine": 4
    }]))
    .unwrap()
}

fn repo_with_source(name: &str) -> tempfile::TempDir {
    let repo = tempdir().unwrap();
    let root = repo.path().join("input/fsh");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join(name),
        "Profile: Example\nParent: Observation\n* status 1..1\n* code 1..1\n",
    )
    .unwrap();
    repo
}

#[test]
fn malformed_index_field_types_fail_closed() {
    let repo = repo_with_source("example.fsh");
    let malformed = br#"[
      {
        "outputFile": "StructureDefinition-example.json",
        "fshFile": "example.fsh",
        "fshName": "Example",
        "fshType": "Profile",
        "startLine": "1",
        "endLine": 4
      }
    ]"#;

    assert!(matches!(
        build_source_mapped_check_report(
            &report(),
            malformed,
            repo.path(),
            Path::new("input/fsh"),
        ),
        Err(SourceMapError::InvalidIndex(_))
    ));
}

#[test]
fn absolute_source_and_fsh_root_paths_fail_closed() {
    let repo = repo_with_source("example.fsh");

    assert!(matches!(
        build_source_mapped_check_report(
            &report(),
            &index_bytes("/tmp/example.fsh"),
            repo.path(),
            Path::new("input/fsh"),
        ),
        Err(SourceMapError::InvalidPath(_))
    ));

    assert!(matches!(
        build_source_mapped_check_report(
            &report(),
            &index_bytes("example.fsh"),
            repo.path(),
            Path::new("/tmp"),
        ),
        Err(SourceMapError::InvalidPath(_))
    ));
}

#[test]
fn missing_and_non_file_sources_fail_closed() {
    let repo = tempdir().unwrap();
    fs::create_dir_all(repo.path().join("input/fsh/not-a-file")).unwrap();

    assert!(matches!(
        build_source_mapped_check_report(
            &report(),
            &index_bytes("missing.fsh"),
            repo.path(),
            Path::new("input/fsh"),
        ),
        Err(SourceMapError::MissingSource(_))
    ));

    assert!(matches!(
        build_source_mapped_check_report(
            &report(),
            &index_bytes("not-a-file"),
            repo.path(),
            Path::new("input/fsh"),
        ),
        Err(SourceMapError::MissingSource(_))
    ));
}

#[test]
fn source_index_byte_overflow_fails_before_json_parse() {
    let repo = repo_with_source("example.fsh");
    let oversized = vec![b' '; MAX_SUSHI_INDEX_INPUT_BYTES + 1];

    assert!(matches!(
        build_source_mapped_check_report(
            &report(),
            &oversized,
            repo.path(),
            Path::new("input/fsh"),
        ),
        Err(SourceMapError::IndexTooLarge { .. })
    ));
}

#[test]
fn source_index_entry_count_overflow_fails_closed() {
    let repo = repo_with_source("example.fsh");
    let entries = (0..=MAX_SUSHI_INDEX_ENTRIES)
        .map(|index| {
            json!({
                "outputFile": format!("s{index}"),
                "fshFile": "example.fsh",
                "fshName": "E",
                "fshType": "P",
                "startLine": 1,
                "endLine": 1
            })
        })
        .collect::<Vec<_>>();
    let bytes = serde_json::to_vec(&entries).unwrap();
    assert!(bytes.len() <= MAX_SUSHI_INDEX_INPUT_BYTES);

    assert!(matches!(
        build_source_mapped_check_report(
            &report(),
            &bytes,
            repo.path(),
            Path::new("input/fsh"),
        ),
        Err(SourceMapError::TooManyEntries { .. })
    ));
}

#[test]
fn inconsistent_check_decision_fails_before_mapping() {
    let repo = repo_with_source("example.fsh");
    let mut report = report();
    report.decision.passed = true;

    assert!(build_source_mapped_check_report(
        &report,
        &index_bytes("example.fsh"),
        repo.path(),
        Path::new("input/fsh"),
    )
    .is_err());
}

#[test]
fn persisted_source_index_entry_count_overflow_is_rejected() {
    let repo = repo_with_source("example.fsh");
    let report = report();
    let mut mapped = build_source_mapped_check_report(
        &report,
        &index_bytes("example.fsh"),
        repo.path(),
        Path::new("input/fsh"),
    )
    .unwrap();
    mapped.source_index.entries = MAX_SUSHI_INDEX_ENTRIES + 1;

    assert!(matches!(
        source_mapped_check_report_to_github_annotations_bytes(&report, &mapped),
        Err(SourceMapError::TooManyEntries { .. })
    ));
}

#[test]
fn mapped_file_workflow_properties_are_escaped() {
    let repo = repo_with_source("weird,%name.fsh");
    let report = report();
    let mapped = build_source_mapped_check_report(
        &report,
        &index_bytes("weird,%name.fsh"),
        repo.path(),
        Path::new("input/fsh"),
    )
    .unwrap();

    let annotations = String::from_utf8(
        source_mapped_check_report_to_github_annotations_bytes(&report, &mapped).unwrap(),
    )
    .unwrap();
    assert!(annotations.contains("file=input/fsh/weird%2C%25name.fsh,line=1,endLine=4"));
    assert_eq!(annotations.lines().count(), 1);
}
