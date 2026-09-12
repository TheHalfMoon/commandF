use std::collections::BTreeMap;

use commandf_pkg::{LockedPackage, Lockfile, ResolvedDependency};
use proptest::prelude::*;

#[path = "support/property_runner.rs"]
mod property_runner;

#[path = "../../tests/assurance/af02_models/lockfile_v2.rs"]
mod lockfile_v2_model;

use lockfile_v2_model::{
    generated_valid_case, invalid_case, validate as model_validate, Invalidity, ModelLockfile,
    Verdict,
};

const PROPERTY_ID: &str = "PROP-LOCKFILE-V2-001";
const SEED_HEX: &str = "6ce9e5ba7a78953b8949a9298bb3089861cf4660919e6fa809b7766fb9139d50";

fn product_accepts(case: &ModelLockfile) -> bool {
    let packages = case
        .packages
        .iter()
        .map(|package| LockedPackage {
            name: package.name.clone(),
            version: package.version.clone(),
            sha256: "0".repeat(64),
            source: "af02:model".to_owned(),
            dependencies: package.dependencies.clone(),
        })
        .collect();
    let edges = case
        .resolved_dependencies
        .iter()
        .map(|edge| ResolvedDependency {
            from_name: edge.from_name.clone(),
            from_version: edge.from_version.clone(),
            to_name: edge.to_name.clone(),
            to_version: edge.to_version.clone(),
            declared_constraint: edge.declared_constraint.clone(),
        })
        .collect();
    let lock = Lockfile {
        schema: Lockfile::SCHEMA_V2,
        roots: case.roots.clone(),
        packages,
        resolved_dependencies: edges,
    };
    lock.to_bytes().is_ok()
}

#[test]
fn af02_lockfile_model_rejects_every_frozen_invalidity_class() {
    let cases = [
        Invalidity::UnsortedRoots,
        Invalidity::DuplicateRoot,
        Invalidity::UnsortedPackages,
        Invalidity::DuplicatePackageIdentity,
        Invalidity::UnsortedEdges,
        Invalidity::DuplicateEdge,
        Invalidity::MissingSourcePackage,
        Invalidity::MissingTargetPackage,
        Invalidity::EmptyConstraint,
        Invalidity::UndeclaredDependency,
        Invalidity::ConstraintMismatch,
        Invalidity::UnsatisfiedTargetVersion,
        Invalidity::MultipleTargetsForDependency,
        Invalidity::MissingResolvedEdge,
    ];

    for invalidity in cases {
        let case = invalid_case(invalidity);
        assert_eq!(model_validate(&case), Verdict::Invalid(invalidity));
        assert!(
            !product_accepts(&case),
            "{PROPERTY_ID} product validator unexpectedly accepted {invalidity:?}"
        );
    }
}

#[test]
fn af02_lockfile_valid_graphs_match_independent_model() {
    let strategy = (0u8..=8, 0u8..=8, 0u8..=16, any::<bool>());
    let mut runner = property_runner::runner(SEED_HEX);
    runner
        .run(&strategy, |(major, minor, patch, wildcard)| {
            let case = generated_valid_case(major, minor, patch, wildcard);
            prop_assert_eq!(model_validate(&case), Verdict::ValidCanonical);
            prop_assert!(product_accepts(&case));
            Ok(())
        })
        .expect("frozen Lockfile property must match the independent model");
}

#[test]
fn af02_lockfile_property_identity_is_frozen() {
    assert_eq!(PROPERTY_ID, "PROP-LOCKFILE-V2-001");
    assert_eq!(property_runner::CASE_COUNT, 256);
    assert_eq!(property_runner::MAX_SHRINK_ITERS, 4096);
    assert_eq!(
        SEED_HEX,
        "6ce9e5ba7a78953b8949a9298bb3089861cf4660919e6fa809b7766fb9139d50"
    );
    let _ = BTreeMap::<String, String>::new();
}
