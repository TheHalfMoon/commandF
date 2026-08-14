use std::fs;

use commandf_pkg::{
    build_source_mapped_check_report, evaluate_compatibility_policy,
    source_mapped_check_report_to_github_annotations_bytes, CheckPolicy, CompatibilityDirection,
    CompatibilityFinding, CompatibilityReport, CompatibilitySeverity, PackageEvidence, ResourceKey,
    ResourceKeyKind, SourceMapError, SourceMappingStatus, StructuralChangeKind,
};
use serde_json::json;
use tempfile::tempdir;

fn finding(rule_id: &str, after_filename: Option<&str>) -> CompatibilityFinding {
    CompatibilityFinding {
        rule_id: rule_id.to_owned(),
        severity: CompatibilitySeverity::Breaking,
        direction: CompatibilityDirection::Producer,
        source_kind: StructuralChangeKind::ElementFieldChanged,
        message: format!("{rule_id} synthetic finding."),
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: "http://example.org/StructureDefinition/example".to_owned(),
        },
        before_filename: Some("StructureDefinition-example.json".to_owned()),
        after_filename: after_filename.map(str::to_owned),
        view: None,
        element_id: Some("Observation.status".to_owned()),
        field: Some("min".to_owned()),
        before: None,
        after: None,
    }
}

fn report(findings: Vec<CompatibilityFinding>) -> commandf_pkg::CheckReport {
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
        findings,
    };
    evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap()
}

fn repo_with_fsh() -> tempfile::TempDir {
    let temp = tempdir().unwrap();
    fs::create_dir_all(temp.path().join("input/fsh/nested")).unwrap();
    fs::write(
        temp.path().join("input/fsh/nested/example.fsh"),
        "Alias: $x = http://example.org\n\nProfile: Example\nParent: Observation\n* status 1..1\n* code 1..1\n",
    )
    .unwrap();
    temp
}

fn index_bytes(output_file: &str, fsh_file: &str, start_line: u32, end_line: u32) -> Vec<u8> {
    serde_json::to_vec(&json!([{
        "outputFile": output_file,
        "fshFile": fsh_file,
        "fshName": "Example",
        "fshType": "Profile",
        "startLine": start_line,
        "endLine": end_line,
        "futureMetadata": {"ignored": true}
    }]))
    .unwrap()
}

#[test]
fn exact_after_filename_maps_to_sushi_definition_range_and_renders_location() {
    let repo = repo_with_fsh();
    let report = report(vec![finding(
        "CF04-CARD-001",
        Some("StructureDefinition-example.json"),
    )]);
    let index = index_bytes(
        "StructureDefinition-example.json",
        "nested\\example.fsh",
        3,
        6,
    );

    let first = build_source_mapped_check_report(
        &report,
        &index,
        repo.path(),
        std::path::Path::new("input/fsh"),
    )
    .unwrap();
    let second = build_source_mapped_check_report(
        &report,
        &index,
        repo.path(),
        std::path::Path::new("input/fsh"),
    )
    .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.to_json_bytes().unwrap(), second.to_json_bytes().unwrap());
    assert_eq!(first.source_index.fsh_root, "input/fsh");
    assert_eq!(first.mappings.len(), 1);
    assert_eq!(first.mappings[0].status, SourceMappingStatus::Mapped);
    let location = first.mappings[0].location.as_ref().unwrap();
    assert_eq!(location.file, "input/fsh/nested/example.fsh");
    assert_eq!(location.line, 3);
    assert_eq!(location.end_line, 6);

    let text = String::from_utf8(
        source_mapped_check_report_to_github_annotations_bytes(&report, &first).unwrap(),
    )
    .unwrap();
    assert!(text.starts_with(
        "::error title=commandF CF04-CARD-001,file=input/fsh/nested/example.fsh,line=3,endLine=6::"
    ));
    assert!(text.contains("exact rule-line attribution not proven"));
}

#[test]
fn current_tree_unmapped_states_do_not_fabricate_locations() {
    let repo = repo_with_fsh();
    let report = report(vec![
        finding("CF04-REMOVE-001", None),
        finding("CF04-MISSING-001", Some("StructureDefinition-missing.json")),
    ]);
    let index = index_bytes(
        "StructureDefinition-example.json",
        "nested/example.fsh",
        3,
        6,
    );

    let mapped = build_source_mapped_check_report(
        &report,
        &index,
        repo.path(),
        std::path::Path::new("input/fsh"),
    )
    .unwrap();
    assert_eq!(
        mapped.mappings[0].status,
        SourceMappingStatus::UnmappedNoAfterFilename
    );
    assert_eq!(
        mapped.mappings[1].status,
        SourceMappingStatus::UnmappedNoIndexEntry
    );
    assert!(mapped.mappings.iter().all(|entry| entry.location.is_none()));

    let text = String::from_utf8(
        source_mapped_check_report_to_github_annotations_bytes(&report, &mapped).unwrap(),
    )
    .unwrap();
    assert!(!text.contains(",file="));
    assert!(text.contains("no proven current FSH source mapping"));
}

