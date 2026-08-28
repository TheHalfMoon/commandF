use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::canonical::parse_json_no_duplicates;

const RESOURCE_SCHEMA: &str = "commandf.af02-resource-policy/v1";
const RESOURCE_POLICY_PATH: &str =
    "specs/016-af-02-adversarial-test-strength/resource-policy.json";
const LINEAGE_RULE: &str = "BASE_CONTROLLED_PREDECESSOR_OR_SINGLE_BOOTSTRAP";
const RUNNER_IMAGE_REPOSITORY: &str = "docker.io/library/rust";
const RUNNER_IMAGE_DIGEST: &str =
    "sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f";
const RUNNER_IMAGE: &str = "docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f";
const SOURCE_MOUNT: &str = "/workspace";
const OUTPUT_MOUNT: &str = "/output";
const TMP_MOUNT: &str = "/tmp";
const CAPTURE_LIMIT_BYTES: usize = 16 * 1024 * 1024;

const PROBE_WRAPPER: &str = r#"set -euo pipefail
fail_probe() {
  printf 'AF02_RESOURCE_PROBE_FAIL=%s\n' "$1" >&2
  exit 125
}
if touch /.af02-root-write-probe 2>/dev/null; then
  fail_probe ROOT_WRITABLE
fi
if touch /workspace/.af02-source-write-probe 2>/dev/null; then
  fail_probe SOURCE_WRITABLE
fi
touch /output/.af02-output-write-probe 2>/dev/null || fail_probe OUTPUT_NOT_WRITABLE
rm -f /output/.af02-output-write-probe
if command -v timeout >/dev/null 2>&1; then
  if timeout 2 bash -c 'exec 3<>/dev/tcp/1.1.1.1/53' 2>/dev/null; then
    fail_probe NETWORK_REACHABLE
  fi
else
  fail_probe TIMEOUT_COMMAND_MISSING
fi
set +e
"$@"
command_status=$?
set -e
tmp_files="$(find /tmp -xdev -type f | wc -l)"
tmp_bytes="$(du -sb /tmp | awk '{print $1}')"
if [ "$tmp_files" -gt "$AF02_MAX_TEMP_FILES" ]; then
  fail_probe TEMP_FILE_LIMIT
fi
if [ "$tmp_bytes" -gt "$AF02_MAX_GENERATED_BYTES" ]; then
  fail_probe TEMP_BYTE_LIMIT
fi
exit "$command_status"
"#;

