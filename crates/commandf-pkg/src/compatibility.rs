use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::{
    CompatibilityDirection, CompatibilityError, CompatibilityFinding, CompatibilityReport,
    CompatibilitySeverity, ElementView, ResourceKey, StructuralChange, StructuralChangeKind,
    StructuralDiffReport,
};

pub fn classify_structural_diff(
    report: &StructuralDiffReport,
) -> Result<CompatibilityReport, CompatibilityError> {
    if report.schema != StructuralDiffReport::SCHEMA_V1 {
        return Err(CompatibilityError::UnsupportedDiffSchema {
            schema: report.schema,
        });
    }

    let resources_with_precise_change = report
        .changes
        .iter()
        .filter(|change| change.kind != StructuralChangeKind::ResourceBytesChanged)
        .map(|change| change.resource.clone())
        .collect::<BTreeSet<_>>();

    let mut findings = Vec::new();
    for change in &report.changes {
        if change.kind == StructuralChangeKind::ResourceBytesChanged
            && resources_with_precise_change.contains(&change.resource)
        {
            continue;
        }
        if is_duplicate_differential(change, &report.changes) {
            continue;
        }
        classify_change(change, &mut findings)?;
    }
    sort_findings(&mut findings);

    Ok(CompatibilityReport {
        schema: CompatibilityReport::SCHEMA_V1,
        ruleset: CompatibilityReport::RULESET_V1.to_owned(),
        package_name: report.package_name.clone(),
        before: report.before.clone(),
        after: report.after.clone(),
        findings,
    })
}

fn classify_change(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    match change.kind {
        StructuralChangeKind::ResourceAdded => emit_both(
            findings,
            change,
            "CF04-RESOURCE-001",
            CompatibilitySeverity::Additive,
            "conformance resource added to the package contract",
        ),
        StructuralChangeKind::ResourceRemoved => emit_both(
            findings,
            change,
            "CF04-RESOURCE-002",
            CompatibilitySeverity::Breaking,
            "conformance resource removed from the package contract",
        ),
        StructuralChangeKind::ResourceFilenameChanged => emit_both(
            findings,
            change,
            "CF04-RESOURCE-003",
            CompatibilitySeverity::Risky,
            "package resource filename changed and filename-coupled tooling may be affected",
        ),
        StructuralChangeKind::ResourceVersionChanged => emit_both(
            findings,
            change,
            "CF04-RESOURCE-004",
            CompatibilitySeverity::Risky,
            "canonical resource version changed without a proven compatibility relation",
        ),
        StructuralChangeKind::ResourceTypeChanged => emit_both(
            findings,
            change,
            "CF04-RESOURCE-005",
            CompatibilitySeverity::Breaking,
            "resourceType changed for a matched package resource",
        ),
        StructuralChangeKind::ResourceIdChanged => emit_both(
            findings,
            change,
            "CF04-RESOURCE-006",
            CompatibilitySeverity::Risky,
            "resource id changed and id-coupled tooling or references may be affected",
        ),
        StructuralChangeKind::ResourceBytesChanged => emit_both(
            findings,
            change,
            "CF04-RESOURCE-007",
            CompatibilitySeverity::Risky,
            "resource bytes changed without a more precise CF-03 structural fact",
        ),
        StructuralChangeKind::StructureFieldChanged => classify_structure_field(change, findings)?,
        StructuralChangeKind::ViewAdded => emit_both(
            findings,
            change,
            "CF04-VIEW-001",
            CompatibilitySeverity::Additive,
            "StructureDefinition representation view added",
        ),
        StructuralChangeKind::ViewRemoved => emit_both(
            findings,
            change,
            "CF04-VIEW-002",
            CompatibilitySeverity::Risky,
            "StructureDefinition representation view removed and tooling may rely on it",
        ),
        StructuralChangeKind::ElementAdded => emit_both(
            findings,
            change,
            "CF04-ELEMENT-001",
            CompatibilitySeverity::Risky,
            "element added but CF-03 add evidence does not carry enough cardinality/modifier context to prove safe compatibility",
        ),
        StructuralChangeKind::ElementRemoved => {
            if change.view == Some(ElementView::Snapshot) {
                emit(
                    findings,
                    change,
                    "CF04-ELEMENT-002",
                    CompatibilitySeverity::Breaking,
                    CompatibilityDirection::Producer,
                    "snapshot element removed; before-valid producers may emit data no longer allowed by the after contract",
                );
            } else {
                emit_both(
                    findings,
                    change,
                    "CF04-ELEMENT-003",
                    CompatibilitySeverity::Risky,
                    "differential element removed and the effective compatibility relation requires base-aware interpretation",
                );
            }
        }
        StructuralChangeKind::ElementFieldChanged => classify_element_field(change, findings)?,
    }
    Ok(())
}

