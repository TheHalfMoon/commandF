use serde::{Deserialize, Serialize};

use crate::{CheckReport, PackageEvidence};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FindingFingerprint {
    pub schema: u32,
    pub digest: String,
}

impl FindingFingerprint {
    pub const SCHEMA_V1: u32 = 1;
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateSuppression {
    pub finding_fingerprint: FindingFingerprint,
    pub rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateSuppressions {
    pub schema: u32,
    pub suppressions: Vec<GateSuppression>,
}

impl GateSuppressions {
    pub const SCHEMA_V1: u32 = 1;

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityGateDisposition {
    New,
    Baseline,
    Suppressed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityGateFinding {
    pub finding_index: usize,
    pub fingerprint: FindingFingerprint,
    pub disposition: QualityGateDisposition,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_suppression: Option<GateSuppression>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityGateBaselineEvidence {
    pub canonical_sha256: String,
    pub fingerprint_schema: u32,
    pub package_name: String,
    pub ruleset: String,
    pub before: PackageEvidence,
    pub after: PackageEvidence,
    pub finding_count: usize,
    pub fingerprints: Vec<FindingFingerprint>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityGateSuppressionEvidence {
    pub canonical_sha256: String,
    pub schema: u32,
    pub fingerprint_schema: u32,
    pub entry_count: usize,
    pub suppressions: Vec<GateSuppression>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityGateDecision {
    pub passed: bool,
    pub total_findings: usize,
    pub selected_findings: usize,
    pub new_findings: usize,
    pub baseline_findings: usize,
    pub suppressed_findings: usize,
    pub new_selected_breaking_findings: usize,
    pub new_selected_risky_findings: usize,
    pub new_selected_additive_findings: usize,
    pub blocking_findings: usize,
    pub unused_suppressions: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityGateReport {
    pub schema: u32,
    pub current: CheckReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<QualityGateBaselineEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppression_evidence: Option<QualityGateSuppressionEvidence>,
    pub findings: Vec<QualityGateFinding>,
    pub unused_suppressions: Vec<FindingFingerprint>,
    pub decision: QualityGateDecision,
}

impl QualityGateReport {
    pub const SCHEMA_V1: u32 = 1;

    pub fn from_json_slice(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
