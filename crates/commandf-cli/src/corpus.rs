use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use commandf_pkg::{
    evaluate_corpus_case, failed_corpus_case_summary, parse_corpus_manifest, summarize_corpus_case,
    CorpusCaseStatus, CorpusCaseSummary, CorpusError, CorpusPackageStateInput, CorpusRunSummary,
    FhirRegistrySource, Lockfile, PackageCache, PackageRequest, RealIgCase, Resolver,
    MAX_CORPUS_MANIFEST_BYTES,
};

use crate::oracle;

const MAX_FAILURE_DIAGNOSTIC_CHARS: usize = 16_384;
const MAX_CORPUS_RAW_REPORT_BYTES: usize = 64 * 1024 * 1024;
const MAX_CORPUS_SUMMARY_BYTES: usize = 1024 * 1024;

pub struct CorpusExecution {
    pub summary: CorpusRunSummary,
    pub failed: bool,
}

struct ResolvedState {
    cache_path: PathBuf,
    lock_path: PathBuf,
    cache: PackageCache,
    lockfile: Lockfile,
}

pub fn run(
    manifest_path: PathBuf,
    work_root: PathBuf,
    oracle_adapter: PathBuf,
    oracle_java: Option<PathBuf>,
) -> Result<CorpusExecution, Box<dyn std::error::Error>> {
    let manifest_bytes = read_bounded_file(&manifest_path, MAX_CORPUS_MANIFEST_BYTES as u64)?;
    let corpus = parse_corpus_manifest(&manifest_bytes)?;
    let manifest_sha256 = PackageCache::digest(&manifest_bytes);

    prepare_fresh_work_root(&work_root)?;
    let evidence_root = work_root.join("evidence");
    fs::create_dir_all(&evidence_root)?;

    let mut summaries = Vec::with_capacity(corpus.cases.len());
    let mut failed = false;

    for case in &corpus.cases {
        let evidence_dir = evidence_root.join(&case.id);
        fs::create_dir_all(&evidence_dir)?;

        let before = resolve_state(case, true, &work_root);
        let after = resolve_state(case, false, &work_root);
        let (before, after) = match (before, after) {
            (Ok(before), Ok(after)) => (before, after),
            (before, after) => {
                failed = true;
                write_resolution_failures(&evidence_dir, before.err(), after.err())?;
                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::AcquisitionFailed,
                ));
                continue;
            }
        };

        let reports = match evaluate_corpus_case(
            case,
            CorpusPackageStateInput {
                lockfile: &before.lockfile,
                cache: &before.cache,
            },
            CorpusPackageStateInput {
                lockfile: &after.lockfile,
                cache: &after.cache,
            },
        ) {
            Ok(reports) => reports,
            Err(error) => {
                failed = true;
                let status = evaluation_status(&error);
                write_failure(&evidence_dir, "evaluation-failure.txt", &error)?;
                summaries.push(failed_corpus_case_summary(case, status));
                continue;
            }
        };

        let structural_bytes = match bounded_report_bytes(
            "structural",
            reports
                .structural
                .to_json_bytes()
                .map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                ));
                continue;
            }
        };
        let compatibility_bytes = match bounded_report_bytes(
            "compatibility",
            reports
                .compatibility
                .to_json_bytes()
                .map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                ));
                continue;
            }
        };
        let terminology_bytes = match bounded_report_bytes(
            "terminology",
            reports
                .terminology
                .to_json_bytes()
                .map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                ));
                continue;
            }
        };
        fs::write(evidence_dir.join("structural.json"), structural_bytes)?;
        fs::write(evidence_dir.join("compatibility.json"), compatibility_bytes)?;
        fs::write(evidence_dir.join("terminology.json"), terminology_bytes)?;

        let oracle_report = match oracle::run_changed_report(
            case.package.clone(),
            before.lock_path.clone(),
            before.cache_path.clone(),
            after.lock_path.clone(),
            after.cache_path.clone(),
            oracle_adapter.clone(),
            oracle_java.clone(),
        ) {
            Ok(report) => report,
            Err(error) => {
                failed = true;
                summaries.push(record_oracle_failure(
                    case,
                    &evidence_dir,
                    error.as_ref(),
                )?);
                continue;
            }
        };
        let oracle_bytes = match bounded_report_bytes(
            "oracle",
            oracle_report
                .to_json_bytes()
                .map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                ));
                continue;
            }
        };
        fs::write(evidence_dir.join("oracle.json"), oracle_bytes)?;

        match summarize_corpus_case(case, &reports, &oracle_report) {
            Ok(summary) => summaries.push(summary),
            Err(error) => {
                failed = true;
                write_failure(&evidence_dir, "summary-failure.txt", &error)?;
                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                ));
            }
        }
    }

    let summary = CorpusRunSummary {
        schema: CorpusRunSummary::SCHEMA_V1,
        manifest_sha256,
        cases: summaries,
    };
    let summary_bytes = summary.to_json_bytes()?;
    if summary_bytes.len() > MAX_CORPUS_SUMMARY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "corpus summary is {} bytes; maximum is {}",
                summary_bytes.len(),
                MAX_CORPUS_SUMMARY_BYTES
            ),
        )
        .into());
    }
    fs::write(work_root.join("summary.json"), summary_bytes)?;

    Ok(CorpusExecution { summary, failed })
}

