//! Machine-readable transformation evidence for commandF.
//!
//! A certificate records what was transformed, with which exact contracts and
//! tools, what validators observed, and what semantic losses were declared.
//! It is evidence about a transformation run; it is not by itself a clinical
//! safety claim.

use commandf_csir::{ContentHash, DialectRef, LossEvent};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub content_hash: ContentHash,
    pub media_type: String,
    pub dialect: DialectRef,
    #[serde(default)]
    pub package_closure: Vec<ContentHash>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolRef {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub build_hash: Option<ContentHash>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Pass,
    Warning,
    Fail,
    NotRun,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEvidence {
    pub validator: ToolRef,
    pub status: ValidationStatus,
    #[serde(default)]
    pub profile_or_contracts: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
    #[serde(default)]
    pub report_hash: Option<ContentHash>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticCheckStatus {
    Pass,
    Fail,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticCheck {
    pub id: String,
    pub status: SemanticCheckStatus,
    pub explanation: String,
    #[serde(default)]
    pub evidence: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MappingRef {
    pub id: String,
    pub version: String,
    pub content_hash: ContentHash,
    #[serde(default)]
    pub source_language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologyEvidence {
    pub operation: String,
    pub terminology_system: String,
    #[serde(default)]
    pub terminology_version: Option<String>,
    pub resolver: ToolRef,
    #[serde(default)]
    pub request_hash: Option<ContentHash>,
    #[serde(default)]
    pub response_hash: Option<ContentHash>,
}

/// Coarse run state. `Verified` means the configured commandF verification
/// policy passed; the policy identifier must be recorded in the certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerificationState {
    Pending,
    Failed,
    VerifiedWithWarnings,
    Verified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformationCertificate {
    /// Version of the commandF certificate schema, independent of FHIR or any
    /// other healthcare standard version.
    pub certificate_schema: String,
    pub run_id: String,
    pub source: ArtifactRef,
    pub target: ArtifactRef,
    #[serde(default)]
    pub mappings: Vec<MappingRef>,
    #[serde(default)]
    pub terminology: Vec<TerminologyEvidence>,
    #[serde(default)]
    pub validations: Vec<ValidationEvidence>,
    #[serde(default)]
    pub semantic_checks: Vec<SemanticCheck>,
    #[serde(default)]
    pub losses: Vec<LossEvent>,
    /// Content-addressed root of field/assertion-level transformation evidence.
    #[serde(default)]
    pub evidence_root: Option<ContentHash>,
    /// Identifies the exact verification policy used to derive `state`.
    pub verification_policy: String,
    pub state: VerificationState,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
}

impl TransformationCertificate {
    /// Returns true only for successful policy outcomes. Callers must not infer
    /// that this is a clinical-safety certification.
    pub fn policy_verified(&self) -> bool {
        matches!(
            self.state,
            VerificationState::Verified | VerificationState::VerifiedWithWarnings
        )
    }

    pub fn has_irreversible_loss(&self) -> bool {
        use commandf_csir::Recoverability;
        self.losses
            .iter()
            .any(|loss| loss.recoverability == Recoverability::Irreversible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use commandf_csir::{DialectRef, Recoverability};

    fn hash(value: &str) -> ContentHash {
        ContentHash {
            algorithm: "sha256".into(),
            value: value.into(),
        }
    }

    fn artifact(value: &str, system: &str, version: &str) -> ArtifactRef {
        ArtifactRef {
            content_hash: hash(value),
            media_type: "application/json".into(),
            dialect: DialectRef {
                system: system.into(),
                version: version.into(),
                constraints: vec![],
            },
            package_closure: vec![],
        }
    }

    #[test]
    fn verified_state_is_explicit_not_inferred_from_validator_count() {
        let certificate = TransformationCertificate {
            certificate_schema: "commandf.certificate/0".into(),
            run_id: "run-1".into(),
            source: artifact("a", "hl7-fhir", "4.0.1"),
            target: artifact("b", "hl7-fhir", "5.0.0"),
            mappings: vec![],
            terminology: vec![],
            validations: vec![],
            semantic_checks: vec![],
            losses: vec![],
            evidence_root: None,
            verification_policy: "commandf.bootstrap/strict".into(),
            state: VerificationState::Pending,
            environment: BTreeMap::new(),
        };

        assert!(!certificate.policy_verified());
    }

    #[test]
    fn irreversible_loss_is_detected() {
        use commandf_csir::{LossEvent, LossKind, LossSeverity, SourcePointer};

        let certificate = TransformationCertificate {
            certificate_schema: "commandf.certificate/0".into(),
            run_id: "run-2".into(),
            source: artifact("a", "openehr", "1.1.0"),
            target: artifact("b", "omop-cdm", "5.4"),
            mappings: vec![],
            terminology: vec![],
            validations: vec![],
            semantic_checks: vec![],
            losses: vec![LossEvent {
                id: "loss-1".into(),
                kind: LossKind::UnsupportedTargetFeature,
                severity: LossSeverity::Significant,
                recoverability: Recoverability::Irreversible,
                source: SourcePointer {
                    artifact: hash("source"),
                    path: "source/path".into(),
                    fragment_hash: None,
                },
                target_path: None,
                rule_id: "rule-1".into(),
                explanation: "target model has no equivalent feature".into(),
                evidence: BTreeMap::new(),
            }],
            evidence_root: None,
            verification_policy: "commandf.bootstrap/strict".into(),
            state: VerificationState::VerifiedWithWarnings,
            environment: BTreeMap::new(),
        };

        assert!(certificate.has_irreversible_loss());
    }
}
