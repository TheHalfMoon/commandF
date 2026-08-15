use std::collections::BTreeMap;
use std::io::Cursor;

use commandf_pkg::{
    evaluate_corpus_case, failed_corpus_case_summary, summarize_corpus_case, CorpusCaseStatus,
    CorpusError, CorpusOracleMode, CorpusPackageState, CorpusPackageStateInput, CorpusRightsMode,
    CorpusRunSummary, LockedPackage, Lockfile, OracleDivergenceReport, OracleIdentity,
    OracleResourceResult, OracleResourceStatus, PackageCache, RealIgCase, ResourceKey,
    ResourceKeyKind,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};
use tempfile::TempDir;

fn archive(name: &str, version: &str, patient: &[u8]) -> Vec<u8> {
    let manifest = format!(r#"{{"name":"{name}","version":"{version}","dependencies":{{}}}}"#);
    let entries = [
        ("package/package.json", manifest.as_bytes()),
        ("package/Patient-example.json", patient),
    ];

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        for (path, body) in entries {
            let mut header = Header::new_gnu();
            header.set_path(path).unwrap();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, Cursor::new(body)).unwrap();
        }
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

fn locked(name: &str, version: &str, sha256: &str) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256: sha256.to_owned(),
        source: format!("https://packages.example.org/{name}/{version}"),
        dependencies: BTreeMap::new(),
    }
}

struct Fixture {
    _before_temp: TempDir,
    _after_temp: TempDir,
    before_cache: PackageCache,
    after_cache: PackageCache,
    before_lock: Lockfile,
    after_lock: Lockfile,
    case: RealIgCase,
}

fn fixture() -> Fixture {
    let before_temp = TempDir::new().unwrap();
    let after_temp = TempDir::new().unwrap();
    let before_cache = PackageCache::new(before_temp.path());
    let after_cache = PackageCache::new(after_temp.path());

    let before_bytes = archive(
        "example.pkg",
        "1.0.0",
        br#"{"resourceType":"Patient","id":"example","active":true}"#,
    );
    let after_bytes = archive(
        "example.pkg",
        "2.0.0",
        br#"{"resourceType":"Patient","id":"example","active":false}"#,
    );
    let before_sha = before_cache.put(&before_bytes).unwrap();
    let after_sha = after_cache.put(&after_bytes).unwrap();

    let before_lock = Lockfile::new(
        vec!["example.pkg@1.0.0".to_owned()],
        vec![locked("example.pkg", "1.0.0", &before_sha)],
    );
    let after_lock = Lockfile::new(
        vec!["example.pkg@2.0.0".to_owned()],
        vec![locked("example.pkg", "2.0.0", &after_sha)],
    );

    let case = RealIgCase {
        id: "C001".to_owned(),
        package: "example.pkg".to_owned(),
        before: CorpusPackageState {
            version: "1.0.0".to_owned(),
            archive_sha256: before_sha,
            archive_bytes: before_bytes.len() as u64,
            publication_url: "https://example.org/before".to_owned(),
        },
        after: CorpusPackageState {
            version: "2.0.0".to_owned(),
            archive_sha256: after_sha,
            archive_bytes: after_bytes.len() as u64,
            publication_url: "https://example.org/after".to_owned(),
        },
        fhir_version: "4.0.1".to_owned(),
        publisher: "Example Publisher".to_owned(),
        change_evidence_url: "https://example.org/changes".to_owned(),
        rights_evidence_url: "https://example.org/rights".to_owned(),
        rights_mode: CorpusRightsMode::MetadataOnlyNoRedistribution,
        oracle_mode: CorpusOracleMode::ChangedStructureDefinitionsOnly,
    };

    Fixture {
        _before_temp: before_temp,
        _after_temp: after_temp,
        before_cache,
        after_cache,
        before_lock,
        after_lock,
        case,
    }
}

