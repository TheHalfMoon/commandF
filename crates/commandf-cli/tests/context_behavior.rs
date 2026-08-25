use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache, ResolvedDependency};

const PARENT_A_ARCHIVE: &[u8] = include_bytes!("fixtures/parent-a.tgz");
const PARENT_B_ARCHIVE: &[u8] = include_bytes!("fixtures/parent-b.tgz");
const SHARED_V1_ARCHIVE: &[u8] = include_bytes!("fixtures/shared-v1.tgz");
const SHARED_V2_ARCHIVE: &[u8] = include_bytes!("fixtures/shared-v2.tgz");
const MALFORMED_ARCHIVE: &[u8] = include_bytes!("fixtures/malformed.tgz");

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "commandf-context-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn run_context(lock: &Path, cache: &Path) -> Output {
    commandf()
        .args([
            "context",
            "--lock",
            lock.to_str().expect("UTF-8 lock path"),
            "--cache",
            cache.to_str().expect("UTF-8 cache path"),
            "--format",
            "json",
        ])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("commandf context must execute")
}

#[test]
fn context_help_exposes_offline_inputs() {
    let output = commandf()
        .args(["context", "--help"])
        .output()
        .expect("commandf context help must execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 help");
    for flag in ["--lock", "--cache", "--format"] {
        assert!(stdout.contains(flag), "missing {flag}");
    }
}

