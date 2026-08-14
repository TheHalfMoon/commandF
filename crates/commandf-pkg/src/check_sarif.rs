use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde_json::{json, Value};

use crate::{
    check::validate_compatibility_report, CheckError, CheckReport, CompatibilityFinding,
    CompatibilitySeverity,
};

const SARIF_SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";

#[derive(Serialize)]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
    properties: BTreeMap<String, Value>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: String,
    #[serde(rename = "semanticVersion")]
    semantic_version: String,
    rules: Vec<SarifRule>,
}

#[derive(Serialize)]
struct SarifRule {
    id: String,
}

#[derive(Serialize)]
struct SarifResult {
    #[serde(rename = "ruleId")]
    rule_id: String,
    level: String,
    message: SarifMessage,
    properties: BTreeMap<String, Value>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

pub fn check_report_to_sarif_bytes(report: &CheckReport) -> Result<Vec<u8>, CheckError> {
    if report.schema != CheckReport::SCHEMA_V1 {
        return Err(CheckError::UnsupportedCheckSchema {
            found: report.schema,
            expected: CheckReport::SCHEMA_V1,
        });
    }
    validate_compatibility_report(&report.compatibility)?;

    let rule_ids = report
        .compatibility
        .findings
        .iter()
        .map(|finding| finding.rule_id.clone())
        .collect::<BTreeSet<_>>();
    let rules = rule_ids
        .into_iter()
        .map(|id| SarifRule { id })
        .collect::<Vec<_>>();
    let results = report
        .compatibility
        .findings
        .iter()
        .map(sarif_result)
        .collect::<Result<Vec<_>, _>>()?;

    let log = SarifLog {
        schema: SARIF_SCHEMA.to_owned(),
        version: SARIF_VERSION.to_owned(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "commandF".to_owned(),
                    semantic_version: env!("CARGO_PKG_VERSION").to_owned(),
                    rules,
                },
            },
            results,
            properties: run_properties(report)?,
        }],
    };

    let mut bytes = serde_json::to_vec_pretty(&log)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sarif_result(finding: &CompatibilityFinding) -> Result<SarifResult, CheckError> {
    let mut properties = BTreeMap::new();
    properties.insert(
        "commandf.compatibilitySeverity".to_owned(),
        serde_json::to_value(finding.severity)?,
    );
    properties.insert(
        "commandf.direction".to_owned(),
        serde_json::to_value(finding.direction)?,
    );
    properties.insert(
        "commandf.sourceKind".to_owned(),
        serde_json::to_value(finding.source_kind)?,
    );
    properties.insert(
        "commandf.resourceKind".to_owned(),
        serde_json::to_value(finding.resource.kind)?,
    );
    properties.insert(
        "commandf.resource".to_owned(),
        json!(finding.resource.value),
    );
    insert_string(
        &mut properties,
        "commandf.beforeFilename",
        finding.before_filename.as_deref(),
    );
    insert_string(
        &mut properties,
        "commandf.afterFilename",
        finding.after_filename.as_deref(),
    );
    if let Some(view) = finding.view {
        properties.insert("commandf.view".to_owned(), serde_json::to_value(view)?);
    }
    insert_string(
        &mut properties,
        "commandf.elementId",
        finding.element_id.as_deref(),
    );
    insert_string(&mut properties, "commandf.field", finding.field.as_deref());
    if let Some(before) = &finding.before {
        properties.insert("commandf.before".to_owned(), before.clone());
    }
    if let Some(after) = &finding.after {
        properties.insert("commandf.after".to_owned(), after.clone());
    }

    Ok(SarifResult {
        rule_id: finding.rule_id.clone(),
        level: sarif_level(finding.severity).to_owned(),
        message: SarifMessage {
            text: finding.message.clone(),
        },
        properties,
    })
}

fn run_properties(report: &CheckReport) -> Result<BTreeMap<String, Value>, CheckError> {
    let mut properties = BTreeMap::new();
    properties.insert("commandf.checkSchema".to_owned(), json!(report.schema));
    properties.insert(
        "commandf.ruleset".to_owned(),
        json!(report.compatibility.ruleset),
    );
    properties.insert(
        "commandf.packageName".to_owned(),
        json!(report.compatibility.package_name),
    );
    properties.insert(
        "commandf.beforeVersion".to_owned(),
        json!(report.compatibility.before.version),
    );
    properties.insert(
        "commandf.beforeArchiveSha256".to_owned(),
        json!(report.compatibility.before.archive_sha256),
    );
    properties.insert(
        "commandf.afterVersion".to_owned(),
        json!(report.compatibility.after.version),
    );
    properties.insert(
        "commandf.afterArchiveSha256".to_owned(),
        json!(report.compatibility.after.archive_sha256),
    );
    properties.insert(
        "commandf.policy.direction".to_owned(),
        serde_json::to_value(report.policy.direction)?,
    );
    properties.insert(
        "commandf.policy.failOn".to_owned(),
        serde_json::to_value(report.policy.fail_on)?,
    );
    properties.insert(
        "commandf.decision.passed".to_owned(),
        json!(report.decision.passed),
    );
    properties.insert(
        "commandf.decision.totalFindings".to_owned(),
        json!(report.decision.total_findings),
    );
    properties.insert(
        "commandf.decision.selectedFindings".to_owned(),
        json!(report.decision.selected_findings),
    );
    properties.insert(
        "commandf.decision.breakingFindings".to_owned(),
        json!(report.decision.breaking_findings),
    );
    properties.insert(
        "commandf.decision.riskyFindings".to_owned(),
        json!(report.decision.risky_findings),
    );
    properties.insert(
        "commandf.decision.additiveFindings".to_owned(),
        json!(report.decision.additive_findings),
    );
    properties.insert(
        "commandf.decision.blockingFindings".to_owned(),
        json!(report.decision.blocking_findings),
    );
    properties.insert("commandf.sourceMapping".to_owned(), json!("deferred_cf09"));
    Ok(properties)
}

fn insert_string(properties: &mut BTreeMap<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        properties.insert(key.to_owned(), json!(value));
    }
}

fn sarif_level(severity: CompatibilitySeverity) -> &'static str {
    match severity {
        CompatibilitySeverity::Breaking => "error",
        CompatibilitySeverity::Risky => "warning",
        CompatibilitySeverity::Additive => "note",
    }
}
