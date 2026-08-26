use commandf_pkg::{
    evaluate_compatibility_policy, evaluate_quality_gate, finding_fingerprint_v1,
    validate_quality_gate_report, CheckDirection, CheckFailOn, CheckPolicy, CompatibilityDirection,
    CompatibilityFinding, CompatibilityReport, CompatibilitySeverity, ElementView, FindingFingerprint,
    GateSuppression, GateSuppressions, PackageEvidence, QualityGateDisposition, QualityGateError,
    ResourceKey, ResourceKeyKind, StructuralChangeKind, MAX_GATE_SUPPRESSION_RATIONALE_CHARS,
};
use serde_json::{json, Value};

fn finding(
    rule_id: &str,
    severity: CompatibilitySeverity,
    direction: CompatibilityDirection,
    before: Value,
    after: Value,
) -> CompatibilityFinding {
    CompatibilityFinding {
        rule_id: rule_id.to_owned(),
        severity,
        direction,
        source_kind: StructuralChangeKind::ElementFieldChanged,
        message: format!("{rule_id} finding"),
        resource: ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value: "http://example.org/StructureDefinition/example".to_owned(),
        },
        before_filename: Some("StructureDefinition-example.json".to_owned()),
        after_filename: Some("StructureDefinition-example.json".to_owned()),
        view: Some(ElementView::Snapshot),
        element_id: Some("Observation.status".to_owned()),
        field: Some("min".to_owned()),
        before: Some(before),
        after: Some(after),
    }
}

fn compatibility(findings: Vec<CompatibilityFinding>, before_version: &str, after_version: &str) -> CompatibilityReport {
    CompatibilityReport {
        schema: CompatibilityReport::SCHEMA_V1,
        ruleset: CompatibilityReport::RULESET_V1.to_owned(),
        package_name: "example.package".to_owned(),
        before: PackageEvidence {
            version: before_version.to_owned(),
            archive_sha256: "a".repeat(64),
        },
        after: PackageEvidence {
            version: after_version.to_owned(),
            archive_sha256: "b".repeat(64),
        },
        findings,
    }
}

fn check(
    findings: Vec<CompatibilityFinding>,
    before_version: &str,
    after_version: &str,
    policy: CheckPolicy,
) -> commandf_pkg::CheckReport {
    evaluate_compatibility_policy(
        &compatibility(findings, before_version, after_version),
        policy,
    )
    .unwrap()
}

fn default_policy() -> CheckPolicy {
    CheckPolicy {
        direction: CheckDirection::Both,
        fail_on: CheckFailOn::Breaking,
    }
}

fn breaking() -> CompatibilityFinding {
    finding(
        "CF04-TEST-001",
        CompatibilitySeverity::Breaking,
        CompatibilityDirection::Producer,
        json!(0),
        json!(1),
    )
}

#[test]
fn no_baseline_selected_breaking_finding_is_new_and_blocks() {
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let report = evaluate_quality_gate(&current, None, None).unwrap();

    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].disposition, QualityGateDisposition::New);
    assert!(report.findings[0].matched_suppression.is_none());
    assert_eq!(report.decision.total_findings, 1);
    assert_eq!(report.decision.selected_findings, 1);
    assert_eq!(report.decision.new_findings, 1);
    assert_eq!(report.decision.new_selected_breaking_findings, 1);
    assert_eq!(report.decision.blocking_findings, 1);
    assert!(!report.decision.passed);
    validate_quality_gate_report(&report).unwrap();
}

#[test]
fn baseline_membership_is_non_blocking_across_package_versions() {
    let baseline = check(vec![breaking()], "0.8.0", "0.9.0", default_policy());
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let report = evaluate_quality_gate(&current, Some(&baseline), None).unwrap();

    assert_eq!(report.findings[0].disposition, QualityGateDisposition::Baseline);
    assert_eq!(report.decision.baseline_findings, 1);
    assert_eq!(report.decision.blocking_findings, 0);
    assert!(report.decision.passed);

    let evidence = report.baseline.as_ref().unwrap();
    assert_eq!(evidence.before.version, "0.8.0");
    assert_eq!(evidence.after.version, "0.9.0");
    assert_eq!(evidence.finding_count, 1);
    assert_eq!(evidence.fingerprints, vec![report.findings[0].fingerprint.clone()]);
    validate_quality_gate_report(&report).unwrap();
}

