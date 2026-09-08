use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("commandf-pkg must live under crates/ in the repository")
        .to_path_buf()
}

fn lock_has_package(lock: &str, name: &str) -> bool {
    lock.split("[[package]]")
        .any(|entry| entry.contains(&format!("\nname = \"{name}\"\n")))
}

#[test]
fn af02_t030_fuzz_workspace_is_isolated_and_pinned() {
    let root = repo_root();
    let root_manifest = fs::read_to_string(root.join("Cargo.toml")).expect("read root Cargo.toml");
    let root_lock = fs::read_to_string(root.join("Cargo.lock")).expect("read root Cargo.lock");
    let fuzz_manifest =
        fs::read_to_string(root.join("fuzz/Cargo.toml")).expect("read isolated fuzz manifest");
    let fuzz_toolchain = fs::read_to_string(root.join("fuzz/rust-toolchain.toml"))
        .expect("read isolated fuzz toolchain");

    let root_workspace = root_manifest
        .split("[workspace.package]")
        .next()
        .expect("root manifest must define workspace package metadata");
    assert!(
        !root_workspace
            .lines()
            .any(|line| line.trim().trim_end_matches(',').trim_matches('"') == "fuzz"),
        "fuzz must not be a member of the product workspace"
    );

    assert!(fuzz_manifest.contains("[workspace]"));
    assert!(fuzz_manifest.contains("members = []"));
    assert!(fuzz_manifest.contains("resolver = \"2\""));
    assert!(fuzz_manifest.contains("libfuzzer-sys = \"=0.4.13\""));
    assert!(fuzz_manifest.contains("arbitrary = { version = \"=1.4.2\", features = [\"derive\"] }"));
    assert!(fuzz_manifest.contains("proptest = \"=1.11.0\""));
    assert!(fuzz_manifest.contains("cargo-fuzz-version = \"0.13.2\""));
    assert!(fuzz_manifest
        .contains("cargo-fuzz-upstream-commit = \"984c861c8dfea28055254c5f1d2659ab2cd63f76\""));
    assert!(fuzz_manifest.contains("fuzz-toolchain = \"nightly-2026-08-25\""));
    assert!(fuzz_manifest.contains("product-toolchain = \"1.97.1\""));
    assert!(fuzz_manifest.contains("target = \"x86_64-unknown-linux-gnu\""));

    assert!(fuzz_toolchain.contains("channel = \"nightly-2026-08-25\""));
    assert!(fuzz_toolchain.contains("profile = \"minimal\""));
    assert!(fuzz_toolchain.contains("targets = [\"x86_64-unknown-linux-gnu\"]"));

    for package in ["libfuzzer-sys", "arbitrary", "proptest"] {
        assert!(
            !lock_has_package(&root_lock, package),
            "fuzz-only package {package} must not leak into the product Cargo.lock"
        );
    }
}

#[test]
fn af02_t031_archive_package_raw_fuzzer_is_bounded_and_uses_product_seams() {
    let root = repo_root();
    let fuzz_manifest =
        fs::read_to_string(root.join("fuzz/Cargo.toml")).expect("read isolated fuzz manifest");
    let target = fs::read_to_string(root.join("fuzz/fuzz_targets/archive_package_raw.rs"))
        .expect("read archive/package raw fuzz target");

    assert!(fuzz_manifest.contains("name = \"commandf-af02-fuzz\""));
    assert!(fuzz_manifest.contains("cargo-fuzz = true"));
    assert!(fuzz_manifest.contains("commandf-pkg = { path = \"../crates/commandf-pkg\" }"));
    assert!(fuzz_manifest.contains("libfuzzer-sys = { workspace = true }"));
    assert!(fuzz_manifest.contains("task = \"T031\""));
    assert!(fuzz_manifest.contains("name = \"archive_package_raw\""));
    assert!(fuzz_manifest.contains("path = \"fuzz_targets/archive_package_raw.rs\""));

    assert!(target.contains("const MAX_INPUT_BYTES: usize = 256 * 1024;"));
    assert!(target.contains("inspect_package("));
    assert!(target.contains("LocalMirrorSource::new("));
    assert!(target.contains("Resolver::new("));
    assert!(target.contains("PackageCache::new("));
    assert!(target.contains("fs::remove_dir_all("));
    assert!(!target.contains("read_manifest("));
    assert!(!target.contains("commandf_pkg::archive"));
}
