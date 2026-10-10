use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile};

fn test_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "commandf-cf11-lock-compare-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&path).unwrap();
    path
}

fn locked(digest: &str, source: &str) -> Lockfile {
    Lockfile::new_v2(
        vec!["acme.root@1.0.0".to_owned()],
        vec![LockedPackage {
            name: "acme.root".to_owned(),
            version: "1.0.0".to_owned(),
            sha256: digest.to_owned(),
            source: source.to_owned(),
            dependencies: BTreeMap::new(),
        }],
        Vec::new(),
    )
}

fn invoke(first: &Path, second: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
        .args([
            "pkg",
            "compare-locks",
            "--first-lock",
            first.to_str().unwrap(),
            "--second-lock",
            second.to_str().unwrap(),
        ])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("offline command must run")
}

#[test]
fn cli_reports_source_redacted_digest_conflict_and_stable_exit_codes() {
    let dir = test_dir();
    let left = dir.join("first.lock");
    let right = dir.join("second.lock");
    let digest_a = "a".repeat(64);
    let digest_b = "b".repeat(64);
    fs::write(
        &left,
        locked(&digest_a, "secret-first-origin").to_bytes().unwrap(),
    )
    .unwrap();
    fs::write(
        &right,
        locked(&digest_a, "secret-second-origin")
            .to_bytes()
            .unwrap(),
    )
    .unwrap();

    let same = invoke(&left, &right);
    assert_eq!(same.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&same.stdout).contains("\"equivalent\": true"));
    assert!(!String::from_utf8_lossy(&same.stdout).contains("secret"));

    fs::write(
        &right,
        locked(&digest_b, "secret-second-origin")
            .to_bytes()
            .unwrap(),
    )
    .unwrap();
    let different = invoke(&left, &right);
    assert_eq!(different.status.code(), Some(2));
    let output = String::from_utf8_lossy(&different.stdout);
    assert!(output.contains("\"equivalent\": false"));
    assert!(output.contains(&digest_a));
    assert!(output.contains(&digest_b));
    assert!(output.contains("\"acme.root\""));
    assert!(!output.contains("secret"));
    assert!(different.stderr.is_empty());

    fs::write(&right, b"invalid lockfile").unwrap();
    let malformed = invoke(&left, &right);
    assert_eq!(malformed.status.code(), Some(1));
    assert!(malformed.stdout.is_empty());
    assert!(!malformed.stderr.is_empty());

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn cli_refuses_legacy_v1_vs_v2_without_pretending_equivalence() {
    let dir = test_dir();
    let left = dir.join("first.lock");
    let right = dir.join("second.lock");
    fs::write(
        &left,
        Lockfile::new(Vec::new(), Vec::new()).to_bytes().unwrap(),
    )
    .unwrap();
    fs::write(
        &right,
        locked(&"a".repeat(64), "private-origin")
            .to_bytes()
            .unwrap(),
    )
    .unwrap();
    let result = invoke(&left, &right);
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("schema-v2"));
    let _ = fs::remove_dir_all(dir);
}
