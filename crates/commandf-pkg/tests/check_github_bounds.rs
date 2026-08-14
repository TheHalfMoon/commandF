use commandf_pkg::{
    check_report_to_github_annotations_bytes, evaluate_compatibility_policy, CheckPolicy,
    CompatibilityDirection, CompatibilityFinding, CompatibilityReport, CompatibilitySeverity,
    PackageEvidence, ResourceKey, ResourceKeyKind, StructuralChangeKind,
};

#[test]
fn annotation_title_and_message_are_bounded() {
    let finding = CompatibilityFinding {
        rule_id: format!("CF04-{}", "R".repeat(1_000)),
        severity: CompatibilitySeverity::Breaking,
        direction: CompatibilityDirection::Producer,
        source_kind: StructuralChangeKind::ElementFieldChanged,
        message: "M".repeat(10_000),
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: "http://example.org/StructureDefinition/example".to_owned(),
        },
        before_filename: None,
        after_filename: None,
        view: None,
        element_id: None,
        field: None,
        before: None,
        after: None,
    };
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
        findings: vec![finding],
    };
    let report = evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    let text = String::from_utf8(check_report_to_github_annotations_bytes(&report).unwrap()).unwrap();

    assert_eq!(text.lines().count(), 1);
    assert!(text.contains("[title truncated]"));
    assert!(text.contains("[projection truncated]"));
    assert!(text.len() < 5_000);
}
