use commandf_pkg::{
    evaluate_compatibility_policy, evaluate_quality_gate, finding_fingerprint_v1,
    validate_quality_gate_report, CheckPolicy, CompatibilityDirection, CompatibilityFinding,
    CompatibilityReport, CompatibilitySeverity, ElementView, FindingFingerprint, GateSuppression,
    GateSuppressions, PackageEvidence, QualityGateError, QualityGateReport, ResourceKey,
    ResourceKeyKind, StructuralChangeKind, MAX_GATE_SUPPRESSIONS,
    MAX_GATE_SUPPRESSION_REFERENCE_CHARS,
};
use serde_json::json;

fn breaking() -> CompatibilityFinding {
    CompatibilityFinding {
        rule_id: "CF04-TEST-VALIDATION".to_owned(),
        severity: CompatibilitySeverity::Breaking,
        direction: CompatibilityDirection::Producer,
        source_kind: StructuralChangeKind::ElementFieldChanged,
        message: "validation finding".to_owned(),
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

fn check(before_version: &str, after_version: &str) -> commandf_pkg::CheckReport {
    evaluate_compatibility_policy(
        &CompatibilityReport {
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
            findings: vec![breaking()],
        },
        CheckPolicy::default(),
    )
    .unwrap()
}

fn fingerprint() -> FindingFingerprint {
    finding_fingerprint_v1(CompatibilityReport::RULESET_V1, &breaking()).unwrap()
}

#[test]
fn suppression_schema_and_empty_rationale_fail_closed() {
    let current = check("1.0.0", "1.1.0");
    let unsupported_schema = GateSuppressions {
        schema: 2,
        suppressions: Vec::new(),
    };
    assert!(matches!(
        evaluate_quality_gate(&current, None, Some(&unsupported_schema)),
        Err(QualityGateError::UnsupportedSuppressionSchema {
            found: 2,
            expected: 1
        })
    ));

    let empty_rationale = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: fingerprint(),
            rationale: " \n\t ".to_owned(),
            reference: None,
        }],
    };
    assert!(matches!(
        evaluate_quality_gate(&current, None, Some(&empty_rationale)),
        Err(QualityGateError::EmptySuppressionRationale)
    ));
}

#[test]
fn suppression_entry_and_reference_bounds_fail_closed() {
    let current = check("1.0.0", "1.1.0");
    let suppressions = (0..=MAX_GATE_SUPPRESSIONS)
        .map(|index| GateSuppression {
            finding_fingerprint: FindingFingerprint {
                schema: FindingFingerprint::SCHEMA_V1,
                digest: format!("sha256:{index:064x}"),
            },
            rationale: "bounded".to_owned(),
            reference: None,
        })
        .collect();
    assert!(matches!(
        evaluate_quality_gate(
            &current,
            None,
            Some(&GateSuppressions {
                schema: GateSuppressions::SCHEMA_V1,
                suppressions,
            })
        ),
        Err(QualityGateError::TooManySuppressions {
            found,
            maximum
        }) if found == MAX_GATE_SUPPRESSIONS + 1 && maximum == MAX_GATE_SUPPRESSIONS
    ));

    let oversized_reference = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: fingerprint(),
            rationale: "approved".to_owned(),
            reference: Some("x".repeat(MAX_GATE_SUPPRESSION_REFERENCE_CHARS + 1)),
        }],
    };
    assert!(matches!(
        evaluate_quality_gate(&current, None, Some(&oversized_reference)),
        Err(QualityGateError::SuppressionStringTooLong {
            field: "reference",
            ..
        })
    ));
}

#[test]
fn persisted_report_rejects_report_and_membership_count_tampering() {
    let baseline = check("0.8.0", "0.9.0");
    let current = check("1.0.0", "1.1.0");
    let mut report = evaluate_quality_gate(&current, Some(&baseline), None).unwrap();

    report.schema = 2;
    assert!(matches!(
        validate_quality_gate_report(&report),
        Err(QualityGateError::UnsupportedGateSchema {
            found: 2,
            expected: 1
        })
    ));

    let mut baseline_count = evaluate_quality_gate(&current, Some(&baseline), None).unwrap();
    baseline_count.baseline.as_mut().unwrap().finding_count += 1;
    assert!(matches!(
        validate_quality_gate_report(&baseline_count),
        Err(QualityGateError::InconsistentReport { .. })
    ));

    let suppression = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: fingerprint(),
            rationale: "approved".to_owned(),
            reference: None,
        }],
    };
    let mut suppression_count = evaluate_quality_gate(&current, None, Some(&suppression)).unwrap();
    suppression_count
        .suppression_evidence
        .as_mut()
        .unwrap()
        .entry_count += 1;
    assert!(matches!(
        validate_quality_gate_report(&suppression_count),
        Err(QualityGateError::InconsistentReport { .. })
    ));
}

#[test]
fn persisted_report_decoder_rejects_unknown_disposition_value() {
    let current = check("1.0.0", "1.1.0");
    let report = evaluate_quality_gate(&current, None, None).unwrap();
    let json = String::from_utf8(report.to_json_bytes().unwrap()).unwrap();
    let tampered = json.replacen("\"disposition\": \"new\"", "\"disposition\": \"future\"", 1);

    assert!(QualityGateReport::from_json_slice(tampered.as_bytes()).is_err());
}

#[test]
fn suppression_decoder_rejects_unknown_policy_fields() {
    let payload = json!({
        "schema": 1,
        "suppressions": [{
            "finding_fingerprint": {
                "schema": 1,
                "digest": fingerprint().digest,
                "algorithm": "sha256"
            },
            "rationale": "approved",
            "expires_at": "2099-01-01T00:00:00Z"
        }]
    });
    let bytes = serde_json::to_vec(&payload).unwrap();

    assert!(GateSuppressions::from_json_slice(&bytes).is_err());
}

#[test]
fn persisted_report_decoder_rejects_unknown_gate_fields() {
    let current = check("1.0.0", "1.1.0");
    let report = evaluate_quality_gate(&current, None, None).unwrap();
    let mut value = serde_json::to_value(&report).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("future_policy".to_owned(), json!(true));
    let bytes = serde_json::to_vec(&value).unwrap();

    assert!(QualityGateReport::from_json_slice(&bytes).is_err());
}
