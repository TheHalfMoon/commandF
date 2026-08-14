use commandf_pkg::{
    classify_structural_diff, CompatibilityDirection, CompatibilityError, CompatibilitySeverity,
    ElementView, PackageEvidence, ResourceKey, ResourceKeyKind, StructuralChange,
    StructuralChangeKind, StructuralDiffReport,
};
use serde_json::json;

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
    view: ElementView,
    element_id: &str,
    field: &str,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
) -> StructuralChange {
    StructuralChange {
        kind: StructuralChangeKind::ElementFieldChanged,
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: "http://example.org/StructureDefinition/example".to_owned(),
        },
        before_filename: Some("StructureDefinition-example.json".to_owned()),
        after_filename: Some("StructureDefinition-example.json".to_owned()),
        view: Some(view),
        element_id: Some(element_id.to_owned()),
        field: Some(field.to_owned()),
        before,
        after,
    }
}

fn resource_change(kind: StructuralChangeKind, value: &str) -> StructuralChange {
    StructuralChange {
        kind,
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: format!("http://example.org/StructureDefinition/{value}"),
        },
        before_filename: Some(format!("{value}-before.json")),
        after_filename: Some(format!("{value}-after.json")),
        view: None,
        element_id: None,
        field: None,
        before: None,
        after: None,
    }
}

#[test]
fn empty_report_is_deterministic_and_has_no_findings() {
    let classified = classify_structural_diff(&report(vec![])).unwrap();
    assert_eq!(classified.schema, 1);
    assert_eq!(classified.ruleset, "cf04-rules-v1");
    assert!(classified.findings.is_empty());
    assert_eq!(
        classified.to_json_bytes().unwrap(),
        classified.to_json_bytes().unwrap()
    );
}

#[test]
fn cardinality_changes_are_directional() {
    let classified = classify_structural_diff(&report(vec![
        element_field(
            ElementView::Snapshot,
            "Observation.status",
            "min",
            Some(json!(0)),
            Some(json!(1)),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.code",
            "min",
            Some(json!(1)),
            Some(json!(0)),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.identifier",
            "max",
            Some(json!("*")),
            Some(json!("1")),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.category",
            "max",
            Some(json!("1")),
            Some(json!("*")),
        ),
    ]))
    .unwrap();

    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-CARD-001"
            && finding.severity == CompatibilitySeverity::Breaking
            && finding.direction == CompatibilityDirection::Producer
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-CARD-002" && finding.direction == CompatibilityDirection::Consumer
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-CARD-003" && finding.direction == CompatibilityDirection::Producer
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-CARD-004" && finding.direction == CompatibilityDirection::Consumer
    }));
}

#[test]
fn type_code_set_narrowing_and_widening_reverse_direction() {
    let string_type = json!({"code": "string"});
    let code_type = json!({"code": "code"});
    let classified = classify_structural_diff(&report(vec![
        element_field(
            ElementView::Snapshot,
            "Observation.value[x]",
            "type",
            Some(json!([string_type.clone(), code_type.clone()])),
            Some(json!([string_type.clone()])),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.component.value[x]",
            "type",
            Some(json!([string_type.clone()])),
            Some(json!([string_type, code_type])),
        ),
    ]))
    .unwrap();

    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-TYPE-001" && finding.direction == CompatibilityDirection::Producer
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-TYPE-002" && finding.direction == CompatibilityDirection::Consumer
    }));
}

#[test]
fn type_profile_qualifier_change_is_risky_not_false_breaking() {
    let classified = classify_structural_diff(&report(vec![element_field(
        ElementView::Snapshot,
        "Observation.subject",
        "type",
        Some(json!([{
            "code": "Reference",
            "targetProfile": ["http://example.org/StructureDefinition/old"]
        }])),
        Some(json!([{
            "code": "Reference",
            "targetProfile": ["http://example.org/StructureDefinition/new"]
        }])),
    )]))
    .unwrap();

    assert_eq!(classified.findings.len(), 2);
    assert!(classified.findings.iter().all(|finding| {
        finding.rule_id == "CF04-TYPE-005" && finding.severity == CompatibilitySeverity::Risky
    }));
}