#[derive(Debug, Error)]
pub enum ResourceError {
    #[error("resource policy JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("resource policy violation: {0}")]
    Policy(String),
    #[error("invalid resource-runner path: {0}")]
    Path(String),
    #[error("I/O error for {path}: {source}")]
    Io { path: String, source: std::io::Error },
    #[error("command {program} failed: {stderr}")]
    Command { program: String, stderr: String },
    #[error("resource-runner output exceeded the closed capture limit")]
    CaptureLimit,
    #[error("resource-runner command timed out")]
    Timeout,
    #[error("Docker runtime inspection failed closed: {0}")]
    RuntimeInspection(String),
    #[error("artifact policy violation: {0}")]
    Artifact(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourcePolicy {
    pub artifact_retention_days: u64,
    pub campaign_wall_seconds: u64,
    pub cpu_count: u64,
    pub lineage: ResourceLineage,
    pub machine: String,
    pub max_committed_corpus_bytes: u64,
    pub max_decompressed_or_generated_bytes: u64,
    pub max_executions_or_zero_if_time_bounded: u64,
    pub max_input_bytes: u64,
    pub max_single_artifact_bytes: u64,
    pub max_temporary_files: u64,
    pub max_total_artifact_bytes: u64,
    pub network_mode: String,
    pub offline_required: bool,
    pub per_input_timeout_seconds: u64,
    pub pids_limit: u64,
    pub process_memory_mib: u64,
    pub runner_image_digest: String,
    pub schema: String,
    pub subprocess_timeout_seconds: u64,
    pub tmpfs_mib: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLineage {
    pub canonical_base_sha: String,
    pub canonical_base_tree: String,
    pub change_is_policy_only: bool,
    pub comparison_rule: String,
    pub dependent_evidence_allowed_in_same_candidate: bool,
    pub mode: ResourceLineageMode,
    pub policy_path: String,
    pub predecessor_blob_sha: Option<String>,
    pub predecessor_sha256: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResourceLineageMode {
    Bootstrap,
    Rebase,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DockerPlan {
    pub image: String,
    pub create_args: Vec<String>,
    pub source_dir: String,
    pub output_dir: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RunnerOutcome {
    pub image: String,
    pub image_digest: String,
    pub container_id: String,
    pub process_exit_code: i32,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub output_regular_files: u64,
    pub output_total_bytes: u64,
}

#[derive(Debug)]
struct CapturedProcess {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_overflow: bool,
    stderr_overflow: bool,
}

pub fn parse_resource_policy(bytes: &[u8]) -> Result<ResourcePolicy, ResourceError> {
    let value = parse_json_no_duplicates(bytes)?;
    let policy: ResourcePolicy = serde_json::from_value(value)?;
    validate_resource_policy(&policy)?;
    Ok(policy)
}

pub fn build_docker_plan(
    policy: &ResourcePolicy,
    source_dir: &Path,
    output_dir: &Path,
    command: &[String],
    user: &str,
    container_name: &str,
) -> Result<DockerPlan, ResourceError> {
    validate_resource_policy(policy)?;
    if command.is_empty() || command.iter().any(|argument| argument.contains('\0')) {
        return policy_error("bounded command must contain at least one NUL-free argument");
    }
    validate_container_name(container_name)?;
    validate_user(user)?;

    let source = validate_directory(source_dir, false)?;
    let output = validate_directory(output_dir, true)?;
    if output.starts_with(&source) || source.starts_with(&output) {
        return Err(ResourceError::Path(
            "source and output directories must be disjoint".to_owned(),
        ));
    }

    let mut args = vec![
        "create".to_owned(),
        "--pull=never".to_owned(),
        "--name".to_owned(),
        container_name.to_owned(),
        "--network=none".to_owned(),
        "--read-only".to_owned(),
        "--cap-drop=ALL".to_owned(),
        "--security-opt=no-new-privileges".to_owned(),
        "--cpus".to_owned(),
        policy.cpu_count.to_string(),
        "--memory".to_owned(),
        format!("{}m", policy.process_memory_mib),
        "--pids-limit".to_owned(),
        policy.pids_limit.to_string(),
        "--tmpfs".to_owned(),
        format!(
            "{TMP_MOUNT}:rw,noexec,nosuid,nodev,size={}m",
            policy.tmpfs_mib
        ),
        "--mount".to_owned(),
        format!(
            "type=bind,src={},dst={SOURCE_MOUNT},readonly",
            source.display()
        ),
        "--mount".to_owned(),
        format!("type=bind,src={},dst={OUTPUT_MOUNT}", output.display()),
        "--workdir".to_owned(),
        SOURCE_MOUNT.to_owned(),
        "--user".to_owned(),
        user.to_owned(),
        "--env".to_owned(),
        format!("AF02_MAX_TEMP_FILES={}", policy.max_temporary_files),
        "--env".to_owned(),
        format!(
            "AF02_MAX_GENERATED_BYTES={}",
            policy.max_decompressed_or_generated_bytes
        ),
        RUNNER_IMAGE.to_owned(),
        "bash".to_owned(),
        "-ceu".to_owned(),
        PROBE_WRAPPER.to_owned(),
        "--".to_owned(),
    ];
    args.extend(command.iter().cloned());

    Ok(DockerPlan {
        image: RUNNER_IMAGE.to_owned(),
        create_args: args,
        source_dir: source.display().to_string(),
        output_dir: output.display().to_string(),
        user: user.to_owned(),
    })
}

pub fn run_bounded(
    policy: &ResourcePolicy,
    source_dir: &Path,
    output_dir: &Path,
    command: &[String],
) -> Result<RunnerOutcome, ResourceError> {
    validate_resource_policy(policy)?;
    verify_docker_image(policy)?;
    let user = host_user()?;
    let container_name = unique_container_name();
    let plan = build_docker_plan(
        policy,
        source_dir,
        output_dir,
        command,
        &user,
        &container_name,
    )?;

    let create = run_simple("docker", &plan.create_args)?;
    let container_id = String::from_utf8_lossy(&create.stdout).trim().to_owned();
    if container_id.is_empty() {
        return Err(ResourceError::RuntimeInspection(
            "docker create returned an empty container id".to_owned(),
        ));
    }

    let execution = execute_created_container(policy, output_dir, &plan, &container_id);
    let _ = run_simple(
        "docker",
        &[
            "rm".to_owned(),
            "--force".to_owned(),
            container_id,
        ],
    );
    execution
}

fn execute_created_container(
    policy: &ResourcePolicy,
    output_dir: &Path,
    plan: &DockerPlan,
    container_id: &str,
) -> Result<RunnerOutcome, ResourceError> {
    let inspect = run_simple(
        "docker",
        &["inspect".to_owned(), container_id.to_owned()],
    )?;
    let inspect_value = parse_json_no_duplicates(&inspect.stdout)?;
    verify_runtime_inspection(policy, &inspect_value, plan)?;

    let timeout = Duration::from_secs(
        policy
            .subprocess_timeout_seconds
            .min(policy.campaign_wall_seconds),
    );
    let mut start = Command::new("docker");
    start.args(["start", "--attach", container_id]);
    let captured = run_captured_bounded(&mut start, timeout, CAPTURE_LIMIT_BYTES)?;
    if captured.stdout_overflow || captured.stderr_overflow {
        return Err(ResourceError::CaptureLimit);
    }

    let state = run_simple(
        "docker",
        &[
            "inspect".to_owned(),
            "--format={{.State.ExitCode}}".to_owned(),
            container_id.to_owned(),
        ],
    )?;
    let process_exit_code = String::from_utf8_lossy(&state.stdout)
        .trim()
        .parse::<i32>()
        .map_err(|_| {
            ResourceError::RuntimeInspection("container exit code is not an integer".to_owned())
        })?;
    let observed = captured.status.code().unwrap_or(-1);
    if observed != process_exit_code && observed != 125 {
        return Err(ResourceError::RuntimeInspection(format!(
            "docker attach exit {observed} disagrees with container exit {process_exit_code}"
        )));
    }
    if captured
        .stderr
        .windows(b"AF02_RESOURCE_PROBE_FAIL=".len())
        .any(|window| window == b"AF02_RESOURCE_PROBE_FAIL=")
    {
        return Err(ResourceError::RuntimeInspection(
            String::from_utf8_lossy(&captured.stderr).into_owned(),
        ));
    }

    let (output_regular_files, output_total_bytes) = validate_artifact_tree(output_dir, policy)?;

    Ok(RunnerOutcome {
        image: RUNNER_IMAGE.to_owned(),
        image_digest: RUNNER_IMAGE_DIGEST.to_owned(),
        container_id: container_id.to_owned(),
        process_exit_code,
        stdout_sha256: sha256_hex(&captured.stdout),
        stderr_sha256: sha256_hex(&captured.stderr),
        stdout_bytes: captured.stdout.len() as u64,
        stderr_bytes: captured.stderr.len() as u64,
        output_regular_files,
        output_total_bytes,
    })
}

fn validate_resource_policy(policy: &ResourcePolicy) -> Result<(), ResourceError> {
    if policy.schema != RESOURCE_SCHEMA {
        return policy_error("unexpected resource policy schema");
    }
    if policy.runner_image_digest != RUNNER_IMAGE_DIGEST {
        return policy_error("runner image digest disagrees with the closed canonical digest");
    }
    if policy.machine != "x86_64" {
        return policy_error("canonical resource runner machine must be x86_64");
    }
    if !policy.offline_required || policy.network_mode != "none" {
        return policy_error("canonical resource execution must be offline with network none");
    }
    bounded(policy.campaign_wall_seconds, 1, 3600, "campaign_wall_seconds")?;
    bounded(
        policy.max_executions_or_zero_if_time_bounded,
        0,
        100_000_000,
        "max_executions_or_zero_if_time_bounded",
    )?;
    bounded(
        policy.per_input_timeout_seconds,
        1,
        60,
        "per_input_timeout_seconds",
    )?;
    bounded(policy.max_input_bytes, 1, 262_144, "max_input_bytes")?;
    bounded(
        policy.process_memory_mib,
        64,
        4096,
        "process_memory_mib",
    )?;
    bounded(policy.cpu_count, 1, 8, "cpu_count")?;
    bounded(policy.pids_limit, 32, 1024, "pids_limit")?;
    bounded(policy.tmpfs_mib, 64, 2048, "tmpfs_mib")?;
    bounded(
        policy.max_decompressed_or_generated_bytes,
        1,
        67_108_864,
        "max_decompressed_or_generated_bytes",
    )?;
    bounded(
        policy.max_temporary_files,
        1,
        10_000,
        "max_temporary_files",
    )?;
    bounded(
        policy.subprocess_timeout_seconds,
        1,
        600,
        "subprocess_timeout_seconds",
    )?;
    bounded(
        policy.max_single_artifact_bytes,
        1,
        16_777_216,
        "max_single_artifact_bytes",
    )?;
    bounded(
        policy.max_total_artifact_bytes,
        1,
        134_217_728,
        "max_total_artifact_bytes",
    )?;
    if policy.max_committed_corpus_bytes != 8_388_608 {
        return policy_error("max_committed_corpus_bytes must equal 8 MiB");
    }
    bounded(
        policy.artifact_retention_days,
        1,
        30,
        "artifact_retention_days",
    )?;
    validate_lineage(&policy.lineage)?;
    Ok(())
}

fn validate_lineage(lineage: &ResourceLineage) -> Result<(), ResourceError> {
    validate_hex(&lineage.canonical_base_sha, 40, "canonical_base_sha")?;
    validate_hex(&lineage.canonical_base_tree, 40, "canonical_base_tree")?;
    if lineage.policy_path != RESOURCE_POLICY_PATH {
        return policy_error("resource lineage policy_path is not canonical");
    }
    if !lineage.change_is_policy_only || lineage.dependent_evidence_allowed_in_same_candidate {
        return policy_error("resource policy lineage violates policy-only temporal closure");
    }
    if lineage.comparison_rule != LINEAGE_RULE {
        return policy_error("unexpected resource policy comparison rule");
    }
    match lineage.mode {
        ResourceLineageMode::Bootstrap => {
            if lineage.predecessor_blob_sha.is_some() || lineage.predecessor_sha256.is_some() {
                return policy_error("BOOTSTRAP resource lineage cannot bind a predecessor");
            }
        }
        ResourceLineageMode::Rebase => {
            validate_hex(
                lineage.predecessor_blob_sha.as_deref().ok_or_else(|| {
                    ResourceError::Policy("REBASE resource lineage lacks predecessor blob".into())
                })?,
                40,
                "predecessor_blob_sha",
            )?;
            validate_hex(
                lineage.predecessor_sha256.as_deref().ok_or_else(|| {
                    ResourceError::Policy("REBASE resource lineage lacks predecessor digest".into())
                })?,
                64,
                "predecessor_sha256",
            )?;
        }
    }
    Ok(())
}

fn verify_docker_image(policy: &ResourcePolicy) -> Result<(), ResourceError> {
    let output = run_simple(
        "docker",
        &[
            "image".to_owned(),
            "inspect".to_owned(),
            "--format={{json .RepoDigests}}".to_owned(),
            RUNNER_IMAGE.to_owned(),
        ],
    )?;
    let digests = parse_json_no_duplicates(&output.stdout)?;
    let values = digests.as_array().ok_or_else(|| {
        ResourceError::RuntimeInspection("Docker RepoDigests is not an array".to_owned())
    })?;
    let expected_suffix = format!("@{}", policy.runner_image_digest);
    let matched = values.iter().filter_map(Value::as_str).any(|value| {
        let Some(repository) = value.strip_suffix(&expected_suffix) else {
            return false;
        };
        matches!(
            repository,
            "rust" | "library/rust" | "docker.io/library/rust"
        )
    });
    if !matched {
        return Err(ResourceError::RuntimeInspection(format!(
            "pre-acquired Docker image is not {RUNNER_IMAGE_REPOSITORY}@{}",
            policy.runner_image_digest
        )));
    }
    Ok(())
}

fn verify_runtime_inspection(
    policy: &ResourcePolicy,
    value: &Value,
    plan: &DockerPlan,
) -> Result<(), ResourceError> {
    let entries = value.as_array().ok_or_else(|| {
        ResourceError::RuntimeInspection("docker inspect result is not an array".to_owned())
    })?;
    if entries.len() != 1 {
        return Err(ResourceError::RuntimeInspection(format!(
            "docker inspect returned {} objects",
            entries.len()
        )));
    }
    let object = &entries[0];
    let host = object.get("HostConfig").and_then(Value::as_object).ok_or_else(|| {
        ResourceError::RuntimeInspection("missing HostConfig".to_owned())
    })?;
    let config = object.get("Config").and_then(Value::as_object).ok_or_else(|| {
        ResourceError::RuntimeInspection("missing Config".to_owned())
    })?;

    expect_string(host.get("NetworkMode"), "none", "HostConfig.NetworkMode")?;
    expect_bool(host.get("ReadonlyRootfs"), true, "HostConfig.ReadonlyRootfs")?;
    expect_i64(
        host.get("Memory"),
        mib_to_bytes(policy.process_memory_mib) as i64,
        "HostConfig.Memory",
    )?;
    expect_i64(
        host.get("NanoCpus"),
        (policy.cpu_count * 1_000_000_000) as i64,
        "HostConfig.NanoCpus",
    )?;
    expect_i64(
        host.get("PidsLimit"),
        policy.pids_limit as i64,
        "HostConfig.PidsLimit",
    )?;
    expect_string(config.get("User"), &plan.user, "Config.User")?;
    expect_string(
        config.get("WorkingDir"),
        SOURCE_MOUNT,
        "Config.WorkingDir",
    )?;

    let cap_drop = host
        .get("CapDrop")
        .and_then(Value::as_array)
        .ok_or_else(|| ResourceError::RuntimeInspection("missing CapDrop".to_owned()))?;
    if !cap_drop.iter().any(|item| item.as_str() == Some("ALL")) {
        return Err(ResourceError::RuntimeInspection(
            "CapDrop does not contain ALL".to_owned(),
        ));
    }
    let security = host
        .get("SecurityOpt")
        .and_then(Value::as_array)
        .ok_or_else(|| ResourceError::RuntimeInspection("missing SecurityOpt".to_owned()))?;
    if !security.iter().filter_map(Value::as_str).any(|item| {
        item == "no-new-privileges" || item == "no-new-privileges:true"
    }) {
        return Err(ResourceError::RuntimeInspection(
            "SecurityOpt does not enforce no-new-privileges".to_owned(),
        ));
    }

    let tmpfs = host
        .get("Tmpfs")
        .and_then(Value::as_object)
        .and_then(|map| map.get(TMP_MOUNT))
        .and_then(Value::as_str)
        .ok_or_else(|| ResourceError::RuntimeInspection("missing /tmp tmpfs".to_owned()))?;
    let expected_tmpfs = format!("size={}", mib_to_bytes(policy.tmpfs_mib));
    if !tmpfs.contains(&expected_tmpfs) || !tmpfs.contains("noexec") || !tmpfs.contains("nosuid") {
        return Err(ResourceError::RuntimeInspection(format!(
            "unexpected /tmp options {tmpfs}"
        )));
    }

    let mounts = host
        .get("Mounts")
        .and_then(Value::as_array)
        .ok_or_else(|| ResourceError::RuntimeInspection("missing HostConfig.Mounts".to_owned()))?;
    verify_bind_mount(mounts, &plan.source_dir, SOURCE_MOUNT, true)?;
    verify_bind_mount(mounts, &plan.output_dir, OUTPUT_MOUNT, false)?;
    Ok(())
}

fn verify_bind_mount(
    mounts: &[Value],
    source: &str,
    target: &str,
    read_only: bool,
) -> Result<(), ResourceError> {
    let matched = mounts.iter().any(|mount| {
        mount.get("Type").and_then(Value::as_str) == Some("bind")
            && mount.get("Source").and_then(Value::as_str) == Some(source)
            && mount.get("Target").and_then(Value::as_str) == Some(target)
            && mount.get("ReadOnly").and_then(Value::as_bool) == Some(read_only)
    });
    if !matched {
        return Err(ResourceError::RuntimeInspection(format!(
            "missing exact bind mount {source} -> {target} read_only={read_only}"
        )));
    }
    Ok(())
}

fn validate_artifact_tree(
    output_dir: &Path,
    policy: &ResourcePolicy,
) -> Result<(u64, u64), ResourceError> {
    let root = validate_directory(output_dir, false)?;
    let mut pending = vec![root];
    let mut files = 0_u64;
    let mut total = 0_u64;
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).map_err(|source| ResourceError::Io {
            path: directory.display().to_string(),
            source,
        })? {
            let entry = entry.map_err(|source| ResourceError::Io {
                path: directory.display().to_string(),
                source,
            })?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|source| ResourceError::Io {
                path: path.display().to_string(),
                source,
            })?;
            if metadata.file_type().is_symlink() {
                return Err(ResourceError::Artifact(format!(
                    "symlink output is prohibited: {}",
                    path.display()
                )));
            }
            if metadata.is_dir() {
                pending.push(path);
                continue;
            }
            if !metadata.is_file() {
                return Err(ResourceError::Artifact(format!(
                    "non-regular output is prohibited: {}",
                    path.display()
                )));
            }
            files = files.checked_add(1).ok_or_else(|| {
                ResourceError::Artifact("output file count overflow".to_owned())
            })?;
            if files > policy.max_temporary_files {
                return Err(ResourceError::Artifact(
                    "output file count exceeds max_temporary_files".to_owned(),
                ));
            }
            if metadata.len() > policy.max_single_artifact_bytes {
                return Err(ResourceError::Artifact(format!(
                    "single artifact exceeds {} bytes: {}",
                    policy.max_single_artifact_bytes,
                    path.display()
                )));
            }
            total = total.checked_add(metadata.len()).ok_or_else(|| {
                ResourceError::Artifact("output byte count overflow".to_owned())
            })?;
            if total > policy.max_total_artifact_bytes {
                return Err(ResourceError::Artifact(
                    "total output exceeds max_total_artifact_bytes".to_owned(),
                ));
            }
        }
    }
    Ok((files, total))
}

fn validate_directory(path: &Path, require_empty: bool) -> Result<PathBuf, ResourceError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| ResourceError::Io {
        path: path.display().to_string(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ResourceError::Path(path.display().to_string()));
    }
    if require_empty
        && fs::read_dir(path)
            .map_err(|source| ResourceError::Io {
                path: path.display().to_string(),
                source,
            })?
            .next()
            .is_some()
    {
        return Err(ResourceError::Path(format!(
            "output directory is not empty: {}",
            path.display()
        )));
    }
    fs::canonicalize(path).map_err(|source| ResourceError::Io {
        path: path.display().to_string(),
        source,
    })
}

fn run_simple(program: &str, args: &[String]) -> Result<std::process::Output, ResourceError> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|source| ResourceError::Io {
            path: program.to_owned(),
            source,
        })?;
    if !output.status.success() {
        return Err(ResourceError::Command {
            program: program.to_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        });
    }
    Ok(output)
}

