//! Typed, bounded composition of existing policy and declared-graph reports.
//!
//! This is a preview. It is not an atomic snapshot, signed receipt,
//! complete consumer-contract review, or a new compatibility oracle.

use std::io;

use serde::Serialize;
use serde_json::{json, Value};

use crate::{
    check_report_to_sarif_bytes, validate_check_report, CheckReport, ContextGraphReport,
    ImpactReport, Lockfile,
};

const MAX_PART_BYTES: usize = 64 * 1024 * 1024;

#[derive(Serialize)]
struct ReviewPreviewEnvelope<'a> {
    schema: u32,
    scope: &'static str,
    complete_consumer_contract_review: bool,
    atomic_cross_step_snapshot: bool,
    signed_receipt: bool,
    check: &'a CheckReport,
    impact: &'a ImpactReport,
}

/// Compose typed internal reports, refusing mismatched input identities.
/// Does not guarantee that independently read lock/cache files stayed unchanged
/// between the two evaluations. The envelope states that limitation openly.
pub fn compose_review_preview(check: &CheckReport, impact_bytes: &[u8]) -> io::Result<Vec<u8>> {
    let check_bytes = check
        .to_json_bytes()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if check_bytes.len() > MAX_PART_BYTES || impact_bytes.len() > MAX_PART_BYTES {
        return Err(invalid("review-preview report part exceeds 64 MiB limit"));
    }

    validate_check_report(check)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let impact: ImpactReport = serde_json::from_slice(impact_bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if impact.schema != ImpactReport::SCHEMA_V1
        || impact.before_evidence.graph_schema != ContextGraphReport::SCHEMA_V1
        || impact.after_evidence.graph_schema != ContextGraphReport::SCHEMA_V1
        || impact.before_evidence.lock_schema != Lockfile::SCHEMA_V2
        || impact.after_evidence.lock_schema != Lockfile::SCHEMA_V2
    {
        return Err(invalid("review-preview impact schema identity unsupported"));
    }

    let compatibility = &check.compatibility;
    let subject = &impact.subject;
    if compatibility.package_name != subject.package_name
        || subject.before.name != subject.package_name
        || subject.after.name != subject.package_name
        || subject.before.version != compatibility.before.version
        || subject.after.version != compatibility.after.version
        || subject.before.sha256 != compatibility.before.archive_sha256
        || subject.after.sha256 != compatibility.after.archive_sha256
        || impact.before_evidence.subject.identity != subject.before
        || impact.after_evidence.subject.identity != subject.after
    {
        return Err(invalid(
            "review-preview check and impact identities disagree",
        ));
    }

    let envelope = ReviewPreviewEnvelope {
        schema: 1,
        scope: "structural-and-declared-graph-preview",
        complete_consumer_contract_review: false,
        atomic_cross_step_snapshot: false,
        signed_receipt: false,
        check,
        impact: &impact,
    };
    let mut bytes = serde_json::to_vec_pretty(&envelope)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    bytes.push(b'\n');
    Ok(bytes)
}

/// Emit a genuine SARIF 2.1.0 log using the existing deterministic
/// compatibility projection, carrying the independently verified graph
/// impact under one run-level property. This is not a full consumer review.
pub fn compose_review_preview_sarif(
    check: &CheckReport,
    impact_bytes: &[u8],
) -> io::Result<Vec<u8>> {
    // Reuse all existing typed report, schema and identity consistency checks.
    // Do not create a second weaker path to emit SARIF.
    let preview_bytes = compose_review_preview(check, impact_bytes)?;
    let preview: Value = serde_json::from_slice(&preview_bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let impact = preview
        .get("impact")
        .cloned()
        .ok_or_else(|| invalid("review-preview missing verified impact report"))?;

    let raw_sarif = check_report_to_sarif_bytes(check)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if raw_sarif.len() > MAX_PART_BYTES {
        return Err(invalid("review-preview SARIF report exceeds 64 MiB limit"));
    }
    let mut sarif: Value = serde_json::from_slice(&raw_sarif)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let runs = sarif
        .get_mut("runs")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| invalid("review-preview SARIF runs missing"))?;
    if runs.len() != 1 {
        return Err(invalid("review-preview SARIF must contain one run"));
    }
    let properties = runs[0]
        .get_mut("properties")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| invalid("review-preview SARIF run properties missing"))?;
    properties.insert("commandf.reviewPreviewSchema".to_owned(), json!(1));
    properties.insert(
        "commandf.previewScope".to_owned(),
        json!("structural-and-declared-graph-preview"),
    );
    properties.insert(
        "commandf.completeConsumerContractReview".to_owned(),
        json!(false),
    );
    properties.insert("commandf.atomicCrossStepSnapshot".to_owned(), json!(false));
    properties.insert("commandf.signedReceipt".to_owned(), json!(false));
    properties.insert("commandf.impactReport".to_owned(), impact);

    let mut output = serde_json::to_vec_pretty(&sarif)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if output.len() > MAX_PART_BYTES * 3 {
        return Err(invalid("review-preview combined SARIF exceeds 192 MiB limit"));
    }
    output.push(b'\n');
    Ok(output)
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
