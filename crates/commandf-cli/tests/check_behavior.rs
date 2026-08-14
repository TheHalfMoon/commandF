use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};

const BEFORE_ARCHIVE: &[u8] = &[
    31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 237, 148, 77, 79, 195, 48, 12, 134, 251, 83, 80, 206, 163, 31,
    99, 180, 82, 207, 112, 230, 0, 55, 196, 33, 107, 189, 53, 208, 166, 85, 146, 78, 67, 211, 254, 59, 238, 214,
    109, 108, 171, 196, 1, 52, 9, 120, 159, 30, 220, 56, 141, 243, 54, 177, 221, 200, 236, 77, 206, 41, 104, 182,
    214, 127, 181, 181, 246, 126, 152, 144, 137, 39, 147, 141, 101, 78, 109, 24, 38, 241, 225, 189, 243, 71, 81, 124,
    19, 121, 87, 161, 119, 1, 90, 235, 164, 225, 237, 189, 255, 201, 74, 104, 89, 145, 72, 5, 45, 101, 213, 148,
    228, 247, 137, 32, 70, 98, 65, 198, 170, 90, 243, 92, 228, 135, 126, 200, 158, 156, 26, 210, 57, 233, 76, 145,
    21, 233, 106, 189, 246, 192, 47, 167, 191, 238, 224, 209, 153, 54, 115, 173, 161, 59, 154, 41, 173, 28, 95, 252,
    245, 46, 37, 190, 219, 19, 190, 170, 255, 36, 30, 159, 212, 255, 109, 18, 142, 81, 255, 151, 169, 127, 67, 182,
    110, 77, 70, 79, 239, 77, 215, 7, 6, 18, 129, 43, 95, 229, 135, 22, 193, 195, 214, 148, 60, 46, 156, 107,
    210, 32, 216, 165, 73, 109, 230, 67, 105, 20, 28, 150, 157, 119, 148, 190, 249, 220, 239, 63, 225, 219, 112, 45,
    55, 23, 33, 51, 167, 22, 157, 231, 77, 233, 110, 243, 157, 76, 246, 200, 169, 117, 134, 231, 69, 58, 147, 165,
    165, 145, 112, 91, 233, 15, 83, 75, 102, 33, 123, 201, 83, 105, 63, 255, 195, 94, 110, 81, 38, 27, 169, 179,
    66, 153, 65, 189, 199, 97, 114, 50, 170, 31, 164, 34, 171, 117, 183, 181, 210, 174, 147, 170, 101, 99, 139, 154,
    101, 172, 4, 149, 84, 17, 123, 211, 231, 213, 246, 172, 142, 131, 52, 210, 21, 103, 206, 74, 113, 200, 144, 173,
    92, 118, 39, 34, 214, 163, 243, 181, 126, 127, 30, 67, 33, 14, 115, 167, 145, 118, 39, 194, 98, 178, 58, 167,
    141, 112, 54, 235, 23, 126, 208, 179, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 128, 63, 199, 7, 94, 196, 156, 99, 0, 40, 0, 0,
];