#[test]
fn fixed_pattern_and_bounds_do_not_overclaim_equivalence() {
    let classified = classify_structural_diff(&report(vec![
        element_field(
            ElementView::Snapshot,
            "Observation.status",
            "fixedCode",
            None,
            Some(json!("final")),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.code",
            "patternCodeableConcept",
            Some(json!({"coding": [{"code": "a"}]})),
            Some(json!({"coding": [{"code": "b"}]})),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.valueInteger",
            "minValueInteger",
            Some(json!(1)),
            Some(json!(2)),
        ),
    ]))
    .unwrap();

    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-FIXED-001"
            && finding.severity == CompatibilitySeverity::Breaking
            && finding.direction == CompatibilityDirection::Producer
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-PATTERN-003" && finding.severity == CompatibilitySeverity::Risky
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-BOUND-003" && finding.severity == CompatibilitySeverity::Risky
    }));
}

#[test]
fn binding_strength_is_directional_but_value_set_change_is_risky() {
    let classified = classify_structural_diff(&report(vec![
        element_field(
            ElementView::Snapshot,
            "Observation.status",
            "binding",
            Some(json!({
                "strength": "preferred",
                "valueSet": "http://example.org/ValueSet/old"
            })),
            Some(json!({
                "strength": "required",
                "valueSet": "http://example.org/ValueSet/new"
            })),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.category",
            "binding",
            Some(json!({"strength": "required"})),
            Some(json!({"strength": "extensible"})),
        ),
    ]))
    .unwrap();

    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-BIND-003"
            && finding.direction == CompatibilityDirection::Producer
            && finding.severity == CompatibilitySeverity::Breaking
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-BIND-004"
            && finding.direction == CompatibilityDirection::Consumer
            && finding.severity == CompatibilitySeverity::Breaking
    }));
    assert_eq!(
        classified
            .findings
            .iter()
            .filter(|finding| finding.rule_id == "CF04-BIND-005")
            .count(),
        2
    );
}

#[test]
fn constraints_must_support_modifier_and_slicing_are_explicit() {
    let classified = classify_structural_diff(&report(vec![
        element_field(
            ElementView::Snapshot,
            "Observation",
            "constraint",
            Some(json!([])),
            Some(json!([{
                "key": "obs-test",
                "severity": "error",
                "human": "required",
                "expression": "status.exists()"
            }])),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.code",
            "mustSupport",
            Some(json!(false)),
            Some(json!(true)),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.value[x]",
            "isModifier",
            Some(json!(false)),
            Some(json!(true)),
        ),
        element_field(
            ElementView::Snapshot,
            "Observation.component",
            "slicing",
            Some(json!({"rules": "open", "ordered": false})),
            Some(json!({"rules": "closed", "ordered": true})),
        ),
    ]))
    .unwrap();

    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-CONSTRAINT-001"
            && finding.direction == CompatibilityDirection::Producer
            && finding.severity == CompatibilitySeverity::Breaking
    }));
    assert_eq!(
        classified
            .findings
            .iter()
            .filter(|finding| finding.rule_id == "CF04-SUPPORT-001")
            .count(),
        2
    );
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-MODIFIER-001"
            && finding.direction == CompatibilityDirection::Consumer
            && finding.severity == CompatibilitySeverity::Breaking
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-SLICING-001"
            && finding.direction == CompatibilityDirection::Producer
    }));
    assert!(classified.findings.iter().any(|finding| {
        finding.rule_id == "CF04-SLICING-003"
            && finding.direction == CompatibilityDirection::Producer
    }));
}

