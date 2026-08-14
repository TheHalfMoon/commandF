use std::collections::BTreeSet;

use serde_json::Value;

use crate::{
    compatibility, CompatibilityError, CompatibilityFinding, CompatibilityReport,
    CompatibilitySeverity, ElementView, ResourceKey, StructuralChange, StructuralChangeKind,
    StructuralDiffReport,
};

type SnapshotFieldIdentity = (
    ResourceKey,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
);

pub fn classify_structural_diff(
    report: &StructuralDiffReport,
) -> Result<CompatibilityReport, CompatibilityError> {
    if report.schema != StructuralDiffReport::SCHEMA_V1 {
        return Err(CompatibilityError::UnsupportedDiffSchema {
            schema: report.schema,
        });
    }

    validate_classifier_input(report)?;

    let resources_with_precise_change = report
        .changes
        .iter()
        .filter(|change| change.kind != StructuralChangeKind::ResourceBytesChanged)
        .map(|change| change.resource.clone())
        .collect::<BTreeSet<_>>();
    let snapshot_identities = snapshot_field_identities(&report.changes)?;

    let mut findings = Vec::new();
    for change in &report.changes {
        if change.kind == StructuralChangeKind::ResourceBytesChanged
            && resources_with_precise_change.contains(&change.resource)
        {
            continue;
        }
        if is_indexed_differential_duplicate(change, &snapshot_identities)? {
            continue;
        }

        let single_change_report = StructuralDiffReport {
            schema: report.schema,
            package_name: report.package_name.clone(),
            before: report.before.clone(),
            after: report.after.clone(),
            changes: vec![change.clone()],
        };
        findings.extend(compatibility::classify_structural_diff(&single_change_report)?.findings);
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

fn validate_classifier_input(report: &StructuralDiffReport) -> Result<(), CompatibilityError> {
    for change in &report.changes {
        if change.kind != StructuralChangeKind::ElementFieldChanged {
            continue;
        }
        match change.field.as_deref() {
            Some("constraint") => {
                validate_unique_constraint_keys(change.before.as_ref(), "before")?;
                validate_unique_constraint_keys(change.after.as_ref(), "after")?;
            }
            Some("binding") => {
                validate_binding_strength(change.before.as_ref(), "before")?;
                validate_binding_strength(change.after.as_ref(), "after")?;
            }
            Some("slicing") => {
                validate_slicing_rules(change.before.as_ref(), "before")?;
                validate_slicing_rules(change.after.as_ref(), "after")?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_unique_constraint_keys(
    value: Option<&Value>,
    side: &str,
) -> Result<(), CompatibilityError> {
    let Some(value) = value else {
        return Ok(());
    };
    let Some(constraints) = value.as_array() else {
        return Ok(());
    };

    let mut keys = BTreeSet::new();
    for constraint in constraints {
        let Some(key) = constraint
            .as_object()
            .and_then(|object| object.get("key"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        if !keys.insert(key.to_owned()) {
            return Err(CompatibilityError::InvalidChangeValue {
                field: "constraint".to_owned(),
                message: format!("duplicate constraint key {key:?} in {side} evidence"),
            });
        }
    }
    Ok(())
}

fn validate_binding_strength(value: Option<&Value>, side: &str) -> Result<(), CompatibilityError> {
    let Some(value) = value else {
        return Ok(());
    };
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    let Some(strength) = object.get("strength") else {
        return Ok(());
    };
    let Some(strength) = strength.as_str() else {
        return Err(CompatibilityError::InvalidChangeValue {
            field: "binding".to_owned(),
            message: format!("{side} binding.strength must be a string"),
        });
    };
    if !matches!(
        strength,
        "example" | "preferred" | "extensible" | "required"
    ) {
        return Err(CompatibilityError::InvalidChangeValue {
            field: "binding".to_owned(),
            message: format!("unrecognized {side} binding.strength {strength:?}"),
        });
    }
    Ok(())
}

fn validate_slicing_rules(value: Option<&Value>, side: &str) -> Result<(), CompatibilityError> {
    let Some(value) = value else {
        return Ok(());
    };
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    let Some(rules) = object.get("rules") else {
        return Ok(());
    };
    let Some(rules) = rules.as_str() else {
        return Err(CompatibilityError::InvalidChangeValue {
            field: "slicing".to_owned(),
            message: format!("{side} slicing.rules must be a string"),
        });
    };
    if !matches!(rules, "open" | "openAtEnd" | "closed") {
        return Err(CompatibilityError::InvalidChangeValue {
            field: "slicing".to_owned(),
            message: format!("unrecognized {side} slicing.rules {rules:?}"),
        });
    }
    Ok(())
}

fn snapshot_field_identities(
    changes: &[StructuralChange],
) -> Result<BTreeSet<SnapshotFieldIdentity>, CompatibilityError> {
    changes
        .iter()
        .filter(|change| {
            change.view == Some(ElementView::Snapshot)
                && change.kind == StructuralChangeKind::ElementFieldChanged
        })
        .map(change_identity)
        .collect()
}

fn is_indexed_differential_duplicate(
    change: &StructuralChange,
    snapshot_identities: &BTreeSet<SnapshotFieldIdentity>,
) -> Result<bool, CompatibilityError> {
    if change.view != Some(ElementView::Differential)
        || change.kind != StructuralChangeKind::ElementFieldChanged
    {
        return Ok(false);
    }
    Ok(snapshot_identities.contains(&change_identity(change)?))
}

fn change_identity(change: &StructuralChange) -> Result<SnapshotFieldIdentity, CompatibilityError> {
    Ok((
        change.resource.clone(),
        change.element_id.clone(),
        change.field.clone(),
        json_identity(change.before.as_ref())?,
        json_identity(change.after.as_ref())?,
    ))
}

fn json_identity(value: Option<&Value>) -> Result<Option<String>, CompatibilityError> {
    value
        .map(|value| {
            serde_json::to_string(value).map_err(|error| CompatibilityError::InvalidChangeValue {
                field: "<dedupe>".to_owned(),
                message: error.to_string(),
            })
        })
        .transpose()
}

fn sort_findings(findings: &mut [CompatibilityFinding]) {
    findings.sort_by(|left, right| {
        left.resource
            .cmp(&right.resource)
            .then_with(|| view_rank(left.view).cmp(&view_rank(right.view)))
            .then_with(|| left.element_id.cmp(&right.element_id))
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.direction.cmp(&right.direction))
            .then_with(|| severity_rank(left.severity).cmp(&severity_rank(right.severity)))
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

fn severity_rank(severity: CompatibilitySeverity) -> u8 {
    match severity {
        CompatibilitySeverity::Breaking => 0,
        CompatibilitySeverity::Risky => 1,
        CompatibilitySeverity::Additive => 2,
    }
}
