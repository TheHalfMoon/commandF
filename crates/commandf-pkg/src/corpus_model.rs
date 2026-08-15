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
