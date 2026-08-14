use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::{
    ResourceKey, TerminologyError, TerminologyIndeterminateReason, TerminologyMember,
    TerminologyProofMode, TerminologyRelation, TerminologySetDelta,
};

pub(crate) const MAX_TERMINOLOGY_MEMBERS: usize = 1_000_000;
pub(crate) const MAX_EXPANSION_PARAMETERS: usize = 10_000;
pub(crate) const MAX_PUBLIC_MEMBER_DELTA: usize = 100_000;

struct FiniteSet {
    members: BTreeSet<TerminologyMember>,
    version: Option<String>,
    binding_proof_eligible: bool,
    reason: Option<TerminologyIndeterminateReason>,
    parameters: Option<Vec<String>>,
}

enum Extraction {
    Finite(FiniteSet),
    Indeterminate {
        reason: TerminologyIndeterminateReason,
        version: Option<String>,
    },
}

pub fn compare_complete_code_systems(
    resource: ResourceKey,
    before: &Value,
    after: &Value,
) -> Result<TerminologySetDelta, TerminologyError> {
    require_resource_type(before, "CodeSystem", &resource.value)?;
    require_resource_type(after, "CodeSystem", &resource.value)?;

    let before_url = required_string(before, "url", &resource.value)?;
    let after_url = required_string(after, "url", &resource.value)?;
    if before_url != after_url {
        return Err(invalid(
            &resource.value,
            "url",
            "matched CodeSystem canonical URLs differ",
        ));
    }

    let before_case = optional_bool(before, "caseSensitive", &resource.value)?;
    let after_case = optional_bool(after, "caseSensitive", &resource.value)?;
    if before_case != after_case {
        return Ok(indeterminate_delta(
            resource,
            "CodeSystem",
            optional_string(before, "version", &before_url)?,
            optional_string(after, "version", &after_url)?,
            TerminologyIndeterminateReason::CodeSystemCaseSensitivityChanged,
        ));
    }

    let before_set = extract_code_system(before, &before_url)?;
    let after_set = extract_code_system(after, &after_url)?;
    compare_extractions(
        resource,
        "CodeSystem",
        TerminologyProofMode::CodeSystemComplete,
        before_set,
        after_set,
    )
}

pub fn compare_value_set_expansions(
    resource: ResourceKey,
    before: &Value,
    after: &Value,
) -> Result<TerminologySetDelta, TerminologyError> {
    require_resource_type(before, "ValueSet", &resource.value)?;
    require_resource_type(after, "ValueSet", &resource.value)?;
    let before_url = required_string(before, "url", &resource.value)?;
    let after_url = required_string(after, "url", &resource.value)?;
    if before_url != after_url {
        return Err(invalid(
            &resource.value,
            "url",
            "matched ValueSet canonical URLs differ",
        ));
    }

    let before_set = extract_value_set(before, &before_url)?;
    let after_set = extract_value_set(after, &after_url)?;

    if let (Extraction::Finite(left), Extraction::Finite(right)) = (&before_set, &after_set) {
        if left.parameters != right.parameters {
            return Ok(indeterminate_delta(
                resource,
                "ValueSet",
                left.version.clone(),
                right.version.clone(),
                TerminologyIndeterminateReason::ExpansionContextMismatch,
            ));
        }
    }

    compare_extractions(
        resource,
        "ValueSet",
        TerminologyProofMode::ValueSetExpansion,
        before_set,
        after_set,
    )
}

fn extract_code_system(value: &Value, resource: &str) -> Result<Extraction, TerminologyError> {
    let version = optional_string(value, "version", resource)?;
    let content = required_string(value, "content", resource)?;
    if content != "complete" {
        return Ok(Extraction::Indeterminate {
            reason: TerminologyIndeterminateReason::CodeSystemNotComplete,
            version,
        });
    }
    if optional_bool(value, "compositional", resource)?.unwrap_or(false) {
        return Ok(Extraction::Indeterminate {
            reason: TerminologyIndeterminateReason::CodeSystemCompositional,
            version,
        });
    }

    let system = required_string(value, "url", resource)?;
    let mut codes = BTreeSet::new();
    let mut visited = 0_usize;
    if let Some(concepts) = optional_array(value, "concept", resource)? {
        flatten_code_system_concepts(concepts, &system, resource, &mut codes, &mut visited)?;
    }
    if let Some(count) = optional_u64(value, "count", resource)? {
        if count != codes.len() as u64 {
            return Ok(Extraction::Indeterminate {
                reason: TerminologyIndeterminateReason::CodeSystemCountMismatch,
                version,
            });
        }
    }

    Ok(Extraction::Finite(FiniteSet {
        members: codes,
        version,
        binding_proof_eligible: true,
        reason: None,
        parameters: None,
    }))
}

