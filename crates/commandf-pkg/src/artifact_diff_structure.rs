use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::{
    artifact_diff_change::{base_change, element_change, view_change},
    artifact_diff_error::StructuralDiffError,
    artifact_diff_model::{ResourceKey, StructuralChange, StructuralChangeKind},
    artifact_diff_normalize::{
        normalize_structural_field, validate_element_structural_field,
        validate_resource_structural_field,
    },
    ElementView, ResourceArtifact,
};

type ElementMap = BTreeMap<String, Map<String, Value>>;

#[derive(Clone, Debug)]
pub(crate) struct MatchedElementBinding {
    pub view: ElementView,
    pub element_id: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

pub(crate) fn matched_element_bindings(
    before_resource: &ResourceArtifact,
    after_resource: &ResourceArtifact,
    before: &Value,
    after: &Value,
) -> Result<Vec<MatchedElementBinding>, StructuralDiffError> {
    let before = structural_object(before, &before_resource.filename, "resource")?;
    let after = structural_object(after, &after_resource.filename, "resource")?;
    let mut output = Vec::new();

    for (view_name_value, view) in [
        ("snapshot", ElementView::Snapshot),
        ("differential", ElementView::Differential),
    ] {
        let before_view =
            parse_view(before, view_name_value, &before_resource.filename)?.unwrap_or_default();
        let after_view =
            parse_view(after, view_name_value, &after_resource.filename)?.unwrap_or_default();
        for element_id in before_view.keys().filter(|id| after_view.contains_key(*id)) {
            let before_element = &before_view[element_id];
            let after_element = &after_view[element_id];
            let before_binding = normalized_optional_element_field(
                before_element,
                "binding",
                &before_resource.filename,
                view,
                element_id,
            )?;
            let after_binding = normalized_optional_element_field(
                after_element,
                "binding",
                &after_resource.filename,
                view,
                element_id,
            )?;
            if before_binding.is_some() || after_binding.is_some() {
                output.push(MatchedElementBinding {
                    view,
                    element_id: element_id.clone(),
                    before: before_binding,
                    after: after_binding,
                });
            }
        }
    }

    output.sort_by(|left, right| {
        view_rank(left.view)
            .cmp(&view_rank(right.view))
            .then_with(|| left.element_id.cmp(&right.element_id))
    });
    Ok(output)
}

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
        let before_value =
            normalized_optional_resource_field(before, field, &before_resource.filename)?;
        let after_value =
            normalized_optional_resource_field(after, field, &after_resource.filename)?;
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

    for (view_name_value, view) in [
        ("snapshot", ElementView::Snapshot),
        ("differential", ElementView::Differential),
    ] {
        let before_view = parse_view(before, view_name_value, &before_resource.filename)?;
        let after_view = parse_view(after, view_name_value, &after_resource.filename)?;
        compare_view(
            key,
            before_resource,
            after_resource,
            view,
            before_view,
            after_view,
            changes,
        )?;
    }

    Ok(())
}

fn parse_view(
    resource: &Map<String, Value>,
    view: &str,
    file: &str,
) -> Result<Option<ElementMap>, StructuralDiffError> {
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
    before: Option<ElementMap>,
    after: Option<ElementMap>,
    changes: &mut Vec<StructuralChange>,
) -> Result<(), StructuralDiffError> {
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
            )?,
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
    Ok(())
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
) -> Result<(), StructuralDiffError> {
    let fields = before
        .keys()
        .chain(after.keys())
        .filter(|field| is_structural_element_field(field))
        .cloned()
        .collect::<BTreeSet<_>>();

    for field in fields {
        let before_value = normalized_optional_element_field(
            before,
            &field,
            &before_resource.filename,
            view,
            element_id,
        )?;
        let after_value = normalized_optional_element_field(
            after,
            &field,
            &after_resource.filename,
            view,
            element_id,
        )?;
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
    Ok(())
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

fn normalized_optional_resource_field(
    object: &Map<String, Value>,
    field: &str,
    file: &str,
) -> Result<Option<Value>, StructuralDiffError> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };
    validate_resource_structural_field(object, field, value).map_err(|message| {
        StructuralDiffError::InvalidStructuralField {
            file: file.to_owned(),
            field: field.to_owned(),
            message,
        }
    })?;
    Ok(Some(normalize_structural_field(field, value)))
}

fn normalized_optional_element_field(
    object: &Map<String, Value>,
    field: &str,
    file: &str,
    view: ElementView,
    element_id: &str,
) -> Result<Option<Value>, StructuralDiffError> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };
    validate_element_structural_field(object, field, value).map_err(|message| {
        StructuralDiffError::InvalidStructuralField {
            file: file.to_owned(),
            field: format!("{}.element[{element_id}].{field}", view_name(view)),
            message,
        }
    })?;
    Ok(Some(normalize_structural_field(field, value)))
}

fn view_name(view: ElementView) -> &'static str {
    match view {
        ElementView::Snapshot => "snapshot",
        ElementView::Differential => "differential",
    }
}

fn view_rank(view: ElementView) -> u8 {
    match view {
        ElementView::Snapshot => 0,
        ElementView::Differential => 1,
    }
}
