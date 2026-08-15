use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::MAX_CORPUS_MANIFEST_BYTES;

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

fn valid_manifest() -> &'static [u8] {
    br#"{
  "schema": 1,
  "selection_policy": "frozen_pre_result_v1",
  "cases": [
    {
      "id": "C001",
      "package": "example.package",
      "before": {
        "version": "1.0.0",
        "archive_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "archive_bytes": 1,
        "publication_url": "https://example.org/before"
      },
      "after": {
        "version": "2.0.0",
        "archive_sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "archive_bytes": 1,
        "publication_url": "https://example.org/after"
      },
      "fhir_version": "4.0.1",
      "publisher": "Example Publisher",
      "change_evidence_url": "https://example.org/changes",
      "rights_evidence_url": "https://example.org/rights",
      "rights_mode": "metadata_only_no_redistribution",
      "oracle_mode": "changed_structure_definitions_only"
    }
  ]
}
"#
}

#[test]
fn corpus_run_help_is_available() {
    let output = commandf()
        .args(["corpus", "run", "--help"])
        .output()
        .expect("commandf corpus run --help must execute");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--manifest"));
    assert!(stdout.contains("--work-root"));
    assert!(stdout.contains("--oracle-adapter"));
}

#[test]
fn oversized_manifest_fails_before_work_root_creation() {
    let root = unique_temp_dir("corpus-oversized");
    fs::create_dir_all(&root).expect("create test root");
    let manifest = root.join("oversized.json");
    let work_root = root.join("work");
    fs::write(&manifest, vec![b' '; MAX_CORPUS_MANIFEST_BYTES + 1]).expect("write oversized manifest");

    let output = commandf()
        .args([
            "corpus",
            "run",
            "--manifest",
            manifest.to_str().expect("UTF-8 path"),
            "--work-root",
            work_root.to_str().expect("UTF-8 path"),
            "--oracle-adapter",
            root.join("missing-oracle.jar").to_str().expect("UTF-8 path"),
            "--format",
            "json",
        ])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("commandf corpus run must execute");

    assert_eq!(output.status.code(), Some(1));
    assert!(!work_root.exists());
    assert!(output.stdout.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn existing_work_root_fails_before_acquisition() {
    let root = unique_temp_dir("corpus-existing-root");
    let work_root = root.join("work");
    fs::create_dir_all(&work_root).expect("create existing work root");
    let marker = work_root.join("do-not-delete.txt");
    fs::write(&marker, b"preserve").expect("write marker");
    let manifest = root.join("corpus.json");
    fs::write(&manifest, valid_manifest()).expect("write manifest");
    let oracle = root.join("missing-oracle.jar");

    let output = commandf()
        .args([
            "corpus",
            "run",
            "--manifest",
            manifest.to_str().expect("UTF-8 path"),
            "--work-root",
            work_root.to_str().expect("UTF-8 path"),
            "--oracle-adapter",
            oracle.to_str().expect("UTF-8 path"),
            "--format",
            "json",
        ])
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("commandf corpus run must execute");

    assert_eq!(output.status.code(), Some(1));
    assert!(marker.exists(), "existing work root content must be preserved");
    assert!(!oracle.exists(), "oracle must not be touched before work-root gate");
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("work root already exists"));
    let _ = fs::remove_dir_all(root);
}
