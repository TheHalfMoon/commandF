use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const OFFLINE_PROOF_ARCHIVE: &[u8] = include_bytes!("fixtures/proof.tgz");

fn workdir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "commandf-registry-origin-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir(&dir).expect("create isolated test state");
    dir
}

fn command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
        .args(args)
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("local process")
}

#[test]
fn automatic_explicitly_preserves_offline_mirror_resolution_and_verification() {
    let root = workdir();
    let mirror = root.join("mirror");
    let package_dir = mirror.join("acme.proof").join("1.0.0");
    fs::create_dir_all(&package_dir).expect("create mirror hierarchy");
    fs::write(package_dir.join("package.tgz"), OFFLINE_PROOF_ARCHIVE).expect("fixture");
    let cache = root.join("cache");
    let lock = root.join("commandf.lock");
    let run = command(&[
        "pkg",
        "resolve",
        "acme.proof@1.0.0",
        "--source-dir",
        mirror.to_str().unwrap(),
        "--registry-origin",
        "automatic",
        "--cache",
        cache.to_str().unwrap(),
        "--lock",
        lock.to_str().unwrap(),
    ]);
    assert_eq!(run.status.code(), Some(0), "{:?}", run.stderr);
    assert!(lock.is_file());
    let verify = command(&[
        "pkg",
        "verify",
        "--cache",
        cache.to_str().unwrap(),
        "--lock",
        lock.to_str().unwrap(),
    ]);
    assert_eq!(verify.status.code(), Some(0), "{:?}", verify.stderr);
    fs::remove_dir_all(root).expect("remove test state");
}

#[test]
fn explicit_official_origin_is_incompatible_with_local_mirror_and_fails_before_io() {
    let root = workdir();
    let lock = root.join("never-written.lock");
    for origin in ["primary", "secondary"] {
        let run = command(&[
            "pkg",
            "resolve",
            "acme.proof@1.0.0",
            "--source-dir",
            "/nonexistent/commandf-test-mirror",
            "--registry-origin",
            origin,
            "--lock",
            lock.to_str().unwrap(),
        ]);
        assert_eq!(run.status.code(), Some(1), "{origin}: {:?}", run.stderr);
        assert!(run.stdout.is_empty());
        assert!(String::from_utf8_lossy(&run.stderr)
            .contains("--registry-origin primary/secondary cannot be combined"));
        assert!(
            !lock.exists(),
            "must not persist lockfile for invalid source policy"
        );
    }
    fs::remove_dir_all(root).expect("remove test state");
}

#[test]
fn arbitrary_external_origin_is_not_accepted_as_a_cli_option() {
    let out = command(&[
        "pkg",
        "resolve",
        "acme.proof@1.0.0",
        "--registry-origin",
        "https://attacker.example.invalid/",
    ]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let text = String::from_utf8_lossy(&out.stderr);
    assert!(text.contains("invalid value"));
    assert!(text.contains("primary"));
    assert!(text.contains("secondary"));
}

#[test]
fn help_describes_bounded_official_origin_policy() {
    let out = command(&["pkg", "resolve", "--help"]);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("--registry-origin"));
    assert!(text.contains("automatic"));
    assert!(text.contains("primary"));
    assert!(text.contains("secondary"));
}
