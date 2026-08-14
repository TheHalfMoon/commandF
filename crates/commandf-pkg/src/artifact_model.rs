use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackageInspection {
    pub schema: u32,
    pub package_name: String,
    pub package_version: String,
    pub archive_sha256: String,
    pub resources: Vec<ResourceArtifact>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceArtifact {
    pub filename: String,
    pub resource_type: String,
    pub id: Option<String>,
    pub canonical_url: Option<String>,
    pub canonical_version: Option<String>,
    pub sha256: String,
    pub elements: Vec<ElementAddress>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ElementAddress {
    pub view: ElementView,
    pub element_id: String,
    pub path: Option<String>,
    pub slice_name: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElementView {
    Snapshot,
    Differential,
}

impl PackageInspection {
    pub const SCHEMA_V1: u32 = 1;

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
