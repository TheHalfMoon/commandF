use std::collections::BTreeSet;

use serde_json::Value;

use crate::{
    compatibility, CompatibilityError, CompatibilityReport, StructuralChangeKind,
    StructuralDiffReport,
};

pub fn classify_structural_diff(
    report: &StructuralDiffReport,
) -> Result<CompatibilityReport, CompatibilityError> {
    validate_classifier_input(report)?;
    compatibility::classify_structural_diff(report)
}

fn validate_classifier_input(report: &StructuralDiffReport) -> Result<(), CompatibilityError> {
    for change in &report.changes {
        if change.kind != StructuralChangeKind::ElementFieldChanged
            || change.field.as_deref() != Some("constraint")
        {
            continue;
        }
        validate_unique_constraint_keys(change.before.as_ref(), "before")?;
        validate_unique_constraint_keys(change.after.as_ref(), "after")?;
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