fn flatten_code_system_concepts(
    concepts: &[Value],
    system: &str,
    resource: &str,
    output: &mut BTreeSet<TerminologyMember>,
    visited: &mut usize,
) -> Result<(), TerminologyError> {
    for concept in concepts {
        *visited += 1;
        enforce_limit("terminology_members", *visited, MAX_TERMINOLOGY_MEMBERS)?;
        let object = concept.as_object().ok_or_else(|| {
            invalid(resource, "concept", "CodeSystem concept must be an object")
        })?;
        let code = required_object_string(object, "code", resource, "concept.code")?;
        let member = TerminologyMember {
            system: system.to_owned(),
            version: None,
            code: code.clone(),
        };
        if !output.insert(member) {
            return Err(TerminologyError::DuplicateCode {
                resource: resource.to_owned(),
                code,
            });
        }
        if let Some(children) = object.get("concept") {
            let children = children.as_array().ok_or_else(|| {
                invalid(resource, "concept.concept", "nested concepts must be an array")
            })?;
            flatten_code_system_concepts(children, system, resource, output, visited)?;
        }
    }
    Ok(())
}

fn extract_value_set(value: &Value, resource: &str) -> Result<Extraction, TerminologyError> {
    let version = optional_string(value, "version", resource)?;
    let Some(expansion) = value.get("expansion") else {
        return Ok(Extraction::Indeterminate {
            reason: TerminologyIndeterminateReason::MissingExpansion,
            version,
        });
    };
    let expansion = expansion
        .as_object()
        .ok_or_else(|| invalid(resource, "expansion", "ValueSet expansion must be an object"))?;

    if let Some(offset) = optional_object_u64(expansion, "offset", resource, "expansion.offset")? {
        if offset != 0 {
            return Ok(Extraction::Indeterminate {
                reason: TerminologyIndeterminateReason::IncompleteOrPagedExpansion,
                version,
            });
        }
    }

    let Some(total) = optional_object_u64(expansion, "total", resource, "expansion.total")? else {
        return Ok(Extraction::Indeterminate {
            reason: TerminologyIndeterminateReason::IncompleteOrPagedExpansion,
            version,
        });
    };

    let parameters = normalize_expansion_parameters(expansion, resource)?;
    let Some(parameters) = parameters else {
        return Ok(Extraction::Indeterminate {
            reason: TerminologyIndeterminateReason::UnsupportedExpansionParameter,
            version,
        });
    };

    let mut members = BTreeSet::new();
    let mut visited = 0_usize;
    let mut has_abstract_code = false;
    if let Some(contains) = expansion.get("contains") {
        let contains = contains.as_array().ok_or_else(|| {
            invalid(resource, "expansion.contains", "contains must be an array")
        })?;
        flatten_expansion_contains(
            contains,
            resource,
            &mut members,
            &mut visited,
            &mut has_abstract_code,
        )?;
    }

    if total != members.len() as u64 {
        return Ok(Extraction::Indeterminate {
            reason: TerminologyIndeterminateReason::IncompleteOrPagedExpansion,
            version,
        });
    }

    Ok(Extraction::Finite(FiniteSet {
        members,
        version,
        binding_proof_eligible: !has_abstract_code,
        reason: has_abstract_code.then_some(TerminologyIndeterminateReason::AbstractMemberPresent),
        parameters: Some(parameters),
    }))
}

fn flatten_expansion_contains(
    contains: &[Value],
    resource: &str,
    output: &mut BTreeSet<TerminologyMember>,
    visited: &mut usize,
    has_abstract_code: &mut bool,
) -> Result<(), TerminologyError> {
    for item in contains {
        *visited += 1;
        enforce_limit("terminology_members", *visited, MAX_TERMINOLOGY_MEMBERS)?;
        let object = item.as_object().ok_or_else(|| {
            invalid(
                resource,
                "expansion.contains",
                "expansion contains entry must be an object",
            )
        })?;
        let code = optional_object_string(object, "code", resource, "expansion.contains.code")?;
        if let Some(code) = code {
            let system = required_object_string(
                object,
                "system",
                resource,
                "expansion.contains.system",
            )?;
            let version = optional_object_string(
                object,
                "version",
                resource,
                "expansion.contains.version",
            )?;
            if optional_object_bool(
                object,
                "abstract",
                resource,
                "expansion.contains.abstract",
            )?
            .unwrap_or(false)
            {
                *has_abstract_code = true;
            }
            // Hierarchical expansions may repeat a concept for navigation. Membership
            // identity is therefore de-duplicated as system+version+code.
            output.insert(TerminologyMember {
                system,
                version,
                code,
            });
        }
        if let Some(children) = object.get("contains") {
            let children = children.as_array().ok_or_else(|| {
                invalid(
                    resource,
                    "expansion.contains.contains",
                    "nested contains must be an array",
                )
            })?;
            flatten_expansion_contains(children, resource, output, visited, has_abstract_code)?;
        }
    }
    Ok(())
}