fn run_captured_bounded(
    command: &mut Command,
    timeout: Duration,
    limit: usize,
) -> Result<CapturedProcess, ResourceError> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|source| ResourceError::Io {
        path: "docker start".to_owned(),
        source,
    })?;
    let stdout = child.stdout.take().ok_or_else(|| {
        ResourceError::RuntimeInspection("docker stdout was not captured".to_owned())
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        ResourceError::RuntimeInspection("docker stderr was not captured".to_owned())
    })?;
    let stdout_reader = thread::spawn(move || read_bounded(stdout, limit));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, limit));
    let started = Instant::now();
    let status = wait_with_timeout(&mut child, started, timeout)?;
    let (stdout, stdout_overflow) = stdout_reader
        .join()
        .map_err(|_| ResourceError::RuntimeInspection("stdout reader panicked".to_owned()))??;
    let (stderr, stderr_overflow) = stderr_reader
        .join()
        .map_err(|_| ResourceError::RuntimeInspection("stderr reader panicked".to_owned()))??;
    Ok(CapturedProcess {
        status,
        stdout,
        stderr,
        stdout_overflow,
        stderr_overflow,
    })
}

fn wait_with_timeout(
    child: &mut Child,
    started: Instant,
    timeout: Duration,
) -> Result<ExitStatus, ResourceError> {
    loop {
        if let Some(status) = child.try_wait().map_err(|source| ResourceError::Io {
            path: "docker start".to_owned(),
            source,
        })? {
            return Ok(status);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ResourceError::Timeout);
        }
        thread::sleep(Duration::from_millis(25));
    }
}

