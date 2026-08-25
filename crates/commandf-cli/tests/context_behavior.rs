use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use commandf_pkg::{LockedPackage, Lockfile, PackageCache, ResolvedDependency};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use tar::{Builder, Header};
use tempfile::{tempdir, TempDir};

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
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
    let state = write_context_state();

    let first = run_context(&state.lock, &state.cache);
    let second = run_context(&state.lock, &state.cache);
    assert_success(&first);
    assert_success(&second);
    assert_eq!(first.stdout, second.stdout);

    let report: Value = serde_json::from_slice(&first.stdout).expect("context JSON");
    assert_eq!(report["schema"], 1);
    assert_eq!(report["lock_schema"], 2);
    assert_eq!(report["packages"].as_array().unwrap().len(), 4);

    let package_edges = report["package_dependency_edges"].as_array().unwrap();
    assert_eq!(package_edges.len(), 2);
    let selected_shared_versions = package_edges
        .iter()
        .map(|edge| edge["to"]["version"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(selected_shared_versions.contains(&"1.0.0"));
    assert!(selected_shared_versions.contains(&"2.0.0"));

    let artifacts = report["artifacts"].as_array().unwrap();
    assert!(artifacts.iter().any(|artifact| {
        artifact["resource_type"] == "StructureDefinition" && artifact["id"] == "extension"
    }));
    assert!(artifacts
        .iter()
        .any(|artifact| artifact["resource_type"] == "Patient"));

    let reference_edges = report["canonical_reference_edges"].as_array().unwrap();
    let resolutions = reference_edges
        .iter()
        .map(|edge| edge["resolution"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(resolutions.contains(&"resolved"));
    assert!(resolutions.contains(&"external"));
    assert!(resolutions.contains(&"ambiguous"));

    for relation in [
        "structure_base_definition",
        "structure_type_profile",
        "structure_type_target_profile",
        "structure_binding_value_set",
        "value_set_include_system",
        "value_set_include_value_set",
        "value_set_exclude_system",
        "code_system_supplements",
    ] {
        assert!(reference_edges
            .iter()
            .any(|edge| edge["relation"].as_str() == Some(relation)),
            "missing relation {relation}");
    }

    assert_eq!(
        report["coverage"]["unsupported_source_resource_types"],
        serde_json::json!(["Patient"])
    );
}

#[test]
fn context_rejects_schema_v1_with_stable_migration_diagnostic() {
    let dir = tempdir().unwrap();
    let lock = dir.path().join("commandf.lock");
    let cache = dir.path().join("cache");
    fs::create_dir_all(&cache).unwrap();
    fs::write(&lock, Lockfile::new(Vec::new(), Vec::new()).to_bytes().unwrap()).unwrap();

    let output = run_context(&lock, &cache);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("commandf context requires commandf.lock schema 2; found schema 1"));
}

#[test]
fn context_fails_closed_on_missing_or_corrupted_cache() {
    let missing = tempdir().unwrap();
    let missing_lock = missing.path().join("commandf.lock");
    let missing_cache = missing.path().join("cache");
    let lock = Lockfile::new_v2(
        vec!["acme.root@1.0.0".to_owned()],
        vec![locked_package(
            "acme.root",
            "1.0.0",
            &"a".repeat(64),
            BTreeMap::new(),
        )],
        vec![],
    );
    fs::write(&missing_lock, lock.to_bytes().unwrap()).unwrap();
    let missing_output = run_context(&missing_lock, &missing_cache);
    assert_eq!(missing_output.status.code(), Some(1));
    assert!(missing_output.stdout.is_empty());

    let state = write_context_state();
    let lockfile = Lockfile::from_slice(&fs::read(&state.lock).unwrap()).unwrap();
    let digest = &lockfile.packages[0].sha256;
    fs::write(
        state.cache.join("sha256").join(format!("{digest}.tgz")),
        b"corrupted",
    )
    .unwrap();
    let corrupted_output = run_context(&state.lock, &state.cache);
    assert_eq!(corrupted_output.status.code(), Some(1));
    assert!(corrupted_output.stdout.is_empty());
}

#[test]
fn context_rejects_malformed_supported_reference_shape() {
    let dir = tempdir().unwrap();
    let cache_path = dir.path().join("cache");
    let lock_path = dir.path().join("commandf.lock");
    let cache = PackageCache::new(&cache_path);
    let archive = package_archive(
        "acme.bad",
        "1.0.0",
        &[(
            "package/StructureDefinition-bad.json",
            br#"{
              "resourceType":"StructureDefinition",
              "id":"bad",
              "url":"https://example.org/StructureDefinition/bad",
              "version":"1.0.0",
              "differential":{"element":[{
                "id":"Observation.subject",
                "type":[{"code":"Reference","profile":"not-an-array"}]
              }]}
            }"#,
        )],
    );
    let digest = cache.put(&archive).unwrap();
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
    fs::write(&lock_path, lock.to_bytes().unwrap()).unwrap();

    let output = run_context(&lock_path, &cache_path);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("must be an array"));
}

struct ContextState {
    _dir: TempDir,
    lock: PathBuf,
    cache: PathBuf,
}

