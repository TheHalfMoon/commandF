use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};

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
    println!("CF11G_CONTEXT_SHA256={}", PackageCache::digest(&first.stdout));
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

const PROOF_ARCHIVE: &[u8] = &[
    31,139,8,0,0,0,0,0,0,255,237,212,209,78,195,32,20,6,224,61,138,225,122,2,117,181,75,118,237,27,232,11,96,61,155,213,22,200,1,204,150,197,119,151,214,25,27,83,227,77,157,94,252,223,13,133,166,7,146,114,126,111,234,103,179,35,229,223,71,249,20,156,93,204,76,103,85,89,14,99,246,117,212,186,92,127,62,247,235,69,81,93,173,23,23,122,238,131,76,73,33,26,206,219,159,99,175,127,232,40,172,233,72,108,132,169,59,146,158,157,219,138,165,120,33,14,141,179,121,185,144,90,106,241,250,215,199,132,95,114,234,123,117,27,57,213,49,49,221,208,182,177,77,204,127,255,114,184,13,51,36,194,79,253,191,26,103,193,208,255,215,171,74,163,255,207,225,40,152,130,75,92,211,221,193,247,57,48,113,17,114,32,52,15,249,213,71,58,36,110,243,236,49,70,31,54,74,209,222,116,190,37,233,120,55,117,139,212,119,153,178,20,247,38,140,119,25,151,140,196,214,180,242,84,123,178,110,255,53,130,9,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,224,13,155,19,112,211,0,40,0,0,
];