fn read_bounded<R: Read>(mut reader: R, limit: usize) -> Result<(Vec<u8>, bool), ResourceError> {
    let mut output = Vec::with_capacity(limit.min(64 * 1024));
    let mut overflow = false;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer).map_err(|source| ResourceError::Io {
            path: "docker output pipe".to_owned(),
            source,
        })?;
        if read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(output.len());
        let retained = remaining.min(read);
        output.extend_from_slice(&buffer[..retained]);
        if retained < read {
            overflow = true;
        }
    }
    Ok((output, overflow))
}

fn host_user() -> Result<String, ResourceError> {
    let uid = run_simple("id", &["-u".to_owned()])?;
    let gid = run_simple("id", &["-g".to_owned()])?;
    let uid = String::from_utf8_lossy(&uid.stdout).trim().to_owned();
    let gid = String::from_utf8_lossy(&gid.stdout).trim().to_owned();
    let user = format!("{uid}:{gid}");
    validate_user(&user)?;
    Ok(user)
}

fn unique_container_name() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("commandf-af02-{}-{nanos}", std::process::id())
}

fn validate_container_name(value: &str) -> Result<(), ResourceError> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return policy_error("invalid Docker container name");
    }
    Ok(())
}

fn validate_user(value: &str) -> Result<(), ResourceError> {
    let mut parts = value.split(':');
    let valid = parts
        .next()
        .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts
            .next()
            .is_some_and(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        && parts.next().is_none();
    if !valid {
        return policy_error("Docker user must be numeric uid:gid");
    }
    Ok(())
}

fn expect_string(value: Option<&Value>, expected: &str, label: &str) -> Result<(), ResourceError> {
    if value.and_then(Value::as_str) != Some(expected) {
        return Err(ResourceError::RuntimeInspection(format!(
            "{label} does not equal {expected}"
        )));
    }
    Ok(())
}

fn expect_bool(value: Option<&Value>, expected: bool, label: &str) -> Result<(), ResourceError> {
    if value.and_then(Value::as_bool) != Some(expected) {
        return Err(ResourceError::RuntimeInspection(format!(
            "{label} does not equal {expected}"
        )));
    }
    Ok(())
}

fn expect_i64(value: Option<&Value>, expected: i64, label: &str) -> Result<(), ResourceError> {
    if value.and_then(Value::as_i64) != Some(expected) {
        return Err(ResourceError::RuntimeInspection(format!(
            "{label} does not equal {expected}"
        )));
    }
    Ok(())
}

fn bounded(value: u64, minimum: u64, maximum: u64, label: &str) -> Result<(), ResourceError> {
    if !(minimum..=maximum).contains(&value) {
        return policy_error(format!("{label} is outside [{minimum}, {maximum}]"));
    }
    Ok(())
}

fn validate_hex(value: &str, len: usize, label: &str) -> Result<(), ResourceError> {
    if value.len() != len
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return policy_error(format!("invalid lowercase hex {label}"));
    }
    Ok(())
}

