use serde::{Deserialize, Serialize};

use crate::{
    CanonicalReferenceRelation, CanonicalResolutionStatus, ContextArtifactIdentity,
    ContextCoverage, ContextPackageIdentity, ContextPackageNode,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactSide {
    Before,
    After,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImpactSeedKind {
    Added,
    Removed,
    Modified,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ImpactSeed {
    pub kind: ImpactSeedKind,
    pub canonical: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<ContextArtifactIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<ContextArtifactIdentity>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ImpactArtifactPathStep {
    pub source: ContextArtifactIdentity,
    pub target: ContextArtifactIdentity,
    pub relation: CanonicalReferenceRelation,
    pub source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_element_id: Option<String>,
    pub canonical: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ImpactArtifactRelation {
    pub impacted: ContextArtifactIdentity,
    pub seed: ContextArtifactIdentity,
    pub side: ImpactSide,
    pub path: Vec<ImpactArtifactPathStep>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ImpactPackagePathStep {
    pub source: ContextPackageIdentity,
    pub target: ContextPackageIdentity,
    pub declared_constraint: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ImpactPackageRelation {
    pub impacted: ContextPackageIdentity,
    pub subject: ContextPackageIdentity,
    pub side: ImpactSide,
    pub path: Vec<ImpactPackagePathStep>,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ImpactUnresolvedBoundary {
    pub source: ContextArtifactIdentity,
    pub seed: ContextArtifactIdentity,
    pub side: ImpactSide,
    pub relation: CanonicalReferenceRelation,
    pub source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_element_id: Option<String>,
    pub canonical: String,
    pub resolution: CanonicalResolutionStatus,
    pub candidates: Vec<ContextArtifactIdentity>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImpactGraphEvidence {
    pub graph_schema: u32,
    pub lock_schema: u32,
    pub root_requests: Vec<String>,
    pub subject: ContextPackageNode,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImpactCoverage {
    pub before: ContextCoverage,
    pub after: ContextCoverage,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImpactSubject {
    pub package_name: String,
    pub before: ContextPackageIdentity,
    pub after: ContextPackageIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImpactReport {
    pub schema: u32,
    pub subject: ImpactSubject,
    pub before_evidence: ImpactGraphEvidence,
    pub after_evidence: ImpactGraphEvidence,
    pub seeds: Vec<ImpactSeed>,
    pub artifact_impacts: Vec<ImpactArtifactRelation>,
    pub package_impacts: Vec<ImpactPackageRelation>,
    pub unresolved_boundaries: Vec<ImpactUnresolvedBoundary>,
    pub coverage: ImpactCoverage,
}

impl ImpactReport {
    pub const SCHEMA_V1: u32 = 1;

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
