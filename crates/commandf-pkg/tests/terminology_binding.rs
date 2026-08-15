use std::collections::BTreeMap;
use std::io::Cursor;

use commandf_pkg::{
    build_terminology_diff_report, classify_structural_diff, diff_package_archives,
    CompatibilityDirection, CompatibilitySeverity, LockedPackage, Lockfile, PackageCache,
    TerminologyIndeterminateReason, TerminologyPackageState, TerminologyRelation,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use tar::{Builder, Header};
use tempfile::tempdir;

fn archive(name: &str, version: &str, dependencies: Value, resources: &[(&str, Value)]) -> Vec<u8> {
    let mut entries = vec![(
        "package/package.json".to_owned(),
        serde_json::to_vec(&json!({
            "name": name,
            "version": version,
            "dependencies": dependencies,
        }))
        .unwrap(),
    )];
    for (filename, resource) in resources {
        entries.push((
            format!("package/{filename}"),
            serde_json::to_vec(resource).unwrap(),
        ));
    }

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        for (path, body) in entries {
            let mut header = Header::new_gnu();
            header.set_path(path).unwrap();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append(&header, Cursor::new(body)).unwrap();
        }
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

fn profile() -> Value {
    json!({
        "resourceType": "StructureDefinition",
        "id": "test-profile",
        "url": "http://example.org/StructureDefinition/test-profile",
        "version": "1",
        "name": "TestProfile",
        "status": "active",
        "kind": "resource",
        "abstract": false,
        "type": "Patient",
        "baseDefinition": "http://hl7.org/fhir/StructureDefinition/Patient",
        "derivation": "constraint",
        "snapshot": {
            "element": [
                {"id": "Patient", "path": "Patient", "min": 0, "max": "1"},
                {
                    "id": "Patient.gender",
                    "path": "Patient.gender",
                    "min": 0,
                    "max": "1",
                    "type": [{"code": "code"}],
                    "binding": {
                        "strength": "required",
                        "valueSet": "http://example.org/ValueSet/gender"
                    }
                }
            ]
        }
    })
}

fn value_set(version: &str, codes: &[&str]) -> Value {
    json!({
        "resourceType": "ValueSet",
        "id": "gender",
        "url": "http://example.org/ValueSet/gender",
        "version": version,
        "name": "Gender",
        "status": "active",
        "expansion": {
            "total": codes.len(),
            "contains": codes.iter().map(|code| json!({
                "system": "http://example.org/CodeSystem/gender",
                "version": "1",
                "code": code
            })).collect::<Vec<_>>()
        }
    })
}

fn locked(
    name: &str,
    version: &str,
    sha256: String,
    dependencies: BTreeMap<String, String>,
) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256,
        source: "fixture".to_owned(),
        dependencies,
    }
}