#[test]
fn context_emits_byte_identical_multi_version_graph_evidence() {
    let root = unique_temp_dir("success");
    let (lock, cache) = write_context_state(&root);

    let first = run_context(&lock, &cache);
    let second = run_context(&lock, &cache);
    assert_success(&first);
    assert_success(&second);
    assert_eq!(first.stdout, second.stdout);

    let json = String::from_utf8(first.stdout).expect("UTF-8 context JSON");
    for evidence in [
        "\"lock_schema\": 2",
        "\"name\": \"acme.shared\"",
        "\"version\": \"1.0.0\"",
        "\"version\": \"2.0.0\"",
        "\"id\": \"extension\"",
        "\"resource_type\": \"Patient\"",
        "\"resolution\": \"resolved\"",
        "\"resolution\": \"external\"",
        "\"resolution\": \"ambiguous\"",
        "\"relation\": \"structure_base_definition\"",
        "\"relation\": \"structure_type_profile\"",
        "\"relation\": \"structure_type_target_profile\"",
        "\"relation\": \"structure_binding_value_set\"",
        "\"relation\": \"value_set_include_system\"",
        "\"relation\": \"value_set_include_value_set\"",
        "\"relation\": \"value_set_exclude_system\"",
        "\"relation\": \"code_system_supplements\"",
        "\"unsupported_source_resource_types\": [\n      \"Patient\"\n    ]",
    ] {
        assert!(json.contains(evidence), "missing evidence: {evidence}");
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn context_rejects_schema_v1_with_stable_migration_diagnostic() {
    let root = unique_temp_dir("schema-v1");
    let lock = root.join("commandf.lock");
    let cache = root.join("cache");
    fs::create_dir_all(&cache).unwrap();
    fs::write(
        &lock,
        Lockfile::new(Vec::new(), Vec::new()).to_bytes().unwrap(),
    )
    .unwrap();

    let output = run_context(&lock, &cache);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("commandf context requires commandf.lock schema 2; found schema 1"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn context_fails_closed_on_missing_corrupt_and_malformed_inputs() {
    let missing_root = unique_temp_dir("missing");
    fs::create_dir_all(&missing_root).unwrap();
    let missing_lock = missing_root.join("commandf.lock");
    let missing_cache = missing_root.join("cache");
    let missing = Lockfile::new_v2(
        vec!["acme.root@1.0.0".to_owned()],
        vec![locked_package(
            "acme.root",
            "1.0.0",
            &"a".repeat(64),
            BTreeMap::new(),
        )],
        vec![],
    );
    fs::write(&missing_lock, missing.to_bytes().unwrap()).unwrap();
    let output = run_context(&missing_lock, &missing_cache);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let _ = fs::remove_dir_all(missing_root);

    let corrupt_root = unique_temp_dir("corrupt");
    let (corrupt_lock, corrupt_cache) = write_context_state(&corrupt_root);
    let lockfile = Lockfile::from_slice(&fs::read(&corrupt_lock).unwrap()).unwrap();
    let digest = &lockfile.packages[0].sha256;
    fs::write(
        corrupt_cache.join("sha256").join(format!("{digest}.tgz")),
        b"corrupted",
    )
    .unwrap();
    let output = run_context(&corrupt_lock, &corrupt_cache);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let _ = fs::remove_dir_all(corrupt_root);

    let malformed_root = unique_temp_dir("malformed");
    let malformed_cache = malformed_root.join("cache");
    let malformed_lock = malformed_root.join("commandf.lock");
    fs::create_dir_all(&malformed_root).unwrap();
    let cache = PackageCache::new(&malformed_cache);
    let digest = cache.put(MALFORMED_ARCHIVE).unwrap();
    let lock = Lockfile::new_v2(
        vec!["acme.bad@1.0.0".to_owned()],
        vec![locked_package(
            "acme.bad",
            "1.0.0",
            &digest,
            BTreeMap::new(),
        )],
        vec![],
    );
    fs::write(&malformed_lock, lock.to_bytes().unwrap()).unwrap();
    let output = run_context(&malformed_lock, &malformed_cache);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must be an array"));
    let _ = fs::remove_dir_all(malformed_root);
}

fn write_context_state(root: &Path) -> (PathBuf, PathBuf) {
    fs::create_dir_all(root).unwrap();
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    let cache = PackageCache::new(&cache_path);

    let parent_a_sha = cache.put(PARENT_A_ARCHIVE).unwrap();
    let parent_b_sha = cache.put(PARENT_B_ARCHIVE).unwrap();
    let shared_v1_sha = cache.put(SHARED_V1_ARCHIVE).unwrap();
    let shared_v2_sha = cache.put(SHARED_V2_ARCHIVE).unwrap();

    let mut parent_a_dependencies = BTreeMap::new();
    parent_a_dependencies.insert("acme.shared".to_owned(), "1.0.0".to_owned());
    let mut parent_b_dependencies = BTreeMap::new();
    parent_b_dependencies.insert("acme.shared".to_owned(), "2.0.0".to_owned());

    let lock = Lockfile::new_v2(
        vec![
            "acme.parentb@1.0.0".to_owned(),
            "acme.parenta@1.0.0".to_owned(),
        ],
        vec![
            locked_package(
                "acme.parenta",
                "1.0.0",
                &parent_a_sha,
                parent_a_dependencies,
            ),
            locked_package(
                "acme.parentb",
                "1.0.0",
                &parent_b_sha,
                parent_b_dependencies,
            ),
            locked_package("acme.shared", "1.0.0", &shared_v1_sha, BTreeMap::new()),
            locked_package("acme.shared", "2.0.0", &shared_v2_sha, BTreeMap::new()),
        ],
        vec![
            ResolvedDependency {
                from_name: "acme.parenta".to_owned(),
                from_version: "1.0.0".to_owned(),
                to_name: "acme.shared".to_owned(),
                to_version: "1.0.0".to_owned(),
                declared_constraint: "1.0.0".to_owned(),
            },
            ResolvedDependency {
                from_name: "acme.parentb".to_owned(),
                from_version: "1.0.0".to_owned(),
                to_name: "acme.shared".to_owned(),
                to_version: "2.0.0".to_owned(),
                declared_constraint: "2.0.0".to_owned(),
            },
        ],
    );
    fs::write(&lock_path, lock.to_bytes().unwrap()).unwrap();
    (lock_path, cache_path)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
}

fn locked_package(
    name: &str,
    version: &str,
    sha256: &str,
    dependencies: BTreeMap<String, String>,
) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256: sha256.to_owned(),
        source: "synthetic-context-test".to_owned(),
        dependencies,
    }
}
