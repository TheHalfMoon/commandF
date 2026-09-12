use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const VERIFIER_MANIFEST: &str = "tools/af02-verifier/Cargo.toml";
const CORPUS_MANIFEST: &str = "specs/016-af-02-adversarial-test-strength/corpus-manifest.json";
const CORPUS_SCHEMA: &str =
    "specs/016-af-02-adversarial-test-strength/schemas/af02-corpus-v1.schema.json";
const ASSERTION_REGISTRY: &str =
    "specs/016-af-02-adversarial-test-strength/assertion-registry.json";
const SURFACE_POLICY: &str = "specs/016-af-02-adversarial-test-strength/surface-policy.json";

const FROZEN_MAX_FIXTURE_BYTES: u64 = 256 * 1024;
const FROZEN_MAX_TOTAL_BYTES: u64 = 8 * 1024 * 1024;

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

fn validate_corpus(
    root: &Path,
    corpus: &Path,
    schema: &Path,
    assertions: &Path,
    surface: &Path,
) -> Output {
    run(
        root,
        "cargo",
        &[
            "run",
            "--quiet",
            "--locked",
            "--manifest-path",
            VERIFIER_MANIFEST,
            "--",
            "validate-corpus-assertions",
            corpus.to_str().expect("corpus path must be UTF-8"),
            schema.to_str().expect("schema path must be UTF-8"),
            assertions.to_str().expect("assertion path must be UTF-8"),
            surface.to_str().expect("surface path must be UTF-8"),
            root.to_str().expect("repo root must be UTF-8"),
        ],
    )
}

fn assert_success(output: &Output, context: &str) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{context} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "{context} did not emit JSON on stdout: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context} unexpectedly succeeded\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn af02_t037_checked_in_corpus_proves_bijection_bounds_and_no_phi() {
    let root = repo_root();
    let corpus = root.join(CORPUS_MANIFEST);
    let schema = root.join(CORPUS_SCHEMA);
    let assertions = root.join(ASSERTION_REGISTRY);
    let surface = root.join(SURFACE_POLICY);

    let report = assert_success(
        &validate_corpus(&root, &corpus, &schema, &assertions, &surface),
        "checked-in corpus/assertion pair must validate",
    );
    assert_eq!(
        report.get("schema").and_then(serde_json::Value::as_str),
        Some("commandf.af02-corpus-assertion-validation/v1")
    );
    assert_eq!(
        report
            .get("scenario_count")
            .and_then(serde_json::Value::as_u64),
        report
            .get("assertion_count")
            .and_then(serde_json::Value::as_u64),
        "every corpus scenario must have exactly one assertion entry and vice versa"
    );

    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&corpus).expect("read checked-in corpus manifest"))
            .expect("checked-in corpus manifest must be JSON");
    assert_eq!(
        manifest
            .get("max_fixture_bytes")
            .and_then(serde_json::Value::as_u64),
        Some(FROZEN_MAX_FIXTURE_BYTES),
        "per-fixture limit must stay at the planning-frozen 256 KiB"
    );
    assert_eq!(
        manifest
            .get("max_total_bytes")
            .and_then(serde_json::Value::as_u64),
        Some(FROZEN_MAX_TOTAL_BYTES),
        "aggregate limit must stay at the planning-frozen 8 MiB"
    );

    let entries = manifest
        .get("entries")
        .and_then(serde_json::Value::as_array)
        .expect("corpus manifest must carry an entries array");
    let mut total_bytes = 0_u64;
    let mut scenarios = std::collections::BTreeSet::new();
    let mut assertion_ids = std::collections::BTreeSet::new();
    let mut replay_ids = std::collections::BTreeSet::new();
    for entry in entries {
        let scenario = entry
            .get("scenario_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("<missing scenario_id>");
        let byte_length = entry
            .get("byte_length")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| panic!("scenario {scenario} must declare byte_length"));
        assert!(
            byte_length <= FROZEN_MAX_FIXTURE_BYTES,
            "scenario {scenario} exceeds the per-fixture byte limit"
        );
        total_bytes = total_bytes
            .checked_add(byte_length)
            .expect("corpus byte total must not overflow");
        assert_eq!(
            entry
                .get("contains_phi")
                .and_then(serde_json::Value::as_bool),
            Some(false),
            "scenario {scenario} must not contain PHI"
        );
        let assertion_id = entry
            .get("assertion_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("scenario {scenario} must declare assertion_id"));
        let replay_id = entry
            .get("replay_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("scenario {scenario} must declare replay_id"));
        assert!(
            scenarios.insert(
                entry
                    .get("scenario_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(scenario)
            ),
            "duplicate scenario id {scenario}"
        );
        assert!(
            assertion_ids.insert(assertion_id),
            "duplicate corpus assertion id {assertion_id}"
        );
        assert!(
            replay_ids.insert(replay_id),
            "duplicate replay id {replay_id}"
        );
    }
    assert!(
        total_bytes <= FROZEN_MAX_TOTAL_BYTES,
        "corpus aggregate {total_bytes} exceeds the committed 8 MiB limit"
    );
}

#[test]
fn af02_t037_tampered_corpus_or_registry_cannot_self_green() {
    let root = repo_root();
    let corpus = root.join(CORPUS_MANIFEST);
    let schema = root.join(CORPUS_SCHEMA);
    let assertions = root.join(ASSERTION_REGISTRY);
    let surface = root.join(SURFACE_POLICY);

    let scratch = tempfile::Builder::new()
        .prefix("af02-t037-tamper-")
        .tempdir()
        .expect("create tamper scratch directory");

    let mut tampered_corpus = fs::read(&corpus).expect("read checked-in corpus manifest");
    tampered_corpus.push(b' ');
    let tampered_corpus_path = scratch.path().join("corpus-manifest.json");
    fs::write(&tampered_corpus_path, &tampered_corpus).expect("write tampered corpus");
    assert_failure(
        &validate_corpus(&root, &tampered_corpus_path, &schema, &assertions, &surface),
        "tampered corpus bytes must fail closed",
    );

    let mut tampered_registry = fs::read(&assertions).expect("read checked-in assertion registry");
    if !tampered_registry.is_empty() {
        let last = tampered_registry.len() - 1;
        tampered_registry[last] ^= 0x01;
    }
    let tampered_registry_path = scratch.path().join("assertion-registry.json");
    fs::write(&tampered_registry_path, &tampered_registry).expect("write tampered registry");
    assert_failure(
        &validate_corpus(&root, &corpus, &schema, &tampered_registry_path, &surface),
        "tampered assertion registry must fail closed",
    );
}