#[test]
fn exact_suppression_precedes_baseline_and_unused_suppressions_are_retained() {
    let baseline = check(vec![breaking()], "0.8.0", "0.9.0", default_policy());
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let fingerprint = finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &breaking()).unwrap();
    let unused = FindingFingerprint {
        schema: FindingFingerprint::SCHEMA_V1,
        digest: format!("sha256:{}", "c".repeat(64)),
    };
    let suppressions = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![
            GateSuppression {
                finding_fingerprint: unused.clone(),
                rationale: "Stale historical waiver".to_owned(),
                reference: None,
            },
            GateSuppression {
                finding_fingerprint: fingerprint.clone(),
                rationale: "Accepted interoperability exception".to_owned(),
                reference: Some("INT-123".to_owned()),
            },
        ],
    };

    let report = evaluate_quality_gate(&current, Some(&baseline), Some(&suppressions)).unwrap();
    assert_eq!(report.findings[0].disposition, QualityGateDisposition::Suppressed);
    assert_eq!(
        report.findings[0]
            .matched_suppression
            .as_ref()
            .unwrap()
            .rationale,
        "Accepted interoperability exception"
    );
    assert_eq!(report.decision.suppressed_findings, 1);
    assert_eq!(report.decision.baseline_findings, 0);
    assert_eq!(report.decision.blocking_findings, 0);
    assert!(report.decision.passed);
    assert_eq!(report.unused_suppressions, vec![unused]);
    validate_quality_gate_report(&report).unwrap();
}

#[test]
fn direction_and_threshold_semantics_match_cf05_for_new_findings() {
    let findings = vec![
        finding(
            "CF04-BREAK",
            CompatibilitySeverity::Breaking,
            CompatibilityDirection::Producer,
            json!(0),
            json!(1),
        ),
        finding(
            "CF04-RISK",
            CompatibilitySeverity::Risky,
            CompatibilityDirection::Consumer,
            json!("a"),
            json!("b"),
        ),
    ];

    let consumer_breaking = check(
        findings.clone(),
        "1.0.0",
        "1.1.0",
        CheckPolicy {
            direction: CheckDirection::Consumer,
            fail_on: CheckFailOn::Breaking,
        },
    );
    let gate = evaluate_quality_gate(&consumer_breaking, None, None).unwrap();
    assert_eq!(gate.decision.selected_findings, consumer_breaking.decision.selected_findings);
    assert_eq!(gate.decision.blocking_findings, 0);
    assert!(gate.decision.passed);

    let consumer_risky = check(
        findings.clone(),
        "1.0.0",
        "1.1.0",
        CheckPolicy {
            direction: CheckDirection::Consumer,
            fail_on: CheckFailOn::Risky,
        },
    );
    let gate = evaluate_quality_gate(&consumer_risky, None, None).unwrap();
    assert_eq!(gate.decision.blocking_findings, consumer_risky.decision.blocking_findings);
    assert_eq!(gate.decision.blocking_findings, 1);

    let none = check(
        findings,
        "1.0.0",
        "1.1.0",
        CheckPolicy {
            direction: CheckDirection::Both,
            fail_on: CheckFailOn::None,
        },
    );
    let gate = evaluate_quality_gate(&none, None, None).unwrap();
    assert_eq!(gate.decision.blocking_findings, 0);
    assert!(gate.decision.passed);
}

#[test]
fn fingerprint_canonicalizes_nested_object_keys_but_preserves_array_order() {
    let before_a: Value = serde_json::from_str(r#"{"z":1,"a":{"y":2,"x":3}}"#).unwrap();
    let before_b: Value = serde_json::from_str(r#"{"a":{"x":3,"y":2},"z":1}"#).unwrap();
    let after_a: Value = serde_json::from_str(r#"{"items":[1,2,3]}"#).unwrap();
    let after_b: Value = serde_json::from_str(r#"{"items":[3,2,1]}"#).unwrap();

    let left = finding(
        "CF04-CANON",
        CompatibilitySeverity::Breaking,
        CompatibilityDirection::Producer,
        before_a,
        after_a.clone(),
    );
    let right = finding(
        "CF04-CANON",
        CompatibilitySeverity::Breaking,
        CompatibilityDirection::Producer,
        before_b,
        after_a,
    );
    assert_eq!(
        finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &left).unwrap(),
        finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &right).unwrap()
    );

    let array_changed = finding(
        "CF04-CANON",
        CompatibilitySeverity::Breaking,
        CompatibilityDirection::Producer,
        serde_json::from_str(r#"{"a":{"x":3,"y":2},"z":1}"#).unwrap(),
        after_b,
    );
    assert_ne!(
        finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &left).unwrap(),
        finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &array_changed).unwrap()
    );
}

#[test]
fn message_only_change_preserves_fingerprint_but_semantic_change_does_not() {
    let original = breaking();
    let mut wording = original.clone();
    wording.message = "different human wording".to_owned();
    assert_eq!(
        finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &original).unwrap(),
        finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &wording).unwrap()
    );

    let mut severity = original.clone();
    severity.severity = CompatibilitySeverity::Risky;
    assert_ne!(
        finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &original).unwrap(),
        finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &severity).unwrap()
    );
}