#[test]
fn duplicate_generated_output_identity_fails_closed() {
    let repo = repo_with_fsh();
    let report = report(vec![finding(
        "CF04-CARD-001",
        Some("StructureDefinition-example.json"),
    )]);
    let index = serde_json::to_vec(&json!([
        {
            "outputFile": "StructureDefinition-example.json",
            "fshFile": "nested/example.fsh",
            "fshName": "Example",
            "fshType": "Profile",
            "startLine": 3,
            "endLine": 6
        },
        {
            "outputFile": "StructureDefinition-example.json",
            "fshFile": "nested/other.fsh",
            "fshName": "Other",
            "fshType": "Profile",
            "startLine": 8,
            "endLine": 10
        }
    ]))
    .unwrap();

    assert!(matches!(
        build_source_mapped_check_report(
            &report,
            &index,
            repo.path(),
            std::path::Path::new("input/fsh"),
        ),
        Err(SourceMapError::DuplicateOutputFile(_))
    ));
}

#[test]
fn malformed_ranges_and_traversal_fail_closed() {
    let repo = repo_with_fsh();
    let report = report(vec![finding(
        "CF04-CARD-001",
        Some("StructureDefinition-example.json"),
    )]);

    let invalid_range = index_bytes(
        "StructureDefinition-example.json",
        "nested/example.fsh",
        7,
        6,
    );
    assert!(matches!(
        build_source_mapped_check_report(
            &report,
            &invalid_range,
            repo.path(),
            std::path::Path::new("input/fsh"),
        ),
        Err(SourceMapError::InvalidIndex(_))
    ));

    let traversal = index_bytes(
        "StructureDefinition-example.json",
        "../escape.fsh",
        3,
        6,
    );
    assert!(matches!(
        build_source_mapped_check_report(
            &report,
            &traversal,
            repo.path(),
            std::path::Path::new("input/fsh"),
        ),
        Err(SourceMapError::InvalidPath(_))
    ));
}

#[cfg(unix)]
#[test]
fn symlink_escape_fails_closed() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let repo_root = temp.path().join("repo");
    let fsh_root = repo_root.join("input/fsh");
    fs::create_dir_all(&fsh_root).unwrap();
    let outside = temp.path().join("outside.fsh");
    fs::write(&outside, "Profile: Outside\nParent: Observation\n").unwrap();
    symlink(&outside, fsh_root.join("escape.fsh")).unwrap();

    let report = report(vec![finding(
        "CF04-CARD-001",
        Some("StructureDefinition-example.json"),
    )]);
    let index = index_bytes(
        "StructureDefinition-example.json",
        "escape.fsh",
        1,
        2,
    );
    assert!(matches!(
        build_source_mapped_check_report(
            &report,
            &index,
            &repo_root,
            std::path::Path::new("input/fsh"),
        ),
        Err(SourceMapError::SourceEscape(_))
    ));
}

#[test]
fn renderer_rejects_a_source_map_for_a_different_valid_check_report() {
    let repo = repo_with_fsh();
    let first_report = report(vec![finding(
        "CF04-CARD-001",
        Some("StructureDefinition-example.json"),
    )]);
    let second_report = report(vec![finding(
        "CF04-CARD-002",
        Some("StructureDefinition-example.json"),
    )]);
    let index = index_bytes(
        "StructureDefinition-example.json",
        "nested/example.fsh",
        3,
        6,
    );
    let mapped = build_source_mapped_check_report(
        &first_report,
        &index,
        repo.path(),
        std::path::Path::new("input/fsh"),
    )
    .unwrap();

    assert!(matches!(
        source_mapped_check_report_to_github_annotations_bytes(&second_report, &mapped),
        Err(SourceMapError::CheckReportMismatch)
    ));
}

#[test]
fn renderer_rejects_tampered_location_outside_declared_fsh_root() {
    let repo = repo_with_fsh();
    let report = report(vec![finding(
        "CF04-CARD-001",
        Some("StructureDefinition-example.json"),
    )]);
    let index = index_bytes(
        "StructureDefinition-example.json",
        "nested/example.fsh",
        3,
        6,
    );
    let mut mapped = build_source_mapped_check_report(
        &report,
        &index,
        repo.path(),
        std::path::Path::new("input/fsh"),
    )
    .unwrap();
    mapped.mappings[0].location.as_mut().unwrap().file = "README.md".to_owned();

    assert!(matches!(
        source_mapped_check_report_to_github_annotations_bytes(&report, &mapped),
        Err(SourceMapError::InvalidMappingEntry { index: 0 })
    ));
}
