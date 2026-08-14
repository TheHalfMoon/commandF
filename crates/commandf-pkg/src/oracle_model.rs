use serde::{Deserialize, Serialize};

use crate::{ResourceKey, StructuralChangeKind, StructuralDiffReport};

pub const HL7_ORACLE_PROJECT: &str = "hapifhir/org.hl7.fhir.core";
pub const HL7_ORACLE_RELEASE: &str = "6.10.2";
pub const HL7_ORACLE_SOURCE_COMMIT: &str = "d06577dbc5c62c74a2a8823fbc4830a3024d5b0b";
pub const HL7_VALIDATOR_JAR_SHA256: &str =
    "a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleIdentity {
    pub project: String,
    pub release: String,
    pub source_commit: String,
}

impl OracleIdentity {
    pub fn pinned_hl7() -> Self {
        Self {
            project: HL7_ORACLE_PROJECT.to_owned(),
            release: HL7_ORACLE_RELEASE.to_owned(),
            source_commit: HL7_ORACLE_SOURCE_COMMIT.to_owned(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleResourceIdentity {
    pub url: Option<String>,
    pub version: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
}

impl OracleResourceIdentity {
    pub fn canonical_identity(&self) -> Option<String> {
        let url = self.url.as_deref()?.trim();
        if url.is_empty() {
            return None;
        }
        match self
            .version
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(version) => Some(format!("{url}|{version}")),
            None => Some(url.to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleChangeState {
    Unknown,
    NotChanged,
    Changed,
    CannotEvaluate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleStates {
    pub metadata: OracleChangeState,
    pub definitions: OracleChangeState,
    pub content: OracleChangeState,
    pub content_interpretation: OracleChangeState,
}

impl OracleStates {
    pub fn has_change_signal(&self) -> bool {
        [
            self.metadata,
            self.definitions,
            self.content,
            self.content_interpretation,
        ]
        .into_iter()
        .any(|state| {
            matches!(
                state,
                OracleChangeState::Changed | OracleChangeState::CannotEvaluate
            )
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleMessageLevel {
    Fatal,
    Error,
    Warning,
    Information,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleMessage {
    pub level: OracleMessageLevel,
    pub location: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hl7OracleReport {
    pub schema: u32,
    pub oracle: OracleIdentity,
    pub left: OracleResourceIdentity,
    pub right: OracleResourceIdentity,
    pub states: OracleStates,
    pub messages: Vec<OracleMessage>,
}

impl Hl7OracleReport {
    pub const SCHEMA_V1: u32 = 1;

    pub fn has_change_signal(&self) -> bool {
        self.states.has_change_signal() || !self.messages.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleResourceStatus {
    Agreement,
    CommandfOnly,
    AuthorityOnly,
    BothChanged,
    Uncomparable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OracleResourceResult {
    pub resource: ResourceKey,
    pub status: OracleResourceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oracle: Option<Hl7OracleReport>,
    pub commandf_change_kinds: Vec<StructuralChangeKind>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OracleDivergenceReport {
    pub schema: u32,
    pub oracle: OracleIdentity,
    pub package_name: String,
    pub structural_diff: StructuralDiffReport,
    pub resources: Vec<OracleResourceResult>,
}

impl OracleDivergenceReport {
    pub const SCHEMA_V1: u32 = 1;

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