#[test]
fn same_key_constraint_expression_change_is_risky_not_false_breaking() {
    let classified = classify_structural_diff(&report(vec![element_field(
        ElementView::Snapshot,
        "Observation",
        "constraint",
        Some(json!([{
            "key": "obs-test",
            "severity": "error",
            "human": "old",
            "expression": "status.exists()"
        }])),
        Some(json!([{
            "key": "obs-test",
            "severity": "error",
            "human": "new",
            "expression": "status.exists() and code.exists()"
        }])),
    )]))
    .unwrap();

    assert_eq!(classified.findings.len(), 2);
    assert!(classified.findings.iter().all(|finding| {
        finding.rule_id == "CF04-CONSTRAINT-003" && finding.severity == CompatibilitySeverity::Risky
    }));
}

#[test]
fn resource_rules_cover_add_remove_and_residual_bytes() {
    let classified = classify_structural_diff(&report(vec![
        resource_change(StructuralChangeKind::ResourceAdded, "added"),
        resource_change(StructuralChangeKind::ResourceRemoved, "removed"),
        resource_change(StructuralChangeKind::ResourceBytesChanged, "bytes"),
    ]))
    .unwrap();

    assert_eq!(
        classified
            .findings
            .iter()
            .filter(|finding| {
                finding.rule_id == "CF04-RESOURCE-001"
                    && finding.severity == CompatibilitySeverity::Additive
            })
            .count(),
        2
    );
    assert_eq!(
        classified
            .findings
            .iter()
            .filter(|finding| {
                finding.rule_id == "CF04-RESOURCE-002"
                    && finding.severity == CompatibilitySeverity::Breaking
            })
            .count(),
        2
    );
    assert_eq!(
        classified
            .findings
            .iter()
            .filter(|finding| {
                finding.rule_id == "CF04-RESOURCE-007"
                    && finding.severity == CompatibilitySeverity::Risky
            })
            .count(),
        2
    );
}

#[test]
fn byte_hash_fact_is_subsumed_when_precise_structural_fact_exists() {
    let classified = classify_structural_diff(&report(vec![
        resource_change(StructuralChangeKind::ResourceVersionChanged, "same"),
        resource_change(StructuralChangeKind::ResourceBytesChanged, "same"),
    ]))
    .unwrap();

    assert_eq!(
        classified
            .findings
            .iter()
            .filter(|finding| finding.rule_id == "CF04-RESOURCE-004")
            .count(),
        2
    );
    assert!(!classified
        .findings
        .iter()
        .any(|finding| finding.rule_id == "CF04-RESOURCE-007"));
}

#[test]
fn equivalent_differential_field_fact_is_deduplicated_in_favor_of_snapshot() {
    let snapshot = element_field(
        ElementView::Snapshot,
        "Observation.status",
        "min",
        Some(json!(0)),
        Some(json!(1)),
    );
    let differential = element_field(
        ElementView::Differential,
        "Observation.status",
        "min",
        Some(json!(0)),
        Some(json!(1)),
    );
    let classified = classify_structural_diff(&report(vec![snapshot, differential])).unwrap();

    let findings = classified
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "CF04-CARD-001")
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].view, Some(ElementView::Snapshot));
}

#[test]
fn unknown_future_structural_field_fails_closed() {
    let error = classify_structural_diff(&report(vec![element_field(
        ElementView::Snapshot,
        "Observation.status",
        "futureField",
        Some(json!(1)),
        Some(json!(2)),
    )]))
    .unwrap_err();

    assert!(matches!(
        error,
        CompatibilityError::UnsupportedStructuralField { ref field }
            if field == "futureField"
    ));
}

#[test]
fn unsupported_cf03_schema_fails_closed() {
    let mut input = report(vec![]);
    input.schema = 2;
    let error = classify_structural_diff(&input).unwrap_err();
    assert!(matches!(
        error,
        CompatibilityError::UnsupportedDiffSchema { schema: 2 }
    ));
}

#[test]
fn malformed_boolean_evidence_fails_closed() {
    let error = classify_structural_diff(&report(vec![element_field(
        ElementView::Snapshot,
        "Observation.code",
        "mustSupport",
        Some(json!(false)),
        Some(json!("yes")),
    )]))
    .unwrap_err();
    assert!(matches!(
        error,
        CompatibilityError::InvalidChangeValue { ref field, .. }
            if field == "mustSupport"
    ));
}
