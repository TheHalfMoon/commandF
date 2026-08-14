use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::{
    artifact_diff_change::{base_change, element_change, view_change},
    artifact_diff_error::StructuralDiffError,
    artifact_diff_model::{ResourceKey, StructuralChange, StructuralChangeKind},
    artifact_diff_normalize::normalize_structural_field,
    ElementView, ResourceArtifact,
};

pub(crate) fn compare_structure_definition(
    key: &ResourceKey,
    before_resource: &ResourceArtifact,
    after_resource: &ResourceArtifact,
    before: &Value,
    after: &Value,
    changes: &mut Vec<StructuralChange>,
) -> Result<(), StructuralDiffError> {
    let before = structural_object(before, &before_resource.filename, "resource")?;
    let after = structural_object(after, &after_resource.filename, "resource")?;

    for field in [
        "kind",
        "abstract",
        "type",
        "baseDefinition",
        "derivation",
        "fhirVersion",
        "context",
        "contextInvariant",
    ] {
        let before_value = normalized_optional_field(before, field);
        let after_value = normalized_optional_field(after, field);
        if before_value != after_value {
            let mut change = base_change(
                StructuralChangeKind::StructureFieldChanged,
                key,
                Some(&before_resource.filename),
                Some(&after_resource.filename),
            );
            change.field = Some(field.to_owned());
            change.before = before_value;
            change.after = after_value;
            changes.push(change);
        }
    }

    for (view_name, view) in [
        ("snapshot", ElementView::Snapshot),
        ("differential", ElementView::Differential),
    ] {
        let before_view = parse_view(before, view_name, &before_resource.filename)?;
        let after_view = parse_view(after, view_name, &after_resource.filename)?;
        compare_view(
            key,
            before_resource,
            after_resource,
            view,
            before_view,
            after_view,
            changes,
        );
    }

    Ok(())
}

fn parse_view(
    resource: &Map<String, Value>,
    view: &str,
    file: &str,
) -> Result<Option<BTreeMap<String, Map<String, Value>>>, StructuralDiffError> {
    let Some(container) = resource.get(view) else {
        return Ok(None);
    };
    let Some(elements) = container.get("element").and_then(Value::as_array) else {
        return Err(StructuralDiffError::InvalidStructuralField {
            file: file.to_owned(),
            field: format!("{view}.element"),
            message: "expected an array".to_owned(),
        });
    };

    let mut output = BTreeMap::new();
    for (index, value) in elements.iter().enumerate() {
        let Some(element) = value.as_object() else {
            return Err(StructuralDiffError::InvalidStructuralField {
                file: file.to_owned(),
                field: format!("{view}.element[{index}]"),
                message: "expected an object".to_owned(),
            });
        };
        let Some(id) = element.get("id").and_then(Value::as_str) else {
            return Err(StructuralDiffError::InvalidStructuralField {
                file: file.to_owned(),
                field: format!("{view}.element[{index}].id"),
                message: "expected a string".to_owned(),
            });
        };
        if output.insert(id.to_owned(), element.clone()).is_some() {
            return Err(StructuralDiffError::InvalidStructuralField {
                file: file.to_owned(),
                field: format!("{view}.element.id"),
                message: format!("duplicate id {id}"),
            });
        }
    }
    Ok(Some(output))
}

#[allow(clippy::too_many_arguments)]
fn compare_view(
    key: &ResourceKey,
    before_resource: &ResourceArtifact,
    after_resource: &ResourceArtifact,
    view: ElementView,
    before: Option<BTreeMap<String, Map<String, Value>>>,
    after: Option<BTreeMap<String, Map<String, Value>>>,
    changes: &mut Vec<StructuralChange>,
) {
    match (&before, &after) {
        (None, Some(_)) => changes.push(view_change(
            StructuralChangeKind::ViewAdded,
            key,
            before_resource,
            after_resource,
            view,
        )),
        (Some(_), None) => changes.push(view_change(
            StructuralChangeKind::ViewRemoved,
            key,
            before_resource,
            after_resource,
            view,
        )),
        _ => {}
    }

    let before = before.unwrap_or_default();
    let after = after.unwrap_or_default();
    let ids = before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    for id in ids {
        match (before.get(&id), after.get(&id)) {
            (Some(before_element), Some(after_element)) => compare_element(
                key,
                before_resource,
                after_resource,
                view,
                &id,
                before_element,
                after_element,
                changes,
            ),
            (Some(_), None) => changes.push(element_change(
                StructuralChangeKind::ElementRemoved,
                key,
                before_resource,
                after_resource,
                view,
                &id,
            )),
            (None, Some(_)) => changes.push(element_change(
                StructuralChangeKind::ElementAdded,
                key,
                before_resource,
                after_resource,
                view,
                &id,
            )),
            (None, None) => unreachable!("union id must exist on at least one side"),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compare_element(
    key: &ResourceKey,
    before_resource: &ResourceArtifact,
    after_resource: &ResourceArtifact,
    view: ElementView,
    element_id: &str,
    before: &Map<String, Value>,
    after: &Map<String, Value>,
    changes: &mut Vec<StructuralChange>,
) {
    let fields = before
        .keys()
        .chain(after.keys())
        .filter(|field| is_structural_element_field(field))
        .cloned()
        .collect::<BTreeSet<_>>();

    for field in fields {
        let before_value = before
            .get(&field)
            .map(|value| normalize_structural_field(&field, value));
        let after_value = after
            .get(&field)
            .map(|value| normalize_structural_field(&field, value));
        if before_value != after_value {
            let mut change = base_change(
                StructuralChangeKind::ElementFieldChanged,
                key,
                Some(&before_resource.filename),
                Some(&after_resource.filename),
            );
            change.view = Some(view);
            change.element_id = Some(element_id.to_owned());
            change.field = Some(field);
            change.before = before_value;
            change.after = after_value;
            changes.push(change);
        }
    }
}

fn is_structural_element_field(field: &str) -> bool {
    matches!(
        field,
        "path"
            | "sliceName"
            | "sliceIsConstraining"
            | "representation"
            | "slicing"
            | "min"
            | "max"
            | "contentReference"
            | "type"
            | "meaningWhenMissing"
            | "orderMeaning"
            | "maxLength"
            | "condition"
            | "constraint"
            | "mustSupport"
            | "isModifier"
            | "isModifierReason"
            | "isSummary"
            | "binding"
            | "extension"
    ) || ["defaultValue", "fixed", "pattern", "minValue", "maxValue"]
        .iter()
        .any(|prefix| field.starts_with(prefix))
}

fn structural_object<'a>(
    value: &'a Value,
    file: &str,
    field: &str,
) -> Result<&'a Map<String, Value>, StructuralDiffError> {
    value
        .as_object()
        .ok_or_else(|| StructuralDiffError::InvalidStructuralField {
            file: file.to_owned(),
            field: field.to_owned(),
            message: "expected an object".to_owned(),
        })
}

fn normalized_optional_field(object: &Map<String, Value>, field: &str) -> Option<Value> {
    object
        .get(field)
        .map(|value| normalize_structural_field(field, value))
}
