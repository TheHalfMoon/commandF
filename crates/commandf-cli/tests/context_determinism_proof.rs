use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};

const PROOF_ARCHIVE: &[u8] = include_bytes!("fixtures/proof.tgz");

#[test]
fn context_cli_output_is_byte_identical_and_reports_sha256() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).unwrap();
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    let cache = PackageCache::new(&cache_path);
    let digest = cache.put(PROOF_ARCHIVE).unwrap();
    let lock = Lockfile::new_v2(
        vec!["acme.proof@1.0.0".to_owned()],
        vec![LockedPackage {
            name: "acme.proof".to_owned(),
            version: "1.0.0".to_owned(),
            sha256: digest,
            source: "synthetic-context-proof".to_owned(),
            dependencies: BTreeMap::new(),
        }],
        vec![],
    );
    fs::write(&lock_path, lock.to_bytes().unwrap()).unwrap();

    let first = run_context(&lock_path, &cache_path);
    let second = run_context(&lock_path, &cache_path);
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
    println!(
        "CF11G_CONTEXT_SHA256={}",
        PackageCache::digest(&first.stdout)
    );
    let _ = fs::remove_dir_all(root);
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "commandf-context-proof-{}-{nonce}",
        std::process::id()
    ))
}

fn run_context(lock: &Path, cache: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
        .args([
            "context",
            "--lock",
            lock.to_str().unwrap(),
            "--cache",
            cache.to_str().unwrap(),
            "--format",
            "json",
        ])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .unwrap()
}
