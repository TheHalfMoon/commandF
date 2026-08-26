use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use commandf_pkg::{
    validate_quality_gate_report, GateSuppression, GateSuppressions, LockedPackage, Lockfile,
    PackageCache, QualityGateDisposition, QualityGateReport,
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

#[test]
fn gate_cli_proof_is_deterministic_and_covers_new_baseline_suppression() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).unwrap();
    let current = write_changed_states(&root.join("current"), "1.0.0", "1.1.0");
    let historical = write_changed_states(&root.join("historical"), "0.8.0", "0.9.0");

    let new_first = run_gate(&current, &[]);
    let new_second = run_gate(&current, &[]);
    assert_eq!(new_first.status.code(), Some(2));
    assert_eq!(new_second.status.code(), Some(2));
    assert_eq!(new_first.stdout, new_second.stdout);
    let new_report = QualityGateReport::from_json_slice(&new_first.stdout).unwrap();
    validate_quality_gate_report(&new_report).unwrap();
    assert!(!new_report.decision.passed);
    assert!(new_report
        .findings
        .iter()
        .all(|finding| finding.disposition == QualityGateDisposition::New));

    let baseline_path = root.join("baseline.json");
    let baseline_check = run_check(&historical);
    assert_eq!(baseline_check.status.code(), Some(2));
    fs::write(&baseline_path, baseline_check.stdout).unwrap();
    let baseline = run_gate(
        &current,
        &[
            "--baseline".to_owned(),
            baseline_path.to_str().unwrap().to_owned(),
        ],
    );
    assert_eq!(baseline.status.code(), Some(0));
    let baseline_report = QualityGateReport::from_json_slice(&baseline.stdout).unwrap();
    validate_quality_gate_report(&baseline_report).unwrap();
    assert!(baseline_report.decision.passed);

    let suppression_path = root.join("suppressions.json");
    let suppression = GateSuppressions {
        schema: GateSuppressions::SCHEMA_V1,
        suppressions: new_report
            .findings
            .iter()
            .map(|finding| GateSuppression {
                finding_fingerprint: finding.fingerprint.clone(),
                rationale: "CF-13 deterministic proof suppression".to_owned(),
                reference: Some("CF13-PROOF".to_owned()),
            })
            .collect(),
    };
    fs::write(&suppression_path, suppression.to_json_bytes().unwrap()).unwrap();
    let suppressed = run_gate(
        &current,
        &[
            "--suppressions".to_owned(),
            suppression_path.to_str().unwrap().to_owned(),
        ],
    );
    assert_eq!(suppressed.status.code(), Some(0));
    let suppression_report = QualityGateReport::from_json_slice(&suppressed.stdout).unwrap();
    validate_quality_gate_report(&suppression_report).unwrap();
    assert!(suppression_report.decision.passed);

    println!("CF13_GATE_SHA256={}", PackageCache::digest(&new_first.stdout));
    println!(
        "CF13_BASELINE_CANONICAL_SHA256={}",
        baseline_report.baseline.as_ref().unwrap().canonical_sha256
    );
    println!(
        "CF13_SUPPRESSION_CANONICAL_SHA256={}",
        suppression_report
            .suppression_evidence
            .as_ref()
            .unwrap()
            .canonical_sha256
    );
    println!(
        "CF13_BEFORE_ARCHIVE_SHA256={}",
        new_report.current.compatibility.before.archive_sha256
    );
    println!(
        "CF13_AFTER_ARCHIVE_SHA256={}",
        new_report.current.compatibility.after.archive_sha256
    );

    let _ = fs::remove_dir_all(root);
}

type State = (PathBuf, PathBuf, PathBuf, PathBuf);

fn write_changed_states(root: &Path, before_version: &str, after_version: &str) -> State {
    let before = decode_hex(BEFORE_HEX);
    let after = decode_hex(AFTER_HEX);
    let (before_lock, before_cache) =
        write_locked_state(&root.join("before"), &before, before_version);
    let (after_lock, after_cache) = write_locked_state(&root.join("after"), &after, after_version);
    (before_lock, before_cache, after_lock, after_cache)
}

fn write_locked_state(root: &Path, archive: &[u8], version: &str) -> (PathBuf, PathBuf) {
    fs::create_dir_all(root).unwrap();
    let cache_path = root.join("cache");
    let lock_path = root.join("commandf.lock");
    let cache = PackageCache::new(&cache_path);
    let sha256 = cache.put(archive).unwrap();
    let lockfile = Lockfile::new(
        vec![format!("example.package@{version}")],
        vec![LockedPackage {
            name: "example.package".to_owned(),
            version: version.to_owned(),
            sha256,
            source: "synthetic-cf13-proof".to_owned(),
            dependencies: BTreeMap::new(),
        }],
    );
    fs::write(lock_path.clone(), lockfile.to_bytes().unwrap()).unwrap();
    (lock_path, cache_path)
}

fn run_check(state: &State) -> Output {
    run("check", state, &[])
}

fn run_gate(state: &State, extra: &[String]) -> Output {
    run("gate", state, extra)
}

fn run(subcommand: &str, state: &State, extra: &[String]) -> Output {
    let (before_lock, before_cache, after_lock, after_cache) = state;
    let mut command = Command::new(env!("CARGO_BIN_EXE_commandf"));
    command.args([
        subcommand,
        "example.package",
        "--before-lock",
        before_lock.to_str().unwrap(),
        "--before-cache",
        before_cache.to_str().unwrap(),
        "--after-lock",
        after_lock.to_str().unwrap(),
        "--after-cache",
        after_cache.to_str().unwrap(),
    ]);
    command.args(extra);
    command
        .env("HTTP_PROXY", "http://127.0.0.1:9")
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "")
        .output()
        .unwrap()
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
        _ => panic!("invalid hex digit"),
    }
}

fn unique_temp_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("commandf-cf13-proof-{}-{nonce}", std::process::id()))
}