#[test]
fn evaluator_reuses_canonical_reports_and_summary_hashes_them() {
    let fixture = fixture();
    let reports = evaluate_corpus_case(
        &fixture.case,
        CorpusPackageStateInput {
            lockfile: &fixture.before_lock,
            cache: &fixture.before_cache,
        },
        CorpusPackageStateInput {
            lockfile: &fixture.after_lock,
            cache: &fixture.after_cache,
        },
    )
    .unwrap();

    assert!(!reports.structural.changes.is_empty());
    assert_eq!(reports.compatibility.package_name, fixture.case.package);
    assert_eq!(reports.terminology.package_name, fixture.case.package);
    assert!(reports.terminology.code_systems.is_empty());
    assert!(reports.terminology.value_sets.is_empty());
    assert!(reports.terminology.binding_refinements.is_empty());

    let oracle = OracleDivergenceReport {
        schema: OracleDivergenceReport::SCHEMA_V1,
        oracle: OracleIdentity::pinned_hl7(),
        package_name: fixture.case.package.clone(),
        structural_diff: reports.structural.clone(),
        resources: vec![OracleResourceResult {
            resource: ResourceKey {
                kind: ResourceKeyKind::ResourceId,
                value: "Patient/example".to_owned(),
            },
            status: OracleResourceStatus::Uncomparable,
            oracle: None,
            commandf_change_kinds: Vec::new(),
        }],
    };
    let summary = summarize_corpus_case(
        &fixture.case,
        &reports,
        &oracle,
        &fixture.before_lock,
        &fixture.after_lock,
    )
    .unwrap();

    assert_eq!(summary.status, CorpusCaseStatus::Complete);
    assert_eq!(
        summary.structural.as_ref().unwrap().changes,
        reports.structural.changes.len()
    );
    assert_eq!(
        summary.structural.as_ref().unwrap().report_sha256,
        PackageCache::digest(&reports.structural.to_json_bytes().unwrap())
    );
    assert_eq!(
        summary.compatibility.as_ref().unwrap().report_sha256,
        PackageCache::digest(&reports.compatibility.to_json_bytes().unwrap())
    );
    assert_eq!(
        summary.terminology.as_ref().unwrap().report_sha256,
        PackageCache::digest(&reports.terminology.to_json_bytes().unwrap())
    );
    assert_eq!(
        summary.oracle.as_ref().unwrap().report_sha256,
        PackageCache::digest(&oracle.to_json_bytes().unwrap())
    );
    assert_eq!(summary.oracle.as_ref().unwrap().compared, 0);
    assert_eq!(summary.oracle.as_ref().unwrap().uncomparable, 1);
    let before_closure = summary
        .before
        .closure
        .as_ref()
        .expect("before closure evidence");
    assert_eq!(before_closure.len(), fixture.before_lock.packages.len());
    assert_eq!(before_closure[0].name, "example.pkg");
    assert_eq!(before_closure[0].sha256, fixture.case.before.archive_sha256);
    assert_eq!(
        summary.before.closure_sha256.as_deref().map(str::len),
        Some(64)
    );

    let mut transport_changed = fixture.before_lock.clone();
    transport_changed.packages[0].source =
        "https://fallback.example.org/example.pkg/1.0.0".to_owned();
    let transport_summary = summarize_corpus_case(
        &fixture.case,
        &reports,
        &oracle,
        &transport_changed,
        &fixture.after_lock,
    )
    .unwrap();
    assert_eq!(
        transport_summary.before.closure_sha256,
        summary.before.closure_sha256
    );

    let mut dependency_digest_changed = fixture.before_lock.clone();
    dependency_digest_changed
        .packages
        .push(locked("example.dep", "1.0.0", &"c".repeat(64)));
    dependency_digest_changed.packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
    });
    let changed_summary = summarize_corpus_case(
        &fixture.case,
        &reports,
        &oracle,
        &dependency_digest_changed,
        &fixture.after_lock,
    )
    .unwrap();
    assert_ne!(
        changed_summary.before.closure_sha256,
        summary.before.closure_sha256
    );
}

#[test]
fn summary_rejects_cross_case_oracle_identity() {
    let fixture = fixture();
    let reports = evaluate_corpus_case(
        &fixture.case,
        CorpusPackageStateInput {
            lockfile: &fixture.before_lock,
            cache: &fixture.before_cache,
        },
        CorpusPackageStateInput {
            lockfile: &fixture.after_lock,
            cache: &fixture.after_cache,
        },
    )
    .unwrap();
    let oracle = OracleDivergenceReport {
        schema: OracleDivergenceReport::SCHEMA_V1,
        oracle: OracleIdentity::pinned_hl7(),
        package_name: "other.pkg".to_owned(),
        structural_diff: reports.structural.clone(),
        resources: Vec::new(),
    };

    assert_eq!(
        summarize_corpus_case(
            &fixture.case,
            &reports,
            &oracle,
            &fixture.before_lock,
            &fixture.after_lock,
        ),
        Err(CorpusError::ReportIdentityMismatch {
            case_id: "C001".to_owned(),
            report: "oracle",
        })
    );
}

#[test]
fn deterministic_run_summary_has_no_failure_detail_or_paths() {
    let fixture = fixture();
    let failed = failed_corpus_case_summary(&fixture.case, CorpusCaseStatus::OracleFailed);
    assert!(failed.structural.is_none());
    assert!(failed.compatibility.is_none());
    assert!(failed.terminology.is_none());
    assert!(failed.oracle.is_none());
    assert!(failed.before.closure.is_none());
    assert!(failed.after.closure.is_none());

    let report = CorpusRunSummary {
        schema: CorpusRunSummary::SCHEMA_V1,
        manifest_sha256: "a".repeat(64),
        cases: vec![failed],
    };
    let first = report.to_json_bytes().unwrap();
    let second = report.to_json_bytes().unwrap();
    assert_eq!(first, second);

    let text = String::from_utf8(first).unwrap();
    assert!(text.contains("oracle_failed"));
    assert!(!text.contains("/tmp"));
    assert!(!text.contains("\\\\"));
}