fn resolve_state(
    case: &RealIgCase,
    before: bool,
    work_root: &Path,
) -> Result<ResolvedState, Box<dyn std::error::Error>> {
    let (side, state) = if before {
        ("before", &case.before)
    } else {
        ("after", &case.after)
    };
    let state_root = work_root.join("states").join(&case.id).join(side);
    let cache_path = state_root.join("cache");
    let lock_path = state_root.join("commandf.lock");
    fs::create_dir_all(&state_root)?;

    let request = PackageRequest::parse(&format!("{}@{}", case.package, state.version))?;
    let cache = PackageCache::new(&cache_path);
    let lockfile = Resolver::new(&FhirRegistrySource::new(), &cache).resolve(vec![request])?;
    lockfile.verify_cache(&cache)?;
    fs::write(&lock_path, lockfile.to_bytes()?)?;

    Ok(ResolvedState {
        cache_path,
        lock_path,
        cache,
        lockfile,
    })
}

fn prepare_fresh_work_root(work_root: &Path) -> io::Result<()> {
    if work_root.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "corpus work root already exists; refusing to reuse or delete it: {}",
                work_root.display()
            ),
        ));
    }
    fs::create_dir_all(work_root)
}

fn read_bounded_file(path: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
    let file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(max_bytes + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("input exceeds {max_bytes} byte limit: {}", path.display()),
        ));
    }
    Ok(bytes)
}

