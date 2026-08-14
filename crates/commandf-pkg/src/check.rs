use crate::{
    CheckDecision, CheckDirection, CheckError, CheckFailOn, CheckPolicy, CheckReport,
    CompatibilityDirection, CompatibilityReport, CompatibilitySeverity,
};

pub fn evaluate_compatibility_policy(
    compatibility: &CompatibilityReport,
    policy: CheckPolicy,
) -> Result<CheckReport, CheckError> {
    validate_compatibility_report(compatibility)?;

    let total_findings = compatibility.findings.len();
    let selected = compatibility
        .findings
        .iter()
        .filter(|finding| direction_selected(policy.direction, finding.direction))
        .collect::<Vec<_>>();

    let breaking_findings = selected
        .iter()
        .filter(|finding| finding.severity == CompatibilitySeverity::Breaking)
        .count();
    let risky_findings = selected
        .iter()
        .filter(|finding| finding.severity == CompatibilitySeverity::Risky)
        .count();
    let additive_findings = selected
        .iter()
        .filter(|finding| finding.severity == CompatibilitySeverity::Additive)
        .count();
    let blocking_findings = selected
        .iter()
        .filter(|finding| severity_blocks(policy.fail_on, finding.severity))
        .count();

    Ok(CheckReport {
        schema: CheckReport::SCHEMA_V1,
        policy,
        decision: CheckDecision {
            passed: blocking_findings == 0,
            total_findings,
            selected_findings: selected.len(),
            breaking_findings,
            risky_findings,
            additive_findings,
            blocking_findings,
        },
        compatibility: compatibility.clone(),
    })
}

pub(crate) fn validate_compatibility_report(
    compatibility: &CompatibilityReport,
) -> Result<(), CheckError> {
    if compatibility.schema != CompatibilityReport::SCHEMA_V1 {
        return Err(CheckError::UnsupportedCompatibilitySchema {
            found: compatibility.schema,
            expected: CompatibilityReport::SCHEMA_V1,
        });
    }
    if compatibility.ruleset != CompatibilityReport::RULESET_V1 {
        return Err(CheckError::UnsupportedCompatibilityRuleset {
            found: compatibility.ruleset.clone(),
            expected: CompatibilityReport::RULESET_V1.to_owned(),
        });
    }
    Ok(())
}

pub(crate) fn direction_selected(
    policy: CheckDirection,
    finding: CompatibilityDirection,
) -> bool {
    match policy {
        CheckDirection::Both => true,
        CheckDirection::Producer => finding == CompatibilityDirection::Producer,
        CheckDirection::Consumer => finding == CompatibilityDirection::Consumer,
    }
}

fn severity_blocks(policy: CheckFailOn, severity: CompatibilitySeverity) -> bool {
    match policy {
        CheckFailOn::Breaking => severity == CompatibilitySeverity::Breaking,
        CheckFailOn::Risky => matches!(
            severity,
            CompatibilitySeverity::Breaking | CompatibilitySeverity::Risky
        ),
        CheckFailOn::None => false,
    }
}
