use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContextPackageIdentity {
    pub name: String,
    pub version: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContextPackageNode {
    pub identity: ContextPackageIdentity,
    pub source: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContextArtifactIdentity {
    pub package: ContextPackageIdentity,
    pub filename: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContextArtifactNode {
    pub identity: ContextArtifactIdentity,
    pub resource_type: String,
    pub id: Option<String>,
    pub canonical_url: Option<String>,
    pub canonical_version: Option<String>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContextPackageDependencyEdge {
    pub from: ContextPackageIdentity,
    pub to: ContextPackageIdentity,
    pub declared_constraint: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalReferenceRelation {
    StructureBaseDefinition,
    StructureTypeProfile,
    StructureTypeTargetProfile,
    StructureBindingValueSet,
    ValueSetIncludeSystem,
    ValueSetIncludeValueSet,
    ValueSetExcludeSystem,
    ValueSetExcludeValueSet,
    CodeSystemSupplements,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonicalResolutionStatus {
    Resolved,
    External,
    Ambiguous,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ContextCanonicalReferenceEdge {
    pub source: ContextArtifactIdentity,
    pub relation: CanonicalReferenceRelation,
    pub source_path: String,
    pub source_element_id: Option<String>,
    pub canonical: String,
    pub resolution: CanonicalResolutionStatus,
    pub candidates: Vec<ContextArtifactIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextCoverage {
    pub extractor_schema: u32,
    pub supported_source_resource_types: Vec<String>,
    pub unsupported_source_resource_types: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ContextGraphReport {
    pub schema: u32,
    pub lock_schema: u32,
    pub root_requests: Vec<String>,
    pub packages: Vec<ContextPackageNode>,
    pub artifacts: Vec<ContextArtifactNode>,
    pub package_dependency_edges: Vec<ContextPackageDependencyEdge>,
    pub canonical_reference_edges: Vec<ContextCanonicalReferenceEdge>,
    pub coverage: ContextCoverage,
}

impl ContextGraphReport {
    pub const SCHEMA_V1: u32 = 1;

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
