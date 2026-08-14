use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ElementView, PackageEvidence, ResourceKey, StructuralChangeKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CompatibilitySeverity {
    Breaking,
    Risky,
    Additive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityDirection {
    Producer,
    Consumer,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityFinding {
    pub rule_id: String,
    pub severity: CompatibilitySeverity,
    pub direction: CompatibilityDirection,
    pub source_kind: StructuralChangeKind,
    pub message: String,
    pub resource: ResourceKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<ElementView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityReport {
    pub schema: u32,
    pub ruleset: String,
    pub package_name: String,
    pub before: PackageEvidence,
    pub after: PackageEvidence,
    pub findings: Vec<CompatibilityFinding>,
}

impl CompatibilityReport {
    pub const SCHEMA_V1: u32 = 1;
    pub const RULESET_V1: &'static str = "cf04-rules-v1";

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