fn classify_structure_field(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    let field = required_field(change)?;
    match field {
        "kind" | "type" | "fhirVersion" => emit_both(
            findings,
            change,
            "CF04-STRUCTURE-001",
            CompatibilitySeverity::Breaking,
            "StructureDefinition identity/target semantics changed",
        ),
        "abstract" => match bool_pair(change, field)? {
            (Some(false), Some(true)) => emit(
                findings,
                change,
                "CF04-STRUCTURE-002",
                CompatibilitySeverity::Breaking,
                CompatibilityDirection::Producer,
                "StructureDefinition became abstract and can no longer describe directly produced instances",
            ),
            (Some(true), Some(false)) => emit_both(
                findings,
                change,
                "CF04-STRUCTURE-003",
                CompatibilitySeverity::Additive,
                "StructureDefinition became non-abstract",
            ),
            _ => emit_both(
                findings,
                change,
                "CF04-STRUCTURE-004",
                CompatibilitySeverity::Risky,
                "abstract-state change could not be reduced to a proven compatibility subset",
            ),
        },
        "baseDefinition" | "derivation" | "context" | "contextInvariant" => emit_both(
            findings,
            change,
            "CF04-STRUCTURE-005",
            CompatibilitySeverity::Risky,
            "StructureDefinition ancestry/context semantics changed without a locally provable subset relation",
        ),
        _ => {
            return Err(CompatibilityError::UnsupportedStructuralField {
                field: field.to_owned(),
            });
        }
    }
    Ok(())
}

fn classify_element_field(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    let field = required_field(change)?;
    match field {
        "min" => classify_min(change, findings)?,
        "max" => classify_max(change, findings)?,
        "maxLength" => classify_max_length(change, findings)?,
        "type" => classify_type(change, findings)?,
        "binding" => classify_binding(change, findings),
        "constraint" => classify_constraints(change, findings)?,
        "mustSupport" => {
            bool_pair(change, field)?;
            emit_both(
                findings,
                change,
                "CF04-SUPPORT-001",
                CompatibilitySeverity::Risky,
                "Must Support changed; FHIR defines support obligations as context-dependent and distinct from cardinality",
            );
        }
        "isModifier" => classify_modifier(change, findings)?,
        "slicing" => classify_slicing(change, findings)?,
        "path" | "contentReference" => emit_both(
            findings,
            change,
            "CF04-ELEMENT-004",
            CompatibilitySeverity::Breaking,
            "element structural target/path changed",
        ),
        "sliceName"
        | "sliceIsConstraining"
        | "representation"
        | "meaningWhenMissing"
        | "orderMeaning"
        | "condition"
        | "isModifierReason"
        | "isSummary"
        | "extension" => emit_both(
            findings,
            change,
            "CF04-ELEMENT-005",
            CompatibilitySeverity::Risky,
            "element structural semantics changed without a provable directional subset relation in CF-04",
        ),
        _ if field.starts_with("fixed") => classify_fixed(change, findings),
        _ if field.starts_with("pattern") => classify_pattern(change, findings),
        _ if field.starts_with("minValue") || field.starts_with("maxValue") => {
            classify_value_bound(change, findings);
        }
        _ if field.starts_with("defaultValue") => emit_both(
            findings,
            change,
            "CF04-DEFAULT-001",
            CompatibilitySeverity::Risky,
            "default value semantics changed",
        ),
        _ => {
            return Err(CompatibilityError::UnsupportedStructuralField {
                field: field.to_owned(),
            });
        }
    }
    Ok(())
}

fn classify_min(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    let field = required_field(change)?;
    let (Some(before), Some(after)) = (
        optional_u64(&change.before, field)?,
        optional_u64(&change.after, field)?,
    ) else {
        emit_both(
            findings,
            change,
            "CF04-CARD-005",
            CompatibilitySeverity::Risky,
            "minimum cardinality changed with an implicit side that requires base-aware interpretation",
        );
        return Ok(());
    };

    match after.cmp(&before) {
        Ordering::Greater => emit(
            findings,
            change,
            "CF04-CARD-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            "minimum cardinality increased; before-valid producers may omit data now required",
        ),
        Ordering::Less => emit(
            findings,
            change,
            "CF04-CARD-002",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Consumer,
            "minimum cardinality decreased; after-valid producers may omit data before-consumers could rely on",
        ),
        Ordering::Equal => {}
    }
    Ok(())
}

