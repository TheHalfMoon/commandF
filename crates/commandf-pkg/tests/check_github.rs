use commandf_pkg::{
    check_report_to_github_annotations_bytes, evaluate_compatibility_policy, CheckDirection,
    CheckError, CheckFailOn, CheckPolicy, CompatibilityDirection, CompatibilityFinding,
    CompatibilityReport, CompatibilitySeverity, PackageEvidence, ResourceKey, ResourceKeyKind,
    StructuralChangeKind,
};

fn finding(
    rule_id: &str,
    severity: CompatibilitySeverity,
    direction: CompatibilityDirection,
) -> CompatibilityFinding {
    CompatibilityFinding {
        rule_id: rule_id.to_owned(),
        severity,
        direction,
        source_kind: StructuralChangeKind::ElementFieldChanged,
        message: format!("{rule_id} synthetic finding."),
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: "http://example.org/StructureDefinition/example".to_owned(),
        },
        before_filename: Some("StructureDefinition-example.json".to_owned()),
        after_filename: Some("StructureDefinition-example.json".to_owned()),
        view: None,
        element_id: Some("Observation.status".to_owned()),
        field: Some("min".to_owned()),
        before: None,
        after: None,
    }
}

fn compatibility(findings: Vec<CompatibilityFinding>) -> CompatibilityReport {
    CompatibilityReport {
        schema: CompatibilityReport::SCHEMA_V1,
        ruleset: CompatibilityReport::RULESET_V1.to_owned(),
        package_name: "example.package".to_owned(),
        before: PackageEvidence {
            version: "1.0.0".to_owned(),
            archive_sha256: "a".repeat(64),
        },
        after: PackageEvidence {
            version: "1.1.0".to_owned(),
            archive_sha256: "b".repeat(64),
        },
        findings,
    }
}

#[test]
fn severity_maps_to_github_levels_without_fake_locations() {
    let compatibility = compatibility(vec![
        finding(
            "CF04-BREAK-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
        ),
        finding(
            "CF04-RISK-001",
            CompatibilitySeverity::Risky,
            CompatibilityDirection::Consumer,
        ),
        finding(
            "CF04-ADD-001",
            CompatibilitySeverity::Additive,
            CompatibilityDirection::Producer,
        ),
    ]);
    let report = evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    let text =
        String::from_utf8(check_report_to_github_annotations_bytes(&report).unwrap()).unwrap();
    let lines = text.lines().collect::<Vec<_>>();

    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("::error title=commandF CF04-BREAK-001::"));
    assert!(lines[1].starts_with("::warning title=commandF CF04-RISK-001::"));
    assert!(lines[2].starts_with("::notice title=commandF CF04-ADD-001::"));
    for line in lines {
        assert!(!line.contains(" file="));
        assert!(!line.contains(",file="));
        assert!(!line.contains("line="));
        assert!(line.contains("artifact-level finding; source mapping deferred to CF-09"));
    }
}

#[test]
fn direction_selection_matches_cf05_and_fail_on_does_not_hide_findings() {
    let compatibility = compatibility(vec![
        finding(
            "CF04-PROD-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
        ),
        finding(
            "CF04-CONS-001",
            CompatibilitySeverity::Risky,
            CompatibilityDirection::Consumer,
        ),
    ]);

    let breaking = evaluate_compatibility_policy(
        &compatibility,
        CheckPolicy {
            direction: CheckDirection::Consumer,
            fail_on: CheckFailOn::Breaking,
        },
    )
    .unwrap();
    let none = evaluate_compatibility_policy(
        &compatibility,
        CheckPolicy {
            direction: CheckDirection::Consumer,
            fail_on: CheckFailOn::None,
        },
    )
    .unwrap();

    assert!(breaking.decision.passed);
    assert!(none.decision.passed);
    let first = check_report_to_github_annotations_bytes(&breaking).unwrap();
    let second = check_report_to_github_annotations_bytes(&none).unwrap();
    assert_eq!(first, second);
    let text = String::from_utf8(first).unwrap();
    assert!(text.contains("CF04-CONS-001"));
    assert!(!text.contains("CF04-PROD-001"));
}

