use crate::{
    attest_corpus_package_state, build_terminology_diff_report, classify_structural_diff,
    diff_package_archives, CompatibilityDirection, CompatibilityReport, CompatibilitySeverity,
    CorpusCaseStatus, CorpusCaseSummary, CorpusCompatibilitySummary, CorpusError,
    CorpusOracleSummary, CorpusPackageSide, CorpusStructuralSummary, CorpusSummaryPackageState,
    CorpusTerminologySummary, Hl7OracleReport, Lockfile, OracleDivergenceReport, OracleIdentity,
    OracleResourceStatus, PackageCache, RealIgCase, StructuralDiffReport, TerminologyDiffReport,
    TerminologyPackageState,
};

pub struct CorpusPackageStateInput<'a> {
    pub lockfile: &'a Lockfile,
    pub cache: &'a PackageCache,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusCaseReports {
    pub structural: StructuralDiffReport,
    pub compatibility: CompatibilityReport,
    pub terminology: TerminologyDiffReport,
}

pub fn evaluate_corpus_case(
    case: &RealIgCase,
    before: CorpusPackageStateInput<'_>,
    after: CorpusPackageStateInput<'_>,
) -> Result<CorpusCaseReports, CorpusError> {
    attest_corpus_package_state(case, CorpusPackageSide::Before, before.lockfile, before.cache)?;
    attest_corpus_package_state(case, CorpusPackageSide::After, after.lockfile, after.cache)?;

    let before_bytes = before
        .cache
        .read_verified(&case.before.archive_sha256)
        .map_err(|error| CorpusError::CacheVerification {
            case_id: case.id.clone(),
            message: error.to_string(),
        })?;
    let after_bytes = after
        .cache
        .read_verified(&case.after.archive_sha256)
        .map_err(|error| CorpusError::CacheVerification {
            case_id: case.id.clone(),
            message: error.to_string(),
        })?;

    let structural = diff_package_archives(
        case.package.clone(),
        case.before.version.clone(),
        case.before.archive_sha256.clone(),
        &before_bytes,
        case.after.version.clone(),
        case.after.archive_sha256.clone(),
        &after_bytes,
    )
    .map_err(|error| CorpusError::Evaluation {
        case_id: case.id.clone(),
        stage: "structural",
        message: error.to_string(),
    })?;

    let compatibility =
        classify_structural_diff(&structural).map_err(|error| CorpusError::Evaluation {
            case_id: case.id.clone(),
            stage: "compatibility",
            message: error.to_string(),
        })?;

    let terminology = build_terminology_diff_report(
        TerminologyPackageState {
            lockfile: before.lockfile,
            cache: before.cache,
            root_bytes: &before_bytes,
        },
        TerminologyPackageState {
            lockfile: after.lockfile,
            cache: after.cache,
            root_bytes: &after_bytes,
        },
        &structural,
        &compatibility,
    )
    .map_err(|error| CorpusError::Evaluation {
        case_id: case.id.clone(),
        stage: "terminology",
        message: error.to_string(),
    })?;

    Ok(CorpusCaseReports {
        structural,
        compatibility,
        terminology,
    })
}

pub fn summarize_corpus_case(
    case: &RealIgCase,
    reports: &CorpusCaseReports,
    oracle: &OracleDivergenceReport,
) -> Result<CorpusCaseSummary, CorpusError> {
    validate_report_identity(case, reports, oracle)?;

    let structural_bytes = reports
        .structural
        .to_json_bytes()
        .map_err(|error| serialization_error(case, "structural", error))?;
    let compatibility_bytes = reports
        .compatibility
        .to_json_bytes()
        .map_err(|error| serialization_error(case, "compatibility", error))?;
    let terminology_bytes = reports
        .terminology
        .to_json_bytes()
        .map_err(|error| serialization_error(case, "terminology", error))?;
    let oracle_bytes = oracle
        .to_json_bytes()
        .map_err(|error| serialization_error(case, "oracle", error))?;

    let mut breaking = 0usize;
    let mut risky = 0usize;
    let mut additive = 0usize;
    let mut producer = 0usize;
    let mut consumer = 0usize;
    for finding in &reports.compatibility.findings {
        match finding.severity {
            CompatibilitySeverity::Breaking => breaking += 1,
            CompatibilitySeverity::Risky => risky += 1,
            CompatibilitySeverity::Additive => additive += 1,
        }
        match finding.direction {
            CompatibilityDirection::Producer => producer += 1,
            CompatibilityDirection::Consumer => consumer += 1,
        }
    }

    let mut agreement = 0usize;
    let mut commandf_only = 0usize;
    let mut authority_only = 0usize;
    let mut both_changed = 0usize;
    let mut uncomparable = 0usize;
    for resource in &oracle.resources {
        match resource.status {
            OracleResourceStatus::Agreement => agreement += 1,
            OracleResourceStatus::CommandfOnly => commandf_only += 1,
            OracleResourceStatus::AuthorityOnly => authority_only += 1,
            OracleResourceStatus::BothChanged => both_changed += 1,
            OracleResourceStatus::Uncomparable => uncomparable += 1,
        }
    }

    Ok(CorpusCaseSummary {
        case_id: case.id.clone(),
        package: case.package.clone(),
        before: summary_state(&case.before.version, &case.before.archive_sha256),
        after: summary_state(&case.after.version, &case.after.archive_sha256),
        status: CorpusCaseStatus::Complete,
        structural: Some(CorpusStructuralSummary {
            changes: reports.structural.changes.len(),
            report_sha256: PackageCache::digest(&structural_bytes),
        }),
        compatibility: Some(CorpusCompatibilitySummary {
            findings: reports.compatibility.findings.len(),
            breaking,
            risky,
            additive,
            producer,
            consumer,
            report_sha256: PackageCache::digest(&compatibility_bytes),
        }),
        terminology: Some(CorpusTerminologySummary {
            code_system_changes: reports.terminology.code_systems.len(),
            value_set_changes: reports.terminology.value_sets.len(),
            binding_refinements: reports.terminology.binding_refinements.len(),
            report_sha256: PackageCache::digest(&terminology_bytes),
        }),
        oracle: Some(CorpusOracleSummary {
            compared: oracle.resources.len(),
            agreement,
            commandf_only,
            authority_only,
            both_changed,
            uncomparable,
            report_sha256: PackageCache::digest(&oracle_bytes),
        }),
    })
}

