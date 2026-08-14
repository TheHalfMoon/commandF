use std::collections::BTreeSet;

use serde_json::Value;

use crate::{
    artifact_diff::matched_resource_pairs,
    compare_complete_code_systems, compare_value_set_expansions,
    terminology_index::{TerminologyClosure, TerminologyResource},
    BindingRefinement, CompatibilityDirection, CompatibilityReport, CompatibilitySeverity,
    ElementView, Lockfile, PackageCache, PackageEvidence, ResourceKeyKind, StructuralDiffReport,
    TerminologyDiffReport, TerminologyError, TerminologyIndeterminateReason, TerminologyRelation,
    TerminologySetDelta,
};

pub fn build_terminology_diff_report(
    before_lock: &Lockfile,
    before_cache: &PackageCache,
    after_lock: &Lockfile,
    after_cache: &PackageCache,
    structural: &StructuralDiffReport,
    compatibility: &CompatibilityReport,
    before_root_bytes: &[u8],
    after_root_bytes: &[u8],
) -> Result<TerminologyDiffReport, TerminologyError> {
    validate_report_contract(structural, compatibility)?;
    validate_root_evidence(before_lock, &structural.package_name, &structural.before)?;
    validate_root_evidence(after_lock, &structural.package_name, &structural.after)?;

    let before_closure = TerminologyClosure::load(before_lock, before_cache)?;
    let after_closure = TerminologyClosure::load(after_lock, after_cache)?;

    let pairs = matched_resource_pairs(
        &structural.package_name,
        &structural.before.version,
        &structural.before.archive_sha256,
        before_root_bytes,
        &structural.after.version,
        &structural.after.archive_sha256,
        after_root_bytes,
    )?;

    let mut code_systems = Vec::new();
    let mut value_sets = Vec::new();
    for pair in pairs {
        if pair.before.sha256 == pair.after.sha256 || pair.key.kind != ResourceKeyKind::Canonical {
            continue;
        }
        match (
            pair.before.resource_type.as_str(),
            pair.after.resource_type.as_str(),
        ) {
            ("CodeSystem", "CodeSystem") => code_systems.push(compare_complete_code_systems(
                pair.key,
                &pair.before_value,
                &pair.after_value,
            )?),
            ("ValueSet", "ValueSet") => value_sets.push(compare_value_set_expansions(
                pair.key,
                &pair.before_value,
                &pair.after_value,
            )?),
            _ => {}
        }
    }
    sort_set_deltas(&mut code_systems);
    sort_set_deltas(&mut value_sets);

    let mut binding_refinements =
        build_binding_refinements(compatibility, &before_closure, &after_closure)?;
    sort_binding_refinements(&mut binding_refinements);

    Ok(TerminologyDiffReport {
        schema: TerminologyDiffReport::SCHEMA_V1,
        ruleset: TerminologyDiffReport::RULESET_V1.to_owned(),
        package_name: structural.package_name.clone(),
        before: structural.before.clone(),
        after: structural.after.clone(),
        compatibility: compatibility.clone(),
        code_systems,
        value_sets,
        binding_refinements,
    })
}

fn validate_report_contract(
    structural: &StructuralDiffReport,
    compatibility: &CompatibilityReport,
) -> Result<(), TerminologyError> {
    if structural.schema != StructuralDiffReport::SCHEMA_V1 {
        return Err(TerminologyError::UnsupportedDiffSchema {
            schema: structural.schema,
        });
    }
    if compatibility.schema != CompatibilityReport::SCHEMA_V1
        || compatibility.ruleset != CompatibilityReport::RULESET_V1
    {
        return Err(TerminologyError::UnsupportedCompatibility {
            schema: compatibility.schema,
            ruleset: compatibility.ruleset.clone(),
        });
    }
    if compatibility.package_name != structural.package_name
        || compatibility.before != structural.before
        || compatibility.after != structural.after
    {
        return Err(TerminologyError::InvalidField {
            resource: structural.package_name.clone(),
            field: "compatibility".to_owned(),
            message: "embedded CF-04 report identity does not match the CF-03 report".to_owned(),
        });
    }
    Ok(())
}