fn normalize_expansion_parameters(
    expansion: &Map<String, Value>,
    resource: &str,
) -> Result<Option<Vec<String>>, TerminologyError> {
    let Some(parameters) = expansion.get("parameter") else {
        return Ok(Some(Vec::new()));
    };
    let parameters = parameters.as_array().ok_or_else(|| {
        invalid(
            resource,
            "expansion.parameter",
            "expansion parameters must be an array",
        )
    })?;
    enforce_limit(
        "expansion_parameters",
        parameters.len(),
        MAX_EXPANSION_PARAMETERS,
    )?;
    let mut normalized = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let object = parameter.as_object().ok_or_else(|| {
            invalid(
                resource,
                "expansion.parameter",
                "expansion parameter must be an object",
            )
        })?;
        if object.contains_key("extension") || object.contains_key("modifierExtension") {
            return Ok(None);
        }
        let name = required_object_string(object, "name", resource, "expansion.parameter.name")?;
        let values = object
            .iter()
            .filter(|(key, _)| key.starts_with("value") && !key.starts_with("_value"))
            .collect::<Vec<_>>();
        if values.len() != 1 {
            return Err(invalid(
                resource,
                "expansion.parameter.value[x]",
                "exactly one value[x] field is required",
            ));
        }
        let (field, value) = values[0];
        if !(value.is_string() || value.is_boolean() || value.is_number()) {
            return Ok(None);
        }
        let encoded = serde_json::to_string(value).map_err(|source| TerminologyError::Json {
            file: resource.to_owned(),
            source,
        })?;
        normalized.push(format!("{name}\u{1f}{field}\u{1f}{encoded}"));
    }
    normalized.sort();
    Ok(Some(normalized))
}

fn compare_extractions(
    resource: ResourceKey,
    resource_type: &str,
    proof_mode: TerminologyProofMode,
    before: Extraction,
    after: Extraction,
) -> Result<TerminologySetDelta, TerminologyError> {
    let (before, after) = match (before, after) {
        (Extraction::Finite(before), Extraction::Finite(after)) => (before, after),
        (Extraction::Indeterminate { reason, version }, Extraction::Finite(after)) => {
            return Ok(indeterminate_delta(
                resource,
                resource_type,
                version,
                after.version,
                reason,
            ));
        }
        (Extraction::Finite(before), Extraction::Indeterminate { reason, version }) => {
            return Ok(indeterminate_delta(
                resource,
                resource_type,
                before.version,
                version,
                reason,
            ));
        }
        (
            Extraction::Indeterminate {
                reason,
                version: before_version,
            },
            Extraction::Indeterminate {
                version: after_version,
                ..
            },
        ) => {
            return Ok(indeterminate_delta(
                resource,
                resource_type,
                before_version,
                after_version,
                reason,
            ));
        }
    };

    let relation = relation(&before.members, &after.members);
    let added = bounded_delta(after.members.difference(&before.members).cloned().collect())?;
    let removed = bounded_delta(before.members.difference(&after.members).cloned().collect())?;
    let reason = before.reason.or(after.reason);
    Ok(TerminologySetDelta {
        resource,
        resource_type: resource_type.to_owned(),
        before_resource_version: before.version,
        after_resource_version: after.version,
        proof_mode: Some(proof_mode),
        relation,
        binding_proof_eligible: before.binding_proof_eligible && after.binding_proof_eligible,
        reason,
        before_count: Some(before.members.len()),
        after_count: Some(after.members.len()),
        added,
        removed,
    })
}

fn relation(
    before: &BTreeSet<TerminologyMember>,
    after: &BTreeSet<TerminologyMember>,
) -> TerminologyRelation {
    if before == after {
        TerminologyRelation::Equal
    } else if after.is_subset(before) {
        TerminologyRelation::Narrowed
    } else if before.is_subset(after) {
        TerminologyRelation::Widened
    } else {
        TerminologyRelation::Incomparable
    }
}

