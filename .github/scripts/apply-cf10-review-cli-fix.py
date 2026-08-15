from pathlib import Path

path = Path('crates/commandf-cli/src/corpus.rs')
text = path.read_text()

old = '''use commandf_pkg::{
    evaluate_corpus_case, failed_corpus_case_summary, parse_corpus_manifest, summarize_corpus_case,
    CorpusCaseStatus, CorpusCaseSummary, CorpusError, CorpusPackageStateInput, CorpusRunSummary,
    FhirRegistrySource, Lockfile, PackageCache, PackageRequest, RealIgCase, Resolver,
    MAX_CORPUS_MANIFEST_BYTES,
};'''
new = '''use commandf_pkg::{
    evaluate_corpus_compatibility, evaluate_corpus_structural, evaluate_corpus_terminology,
    failed_corpus_case_summary, failed_corpus_case_summary_with_closure, parse_corpus_manifest,
    summarize_corpus_case, CorpusCaseReports, CorpusCaseStatus, CorpusCaseSummary, CorpusError,
    CorpusPackageStateInput, CorpusRunSummary, FhirRegistrySource, Lockfile, PackageCache,
    PackageRequest, RealIgCase, Resolver, MAX_CORPUS_MANIFEST_BYTES,
};'''
if text.count(old) != 1:
    raise SystemExit('import block mismatch')
text = text.replace(old, new, 1)

old = '''const MAX_CORPUS_RAW_REPORT_BYTES: usize = 64 * 1024 * 1024;
const MAX_CORPUS_SUMMARY_BYTES: usize = 1024 * 1024;'''
new = '''const MAX_CORPUS_RAW_REPORT_BYTES: usize = 64 * 1024 * 1024;
const MAX_CORPUS_LOCK_BYTES: usize = 4 * 1024 * 1024;
const MAX_CORPUS_SUMMARY_BYTES: usize = 1024 * 1024;'''
if text.count(old) != 1:
    raise SystemExit('constant block mismatch')
text = text.replace(old, new, 1)

start_marker = '''        let reports = match evaluate_corpus_case('''
end_marker = '''        let oracle_report = match oracle::run_changed_report('''
start = text.index(start_marker)
end = text.index(end_marker, start)
replacement = '''        let before_lock_bytes = match bounded_lock_bytes(
            "before",
            before.lockfile.to_bytes().map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_summary_with_closure(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        let after_lock_bytes = match bounded_lock_bytes(
            "after",
            after.lockfile.to_bytes().map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_summary_with_closure(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        fs::write(evidence_dir.join("before.commandf.lock"), before_lock_bytes)?;
        fs::write(evidence_dir.join("after.commandf.lock"), after_lock_bytes)?;

        let before_input = CorpusPackageStateInput {
            lockfile: &before.lockfile,
            cache: &before.cache,
        };
        let after_input = CorpusPackageStateInput {
            lockfile: &after.lockfile,
            cache: &after.cache,
        };

        let structural = match evaluate_corpus_structural(case, before_input, after_input) {
            Ok(report) => report,
            Err(error) => {
                failed = true;
                let status = evaluation_status(&error);
                write_failure(&evidence_dir, "evaluation-failure.txt", &error)?;
                summaries.push(failed_summary_with_closure(
                    case,
                    status,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        let structural_bytes = match bounded_report_bytes(
            "structural",
            structural.to_json_bytes().map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_summary_with_closure(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        fs::write(evidence_dir.join("structural.json"), structural_bytes)?;

        let compatibility = match evaluate_corpus_compatibility(case, &structural) {
            Ok(report) => report,
            Err(error) => {
                failed = true;
                let status = evaluation_status(&error);
                write_failure(&evidence_dir, "evaluation-failure.txt", &error)?;
                summaries.push(failed_summary_with_closure(
                    case,
                    status,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        let compatibility_bytes = match bounded_report_bytes(
            "compatibility",
            compatibility
                .to_json_bytes()
                .map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_summary_with_closure(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        fs::write(evidence_dir.join("compatibility.json"), compatibility_bytes)?;

        let terminology = match evaluate_corpus_terminology(
            case,
            before_input,
            after_input,
            &structural,
            &compatibility,
        ) {
            Ok(report) => report,
            Err(error) => {
                failed = true;
                let status = evaluation_status(&error);
                write_failure(&evidence_dir, "evaluation-failure.txt", &error)?;
                summaries.push(failed_summary_with_closure(
                    case,
                    status,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        let terminology_bytes = match bounded_report_bytes(
            "terminology",
            terminology.to_json_bytes().map_err(|error| error.to_string()),
        ) {
            Ok(bytes) => bytes,
            Err(message) => {
                failed = true;
                write_failure_message(&evidence_dir, "evidence-failure.txt", &message)?;
                summaries.push(failed_summary_with_closure(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        fs::write(evidence_dir.join("terminology.json"), terminology_bytes)?;

        let reports = CorpusCaseReports {
            structural,
            compatibility,
            terminology,
        };

'''
text = text[:start] + replacement + text[end:]

old = '''                summaries.push(record_oracle_failure(case, &evidence_dir, error.as_ref())?);'''
new = '''                summaries.push(record_oracle_failure(
                    case,
                    &evidence_dir,
                    error.as_ref(),
                    &before.lockfile,
                    &after.lockfile,
                )?);'''