pub fn failed_corpus_case_summary(case: &RealIgCase, status: CorpusCaseStatus) -> CorpusCaseSummary {
    CorpusCaseSummary {
        case_id: case.id.clone(),
        package: case.package.clone(),
        before: summary_state(&case.before.version, &case.before.archive_sha256),
        after: summary_state(&case.after.version, &case.after.archive_sha256),
        status,
        structural: None,
        compatibility: None,
        terminology: None,
        oracle: None,
    }
}

fn validate_report_identity(
    case: &RealIgCase,
    reports: &CorpusCaseReports,
    oracle: &OracleDivergenceReport,
) -> Result<(), CorpusError> {
    if reports.structural.schema != StructuralDiffReport::SCHEMA_V1 {
        return Err(unsupported(case, "structural"));
    }
    if reports.structural.package_name != case.package
        || reports.structural.before.version != case.before.version
        || reports.structural.before.archive_sha256 != case.before.archive_sha256
        || reports.structural.after.version != case.after.version
        || reports.structural.after.archive_sha256 != case.after.archive_sha256
    {
        return Err(identity(case, "structural"));
    }

    if reports.compatibility.schema != CompatibilityReport::SCHEMA_V1
        || reports.compatibility.ruleset != CompatibilityReport::RULESET_V1
    {
        return Err(unsupported(case, "compatibility"));
    }
    if reports.compatibility.package_name != case.package
        || reports.compatibility.before != reports.structural.before
        || reports.compatibility.after != reports.structural.after
    {
        return Err(identity(case, "compatibility"));
    }

    if reports.terminology.schema != TerminologyDiffReport::SCHEMA_V1
        || reports.terminology.ruleset != TerminologyDiffReport::RULESET_V1
    {
        return Err(unsupported(case, "terminology"));
    }
    if reports.terminology.package_name != case.package
        || reports.terminology.before != reports.structural.before
        || reports.terminology.after != reports.structural.after
        || reports.terminology.compatibility != reports.compatibility
    {
        return Err(identity(case, "terminology"));
    }

    if oracle.schema != OracleDivergenceReport::SCHEMA_V1 || oracle.oracle != OracleIdentity::pinned_hl7()
    {
        return Err(unsupported(case, "oracle"));
    }
    if oracle.package_name != case.package || oracle.structural_diff != reports.structural {
        return Err(identity(case, "oracle"));
    }

    Ok(())
}

fn summary_state(version: &str, sha256: &str) -> CorpusSummaryPackageState {
    CorpusSummaryPackageState {
        version: version.to_owned(),
        sha256: sha256.to_owned(),
    }
}

fn identity(case: &RealIgCase, report: &'static str) -> CorpusError {
    CorpusError::ReportIdentityMismatch {
        case_id: case.id.clone(),
        report,
    }
}

fn unsupported(case: &RealIgCase, report: &'static str) -> CorpusError {
    CorpusError::UnsupportedReport {
        case_id: case.id.clone(),
        report,
    }
}

fn serialization_error(
    case: &RealIgCase,
    stage: &'static str,
    error: serde_json::Error,
) -> CorpusError {
    CorpusError::Evaluation {
        case_id: case.id.clone(),
        stage,
        message: error.to_string(),
    }
}

#[allow(dead_code)]
fn _assert_oracle_report_type(_: &Hl7OracleReport) {}
