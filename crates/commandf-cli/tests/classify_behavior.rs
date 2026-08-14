use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};

const SYNTHETIC_ARCHIVE: &[u8] = &[
    31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 237, 210, 205, 106, 2, 49, 20, 134, 225, 92, 74, 201, 90,
    38, 9, 254, 44, 188, 141, 22, 247, 97, 60, 216, 177, 83, 103, 200, 207, 160, 136, 247, 238, 81,
    208, 69, 215, 210, 141, 239, 179, 57, 201, 151, 156, 4, 66, 198, 216, 254, 196, 157, 184, 41,
    246, 85, 154, 125, 30, 14, 230, 229, 188, 90, 45, 22, 247, 170, 254, 86, 31, 150, 171, 231,
    248, 158, 135, 48, 95, 6, 243, 225, 205, 63, 168, 185, 196, 164, 215, 155, 247, 116, 182, 221,
    214, 174, 237, 209, 206, 108, 146, 60, 212, 212, 202, 215, 105, 20, 141, 54, 183, 31, 241, 41,
    69, 87, 244, 137, 74, 205, 154, 197, 182, 116, 147, 104, 82, 83, 175, 211, 239, 82, 198, 181,
    115, 114, 140, 191, 99, 47, 205, 144, 118, 238, 209, 229, 110, 39, 78, 146, 114, 55, 28, 116,
    103, 104, 124, 227, 237, 197, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 94, 228, 10, 103, 64,
    75, 71, 0, 40, 0, 0,
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

fn run_classify(
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
) -> Output {
    commandf()
        .args([
            "classify",
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
        .expect("commandf classify must execute")
}

#[test]
fn classify_help_exposes_explicit_two_state_inputs() {
    let output = commandf()
        .args(["classify", "--help"])
        .output()
        .expect("commandf classify help must execute");
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
fn classify_succeeds_offline_and_self_equivalent_state_has_no_findings() {
    let dir = unique_temp_dir("classify-success");
    let (before_lock, before_cache) = write_locked_state(&dir.join("before"));
    let (after_lock, after_cache) = write_locked_state(&dir.join("after"));
    let output = run_classify(&before_lock, &before_cache, &after_lock, &after_cache);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 JSON");
    assert!(stdout.contains("\"schema\": 1"));
    assert!(stdout.contains("\"ruleset\": \"cf04-rules-v1\""));
    assert!(stdout.contains("\"package_name\": \"example.package\""));
    assert!(stdout.contains("\"findings\": []"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn classify_fails_closed_on_corrupted_cache() {
    let dir = unique_temp_dir("classify-corrupt");
    let (before_lock, before_cache) = write_locked_state(&dir.join("before"));
    let (after_lock, after_cache) = write_locked_state(&dir.join("after"));
    let digest = PackageCache::digest(SYNTHETIC_ARCHIVE);
    fs::write(
        after_cache.join("sha256").join(format!("{digest}.tgz")),
        b"corrupt",
    )
    .expect("corrupt after cache");

    let output = run_classify(&before_lock, &before_cache, &after_lock, &after_cache);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cache object digest mismatch"),
        "stderr: {stderr}"
    );
    assert!(
        stderr.contains(&digest),
        "stderr did not include expected digest: {stderr}"
    );
    let _ = fs::remove_dir_all(&dir);
}