#[test]
fn unchanged_binding_detects_narrowed_dependency_value_set() {
    let before_dir = tempdir().unwrap();
    let after_dir = tempdir().unwrap();
    let before_cache = PackageCache::new(before_dir.path());
    let after_cache = PackageCache::new(after_dir.path());

    let before_root = archive(
        "example.root",
        "1.0.0",
        json!({"example.term": "1.0.0"}),
        &[("StructureDefinition-test-profile.json", profile())],
    );
    let after_root = archive(
        "example.root",
        "2.0.0",
        json!({"example.term": "2.0.0"}),
        &[("StructureDefinition-test-profile.json", profile())],
    );
    let before_term = archive(
        "example.term",
        "1.0.0",
        json!({}),
        &[("ValueSet-gender.json", value_set("1", &["a", "b"]))],
    );
    let after_term = archive(
        "example.term",
        "2.0.0",
        json!({}),
        &[("ValueSet-gender.json", value_set("2", &["a"]))],
    );

    let before_root_digest = before_cache.put(&before_root).unwrap();
    let after_root_digest = after_cache.put(&after_root).unwrap();
    let before_term_digest = before_cache.put(&before_term).unwrap();
    let after_term_digest = after_cache.put(&after_term).unwrap();

    let before_lock = Lockfile::new(
        vec!["example.root@1.0.0".to_owned()],
        vec![
            locked(
                "example.root",
                "1.0.0",
                before_root_digest.clone(),
                BTreeMap::from([("example.term".to_owned(), "1.0.0".to_owned())]),
            ),
            locked("example.term", "1.0.0", before_term_digest, BTreeMap::new()),
        ],
    );
    let after_lock = Lockfile::new(
        vec!["example.root@2.0.0".to_owned()],
        vec![
            locked(
                "example.root",
                "2.0.0",
                after_root_digest.clone(),
                BTreeMap::from([("example.term".to_owned(), "2.0.0".to_owned())]),
            ),
            locked("example.term", "2.0.0", after_term_digest, BTreeMap::new()),
        ],
    );

    let structural = diff_package_archives(
        "example.root",
        "1.0.0",
        &before_root_digest,
        &before_root,
        "2.0.0",
        &after_root_digest,
        &after_root,
    )
    .unwrap();
    assert!(structural.changes.is_empty());

    let compatibility = classify_structural_diff(&structural).unwrap();
    assert!(compatibility.findings.is_empty());

    let report = build_terminology_diff_report(
        TerminologyPackageState {
            lockfile: &before_lock,
            cache: &before_cache,
            root_bytes: &before_root,
        },
        TerminologyPackageState {
            lockfile: &after_lock,
            cache: &after_cache,
            root_bytes: &after_root,
        },
        &structural,
        &compatibility,
    )
    .unwrap();

    assert_eq!(report.compatibility, compatibility);
    assert!(report.code_systems.is_empty());
    assert!(report.value_sets.is_empty());
    assert_eq!(report.binding_refinements.len(), 1);
    let refinement = &report.binding_refinements[0];
    assert_eq!(refinement.relation, TerminologyRelation::Narrowed);
    assert_eq!(refinement.rule_id.as_deref(), Some("CF07-BIND-001"));
    assert_eq!(refinement.severity, Some(CompatibilitySeverity::Breaking));
    assert_eq!(refinement.direction, Some(CompatibilityDirection::Producer));
    assert_eq!(
        report.to_json_bytes().unwrap(),
        report.to_json_bytes().unwrap()
    );
}

#[test]
fn bare_binding_ambiguity_is_explicit_indeterminate_evidence() {
    let before_dir = tempdir().unwrap();
    let after_dir = tempdir().unwrap();
    let before_cache = PackageCache::new(before_dir.path());
    let after_cache = PackageCache::new(after_dir.path());

    let root = archive(
        "example.root",
        "1.0.0",
        json!({"example.term": "1.x"}),
        &[("StructureDefinition-test-profile.json", profile())],
    );
    let term_v1 = archive(
        "example.term",
        "1.0.0",
        json!({}),
        &[("ValueSet-gender.json", value_set("1", &["a"]))],
    );
    let term_v2 = archive(
        "example.term",
        "2.0.0",
        json!({}),
        &[("ValueSet-gender.json", value_set("2", &["a", "b"]))],
    );

    let before_root_digest = before_cache.put(&root).unwrap();
    let after_root_digest = after_cache.put(&root).unwrap();
    let before_v1_digest = before_cache.put(&term_v1).unwrap();
    let before_v2_digest = before_cache.put(&term_v2).unwrap();
    let after_v1_digest = after_cache.put(&term_v1).unwrap();
    let after_v2_digest = after_cache.put(&term_v2).unwrap();

    let before_lock = Lockfile::new(
        vec!["example.root@1.0.0".to_owned()],
        vec![
            locked(
                "example.root",
                "1.0.0",
                before_root_digest.clone(),
                BTreeMap::from([("example.term".to_owned(), "1.x".to_owned())]),
            ),
            locked("example.term", "1.0.0", before_v1_digest, BTreeMap::new()),
            locked("example.term", "2.0.0", before_v2_digest, BTreeMap::new()),
        ],
    );
    let after_lock = Lockfile::new(
        vec!["example.root@1.0.0".to_owned()],
        vec![
            locked(
                "example.root",
                "1.0.0",
                after_root_digest.clone(),
                BTreeMap::from([("example.term".to_owned(), "1.x".to_owned())]),
            ),
            locked("example.term", "1.0.0", after_v1_digest, BTreeMap::new()),
            locked("example.term", "2.0.0", after_v2_digest, BTreeMap::new()),
        ],
    );

    let structural = diff_package_archives(
        "example.root",
        "1.0.0",
        &before_root_digest,
        &root,
        "1.0.0",
        &after_root_digest,
        &root,
    )
    .unwrap();
    let compatibility = classify_structural_diff(&structural).unwrap();
    let report = build_terminology_diff_report(
        TerminologyPackageState {
            lockfile: &before_lock,
            cache: &before_cache,
            root_bytes: &root,
        },
        TerminologyPackageState {
            lockfile: &after_lock,
            cache: &after_cache,
            root_bytes: &root,
        },
        &structural,
        &compatibility,
    )
    .unwrap();

    assert_eq!(report.binding_refinements.len(), 1);
    let refinement = &report.binding_refinements[0];
    assert_eq!(refinement.relation, TerminologyRelation::Indeterminate);
    assert_eq!(
        refinement.reason,
        Some(TerminologyIndeterminateReason::AmbiguousCanonical)
    );
    assert!(!refinement.binding_proof_eligible);
    assert!(refinement.proof_mode.is_none());
    assert!(refinement.rule_id.is_none());
    assert!(refinement.severity.is_none());
    assert!(refinement.direction.is_none());
}

