use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RealIgCorpus {
    pub schema: u64,
    pub selection_policy: CorpusSelectionPolicy,
    pub cases: Vec<RealIgCase>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusSelectionPolicy {
    FrozenPreResultV1,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RealIgCase {
    pub id: String,
    pub package: String,
    pub before: CorpusPackageState,
    pub after: CorpusPackageState,
    pub fhir_version: String,
    pub publisher: String,
    pub change_evidence_url: String,
    pub rights_evidence_url: String,
    pub rights_mode: CorpusRightsMode,
    pub oracle_mode: CorpusOracleMode,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusPackageState {
    pub version: String,
    pub archive_sha256: String,
    pub archive_bytes: u64,
    pub publication_url: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusRightsMode {
    MetadataOnlyNoRedistribution,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusOracleMode {
    ChangedStructureDefinitionsOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusPackageSide {
    Before,
    After,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusPackageAttestation {
    pub case_id: String,
    pub package: String,
    pub side: CorpusPackageSide,
    pub version: String,
    pub sha256: String,
    pub archive_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusSummaryPackageState {
    pub version: String,
    pub sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorpusCaseStatus {
    Complete,
    AcquisitionFailed,
    AttestationFailed,
    StructuralFailed,
    CompatibilityFailed,
    TerminologyFailed,
    OracleFailed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusStructuralSummary {
    pub changes: usize,
    pub report_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusCompatibilitySummary {
    pub findings: usize,
    pub breaking: usize,
    pub risky: usize,
    pub additive: usize,
    pub producer: usize,
    pub consumer: usize,
    pub report_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusTerminologySummary {
    pub code_system_changes: usize,
    pub value_set_changes: usize,
    pub binding_refinements: usize,
    pub report_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusOracleSummary {
    pub compared: usize,
    pub agreement: usize,
    pub commandf_only: usize,
    pub authority_only: usize,
    pub both_changed: usize,
    pub uncomparable: usize,
    pub report_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusCaseSummary {
    pub case_id: String,
    pub package: String,
    pub before: CorpusSummaryPackageState,
    pub after: CorpusSummaryPackageState,
    pub status: CorpusCaseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structural: Option<CorpusStructuralSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<CorpusCompatibilitySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminology: Option<CorpusTerminologySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle: Option<CorpusOracleSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusRunSummary {
    pub schema: u32,
    pub manifest_sha256: String,
    pub cases: Vec<CorpusCaseSummary>,
}

impl CorpusRunSummary {
    pub const SCHEMA_V1: u32 = 1;

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