#[test]
fn policy_failed_report_still_renders_before_action_exit() {
    let compatibility = compatibility(vec![finding(
        "CF04-BREAK-001",
        CompatibilitySeverity::Breaking,
        CompatibilityDirection::Producer,
    )]);
    let report = evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    assert!(!report.decision.passed);
    let bytes = check_report_to_github_annotations_bytes(&report).unwrap();
    assert!(String::from_utf8(bytes)
        .unwrap()
        .starts_with("::error title="));
}

#[test]
fn workflow_command_control_characters_are_escaped() {
    let mut injected = finding(
        "CF04:INJECT,001",
        CompatibilitySeverity::Breaking,
        CompatibilityDirection::Producer,
    );
    injected.message = "percent%\n::error title=pwned::second\rline".to_owned();
    injected.resource.value = "http://example.org/a,b:c%".to_owned();
    let compatibility = compatibility(vec![injected]);
    let report = evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    let text =
        String::from_utf8(check_report_to_github_annotations_bytes(&report).unwrap()).unwrap();

    assert_eq!(text.lines().count(), 1);
    assert!(text.starts_with("::error title=commandF CF04%3AINJECT%2C001::"));
    assert!(text.contains("percent%25%0A::error title=pwned::second%0Dline"));
    assert!(!text.contains("\n::error title=pwned"));
}

#[test]
fn annotation_caps_are_bounded_and_overflow_is_disclosed() {
    let mut findings = Vec::new();
    for index in 0..11 {
        findings.push(finding(
            &format!("CF04-E-{index:02}"),
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
        ));
        findings.push(finding(
            &format!("CF04-W-{index:02}"),
            CompatibilitySeverity::Risky,
            CompatibilityDirection::Producer,
        ));
        findings.push(finding(
            &format!("CF04-N-{index:02}"),
            CompatibilitySeverity::Additive,
            CompatibilityDirection::Producer,
        ));
    }
    let compatibility = compatibility(findings);
    let report = evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    let text =
        String::from_utf8(check_report_to_github_annotations_bytes(&report).unwrap()).unwrap();
    let lines = text.lines().collect::<Vec<_>>();

    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("::error "))
            .count(),
        10
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("::warning "))
            .count(),
        10
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("::notice "))
            .count(),
        10
    );
    assert_eq!(lines.len(), 30);
    assert!(lines
        .last()
        .unwrap()
        .contains("omitted errors=1, warnings=1, notices=2"));
    assert_eq!(report.decision.total_findings, 33);
    assert_eq!(report.decision.selected_findings, 33);
}

#[test]
fn repeated_rendering_is_byte_identical_and_empty_report_is_empty() {
    let compatibility = compatibility(Vec::new());
    let first = evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    let second = evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    assert_eq!(
        check_report_to_github_annotations_bytes(&first).unwrap(),
        check_report_to_github_annotations_bytes(&second).unwrap()
    );
    assert!(check_report_to_github_annotations_bytes(&first)
        .unwrap()
        .is_empty());
}

#[test]
fn inconsistent_persisted_decision_fails_closed() {
    let compatibility = compatibility(Vec::new());
    let mut report = evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    report.decision.passed = false;
    report.decision.blocking_findings = 1;
    assert!(matches!(
        check_report_to_github_annotations_bytes(&report),
        Err(CheckError::InconsistentCheckDecision)
    ));
}

#[test]
fn unsupported_check_and_compatibility_authority_fail_closed() {
    let compatibility = compatibility(Vec::new());
    let mut wrong_check =
        evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    wrong_check.schema = 999;
    assert!(matches!(
        check_report_to_github_annotations_bytes(&wrong_check),
        Err(CheckError::UnsupportedCheckSchema { .. })
    ));

    let mut wrong_compatibility =
        evaluate_compatibility_policy(&compatibility, CheckPolicy::default()).unwrap();
    wrong_compatibility.compatibility.ruleset = "future-rules".to_owned();
    assert!(matches!(
        check_report_to_github_annotations_bytes(&wrong_compatibility),
        Err(CheckError::UnsupportedCompatibilityRuleset { .. })
    ));
}
