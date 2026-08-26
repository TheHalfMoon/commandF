use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::check::{direction_selected, severity_blocks};
use crate::{
    validate_check_report, CheckReport, CompatibilityFinding, CompatibilitySeverity,
    FindingFingerprint, GateSuppression, GateSuppressions, QualityGateBaselineEvidence,
    QualityGateDecision, QualityGateDisposition, QualityGateError, QualityGateFinding,
    QualityGateReport, QualityGateSuppressionEvidence,
};

pub const MAX_GATE_SUPPRESSIONS: usize = 4_096;
pub const MAX_GATE_SUPPRESSION_RATIONALE_CHARS: usize = 4_096;
pub const MAX_GATE_SUPPRESSION_REFERENCE_CHARS: usize = 4_096;

#[derive(Serialize)]
struct FindingFingerprintKey {
    schema: u32,
    ruleset: String,
    rule_id: String,
    severity: crate::CompatibilitySeverity,
    direction: crate::CompatibilityDirection,
    source_kind: crate::StructuralChangeKind,
    resource: crate::ResourceKey,
    before_filename: Option<String>,
    after_filename: Option<String>,
    view: Option<crate::ElementView>,
    element_id: Option<String>,
    field: Option<String>,
    before: Option<Value>,
    after: Option<Value>,
}

pub fn finding_fingerprint_v1(
    ruleset: &str,
    finding: &CompatibilityFinding,
) -> Result<FindingFingerprint, QualityGateError> {
    let key = FindingFingerprintKey {
        schema: FindingFingerprint::SCHEMA_V1,
        ruleset: ruleset.to_owned(),
        rule_id: finding.rule_id.clone(),
        severity: finding.severity,
        direction: finding.direction,
        source_kind: finding.source_kind,
        resource: finding.resource.clone(),
        before_filename: finding.before_filename.clone(),
        after_filename: finding.after_filename.clone(),
        view: finding.view,
        element_id: finding.element_id.clone(),
        field: finding.field.clone(),
        before: finding.before.clone().map(canonicalize_json_value),
        after: finding.after.clone().map(canonicalize_json_value),
    };
    let bytes = canonical_json_bytes(&key)?;
    Ok(FindingFingerprint {
        schema: FindingFingerprint::SCHEMA_V1,
        digest: sha256_identity(&bytes),
    })
}

