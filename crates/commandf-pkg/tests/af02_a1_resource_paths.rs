use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[allow(dead_code)]
#[path = "../../../tools/af02-verifier/src/canonical.rs"]
mod canonical;

mod corpus {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ExpectedOutcome {
        AcceptCanonical,
        RejectInvalid,
        FailClosedLimit,
    }
}

#[path = "../../../tools/af02-verifier/src/replay.rs"]
mod replay;
#[rustfmt::skip]
#[allow(dead_code)]
#[path = "../../../tools/af02-verifier/src/resource.rs"]
mod resource;

use corpus::ExpectedOutcome;
use replay::{HarnessFailure, NormalizedOutcome, ReplayExecution, SurfaceObservation};
use resource::{parse_resource_policy, ResourcePolicy, RunnerOutcome};

const RESOURCE_POLICY: &str = "specs/016-af-02-adversarial-test-strength/resource-policy.json";
const PROPERTY_TESTS: &[&str] = &[
    "adversarial_properties",
    "archive_manifest_properties",
    "lockfile_v2_properties",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("commandf-pkg must live under crates/ in the repository")
        .to_path_buf()
}

fn canonical_policy(root: &Path) -> ResourcePolicy {
    parse_resource_policy(&fs::read(root.join(RESOURCE_POLICY)).expect("read resource policy"))
        .expect("parse canonical resource policy")
}

fn completed() -> ReplayExecution {
    ReplayExecution::Completed {
        runner: RunnerOutcome {
            image: "pinned".to_owned(),
            image_digest: "sha256:test".to_owned(),
            container_id: "container".to_owned(),
            process_exit_code: 0,
            stdout_sha256: "0".repeat(64),
            stderr_sha256: "0".repeat(64),
            stdout_bytes: 0,
            stderr_bytes: 0,
            output_regular_files: 0,
            output_total_bytes: 0,
        },
    }
}

#[test]
fn af02_t035_normalizer_preserves_surface_failures_and_unexpected_acceptance() {
    let execution = completed();
    let cases = [
        (
            ExpectedOutcome::AcceptCanonical,
            Some(SurfaceObservation::AcceptCanonical),
            NormalizedOutcome::AcceptCanonical,
        ),
        (
            ExpectedOutcome::RejectInvalid,
            Some(SurfaceObservation::RejectInvalid),
            NormalizedOutcome::RejectInvalid,
        ),
        (
            ExpectedOutcome::FailClosedLimit,
            Some(SurfaceObservation::FailClosedLimit),
            NormalizedOutcome::FailClosedLimit,
        ),
        (
            ExpectedOutcome::RejectInvalid,
            Some(SurfaceObservation::AcceptCanonical),
            NormalizedOutcome::UnexpectedAcceptance,
        ),
        (
            ExpectedOutcome::FailClosedLimit,
            Some(SurfaceObservation::AcceptCanonical),
            NormalizedOutcome::UnexpectedAcceptance,
        ),
        (
            ExpectedOutcome::AcceptCanonical,
            Some(SurfaceObservation::InvariantViolation),
            NormalizedOutcome::InvariantViolation,
        ),
        (
            ExpectedOutcome::AcceptCanonical,
            Some(SurfaceObservation::OracleDivergence),
            NormalizedOutcome::OracleDivergence,
        ),
        (
            ExpectedOutcome::AcceptCanonical,
            Some(SurfaceObservation::PanicOrAbort),
            NormalizedOutcome::PanicOrAbort,
        ),
        (
            ExpectedOutcome::AcceptCanonical,
            None,
            NormalizedOutcome::HarnessInternalError,
        ),
    ];

    for (expected, observed, normalized) in cases {
        assert_eq!(
            replay::normalize_result(expected, &execution, observed),
            normalized
        );
    }
}

#[test]
fn af02_t035_harness_failures_never_collapse_into_surface_outcomes() {
    let cases = [
        (HarnessFailure::Timeout, NormalizedOutcome::HarnessTimeout),
        (
            HarnessFailure::MemoryLimit,
            NormalizedOutcome::HarnessMemoryLimit,
        ),
        (
            HarnessFailure::FilesystemLimit,
            NormalizedOutcome::HarnessFilesystemLimit,
        ),
        (
            HarnessFailure::ProcessLimit,
            NormalizedOutcome::HarnessProcessLimit,
        ),
        (
            HarnessFailure::InternalError,
            NormalizedOutcome::HarnessInternalError,
        ),
    ];

    for (failure, normalized) in cases {
        let execution = ReplayExecution::HarnessFailure { failure };
        assert_eq!(
            replay::normalize_result(
                ExpectedOutcome::AcceptCanonical,
                &execution,
                Some(SurfaceObservation::AcceptCanonical),
            ),
            normalized
        );
    }
}

#[test]
fn af02_t035_replay_role_is_bound_to_the_canonical_resource_runner() {
    let root = repo_root();
    let replay_source = fs::read_to_string(root.join("tools/af02-verifier/src/replay.rs"))
        .expect("read replay authority source");
    let inventory = fs::read_to_string(
        root.join("specs/016-af-02-adversarial-test-strength/enforcement-inventory.json"),
    )
    .expect("read enforcement inventory");
    let corpus_source = fs::read_to_string(root.join("tools/af02-verifier/src/corpus.rs"))
        .expect("read corpus authority source");

    assert!(replay_source.contains("use crate::resource::{"));
    assert!(replay_source.contains("run_bounded"));
    assert!(!replay_source.contains("std::process::Command"));
    assert!(replay_source.contains("pub fn run_replay("));
    assert!(replay_source.contains("pub fn normalize_result("));
    assert!(inventory.contains("\"role\":\"REPLAY_RUNNER\""));
    assert!(inventory.contains("\"role\":\"RESULT_NORMALIZER\""));
    assert!(inventory.contains("\"planned_path\":\"tools/af02-verifier/src/replay.rs\""));
    for variant in ["AcceptCanonical", "RejectInvalid", "FailClosedLimit"] {
        assert!(
            corpus_source.contains(variant),
            "test-side ExpectedOutcome mirror drifted from corpus authority: {variant}"
        );
    }
}

