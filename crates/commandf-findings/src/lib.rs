//! Machine-readable review findings for commandF.
//!
//! Findings are evidence-bearing review artifacts. They may be emitted by
//! deterministic rules, validators, compatibility checks, or AI-assisted
//! reviewers, but the producer and evidence remain explicit.

use commandf_csir::{ContentHash, SourcePointer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Note,
    Low,
    Medium,
    High,
    Blocker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    Conformance,
    Semantic,
    Terminology,
    Compatibility,
    Privacy,
    Identity,
    Security,
    Provenance,
    Performance,
    Quality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingState {
    Open,
    Acknowledged,
    Resolved,
    Suppressed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProducerRef {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub build_hash: Option<ContentHash>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub description: String,
    pub artifact: ContentHash,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub finding_schema: String,
    pub id: String,
    pub rule_id: String,
    pub category: FindingCategory,
    pub severity: Severity,
    pub state: FindingState,
    pub title: String,
    pub message: String,
    pub producer: ProducerRef,
    #[serde(default)]
    pub primary_location: Option<SourcePointer>,
    #[serde(default)]
    pub related_locations: Vec<SourcePointer>,
    #[serde(default)]
    pub evidence: Vec<EvidenceRef>,
    #[serde(default)]
    pub remediation: Option<String>,
}

impl Finding {
    pub fn is_blocker(&self) -> bool {
        self.state == FindingState::Open && self.severity == Severity::Blocker
    }

    /// SARIF-compatible level for interchange. commandF retains the richer
    /// native severity separately.
    pub fn sarif_level(&self) -> &'static str {
        match self.severity {
            Severity::Note | Severity::Low => "note",
            Severity::Medium => "warning",
            Severity::High | Severity::Blocker => "error",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingSet {
    pub finding_set_schema: String,
    #[serde(default)]
    pub findings: Vec<Finding>,
}

impl FindingSet {
    pub fn open_blocker_count(&self) -> usize {
        self.findings.iter().filter(|finding| finding.is_blocker()).count()
    }

    pub fn maximum_open_severity(&self) -> Option<Severity> {
        self.findings
            .iter()
            .filter(|finding| finding.state == FindingState::Open)
            .map(|finding| finding.severity)
            .max()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(value: &str) -> ContentHash {
        ContentHash {
            algorithm: "sha256".into(),
            value: value.into(),
        }
    }

    #[test]
    fn blocker_is_explicit_and_sarif_export_is_lossy_but_stable() {
        let finding = Finding {
            finding_schema: "commandf.finding/0".into(),
            id: "finding-1".into(),
            rule_id: "CF-SEM-001".into(),
            category: FindingCategory::Semantic,
            severity: Severity::Blocker,
            state: FindingState::Open,
            title: "silent loss".into(),
            message: "source fact has no target representation or loss event".into(),
            producer: ProducerRef {
                name: "commandf-semantic-verifier".into(),
                version: "0".into(),
                build_hash: None,
            },
            primary_location: None,
            related_locations: vec![],
            evidence: vec![EvidenceRef {
                description: "verification report".into(),
                artifact: hash("abc"),
                path: None,
            }],
            remediation: Some("declare or prevent the loss".into()),
        };

        assert!(finding.is_blocker());
        assert_eq!(finding.sarif_level(), "error");
        let json = serde_json::to_value(&finding).expect("serialize finding");
        assert_eq!(json["severity"], "blocker");
    }
}
