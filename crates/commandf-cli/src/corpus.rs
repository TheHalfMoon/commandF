use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use commandf_pkg::{
    evaluate_corpus_case, failed_corpus_case_summary, parse_corpus_manifest, summarize_corpus_case,
    CorpusCaseStatus, CorpusError, CorpusPackageStateInput, CorpusRunSummary, FhirRegistrySource,
    Lockfile, PackageCache, PackageRequest, RealIgCase, Resolver, MAX_CORPUS_MANIFEST_BYTES,
};

use crate::oracle;

const MAX_FAILURE_DIAGNOSTIC_CHARS: usize = 16_384;

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

        fs::write(
            evidence_dir.join("structural.json"),
            reports.structural.to_json_bytes()?,
        )?;
        fs::write(
            evidence_dir.join("compatibility.json"),
            reports.compatibility.to_json_bytes()?,
        )?;
        fs::write(
            evidence_dir.join("terminology.json"),
            reports.terminology.to_json_bytes()?,
        )?;

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
                write_failure(&evidence_dir, "oracle-failure.txt", error.as_ref())?;
                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::OracleFailed,
                ));
                continue;
            }
        };
        fs::write(
            evidence_dir.join("oracle.json"),
            oracle_report.to_json_bytes()?,
        )?;

        match summarize_corpus_case(case, &reports, &oracle_report) {
            Ok(summary) => summaries.push(summary),
            Err(error) => {
                failed = true;
                write_failure(&evidence_dir, "summary-failure.txt", &error)?;
                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::OracleFailed,
                ));
            }
        }
    }

    let summary = CorpusRunSummary {
        schema: CorpusRunSummary::SCHEMA_V1,
        manifest_sha256,
        cases: summaries,
    };
    fs::write(work_root.join("summary.json"), summary.to_json_bytes()?)?;

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
    let mut text = bounded_diagnostic(error);
    text.push('\n');
    fs::write(evidence_dir.join(file_name), text)
}

fn bounded_diagnostic(error: &dyn std::fmt::Display) -> String {
    let text = error.to_string();
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
