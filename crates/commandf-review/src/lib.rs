use commandf_csir::ContentHash;
use commandf_findings::{FindingSet, FindingState, Severity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityDimension {
    Structural,
    Fhir,
    Profile,
    Terminology,
    Semantic,
    RoundTrip,
    Consumer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompatibilityState {
    Compatible,
    Incompatible,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompatibilityCheck {
    pub dimension: CompatibilityDimension,
    pub state: CompatibilityState,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BreakingReport {
    pub report_schema: String,
    pub base: ContentHash,
    pub candidate: ContentHash,
    #[serde(default)]
    pub checks: Vec<CompatibilityCheck>,
}

impl BreakingReport {
    pub fn state_for(&self, dimension: CompatibilityDimension) -> Option<CompatibilityState> {
        self.checks
            .iter()
            .find(|check| check.dimension == dimension)
            .map(|check| check.state)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QualityGatePolicy {
    pub id: String,
    pub max_open_blockers: u32,
    pub max_open_high: u32,
    #[serde(default)]
    pub required_compatible_dimensions: Vec<CompatibilityDimension>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateResult {
    pub policy_id: String,
    pub status: GateStatus,
    #[serde(default)]
    pub violations: Vec<String>,
}

pub fn evaluate_quality_gate(
    policy: &QualityGatePolicy,
    findings: &FindingSet,
    breaking: &BreakingReport,
) -> GateResult {
    let blockers = findings.open_blocker_count() as u32;
    let high = findings
        .findings
        .iter()
        .filter(|finding| finding.state == FindingState::Open && finding.severity == Severity::High)
        .count() as u32;
    let mut violations = Vec::new();

    if blockers > policy.max_open_blockers {
        violations.push("open blocker limit exceeded".into());
    }
    if high > policy.max_open_high {
        violations.push("open high-severity limit exceeded".into());
    }
    for dimension in &policy.required_compatible_dimensions {
        if breaking.state_for(*dimension) != Some(CompatibilityState::Compatible) {
            violations.push(format!(
                "required compatibility check {dimension:?} did not pass"
            ));
        }
    }

    GateResult {
        policy_id: policy.id.clone(),
        status: if violations.is_empty() {
            GateStatus::Pass
        } else {
            GateStatus::Fail
        },
        violations,
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
    fn unknown_required_semantics_fails_gate() {
        let findings = FindingSet {
            finding_set_schema: "commandf.findings/0".into(),
            findings: vec![],
        };
        let report = BreakingReport {
            report_schema: "commandf.breaking/0".into(),
            base: hash("a"),
            candidate: hash("b"),
            checks: vec![CompatibilityCheck {
                dimension: CompatibilityDimension::Semantic,
                state: CompatibilityState::Unknown,
                summary: "not evaluated".into(),
            }],
        };
        let policy = QualityGatePolicy {
            id: "strict".into(),
            max_open_blockers: 0,
            max_open_high: 0,
            required_compatible_dimensions: vec![CompatibilityDimension::Semantic],
        };
        assert_eq!(
            evaluate_quality_gate(&policy, &findings, &report).status,
            GateStatus::Fail
        );
    }
}
