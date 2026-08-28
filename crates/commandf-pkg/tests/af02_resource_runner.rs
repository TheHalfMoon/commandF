use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const VERIFIER_MANIFEST: &str = "tools/af02-verifier/Cargo.toml";
const RESOURCE_POLICY: &str = "specs/016-af-02-adversarial-test-strength/resource-policy.json";
const RUNNER_IMAGE: &str = "docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("commandf-pkg must live under crates/ in the repository")
        .to_path_buf()
}

fn run(root: &Path, program: &str, args: &[&str]) -> Output {
    Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|error| panic!("failed to run {program} {args:?}: {error}"))
}

fn verifier(root: &Path, source: &Path, output: &Path, command: &[&str]) -> Output {
    let mut invocation = vec![
        "run",
        "--quiet",
        "--locked",
        "--manifest-path",
        VERIFIER_MANIFEST,
        "--",
        "run-bounded",
        RESOURCE_POLICY,
        source.to_str().expect("source path must be UTF-8"),
        output.to_str().expect("output path must be UTF-8"),
        "--",
    ];
    invocation.extend_from_slice(command);
    run(root, "cargo", &invocation)
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure_contains(output: &Output, needle: &str, context: &str) {
    assert!(
        !output.status.success(),
        "{context} unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(needle),
        "{context} did not contain {needle:?}\nstdout:\n{}\nstderr:\n{stderr}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn af02_resource_runner_uses_real_pinned_oci_isolation_and_bounds() {
    if std::env::var_os("GITHUB_ACTIONS").as_deref() != Some(OsStr::new("true")) {
        eprintln!("AF-02 real OCI qualification runs only inside GitHub Actions");
        return;
    }

    let root = repo_root();
    let pull = run(&root, "docker", &["pull", RUNNER_IMAGE]);
    assert_success(&pull, "pre-acquire pinned AF-02 runner image");

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must follow Unix epoch")
        .as_nanos();
    let scratch = std::env::temp_dir().join(format!(
        "commandf-af02-resource-{}-{nonce}",
        std::process::id()
    ));
    let source = scratch.join("source");
    fs::create_dir_all(&source).expect("create source fixture");
    fs::write(source.join("input.txt"), b"read-only-source\n").expect("write source fixture");

    let success_output = scratch.join("output-success");
    fs::create_dir_all(&success_output).expect("create success output fixture");
    let success = verifier(
        &root,
        &source,
        &success_output,
        &["bash", "-ceu", "printf 'bounded-ok\\n' > /output/result.txt"],
    );
    assert_success(&success, "real bounded OCI success probe");
    assert_eq!(success.stdout.first().copied(), Some(b'{'));
    assert_eq!(
        fs::read(success_output.join("result.txt")).expect("read bounded output"),
        b"bounded-ok\n"
    );

    let temp_output = scratch.join("output-temp-limit");
    fs::create_dir_all(&temp_output).expect("create temp-limit output fixture");
    let temp_limit = verifier(
        &root,
        &source,
        &temp_output,
        &[
            "bash",
            "-ceu",
            "for i in $(seq 1 1025); do : > \"/tmp/af02-$i\"; done",
        ],
    );
    assert_failure_contains(
        &temp_limit,
        "AF02_RESOURCE_PROBE_FAIL=TEMP_FILE_LIMIT",
        "real bounded OCI temp-file negative probe",
    );

    let symlink_output = scratch.join("output-symlink");
    fs::create_dir_all(&symlink_output).expect("create symlink output fixture");
    let symlink = verifier(
        &root,
        &source,
        &symlink_output,
        &["bash", "-ceu", "ln -s /etc/passwd /output/escape"],
    );
    assert_failure_contains(
        &symlink,
        "symlink output is prohibited",
        "real bounded OCI symlink negative probe",
    );

    fs::remove_dir_all(&scratch).expect("remove AF-02 OCI qualification fixtures");
}