fn validate_root_evidence(
    lockfile: &Lockfile,
    package_name: &str,
    evidence: &PackageEvidence,
) -> Result<(), TerminologyError> {
    let mut matches = lockfile
        .packages
        .iter()
        .filter(|package| package.name == package_name);
    let Some(package) = matches.next() else {
        return Err(TerminologyError::InvalidField {
            resource: package_name.to_owned(),
            field: "lockfile".to_owned(),
            message: "root package is not present in the lockfile".to_owned(),
        });
    };
    if matches.next().is_some() {
        return Err(TerminologyError::InvalidField {
            resource: package_name.to_owned(),
            field: "lockfile".to_owned(),
            message: "root package appears more than once in the lockfile".to_owned(),
        });
    }
    if package.version != evidence.version || package.sha256 != evidence.archive_sha256 {
        return Err(TerminologyError::InvalidField {
            resource: package_name.to_owned(),
            field: "lockfile".to_owned(),
            message: "root lock identity does not match CF-03 package evidence".to_owned(),
        });
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct BindingKey {
    resource: crate::ResourceKey,
    view: u8,
    element_id: Option<String>,
    before_value_set: Option<String>,
    after_value_set: Option<String>,
}

fn build_binding_refinements(
    compatibility: &CompatibilityReport,
    before_closure: &TerminologyClosure,
    after_closure: &TerminologyClosure,
) -> Result<Vec<BindingRefinement>, TerminologyError> {
    let mut seen = BTreeSet::new();
    let mut output = Vec::new();

    for finding in compatibility
        .findings
        .iter()
        .filter(|finding| finding.rule_id == "CF04-BIND-005")
    {
        let before_value_set =
            binding_string(finding.before.as_ref(), "valueSet", &finding.resource.value)?;
        let after_value_set =
            binding_string(finding.after.as_ref(), "valueSet", &finding.resource.value)?;
        let before_strength =
            binding_string(finding.before.as_ref(), "strength", &finding.resource.value)?;
        let after_strength =
            binding_string(finding.after.as_ref(), "strength", &finding.resource.value)?;
        validate_binding_strength(before_strength.as_deref(), &finding.resource.value)?;
        validate_binding_strength(after_strength.as_deref(), &finding.resource.value)?;

        let key = BindingKey {
            resource: finding.resource.clone(),
            view: view_rank(finding.view),
            element_id: finding.element_id.clone(),
            before_value_set: before_value_set.clone(),
            after_value_set: after_value_set.clone(),
        };
        if !seen.insert(key) {
            continue;
        }

        let (Some(before_reference), Some(after_reference)) =
            (before_value_set.as_deref(), after_value_set.as_deref())
        else {
            output.push(indeterminate_refinement(
                finding,
                before_value_set,
                after_value_set,
                TerminologyIndeterminateReason::UnsupportedBindingInteraction,
            ));
            continue;
        };

        let before_resolution = resolve_binding_value_set(before_closure, before_reference)?;
        let after_resolution = resolve_binding_value_set(after_closure, after_reference)?;
        let (before_resource, after_resource) = match (before_resolution, after_resolution) {
            (BindingResolution::Found(before), BindingResolution::Found(after)) => (before, after),
            (BindingResolution::Ambiguous, _) | (_, BindingResolution::Ambiguous) => {
                output.push(indeterminate_refinement(
                    finding,
                    before_value_set,
                    after_value_set,
                    TerminologyIndeterminateReason::AmbiguousCanonical,
                ));
                continue;
            }
            _ => {
                output.push(indeterminate_refinement(
                    finding,
                    before_value_set,
                    after_value_set,
                    TerminologyIndeterminateReason::UnresolvedValueSet,
                ));
                continue;
            }
        };

        let delta =
            compare_binding_value_sets(finding.resource.clone(), before_resource, after_resource)?;
        let stable_required = before_strength.as_deref() == Some("required")
            && after_strength.as_deref() == Some("required");
        let interaction_reason = (before_strength != after_strength)
            .then_some(TerminologyIndeterminateReason::UnsupportedBindingInteraction);
        emit_binding_relation(
            &mut output,
            finding,
            before_value_set,
            after_value_set,
            &delta,
            stable_required,
            interaction_reason,
        );
    }

    Ok(output)
}

enum BindingResolution<'a> {
    Found(&'a TerminologyResource),
    Missing,
    Ambiguous,
}

fn resolve_binding_value_set<'a>(
    closure: &'a TerminologyClosure,
    reference: &str,
) -> Result<BindingResolution<'a>, TerminologyError> {
    match closure.resolve_value_set(reference) {
        Ok(Some(resource)) => Ok(BindingResolution::Found(resource)),
        Ok(None) => Ok(BindingResolution::Missing),
        Err(TerminologyError::AmbiguousCanonical { .. }) => Ok(BindingResolution::Ambiguous),
        Err(error) => Err(error),
    }
}

fn compare_binding_value_sets(
    resource: crate::ResourceKey,
    before: &TerminologyResource,
    after: &TerminologyResource,
) -> Result<TerminologySetDelta, TerminologyError> {
    let mut after_value = after.value.clone();
    let before_url =
        before
            .value
            .get("url")
            .cloned()
            .ok_or_else(|| TerminologyError::InvalidField {
                resource: before.filename.clone(),
                field: "url".to_owned(),
                message: "resolved ValueSet is missing its canonical URL".to_owned(),
            })?;
    let after_object =
        after_value
            .as_object_mut()
            .ok_or_else(|| TerminologyError::InvalidField {
                resource: after.filename.clone(),
                field: "resource".to_owned(),
                message: "resolved ValueSet must be an object".to_owned(),
            })?;
    // Root ValueSet comparison requires the same canonical as a matched-resource guard.
    // Binding replacement intentionally compares two different ValueSet canonicals by
    // membership, so only that guard field is normalized on this owned copy.
    after_object.insert("url".to_owned(), before_url);
    compare_value_set_expansions(resource, &before.value, &after_value)
}