#[test]
fn baseline_canonical_digest_is_invariant_to_nested_object_key_order() {
    let left = finding(
        "CF04-CANON",
        CompatibilitySeverity::Breaking,
        CompatibilityDirection::Producer,
        serde_json::from_str(r#"{"z":1,"a":{"y":2,"x":3}}"#).unwrap(),
        json!(1),
    );
    let right = finding(
        "CF04-CANON",
        CompatibilitySeverity::Breaking,
        CompatibilityDirection::Producer,
        serde_json::from_str(r#"{"a":{"x":3,"y":2},"z":1}"#).unwrap(),
        json!(1),
    );
    let baseline_left = check(vec![left.clone()], "0.8.0", "0.9.0", default_policy());
    let baseline_right = check(vec![right.clone()], "0.8.0", "0.9.0", default_policy());
    let current_left = check(vec![left], "1.0.0", "1.1.0", default_policy());
    let current_right = check(vec![right], "1.0.0", "1.1.0", default_policy());

    let report_left = evaluate_quality_gate(&current_left, Some(&baseline_left), None).unwrap();
    let report_right = evaluate_quality_gate(&current_right, Some(&baseline_right), None).unwrap();
    assert_eq!(
        report_left.baseline.unwrap().canonical_sha256,
        report_right.baseline.unwrap().canonical_sha256
    );
}

#[test]
fn suppression_order_is_canonical_and_report_bytes_are_repeatable() {
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let current_fp = finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &breaking()).unwrap();
    let unused_fp = FindingFingerprint {
        schema: 1,
        digest: format!("sha256:{}", "d".repeat(64)),
    };
    let a = GateSuppression {
        finding_fingerprint: current_fp,
        rationale: "accepted".to_owned(),
        reference: None,
    };
    let b = GateSuppression {
        finding_fingerprint: unused_fp,
        rationale: "unused".to_owned(),
        reference: Some("REF".to_owned()),
    };
    let first = GateSuppressions {
        schema: 1,
        suppressions: vec![a.clone(), b.clone()],
    };
    let second = GateSuppressions {
        schema: 1,
        suppressions: vec![b, a],
    };

    let report_a = evaluate_quality_gate(&current, None, Some(&first)).unwrap();
    let report_b = evaluate_quality_gate(&current, None, Some(&second)).unwrap();
    assert_eq!(
        report_a.suppression_evidence.as_ref().unwrap().canonical_sha256,
        report_b.suppression_evidence.as_ref().unwrap().canonical_sha256
    );
    assert_eq!(report_a.to_json_bytes().unwrap(), report_b.to_json_bytes().unwrap());
    let repeated = evaluate_quality_gate(&current, None, Some(&first)).unwrap();
    assert_eq!(report_a.to_json_bytes().unwrap(), repeated.to_json_bytes().unwrap());
}

#[test]
fn invalid_or_ambiguous_suppression_state_fails_closed() {
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let fingerprint = finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &breaking()).unwrap();

    let unsupported = GateSuppressions {
        schema: 1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: FindingFingerprint {
                schema: 2,
                digest: fingerprint.digest.clone(),
            },
            rationale: "reason".to_owned(),
            reference: None,
        }],
    };
    assert!(matches!(
        evaluate_quality_gate(&current, None, Some(&unsupported)),
        Err(QualityGateError::UnsupportedFingerprintSchema { found: 2, expected: 1 })
    ));

    let malformed = GateSuppressions {
        schema: 1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: FindingFingerprint {
                schema: 1,
                digest: "sha256:ABC".to_owned(),
            },
            rationale: "reason".to_owned(),
            reference: None,
        }],
    };
    assert!(matches!(
        evaluate_quality_gate(&current, None, Some(&malformed)),
        Err(QualityGateError::MalformedSha256Identity { .. })
    ));

    let duplicate = GateSuppression {
        finding_fingerprint: fingerprint,
        rationale: "reason".to_owned(),
        reference: None,
    };
    let duplicates = GateSuppressions {
        schema: 1,
        suppressions: vec![duplicate.clone(), duplicate],
    };
    assert!(matches!(
        evaluate_quality_gate(&current, None, Some(&duplicates)),
        Err(QualityGateError::DuplicateSuppressionFingerprint { .. })
    ));

    let oversized = GateSuppressions {
        schema: 1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: finding_fingerprint_v1(
                CompatibilityReport::RULESET_V1,
                &breaking(),
            )
            .unwrap(),
            rationale: "x".repeat(MAX_GATE_SUPPRESSION_RATIONALE_CHARS + 1),
            reference: None,
        }],
    };
    assert!(matches!(
        evaluate_quality_gate(&current, None, Some(&oversized)),
        Err(QualityGateError::SuppressionStringTooLong { field: "rationale", .. })
    ));
}

