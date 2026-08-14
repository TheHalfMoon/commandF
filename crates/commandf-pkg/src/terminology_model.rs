use serde::{Deserialize, Serialize};

use crate::{
    CompatibilityDirection, CompatibilityReport, CompatibilitySeverity, ElementView,
    PackageEvidence, ResourceKey,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminologyProofMode {
    CodeSystemComplete,
    ValueSetExpansion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminologyRelation {
    Equal,
    Narrowed,
    Widened,
    Incomparable,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminologyIndeterminateReason {
    MissingExpansion,
    IncompleteOrPagedExpansion,
    ExpansionContextMismatch,
    UnsupportedExpansionParameter,
    AbstractMemberPresent,
    CodeSystemNotComplete,
    CodeSystemCompositional,
    CodeSystemCaseSensitivityChanged,
    CodeSystemCountMismatch,
    UnresolvedValueSet,
    AmbiguousCanonical,
    UnsupportedBindingInteraction,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct TerminologyMember {
    pub system: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub code: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TerminologySetDelta {
    pub resource: ResourceKey,
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_mode: Option<TerminologyProofMode>,
    pub relation: TerminologyRelation,
    pub binding_proof_eligible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<TerminologyIndeterminateReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_count: Option<usize>,
    pub added: Vec<TerminologyMember>,
    pub removed: Vec<TerminologyMember>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BindingRefinement {
    pub resource: ResourceKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<ElementView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    pub before_value_set: String,
    pub after_value_set: String,
    pub relation: TerminologyRelation,
    pub proof_mode: Option<TerminologyProofMode>,
    pub binding_proof_eligible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<TerminologyIndeterminateReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<CompatibilitySeverity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<CompatibilityDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TerminologyDiffReport {
    pub schema: u32,
    pub ruleset: String,
    pub package_name: String,
    pub before: PackageEvidence,
    pub after: PackageEvidence,
    pub compatibility: CompatibilityReport,
    pub code_systems: Vec<TerminologySetDelta>,
    pub value_sets: Vec<TerminologySetDelta>,
    pub binding_refinements: Vec<BindingRefinement>,
}

impl TerminologyDiffReport {
    pub const SCHEMA_V1: u32 = 1;
    pub const RULESET_V1: &'static str = "cf07-terminology-v1";

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut bytes = serde_json::to_vec_pretty(self)?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
