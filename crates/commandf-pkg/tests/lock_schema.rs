use std::collections::BTreeMap;

use commandf_pkg::{LockedPackage, Lockfile, PackageError, ResolvedDependency};

#[test]
fn schema_v1_remains_supported_and_serializes_without_v2_evidence() {
    let bytes = br#"{"schema":1,"roots":[],"packages":[]}"#;
    let lock = Lockfile::from_slice(bytes).unwrap();

    assert_eq!(lock.schema, Lockfile::SCHEMA_V1);
    assert!(lock.resolved_dependencies.is_empty());
    assert_eq!(
        lock.to_bytes().unwrap(),
        b"{\n  \"schema\": 1,\n  \"roots\": [],\n  \"packages\": []\n}\n"
    );
}

#[test]
fn schema_v1_rejects_resolved_dependency_evidence() {
    let bytes = br#"{"schema":1,"roots":[],"packages":[],"resolved_dependencies":[]}"#;
    let error = Lockfile::from_slice(bytes).unwrap_err();
    assert!(matches!(error, PackageError::InvalidLockfile(_)));
}

#[test]
fn schema_v2_requires_resolved_dependency_field() {
    let bytes = br#"{"schema":2,"roots":[],"packages":[]}"#;
    let error = Lockfile::from_slice(bytes).unwrap_err();
    assert!(matches!(error, PackageError::InvalidLockfile(_)));
}

#[test]
fn schema_v2_accepts_empty_edges_for_root_only_lock() {
    let bytes = br#"{"schema":2,"roots":[],"packages":[],"resolved_dependencies":[]}"#;
    let lock = Lockfile::from_slice(bytes).unwrap();

    assert_eq!(lock.schema, Lockfile::SCHEMA_V2);
    assert!(lock.resolved_dependencies.is_empty());
}

#[test]
fn schema_v2_round_trip_retains_exact_edge_evidence() {
    let mut parent_dependencies = BTreeMap::new();
    parent_dependencies.insert("acme.child".to_owned(), "2.0.x".to_owned());
    let packages = vec![
        package("acme.child", "2.0.0", BTreeMap::new()),
        package("acme.parent", "1.0.0", parent_dependencies),
    ];
    let edge = ResolvedDependency {
        from_name: "acme.parent".to_owned(),
        from_version: "1.0.0".to_owned(),
        to_name: "acme.child".to_owned(),
        to_version: "2.0.0".to_owned(),
        declared_constraint: "2.0.x".to_owned(),
    };
    let lock = Lockfile::new_v2(
        vec!["acme.parent@1.0.0".to_owned()],
        packages,
        vec![edge.clone()],
    );

    let bytes = lock.to_bytes().unwrap();
    let decoded = Lockfile::from_slice(&bytes).unwrap();
    assert_eq!(decoded, lock);
    assert_eq!(decoded.resolved_dependencies, vec![edge]);
}

#[test]
fn schema_v2_rejects_edge_with_missing_endpoint() {
    let mut dependencies = BTreeMap::new();
    dependencies.insert("acme.missing".to_owned(), "2.0.0".to_owned());
    let lock = Lockfile::new_v2(
        vec!["acme.parent@1.0.0".to_owned()],
        vec![package("acme.parent", "1.0.0", dependencies)],
        vec![ResolvedDependency {
            from_name: "acme.parent".to_owned(),
            from_version: "1.0.0".to_owned(),
            to_name: "acme.missing".to_owned(),
            to_version: "2.0.0".to_owned(),
            declared_constraint: "2.0.0".to_owned(),
        }],
    );

    let error = lock.to_bytes().unwrap_err();
    assert!(matches!(error, PackageError::InvalidLockfile(_)));
}

#[test]
fn schema_v2_rejects_missing_edge_for_declared_dependency() {
    let mut dependencies = BTreeMap::new();
    dependencies.insert("acme.child".to_owned(), "2.0.0".to_owned());
    let lock = Lockfile::new_v2(
        vec!["acme.parent@1.0.0".to_owned()],
        vec![
            package("acme.child", "2.0.0", BTreeMap::new()),
            package("acme.parent", "1.0.0", dependencies),
        ],
        vec![],
    );

    let error = lock.to_bytes().unwrap_err();
    assert!(matches!(error, PackageError::InvalidLockfile(_)));
}

#[test]
fn schema_v2_rejects_target_version_that_does_not_match_declared_constraint() {
    let mut dependencies = BTreeMap::new();
    dependencies.insert("acme.child".to_owned(), "2.0.x".to_owned());
    let lock = Lockfile::new_v2(
        vec!["acme.parent@1.0.0".to_owned()],
        vec![
            package("acme.child", "3.0.0", BTreeMap::new()),
            package("acme.parent", "1.0.0", dependencies),
        ],
        vec![ResolvedDependency {
            from_name: "acme.parent".to_owned(),
            from_version: "1.0.0".to_owned(),
            to_name: "acme.child".to_owned(),
            to_version: "3.0.0".to_owned(),
            declared_constraint: "2.0.x".to_owned(),
        }],
    );

    let error = lock.to_bytes().unwrap_err();
    assert!(matches!(error, PackageError::InvalidLockfile(_)));
}

#[test]
fn schema_v2_rejects_noncanonical_edge_order() {
    let bytes = br#"
    {
      "schema": 2,
      "roots": [],
      "packages": [
        {"name":"acme.a","version":"1.0.0","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","source":"memory:test","dependencies":{"acme.c":"1.0.0","acme.b":"1.0.0"}},
        {"name":"acme.b","version":"1.0.0","sha256":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","source":"memory:test","dependencies":{}},
        {"name":"acme.c","version":"1.0.0","sha256":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","source":"memory:test","dependencies":{}}
      ],
      "resolved_dependencies": [
        {"from_name":"acme.a","from_version":"1.0.0","to_name":"acme.c","to_version":"1.0.0","declared_constraint":"1.0.0"},
        {"from_name":"acme.a","from_version":"1.0.0","to_name":"acme.b","to_version":"1.0.0","declared_constraint":"1.0.0"}
      ]
    }
    "#;

    let error = Lockfile::from_slice(bytes).unwrap_err();
    assert!(matches!(error, PackageError::InvalidLockfile(_)));
}

#[test]
fn unsupported_lock_schema_fails_closed() {
    let bytes = br#"{"schema":3,"roots":[],"packages":[],"resolved_dependencies":[]}"#;
    let error = Lockfile::from_slice(bytes).unwrap_err();
    assert!(matches!(
        error,
        PackageError::UnsupportedLockSchema { .. }
    ));
}

fn package(name: &str, version: &str, dependencies: BTreeMap<String, String>) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256: "a".repeat(64),
        source: "memory:test".to_owned(),
        dependencies,
    }
}
