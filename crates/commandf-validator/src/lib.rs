//! Implementation-neutral validator contract for commandF.
//!
//! Validators are independently identified tools used to produce conformance
//! or compatibility evidence. No single FHIR/openEHR/OMOP implementation is
//! treated as universal truth.

use commandf_csir::{ContentHash, DialectRef};
use commandf_findings::{Finding, Severity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationCapability {
    Syntax,
    Profile,
    Terminology,
    SearchBehavior,
    RoundTrip,
    DataQuality,
    Compatibility,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatorDescriptor {
    pub id: String,
    pub version: String,
    #[serde(default)]
    pub build_hash: Option<ContentHash>,
    #[serde(default)]
    pub capabilities: Vec<ValidationCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationTarget {
    pub artifact: ContentHash,
    pub dialect: DialectRef,
    #[serde(default)]
    pub contracts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationRequest {
    pub target: ValidationTarget,
    pub capability: ValidationCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationOutcome {
    Pass,
    Warning,
    Fail,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub validator: ValidatorDescriptor,
    pub outcome: ValidationOutcome,
    #[serde(default)]
    pub findings: Vec<Finding>,
    #[serde(default)]
    pub evidence_hash: Option<ContentHash>,
}

impl ValidationResult {
    pub fn has_open_blocker(&self) -> bool {
        self.findings.iter().any(|finding| finding.is_blocker())
    }

    pub fn maximum_finding_severity(&self) -> Option<Severity> {
        self.findings.iter().map(|finding| finding.severity).max()
    }
}

pub trait Validator {
    fn descriptor(&self) -> ValidatorDescriptor;
    fn validate(&self, request: &ValidationRequest) -> ValidationResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PassingValidator;

    impl Validator for PassingValidator {
        fn descriptor(&self) -> ValidatorDescriptor {
            ValidatorDescriptor {
                id: "test-validator".into(),
                version: "1".into(),
                build_hash: None,
                capabilities: vec![ValidationCapability::Syntax],
            }
        }

        fn validate(&self, _request: &ValidationRequest) -> ValidationResult {
            ValidationResult {
                validator: self.descriptor(),
                outcome: ValidationOutcome::Pass,
                findings: vec![],
                evidence_hash: None,
            }
        }
    }

    #[test]
    fn validator_identity_is_preserved_in_result() {
        let validator = PassingValidator;
        let request = ValidationRequest {
            target: ValidationTarget {
                artifact: ContentHash {
                    algorithm: "sha256".into(),
                    value: "abc".into(),
                },
                dialect: DialectRef {
                    system: "hl7-fhir".into(),
                    version: "5.0.0".into(),
                    constraints: vec![],
                },
                contracts: vec![],
            },
            capability: ValidationCapability::Syntax,
        };

        let result = validator.validate(&request);
        assert_eq!(result.outcome, ValidationOutcome::Pass);
        assert_eq!(result.validator.id, "test-validator");
    }
}