fn bounded_report_bytes(label: &str, result: Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {
    let bytes =
        result.map_err(|message| format!("{label} report serialization failed: {message}"))?;
    if bytes.len() > MAX_CORPUS_RAW_REPORT_BYTES {
        return Err(format!(
            "{label} report is {} bytes; maximum is {}",
            bytes.len(),
            MAX_CORPUS_RAW_REPORT_BYTES
        ));
    }
    Ok(bytes)
}

fn evaluation_status(error: &CorpusError) -> CorpusCaseStatus {
    match error {
        CorpusError::Evaluation { stage, .. } => match *stage {
            "structural" => CorpusCaseStatus::StructuralFailed,
            "compatibility" => CorpusCaseStatus::CompatibilityFailed,
            "terminology" => CorpusCaseStatus::TerminologyFailed,
            _ => CorpusCaseStatus::TerminologyFailed,
        },
        CorpusError::CacheVerification { .. }
        | CorpusError::LockedPackageMissing { .. }
        | CorpusError::LockedPackageAmbiguous { .. }
        | CorpusError::LockedPackageDigestMismatch { .. }
        | CorpusError::ArchiveSizeMismatch { .. }
        | CorpusError::ArchiveDigestMismatch { .. } => CorpusCaseStatus::AttestationFailed,
        _ => CorpusCaseStatus::AttestationFailed,
    }
}

fn record_oracle_failure(
    case: &RealIgCase,
    evidence_dir: &Path,
    error: &dyn std::fmt::Display,
) -> io::Result<CorpusCaseSummary> {
    write_failure(evidence_dir, "oracle-failure.txt", error)?;
    Ok(failed_corpus_case_summary(
        case,
        CorpusCaseStatus::OracleFailed,
    ))
}

fn write_resolution_failures(
    evidence_dir: &Path,
    before: Option<Box<dyn std::error::Error>>,
    after: Option<Box<dyn std::error::Error>>,
) -> io::Result<()> {
    let mut text = String::new();
    if let Some(error) = before {
        text.push_str("before: ");
        text.push_str(&bounded_diagnostic(error.as_ref()));
        text.push('\n');
    }
    if let Some(error) = after {
        text.push_str("after: ");
        text.push_str(&bounded_diagnostic(error.as_ref()));
        text.push('\n');
    }
    fs::write(evidence_dir.join("acquisition-failure.txt"), text)
}

fn write_failure(
    evidence_dir: &Path,
    file_name: &str,
    error: &dyn std::fmt::Display,
) -> io::Result<()> {
    write_failure_message(evidence_dir, file_name, &error.to_string())
}

fn write_failure_message(evidence_dir: &Path, file_name: &str, message: &str) -> io::Result<()> {
    let mut text = bounded_text(message);
    text.push('\n');
    fs::write(evidence_dir.join(file_name), text)
}

fn bounded_diagnostic(error: &dyn std::fmt::Display) -> String {
    bounded_text(&error.to_string())
}

fn bounded_text(text: &str) -> String {
    let mut chars = text.chars();
    let mut output = chars
        .by_ref()
        .take(MAX_FAILURE_DIAGNOSTIC_CHARS)
        .collect::<String>();
    if chars.next().is_some() {
        output.push_str("… [diagnostic truncated]");
    }
    output
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use commandf_pkg::{CorpusOracleMode, CorpusPackageState, CorpusRightsMode};

    use super::*;

    fn test_case() -> RealIgCase {
        RealIgCase {
            id: "C001".to_owned(),
            package: "example.package".to_owned(),
            before: CorpusPackageState {
                version: "1.0.0".to_owned(),
                archive_sha256: "a".repeat(64),
                archive_bytes: 1,
                publication_url: "https://example.org/before".to_owned(),
            },
            after: CorpusPackageState {
                version: "2.0.0".to_owned(),
                archive_sha256: "b".repeat(64),
                archive_bytes: 1,
                publication_url: "https://example.org/after".to_owned(),
            },
            fhir_version: "4.0.1".to_owned(),
            publisher: "Example Publisher".to_owned(),
            change_evidence_url: "https://example.org/changes".to_owned(),
            rights_evidence_url: "https://example.org/rights".to_owned(),
            rights_mode: CorpusRightsMode::MetadataOnlyNoRedistribution,
            oracle_mode: CorpusOracleMode::ChangedStructureDefinitionsOnly,
        }
    }

    #[test]
    fn oracle_failure_records_typed_status_and_bounded_evidence() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let evidence_dir = std::env::temp_dir().join(format!(
            "commandf-corpus-oracle-failure-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&evidence_dir).unwrap();

        let error = io::Error::other("deterministic oracle failure");
        let summary = record_oracle_failure(&test_case(), &evidence_dir, &error).unwrap();
        assert_eq!(summary.status, CorpusCaseStatus::OracleFailed);
        assert!(summary.structural.is_none());
        assert!(summary.compatibility.is_none());
        assert!(summary.terminology.is_none());
        assert!(summary.oracle.is_none());

        let evidence = fs::read_to_string(evidence_dir.join("oracle-failure.txt")).unwrap();
        assert_eq!(evidence, "deterministic oracle failure\n");
        fs::remove_dir_all(evidence_dir).unwrap();
    }
}
