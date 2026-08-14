use serde_json::Value;

use crate::{
    artifact_diff_model::{ResourceKey, StructuralChange, StructuralChangeKind},
    ElementView, ResourceArtifact,
};

pub(crate) fn base_change(
    kind: StructuralChangeKind,
    resource: &ResourceKey,
    before_filename: Option<&str>,
    after_filename: Option<&str>,
) -> StructuralChange {
    StructuralChange {
        kind,
        resource: resource.clone(),
        before_filename: before_filename.map(str::to_owned),
        after_filename: after_filename.map(str::to_owned),
        view: None,
        element_id: None,
        field: None,
        before: None,
        after: None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_scalar_change(
    changes: &mut Vec<StructuralChange>,
    kind: StructuralChangeKind,
    key: &ResourceKey,
    before_resource: &ResourceArtifact,
    after_resource: &ResourceArtifact,
    field: &str,
    before: Option<Value>,
    after: Option<Value>,
    changed: bool,
) {
    if !changed {
        return;
    }
    let mut change = base_change(
        kind,
        key,
        Some(&before_resource.filename),
        Some(&after_resource.filename),
    );
    change.field = Some(field.to_owned());
    change.before = before;
    change.after = after;
    changes.push(change);
}

pub(crate) fn view_change(
    kind: StructuralChangeKind,
    resource: &ResourceKey,
    before: &ResourceArtifact,
    after: &ResourceArtifact,
    view: ElementView,
) -> StructuralChange {
    let mut change = base_change(kind, resource, Some(&before.filename), Some(&after.filename));
    change.view = Some(view);
    change
}

pub(crate) fn element_change(
    kind: StructuralChangeKind,
    resource: &ResourceKey,
    before: &ResourceArtifact,
    after: &ResourceArtifact,
    view: ElementView,
    element_id: &str,
) -> StructuralChange {
    let mut change = view_change(kind, resource, before, after, view);
    change.element_id = Some(element_id.to_owned());
    change
}

pub(crate) fn sort_changes(changes: &mut [StructuralChange]) {
    changes.sort_by(|left, right| {
        left.resource
            .cmp(&right.resource)
            .then_with(|| view_rank(left.view).cmp(&view_rank(right.view)))
            .then_with(|| left.element_id.cmp(&right.element_id))
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.before_filename.cmp(&right.before_filename))
            .then_with(|| left.after_filename.cmp(&right.after_filename))
    });
}

fn view_rank(view: Option<ElementView>) -> u8 {
    match view {
        None => 0,
        Some(ElementView::Snapshot) => 1,
        Some(ElementView::Differential) => 2,
    }
}