fn mib_to_bytes(value: u64) -> u64 {
    value * 1024 * 1024
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn policy_error<T>(message: impl Into<String>) -> Result<T, ResourceError> {
    Err(ResourceError::Policy(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANONICAL_POLICY: &[u8] = include_bytes!(
        "../../../specs/016-af-02-adversarial-test-strength/resource-policy.json"
    );

    fn policy() -> ResourcePolicy {
        parse_resource_policy(CANONICAL_POLICY).expect("canonical resource policy must parse")
    }

    #[test]
    fn canonical_resource_policy_parses() {
        let policy = policy();
        assert_eq!(policy.runner_image_digest, RUNNER_IMAGE_DIGEST);
        assert_eq!(policy.network_mode, "none");
        assert!(policy.offline_required);
    }

    #[test]
    fn docker_plan_freezes_isolation_and_bounds() {
        let root = std::env::temp_dir().join(format!(
            "commandf-af02-resource-plan-{}",
            std::process::id()
        ));
        let source = root.join("source");
        let output = root.join("output");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&output).unwrap();
        let plan = build_docker_plan(
            &policy(),
            &source,
            &output,
            &["rustc".to_owned(), "--version".to_owned()],
            "1000:1000",
            "commandf-af02-test",
        )
        .unwrap();
        assert_eq!(plan.image, RUNNER_IMAGE);
        let joined = plan.create_args.join("\n");
        for required in [
            "--pull=never",
            "--network=none",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--cpus",
            "2",
            "--memory",
            "768m",
            "--pids-limit",
            "256",
            RUNNER_IMAGE,
        ] {
            assert!(joined.lines().any(|line| line == required), "missing {required}");
        }
        assert!(joined.contains("/workspace,readonly"));
        assert!(joined.contains("/output"));
        assert!(joined.contains("/tmp:rw,noexec,nosuid,nodev,size=512m"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn output_directory_must_start_empty_and_disjoint() {
        let root = std::env::temp_dir().join(format!(
            "commandf-af02-resource-invalid-{}",
            std::process::id()
        ));
        let source = root.join("source");
        let output = root.join("output");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&output).unwrap();
        fs::write(output.join("unexpected"), b"x").unwrap();
        let error = build_docker_plan(
            &policy(),
            &source,
            &output,
            &["true".to_owned()],
            "1000:1000",
            "commandf-af02-test",
        )
        .unwrap_err();
        assert!(error.to_string().contains("not empty"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn runtime_probe_script_contains_all_closed_negative_probes() {
        for probe in [
            "ROOT_WRITABLE",
            "SOURCE_WRITABLE",
            "OUTPUT_NOT_WRITABLE",
            "NETWORK_REACHABLE",
            "TEMP_FILE_LIMIT",
            "TEMP_BYTE_LIMIT",
        ] {
            assert!(PROBE_WRAPPER.contains(probe), "missing probe {probe}");
        }
    }

    #[test]
    fn image_reference_is_digest_pinned() {
        assert_eq!(
            RUNNER_IMAGE,
            format!("{RUNNER_IMAGE_REPOSITORY}@{RUNNER_IMAGE_DIGEST}")
        );
    }
}
