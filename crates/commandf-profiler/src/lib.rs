//! Source profiling primitives for commandF.
//!
//! Profiling describes observed source behavior before mapping. It does not
//! infer clinical facts and does not mutate the source.

use commandf_csir::{ContentHash, DialectRef};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldObservation {
    pub path: String,
    pub present_documents: u64,
    pub null_values: u64,
    #[serde(default)]
    pub type_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub examples: Vec<String>,
}

impl FieldObservation {
    pub fn presence_basis_points(&self, total_documents: u64) -> u16 {
        if total_documents == 0 {
            return 0;
        }
        ((self.present_documents.saturating_mul(10_000) / total_documents).min(10_000)) as u16
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceProfile {
    pub profile_schema: String,
    pub artifact: ContentHash,
    pub dialect: DialectRef,
    pub sampled_documents: u64,
    #[serde(default)]
    pub fields: Vec<FieldObservation>,
}

#[derive(Default)]
struct Accumulator {
    present_documents: u64,
    null_values: u64,
    type_counts: BTreeMap<String, u64>,
    examples: Vec<String>,
}

pub fn profile_json_documents(
    artifact: ContentHash,
    dialect: DialectRef,
    documents: &[Value],
) -> SourceProfile {
    let mut stats: BTreeMap<String, Accumulator> = BTreeMap::new();

    for document in documents {
        let mut seen = BTreeSet::new();
        visit(document, "$", &mut seen, &mut stats);
        for path in seen {
            if let Some(accumulator) = stats.get_mut(&path) {
                accumulator.present_documents += 1;
            }
        }
    }

    let fields = stats
        .into_iter()
        .map(|(path, accumulator)| FieldObservation {
            path,
            present_documents: accumulator.present_documents,
            null_values: accumulator.null_values,
            type_counts: accumulator.type_counts,
            examples: accumulator.examples,
        })
        .collect();

    SourceProfile {
        profile_schema: "commandf.source-profile/0".into(),
        artifact,
        dialect,
        sampled_documents: documents.len() as u64,
        fields,
    }
}

fn visit(
    value: &Value,
    path: &str,
    seen: &mut BTreeSet<String>,
    stats: &mut BTreeMap<String, Accumulator>,
) {
    seen.insert(path.to_string());
    let accumulator = stats.entry(path.to_string()).or_default();
    *accumulator
        .type_counts
        .entry(json_type(value).to_string())
        .or_insert(0) += 1;

    if value.is_null() {
        accumulator.null_values += 1;
    }

    if accumulator.examples.len() < 3 {
        if let Some(example) = scalar_example(value) {
            if !accumulator.examples.contains(&example) {
                accumulator.examples.push(example);
            }
        }
    }

    match value {
        Value::Object(object) => {
            for (key, child) in object {
                visit(child, &format!("{path}.{key}"), seen, stats);
            }
        }
        Value::Array(items) => {
            let child_path = format!("{path}[]");
            for child in items {
                visit(child, &child_path, seen, stats);
            }
        }
        _ => {}
    }
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(number) if number.is_i64() || number.is_u64() => "integer",
        Value::Number(_) => "decimal",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn scalar_example(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".into()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::String(value) => Some(value.clone()),
        Value::Array(_) | Value::Object(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn hash() -> ContentHash {
        ContentHash {
            algorithm: "sha256".into(),
            value: "source".into(),
        }
    }

    fn dialect() -> DialectRef {
        DialectRef {
            system: "hl7-fhir".into(),
            version: "4.0.1".into(),
            constraints: vec![],
        }
    }

    #[test]
    fn profiler_records_presence_types_and_nulls_without_mutating_input() {
        let documents = vec![
            json!({"resourceType":"Patient","active":true,"identifier":[{"value":"A"}]}),
            json!({"resourceType":"Patient","active":null}),
        ];
        let profile = profile_json_documents(hash(), dialect(), &documents);

        let active = profile
            .fields
            .iter()
            .find(|field| field.path == "$.active")
            .expect("active field");
        assert_eq!(active.present_documents, 2);
        assert_eq!(active.null_values, 1);
        assert_eq!(
            active.presence_basis_points(profile.sampled_documents),
            10_000
        );
        assert_eq!(active.type_counts.get("boolean"), Some(&1));
        assert_eq!(active.type_counts.get("null"), Some(&1));

        let identifier_value = profile
            .fields
            .iter()
            .find(|field| field.path == "$.identifier[].value")
            .expect("identifier value");
        assert_eq!(identifier_value.present_documents, 1);
        assert_eq!(
            identifier_value.presence_basis_points(profile.sampled_documents),
            5_000
        );
    }
}
