use serde_json::{Map, Value};

pub(crate) fn normalize_structural_field(field: &str, value: &Value) -> Value {
    match field {
        "representation" | "condition" | "contextInvariant" => sort_array(value, canonicalize),
        "type" => normalize_types(value),
        "constraint" => normalize_constraints(value),
        _ => canonicalize(value),
    }
}

pub(crate) fn validate_resource_structural_field(field: &str, value: &Value) -> Result<(), String> {
    match field {
        "kind" | "type" | "baseDefinition" | "derivation" | "fhirVersion" => {
            validate_non_empty_string(value, &format!("{field} must be a non-empty string"))
        }
        "abstract" => validate_bool(value, "abstract must be a boolean"),
        "contextInvariant" => validate_array(value, "contextInvariant must be an array"),
        "context" => {
            let entries = value
                .as_array()
                .ok_or_else(|| "context must be an array of objects".to_owned())?;
            for (index, entry) in entries.iter().enumerate() {
                let object = entry
                    .as_object()
                    .ok_or_else(|| format!("context[{index}] must be an object"))?;
                validate_required_string_primitive(object, "type", &format!("context[{index}]"))?;
                validate_required_string_primitive(
                    object,
                    "expression",
                    &format!("context[{index}]"),
                )?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub(crate) fn validate_element_structural_field(field: &str, value: &Value) -> Result<(), String> {
    match field {
        "path" | "sliceName" | "contentReference" | "meaningWhenMissing" | "orderMeaning"
        | "isModifierReason" => {
            validate_non_empty_string(value, &format!("{field} must be a non-empty string"))
        }
        "sliceIsConstraining" | "mustSupport" | "isModifier" | "isSummary" => {
            validate_bool(value, &format!("{field} must be a boolean"))
        }
        "representation" | "condition" => {
            validate_array(value, &format!("{field} must be an array"))
        }
        "slicing" | "binding" => validate_object(value, &format!("{field} must be an object")),
        "min" => validate_unsigned_integer(value, "min must be a non-negative integer"),
        "max" => validate_non_empty_string(value, "max must be a non-empty string"),
        "maxLength" => validate_integer(value, "maxLength must be an integer"),
        "type" => {
            let types = value
                .as_array()
                .ok_or_else(|| "type must be an array of objects".to_owned())?;
            for (index, entry) in types.iter().enumerate() {
                let object = entry
                    .as_object()
                    .ok_or_else(|| format!("type[{index}] must be an object"))?;
                validate_required_string_primitive(object, "code", &format!("type[{index}]"))?;
                for nested in ["profile", "targetProfile", "aggregation"] {
                    if let Some(value) = object.get(nested) {
                        validate_array(value, &format!("type[{index}].{nested} must be an array"))?;
                    }
                }
            }
            Ok(())
        }
        "constraint" => {
            let constraints = value
                .as_array()
                .ok_or_else(|| "constraint must be an array of objects".to_owned())?;
            for (index, entry) in constraints.iter().enumerate() {
                let object = entry
                    .as_object()
                    .ok_or_else(|| format!("constraint[{index}] must be an object"))?;
                validate_required_string_primitive(object, "key", &format!("constraint[{index}]"))?;
            }
            Ok(())
        }
        "extension" => validate_object_array(value, "extension must be an array of objects"),
        _ => Ok(()),
    }
}

pub(crate) fn canonicalize(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut keys = object.keys().collect::<Vec<_>>();
            keys.sort();
            let mut normalized = Map::new();
            for key in keys {
                normalized.insert(key.clone(), canonicalize(&object[key]));
            }
            Value::Object(normalized)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonicalize).collect()),
        _ => value.clone(),
    }
}

fn normalize_types(value: &Value) -> Value {
    let Value::Array(types) = value else {
        return canonicalize(value);
    };
    let mut normalized = types
        .iter()
        .map(|entry| {
            let mut entry = canonicalize(entry);
            if let Value::Object(object) = &mut entry {
                for field in ["profile", "targetProfile", "aggregation"] {
                    if let Some(value) = object.get_mut(field) {
                        *value = sort_array(value, canonicalize);
                    }
                }
            }
            entry
        })
        .collect::<Vec<_>>();
    normalized.sort_by(|left, right| {
        type_code(left)
            .cmp(type_code(right))
            .then_with(|| stable_json(left).cmp(&stable_json(right)))
    });
    Value::Array(normalized)
}

fn normalize_constraints(value: &Value) -> Value {
    let Value::Array(constraints) = value else {
        return canonicalize(value);
    };
    let mut normalized = constraints.iter().map(canonicalize).collect::<Vec<_>>();
    normalized.sort_by(|left, right| {
        constraint_key(left)
            .cmp(constraint_key(right))
            .then_with(|| stable_json(left).cmp(&stable_json(right)))
    });
    Value::Array(normalized)
}

fn type_code(value: &Value) -> &str {
    value
        .as_object()
        .and_then(|object| object.get("code"))
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn constraint_key(value: &Value) -> &str {
    value
        .as_object()
        .and_then(|object| object.get("key"))
        .and_then(Value::as_str)
        .unwrap_or("")
}

fn validate_array(value: &Value, message: &str) -> Result<(), String> {
    if value.is_array() {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn validate_object(value: &Value, message: &str) -> Result<(), String> {
    if value.is_object() {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn validate_object_array(value: &Value, message: &str) -> Result<(), String> {
    let values = value.as_array().ok_or_else(|| message.to_owned())?;
    if values.iter().all(Value::is_object) {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn validate_non_empty_string(value: &Value, message: &str) -> Result<(), String> {
    match value {
        Value::String(value) if !value.is_empty() => Ok(()),
        _ => Err(message.to_owned()),
    }
}

fn validate_bool(value: &Value, message: &str) -> Result<(), String> {
    if value.is_boolean() {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn validate_unsigned_integer(value: &Value, message: &str) -> Result<(), String> {
    if value.as_u64().is_some() {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn validate_integer(value: &Value, message: &str) -> Result<(), String> {
    if value.as_i64().is_some() || value.as_u64().is_some() {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn validate_required_string_primitive(
    object: &Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    let metadata_field = format!("_{field}");
    let metadata = object.get(&metadata_field);

    match object.get(field) {
        Some(Value::String(value)) if !value.is_empty() => {
            if let Some(metadata) = metadata {
                validate_primitive_metadata(metadata, &format!("{path}.{metadata_field}"))?;
            }
            Ok(())
        }
        Some(Value::String(_)) => Err(format!("{path}.{field} must not be an empty string")),
        Some(_) => Err(format!("{path}.{field} must be a string when present")),
        None => {
            let metadata = metadata.ok_or_else(|| {
                format!(
                    "{path}.{field} requires a non-empty string value or {metadata_field} primitive metadata"
                )
            })?;
            validate_primitive_metadata(metadata, &format!("{path}.{metadata_field}"))
        }
    }
}

fn validate_primitive_metadata(value: &Value, path: &str) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{path} must be an object"))?;

    let has_id = match object.get("id") {
        None => false,
        Some(Value::String(value)) if !value.is_empty() => true,
        Some(Value::String(_)) => return Err(format!("{path}.id must not be an empty string")),
        Some(_) => return Err(format!("{path}.id must be a string when present")),
    };

    let has_extensions = match object.get("extension") {
        None => false,
        Some(Value::Array(extensions)) if extensions.is_empty() => {
            return Err(format!("{path}.extension must not be empty"));
        }
        Some(Value::Array(extensions)) => {
            for (index, extension) in extensions.iter().enumerate() {
                if !extension.is_object() {
                    return Err(format!("{path}.extension[{index}] must be an object"));
                }
            }
            true
        }
        Some(_) => return Err(format!("{path}.extension must be an array when present")),
    };

    if has_id || has_extensions {
        Ok(())
    } else {
        Err(format!(
            "{path} must contain a non-empty id or extension array"
        ))
    }
}

fn sort_array(value: &Value, normalize: fn(&Value) -> Value) -> Value {
    let Value::Array(values) = value else {
        return normalize(value);
    };
    let mut normalized = values.iter().map(normalize).collect::<Vec<_>>();
    normalized.sort_by_key(stable_json);
    Value::Array(normalized)
}

fn stable_json(value: &Value) -> String {
    serde_json::to_string(value).expect("serde_json::Value serialization is infallible")
}
