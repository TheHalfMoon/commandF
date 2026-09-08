pub const INVALIDITY_CLASSES: &[&str] = &[
    "DUPLICATE_CURRENT_FINGERPRINT",
    "DUPLICATE_BASELINE_FINGERPRINT",
    "DUPLICATE_SUPPRESSION_FINGERPRINT",
    "SUPPRESSION_METADATA_MISMATCH",
    "BASELINE_PACKAGE_MISMATCH",
    "BASELINE_RULESET_MISMATCH",
    "FINGERPRINT_FIELD_TAMPER",
    "DECISION_COUNTER_TAMPER",
    "UNUSED_SUPPRESSION_TAMPER",
];

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Breaking,
    Risky,
    Additive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    Producer,
    Consumer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedDirection {
    Both,
    Producer,
    Consumer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailOn {
    Breaking,
    Risky,
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub direction: Direction,
    pub resource_value: String,
    pub before_filename: Option<String>,
    pub after_filename: Option<String>,
    pub element_id: Option<String>,
    pub field: Option<String>,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Suppression {
    pub fingerprint: String,
    pub rationale: String,
    pub reference: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Disposition {
    New,
    Baseline,
    Suppressed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decision {
    pub dispositions: Vec<Disposition>,
    pub unused_suppressions: Vec<String>,
    pub selected_findings: usize,
    pub new_findings: usize,
    pub baseline_findings: usize,
    pub suppressed_findings: usize,
    pub new_selected_breaking_findings: usize,
    pub new_selected_risky_findings: usize,
    pub new_selected_additive_findings: usize,
    pub blocking_findings: usize,
    pub passed: bool,
}

pub fn fingerprint(ruleset: &str, finding: &Finding) -> String {
    let value = serde_json::json!({
        "schema": 1,
        "ruleset": ruleset,
        "rule_id": finding.rule_id.clone(),
        "severity": severity_text(finding.severity),
        "direction": direction_text(finding.direction),
        "source_kind": "element_field_changed",
        "resource": {
            "kind": "canonical",
            "value": finding.resource_value.clone(),
        },
        "before_filename": finding.before_filename.clone(),
        "after_filename": finding.after_filename.clone(),
        "view": "snapshot",
        "element_id": finding.element_id.clone(),
        "field": finding.field.clone(),
        "before": finding.before.clone(),
        "after": finding.after.clone(),
    });
    let canonical = canonicalize(value);
    let bytes = serde_json::to_vec(&canonical).expect("model JSON must serialize");
    format!("sha256:{:x}", Sha256::digest(bytes))
}

pub fn evaluate(
    ruleset: &str,
    current: &[Finding],
    baseline: &[Finding],
    suppressions: &[Suppression],
    selected_direction: SelectedDirection,
    fail_on: FailOn,
) -> Result<Decision, &'static str> {
    let current_fingerprints = unique_fingerprints(ruleset, current, "duplicate current")?;
    let baseline_fingerprints = unique_fingerprints(ruleset, baseline, "duplicate baseline")?
        .into_iter()
        .collect::<BTreeSet<_>>();

    let mut suppression_map = BTreeMap::new();
    for suppression in suppressions {
        if suppression_map
            .insert(suppression.fingerprint.clone(), suppression)
            .is_some()
        {
            return Err("duplicate suppression");
        }
    }

    let mut dispositions = Vec::with_capacity(current.len());
    let mut selected_findings = 0;
    let mut new_findings = 0;
    let mut baseline_findings = 0;
    let mut suppressed_findings = 0;
    let mut new_selected_breaking_findings = 0;
    let mut new_selected_risky_findings = 0;
    let mut new_selected_additive_findings = 0;
    let mut blocking_findings = 0;

    for (finding, fingerprint) in current.iter().zip(&current_fingerprints) {
        let disposition = if suppression_map.contains_key(fingerprint) {
            Disposition::Suppressed
        } else if baseline_fingerprints.contains(fingerprint) {
            Disposition::Baseline
        } else {
            Disposition::New
        };
        dispositions.push(disposition);

        let selected = direction_selected(selected_direction, finding.direction);
        if selected {
            selected_findings += 1;
        }

        match disposition {
            Disposition::New => {
                new_findings += 1;
                if selected {
                    match finding.severity {
                        Severity::Breaking => new_selected_breaking_findings += 1,
                        Severity::Risky => new_selected_risky_findings += 1,
                        Severity::Additive => new_selected_additive_findings += 1,
                    }
                    if severity_blocks(fail_on, finding.severity) {
                        blocking_findings += 1;
                    }
                }
            }
            Disposition::Baseline => baseline_findings += 1,
            Disposition::Suppressed => suppressed_findings += 1,
        }
    }

    let current_set = current_fingerprints.into_iter().collect::<BTreeSet<_>>();
    let unused_suppressions = suppression_map
        .keys()
        .filter(|fingerprint| !current_set.contains(*fingerprint))
        .cloned()
        .collect();

    Ok(Decision {
        dispositions,
        unused_suppressions,
        selected_findings,
        new_findings,
        baseline_findings,
        suppressed_findings,
        new_selected_breaking_findings,
        new_selected_risky_findings,
        new_selected_additive_findings,
        blocking_findings,
        passed: blocking_findings == 0,
    })
}

fn unique_fingerprints(
    ruleset: &str,
    findings: &[Finding],
    duplicate_error: &'static str,
) -> Result<Vec<String>, &'static str> {
    let mut seen = BTreeSet::new();
    let mut output = Vec::with_capacity(findings.len());
    for finding in findings {
        let fingerprint = fingerprint(ruleset, finding);
        if !seen.insert(fingerprint.clone()) {
            return Err(duplicate_error);
        }
        output.push(fingerprint);
    }
    Ok(output)
}

fn direction_selected(selected: SelectedDirection, finding: Direction) -> bool {
    match selected {
        SelectedDirection::Both => true,
        SelectedDirection::Producer => finding == Direction::Producer,
        SelectedDirection::Consumer => finding == Direction::Consumer,
    }
}

fn severity_blocks(fail_on: FailOn, severity: Severity) -> bool {
    match fail_on {
        FailOn::Breaking => severity == Severity::Breaking,
        FailOn::Risky => matches!(severity, Severity::Breaking | Severity::Risky),
        FailOn::None => false,
    }
}

fn severity_text(severity: Severity) -> &'static str {
    match severity {
        Severity::Breaking => "BREAKING",
        Severity::Risky => "RISKY",
        Severity::Additive => "ADDITIVE",
    }
}

fn direction_text(direction: Direction) -> &'static str {
    match direction {
        Direction::Producer => "producer",
        Direction::Consumer => "consumer",
    }
}

fn canonicalize(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
            let mut normalized = Map::new();
            for (key, value) in entries {
                normalized.insert(key, canonicalize(value));
            }
            Value::Object(normalized)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize).collect()),
        other => other,
    }
}
