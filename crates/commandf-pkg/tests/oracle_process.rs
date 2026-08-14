#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use commandf_pkg::{run_hl7_oracle_adapter, Hl7OracleInvocation};

const GOOD_REPORT: &str = r#"{"schema":1,"oracle":{"project":"hapifhir/org.hl7.fhir.core","release":"6.10.2","source_commit":"d06577dbc5c62c74a2a8823fbc4830a3024d5b0b"},"left":{"url":"http://example.org/StructureDefinition/test","version":null,"id":"test","type":"Patient"},"right":{"url":"http://example.org/StructureDefinition/test","version":null,"id":"test","type":"Patient"},"states":{"metadata":"not_changed","definitions":"not_changed","content":"unknown","content_interpretation":"unknown"},"messages":[]}"#;

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "commandf-oracle-process-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}\n")).expect("write executable");
    let mut permissions = fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("chmod executable");
}

fn package_inputs(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let core = root.join("core.tgz");
    let left = root.join("left.tgz");
    let right = root.join("right.tgz");
    for path in [&core, &left, &right] {
        fs::write(path, b"fixture").expect("write package fixture");
    }
    (core, left, right)
}

fn invoke(
    adapter: &Path,
    java: Option<&Path>,
    core: &Path,
    left: &Path,
    right: &Path,
    timeout: Duration,
) -> Result<commandf_pkg::Hl7OracleReport, commandf_pkg::OracleError> {
    let invocation = Hl7OracleInvocation {
        core_package: core,
        left_package: left,
        right_package: right,
        left_url: "http://example.org/StructureDefinition/test",
        left_version: None,
        right_url: "http://example.org/StructureDefinition/test",
        right_version: None,
    };
    run_hl7_oracle_adapter(adapter, java, &invocation, timeout)
}

#[test]
fn executable_adapter_accepts_valid_pinned_json() {
    let root = unique_temp_dir("valid");
    fs::create_dir_all(&root).expect("create temp dir");
    let adapter = root.join("adapter.sh");
    write_executable(&adapter, &format!("printf '%s\\n' '{}'", GOOD_REPORT));
    let (core, left, right) = package_inputs(&root);

    let report = invoke(
        &adapter,
        None,
        &core,
        &left,
        &right,
        Duration::from_secs(1),
    )
    .expect("valid adapter report");
    assert_eq!(report.schema, 1);
    assert!(report.messages.is_empty());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn jar_adapter_requires_explicit_java_path() {
    let root = unique_temp_dir("java-required");
    fs::create_dir_all(&root).expect("create temp dir");
    let adapter = root.join("adapter.jar");
    fs::write(&adapter, b"not-a-real-jar").expect("write jar fixture");
    let (core, left, right) = package_inputs(&root);

    let error = invoke(
        &adapter,
        None,
        &core,
        &left,
        &right,
        Duration::from_secs(1),
    )
    .expect_err("jar without explicit Java must fail");
    assert!(error.to_string().contains("--oracle-java is required"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn malformed_adapter_json_fails_closed() {
    let root = unique_temp_dir("malformed");
    fs::create_dir_all(&root).expect("create temp dir");
    let adapter = root.join("adapter.sh");
    write_executable(&adapter, "printf 'not-json\\n'");
    let (core, left, right) = package_inputs(&root);

    let error = invoke(
        &adapter,
        None,
        &core,
        &left,
        &right,
        Duration::from_secs(1),
    )
    .expect_err("malformed JSON must fail");
    assert!(error.to_string().contains("oracle report JSON is invalid"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn nonzero_adapter_exit_fails_closed_with_bounded_stderr() {
    let root = unique_temp_dir("exit");
    fs::create_dir_all(&root).expect("create temp dir");
    let adapter = root.join("adapter.sh");
    write_executable(&adapter, "printf 'adapter failed' >&2; exit 7");
    let (core, left, right) = package_inputs(&root);

    let error = invoke(
        &adapter,
        None,
        &core,
        &left,
        &right,
        Duration::from_secs(1),
    )
    .expect_err("nonzero exit must fail");
    let message = error.to_string();
    assert!(message.contains("code Some(7)"));
    assert!(message.contains("adapter failed"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn adapter_timeout_kills_the_process() {
    let root = unique_temp_dir("timeout");
    fs::create_dir_all(&root).expect("create temp dir");
    let adapter = root.join("adapter.sh");
    write_executable(&adapter, "sleep 1");
    let (core, left, right) = package_inputs(&root);

    let error = invoke(
        &adapter,
        None,
        &core,
        &left,
        &right,
        Duration::from_millis(20),
    )
    .expect_err("timeout must fail");
    assert!(error.to_string().contains("timed out"));
    let _ = fs::remove_dir_all(root);
}
