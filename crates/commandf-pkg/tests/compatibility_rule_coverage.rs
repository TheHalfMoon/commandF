use commandf_pkg::{
    classify_structural_diff, CompatibilitySeverity, ElementView, PackageEvidence, ResourceKey,
    ResourceKeyKind, StructuralChange, StructuralChangeKind, StructuralDiffReport,
};
use serde_json::json;

fn resource() -> ResourceKey {
    ResourceKey {
        kind: ResourceKeyKind::Canonical,
        value: "http://example.org/StructureDefinition/example".to_owned(),
    }
}

fn report(changes: Vec<StructuralChange>) -> StructuralDiffReport {
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
        changes,
    }
}

fn element_field(
    element_id: &str,
    field: &str,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
) -> StructuralChange {
    StructuralChange {
        kind: StructuralChangeKind::ElementFieldChanged,
        resource: resource(),
        before_filename: Some("before.json".to_owned()),
        after_filename: Some("after.json".to_owned()),
        view: Some(ElementView::Snapshot),
        element_id: Some(element_id.to_owned()),
        field: Some(field.to_owned()),
        before,
        after,
    }
}

fn structure_field(
    field: &str,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
) -> StructuralChange {
    StructuralChange {
        kind: StructuralChangeKind::StructureFieldChanged,
        resource: resource(),
        before_filename: Some("before.json".to_owned()),
        after_filename: Some("after.json".to_owned()),
        view: None,
        element_id: None,
        field: Some(field.to_owned()),
        before,
        after,
    }
}

fn view_change(kind: StructuralChangeKind, view: ElementView) -> StructuralChange {
    StructuralChange {
        kind,
        resource: resource(),
        before_filename: Some("before.json".to_owned()),
        after_filename: Some("after.json".to_owned()),
        view: Some(view),
        element_id: None,
        field: None,
        before: None,
        after: None,
    }
}

fn element_change(kind: StructuralChangeKind, view: ElementView, id: &str) -> StructuralChange {
    StructuralChange {
        kind,
        resource: resource(),
        before_filename: Some("before.json".to_owned()),
        after_filename: Some("after.json".to_owned()),
        view: Some(view),
        element_id: Some(id.to_owned()),
        field: None,
        before: None,
        after: None,
    }
}

#[test]
fn repeated_classification_is_byte_identical() {
    let input = report(vec![
        element_field(
            "Observation.note",
            "maxLength",
            Some(json!(20)),
            Some(json!(10)),
        ),
        structure_field("baseDefinition", Some(json!("old")), Some(json!("new"))),
    ]);
    let first = classify_structural_diff(&input).unwrap();
    let second = classify_structural_diff(&input).unwrap();
    assert_eq!(
        first.to_json_bytes().unwrap(),
        second.to_json_bytes().unwrap()
    );
}

#[test]
fn max_length_rules_cover_tighten_relax_add_and_remove() {
    let classified = classify_structural_diff(&report(vec![
        element_field(
            "Observation.a",
            "maxLength",
            Some(json!(20)),
            Some(json!(10)),
        ),
        element_field(
            "Observation.b",
            "maxLength",
            Some(json!(10)),
            Some(json!(20)),
        ),
        element_field("Observation.c", "maxLength", None, Some(json!(10))),
        element_field("Observation.d", "maxLength", Some(json!(10)), None),
    ]))
    .unwrap();
    let ids = classified
        .findings
        .iter()
        .map(|finding| finding.rule_id.as_str())
        .collect::<Vec<_>>();
    for expected in [
        "CF04-LENGTH-001",
        "CF04-LENGTH-002",
        "CF04-LENGTH-003",
        "CF04-LENGTH-004",
    ] {
        assert!(ids.contains(&expected), "missing {expected}");
    }
}

#[test]
fn view_and_element_rules_are_explicit() {
    let classified = classify_structural_diff(&report(vec![
        view_change(StructuralChangeKind::ViewAdded, ElementView::Differential),
        view_change(StructuralChangeKind::ViewRemoved, ElementView::Snapshot),
        element_change(
            StructuralChangeKind::ElementAdded,
            ElementView::Snapshot,
            "Observation.new",
        ),
        element_change(
            StructuralChangeKind::ElementRemoved,
            ElementView::Snapshot,
            "Observation.old",
        ),
        element_change(
            StructuralChangeKind::ElementRemoved,
            ElementView::Differential,
            "Observation.constraintOnly",
        ),
    ]))
    .unwrap();
    let ids = classified
        .findings
        .iter()
        .map(|finding| finding.rule_id.as_str())
        .collect::<Vec<_>>();
    for expected in [
        "CF04-VIEW-001",
        "CF04-VIEW-002",
        "CF04-ELEMENT-001",
        "CF04-ELEMENT-002",
        "CF04-ELEMENT-003",
    ] {
        assert!(ids.contains(&expected), "missing {expected}");
    }
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-ELEMENT-002" && finding.severity == CompatibilitySeverity::Breaking
    }));
}

#[test]
fn structure_definition_rule_family_is_covered() {
    let classified = classify_structural_diff(&report(vec![
        structure_field("kind", Some(json!("resource")), Some(json!("complex-type"))),
        structure_field("abstract", Some(json!(false)), Some(json!(true))),
        structure_field("abstract", Some(json!(true)), Some(json!(false))),
        structure_field("abstract", None, Some(json!(true))),
        structure_field(
            "baseDefinition",
            Some(json!("http://example.org/old")),
            Some(json!("http://example.org/new")),
        ),
    ]))
    .unwrap();
    let ids = classified
        .findings
        .iter()
        .map(|finding| finding.rule_id.as_str())
        .collect::<Vec<_>>();
    for expected in [
        "CF04-STRUCTURE-001",
        "CF04-STRUCTURE-002",
        "CF04-STRUCTURE-003",
        "CF04-STRUCTURE-004",
        "CF04-STRUCTURE-005",
    ] {
        assert!(ids.contains(&expected), "missing {expected}");
    }
}
