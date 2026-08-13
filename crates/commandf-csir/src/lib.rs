//! commandF Clinical Semantic Intermediate Representation (CSIR).
//!
//! CSIR is not a replacement healthcare wire standard. It is a typed compiler
//! representation used to preserve source assertions while commandF translates
//! between heterogeneous healthcare dialects.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Stable identifier inside one CSIR graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemanticId(pub String);

/// Content-addressed reference. The algorithm name is explicit to avoid
/// treating hashes from different algorithms as interchangeable.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    pub algorithm: String,
    pub value: String,
}

/// Identifies the source or target healthcare dialect and its version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialectRef {
    /// Examples: `hl7-fhir`, `openehr`, `omop-cdm`, `hl7-v2`, `cda`, `dicom`.
    pub system: String,
    /// Exact specification/model version, not a floating alias.
    pub version: String,
    /// Optional implementation/profile/package identifiers that further
    /// constrain the dialect.
    #[serde(default)]
    pub constraints: Vec<String>,
}

/// Points back to the exact source assertion that produced a CSIR assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePointer {
    pub artifact: ContentHash,
    /// Source-native path, e.g. FHIRPath-like location, HL7 field path, AQL
    /// path, column name, DICOM tag path, or another adapter-defined pointer.
    pub path: String,
    #[serde(default)]
    pub fragment_hash: Option<ContentHash>,
}

/// A terminology coding retained without assuming one vocabulary is globally
/// canonical.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coding {
    pub system: String,
    pub code: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub display: Option<String>,
}

/// Quantity value. `value` is lexical decimal text rather than binary floating
/// point so importers can preserve source precision exactly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub system: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
}

/// Temporal values preserve the source lexical representation and an explicit
/// precision label. Adapters may add normalized forms in attributes without
/// destroying the source value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalValue {
    pub lexical: String,
    /// Examples: `year`, `month`, `day`, `second`, `millisecond`, `interval`.
    pub precision: String,
    #[serde(default)]
    pub timezone: Option<String>,
}

/// A semantic value deliberately smaller than any one healthcare standard.
/// Source-native payloads can be retained as opaque content-addressed evidence
/// until a richer CSIR type is defined.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum SemanticValue {
    String(String),
    Boolean(bool),
    Integer(i64),
    Decimal(String),
    Coding(Coding),
    Quantity(Quantity),
    Temporal(TemporalValue),
    Reference(SemanticId),
    Bytes(ContentHash),
    List(Vec<SemanticValue>),
}

/// A typed relationship between assertions/entities in the semantic graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relation {
    pub predicate: String,
    pub target: SemanticId,
}

/// One source-grounded clinical/data assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assertion {
    pub id: SemanticId,
    /// Optional clinical/data concept identifying what this assertion means.
    #[serde(default)]
    pub concept: Option<Coding>,
    pub value: SemanticValue,
    #[serde(default)]
    pub relations: Vec<Relation>,
    /// Exact source evidence. Generated assertions must instead declare their
    /// generation rule in `attributes` and transformation evidence.
    #[serde(default)]
    pub sources: Vec<SourcePointer>,
    /// Namespaced extension bag for lossless adapter-specific facts. This bag
    /// is evidence, not a license for untyped core semantics.
    #[serde(default)]
    pub attributes: BTreeMap<String, String>,
}

/// Root CSIR graph exchanged between compiler passes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClinicalGraph {
    pub dialect: DialectRef,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LossKind {
    Dropped,
    Generalized,
    Narrowed,
    Split,
    Merged,
    Defaulted,
    CodeApproximation,
    UnitConversion,
    PrecisionReduction,
    CardinalityReduction,
    TemporalApproximation,
    ContextLoss,
    ReferenceLoss,
    UnsupportedTargetFeature,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LossSeverity {
    Informational,
    Warning,
    Significant,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Recoverability {
    FullyRecoverable,
    ConditionallyRecoverable,
    Irreversible,
    Unknown,
}

/// Explicit record of an information change introduced by a transformation.
/// A transformation is never allowed to silently discard a known unsupported
/// fact; the lowering pass must either preserve it or emit a LossEvent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LossEvent {
    pub id: String,
    pub kind: LossKind,
    pub severity: LossSeverity,
    pub recoverability: Recoverability,
    pub source: SourcePointer,
    #[serde(default)]
    pub target_path: Option<String>,
    pub rule_id: String,
    pub explanation: String,
    #[serde(default)]
    pub evidence: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantity_keeps_decimal_lexical_precision() {
        let q = Quantity {
            value: "1.2300".into(),
            unit: Some("mg".into()),
            system: Some("http://unitsofmeasure.org".into()),
            code: Some("mg".into()),
        };

        let json = serde_json::to_string(&q).expect("serialize quantity");
        let roundtrip: Quantity = serde_json::from_str(&json).expect("deserialize quantity");
        assert_eq!(roundtrip.value, "1.2300");
    }

    #[test]
    fn loss_event_is_machine_readable() {
        let loss = LossEvent {
            id: "loss-1".into(),
            kind: LossKind::CardinalityReduction,
            severity: LossSeverity::Significant,
            recoverability: Recoverability::Irreversible,
            source: SourcePointer {
                artifact: ContentHash {
                    algorithm: "sha256".into(),
                    value: "abc".into(),
                },
                path: "source.items[1]".into(),
                fragment_hash: None,
            },
            target_path: Some("target.item".into()),
            rule_id: "map-1".into(),
            explanation: "target allows one item while source supplied two".into(),
            evidence: BTreeMap::new(),
        };

        let value = serde_json::to_value(loss).expect("serialize loss");
        assert_eq!(value["kind"], "cardinality_reduction");
        assert_eq!(value["recoverability"], "irreversible");
    }
}
