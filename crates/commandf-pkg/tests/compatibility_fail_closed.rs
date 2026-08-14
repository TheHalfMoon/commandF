use commandf_pkg::{
    classify_structural_diff, CompatibilityError, ElementView, PackageEvidence, ResourceKey,
    ResourceKeyKind, StructuralChange, StructuralChangeKind, StructuralDiffReport,
};
use serde_json::json;

fn report(change: StructuralChange) -> StructuralDiffReport {
    StructuralDiffReport {
        schema: StructuralDiffReport::SCHEMA_V1,
        package_name: "example.fhir.ig".to_owned(),
        before: PackageEvidence {
            version: "1.0.0".to_owned(),
            archive_sha256: "a".repeat(64),
        },
        after: PackageEvidence {
            version: "1.1.0".to_owned(),
            archive_sha256: "b".repeat(64),
        },
        changes: vec![change],
    }
}

fn constraint_change(before: serde_json::Value, after: serde_json::Value) -> StructuralChange {
    StructuralChange {
        kind: StructuralChangeKind::ElementFieldChanged,
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: "http://example.org/StructureDefinition/example".to_owned(),
        },
        before_filename: Some("StructureDefinition-example.json".to_owned()),
        after_filename: Some("StructureDefinition-example.json".to_owned()),
        view: Some(ElementView::Snapshot),
        element_id: Some("Observation".to_owned()),
        field: Some("constraint".to_owned()),
        before: Some(before),
        after: Some(after),
    }
}

#[test]
fn duplicate_constraint_keys_fail_closed_before_classification() {
    let change = constraint_change(
        json!([
            {"key": "obs-1", "severity": "error", "expression": "status.exists()"},
            {"key": "obs-1", "severity": "warning", "expression": "code.exists()"}
        ]),
        json!([]),
    );
    let error = classify_structural_diff(&report(change)).unwrap_err();
    assert!(matches!(
        error,
        CompatibilityError::InvalidChangeValue { ref field, ref message }
            if field == "constraint" && message.contains("duplicate constraint key")
    ));
}

#[test]
fn distinct_constraint_keys_continue_to_classify() {
    let change = constraint_change(
        json!([
            {"key": "obs-1", "severity": "error", "expression": "status.exists()"},
            {"key": "obs-2", "severity": "warning", "expression": "code.exists()"}
        ]),
        json!([]),
    );
    let classified = classify_structural_diff(&report(change)).unwrap();
    assert!(!classified.findings.is_empty());
}
