use std::collections::BTreeMap;
use std::io::Cursor;

use commandf_pkg::{
    build_context_graph, CanonicalReferenceRelation, CanonicalResolutionStatus, ContextGraphError,
    LockedPackage, Lockfile, PackageCache, PackageError, ResolvedDependency,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::{Builder, Header};
use tempfile::tempdir;

#[test]
fn builds_deterministic_context_graph_with_explicit_resolution_states() {
    let refs_archive = package_archive(
        "acme.refs",
        "1.0.0",
        &[
            (
                "package/StructureDefinition-profile.json",
                br#"{
                  "resourceType":"StructureDefinition",
                  "id":"profile",
                  "url":"https://example.org/StructureDefinition/profile",
                  "version":"1.0.0",
                  "baseDefinition":"https://example.org/StructureDefinition/base|1.0.0",
                  "differential":{"element":[{
                    "id":"Observation.subject",
                    "type":[{
                      "code":"Reference",
                      "profile":["https://example.org/StructureDefinition/type|1.0.0"],
                      "targetProfile":["https://example.org/StructureDefinition/target"]
                    }],
                    "binding":{"valueSet":"https://example.org/ValueSet/binding#allowed"}
                  }]}
                }"#,
            ),
            (
                "package/ValueSet-refs.json",
                br#"{
                  "resourceType":"ValueSet",
                  "id":"refs",
                  "url":"https://example.org/ValueSet/refs",
                  "version":"1.0.0",
                  "compose":{
                    "include":[{
                      "system":"https://example.org/CodeSystem/system",
                      "valueSet":["https://example.org/ValueSet/imported"]
                    }],
                    "exclude":[{
                      "system":"https://example.org/CodeSystem/missing",
                      "valueSet":["https://example.org/ValueSet/imported|2.0.0"]
                    }]
                  }
                }"#,
            ),
            (
                "package/CodeSystem-supplement.json",
                br#"{
                  "resourceType":"CodeSystem",
                  "id":"supplement",
                  "url":"https://example.org/CodeSystem/supplement",
                  "version":"1.0.0",
                  "supplements":"https://example.org/CodeSystem/base|1.0.0"
                }"#,
            ),
            (
                "package/Patient-unsupported.json",
                br#"{"resourceType":"Patient","id":"unsupported"}"#,
            ),
        ],
    );
    let targets_archive = package_archive(
        "acme.targets",
        "1.0.0",
        &[
            canonical_resource(
                "StructureDefinition",
                "base",
                "https://example.org/StructureDefinition/base",
                "1.0.0",
            ),
            canonical_resource(
                "StructureDefinition",
                "type",
                "https://example.org/StructureDefinition/type",
                "1.0.0",
            ),
            canonical_resource(
                "StructureDefinition",
                "target",
                "https://example.org/StructureDefinition/target",
                "1.0.0",
            ),
            canonical_resource(
                "ValueSet",
                "binding",
                "https://example.org/ValueSet/binding",
                "1.0.0",
            ),
            canonical_resource(
                "CodeSystem",
                "system",
                "https://example.org/CodeSystem/system",
                "1.0.0",
            ),
            canonical_resource(
                "ValueSet",
                "imported-v1",
                "https://example.org/ValueSet/imported",
                "1.0.0",
            ),
            canonical_resource(
                "ValueSet",
                "imported-v2",
                "https://example.org/ValueSet/imported",
                "2.0.0",
            ),
            canonical_resource(
                "CodeSystem",
                "base-system",
                "https://example.org/CodeSystem/base",
                "1.0.0",
            ),
        ],
    );

    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());
    let refs_sha = cache.put(&refs_archive).unwrap();
    let targets_sha = cache.put(&targets_archive).unwrap();
    let mut refs_dependencies = BTreeMap::new();
    refs_dependencies.insert("acme.targets".to_owned(), "1.0.0".to_owned());
    let lock = Lockfile::new_v2(
        vec!["acme.refs@1.0.0".to_owned()],
        vec![
            locked_package("acme.refs", "1.0.0", &refs_sha, refs_dependencies),
            locked_package("acme.targets", "1.0.0", &targets_sha, BTreeMap::new()),
        ],
        vec![ResolvedDependency {
            from_name: "acme.refs".to_owned(),
            from_version: "1.0.0".to_owned(),
            to_name: "acme.targets".to_owned(),
            to_version: "1.0.0".to_owned(),
            declared_constraint: "1.0.0".to_owned(),
        }],
    );

    let first = build_context_graph(&lock, &cache).unwrap();
    let second = build_context_graph(&lock, &cache).unwrap();
    assert_eq!(
        first.to_json_bytes().unwrap(),
        second.to_json_bytes().unwrap()
    );
    assert_eq!(first.schema, 1);
    assert_eq!(first.lock_schema, Lockfile::SCHEMA_V2);
    assert_eq!(first.packages.len(), 2);
    assert_eq!(first.package_dependency_edges.len(), 1);
    assert_eq!(
        first.coverage.unsupported_source_resource_types,
        vec!["Patient"]
    );

    assert_resolution(
        &first,
        CanonicalReferenceRelation::StructureBaseDefinition,
        "https://example.org/StructureDefinition/base|1.0.0",
        CanonicalResolutionStatus::Resolved,
        1,
    );
    assert_resolution(
        &first,
        CanonicalReferenceRelation::StructureTypeProfile,
        "https://example.org/StructureDefinition/type|1.0.0",
        CanonicalResolutionStatus::Resolved,
        1,
    );
    assert_resolution(
        &first,
        CanonicalReferenceRelation::StructureTypeTargetProfile,
        "https://example.org/StructureDefinition/target",
        CanonicalResolutionStatus::Resolved,
        1,
    );
    assert_resolution(
        &first,
        CanonicalReferenceRelation::StructureBindingValueSet,
        "https://example.org/ValueSet/binding#allowed",
        CanonicalResolutionStatus::Resolved,
        1,
    );
    assert_resolution(
        &first,
        CanonicalReferenceRelation::ValueSetIncludeSystem,
        "https://example.org/CodeSystem/system",
        CanonicalResolutionStatus::Resolved,
        1,
    );
    assert_resolution(
        &first,
        CanonicalReferenceRelation::ValueSetIncludeValueSet,
        "https://example.org/ValueSet/imported",
        CanonicalResolutionStatus::Ambiguous,
        2,
    );
    assert_resolution(
        &first,
        CanonicalReferenceRelation::ValueSetExcludeSystem,
        "https://example.org/CodeSystem/missing",
        CanonicalResolutionStatus::External,
        0,
    );
    assert_resolution(
        &first,
        CanonicalReferenceRelation::ValueSetExcludeValueSet,
        "https://example.org/ValueSet/imported|2.0.0",
        CanonicalResolutionStatus::Resolved,
        1,
    );
    assert_resolution(
        &first,
        CanonicalReferenceRelation::CodeSystemSupplements,
        "https://example.org/CodeSystem/base|1.0.0",
        CanonicalResolutionStatus::Resolved,
        1,
    );
}

