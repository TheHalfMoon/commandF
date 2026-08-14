use commandf_pkg::{
    check_report_to_sarif_bytes, evaluate_compatibility_policy, CheckDirection, CheckError,
    CheckFailOn, CheckPolicy, CompatibilityDirection, CompatibilityFinding, CompatibilityReport,
    CompatibilitySeverity, ElementView, PackageEvidence, ResourceKey, ResourceKeyKind,
    StructuralChangeKind,
};
use serde_json::json;

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
        view: Some(ElementView::Snapshot),
        element_id: Some("Observation.status".to_owned()),
        field: Some("min".to_owned()),
        before: Some(json!(0)),
        after: Some(json!(1)),
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

fn mixed_report() -> CompatibilityReport {
    compatibility(vec![
        finding(
            "CF04-ZZZ-001",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
        ),
        finding(
            "CF04-AAA-001",
            CompatibilitySeverity::Risky,
            CompatibilityDirection::Consumer,
        ),
        finding(
            "CF04-MID-001",
            CompatibilitySeverity::Additive,
            CompatibilityDirection::Producer,
        ),
    ])
}

#[test]
fn default_policy_fails_only_on_selected_breaking_findings() {
    let report = evaluate_compatibility_policy(&mixed_report(), CheckPolicy::default()).unwrap();
    assert!(!report.decision.passed);
    assert_eq!(report.decision.total_findings, 3);
    assert_eq!(report.decision.selected_findings, 3);
    assert_eq!(report.decision.breaking_findings, 1);
    assert_eq!(report.decision.risky_findings, 1);
    assert_eq!(report.decision.additive_findings, 1);
    assert_eq!(report.decision.blocking_findings, 1);
    assert_eq!(report.compatibility, mixed_report());
}

#[test]
fn direction_filtering_precedes_threshold_evaluation() {
    let consumer = evaluate_compatibility_policy(
        &mixed_report(),
        CheckPolicy {
            direction: CheckDirection::Consumer,
            fail_on: CheckFailOn::Breaking,
        },
    )
    .unwrap();
    assert!(consumer.decision.passed);
    assert_eq!(consumer.decision.selected_findings, 1);
    assert_eq!(consumer.decision.risky_findings, 1);
    assert_eq!(consumer.decision.blocking_findings, 0);

    let producer = evaluate_compatibility_policy(
        &mixed_report(),
        CheckPolicy {
            direction: CheckDirection::Producer,
            fail_on: CheckFailOn::Breaking,
        },
    )
    .unwrap();
    assert!(!producer.decision.passed);
    assert_eq!(producer.decision.selected_findings, 2);
    assert_eq!(producer.decision.blocking_findings, 1);
}

#[test]
fn risky_and_none_thresholds_are_distinct() {
    let risky = evaluate_compatibility_policy(
        &mixed_report(),
        CheckPolicy {
            direction: CheckDirection::Consumer,
            fail_on: CheckFailOn::Risky,
        },
    )
    .unwrap();
    assert!(!risky.decision.passed);
    assert_eq!(risky.decision.blocking_findings, 1);

    let none = evaluate_compatibility_policy(
        &mixed_report(),
        CheckPolicy {
            direction: CheckDirection::Both,
            fail_on: CheckFailOn::None,
        },
    )
    .unwrap();
    assert!(none.decision.passed);
    assert_eq!(none.decision.blocking_findings, 0);
}

#[test]
fn unsupported_cf04_authority_fails_closed() {
    let mut wrong_schema = mixed_report();
    wrong_schema.schema = 999;
    assert!(matches!(
        evaluate_compatibility_policy(&wrong_schema, CheckPolicy::default()),
        Err(CheckError::UnsupportedCompatibilitySchema { .. })
    ));

    let mut wrong_ruleset = mixed_report();
    wrong_ruleset.ruleset = "future-rules".to_owned();
    assert!(matches!(
        evaluate_compatibility_policy(&wrong_ruleset, CheckPolicy::default()),
        Err(CheckError::UnsupportedCompatibilityRuleset { .. })
    ));
}

#[test]
fn json_and_sarif_are_byte_deterministic() {
    let check = evaluate_compatibility_policy(&mixed_report(), CheckPolicy::default()).unwrap();
    assert_eq!(check.to_json_bytes().unwrap(), check.to_json_bytes().unwrap());
    assert_eq!(
        check_report_to_sarif_bytes(&check).unwrap(),
        check_report_to_sarif_bytes(&check).unwrap()
    );
}

#[test]
fn sarif_uses_stable_rules_levels_properties_and_no_fake_locations() {
    let check = evaluate_compatibility_policy(&mixed_report(), CheckPolicy::default()).unwrap();
    let bytes = check_report_to_sarif_bytes(&check).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(value["version"], "2.1.0");
    assert_eq!(value["runs"][0]["tool"]["driver"]["name"], "commandF");
    let rules = value["runs"][0]["tool"]["driver"]["rules"]
        .as_array()
        .unwrap();
    let rule_ids = rules
        .iter()
        .map(|rule| rule["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        rule_ids,
        vec!["CF04-AAA-001", "CF04-MID-001", "CF04-ZZZ-001"]
    );

    let results = value["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 3);
    assert_eq!(results[0]["level"], "error");
    assert_eq!(results[1]["level"], "warning");
    assert_eq!(results[2]["level"], "note");
    for result in results {
        assert!(result.get("locations").is_none());
        assert!(result["properties"].get("commandf.resource").is_some());
        assert!(
            result["properties"]
                .get("commandf.compatibilitySeverity")
                .is_some()
        );
        assert!(result["properties"].get("commandf.direction").is_some());
    }
    assert_eq!(
        value["runs"][0]["properties"]["commandf.sourceMapping"],
        "deferred_cf09"
    );
}