fn classify_max(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    let field = required_field(change)?;
    let (Some(before), Some(after)) = (
        optional_max(&change.before, field)?,
        optional_max(&change.after, field)?,
    ) else {
        emit_both(
            findings,
            change,
            "CF04-CARD-006",
            CompatibilitySeverity::Risky,
            "maximum cardinality changed with an implicit side that requires base-aware interpretation",
        );
        return Ok(());
    };

    match compare_max(after, before) {
        Ordering::Less => emit(
            findings,
            change,
            "CF04-CARD-003",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            "maximum cardinality decreased; before-valid producers may emit too many repetitions",
        ),
        Ordering::Greater => emit(
            findings,
            change,
            "CF04-CARD-004",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Consumer,
            "maximum cardinality increased; after-valid producers may emit more repetitions than before allowed",
        ),
        Ordering::Equal => {}
    }
    Ok(())
}

fn classify_max_length(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    let field = required_field(change)?;
    match (
        optional_u64(&change.before, field)?,
        optional_u64(&change.after, field)?,
    ) {
        (Some(before), Some(after)) if after < before => emit(
            findings,
            change,
            "CF04-LENGTH-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            "maximum length decreased",
        ),
        (Some(before), Some(after)) if after > before => emit(
            findings,
            change,
            "CF04-LENGTH-002",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Consumer,
            "maximum length increased and after-valid values may exceed before-consumer assumptions",
        ),
        (None, Some(_)) => emit(
            findings,
            change,
            "CF04-LENGTH-003",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            "maximum length constraint added",
        ),
        (Some(_), None) => emit(
            findings,
            change,
            "CF04-LENGTH-004",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Consumer,
            "maximum length constraint removed and longer after-valid values may appear",
        ),
        _ => {}
    }
    Ok(())
}

fn classify_type(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    let field = required_field(change)?;
    let (Some(before), Some(after)) = (&change.before, &change.after) else {
        emit_both(
            findings,
            change,
            "CF04-TYPE-004",
            CompatibilitySeverity::Risky,
            "type constraint appeared or disappeared in a sparse view; effective base-aware relation is not proven",
        );
        return Ok(());
    };

    let before_canonical = canonical_set(before, field)?;
    let after_canonical = canonical_set(after, field)?;
    if before_canonical == after_canonical {
        return Ok(());
    }

    let (Some(before_codes), Some(after_codes)) = (type_codes(before), type_codes(after)) else {
        emit_both(
            findings,
            change,
            "CF04-TYPE-005",
            CompatibilitySeverity::Risky,
            "type/profile qualifiers changed but direct type-code comparison is unavailable",
        );
        return Ok(());
    };

    if after_codes.is_subset(&before_codes) && after_codes != before_codes {
        emit(
            findings,
            change,
            "CF04-TYPE-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            "allowed type codes narrowed",
        );
    } else if before_codes.is_subset(&after_codes) && after_codes != before_codes {
        emit(
            findings,
            change,
            "CF04-TYPE-002",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Consumer,
            "allowed type codes widened",
        );
    } else if before_codes != after_codes {
        emit_both(
            findings,
            change,
            "CF04-TYPE-003",
            CompatibilitySeverity::Breaking,
            "allowed type codes were replaced with an incomparable set",
        );
    } else {
        emit_both(
            findings,
            change,
            "CF04-TYPE-005",
            CompatibilitySeverity::Risky,
            "type codes are unchanged but profile/targetProfile/aggregation qualifiers changed",
        );
    }
    Ok(())
}

fn classify_fixed(change: &StructuralChange, findings: &mut Vec<CompatibilityFinding>) {
    match (&change.before, &change.after) {
        (None, Some(_)) => emit(
            findings,
            change,
            "CF04-FIXED-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            "fixed value constraint added",
        ),
        (Some(_), None) => emit(
            findings,
            change,
            "CF04-FIXED-002",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Consumer,
            "fixed value constraint removed",
        ),
        (Some(before), Some(after)) if before != after => emit_both(
            findings,
            change,
            "CF04-FIXED-003",
            CompatibilitySeverity::Breaking,
            "fixed value changed",
        ),
        _ => {}
    }
}

