use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache, ResolvedDependency};

const PARENT_A_ARCHIVE: &[u8] = include_bytes!("fixtures/parent-a.tgz");
const PARENT_B_ARCHIVE: &[u8] = include_bytes!("fixtures/parent-b.tgz");
const SHARED_V1_ARCHIVE: &[u8] = include_bytes!("fixtures/shared-v1.tgz");
const SHARED_V2_ARCHIVE: &[u8] = include_bytes!("fixtures/shared-v2.tgz");

#[test]
fn impact_cli_output_is_byte_identical_and_reports_sha256() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).unwrap();
    let (before_lock, before_cache, after_lock, after_cache) = write_state(&root);

    let first = run_impact(&before_lock, &before_cache, &after_lock, &after_cache);
    let second = run_impact(&before_lock, &before_cache, &after_lock, &after_cache);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
    println!("CF12_IMPACT_SHA256={}", PackageCache::digest(&first.stdout));
    let _ = fs::remove_dir_all(root);
}

fn write_state(root: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
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
) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
        .args([
            "impact",
            "acme.subject",
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
        .unwrap()
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "commandf-impact-proof-{}-{nonce}",
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
        source: "synthetic-impact-proof".to_owned(),
        dependencies,
    }
}
