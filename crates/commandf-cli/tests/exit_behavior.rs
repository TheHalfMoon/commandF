use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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
    fs::write(
        &lock,
        b"{\n  \"schema\": 1,\n  \"roots\": [],\n  \"packages\": []\n}\n",
    )
    .expect("write lockfile");

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
