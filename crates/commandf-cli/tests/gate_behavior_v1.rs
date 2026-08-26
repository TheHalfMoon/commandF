use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{
    finding_fingerprint_v1, CheckReport, FindingFingerprint, GateSuppression, GateSuppressions,
    LockedPackage, Lockfile, PackageCache, QualityGateDisposition, QualityGateReport,
};

const BEFORE_HEX: &str = concat!(
    "1f8b08000000000002ffed944d4fc3300c86fb5350cea31f63b452cf70e60037c4216bbd35d0a655924e43d3fe3beed66d6c",
    "abc4013409789f1edc388df336b1ddc8ec4dce2968b6d67fb5b5f67e98908927938d654e6d1826f1e1bdf347517c137957a1",
    "77015aeba4e1edbdffc94a68599148052d65d594e4f78920466241c6aa5af35ce4877ec89e9c1ad239e94c9115e96abdf6c0",
    "2fa7bfeee0d1993673ada13b9a29ad1c5ffcf52e25bedb13beaaff241e9fd4ff6d128e51ff97a97f43b66e4d464fef4dd707",
    "0612812b5fe58716c1c3d6943c2e9c6bd220d8a5496de64369141c969d7794bef9dcef3fe1db702d37172133a7169de74de9",
    "6ef39d4cf6c8a97586e7453a93a5a591705be90f534b66217bc953693fffc35e6e51261ba9b3429941bdc7617232aa1fa422",
    "ab75b7b5d2ae93aa65638b9a65ac049554117bd3e7d5f6ac8e8334d21567ce4a71c890ad5c762722d6a3f3b57e7f1e43210e",
    "73a7917627c262b23aa78d7036eb177ed0b3010000000000000000000000000000000000803fc7075ec49c6300280000",
);

const AFTER_HEX: &str = concat!(
    "1f8b08000000000002ffed944d4fc3300c86fb5350cea31f636c52cf70e60037c4216bbd35d0a555924e4353ff3b6ed7ad6c",
    "abc4013409789f1edc388df336b15dcae44d2e292877d67fb585f67e9890994e26ad654e6d18cea6fd7be38fa2e94de45d85",
    "de05a8ac9386b7f7fe275ba1e58a442c682357654e7e97086224d664ac2a34cf457ee487ec49a9249d924e1459116febda03",
    "bf9ceeba834767aac45586ee68a1b4727cf1d7fb94f86e4ff8aafe67d3f149fddfcec231eaff32f56fc8169549e8e9bd6cfa",
    "c0402270e5abb46f113cac4ccee3ccb9320e827d9a146639944641bfecbca374cde7fef009df86abb8b9089938b56e3c6f4a",
    "379bef65b247cead333c2fe285cc2d8d84db497f985b326bd9499e4bfbf91f0e72b37cd64a5d64ca0cea3d0e939251dd2016",
    "49a19bad95768d542d4b9b152c632b28a715b1377edeeeceea3848295d76e65c290e19b2959be644443d3a5feb77e73114a2",
    "9f6b23457da4fd89b098a448a915cea67ee1073d1b000000000000000000000000000000000000f8737c00934f6565002800",
    "00",
);

fn commandf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_commandf"))
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| (hex_digit(pair[0]) << 4) | hex_digit(pair[1]))
        .collect()
}

fn hex_digit(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => panic!("invalid test hex digit"),
    }
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("commandf-{label}-{}-{nonce}", std::process::id()))
}

fn write_locked_state(root: &Path, archive: &[u8], version: &str) -> (PathBuf, PathBuf) {
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    fs::create_dir_all(root).expect("create state root");
    let cache = PackageCache::new(&cache_path);
    let digest = cache.put(archive).expect("cache synthetic archive");
    let lockfile = Lockfile::new(
        vec![format!("example.package@{version}")],
        vec![LockedPackage {
            name: "example.package".to_owned(),
            version: version.to_owned(),
            sha256: digest,
            source: "synthetic-test".to_owned(),
            dependencies: BTreeMap::new(),
        }],
    );
    fs::write(&lock_path, lockfile.to_bytes().expect("serialize lock")).expect("write lock");
    (lock_path, cache_path)
}

fn changed_states(
    dir: &Path,
    before_version: &str,
    after_version: &str,
) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let before = decode_hex(BEFORE_HEX);
    let after = decode_hex(AFTER_HEX);
    let (before_lock, before_cache) =
        write_locked_state(&dir.join("before"), &before, before_version);
    let (after_lock, after_cache) = write_locked_state(&dir.join("after"), &after, after_version);
    (before_lock, before_cache, after_lock, after_cache)
}