#[test]
fn corrupted_dependency_cache_fails_before_binding_proof() {
    let before_dir = tempdir().unwrap();
    let after_dir = tempdir().unwrap();
    let before_cache = PackageCache::new(before_dir.path());
    let after_cache = PackageCache::new(after_dir.path());

    let root = archive(
        "example.root",
        "1.0.0",
        json!({"example.term": "1.0.0"}),
        &[("StructureDefinition-test-profile.json", profile())],
    );
    let term = archive(
        "example.term",
        "1.0.0",
        json!({}),
        &[("ValueSet-gender.json", value_set("1", &["a"]))],
    );
    let before_root_digest = before_cache.put(&root).unwrap();
    let after_root_digest = after_cache.put(&root).unwrap();
    let before_term_digest = before_cache.put(&term).unwrap();
    let after_term_digest = after_cache.put(&term).unwrap();

    let before_lock = Lockfile::new(
        vec!["example.root@1.0.0".to_owned()],
        vec![
            locked(
                "example.root",
                "1.0.0",
                before_root_digest.clone(),
                BTreeMap::new(),
            ),
            locked("example.term", "1.0.0", before_term_digest, BTreeMap::new()),
        ],
    );
    let after_lock = Lockfile::new(
        vec!["example.root@1.0.0".to_owned()],
        vec![
            locked(
                "example.root",
                "1.0.0",
                after_root_digest.clone(),
                BTreeMap::new(),
            ),
            locked(
                "example.term",
                "1.0.0",
                after_term_digest.clone(),
                BTreeMap::new(),
            ),
        ],
    );

    std::fs::write(
        after_cache
            .root()
            .join("sha256")
            .join(format!("{after_term_digest}.tgz")),
        b"corrupt",
    )
    .unwrap();

    let structural = diff_package_archives(
        "example.root",
        "1.0.0",
        &before_root_digest,
        &root,
        "1.0.0",
        &after_root_digest,
        &root,
    )
    .unwrap();
    let compatibility = classify_structural_diff(&structural).unwrap();
    let error = build_terminology_diff_report(
        TerminologyPackageState {
            lockfile: &before_lock,
            cache: &before_cache,
            root_bytes: &root,
        },
        TerminologyPackageState {
            lockfile: &after_lock,
            cache: &after_cache,
            root_bytes: &root,
        },
        &structural,
        &compatibility,
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("digest mismatch"));
}