if text.count(old) != 1:
    raise SystemExit('oracle failure call mismatch')
text = text.replace(old, new, 1)

old = '''                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                ));
                continue;
            }
        };
        fs::write(evidence_dir.join("oracle.json"), oracle_bytes)?;

        match summarize_corpus_case(case, &reports, &oracle_report) {'''
new = '''                summaries.push(failed_summary_with_closure(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                    &before.lockfile,
                    &after.lockfile,
                ));
                continue;
            }
        };
        fs::write(evidence_dir.join("oracle.json"), oracle_bytes)?;

        match summarize_corpus_case(
            case,
            &reports,
            &oracle_report,
            &before.lockfile,
            &after.lockfile,
        ) {'''
if text.count(old) != 1:
    raise SystemExit('oracle evidence/summarize block mismatch')
text = text.replace(old, new, 1)

old = '''                summaries.push(failed_corpus_case_summary(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                ));
            }
        }
    }
'''
new = '''                summaries.push(failed_summary_with_closure(
                    case,
                    CorpusCaseStatus::EvidenceFailed,
                    &before.lockfile,
                    &after.lockfile,
                ));
            }
        }
    }
'''
# Only replace the last post-summary failure occurrence; earlier identical blocks were replaced above.
idx = text.rfind(old)
if idx == -1:
    raise SystemExit('summary failure block mismatch')
text = text[:idx] + new + text[idx + len(old):]

old = '''fn bounded_report_bytes(label: &str, result: Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {'''
new = '''fn bounded_lock_bytes(label: &str, result: Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {
    let bytes = result.map_err(|message| format!("{label} lock serialization failed: {message}"))?;
    if bytes.len() > MAX_CORPUS_LOCK_BYTES {
        return Err(format!(
            "{label} lock is {} bytes; maximum is {}",
            bytes.len(),
            MAX_CORPUS_LOCK_BYTES
        ));
    }
    Ok(bytes)
}

fn bounded_report_bytes(label: &str, result: Result<Vec<u8>, String>) -> Result<Vec<u8>, String> {'''
if text.count(old) != 1:
    raise SystemExit('bounded report marker mismatch')
text = text.replace(old, new, 1)

old = '''fn record_oracle_failure(
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
'''
new = '''fn failed_summary_with_closure(
    case: &RealIgCase,
    status: CorpusCaseStatus,
    before_lockfile: &Lockfile,
    after_lockfile: &Lockfile,
) -> CorpusCaseSummary {
    failed_corpus_case_summary_with_closure(case, status, before_lockfile, after_lockfile)
        .unwrap_or_else(|_| failed_corpus_case_summary(case, CorpusCaseStatus::EvidenceFailed))
}

fn record_oracle_failure(
    case: &RealIgCase,
    evidence_dir: &Path,
    error: &dyn std::fmt::Display,
    before_lockfile: &Lockfile,
    after_lockfile: &Lockfile,
) -> io::Result<CorpusCaseSummary> {
    write_failure(evidence_dir, "oracle-failure.txt", error)?;
    Ok(failed_summary_with_closure(
        case,
        CorpusCaseStatus::OracleFailed,
        before_lockfile,
        after_lockfile,
    ))
}
'''
if text.count(old) != 1:
    raise SystemExit('record oracle failure block mismatch')
text = text.replace(old, new, 1)

# Update the unit test to pass deterministic lock closure evidence.
old = '''        let error = io::Error::other("deterministic oracle failure");
        let summary = record_oracle_failure(&test_case(), &evidence_dir, &error).unwrap();'''
new = '''        let case = test_case();
        let before_lock = Lockfile::new(
            vec!["example.package@1.0.0".to_owned()],
            vec![commandf_pkg::LockedPackage {
                name: "example.package".to_owned(),
                version: "1.0.0".to_owned(),
                sha256: case.before.archive_sha256.clone(),
                source: "https://example.org/before.tgz".to_owned(),
                dependencies: std::collections::BTreeMap::new(),
            }],
        );
        let after_lock = Lockfile::new(
            vec!["example.package@2.0.0".to_owned()],
            vec![commandf_pkg::LockedPackage {
                name: "example.package".to_owned(),
                version: "2.0.0".to_owned(),
                sha256: case.after.archive_sha256.clone(),
                source: "https://example.org/after.tgz".to_owned(),
                dependencies: std::collections::BTreeMap::new(),
            }],
        );
        let error = io::Error::other("deterministic oracle failure");
        let summary = record_oracle_failure(
            &case,
            &evidence_dir,
            &error,
            &before_lock,
            &after_lock,
        )
        .unwrap();'''
if text.count(old) != 1:
    raise SystemExit('oracle failure test call mismatch')
text = text.replace(old, new, 1)

old = '''        assert!(summary.oracle.is_none());

        let evidence = fs::read_to_string(evidence_dir.join("oracle-failure.txt")).unwrap();'''
new = '''        assert!(summary.oracle.is_none());
        assert!(summary.before.closure.is_some());
        assert!(summary.after.closure.is_some());

        let evidence = fs::read_to_string(evidence_dir.join("oracle-failure.txt")).unwrap();'''
if text.count(old) != 1:
    raise SystemExit('oracle failure test assertion mismatch')
text = text.replace(old, new, 1)

path.write_text(text)
print('CF-10 staged evidence CLI fixes applied')