#[test]
fn rejects_schema_v1_instead_of_inferring_resolved_edges() {
    let lock = Lockfile::new(Vec::new(), Vec::new());
    let dir = tempdir().unwrap();
    let error = build_context_graph(&lock, &PackageCache::new(dir.path())).unwrap_err();

    assert!(matches!(
        error,
        ContextGraphError::RequiresLockV2 {
            found: Lockfile::SCHEMA_V1
        }
    ));
}

#[test]
fn fails_closed_when_cached_archive_bytes_are_corrupted() {
    let archive = package_archive("acme.root", "1.0.0", &[]);
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());
    let digest = cache.put(&archive).unwrap();
    let lock = Lockfile::new_v2(
        vec!["acme.root@1.0.0".to_owned()],
        vec![locked_package(
            "acme.root",
            "1.0.0",
            &digest,
            BTreeMap::new(),
        )],
        vec![],
    );
    let object_path = dir.path().join("sha256").join(format!("{digest}.tgz"));
    std::fs::write(object_path, b"corrupted").unwrap();

    let error = build_context_graph(&lock, &cache).unwrap_err();
    assert!(matches!(
        error,
        ContextGraphError::Package(PackageError::CacheDigestMismatch { .. })
    ));
}

#[test]
fn rejects_malformed_supported_reference_shape() {
    let archive = package_archive(
        "acme.root",
        "1.0.0",
        &[(
            "package/StructureDefinition-bad.json",
            br#"{
              "resourceType":"StructureDefinition",
              "id":"bad",
              "url":"https://example.org/StructureDefinition/bad",
              "version":"1.0.0",
              "differential":{"element":[{
                "id":"Observation.subject",
                "type":[{"code":"Reference","profile":"not-an-array"}]
              }]}
            }"#,
        )],
    );
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());
    let digest = cache.put(&archive).unwrap();
    let lock = Lockfile::new_v2(
        vec!["acme.root@1.0.0".to_owned()],
        vec![locked_package(
            "acme.root",
            "1.0.0",
            &digest,
            BTreeMap::new(),
        )],
        vec![],
    );

    let error = build_context_graph(&lock, &cache).unwrap_err();
    assert!(matches!(
        error,
        ContextGraphError::InvalidResourceField { .. }
    ));
}

fn assert_resolution(
    report: &commandf_pkg::ContextGraphReport,
    relation: CanonicalReferenceRelation,
    canonical: &str,
    status: CanonicalResolutionStatus,
    candidates: usize,
) {
    let edge = report
        .canonical_reference_edges
        .iter()
        .find(|edge| edge.relation == relation && edge.canonical == canonical)
        .unwrap();
    assert_eq!(edge.resolution, status);
    assert_eq!(edge.candidates.len(), candidates);
}

fn locked_package(
    name: &str,
    version: &str,
    sha256: &str,
    dependencies: BTreeMap<String, String>,
) -> LockedPackage {
    LockedPackage {
        name: name.to_owned(),
        version: version.to_owned(),
        sha256: sha256.to_owned(),
        source: "memory:test".to_owned(),
        dependencies,
    }
}

fn canonical_resource(
    resource_type: &'static str,
    id: &'static str,
    url: &'static str,
    version: &'static str,
) -> (&'static str, &'static [u8]) {
    let filename = Box::leak(format!("package/{resource_type}-{id}.json").into_boxed_str());
    let body = Box::leak(
        format!(
            "{{\"resourceType\":\"{resource_type}\",\"id\":\"{id}\",\"url\":\"{url}\",\"version\":\"{version}\"}}"
        )
        .into_bytes()
        .into_boxed_slice(),
    );
    (filename, body)
}

fn package_archive(name: &str, version: &str, resources: &[(&str, &[u8])]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        let manifest = format!("{{\"name\":\"{name}\",\"version\":\"{version}\"}}");
        append_entry(&mut builder, "package/package.json", manifest.as_bytes());
        for (path, body) in resources {
            append_entry(&mut builder, path, body);
        }
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

fn append_entry(builder: &mut Builder<&mut GzEncoder<Vec<u8>>>, path: &str, body: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_path(path).unwrap();
    header.set_size(body.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append(&header, Cursor::new(body)).unwrap();
}
