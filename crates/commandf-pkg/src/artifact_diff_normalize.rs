use serde_json::{Map, Value};

pub(crate) fn normalize_structural_field(field: &str, value: &Value) -> Value {
    match field {
        "representation" | "condition" | "contextInvariant" => sort_array(value, canonicalize),
        "type" => normalize_types(value),
        "constraint" => normalize_constraints(value),
        _ => canonicalize(value),
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
    normalized.sort_by_key(stable_json);
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

fn constraint_key(value: &Value) -> &str {
    value
        .as_object()
        .and_then(|object| object.get("key"))
        .and_then(Value::as_str)
        .unwrap_or("")
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
