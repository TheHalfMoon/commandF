use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ElementView;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StructuralDiffReport {
    pub schema: u32,
    pub package_name: String,
    pub before: PackageEvidence,
    pub after: PackageEvidence,
    pub changes: Vec<StructuralChange>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackageEvidence {
    pub version: String,
    pub archive_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ResourceKey {
    pub kind: ResourceKeyKind,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKeyKind {
    Canonical,
    ResourceId,
    Filename,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuralChangeKind {
    ResourceAdded,
    ResourceRemoved,
    ResourceFilenameChanged,
    ResourceVersionChanged,
    ResourceTypeChanged,
    ResourceIdChanged,
    ResourceBytesChanged,
    StructureFieldChanged,
    ViewAdded,
    ViewRemoved,
    ElementAdded,
    ElementRemoved,
    ElementFieldChanged,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StructuralChange {
    pub kind: StructuralChangeKind,
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

impl StructuralDiffReport {
    pub const SCHEMA_V1: u32 = 1;

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