fn classify_pattern(change: &StructuralChange, findings: &mut Vec<CompatibilityFinding>) {
    match (&change.before, &change.after) {
        (None, Some(_)) => emit(
            findings,
            change,
            "CF04-PATTERN-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            "pattern constraint added",
        ),
        (Some(_), None) => emit(
            findings,
            change,
            "CF04-PATTERN-002",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Consumer,
            "pattern constraint removed",
        ),
        (Some(before), Some(after)) if before != after => emit_both(
            findings,
            change,
            "CF04-PATTERN-003",
            CompatibilitySeverity::Risky,
            "pattern changed and generic pattern implication cannot be proven in CF-04",
        ),
        _ => {}
    }
}

fn classify_value_bound(change: &StructuralChange, findings: &mut Vec<CompatibilityFinding>) {
    match (&change.before, &change.after) {
        (None, Some(_)) => emit(
            findings,
            change,
            "CF04-BOUND-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            "value bound added",
        ),
        (Some(_), None) => emit(
            findings,
            change,
            "CF04-BOUND-002",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Consumer,
            "value bound removed",
        ),
        (Some(before), Some(after)) if before != after => emit_both(
            findings,
            change,
            "CF04-BOUND-003",
            CompatibilitySeverity::Risky,
            "value bound changed and generic FHIR value ordering is not proven by CF-04",
        ),
        _ => {}
    }
}

fn classify_binding(change: &StructuralChange, findings: &mut Vec<CompatibilityFinding>) {
    match (&change.before, &change.after) {
        (None, Some(after)) => {
            let severity = if binding_strength(after).is_some_and(|rank| rank >= 2) {
                CompatibilitySeverity::Breaking
            } else {
                CompatibilitySeverity::Risky
            };
            emit(
                findings,
                change,
                "CF04-BIND-001",
                severity,
                CompatibilityDirection::Producer,
                "terminology binding added",
            );
        }
        (Some(before), None) => {
            let severity = if binding_strength(before).is_some_and(|rank| rank >= 2) {
                CompatibilitySeverity::Breaking
            } else {
                CompatibilitySeverity::Risky
            };
            emit(
                findings,
                change,
                "CF04-BIND-002",
                severity,
                CompatibilityDirection::Consumer,
                "terminology binding removed",
            );
        }
        (Some(before), Some(after)) => {
            let before_strength = binding_strength(before);
            let after_strength = binding_strength(after);
            let mut specific = false;
            match (before_strength, after_strength) {
                (Some(before_rank), Some(after_rank)) if after_rank > before_rank => {
                    emit(
                        findings,
                        change,
                        "CF04-BIND-003",
                        CompatibilitySeverity::Breaking,
                        CompatibilityDirection::Producer,
                        "terminology binding strength increased",
                    );
                    specific = true;
                }
                (Some(before_rank), Some(after_rank)) if after_rank < before_rank => {
                    emit(
                        findings,
                        change,
                        "CF04-BIND-004",
                        CompatibilitySeverity::Breaking,
                        CompatibilityDirection::Consumer,
                        "terminology binding strength decreased",
                    );
                    specific = true;
                }
                _ => {}
            }
            if binding_value_set(before) != binding_value_set(after) {
                emit_both(
                    findings,
                    change,
                    "CF04-BIND-005",
                    CompatibilitySeverity::Risky,
                    "bound ValueSet changed; membership subset/superset proof is deferred to CF-07",
                );
                specific = true;
            }
            if !specific && before != after {
                emit_both(
                    findings,
                    change,
                    "CF04-BIND-006",
                    CompatibilitySeverity::Risky,
                    "binding metadata changed without a proven compatibility relation",
                );
            }
        }
        (None, None) => {}
    }
}