const AFTER_ARCHIVE: &[u8] = &[
    31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 237, 148, 77, 79, 195, 48, 12, 134, 251, 83, 80, 206, 163, 31,
    99, 108, 82, 207, 112, 230, 0, 55, 196, 33, 107, 189, 53, 208, 165, 85, 146, 78, 67, 83, 255, 59, 110, 215,
    173, 108, 171, 196, 1, 52, 9, 120, 159, 30, 220, 56, 141, 243, 54, 177, 93, 202, 228, 77, 46, 41, 40, 119,
    214, 127, 181, 133, 246, 126, 152, 144, 153, 78, 38, 173, 101, 78, 109, 24, 206, 166, 253, 123, 227, 143, 162, 233,
    77, 228, 93, 133, 222, 5, 168, 172, 147, 134, 183, 247, 254, 39, 91, 161, 229, 138, 68, 44, 104, 35, 87, 101,
    78, 126, 151, 8, 98, 36, 214, 100, 172, 42, 52, 207, 69, 126, 228, 135, 236, 73, 169, 36, 157, 146, 78, 20,
    89, 17, 111, 235, 218, 3, 191, 156, 238, 186, 131, 71, 103, 170, 196, 85, 134, 238, 104, 161, 180, 114, 124, 241,
    215, 251, 148, 248, 110, 79, 248, 170, 254, 103, 211, 241, 73, 253, 223, 206, 194, 49, 234, 255, 50, 245, 111, 200,
    22, 149, 73, 232, 233, 189, 108, 250, 192, 64, 34, 112, 229, 171, 180, 111, 17, 60, 172, 76, 206, 227, 204, 185,
    50, 14, 130, 125, 154, 20, 102, 57, 148, 70, 65, 191, 236, 188, 163, 116, 205, 231, 254, 240, 9, 223, 134, 171,
    184, 185, 8, 153, 56, 181, 110, 60, 111, 74, 55, 155, 239, 101, 178, 71, 206, 173, 51, 60, 47, 226, 133, 204,
    45, 141, 132, 219, 73, 127, 152, 91, 50, 107, 217, 73, 158, 75, 251, 249, 31, 14, 114, 179, 124, 214, 74, 93,
    100, 202, 12, 234, 61, 14, 147, 146, 81, 221, 32, 22, 73, 161, 155, 173, 149, 118, 141, 84, 45, 75, 155, 21,
    44, 99, 43, 40, 167, 21, 177, 55, 126, 222, 238, 206, 234, 56, 72, 41, 93, 118, 230, 92, 41, 14, 25, 178,
    149, 155, 230, 68, 68, 61, 58, 95, 235, 119, 231, 49, 20, 162, 159, 107, 35, 69, 125, 164, 253, 137, 176, 152,
    164, 72, 169, 21, 206, 166, 126, 225, 7, 61, 27, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 248, 115, 124, 0, 147, 79, 101, 101, 0, 40, 0, 0,
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

fn write_locked_state(root: &Path, archive: &[u8], version: &str) -> (PathBuf, PathBuf) {
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    fs::create_dir_all(root).expect("create state root");
    let cache = PackageCache::new(&cache_path);
    let digest = cache.put(archive).expect("cache synthetic archive");
    let lockfile = Lockfile::new(
        vec![format!("example.package@{version}")],
        vec![LockedPackage {
            name: "example.package".to_owned(),
            version: version.to_owned(),
            sha256: digest,
            source: "synthetic-test".to_owned(),
            dependencies: BTreeMap::new(),
        }],
    );
    fs::write(&lock_path, lockfile.to_bytes().expect("serialize lock")).expect("write lock");
    (lock_path, cache_path)
}

fn run_check(
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
    extra: &[&str],
) -> Output {
    let mut command = commandf();
    command.args([
        "check",
        "example.package",
        "--before-lock",
        before_lock.to_str().expect("UTF-8 path"),
        "--before-cache",
        before_cache.to_str().expect("UTF-8 path"),
        "--after-lock",
        after_lock.to_str().expect("UTF-8 path"),
        "--after-cache",
        after_cache.to_str().expect("UTF-8 path"),
    ]);
    command.args(extra);
    command
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("commandf check must execute")
}

#[test]
fn check_help_exposes_ci_contract() {
    let output = commandf()
        .args(["check", "--help"])
        .output()
        .expect("commandf check help must execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 help");
    for flag in [
        "--before-lock",
        "--before-cache",
        "--after-lock",
        "--after-cache",
        "--direction",
        "--fail-on",
        "--format",
        "--output",
    ] {
        assert!(stdout.contains(flag), "missing {flag}");
    }
}

#[test]
fn breaking_policy_failure_emits_json_then_exits_two() {
    let dir = unique_temp_dir("check-breaking");
    let (before_lock, before_cache) =
        write_locked_state(&dir.join("before"), BEFORE_ARCHIVE, "1.0.0");
    let (after_lock, after_cache) =
        write_locked_state(&dir.join("after"), AFTER_ARCHIVE, "1.1.0");

    let output = run_check(
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[],
    );
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON output");
    assert_eq!(value["schema"], 1);
    assert_eq!(value["policy"]["direction"], "both");
    assert_eq!(value["policy"]["fail_on"], "breaking");
    assert_eq!(value["decision"]["passed"], false);
    assert!(value["decision"]["blocking_findings"].as_u64().unwrap() > 0);
    assert!(!value["compatibility"]["findings"].as_array().unwrap().is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn fail_on_none_passes_without_removing_findings() {
    let dir = unique_temp_dir("check-none");
    let (before_lock, before_cache) =
        write_locked_state(&dir.join("before"), BEFORE_ARCHIVE, "1.0.0");
    let (after_lock, after_cache) =
        write_locked_state(&dir.join("after"), AFTER_ARCHIVE, "1.1.0");

    let output = run_check(
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &["--fail-on", "none"],
    );
    assert_eq!(output.status.code(), Some(0));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON output");
    assert_eq!(value["decision"]["passed"], true);
    assert_eq!(value["decision"]["blocking_findings"], 0);
    assert!(!value["compatibility"]["findings"].as_array().unwrap().is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sarif_output_file_is_complete_before_policy_exit_two() {
    let dir = unique_temp_dir("check-sarif-file");
    let (before_lock, before_cache) =
        write_locked_state(&dir.join("before"), BEFORE_ARCHIVE, "1.0.0");
    let (after_lock, after_cache) =
        write_locked_state(&dir.join("after"), AFTER_ARCHIVE, "1.1.0");
    let output_path = dir.join("result.sarif");
    let output_string = output_path.to_str().expect("UTF-8 path").to_owned();

    let output = run_check(
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &["--format", "sarif", "--output", &output_string],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let bytes = fs::read(&output_path).expect("SARIF output must exist");
    let value: serde_json::Value = serde_json::from_slice(&bytes).expect("SARIF JSON");
    assert_eq!(value["version"], "2.1.0");
    assert_eq!(value["runs"][0]["tool"]["driver"]["name"], "commandF");
    assert!(!value["runs"][0]["results"].as_array().unwrap().is_empty());
    assert!(value["runs"][0]["results"][0].get("locations").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn corrupted_cache_is_operational_exit_one_not_policy_exit_two() {
    let dir = unique_temp_dir("check-corrupt");
    let (before_lock, before_cache) =
        write_locked_state(&dir.join("before"), BEFORE_ARCHIVE, "1.0.0");
    let (after_lock, after_cache) =
        write_locked_state(&dir.join("after"), AFTER_ARCHIVE, "1.1.0");
    let digest = PackageCache::digest(AFTER_ARCHIVE);
    fs::write(
        after_cache.join("sha256").join(format!("{digest}.tgz")),
        b"corrupt",
    )
    .expect("corrupt after cache");

    let output = run_check(
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[],
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cache object digest mismatch"), "stderr: {stderr}");
    let _ = fs::remove_dir_all(&dir);
}