pub fn evaluate_quality_gate(
    current: &CheckReport,
    baseline: Option<&CheckReport>,
    suppressions: Option<&GateSuppressions>,
) -> Result<QualityGateReport, QualityGateError> {
    validate_check_report(current)?;

    let current_fingerprints = current_fingerprints(current)?;
    let baseline_evidence = baseline
        .map(|baseline| normalize_baseline(current, baseline))
        .transpose()?;
    let suppression_evidence = suppressions.map(normalize_suppressions).transpose()?;

    let baseline_members = baseline_evidence
        .as_ref()
        .map(|evidence| {
            evidence
                .fingerprints
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let suppression_map = suppression_evidence
        .as_ref()
        .map(|evidence| {
            evidence
                .suppressions
                .iter()
                .cloned()
                .map(|entry| (entry.finding_fingerprint.clone(), entry))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut findings = Vec::with_capacity(current_fingerprints.len());
    for (finding_index, fingerprint) in current_fingerprints.iter().cloned().enumerate() {
        let (disposition, matched_suppression) =
            if let Some(suppression) = suppression_map.get(&fingerprint) {
                (
                    QualityGateDisposition::Suppressed,
                    Some(suppression.clone()),
                )
            } else if baseline_members.contains(&fingerprint) {
                (QualityGateDisposition::Baseline, None)
            } else {
                (QualityGateDisposition::New, None)
            };
        findings.push(QualityGateFinding {
            finding_index,
            fingerprint,
            disposition,
            matched_suppression,
        });
    }

    let current_members = current_fingerprints.into_iter().collect::<BTreeSet<_>>();
    let unused_suppressions = suppression_map
        .keys()
        .filter(|fingerprint| !current_members.contains(*fingerprint))
        .cloned()
        .collect::<Vec<_>>();
    let decision = build_quality_gate_decision(current, &findings, unused_suppressions.len());

    let report = QualityGateReport {
        schema: QualityGateReport::SCHEMA_V1,
        current: current.clone(),
        baseline: baseline_evidence,
        suppression_evidence,
        findings,
        unused_suppressions,
        decision,
    };
    validate_quality_gate_report(&report)?;
    Ok(report)
}

pub fn validate_quality_gate_report(report: &QualityGateReport) -> Result<(), QualityGateError> {
    if report.schema != QualityGateReport::SCHEMA_V1 {
        return Err(QualityGateError::UnsupportedGateSchema {
            found: report.schema,
            expected: QualityGateReport::SCHEMA_V1,
        });
    }
    validate_check_report(&report.current)?;

    let current_fingerprints = current_fingerprints(&report.current)?;
    let baseline_members = match report.baseline.as_ref() {
        Some(evidence) => validate_baseline_evidence(&report.current, evidence)?,
        None => BTreeSet::new(),
    };
    let suppression_map = match report.suppression_evidence.as_ref() {
        Some(evidence) => validate_suppression_evidence(evidence)?,
        None => BTreeMap::new(),
    };

    if report.findings.len() != current_fingerprints.len() {
        return Err(inconsistent(
            "finding count does not match current CF-05 evidence",
        ));
    }

    for (index, (gate_finding, current_fingerprint)) in report
        .findings
        .iter()
        .zip(current_fingerprints.iter())
        .enumerate()
    {
        if gate_finding.finding_index != index {
            return Err(inconsistent(
                "finding_index does not match current finding order",
            ));
        }
        validate_fingerprint(&gate_finding.fingerprint)?;
        if &gate_finding.fingerprint != current_fingerprint {
            return Err(inconsistent(
                "persisted finding fingerprint does not match current evidence",
            ));
        }

        let expected_suppression = suppression_map.get(current_fingerprint);
        let expected_disposition = if expected_suppression.is_some() {
            QualityGateDisposition::Suppressed
        } else if baseline_members.contains(current_fingerprint) {
            QualityGateDisposition::Baseline
        } else {
            QualityGateDisposition::New
        };
        if gate_finding.disposition != expected_disposition {
            return Err(inconsistent(
                "finding disposition does not match retained membership evidence",
            ));
        }
        match (
            expected_suppression,
            gate_finding.matched_suppression.as_ref(),
        ) {
            (Some(expected), Some(found)) if expected == found => {}
            (None, None) => {}
            _ => {
                return Err(inconsistent(
                    "matched suppression metadata does not match retained suppression evidence",
                ));
            }
        }
    }

    let current_members = current_fingerprints.into_iter().collect::<BTreeSet<_>>();
    let expected_unused = suppression_map
        .keys()
        .filter(|fingerprint| !current_members.contains(*fingerprint))
        .cloned()
        .collect::<Vec<_>>();
    for fingerprint in &report.unused_suppressions {
        validate_fingerprint(fingerprint)?;
    }
    if report.unused_suppressions != expected_unused {
        return Err(inconsistent(
            "unused suppressions do not match retained suppression evidence",
        ));
    }

    let expected_decision =
        build_quality_gate_decision(&report.current, &report.findings, expected_unused.len());
    if report.decision != expected_decision {
        return Err(inconsistent(
            "quality-gate decision does not match current policy and dispositions",
        ));
    }
    Ok(())
}

fn normalize_baseline(
    current: &CheckReport,
    baseline: &CheckReport,
) -> Result<QualityGateBaselineEvidence, QualityGateError> {
    validate_check_report(baseline)?;
    if baseline.compatibility.package_name != current.compatibility.package_name {
        return Err(QualityGateError::BaselinePackageMismatch {
            current: current.compatibility.package_name.clone(),
            baseline: baseline.compatibility.package_name.clone(),
        });
    }
    if baseline.compatibility.ruleset != current.compatibility.ruleset {
        return Err(QualityGateError::BaselineRulesetMismatch {
            current: current.compatibility.ruleset.clone(),
            baseline: baseline.compatibility.ruleset.clone(),
        });
    }

    let mut seen = BTreeSet::new();
    let mut fingerprints = Vec::with_capacity(baseline.compatibility.findings.len());
    for finding in &baseline.compatibility.findings {
        let fingerprint = finding_fingerprint_v1(&baseline.compatibility.ruleset, finding)?;
        if !seen.insert(fingerprint.clone()) {
            return Err(QualityGateError::DuplicateBaselineFingerprint {
                fingerprint: fingerprint.digest,
            });
        }
        fingerprints.push(fingerprint);
    }
    fingerprints.sort();

    let canonical_sha256 = sha256_identity(&canonical_json_bytes(baseline)?);
    Ok(QualityGateBaselineEvidence {
        canonical_sha256,
        fingerprint_schema: FindingFingerprint::SCHEMA_V1,
        package_name: baseline.compatibility.package_name.clone(),
        ruleset: baseline.compatibility.ruleset.clone(),
        before: baseline.compatibility.before.clone(),
        after: baseline.compatibility.after.clone(),
        finding_count: fingerprints.len(),
        fingerprints,
    })
}

fn normalize_suppressions(
    suppressions: &GateSuppressions,
) -> Result<QualityGateSuppressionEvidence, QualityGateError> {
    if suppressions.schema != GateSuppressions::SCHEMA_V1 {
        return Err(QualityGateError::UnsupportedSuppressionSchema {
            found: suppressions.schema,
            expected: GateSuppressions::SCHEMA_V1,
        });
    }
    if suppressions.suppressions.len() > MAX_GATE_SUPPRESSIONS {
        return Err(QualityGateError::TooManySuppressions {
            found: suppressions.suppressions.len(),
            maximum: MAX_GATE_SUPPRESSIONS,
        });
    }

    let mut normalized = suppressions.suppressions.clone();
    normalized.sort_by(|left, right| left.finding_fingerprint.cmp(&right.finding_fingerprint));
    let mut previous: Option<&FindingFingerprint> = None;
    for suppression in &normalized {
        validate_suppression(suppression)?;
        if previous == Some(&suppression.finding_fingerprint) {
            return Err(QualityGateError::DuplicateSuppressionFingerprint {
                fingerprint: suppression.finding_fingerprint.digest.clone(),
            });
        }
        previous = Some(&suppression.finding_fingerprint);
    }

    let normalized_input = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: normalized.clone(),
    };
    let canonical_sha256 = sha256_identity(&canonical_json_bytes(&normalized_input)?);
    Ok(QualityGateSuppressionEvidence {
        canonical_sha256,
        schema: GateSuppressions::SCHEMA_V1,
        fingerprint_schema: FindingFingerprint::SCHEMA_V1,
        entry_count: normalized.len(),
        suppressions: normalized,
    })
}

fn validate_baseline_evidence(
    current: &CheckReport,
    evidence: &QualityGateBaselineEvidence,
) -> Result<BTreeSet<FindingFingerprint>, QualityGateError> {
    if evidence.fingerprint_schema != FindingFingerprint::SCHEMA_V1 {
        return Err(QualityGateError::UnsupportedFingerprintSchema {
            found: evidence.fingerprint_schema,
            expected: FindingFingerprint::SCHEMA_V1,
        });
    }
    validate_sha256_identity(&evidence.canonical_sha256)?;
    validate_raw_sha256(&evidence.before.archive_sha256)?;
    validate_raw_sha256(&evidence.after.archive_sha256)?;
    if evidence.package_name != current.compatibility.package_name {
        return Err(inconsistent(
            "baseline evidence package does not match current package",
        ));
    }
    if evidence.ruleset != current.compatibility.ruleset {
        return Err(inconsistent(
            "baseline evidence ruleset does not match current ruleset",
        ));
    }
    if evidence.finding_count != evidence.fingerprints.len() {
        return Err(inconsistent(
            "baseline evidence finding count is inconsistent",
        ));
    }
    validate_sorted_unique_fingerprints(
        &evidence.fingerprints,
        "baseline fingerprints are not sorted and unique",
    )?;
    Ok(evidence.fingerprints.iter().cloned().collect())
}

fn validate_suppression_evidence(
    evidence: &QualityGateSuppressionEvidence,
) -> Result<BTreeMap<FindingFingerprint, GateSuppression>, QualityGateError> {
    if evidence.schema != GateSuppressions::SCHEMA_V1 {
        return Err(QualityGateError::UnsupportedSuppressionSchema {
            found: evidence.schema,
            expected: GateSuppressions::SCHEMA_V1,
        });
    }
    if evidence.fingerprint_schema != FindingFingerprint::SCHEMA_V1 {
        return Err(QualityGateError::UnsupportedFingerprintSchema {
            found: evidence.fingerprint_schema,
            expected: FindingFingerprint::SCHEMA_V1,
        });
    }
    validate_sha256_identity(&evidence.canonical_sha256)?;
    if evidence.entry_count != evidence.suppressions.len() {
        return Err(inconsistent(
            "suppression evidence entry count is inconsistent",
        ));
    }
    if evidence.suppressions.len() > MAX_GATE_SUPPRESSIONS {
        return Err(QualityGateError::TooManySuppressions {
            found: evidence.suppressions.len(),
            maximum: MAX_GATE_SUPPRESSIONS,
        });
    }

    let mut map = BTreeMap::new();
    let mut previous: Option<&FindingFingerprint> = None;
    for suppression in &evidence.suppressions {
        validate_suppression(suppression)?;
        if let Some(previous) = previous {
            if previous >= &suppression.finding_fingerprint {
                return Err(inconsistent(
                    "suppression evidence is not sorted and unique",
                ));
            }
        }
        previous = Some(&suppression.finding_fingerprint);
        map.insert(suppression.finding_fingerprint.clone(), suppression.clone());
    }

    let normalized_input = GateSuppressions {
        schema: evidence.schema,
        suppressions: evidence.suppressions.clone(),
    };
    let expected_digest = sha256_identity(&canonical_json_bytes(&normalized_input)?);
    if evidence.canonical_sha256 != expected_digest {
        return Err(inconsistent(
            "suppression canonical digest does not match retained evidence",
        ));
    }
    Ok(map)
}

fn current_fingerprints(
    current: &CheckReport,
) -> Result<Vec<FindingFingerprint>, QualityGateError> {
    let mut seen = BTreeSet::new();
    let mut fingerprints = Vec::with_capacity(current.compatibility.findings.len());
    for finding in &current.compatibility.findings {
        let fingerprint = finding_fingerprint_v1(&current.compatibility.ruleset, finding)?;
        if !seen.insert(fingerprint.clone()) {
            return Err(QualityGateError::DuplicateCurrentFingerprint {
                fingerprint: fingerprint.digest,
            });
        }
        fingerprints.push(fingerprint);
    }
    Ok(fingerprints)
}

fn build_quality_gate_decision(
    current: &CheckReport,
    findings: &[QualityGateFinding],
    unused_suppressions: usize,
) -> QualityGateDecision {
    let mut selected_findings = 0usize;
    let mut new_findings = 0usize;
    let mut baseline_findings = 0usize;
    let mut suppressed_findings = 0usize;
    let mut new_selected_breaking_findings = 0usize;
    let mut new_selected_risky_findings = 0usize;
    let mut new_selected_additive_findings = 0usize;
    let mut blocking_findings = 0usize;

    for (finding, gate_finding) in current.compatibility.findings.iter().zip(findings) {
        let selected = direction_selected(current.policy.direction, finding.direction);
        if selected {
            selected_findings += 1;
        }
        match gate_finding.disposition {
            QualityGateDisposition::New => {
                new_findings += 1;
                if selected {
                    match finding.severity {
                        CompatibilitySeverity::Breaking => new_selected_breaking_findings += 1,
                        CompatibilitySeverity::Risky => new_selected_risky_findings += 1,
                        CompatibilitySeverity::Additive => new_selected_additive_findings += 1,
                    }
                    if severity_blocks(current.policy.fail_on, finding.severity) {
                        blocking_findings += 1;
                    }
                }
            }
            QualityGateDisposition::Baseline => baseline_findings += 1,
            QualityGateDisposition::Suppressed => suppressed_findings += 1,
        }
    }

    QualityGateDecision {
        passed: blocking_findings == 0,
        total_findings: current.compatibility.findings.len(),
        selected_findings,
        new_findings,
        baseline_findings,
        suppressed_findings,
        new_selected_breaking_findings,
        new_selected_risky_findings,
        new_selected_additive_findings,
        blocking_findings,
        unused_suppressions,
    }
}

fn validate_suppression(suppression: &GateSuppression) -> Result<(), QualityGateError> {
    validate_fingerprint(&suppression.finding_fingerprint)?;
    if suppression.rationale.trim().is_empty() {
        return Err(QualityGateError::EmptySuppressionRationale);
    }
    let rationale_chars = suppression.rationale.chars().count();
    if rationale_chars > MAX_GATE_SUPPRESSION_RATIONALE_CHARS {
        return Err(QualityGateError::SuppressionStringTooLong {
            field: "rationale",
            found: rationale_chars,
            maximum: MAX_GATE_SUPPRESSION_RATIONALE_CHARS,
        });
    }
    if let Some(reference) = suppression.reference.as_ref() {
        let reference_chars = reference.chars().count();
        if reference_chars > MAX_GATE_SUPPRESSION_REFERENCE_CHARS {
            return Err(QualityGateError::SuppressionStringTooLong {
                field: "reference",
                found: reference_chars,
                maximum: MAX_GATE_SUPPRESSION_REFERENCE_CHARS,
            });
        }
    }
    Ok(())
}

fn validate_sorted_unique_fingerprints(
    fingerprints: &[FindingFingerprint],
    reason: &'static str,
) -> Result<(), QualityGateError> {
    let mut previous: Option<&FindingFingerprint> = None;
    for fingerprint in fingerprints {
        validate_fingerprint(fingerprint)?;
        if let Some(previous) = previous {
            if previous >= fingerprint {
                return Err(inconsistent(reason));
            }
        }
        previous = Some(fingerprint);
    }
    Ok(())
}

fn validate_fingerprint(fingerprint: &FindingFingerprint) -> Result<(), QualityGateError> {
    if fingerprint.schema != FindingFingerprint::SCHEMA_V1 {
        return Err(QualityGateError::UnsupportedFingerprintSchema {
            found: fingerprint.schema,
            expected: FindingFingerprint::SCHEMA_V1,
        });
    }
    validate_sha256_identity(&fingerprint.digest)
}

fn validate_sha256_identity(value: &str) -> Result<(), QualityGateError> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(QualityGateError::MalformedSha256Identity {
            value: value.to_owned(),
        });
    };
    if !is_lower_hex_64(hex) {
        return Err(QualityGateError::MalformedSha256Identity {
            value: value.to_owned(),
        });
    }
    Ok(())
}

fn validate_raw_sha256(value: &str) -> Result<(), QualityGateError> {
    if !is_lower_hex_64(value) {
        return Err(QualityGateError::MalformedSha256Identity {
            value: value.to_owned(),
        });
    }
    Ok(())
}

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    serde_json::to_vec(&canonicalize_json_value(value))
}

fn canonicalize_json_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            let mut canonical = Map::new();
            for (key, value) in entries {
                canonical.insert(key, canonicalize_json_value(value));
            }
            Value::Object(canonical)
        }
        Value::Array(values) => {
            Value::Array(values.into_iter().map(canonicalize_json_value).collect())
        }
        scalar => scalar,
    }
}

fn sha256_identity(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn inconsistent(reason: &'static str) -> QualityGateError {
    QualityGateError::InconsistentReport { reason }
}
