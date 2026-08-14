use serde_json::{Map, Value};

pub(crate) fn normalize_structural_field(field: &str, value: &Value) -> Value {
    match field {
        "representation" | "condition" | "contextInvariant" => sort_array(value, canonicalize),
        "type" => normalize_types(value),
        "constraint" => normalize_constraints(value),
        _ => canonicalize(value),
    }
}

pub(crate) fn validate_resource_structural_field(
    field: &str,
    value: &Value,
) -> Result<(), String> {
    match field {
        "contextInvariant" => validate_string_array(value, "expected an array of strings"),
        "context" => {
            let entries = value
                .as_array()
                .ok_or_else(|| "expected an array of context objects".to_owned())?;
            for (index, entry) in entries.iter().enumerate() {
                let object = entry
                    .as_object()
                    .ok_or_else(|| format!("context[{index}] must be an object"))?;
                require_string(object, "type", &format!("context[{index}]"))?;
                require_string(object, "expression", &format!("context[{index}]"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub(crate) fn validate_element_structural_field(
    field: &str,
    value: &Value,
) -> Result<(), String> {
    match field {
        "representation" | "condition" => {
            validate_string_array(value, "expected an array of strings")
        }
        "type" => {
            let types = value
                .as_array()
                .ok_or_else(|| "expected an array of type objects".to_owned())?;
            for (index, entry) in types.iter().enumerate() {
                let object = entry
                    .as_object()
                    .ok_or_else(|| format!("type[{index}] must be an object"))?;
                require_string(object, "code", &format!("type[{index}]"))?;
                for nested in ["profile", "targetProfile", "aggregation"] {
                    if let Some(value) = object.get(nested) {
                        validate_string_array(
                            value,
                            &format!("type[{index}].{nested} must be an array of strings"),
                        )?;
                    }
                }
            }
            Ok(())
        }
        "constraint" => {
            let constraints = value
                .as_array()
                .ok_or_else(|| "expected an array of constraint objects".to_owned())?;
            for (index, entry) in constraints.iter().enumerate() {
                let object = entry
                    .as_object()
                    .ok_or_else(|| format!("constraint[{index}] must be an object"))?;
                require_string(object, "key", &format!("constraint[{index}]"))?;
            }
            Ok(())
        }
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

fn validate_string_array(value: &Value, message: &str) -> Result<(), String> {
    let values = value.as_array().ok_or_else(|| message.to_owned())?;
    if values.iter().all(Value::is_string) {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn require_string(object: &Map<String, Value>, field: &str, path: &str) -> Result<(), String> {
    match object.get(field) {
        Some(Value::String(value)) if !value.is_empty() => Ok(()),
        _ => Err(format!("{path}.{field} must be a non-empty string")),
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
