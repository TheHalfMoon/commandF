use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};

// Deterministic gzip-compressed GNU tar containing exactly:
// package/package.json = {"name":"example.package","version":"1.0.0","dependencies":{}}
const SYNTHETIC_ARCHIVE: &[u8] = &[
    31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 237, 210, 203, 10, 194, 48, 20, 132, 225, 60, 74, 201, 90,
    98, 2, 181, 133, 190, 77, 104, 15, 82, 181, 105, 104, 84, 132, 210, 119, 55, 94, 64, 112, 45,
    186, 240, 255, 54, 115, 152, 217, 158, 232, 219, 189, 223, 202, 58, 62, 210, 236, 210, 24, 212,
    135, 217, 172, 42, 203, 123, 102, 239, 105, 109, 93, 189, 238, 91, 239, 220, 166, 118, 170, 176,
    234, 11, 78, 233, 232, 167, 162, 80, 127, 106, 214, 193, 15, 162, 27, 45, 23, 63, 196, 131, 152,
    231, 35, 232, 149, 62, 203, 148, 250, 49, 228, 205, 25, 107, 108, 110, 58, 137, 18, 58, 9, 109,
    47, 73, 55, 243, 178, 40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 143, 92, 1, 197, 214,
    21, 225, 0, 40, 0, 0,
];

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("commandf-{label}-{}-{nonce}", std::process::id()))
}

fn write_locked_state(root: &Path) -> (PathBuf, PathBuf) {
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    fs::create_dir_all(root).expect("create state root");
    let cache = PackageCache::new(&cache_path);
    let digest = cache
        .put(SYNTHETIC_ARCHIVE)
        .expect("cache synthetic archive");
    let lockfile = Lockfile::new(
        vec!["example.package@1.0.0".to_owned()],
        vec![LockedPackage {
            name: "example.package".to_owned(),
            version: "1.0.0".to_owned(),
            sha256: digest,
            source: "synthetic-test".to_owned(),
            dependencies: BTreeMap::new(),
        }],
    );
    fs::write(&lock_path, lockfile.to_bytes().expect("serialize lock")).expect("write lock");
    (lock_path, cache_path)
}

fn run_terminology(
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
) -> Output {
    commandf()
        .args([
            "terminology",
            "example.package",
            "--before-lock",
            before_lock.to_str().expect("UTF-8 path"),
            "--before-cache",
            before_cache.to_str().expect("UTF-8 path"),
            "--after-lock",
            after_lock.to_str().expect("UTF-8 path"),
            "--after-cache",
            after_cache.to_str().expect("UTF-8 path"),
            "--format",
            "json",
        ])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("commandf terminology must execute")
}

#[test]
fn terminology_help_exposes_explicit_two_state_inputs() {
    let output = commandf()
        .args(["terminology", "--help"])
        .output()
        .expect("commandf terminology help must execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 help");
    for flag in [
        "--before-lock",
        "--before-cache",
        "--after-lock",
        "--after-cache",
        "--format",
    ] {
        assert!(stdout.contains(flag), "missing {flag}");
    }
}

#[test]
fn terminology_missing_required_paths_is_a_usage_error() {
    let output = commandf()
        .args(["terminology", "example.package"])
        .output()
        .expect("commandf terminology must execute");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn terminology_succeeds_offline_for_self_equivalent_state() {
    let dir = unique_temp_dir("terminology-success");
    let (before_lock, before_cache) = write_locked_state(&dir.join("before"));
    let (after_lock, after_cache) = write_locked_state(&dir.join("after"));
    let output = run_terminology(&before_lock, &before_cache, &after_lock, &after_cache);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 JSON");
    assert!(stdout.contains("\"schema\": 1"));
    assert!(stdout.contains("\"ruleset\": \"cf07-terminology-v1\""));
    assert!(stdout.contains("\"package_name\": \"example.package\""));
    assert!(stdout.contains("\"findings\": []"));
    assert!(stdout.contains("\"code_systems\": []"));
    assert!(stdout.contains("\"value_sets\": []"));
    assert!(stdout.contains("\"binding_refinements\": []"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn terminology_fails_closed_on_corrupted_root_cache() {
    let dir = unique_temp_dir("terminology-corrupt");
    let (before_lock, before_cache) = write_locked_state(&dir.join("before"));
    let (after_lock, after_cache) = write_locked_state(&dir.join("after"));
    let digest = PackageCache::digest(SYNTHETIC_ARCHIVE);
    fs::write(
        after_cache.join("sha256").join(format!("{digest}.tgz")),
        b"corrupt",
    )
    .expect("corrupt after cache");

    let output = run_terminology(&before_lock, &before_cache, &after_lock, &after_cache);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cache object digest mismatch"),
        "stderr: {stderr}"
    );
    let _ = fs::remove_dir_all(&dir);
}
