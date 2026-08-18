use std::collections::BTreeMap;
use std::fs;

use commandf_pkg::{
    attest_corpus_package_state, CorpusError, CorpusOracleMode, CorpusPackageSide,
    CorpusPackageState, CorpusRightsMode, LockedPackage, Lockfile, PackageCache, RealIgCase,
};
use tempfile::TempDir;

fn package_state(version: &str, digest: String, bytes: u64, label: &str) -> CorpusPackageState {
    CorpusPackageState {
        version: version.to_owned(),
        archive_sha256: digest,
        archive_bytes: bytes,
        publication_url: format!("https://example.org/{label}"),
    }
}

fn case_with_states(before: CorpusPackageState, after: CorpusPackageState) -> RealIgCase {
    RealIgCase {
        id: "C001".to_owned(),
        package: "acme.root".to_owned(),
        before,
        after,
        fhir_version: "4.0.1".to_owned(),
        publisher: "Example Publisher".to_owned(),
        change_evidence_url: "https://example.org/changes".to_owned(),
        rights_evidence_url: "https://example.org/rights".to_owned(),
        rights_mode: CorpusRightsMode::MetadataOnlyNoRedistribution,
        oracle_mode: CorpusOracleMode::ChangedStructureDefinitionsOnly,
    }
}

fn locked(name: &str, version: &str, digest: &str) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256: digest.to_owned(),
        source: format!("https://registry.example.org/{name}/{version}"),
        dependencies: BTreeMap::new(),
    }
}

fn fixture() -> (TempDir, PackageCache, RealIgCase, Lockfile) {
    let temp = TempDir::new().expect("tempdir");
    let cache = PackageCache::new(temp.path());
    let before_bytes = b"before-package-bytes";
    let after_bytes = b"after-package-bytes";
    let before_digest = cache.put(before_bytes).expect("cache before");
    let after_digest = cache.put(after_bytes).expect("cache after");
    let case = case_with_states(
        package_state(
            "1.0.0",
            before_digest.clone(),
            before_bytes.len() as u64,
            "before",
        ),
        package_state(
            "2.0.0",
            after_digest.clone(),
            after_bytes.len() as u64,
            "after",
        ),
    );
    let lockfile = Lockfile::new(
        vec!["acme.root@1.0.0".to_owned(), "acme.root@2.0.0".to_owned()],
        vec![
            locked("acme.root", "1.0.0", &before_digest),
            locked("acme.root", "2.0.0", &after_digest),
        ],
    );
    (temp, cache, case, lockfile)
}

#[test]
fn matching_before_and_after_states_attest() {
    let (_temp, cache, case, lockfile) = fixture();

    let before = attest_corpus_package_state(&case, CorpusPackageSide::Before, &lockfile, &cache)
        .expect("before should attest");
    assert_eq!(before.case_id, "C001");
    assert_eq!(before.package, "acme.root");
    assert_eq!(before.side, CorpusPackageSide::Before);
    assert_eq!(before.version, "1.0.0");
    assert_eq!(before.sha256, case.before.archive_sha256);
    assert_eq!(before.archive_bytes, case.before.archive_bytes);

    let after = attest_corpus_package_state(&case, CorpusPackageSide::After, &lockfile, &cache)
        .expect("after should attest");
    assert_eq!(after.side, CorpusPackageSide::After);
    assert_eq!(after.version, "2.0.0");
    assert_eq!(after.sha256, case.after.archive_sha256);
    assert_eq!(after.archive_bytes, case.after.archive_bytes);
}

#[test]
fn manifest_digest_mismatch_fails_closed() {
    let (_temp, cache, mut case, lockfile) = fixture();
    case.before.archive_sha256 = "0".repeat(64);

    assert!(matches!(
        attest_corpus_package_state(&case, CorpusPackageSide::Before, &lockfile, &cache),
        Err(CorpusError::LockedPackageDigestMismatch { .. })
    ));
}

#[test]
fn manifest_size_mismatch_fails_closed() {
    let (_temp, cache, mut case, lockfile) = fixture();
    case.before.archive_bytes += 1;

    assert!(matches!(
        attest_corpus_package_state(&case, CorpusPackageSide::Before, &lockfile, &cache),
        Err(CorpusError::ArchiveSizeMismatch { .. })
    ));
}

#[test]
fn corrupted_target_cache_fails_during_mandatory_cf01_verification() {
    let (temp, cache, case, lockfile) = fixture();
    let path = temp
        .path()
        .join("sha256")
        .join(format!("{}.tgz", case.before.archive_sha256));
    fs::write(path, b"corrupted").expect("corrupt cache object");

    assert!(matches!(
        attest_corpus_package_state(&case, CorpusPackageSide::Before, &lockfile, &cache),
        Err(CorpusError::CacheVerification { .. })
    ));
}

#[test]
fn missing_exact_locked_state_fails_closed() {
    let (_temp, cache, case, mut lockfile) = fixture();
    lockfile
        .packages
        .retain(|package| package.version != case.before.version);

    assert!(matches!(
        attest_corpus_package_state(&case, CorpusPackageSide::Before, &lockfile, &cache),
        Err(CorpusError::LockedPackageMissing { .. })
    ));
}

#[test]
fn duplicate_exact_locked_state_fails_closed() {
    let (_temp, cache, case, mut lockfile) = fixture();
    let duplicate = lockfile
        .packages
        .iter()
        .find(|package| package.version == case.before.version)
        .expect("before locked package")
        .clone();
    lockfile.packages.push(duplicate);

    assert!(matches!(
        attest_corpus_package_state(&case, CorpusPackageSide::Before, &lockfile, &cache),
        Err(CorpusError::LockedPackageAmbiguous { .. })
    ));
}

#[test]
fn unrelated_unverified_lock_entry_blocks_attestation() {
    let (_temp, cache, case, mut lockfile) = fixture();
    lockfile
        .packages
        .push(locked("acme.unrelated", "1.0.0", &"f".repeat(64)));

    assert!(matches!(
        attest_corpus_package_state(&case, CorpusPackageSide::Before, &lockfile, &cache),
        Err(CorpusError::CacheVerification { .. })
    ));
}