fn command_output(command: &mut Command, context: &str) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("failed to run {context}: {error}"));
    assert!(
        output.status.success(),
        "{context} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[cfg(unix)]
fn host_user() -> String {
    let uid = command_output(Command::new("id").arg("-u"), "id -u");
    let gid = command_output(Command::new("id").arg("-g"), "id -g");
    format!(
        "{}:{}",
        String::from_utf8_lossy(&uid.stdout).trim(),
        String::from_utf8_lossy(&gid.stdout).trim()
    )
}

#[cfg(unix)]
fn prepare_property_binaries(root: &Path, image: &str, build: &Path) -> Vec<PathBuf> {
    let user = host_user();
    let output = command_output(
        Command::new("docker")
            .arg("run")
            .arg("--rm")
            .arg("--user")
            .arg(&user)
            .arg("--mount")
            .arg(format!(
                "type=bind,src={},dst=/workspace,readonly",
                root.display()
            ))
            .arg("--mount")
            .arg(format!("type=bind,src={},dst=/build", build.display()))
            .arg("--workdir")
            .arg("/workspace")
            .arg(image)
            .arg("bash")
            .arg("-ceu")
            .arg(
                "export CARGO_HOME=/build/cargo-home CARGO_TARGET_DIR=/build/target CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0; cargo test --manifest-path fuzz/Cargo.toml --locked --tests --no-run --message-format=json",
            ),
        "network-enabled property preparation",
    );

    let mut binaries = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if value.get("reason").and_then(serde_json::Value::as_str) != Some("compiler-artifact") {
            continue;
        }
        let Some(name) = value
            .pointer("/target/name")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        if !PROPERTY_TESTS.contains(&name) {
            continue;
        }
        let executable = value
            .get("executable")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("property test {name} lacks executable"));
        let relative = Path::new(executable)
            .strip_prefix("/build/")
            .unwrap_or_else(|_| panic!("unexpected property executable path {executable}"));
        binaries.insert(name.to_owned(), build.join(relative));
    }

    assert_eq!(binaries.len(), PROPERTY_TESTS.len());
    PROPERTY_TESTS
        .iter()
        .map(|name| {
            binaries
                .remove(*name)
                .unwrap_or_else(|| panic!("missing prepared property binary {name}"))
        })
        .collect()
}

#[cfg(unix)]
#[test]
fn af02_t035_build_replay_and_property_paths_execute_under_canonical_oci() {
    if std::env::var_os("GITHUB_ACTIONS").as_deref() != Some(std::ffi::OsStr::new("true")) {
        eprintln!("AF-02 T035 canonical OCI execution runs only inside GitHub Actions");
        return;
    }

    let root = repo_root();
    let policy = canonical_policy(&root);
    let image = format!("docker.io/library/rust@{}", policy.runner_image_digest);
    command_output(
        Command::new("docker").arg("pull").arg(&image),
        "pre-acquire pinned AF-02 runner image",
    );

    let target_root = root.join("target");
    fs::create_dir_all(&target_root).expect("create target root");
    let build = tempfile::Builder::new()
        .prefix("af02-t035-build-")
        .tempdir_in(&target_root)
        .expect("create property preparation directory");
    let binaries = prepare_property_binaries(&root, &image, build.path());

    let build_output = tempfile::tempdir().expect("create bounded build output");
    let build_command = vec![
        "rustc".to_owned(),
        "/workspace/tests/assurance/af02_models/portable_path.rs".to_owned(),
        "--crate-name".to_owned(),
        "af02_portable_path_model".to_owned(),
        "--crate-type".to_owned(),
        "lib".to_owned(),
        "--edition".to_owned(),
        "2021".to_owned(),
        "-C".to_owned(),
        "debuginfo=0".to_owned(),
        "-o".to_owned(),
        "/output/libaf02_portable_path_model.rlib".to_owned(),
    ];
    let build_execution = replay::run_replay(&policy, &root, build_output.path(), &build_command);
    let ReplayExecution::Completed { runner } = build_execution else {
        panic!("bounded AF-02 build path did not complete: {build_execution:?}");
    };
    assert_eq!(runner.process_exit_code, 0);
    assert_eq!(runner.image_digest, policy.runner_image_digest);
    assert_eq!(runner.output_regular_files, 1);
    assert!(runner.output_total_bytes > 0);

    for binary in binaries {
        let relative = binary
            .strip_prefix(&root)
            .expect("prepared property binary must remain inside repository source mount");
        let output = tempfile::tempdir().expect("create bounded property output");
        let command = vec![
            format!("/workspace/{}", relative.to_string_lossy()),
            "--test-threads=1".to_owned(),
        ];
        let execution = replay::run_replay(&policy, &root, output.path(), &command);
        let ReplayExecution::Completed { runner } = execution else {
            panic!("bounded AF-02 property path did not complete: {execution:?}");
        };
        assert_eq!(runner.process_exit_code, 0);
        assert_eq!(runner.image_digest, policy.runner_image_digest);
        assert_eq!(runner.output_regular_files, 0);
        assert_eq!(runner.output_total_bytes, 0);
    }
}