fn write_context_state() -> ContextState {
    let dir = tempdir().unwrap();
    let cache_path = dir.path().join("cache");
    let lock_path = dir.path().join("commandf.lock");
    let cache = PackageCache::new(&cache_path);

    let parent_a_archive = package_archive(
        "acme.parenta",
        "1.0.0",
        &[
            (
                "package/StructureDefinition-profile.json",
                br#"{
                  "resourceType":"StructureDefinition",
                  "id":"profile",
                  "url":"https://example.org/StructureDefinition/profile",
                  "version":"1.0.0",
                  "baseDefinition":"https://example.org/StructureDefinition/base|1.0.0",
                  "differential":{"element":[{
                    "id":"Observation.subject",
                    "type":[{
                      "code":"Reference",
                      "profile":["https://example.org/StructureDefinition/shared"],
                      "targetProfile":["https://external.example/StructureDefinition/missing"]
                    }],
                    "binding":{"valueSet":"https://example.org/ValueSet/binding|1.0.0"}
                  }]}
                }"#,
            ),
            (
                "package/ValueSet-refs.json",
                br#"{
                  "resourceType":"ValueSet",
                  "id":"refs",
                  "url":"https://example.org/ValueSet/refs",
                  "version":"1.0.0",
                  "compose":{
                    "include":[{
                      "system":"https://example.org/CodeSystem/system|1.0.0",
                      "valueSet":["https://example.org/ValueSet/imported|1.0.0"]
                    }],
                    "exclude":[{"system":"https://external.example/CodeSystem/missing"}]
                  }
                }"#,
            ),
            (
                "package/Patient-unsupported.json",
                br#"{"resourceType":"Patient","id":"unsupported"}"#,
            ),
        ],
    );
    let parent_b_archive = package_archive(
        "acme.parentb",
        "1.0.0",
        &[
            (
                "package/StructureDefinition-extension.json",
                br#"{
                  "resourceType":"StructureDefinition",
                  "id":"extension",
                  "url":"https://example.org/StructureDefinition/extension",
                  "version":"1.0.0",
                  "type":"Extension",
                  "baseDefinition":"https://example.org/StructureDefinition/profile|1.0.0",
                  "differential":{"element":[{
                    "id":"Extension.value[x]",
                    "type":[{"code":"Reference","profile":["https://example.org/StructureDefinition/shared|2.0.0"]}]
                  }]}
                }"#,
            ),
            canonical_resource(
                "StructureDefinition",
                "base",
                "https://example.org/StructureDefinition/base",
                "1.0.0",
            ),
            canonical_resource(
                "ValueSet",
                "binding",
                "https://example.org/ValueSet/binding",
                "1.0.0",
            ),
            canonical_resource(
                "ValueSet",
                "imported",
                "https://example.org/ValueSet/imported",
                "1.0.0",
            ),
            canonical_resource(
                "CodeSystem",
                "system",
                "https://example.org/CodeSystem/system",
                "1.0.0",
            ),
            (
                "package/CodeSystem-supplement.json",
                br#"{
                  "resourceType":"CodeSystem",
                  "id":"supplement",
                  "url":"https://example.org/CodeSystem/supplement",
                  "version":"1.0.0",
                  "supplements":"https://example.org/CodeSystem/system|1.0.0"
                }"#,
            ),
        ],
    );
    let shared_v1_archive = package_archive(
        "acme.shared",
        "1.0.0",
        &[canonical_resource(
            "StructureDefinition",
            "shared-v1",
            "https://example.org/StructureDefinition/shared",
            "1.0.0",
        )],
    );
    let shared_v2_archive = package_archive(
        "acme.shared",
        "2.0.0",
        &[canonical_resource(
            "StructureDefinition",
            "shared-v2",
            "https://example.org/StructureDefinition/shared",
            "2.0.0",
        )],
    );

    let parent_a_sha = cache.put(&parent_a_archive).unwrap();
    let parent_b_sha = cache.put(&parent_b_archive).unwrap();
    let shared_v1_sha = cache.put(&shared_v1_archive).unwrap();
    let shared_v2_sha = cache.put(&shared_v2_archive).unwrap();

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
            locked_package(
                "acme.shared",
                "1.0.0",
                &shared_v1_sha,
                BTreeMap::new(),
            ),
            locked_package(
                "acme.shared",
                "2.0.0",
                &shared_v2_sha,
                BTreeMap::new(),
            ),
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

    ContextState {
        _dir: dir,
        lock: lock_path,
        cache: cache_path,
    }
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

fn canonical_resource(
    resource_type: &'static str,
    id: &'static str,
    url: &'static str,
    version: &'static str,
) -> (&'static str, &'static [u8]) {
    let filename = Box::leak(format!("package/{resource_type}-{id}.json").into_boxed_str());
    let body = Box::leak(
        format!(
            "{{\"resourceType\":\"{resource_type}\",\"id\":\"{id}\",\"url\":\"{url}\",\"version\":\"{version}\"}}"
        )
        .into_bytes()
        .into_boxed_slice(),
    );
    (filename, body)
}

fn package_archive(name: &str, version: &str, resources: &[(&str, &[u8])]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        let manifest = format!("{{\"name\":\"{name}\",\"version\":\"{version}\"}}");
        append_entry(&mut builder, "package/package.json", manifest.as_bytes());
        for (path, body) in resources {
            append_entry(&mut builder, path, body);
        }
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

fn append_entry(builder: &mut Builder<&mut GzEncoder<Vec<u8>>>, path: &str, body: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_path(path).unwrap();
    header.set_size(body.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append(&header, Cursor::new(body)).unwrap();
}