#[test]
fn duplicate_baseline_or_current_fingerprints_fail_closed() {
    let duplicated = vec![breaking(), breaking()];
    let baseline = check(duplicated.clone(), "0.8.0", "0.9.0", default_policy());
    let current_single = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    assert!(matches!(
        evaluate_quality_gate(&current_single, Some(&baseline), None),
        Err(QualityGateError::DuplicateBaselineFingerprint { .. })
    ));

    let current_duplicate = check(duplicated, "1.0.0", "1.1.0", default_policy());
    assert!(matches!(
        evaluate_quality_gate(&current_duplicate, None, None),
        Err(QualityGateError::DuplicateCurrentFingerprint { .. })
    ));
}

#[test]
fn baseline_package_mismatch_fails_closed() {
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let mut baseline = check(vec![breaking()], "0.8.0", "0.9.0", default_policy());
    baseline.compatibility.package_name = "other.package".to_owned();
    assert!(matches!(
        evaluate_quality_gate(&current, Some(&baseline), None),
        Err(QualityGateError::BaselinePackageMismatch { .. })
    ));
}

#[test]
fn persisted_report_validation_rejects_forged_baseline_disposition() {
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let report = evaluate_quality_gate(&current, None, None).unwrap();
    validate_quality_gate_report(&report).unwrap();

    let mut forged = report;
    forged.findings[0].disposition = QualityGateDisposition::Baseline;
    forged.decision.passed = true;
    forged.decision.new_findings = 0;
    forged.decision.baseline_findings = 1;
    forged.decision.new_selected_breaking_findings = 0;
    forged.decision.blocking_findings = 0;
    assert!(matches!(
        validate_quality_gate_report(&forged),
        Err(QualityGateError::InconsistentReport { .. })
    ));
}

#[test]
fn persisted_report_validation_rejects_altered_fingerprint_and_decision() {
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let report = evaluate_quality_gate(&current, None, None).unwrap();

    let mut fingerprint_tamper = report.clone();
    fingerprint_tamper.findings[0].fingerprint.digest = format!("sha256:{}", "0".repeat(64));
    assert!(matches!(
        validate_quality_gate_report(&fingerprint_tamper),
        Err(QualityGateError::InconsistentReport { .. })
    ));

    let mut decision_tamper = report;
    decision_tamper.decision.blocking_findings = 0;
    decision_tamper.decision.passed = true;
    assert!(matches!(
        validate_quality_gate_report(&decision_tamper),
        Err(QualityGateError::InconsistentReport { .. })
    ));
}

#[test]
fn persisted_report_validation_rejects_missing_baseline_membership() {
    let baseline = check(vec![breaking()], "0.8.0", "0.9.0", default_policy());
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let mut report = evaluate_quality_gate(&current, Some(&baseline), None).unwrap();
    validate_quality_gate_report(&report).unwrap();

    let evidence = report.baseline.as_mut().unwrap();
    evidence.fingerprints.clear();
    evidence.finding_count = 0;
    assert!(matches!(
        validate_quality_gate_report(&report),
        Err(QualityGateError::InconsistentReport { .. })
    ));
}

#[test]
fn persisted_report_validation_rejects_suppression_metadata_tampering() {
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let fingerprint = finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &breaking()).unwrap();
    let suppressions = GateSuppressions {
        schema: 1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: fingerprint,
            rationale: "approved".to_owned(),
            reference: Some("INT-123".to_owned()),
        }],
    };
    let report = evaluate_quality_gate(&current, None, Some(&suppressions)).unwrap();
    validate_quality_gate_report(&report).unwrap();

    let mut matched_tamper = report.clone();
    matched_tamper.findings[0]
        .matched_suppression
        .as_mut()
        .unwrap()
        .rationale = "forged".to_owned();
    assert!(matches!(
        validate_quality_gate_report(&matched_tamper),
        Err(QualityGateError::InconsistentReport { .. })
    ));

    let mut evidence_tamper = report;
    evidence_tamper
        .suppression_evidence
        .as_mut()
        .unwrap()
        .suppressions[0]
        .rationale = "forged".to_owned();
    assert!(matches!(
        validate_quality_gate_report(&evidence_tamper),
        Err(QualityGateError::InconsistentReport { .. })
    ));
}

#[test]
fn persisted_report_rejects_unknown_fingerprint_version() {
    let current = check(vec![breaking()], "1.0.0", "1.1.0", default_policy());
    let mut report = evaluate_quality_gate(&current, None, None).unwrap();
    report.findings[0].fingerprint.schema = 2;
    assert!(matches!(
        validate_quality_gate_report(&report),
        Err(QualityGateError::UnsupportedFingerprintSchema { found: 2, expected: 1 })
    ));
}