fn run_command(
    subcommand: &str,
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
    extra: &[String],
) -> Output {
    let mut command = commandf();
    command.args([
        subcommand,
        "example.package",
        "--before-lock",
        before_lock.to_str().expect("UTF-8 path"),
        "--before-cache",
        before_cache.to_str().expect("UTF-8 path"),
        "--after-lock",
        after_lock.to_str().expect("UTF-8 path"),
        "--after-cache",
        after_cache.to_str().expect("UTF-8 path"),
    ]);
    command.args(extra);
    command
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .expect("commandf must execute")
}

fn current_check_report(
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
) -> CheckReport {
    let output = run_command(
        "check",
        before_lock,
        before_cache,
        after_lock,
        after_cache,
        &[],
    );
    assert_eq!(output.status.code(), Some(2));
    CheckReport::from_json_slice(&output.stdout).expect("valid check report")
}

fn write_baseline(
    path: &Path,
    before_lock: &Path,
    before_cache: &Path,
    after_lock: &Path,
    after_cache: &Path,
) {
    let report = run_command(
        "check",
        before_lock,
        before_cache,
        after_lock,
        after_cache,
        &[],
    );
    assert_eq!(report.status.code(), Some(2));
    fs::write(path, report.stdout).expect("write baseline report");
}

#[test]
fn gate_help_exposes_v1_contract() {
    let output = commandf()
        .args(["gate", "--help"])
        .output()
        .expect("gate help must execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("UTF-8 help");
    for flag in [
        "--before-lock",
        "--before-cache",
        "--after-lock",
        "--after-cache",
        "--direction",
        "--fail-on",
        "--baseline",
        "--suppressions",
        "--format",
        "--output",
    ] {
        assert!(stdout.contains(flag), "missing {flag}");
    }
}

#[test]
fn new_blocker_emits_complete_json_before_exit_two_and_replaces_output() {
    let dir = unique_temp_dir("gate-new-blocker");
    let (before_lock, before_cache, after_lock, after_cache) =
        changed_states(&dir, "1.0.0", "1.1.0");
    let output_path = dir.join("gate.json");
    fs::write(&output_path, b"stale-report").expect("write stale output");

    let output = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[
            "--output".to_owned(),
            output_path.to_str().expect("UTF-8 path").to_owned(),
        ],
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let bytes = fs::read(&output_path).expect("gate output exists");
    let report = QualityGateReport::from_json_slice(&bytes).expect("complete gate JSON");
    assert!(!report.decision.passed);
    assert!(report.decision.blocking_findings > 0);
    assert!(report
        .findings
        .iter()
        .all(|finding| finding.disposition == QualityGateDisposition::New));
    assert!(!String::from_utf8_lossy(&bytes).contains("stale-report"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn historical_baseline_allows_same_semantic_finding() {
    let dir = unique_temp_dir("gate-baseline");
    let (base_before_lock, base_before_cache, base_after_lock, base_after_cache) =
        changed_states(&dir.join("baseline-state"), "0.8.0", "0.9.0");
    let (before_lock, before_cache, after_lock, after_cache) =
        changed_states(&dir.join("current-state"), "1.0.0", "1.1.0");
    let baseline_path = dir.join("baseline.json");
    write_baseline(
        &baseline_path,
        &base_before_lock,
        &base_before_cache,
        &base_after_lock,
        &base_after_cache,
    );

    let output = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[
            "--baseline".to_owned(),
            baseline_path.to_str().expect("UTF-8 path").to_owned(),
        ],
    );

    assert_eq!(output.status.code(), Some(0));
    let report = QualityGateReport::from_json_slice(&output.stdout).expect("gate report");
    assert!(report.decision.passed);
    assert!(report.decision.baseline_findings > 0);
    assert_eq!(report.decision.blocking_findings, 0);
    assert!(report
        .findings
        .iter()
        .all(|finding| finding.disposition == QualityGateDisposition::Baseline));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn exact_suppression_passes_and_stale_suppression_does_not_hide_blocker() {
    let dir = unique_temp_dir("gate-suppression");
    let (before_lock, before_cache, after_lock, after_cache) =
        changed_states(&dir, "1.0.0", "1.1.0");
    let current = current_check_report(&before_lock, &before_cache, &after_lock, &after_cache);
    let exact = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: current
            .compatibility
            .findings
            .iter()
            .enumerate()
            .map(|(index, finding)| GateSuppression {
                finding_fingerprint: finding_fingerprint_v1(&current.compatibility.ruleset, finding)
                    .expect("fingerprint"),
                rationale: "approved interoperability exception".to_owned(),
                reference: Some(format!("TEST-{}", index + 1)),
            })
            .collect(),
    };
    let exact_path = dir.join("exact-suppressions.json");
    fs::write(
        &exact_path,
        exact.to_json_bytes().expect("suppression JSON"),
    )
    .expect("write exact suppression");

    let exact_output = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[
            "--suppressions".to_owned(),
            exact_path.to_str().expect("UTF-8 path").to_owned(),
        ],
    );
    assert_eq!(exact_output.status.code(), Some(0));
    let exact_report =
        QualityGateReport::from_json_slice(&exact_output.stdout).expect("exact gate report");
    assert!(exact_report.decision.passed);
    assert_eq!(exact_report.decision.blocking_findings, 0);
    assert!(exact_report
        .findings
        .iter()
        .all(|finding| finding.disposition == QualityGateDisposition::Suppressed));

    let stale_path = dir.join("stale-suppressions.json");
    let stale = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: FindingFingerprint {
                schema: FindingFingerprint::SCHEMA_V1,
                digest: format!("sha256:{}", "f".repeat(64)),
            },
            rationale: "stale exception".to_owned(),
            reference: None,
        }],
    };
    fs::write(&stale_path, stale.to_json_bytes().expect("stale JSON"))
        .expect("write stale suppression");
    let stale_output = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[
            "--suppressions".to_owned(),
            stale_path.to_str().expect("UTF-8 path").to_owned(),
        ],
    );
    assert_eq!(stale_output.status.code(), Some(2));
    let stale_report =
        QualityGateReport::from_json_slice(&stale_output.stdout).expect("stale gate report");
    assert!(!stale_report.decision.passed);
    assert_eq!(stale_report.unused_suppressions.len(), 1);
    assert!(stale_report
        .findings
        .iter()
        .all(|finding| finding.disposition == QualityGateDisposition::New));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn malformed_mismatched_and_version_incompatible_inputs_exit_one() {
    let dir = unique_temp_dir("gate-invalid-input");
    let (before_lock, before_cache, after_lock, after_cache) =
        changed_states(&dir, "1.0.0", "1.1.0");

    let malformed_path = dir.join("malformed.json");
    fs::write(&malformed_path, b"{").expect("write malformed input");
    let malformed = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[
            "--suppressions".to_owned(),
            malformed_path.to_str().expect("UTF-8 path").to_owned(),
        ],
    );
    assert_eq!(malformed.status.code(), Some(1));

    let mut mismatched =
        current_check_report(&before_lock, &before_cache, &after_lock, &after_cache);
    mismatched.compatibility.package_name = "other.package".to_owned();
    let mismatched_path = dir.join("mismatched-baseline.json");
    fs::write(
        &mismatched_path,
        mismatched.to_json_bytes().expect("baseline JSON"),
    )
    .expect("write mismatched baseline");
    let mismatch = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[
            "--baseline".to_owned(),
            mismatched_path.to_str().expect("UTF-8 path").to_owned(),
        ],
    );
    assert_eq!(mismatch.status.code(), Some(1));

    let current = current_check_report(&before_lock, &before_cache, &after_lock, &after_cache);
    let current_fingerprint = finding_fingerprint_v1(
        &current.compatibility.ruleset,
        current.compatibility.findings.first().expect("finding"),
    )
    .expect("fingerprint");
    let incompatible_path = dir.join("incompatible-suppressions.json");
    let incompatible = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: vec![GateSuppression {
            finding_fingerprint: FindingFingerprint {
                schema: 2,
                digest: current_fingerprint.digest,
            },
            rationale: "unsupported schema".to_owned(),
            reference: None,
        }],
    };
    fs::write(
        &incompatible_path,
        incompatible.to_json_bytes().expect("incompatible JSON"),
    )
    .expect("write incompatible suppressions");
    let incompatible_output = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[
            "--suppressions".to_owned(),
            incompatible_path.to_str().expect("UTF-8 path").to_owned(),
        ],
    );
    assert_eq!(incompatible_output.status.code(), Some(1));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn repeated_gate_runs_are_byte_identical() {
    let dir = unique_temp_dir("gate-determinism");
    let (before_lock, before_cache, after_lock, after_cache) =
        changed_states(&dir, "1.0.0", "1.1.0");
    let first = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[],
    );
    let second = run_command(
        "gate",
        &before_lock,
        &before_cache,
        &after_lock,
        &after_cache,
        &[],
    );

    assert_eq!(first.status.code(), Some(2));
    assert_eq!(second.status.code(), Some(2));
    assert_eq!(first.stdout, second.stdout);
    QualityGateReport::from_json_slice(&first.stdout).expect("deterministic report");
    let _ = fs::remove_dir_all(&dir);
}
