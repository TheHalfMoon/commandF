use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::process::Command;

use commandf_pkg::{LockedPackage, Lockfile, PackageCache};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};
use tempfile::tempdir;

#[test]
fn context_cli_output_is_byte_identical_and_reports_sha256() {
    let dir = tempdir().unwrap();
    let cache_path = dir.path().join("cache");
    let lock_path = dir.path().join("commandf.lock");
    let cache = PackageCache::new(&cache_path);
    let archive = package_archive(&[(
        "package/StructureDefinition-proof.json",
        br#"{
          "resourceType":"StructureDefinition",
          "id":"proof",
          "url":"https://example.org/StructureDefinition/proof",
          "version":"1.0.0",
          "baseDefinition":"https://external.example/StructureDefinition/base"
        }"#,
    )]);
    let digest = cache.put(&archive).unwrap();
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
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(first.stdout, second.stdout);
    println!("CF11G_CONTEXT_SHA256={}", PackageCache::digest(&first.stdout));
}

fn run_context(lock: &std::path::Path, cache: &std::path::Path) -> std::process::Output {
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

fn package_archive(resources: &[(&str, &[u8])]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        append_entry(
            &mut builder,
            "package/package.json",
            br#"{"name":"acme.proof","version":"1.0.0"}"#,
        );
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
