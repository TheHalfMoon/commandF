use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};

const PROOF_ARCHIVE: &[u8] = include_bytes!("fixtures/proof.tgz");
const GATE_LOCKFILE_LIMIT: u64 = 16 * 1024 * 1024;
const GATE_BASELINE_LIMIT: u64 = 64 * 1024 * 1024;

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

fn write_valid_state(root: &Path) -> (PathBuf, PathBuf) {
    fs::create_dir_all(root).expect("create state root");
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    let cache = PackageCache::new(&cache_path);
    let digest = cache.put(PROOF_ARCHIVE).expect("cache proof archive");
    let lockfile = Lockfile::new_v2(
        vec!["acme.proof@1.0.0".to_owned()],
        vec![LockedPackage {
            name: "acme.proof".to_owned(),
            version: "1.0.0".to_owned(),
            sha256: digest,
            source: "synthetic-gate-bounds".to_owned(),
            dependencies: BTreeMap::new(),
        }],
        vec![],
    );
    fs::write(&lock_path, lockfile.to_bytes().expect("serialize lockfile"))
        .expect("write lockfile");
    (lock_path, cache_path)
}

fn run_gate(
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
    extra: &[String],
) -> Output {
    let mut command = commandf();
    command.args([
        "gate",
        "acme.proof",
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
        .expect("commandf gate must execute")
}

fn create_sparse_file(path: &Path, bytes: u64) {
    let file = fs::File::create(path).expect("create sparse input");
    file.set_len(bytes).expect("size sparse input");
}

#[test]
fn oversized_baseline_is_operational_exit_one() {
    let root = unique_temp_dir("gate-oversized-baseline");
    let (lock, cache) = write_valid_state(&root.join("state"));
    let baseline = root.join("oversized-baseline.json");
    create_sparse_file(&baseline, GATE_BASELINE_LIMIT + 1);

    let output = run_gate(
        &lock,
        &cache,
        &lock,
        &cache,
        &[
            "--baseline".to_owned(),
            baseline.to_str().expect("UTF-8 path").to_owned(),
        ],
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("exceeds"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn oversized_primary_lockfile_is_operational_exit_one() {
    let root = unique_temp_dir("gate-oversized-lockfile");
    let (valid_lock, cache) = write_valid_state(&root.join("state"));
    let oversized_lock = root.join("oversized.lock");
    create_sparse_file(&oversized_lock, GATE_LOCKFILE_LIMIT + 1);

    let output = run_gate(
        &oversized_lock,
        &cache,
        &valid_lock,
        &cache,
        &[],
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("exceeds"));
    let _ = fs::remove_dir_all(root);
}
