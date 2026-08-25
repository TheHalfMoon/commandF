use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache, ResolvedDependency};

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("commandf-context-{label}-{}-{nonce}", std::process::id()))
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
    fs::write(&lock, Lockfile::new(Vec::new(), Vec::new()).to_bytes().unwrap()).unwrap();

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
            locked_package("acme.parenta", "1.0.0", &parent_a_sha, parent_a_dependencies),
            locked_package("acme.parentb", "1.0.0", &parent_b_sha, parent_b_dependencies),
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

const PARENT_A_ARCHIVE: &[u8] = &[
    31,139,8,0,0,0,0,0,0,255,237,151,75,111,155,64,16,128,115,246,175,168,246,28,3,166,96,75,190,182,247,70,117,213,75,228,195,26,6,103,83,94,218,135,149,136,242,223,59,203,195,177,8,137,35,133,58,173,52,223,5,175,217,157,29,96,190,17,148,60,250,197,247,224,150,237,209,185,87,69,126,53,49,30,178,12,130,230,136,12,143,158,23,46,158,126,219,255,23,139,165,239,95,125,242,166,78,100,12,163,52,151,184,253,37,246,250,7,169,88,206,51,96,107,198,163,12,156,146,75,200,53,103,215,236,0,82,137,34,199,19,11,199,115,60,86,127,116,162,196,95,161,243,222,221,104,105,34,109,36,124,133,68,228,66,227,179,159,151,178,72,68,250,254,158,112,206,255,85,248,121,224,127,184,242,86,228,255,37,168,102,76,130,42,140,140,224,199,99,105,27,193,72,37,96,63,16,49,158,234,10,2,135,70,166,56,190,211,186,84,107,215,133,7,158,149,88,40,133,220,143,21,146,251,180,110,216,86,174,103,108,199,213,233,86,111,143,106,23,254,62,134,137,69,146,128,237,94,130,99,106,21,131,20,50,28,177,245,109,213,38,255,109,167,64,30,184,93,234,40,179,187,135,72,99,66,186,185,104,156,19,21,177,189,250,239,208,68,137,108,178,125,218,235,219,55,231,164,238,176,131,198,108,139,129,185,220,131,190,25,11,161,65,230,60,117,186,88,163,113,50,161,148,200,247,108,91,99,168,157,200,99,59,192,171,58,240,212,192,6,244,11,183,233,103,119,218,237,150,116,183,167,174,183,117,61,27,125,254,189,255,253,202,185,132,68,77,252,22,112,206,255,96,25,12,252,247,87,126,64,254,95,130,106,168,127,95,8,189,243,182,30,94,21,254,88,115,221,204,103,138,163,90,89,89,40,176,245,43,242,40,53,113,107,156,122,84,26,178,23,162,126,65,29,55,205,4,183,157,215,155,126,162,192,184,150,199,124,4,238,42,53,196,221,202,70,37,120,120,117,255,129,153,39,73,244,66,90,147,62,250,153,77,73,239,255,13,54,70,108,151,115,147,43,83,182,247,109,178,46,112,254,253,63,28,248,31,248,126,72,254,95,130,103,254,119,133,208,235,127,82,15,244,13,64,16,4,65,16,4,65,16,4,65,16,4,65,16,4,65,16,4,241,31,240,7,94,0,20,74,0,40,0,0,
];
const PARENT_B_ARCHIVE: &[u8] = &[
    31,139,8,0,0,0,0,0,0,255,237,153,203,110,163,48,20,134,251,40,35,175,27,174,129,72,217,118,230,5,38,163,217,84,93,16,56,73,221,225,98,217,38,74,149,244,221,199,132,112,81,72,3,153,161,132,72,231,207,194,128,109,124,34,206,247,219,6,230,249,127,188,53,232,44,47,181,55,145,196,15,61,203,80,114,167,211,67,169,116,90,26,134,99,86,199,217,117,211,116,45,235,225,155,209,119,32,231,148,10,233,113,53,252,16,99,141,80,59,18,123,17,144,57,241,252,8,52,230,113,136,229,146,60,146,13,112,65,147,88,85,152,154,161,25,228,227,214,129,162,190,68,71,238,245,133,228,169,47,83,14,223,97,69,99,42,213,179,159,192,86,66,156,101,193,127,186,66,27,255,206,236,148,127,215,114,77,228,127,8,237,8,7,145,164,220,135,95,239,44,243,129,51,137,160,236,128,6,170,170,204,7,117,33,229,161,186,242,42,37,19,115,93,135,173,23,177,16,180,132,175,207,101,146,94,239,121,234,44,143,68,230,35,255,168,53,90,122,162,30,64,247,145,24,79,86,52,132,125,113,239,128,174,86,144,153,26,245,84,192,59,2,33,68,234,140,204,159,119,249,159,42,71,213,54,94,152,194,243,246,165,140,72,53,241,147,32,11,237,39,28,110,226,131,170,59,142,160,170,59,7,37,94,149,175,6,123,235,16,211,203,71,246,27,139,159,94,226,63,123,8,125,44,8,218,248,55,27,252,59,166,99,35,255,67,232,26,254,179,124,184,26,253,99,39,92,79,140,83,151,248,207,109,107,178,49,191,120,254,183,234,123,129,124,254,55,172,41,242,63,132,174,225,191,204,135,171,77,32,239,137,54,48,62,117,225,223,186,5,255,14,242,63,132,254,133,127,171,15,254,45,228,127,12,42,248,255,157,109,126,22,32,39,75,26,7,52,94,247,249,34,176,117,253,223,120,255,103,219,51,3,249,31,66,13,254,139,68,40,23,253,121,62,92,68,190,232,163,87,141,113,162,191,15,53,248,167,17,75,184,132,160,71,3,104,231,223,62,229,223,53,112,255,63,136,90,249,47,242,161,155,1,212,90,163,3,220,131,10,254,159,146,0,22,239,66,66,52,17,135,226,198,252,219,200,255,32,106,240,95,37,66,185,236,47,206,62,231,191,234,164,151,173,145,255,123,208,57,254,83,198,242,175,36,61,121,64,235,254,223,153,157,240,63,117,76,92,255,15,162,46,252,151,249,208,217,3,234,61,154,223,251,170,106,209,213,79,246,232,33,40,20,10,213,175,254,2,236,72,28,58,0,40,0,0,
];
const SHARED_V1_ARCHIVE: &[u8] = &[
    31,139,8,0,0,0,0,0,0,255,237,206,59,14,194,48,16,132,225,28,5,109,141,204,26,133,20,220,102,21,44,94,138,19,217,64,131,184,59,6,26,64,148,17,80,252,95,51,171,105,102,7,107,247,182,14,179,225,145,110,151,251,88,141,76,139,166,174,239,89,188,167,234,226,233,190,245,222,55,115,95,77,116,236,71,62,57,230,131,165,50,255,141,173,63,116,150,104,93,144,165,88,219,5,151,55,150,194,74,166,114,10,41,111,251,88,122,239,212,169,92,126,253,39,0,0,0,0,0,0,0,0,0,0,0,0,0,224,213,21,117,232,185,119,0,40,0,0,
];
const SHARED_V2_ARCHIVE: &[u8] = &[
    31,139,8,0,0,0,0,0,0,255,237,206,59,14,194,48,16,132,225,28,5,109,141,204,58,10,41,184,205,42,88,188,20,39,138,129,38,226,238,24,104,0,81,70,64,241,127,205,172,166,153,237,173,57,216,38,44,250,71,186,125,234,98,49,49,205,234,170,186,103,246,158,170,203,167,251,214,123,95,151,190,152,233,212,143,124,114,74,71,27,242,252,55,182,254,208,40,209,218,32,43,177,166,13,46,109,109,8,107,153,203,57,12,105,215,197,220,151,78,157,202,229,215,127,2,0,0,0,0,0,0,0,0,0,0,0,0,0,94,93,1,133,109,57,185,0,40,0,0,
];
const MALFORMED_ARCHIVE: &[u8] = &[
    31,139,8,0,0,0,0,0,0,255,237,212,65,75,195,48,20,7,240,125,20,201,121,109,51,237,42,236,236,93,80,111,226,33,75,95,103,102,155,148,36,29,142,178,239,238,107,39,8,163,224,101,78,145,255,175,135,164,73,154,60,104,222,107,149,126,83,27,202,218,99,155,110,131,179,179,51,147,172,200,243,177,101,167,173,148,249,242,171,63,140,47,22,197,245,114,118,37,207,29,200,148,46,68,229,249,248,75,156,245,7,245,194,170,134,196,74,40,221,80,186,86,165,152,139,29,249,96,156,229,193,69,42,83,41,14,191,29,36,252,152,207,188,207,30,163,239,116,236,60,221,81,101,172,137,252,255,19,190,13,103,169,7,223,229,255,205,82,158,228,127,126,91,20,200,255,75,232,133,167,224,58,175,233,105,223,14,117,96,226,34,112,73,48,37,79,29,171,67,231,107,238,191,198,216,134,85,150,209,187,106,218,154,82,231,55,83,119,40,155,174,40,115,81,154,170,34,79,54,26,197,219,245,130,106,106,248,77,172,158,251,227,97,247,235,64,126,167,134,77,210,208,173,183,164,35,127,22,199,32,121,141,118,229,16,237,3,141,187,104,226,185,214,187,202,212,195,168,117,49,81,54,81,222,171,189,56,188,240,131,18,6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,255,214,7,111,234,157,86,0,40,0,0,
];