fn classify_constraints(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    let Some(before) = constraint_map(change.before.as_ref())? else {
        emit_both(
            findings,
            change,
            "CF04-CONSTRAINT-006",
            CompatibilitySeverity::Risky,
            "constraint keys are not directly comparable; implication cannot be proven",
        );
        return Ok(());
    };
    let Some(after) = constraint_map(change.after.as_ref())? else {
        emit_both(
            findings,
            change,
            "CF04-CONSTRAINT-006",
            CompatibilitySeverity::Risky,
            "constraint keys are not directly comparable; implication cannot be proven",
        );
        return Ok(());
    };

    let keys = before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    for key in keys {
        match (before.get(&key), after.get(&key)) {
            (None, Some(after_entry)) => emit(
                findings,
                change,
                "CF04-CONSTRAINT-001",
                if after_entry.is_error {
                    CompatibilitySeverity::Breaking
                } else {
                    CompatibilitySeverity::Risky
                },
                CompatibilityDirection::Producer,
                "constraint added to the after contract",
            ),
            (Some(before_entry), None) => emit(
                findings,
                change,
                "CF04-CONSTRAINT-002",
                if before_entry.is_error {
                    CompatibilitySeverity::Breaking
                } else {
                    CompatibilitySeverity::Risky
                },
                CompatibilityDirection::Consumer,
                "constraint removed from the before contract",
            ),
            (Some(before_entry), Some(after_entry))
                if before_entry.canonical != after_entry.canonical =>
            {
                match (before_entry.is_error, after_entry.is_error) {
                    (false, true) => emit(
                        findings,
                        change,
                        "CF04-CONSTRAINT-004",
                        CompatibilitySeverity::Breaking,
                        CompatibilityDirection::Producer,
                        "constraint severity strengthened from warning to error",
                    ),
                    (true, false) => emit(
                        findings,
                        change,
                        "CF04-CONSTRAINT-005",
                        CompatibilitySeverity::Breaking,
                        CompatibilityDirection::Consumer,
                        "constraint severity weakened from error to warning",
                    ),
                    _ => emit_both(
                        findings,
                        change,
                        "CF04-CONSTRAINT-003",
                        CompatibilitySeverity::Risky,
                        "constraint with the same key changed; FHIRPath implication/equivalence is not proven in CF-04",
                    ),
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn classify_modifier(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    match bool_pair(change, "isModifier")? {
        (Some(false) | None, Some(true)) => {
            emit(
                findings,
                change,
                "CF04-MODIFIER-001",
                CompatibilitySeverity::Breaking,
                CompatibilityDirection::Consumer,
                "element became a modifier and cannot be safely ignored by consumers",
            );
            emit(
                findings,
                change,
                "CF04-MODIFIER-002",
                CompatibilitySeverity::Risky,
                CompatibilityDirection::Producer,
                "element became a modifier and producer obligations may change",
            );
        }
        _ => emit_both(
            findings,
            change,
            "CF04-MODIFIER-003",
            CompatibilitySeverity::Risky,
            "modifier semantics changed",
        ),
    }
    Ok(())
}

fn classify_slicing(
    change: &StructuralChange,
    findings: &mut Vec<CompatibilityFinding>,
) -> Result<(), CompatibilityError> {
    let (Some(before), Some(after)) = (&change.before, &change.after) else {
        emit_both(
            findings,
            change,
            "CF04-SLICING-004",
            CompatibilitySeverity::Risky,
            "slicing definition appeared or disappeared and effective compatibility requires profile context",
        );
        return Ok(());
    };
    if !before.is_object() || !after.is_object() {
        return Err(CompatibilityError::InvalidChangeValue {
            field: "slicing".to_owned(),
            message: "expected objects".to_owned(),
        });
    }

    let mut specific = false;
    if let (Some(before_rank), Some(after_rank)) = (slicing_rank(before), slicing_rank(after)) {
        if after_rank > before_rank {
            emit(
                findings,
                change,
                "CF04-SLICING-001",
                CompatibilitySeverity::Breaking,
                CompatibilityDirection::Producer,
                "slicing became more restrictive",
            );
            specific = true;
        } else if after_rank < before_rank {
            emit(
                findings,
                change,
                "CF04-SLICING-002",
                CompatibilitySeverity::Risky,
                CompatibilityDirection::Consumer,
                "slicing became more permissive and after-valid slice structures may be new to before-consumers",
            );
            specific = true;
        }
    }
    match (slicing_ordered(before), slicing_ordered(after)) {
        (Some(false), Some(true)) => {
            emit(
                findings,
                change,
                "CF04-SLICING-003",
                CompatibilitySeverity::Breaking,
                CompatibilityDirection::Producer,
                "slicing changed from unordered to ordered",
            );
            specific = true;
        }
        (Some(true), Some(false)) => {
            emit(
                findings,
                change,
                "CF04-SLICING-005",
                CompatibilitySeverity::Risky,
                CompatibilityDirection::Consumer,
                "slicing ordering requirement was relaxed",
            );
            specific = true;
        }
        _ => {}
    }
    if slicing_residual(before) != slicing_residual(after) || !specific {
        emit_both(
            findings,
            change,
            "CF04-SLICING-006",
            CompatibilitySeverity::Risky,
            "slicing discriminator or other slicing semantics changed without a proven equivalence",
        );
    }
    Ok(())
}

fn required_field(change: &StructuralChange) -> Result<&str, CompatibilityError> {
    change
        .field
        .as_deref()
        .ok_or_else(|| CompatibilityError::InvalidChangeValue {
            field: "<missing>".to_owned(),
            message: "field-specific structural change has no field name".to_owned(),
        })
}

fn optional_u64(value: &Option<Value>, field: &str) -> Result<Option<u64>, CompatibilityError> {
    match value {
        None => Ok(None),
        Some(value) => {
            value
                .as_u64()
                .map(Some)
                .ok_or_else(|| CompatibilityError::InvalidChangeValue {
                    field: field.to_owned(),
                    message: "expected a non-negative integer".to_owned(),
                })
        }
    }
}

#[derive(Clone, Copy)]
enum MaxBound {
    Finite(u64),
    Unbounded,
}

fn optional_max(
    value: &Option<Value>,
    field: &str,
) -> Result<Option<MaxBound>, CompatibilityError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(CompatibilityError::InvalidChangeValue {
            field: field.to_owned(),
            message: "expected max as a string".to_owned(),
        });
    };
    if value == "*" {
        return Ok(Some(MaxBound::Unbounded));
    }
    let parsed = value
        .parse::<u64>()
        .map_err(|_| CompatibilityError::InvalidChangeValue {
            field: field.to_owned(),
            message: format!("invalid finite max {value}"),
        })?;
    Ok(Some(MaxBound::Finite(parsed)))
}

fn compare_max(left: MaxBound, right: MaxBound) -> Ordering {
    match (left, right) {
        (MaxBound::Unbounded, MaxBound::Unbounded) => Ordering::Equal,
        (MaxBound::Unbounded, MaxBound::Finite(_)) => Ordering::Greater,
        (MaxBound::Finite(_), MaxBound::Unbounded) => Ordering::Less,
        (MaxBound::Finite(left), MaxBound::Finite(right)) => left.cmp(&right),
    }
}

fn canonical_set(value: &Value, field: &str) -> Result<BTreeSet<String>, CompatibilityError> {
    let Some(values) = value.as_array() else {
        return Err(CompatibilityError::InvalidChangeValue {
            field: field.to_owned(),
            message: "expected an array".to_owned(),
        });
    };
    values
        .iter()
        .map(|value| {
            serde_json::to_string(value).map_err(|error| CompatibilityError::InvalidChangeValue {
                field: field.to_owned(),
                message: error.to_string(),
            })
        })
        .collect()
}

fn type_codes(value: &Value) -> Option<BTreeSet<String>> {
    let entries = value.as_array()?;
    let mut codes = BTreeSet::new();
    for entry in entries {
        let code = entry.as_object()?.get("code")?.as_str()?;
        codes.insert(code.to_owned());
    }
    Some(codes)
}

fn binding_strength(value: &Value) -> Option<u8> {
    let strength = value.as_object()?.get("strength")?.as_str()?;
    match strength {
        "example" => Some(0),
        "preferred" => Some(1),
        "extensible" => Some(2),
        "required" => Some(3),
        _ => None,
    }
}

fn binding_value_set(value: &Value) -> Option<&str> {
    value.as_object()?.get("valueSet")?.as_str()
}

#[derive(Clone, Debug)]
struct ConstraintEntry {
    canonical: String,
    is_error: bool,
}

fn constraint_map(
    value: Option<&Value>,
) -> Result<Option<BTreeMap<String, ConstraintEntry>>, CompatibilityError> {
    let Some(value) = value else {
        return Ok(Some(BTreeMap::new()));
    };
    let Some(values) = value.as_array() else {
        return Err(CompatibilityError::InvalidChangeValue {
            field: "constraint".to_owned(),
            message: "expected an array".to_owned(),
        });
    };

    let mut output = BTreeMap::new();
    for value in values {
        let Some(object) = value.as_object() else {
            return Err(CompatibilityError::InvalidChangeValue {
                field: "constraint".to_owned(),
                message: "expected constraint entries to be objects".to_owned(),
            });
        };
        let Some(key) = object.get("key").and_then(Value::as_str) else {
            return Ok(None);
        };
        let canonical = serde_json::to_string(value).map_err(|error| {
            CompatibilityError::InvalidChangeValue {
                field: "constraint".to_owned(),
                message: error.to_string(),
            }
        })?;
        let is_error = object.get("severity").and_then(Value::as_str) == Some("error");
        output.insert(
            key.to_owned(),
            ConstraintEntry {
                canonical,
                is_error,
            },
        );
    }
    Ok(Some(output))
}

fn slicing_rank(value: &Value) -> Option<u8> {
    let rules = value.as_object()?.get("rules")?.as_str()?;
    match rules {
        "open" => Some(0),
        "openAtEnd" => Some(1),
        "closed" => Some(2),
        _ => None,
    }
}

fn slicing_ordered(value: &Value) -> Option<bool> {
    value.as_object()?.get("ordered")?.as_bool()
}

fn slicing_residual(value: &Value) -> Option<Value> {
    let mut object = value.as_object()?.clone();
    object.remove("rules");
    object.remove("ordered");
    Some(Value::Object(object))
}

fn bool_pair(
    change: &StructuralChange,
    field: &str,
) -> Result<(Option<bool>, Option<bool>), CompatibilityError> {
    Ok((
        optional_bool(&change.before, field)?,
        optional_bool(&change.after, field)?,
    ))
}

fn optional_bool(value: &Option<Value>, field: &str) -> Result<Option<bool>, CompatibilityError> {
    match value {
        None => Ok(None),
        Some(value) => {
            value
                .as_bool()
                .map(Some)
                .ok_or_else(|| CompatibilityError::InvalidChangeValue {
                    field: field.to_owned(),
                    message: "expected a boolean".to_owned(),
                })
        }
    }
}

fn emit_both(
    findings: &mut Vec<CompatibilityFinding>,
    change: &StructuralChange,
    rule_id: &str,
    severity: CompatibilitySeverity,
    message: &str,
) {
    emit(
        findings,
        change,
        rule_id,
        severity,
        CompatibilityDirection::Producer,
        message,
    );
    emit(
        findings,
        change,
        rule_id,
        severity,
        CompatibilityDirection::Consumer,
        message,
    );
}

fn emit(
    findings: &mut Vec<CompatibilityFinding>,
    change: &StructuralChange,
    rule_id: &str,
    severity: CompatibilitySeverity,
    direction: CompatibilityDirection,
    message: &str,
) {
    findings.push(CompatibilityFinding {
        rule_id: rule_id.to_owned(),
        severity,
        direction,
        source_kind: change.kind,
        message: message.to_owned(),
        resource: change.resource.clone(),
        before_filename: change.before_filename.clone(),
        after_filename: change.after_filename.clone(),
        view: change.view,
        element_id: change.element_id.clone(),
        field: change.field.clone(),
        before: change.before.clone(),
        after: change.after.clone(),
    });
}

fn sort_findings(findings: &mut [CompatibilityFinding]) {
    findings.sort_by(|left, right| {
        left.resource
            .cmp(&right.resource)
            .then_with(|| view_rank(left.view).cmp(&view_rank(right.view)))
            .then_with(|| left.element_id.cmp(&right.element_id))
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.direction.cmp(&right.direction))
            .then_with(|| left.severity.cmp(&right.severity))
            .then_with(|| left.rule_id.cmp(&right.rule_id))
            .then_with(|| left.message.cmp(&right.message))
    });
}

fn view_rank(view: Option<ElementView>) -> u8 {
    match view {
        None => 0,
        Some(ElementView::Snapshot) => 1,
        Some(ElementView::Differential) => 2,
    }
}

fn is_duplicate_differential(change: &StructuralChange, changes: &[StructuralChange]) -> bool {
    if change.view != Some(ElementView::Differential)
        || change.kind != StructuralChangeKind::ElementFieldChanged
    {
        return false;
    }
    changes.iter().any(|candidate| {
        candidate.view == Some(ElementView::Snapshot)
            && candidate.kind == change.kind
            && candidate.resource == change.resource
            && candidate.element_id == change.element_id
            && candidate.field == change.field
            && candidate.before == change.before
            && candidate.after == change.after
    })
}