fn bounded_delta(
    values: Vec<TerminologyMember>,
) -> Result<Vec<TerminologyMember>, TerminologyError> {
    enforce_limit("public_member_delta", values.len(), MAX_PUBLIC_MEMBER_DELTA)?;
    Ok(values)
}

fn indeterminate_delta(
    resource: ResourceKey,
    resource_type: &str,
    before_version: Option<String>,
    after_version: Option<String>,
    reason: TerminologyIndeterminateReason,
) -> TerminologySetDelta {
    TerminologySetDelta {
        resource,
        resource_type: resource_type.to_owned(),
        before_resource_version: before_version,
        after_resource_version: after_version,
        proof_mode: None,
        relation: TerminologyRelation::Indeterminate,
        binding_proof_eligible: false,
        reason: Some(reason),
        before_count: None,
        after_count: None,
        added: Vec::new(),
        removed: Vec::new(),
    }
}

fn require_resource_type(
    value: &Value,
    expected: &str,
    resource: &str,
) -> Result<(), TerminologyError> {
    let actual = required_string(value, "resourceType", resource)?;
    if actual != expected {
        return Err(invalid(
            resource,
            "resourceType",
            format!("expected {expected}, got {actual}"),
        ));
    }
    Ok(())
}

fn required_string(value: &Value, field: &str, resource: &str) -> Result<String, TerminologyError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(resource, field, "resource must be an object"))?;
    required_object_string(object, field, resource, field)
}

fn optional_string(
    value: &Value,
    field: &str,
    resource: &str,
) -> Result<Option<String>, TerminologyError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(resource, field, "resource must be an object"))?;
    optional_object_string(object, field, resource, field)
}

fn required_object_string(
    object: &Map<String, Value>,
    field: &str,
    resource: &str,
    path: &str,
) -> Result<String, TerminologyError> {
    let value = object
        .get(field)
        .ok_or_else(|| invalid(resource, path, "required string is missing"))?;
    let value = value
        .as_str()
        .ok_or_else(|| invalid(resource, path, "must be a string"))?
        .trim();
    if value.is_empty() {
        return Err(invalid(resource, path, "must not be empty"));
    }
    Ok(value.to_owned())
}

fn optional_object_string(
    object: &Map<String, Value>,
    field: &str,
    resource: &str,
    path: &str,
) -> Result<Option<String>, TerminologyError> {
    let Some(value) = object.get(field) else {
        return Ok(None);
    };
    let value = value
        .as_str()
        .ok_or_else(|| invalid(resource, path, "must be a string"))?
        .trim();
    if value.is_empty() {
        return Err(invalid(resource, path, "must not be empty"));
    }
    Ok(Some(value.to_owned()))
}

fn optional_bool(
    value: &Value,
    field: &str,
    resource: &str,
) -> Result<Option<bool>, TerminologyError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(resource, field, "resource must be an object"))?;
    optional_object_bool(object, field, resource, field)
}

fn optional_object_bool(
    object: &Map<String, Value>,
    field: &str,
    resource: &str,
    path: &str,
) -> Result<Option<bool>, TerminologyError> {
    match object.get(field) {
        None => Ok(None),
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| invalid(resource, path, "must be a boolean")),
    }
}

fn optional_u64(
    value: &Value,
    field: &str,
    resource: &str,
) -> Result<Option<u64>, TerminologyError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(resource, field, "resource must be an object"))?;
    optional_object_u64(object, field, resource, field)
}

fn optional_object_u64(
    object: &Map<String, Value>,
    field: &str,
    resource: &str,
    path: &str,
) -> Result<Option<u64>, TerminologyError> {
    match object.get(field) {
        None => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| invalid(resource, path, "must be a non-negative integer")),
    }
}

fn optional_array<'a>(
    value: &'a Value,
    field: &str,
    resource: &str,
) -> Result<Option<&'a [Value]>, TerminologyError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(resource, field, "resource must be an object"))?;
    match object.get(field) {
        None => Ok(None),
        Some(value) => value
            .as_array()
            .map(|value| Some(value.as_slice()))
            .ok_or_else(|| invalid(resource, field, "must be an array")),
    }
}

fn invalid(
    resource: &str,
    field: impl Into<String>,
    message: impl Into<String>,
) -> TerminologyError {
    TerminologyError::InvalidField {
        resource: resource.to_owned(),
        field: field.into(),
        message: message.into(),
    }
}

fn enforce_limit(
    field: &'static str,
    actual: usize,
    limit: usize,
) -> Result<(), TerminologyError> {
    if actual > limit {
        return Err(TerminologyError::Limit {
            field,
            actual,
            limit,
        });
    }
    Ok(())
}
