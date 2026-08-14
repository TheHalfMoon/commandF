use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};

const SYNTHETIC_ARCHIVE: &[u8] = &[
    31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 237, 210, 205, 106, 2, 49, 20, 134, 225, 92, 74,
    201, 90, 38, 9, 254, 44, 188, 141, 22, 247, 97, 60, 216, 177, 83, 103, 200, 207, 160,
    136, 247, 238, 81, 208, 69, 215, 210, 141, 239, 179, 57, 201, 151, 156, 4, 66, 198,
    216, 254, 196, 157, 184, 41, 246, 85, 154, 125, 30, 14, 230, 229, 188, 90, 45, 22,
    247, 170, 254, 86, 31, 150, 171, 231, 248, 158, 135, 48, 95, 6, 243, 225, 205, 63,
    168, 185, 196, 164, 215, 155, 247, 116, 182, 221, 214, 174, 237, 209, 206, 108, 146,
    60, 212, 212, 202, 215, 105, 20, 141, 54, 183, 31, 241, 41, 69, 87, 244, 137, 74,
    205, 154, 197, 182, 116, 147, 104, 82, 83, 175, 211, 239, 82, 198, 181, 115, 114,
    140, 191, 99, 47, 205, 144, 118, 238, 209, 229, 110, 39, 78, 146, 114, 55, 28, 116,
    103, 104, 124, 227, 237, 197, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 94, 228,
    10, 103, 64, 75, 71, 0, 40, 0, 0,
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

fn write_empty_lock(path: &Path) {
    fs::create_dir_all(path.parent().expect("lock parent")).expect("create lock parent");
    fs::write(
        path,
        b"{\n  \"schema\": 1,\n  \"roots\": [],\n  \"packages\": []\n}\n",
    )
    .expect("write empty lock");
}

fn write_locked_state(root: &Path) -> (PathBuf, PathBuf) {
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    fs::create_dir_all(root).expect("create state root");
    let cache = PackageCache::new(&cache_path);
    let digest = cache.put(SYNTHETIC_ARCHIVE).expect("cache synthetic archive");
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

fn run_diff(
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
) -> std::process::Output {
    commandf()
        .args([
            "diff",
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
        .expect("commandf diff must execute")
}

#[test]
fn missing_subcommand_is_a_usage_error() {
    let output = commandf().output().expect("commandf must execute");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn missing_lockfile_is_a_verification_failure() {
    let dir = unique_temp_dir("missing-lock");
    let lock = dir.join("missing.lock");
    let cache = dir.join("cache");

    let output = commandf()
        .args([
            "pkg",
            "verify",
            "--cache",
            cache.to_str().expect("UTF-8 temp path"),
            "--lock",
            lock.to_str().expect("UTF-8 temp path"),
        ])
        .output()
        .expect("commandf must execute");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn valid_empty_lockfile_verifies_successfully() {
    let dir = unique_temp_dir("empty-lock");
    let lock = dir.join("commandf.lock");
    let cache = dir.join("cache");
    fs::create_dir_all(&cache).expect("create cache directory");
    write_empty_lock(&lock);

    let output = commandf()
        .args([
            "pkg",
            "verify",
            "--cache",
            cache.to_str().expect("UTF-8 temp path"),
            "--lock",
            lock.to_str().expect("UTF-8 temp path"),
        ])
        .output()
        .expect("commandf must execute");

    let _ = fs::remove_dir_all(&dir);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn diff_help_exposes_explicit_two_state_inputs() {
    let output = commandf()
        .args(["diff", "--help"])
        .output()
        .expect("commandf diff help must execute");
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
fn diff_missing_required_paths_is_a_usage_error() {
    let output = commandf()
        .args(["diff", "example.package"])
        .output()
        .expect("commandf diff must execute");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn diff_fails_when_package_is_absent_from_before_or_after_lock() {
    let dir = unique_temp_dir("diff-absent");
    let (valid_lock, valid_cache) = write_locked_state(&dir.join("valid"));
    let empty_lock = dir.join("empty/commandf.lock");
    let empty_cache = dir.join("empty/cache");
    fs::create_dir_all(&empty_cache).expect("create empty cache");
    write_empty_lock(&empty_lock);

    let before_absent = run_diff(&empty_lock, &empty_cache, &valid_lock, &valid_cache);
    assert_eq!(before_absent.status.code(), Some(1));
    let after_absent = run_diff(&valid_lock, &valid_cache, &empty_lock, &empty_cache);
    assert_eq!(after_absent.status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn diff_fails_closed_on_corrupted_before_or_after_cache() {
    let dir = unique_temp_dir("diff-corrupt");
    let (before_lock, before_cache) = write_locked_state(&dir.join("before"));
    let (after_lock, after_cache) = write_locked_state(&dir.join("after"));
    let digest = PackageCache::digest(SYNTHETIC_ARCHIVE);

    fs::write(
        before_cache.join("sha256").join(format!("{digest}.tgz")),
        b"corrupt",
    )
    .expect("corrupt before cache");
    let before_corrupt = run_diff(&before_lock, &before_cache, &after_lock, &after_cache);
    assert_eq!(before_corrupt.status.code(), Some(1));

    let (fresh_before_lock, fresh_before_cache) = write_locked_state(&dir.join("before-fresh"));
    fs::write(
        after_cache.join("sha256").join(format!("{digest}.tgz")),
        b"corrupt",
    )
    .expect("corrupt after cache");
    let after_corrupt = run_diff(
        &fresh_before_lock,
        &fresh_before_cache,
        &after_lock,
        &after_cache,
    );
    assert_eq!(after_corrupt.status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn diff_succeeds_offline_and_emits_schema_v1_json() {
    let dir = unique_temp_dir("diff-success");
    let (before_lock, before_cache) = write_locked_state(&dir.join("before"));
    let (after_lock, after_cache) = write_locked_state(&dir.join("after"));
    let output = run_diff(&before_lock, &before_cache, &after_lock, &after_cache);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 JSON");
    assert!(stdout.starts_with('{'));
    assert!(stdout.ends_with("}\n"));
    assert!(stdout.contains("\"schema\": 1"));
    assert!(stdout.contains("\"package_name\": \"example.package\""));
    assert!(stdout.contains("\"changes\": []"));
    let _ = fs::remove_dir_all(&dir);
}
