use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const VERIFIER_MANIFEST: &str = "tools/af02-verifier/Cargo.toml";
const VERIFIER_LOCK: &str = "tools/af02-verifier/Cargo.lock";
const SURFACE_POLICY: &str = "specs/016-af-02-adversarial-test-strength/surface-policy.json";
const SYN_VERSION: &str = "3.0.3";
const SYN_CHECKSUM: &str = "53e9bae58849f64dfa4f5d5ae372c8341f7305f82a3868709269343628b659a3";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("commandf-pkg must live under crates/ in the repository")
        .to_path_buf()
}

fn cargo(root: &Path, args: &[&str]) -> Output {
    let output = Command::new("cargo")
        .args(args)
        .current_dir(root)
        .output()
        .expect("run cargo for isolated AF-02 verifier");
    assert!(
        output.status.success(),
        "cargo {args:?} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn af02_surface_scanner_is_locked_linted_tested_and_runnable() {
    let root = repo_root();
    let lock = fs::read_to_string(root.join(VERIFIER_LOCK)).expect("read AF-02 verifier lockfile");
    let syn_entries = lock
        .split("[[package]]")
        .filter(|entry| {
            entry.contains("\nname = \"syn\"\n")
                && entry.contains(&format!("\nversion = \"{SYN_VERSION}\"\n"))
        })
        .collect::<Vec<_>>();
    assert_eq!(syn_entries.len(), 1, "expected exactly one syn 3.0.3 entry");
    assert!(
        syn_entries[0].contains(&format!("checksum = \"{SYN_CHECKSUM}\"")),
        "syn 3.0.3 checksum must equal the planning-frozen registry checksum"
    );

    cargo(
        &root,
        &[
            "clippy",
            "--locked",
            "--manifest-path",
            VERIFIER_MANIFEST,
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    );
    cargo(
        &root,
        &["test", "--locked", "--manifest-path", VERIFIER_MANIFEST],
    );

    let parsed = cargo(
        &root,
        &[
            "run",
            "--quiet",
            "--locked",
            "--manifest-path",
            VERIFIER_MANIFEST,
            "--",
            "parse-surface-policy",
            SURFACE_POLICY,
        ],
    );
    assert_eq!(parsed.stdout.first().copied(), Some(b'{'));

    let scanned = cargo(
        &root,
        &[
            "run",
            "--quiet",
            "--locked",
            "--manifest-path",
            VERIFIER_MANIFEST,
            "--",
            "scan-surface",
            SURFACE_POLICY,
            ".",
        ],
    );
    assert_eq!(scanned.stdout.first().copied(), Some(b'['));
    assert!(scanned.stdout.len() > 2, "surface scan must emit findings");
}
