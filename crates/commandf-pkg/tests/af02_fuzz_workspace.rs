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

#[test]
fn af02_t032_lockfile_raw_and_structured_properties_bind_independent_model() {
    let root = repo_root();
    let fuzz_manifest =
        fs::read_to_string(root.join("fuzz/Cargo.toml")).expect("read isolated fuzz manifest");
    let raw_target = fs::read_to_string(root.join("fuzz/fuzz_targets/lockfile_raw.rs"))
        .expect("read lockfile raw target");
    let property = fs::read_to_string(root.join("fuzz/tests/lockfile_v2_properties.rs"))
        .expect("read Lockfile V2 property suite");
    let model = fs::read_to_string(root.join("tests/assurance/af02_models/lockfile_v2.rs"))
        .expect("read independent Lockfile V2 model");

    assert!(fuzz_manifest.contains("stack = \"A1\""));
    assert!(fuzz_manifest.contains("proptest = { workspace = true }"));
    assert!(fuzz_manifest.contains("name = \"lockfile_raw\""));
    assert!(fuzz_manifest.contains("path = \"fuzz_targets/lockfile_raw.rs\""));
    assert!(raw_target.contains("const MAX_INPUT_BYTES: usize = 256 * 1024;"));
    assert!(raw_target.contains("Lockfile::from_slice(data)"));

    assert!(property.contains("PROP-LOCKFILE-V2-001"));
    assert!(property.contains("ProptestConfig::with_cases(CASE_COUNT)"));
    assert!(property.contains("CASE_COUNT: u32 = 256"));
    assert!(property.contains("../../tests/assurance/af02_models/lockfile_v2.rs"));
    assert!(property.contains("model_validate(&case)"));
    assert!(property.contains("product_accepts(&case)"));

    for forbidden in [
        "commandf_pkg",
        "Lockfile::validate_v2",
        "Lockfile::new_v2",
        "Lockfile::to_bytes",
        "Lockfile::from_slice",
    ] {
        assert!(
            !model.contains(forbidden),
            "independent model must not reuse product validator API: {forbidden}"
        );
    }
    for class in [
        "UnsortedRoots",
        "DuplicateRoot",
        "UnsortedPackages",
        "DuplicatePackageIdentity",
        "UnsortedEdges",
        "DuplicateEdge",
        "MissingSourcePackage",
        "MissingTargetPackage",
        "EmptyConstraint",
        "UndeclaredDependency",
        "ConstraintMismatch",
        "UnsatisfiedTargetVersion",
        "MultipleTargetsForDependency",
        "MissingResolvedEdge",
    ] {
        assert!(
            model.contains(class),
            "missing frozen invalidity class {class}"
        );
    }
}

#[test]
fn af02_t033_adversarial_properties_bind_all_frozen_models() {
    let root = repo_root();
    let fuzz_manifest =
        fs::read_to_string(root.join("fuzz/Cargo.toml")).expect("read isolated fuzz manifest");
    let property = fs::read_to_string(root.join("fuzz/tests/adversarial_properties.rs"))
        .expect("read T033 adversarial property suite");

    assert!(fuzz_manifest.contains("task = \"T033\""));
    for dependency in [
        "flate2 = \"=1.1.9\"",
        "serde_json = \"=1.0.151\"",
        "sha2 = \"=0.10.9\"",
        "tar = \"=0.4.46\"",
    ] {
        assert!(
            fuzz_manifest.contains(dependency),
            "T033 helper dependency must stay pinned to canonical product-lock identity: {dependency}"
        );
    }

    for property_id in [
        "PROP-CANONICAL-REFERENCE-001",
        "PROP-CONTEXT-GRAPH-ORDER-001",
        "PROP-GATE-FINGERPRINT-SUPPRESSION-001",
        "PROP-PORTABLE-PATH-001",
    ] {
        assert!(
            property.contains(property_id),
            "missing frozen T033 property id {property_id}"
        );
    }
    assert!(property.contains("const CASE_COUNT: u32 = 256;"));
    assert!(property.contains("ProptestConfig::with_cases(CASE_COUNT)"));

    let models = [
        (
            "tests/assurance/af02_models/canonical_reference.rs",
            &[
                "commandf_pkg",
                "parse_canonical_target",
                "resolve_reference",
                "TerminologyClosure::resolve_value_set",
            ][..],
            &[
                "EMPTY_TARGET",
                "EMPTY_TARGET_BEFORE_VERSION",
                "EMPTY_EXPLICIT_VERSION",
                "UNKNOWN_URL",
                "VERSION_WITHOUT_MATCH",
                "DUPLICATE_CANDIDATE_IDENTITY",
            ][..],
        ),
        (
            "tests/assurance/af02_models/context_graph_order.rs",
            &["commandf_pkg", "build_context_graph"][..],
            &[
                "PACKAGE_ORDER_PERMUTATION",
                "ARTIFACT_ORDER_PERMUTATION",
                "DEPENDENCY_EDGE_ORDER_PERMUTATION",
                "REFERENCE_ORDER_PERMUTATION",
                "DUPLICATE_ARTIFACT",
                "DUPLICATE_EDGE",
                "DUPLICATE_REFERENCE",
            ][..],
        ),
        (
            "tests/assurance/af02_models/gate_truth_table.rs",
            &[
                "commandf_pkg",
                "finding_fingerprint_v1",
                "evaluate_quality_gate",
                "validate_quality_gate_report",
                "evaluate_compatibility_policy",
            ][..],
            &[
                "DUPLICATE_CURRENT_FINGERPRINT",
                "DUPLICATE_BASELINE_FINGERPRINT",
                "DUPLICATE_SUPPRESSION_FINGERPRINT",
                "SUPPRESSION_METADATA_MISMATCH",
                "BASELINE_PACKAGE_MISMATCH",
                "BASELINE_RULESET_MISMATCH",
                "FINGERPRINT_FIELD_TAMPER",
                "DECISION_COUNTER_TAMPER",
                "UNUSED_SUPPRESSION_TAMPER",
            ][..],
        ),
        (
            "tests/assurance/af02_models/portable_path.rs",
            &[
                "commandf_pkg",
                "portable_relative_path",
                "relative_path_to_slash",
            ][..],
            &[
                "EMPTY",
                "LEADING_SLASH",
                "UNC_PREFIX",
                "DRIVE_PREFIX",
                "EMPTY_COMPONENT",
                "DOT_COMPONENT",
                "PARENT_COMPONENT",
                "DISALLOWED_SINGLE_DOT",
            ][..],
        ),
    ];

    for (path, forbidden_calls, invalidities) in models {
        let model = fs::read_to_string(root.join(path)).expect("read frozen independent model");
        for forbidden in forbidden_calls {
            assert!(
                !model.contains(forbidden),
                "{path} must not reuse forbidden product implementation: {forbidden}"
            );
        }
        for invalidity in invalidities {
            assert!(
                model.contains(invalidity),
                "{path} is missing frozen invalidity {invalidity}"
            );
        }
    }
}
