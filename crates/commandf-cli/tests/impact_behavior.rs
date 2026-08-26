use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache, ResolvedDependency};

const PARENT_A_ARCHIVE: &[u8] = include_bytes!("fixtures/parent-a.tgz");
const PARENT_B_ARCHIVE: &[u8] = include_bytes!("fixtures/parent-b.tgz");
const SHARED_V1_ARCHIVE: &[u8] = include_bytes!("fixtures/shared-v1.tgz");
const SHARED_V2_ARCHIVE: &[u8] = include_bytes!("fixtures/shared-v2.tgz");

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

#[test]
fn impact_help_exposes_pinned_before_after_inputs() {
    let output = commandf()
        .args(["impact", "--help"])
        .output()
        .expect("commandf impact help must execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 help");
    for expected in [
        "<PACKAGE>",
        "--before-lock",
        "--before-cache",
        "--after-lock",
        "--after-cache",
        "--format",
    ] {
        assert!(stdout.contains(expected), "missing {expected}");
    }
}

#[test]
fn impact_is_byte_identical_and_reports_dependency_evidence_without_severity() {
    let root = unique_temp_dir("success");
    let (before_lock, before_cache, after_lock, after_cache) = write_impact_state(&root);

    let first = run_impact(&before_lock, &before_cache, &after_lock, &after_cache);
    let second = run_impact(&before_lock, &before_cache, &after_lock, &after_cache);
    assert_success(&first);
    assert_success(&second);
    assert_eq!(first.stdout, second.stdout);

    let json = String::from_utf8(first.stdout).expect("UTF-8 impact JSON");
    for expected in [
        "\"schema\": 1",
        "\"package_name\": \"acme.subject\"",
        "\"before_evidence\"",
        "\"after_evidence\"",
        "\"seeds\"",
        "\"artifact_impacts\"",
        "\"package_impacts\"",
        "\"unresolved_boundaries\"",
        "\"coverage\"",
    ] {
        assert!(json.contains(expected), "missing {expected}");
    }
    for forbidden in ["\"breaking\"", "\"risky\"", "\"additive\""] {
        assert!(
            !json.contains(forbidden),
            "impact invented severity: {forbidden}"
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn impact_reports_reverse_package_exposure_for_changed_dependency() {
    let root = unique_temp_dir("package-exposure");
    let (before_lock, before_cache, after_lock, after_cache) = write_impact_state(&root);

    let output = run_impact_for(
        "acme.shared",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
    );
    assert_success(&output);

    let json = String::from_utf8(output.stdout).expect("UTF-8 impact JSON");
    assert!(json.contains("\"package_name\": \"acme.shared\""));

    let package_impacts_start = json
        .find("\"package_impacts\": [")
        .expect("package impacts field");
    let unresolved_offset = json[package_impacts_start..]
        .find("\"unresolved_boundaries\":")
        .expect("unresolved boundaries field after package impacts");
    let package_impacts = &json[package_impacts_start..package_impacts_start + unresolved_offset];

    let before_relation = package_impact_relation(package_impacts, "acme.subject", "1.0.0");
    assert!(
        before_relation.contains("\"side\": \"before\""),
        "before dependent relation must retain its side"
    );
    assert!(
        before_relation.contains("\"declared_constraint\": \"1.0.0\""),
        "before dependent relation must retain its exact declared constraint"
    );

    let after_relation = package_impact_relation(package_impacts, "acme.subject", "2.0.0");
    assert!(
        after_relation.contains("\"side\": \"after\""),
        "after dependent relation must retain its side"
    );
    assert!(
        after_relation.contains("\"declared_constraint\": \"2.0.0\""),
        "after dependent relation must retain its exact declared constraint"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn impact_rejects_schema_v1_and_corrupt_cache_without_stdout() {
    let schema_root = unique_temp_dir("schema-v1");
    let (before_lock, before_cache, after_lock, after_cache) = write_impact_state(&schema_root);
    fs::write(
        &before_lock,
        Lockfile::new(Vec::new(), Vec::new()).to_bytes().unwrap(),
    )
    .unwrap();
    let output = run_impact(&before_lock, &before_cache, &after_lock, &after_cache);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires commandf.lock schema 2"));
    let _ = fs::remove_dir_all(schema_root);

    let corrupt_root = unique_temp_dir("corrupt");
    let (before_lock, before_cache, after_lock, after_cache) = write_impact_state(&corrupt_root);
    let lock = Lockfile::from_slice(&fs::read(&before_lock).unwrap()).unwrap();
    let subject = lock
        .packages
        .iter()
        .find(|package| package.name == "acme.subject")
        .unwrap();
    fs::write(
        before_cache
            .join("sha256")
            .join(format!("{}.tgz", subject.sha256)),
        b"corrupted",
    )
    .unwrap();
    let output = run_impact(&before_lock, &before_cache, &after_lock, &after_cache);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let _ = fs::remove_dir_all(corrupt_root);
}

fn package_impact_relation<'a>(
    package_impacts: &'a str,
    impacted_name: &str,
    impacted_version: &str,
) -> &'a str {
    let marker = format!(
        "\"impacted\": {{\n        \"name\": \"{impacted_name}\",\n        \"version\": \"{impacted_version}\""
    );
    let start = package_impacts
        .find(&marker)
        .unwrap_or_else(|| panic!("missing package impact for {impacted_name}@{impacted_version}"));
    let remainder_start = start + marker.len();
    let end = package_impacts[remainder_start..]
        .find("\"impacted\": {")
        .map(|offset| remainder_start + offset)
        .unwrap_or(package_impacts.len());
    &package_impacts[start..end]
}

fn write_impact_state(root: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    fs::create_dir_all(root).unwrap();
    let before_cache_path = root.join("before-cache");
    let after_cache_path = root.join("after-cache");
    let before_lock_path = root.join("before.lock");
    let after_lock_path = root.join("after.lock");
    let before_cache = PackageCache::new(&before_cache_path);
    let after_cache = PackageCache::new(&after_cache_path);

    let before_subject_sha = before_cache.put(PARENT_A_ARCHIVE).unwrap();
    let before_shared_sha = before_cache.put(SHARED_V1_ARCHIVE).unwrap();
    let after_subject_sha = after_cache.put(PARENT_B_ARCHIVE).unwrap();
    let after_shared_sha = after_cache.put(SHARED_V2_ARCHIVE).unwrap();

    let mut before_dependencies = BTreeMap::new();
    before_dependencies.insert("acme.shared".to_owned(), "1.0.0".to_owned());
    let mut after_dependencies = BTreeMap::new();
    after_dependencies.insert("acme.shared".to_owned(), "2.0.0".to_owned());

    let before = Lockfile::new_v2(
        vec!["acme.subject@1.0.0".to_owned()],
        vec![
            locked_package(
                "acme.subject",
                "1.0.0",
                &before_subject_sha,
                before_dependencies,
            ),
            locked_package("acme.shared", "1.0.0", &before_shared_sha, BTreeMap::new()),
        ],
        vec![ResolvedDependency {
            from_name: "acme.subject".to_owned(),
            from_version: "1.0.0".to_owned(),
            to_name: "acme.shared".to_owned(),
            to_version: "1.0.0".to_owned(),
            declared_constraint: "1.0.0".to_owned(),
        }],
    );
    let after = Lockfile::new_v2(
        vec!["acme.subject@2.0.0".to_owned()],
        vec![
            locked_package(
                "acme.subject",
                "2.0.0",
                &after_subject_sha,
                after_dependencies,
            ),
            locked_package("acme.shared", "2.0.0", &after_shared_sha, BTreeMap::new()),
        ],
        vec![ResolvedDependency {
            from_name: "acme.subject".to_owned(),
            from_version: "2.0.0".to_owned(),
            to_name: "acme.shared".to_owned(),
            to_version: "2.0.0".to_owned(),
            declared_constraint: "2.0.0".to_owned(),
        }],
    );
    fs::write(&before_lock_path, before.to_bytes().unwrap()).unwrap();
    fs::write(&after_lock_path, after.to_bytes().unwrap()).unwrap();

    (
        before_lock_path,
        before_cache_path,
        after_lock_path,
        after_cache_path,
    )
}

fn run_impact(
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
) -> Output {
    run_impact_for(
        "acme.subject",
        before_lock,
        before_cache,
        after_lock,
        after_cache,
    )
}

fn run_impact_for(
    package: &str,
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
) -> Output {
    commandf()
        .args([
            "impact",
            package,
            "--before-lock",
            before_lock.to_str().unwrap(),
            "--before-cache",
            before_cache.to_str().unwrap(),
            "--after-lock",
            after_lock.to_str().unwrap(),
            "--after-cache",
            after_cache.to_str().unwrap(),
            "--format",
            "json",
        ])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("commandf impact must execute")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "commandf-impact-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn locked_package(
    name: &str,
    version: &str,
    sha256: &str,
    dependencies: BTreeMap<String, String>,
) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256: sha256.to_owned(),
        source: "synthetic-impact-test".to_owned(),
        dependencies,
    }
}