fn emit_binding_relation(
    output: &mut Vec<BindingRefinement>,
    finding: &crate::CompatibilityFinding,
    before_value_set: Option<String>,
    after_value_set: Option<String>,
    delta: &TerminologySetDelta,
    stable_required: bool,
    interaction_reason: Option<TerminologyIndeterminateReason>,
) {
    let reason = interaction_reason.or(delta.reason);
    let base = BindingRefinement {
        resource: finding.resource.clone(),
        view: finding.view,
        element_id: finding.element_id.clone(),
        before_value_set,
        after_value_set,
        relation: delta.relation,
        proof_mode: delta.proof_mode,
        binding_proof_eligible: delta.binding_proof_eligible,
        reason,
        rule_id: None,
        severity: None,
        direction: None,
        message: None,
    };

    if !stable_required || !delta.binding_proof_eligible {
        output.push(base);
        return;
    }

    match delta.relation {
        TerminologyRelation::Narrowed => output.push(hard_refinement(
            base,
            "CF07-BIND-001",
            CompatibilityDirection::Producer,
            "required binding allowed-membership set narrowed",
        )),
        TerminologyRelation::Widened => output.push(hard_refinement(
            base,
            "CF07-BIND-002",
            CompatibilityDirection::Consumer,
            "required binding allowed-membership set widened",
        )),
        TerminologyRelation::Incomparable => {
            output.push(hard_refinement(
                base.clone(),
                "CF07-BIND-003",
                CompatibilityDirection::Producer,
                "required binding replaced with an incomparable allowed-membership set",
            ));
            output.push(hard_refinement(
                base,
                "CF07-BIND-004",
                CompatibilityDirection::Consumer,
                "required binding replaced with an incomparable allowed-membership set",
            ));
        }
        TerminologyRelation::Equal | TerminologyRelation::Indeterminate => output.push(base),
    }
}

fn hard_refinement(
    mut base: BindingRefinement,
    rule_id: &str,
    direction: CompatibilityDirection,
    message: &str,
) -> BindingRefinement {
    base.rule_id = Some(rule_id.to_owned());
    base.severity = Some(CompatibilitySeverity::Breaking);
    base.direction = Some(direction);
    base.message = Some(message.to_owned());
    base
}

fn indeterminate_refinement(
    finding: &crate::CompatibilityFinding,
    before_value_set: Option<String>,
    after_value_set: Option<String>,
    reason: TerminologyIndeterminateReason,
) -> BindingRefinement {
    BindingRefinement {
        resource: finding.resource.clone(),
        view: finding.view,
        element_id: finding.element_id.clone(),
        before_value_set,
        after_value_set,
        relation: TerminologyRelation::Indeterminate,
        proof_mode: None,
        binding_proof_eligible: false,
        reason: Some(reason),
        rule_id: None,
        severity: None,
        direction: None,
        message: None,
    }
}

fn binding_string(
    value: Option<&Value>,
    field: &str,
    resource: &str,
) -> Result<Option<String>, TerminologyError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let object = value
        .as_object()
        .ok_or_else(|| TerminologyError::InvalidField {
            resource: resource.to_owned(),
            field: "binding".to_owned(),
            message: "binding evidence must be an object".to_owned(),
        })?;
    match object.get(field) {
        None => Ok(None),
        Some(Value::String(value)) if !value.trim().is_empty() && value.trim() == value => {
            Ok(Some(value.clone()))
        }
        Some(_) => Err(TerminologyError::InvalidField {
            resource: resource.to_owned(),
            field: format!("binding.{field}"),
            message: "must be a non-empty trimmed string".to_owned(),
        }),
    }
}

fn validate_binding_strength(
    strength: Option<&str>,
    resource: &str,
) -> Result<(), TerminologyError> {
    if strength.is_none_or(|strength| {
        matches!(
            strength,
            "example" | "preferred" | "extensible" | "required"
        )
    }) {
        Ok(())
    } else {
        Err(TerminologyError::InvalidField {
            resource: resource.to_owned(),
            field: "binding.strength".to_owned(),
            message: "unrecognized binding strength".to_owned(),
        })
    }
}

fn sort_set_deltas(values: &mut [TerminologySetDelta]) {
    values.sort_by(|left, right| {
        left.resource
            .cmp(&right.resource)
            .then_with(|| left.resource_type.cmp(&right.resource_type))
            .then_with(|| left.relation.cmp(&right.relation))
    });
}

fn sort_binding_refinements(values: &mut [BindingRefinement]) {
    values.sort_by(|left, right| {
        left.resource
            .cmp(&right.resource)
            .then_with(|| view_rank(left.view).cmp(&view_rank(right.view)))
            .then_with(|| left.element_id.cmp(&right.element_id))
            .then_with(|| left.before_value_set.cmp(&right.before_value_set))
            .then_with(|| left.after_value_set.cmp(&right.after_value_set))
            .then_with(|| left.direction.cmp(&right.direction))
            .then_with(|| left.rule_id.cmp(&right.rule_id))
    });
}

fn view_rank(view: Option<ElementView>) -> u8 {
    match view {
        None => 0,
        Some(ElementView::Snapshot) => 1,
        Some(ElementView::Differential) => 2,
    }
}
