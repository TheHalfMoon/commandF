use std::collections::BTreeMap;
use std::io::Cursor;

use commandf_pkg::{
    Lockfile, PackageCache, PackageError, PackageName, PackageRequest, PackageSource, Resolver,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use semver::Version;
use tar::{Builder, Header};
use tempfile::tempdir;

#[derive(Default)]
struct MemorySource {
    packages: BTreeMap<(String, String), Vec<u8>>,
}

impl MemorySource {
    fn add(&mut self, name: &str, version: &str, dependencies: &[(&str, &str)]) {
        self.packages.insert(
            (name.to_owned(), version.to_owned()),
            package_tgz(name, version, dependencies),
        );
    }
}

impl PackageSource for MemorySource {
    fn source_id(&self) -> String {
        "memory:test".to_owned()
    }

    fn available_versions(&self, name: &PackageName) -> Result<Vec<Version>, PackageError> {
        let mut versions = self
            .packages
            .keys()
            .filter(|(candidate, _)| candidate == name.as_str())
            .map(|(_, version)| Version::parse(version))
            .collect::<Result<Vec<_>, _>>()?;
        versions.sort();
        Ok(versions)
    }

    fn archive(&self, name: &PackageName, version: &Version) -> Result<Vec<u8>, PackageError> {
        self.packages
            .get(&(name.to_string(), version.to_string()))
            .cloned()
            .ok_or_else(|| PackageError::PackageNotFound {
                name: name.to_string(),
                version: version.to_string(),
            })
    }
}

fn package_tgz(name: &str, version: &str, dependencies: &[(&str, &str)]) -> Vec<u8> {
    package_tgz_with_extra_evidence(name, version, dependencies, None)
}

/// Builds two archives with the same parsed FHIR identity/dependencies but
/// potentially different raw bytes, mirroring a cross-registry source conflict.
fn package_tgz_with_extra_evidence(
    name: &str,
    version: &str,
    dependencies: &[(&str, &str)],
    extra_evidence: Option<&[u8]>,
) -> Vec<u8> {
    let dependencies: BTreeMap<_, _> = dependencies
        .iter()
        .map(|(name, version)| ((*name).to_owned(), (*version).to_owned()))
        .collect();
    let body = serde_json::to_vec(&serde_json::json!({
        "name": name,
        "version": version,
        "dependencies": dependencies
    }))
    .unwrap();

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    {
        let mut builder = Builder::new(&mut encoder);
        let mut header = Header::new_gnu();
        header.set_path("package/package.json").unwrap();
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, Cursor::new(body)).unwrap();

        if let Some(extra_evidence) = extra_evidence {
            let mut header = Header::new_gnu();
            header
                .set_path("package/other/publication-request.json")
                .unwrap();
            header.set_size(extra_evidence.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append(&header, Cursor::new(extra_evidence))
                .unwrap();
        }
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
}

#[test]
fn identical_name_version_and_dependencies_do_not_hide_cross_source_byte_conflict() {
    // Synthetic-only regression inspired by #214. Neither package is a
    // canonical registry snapshot or a substitution for independent evidence.
    let request = PackageRequest::parse("acme.example@1.0.0").unwrap();
    let mut primary = MemorySource::default();
    primary.add("acme.example", "1.0.0", &[]);
    let mut secondary = MemorySource::default();
    secondary.packages.insert(
        ("acme.example".to_owned(), "1.0.0".to_owned()),
        package_tgz_with_extra_evidence(
            "acme.example",
            "1.0.0",
            &[],
            Some(br#"{"publication-request":"synthetic"}"#),
        ),
    );

    let first_dir = tempdir().unwrap();
    let second_dir = tempdir().unwrap();
    let first_cache = PackageCache::new(first_dir.path());
    let second_cache = PackageCache::new(second_dir.path());
    let first = Resolver::new(&primary, &first_cache)
        .resolve(vec![request.clone()])
        .unwrap();
    let second = Resolver::new(&secondary, &second_cache)
        .resolve(vec![request])
        .unwrap();

    first.verify_cache(&first_cache).unwrap();
    second.verify_cache(&second_cache).unwrap();
    assert_eq!(first.schema, second.schema);
    assert_eq!(first.roots, second.roots);
    assert_eq!(first.packages.len(), 1);
    assert_eq!(second.packages.len(), 1);
    assert_eq!(first.packages[0].name, second.packages[0].name);
    assert_eq!(first.packages[0].version, second.packages[0].version);
    assert_eq!(
        first.packages[0].dependencies,
        second.packages[0].dependencies
    );

    // This must never silently collapse two independently verified, but
    // differently sourced, byte identities into one canonical claim.
    assert_ne!(first.packages[0].sha256, second.packages[0].sha256);
    assert_ne!(first.to_bytes().unwrap(), second.to_bytes().unwrap());
    assert!(first.verify_cache(&second_cache).is_err());
    assert!(second.verify_cache(&first_cache).is_err());
}

#[test]
fn resolves_transitive_dependency_and_highest_stable_patch() {
    let mut source = MemorySource::default();
    source.add("acme.root", "1.0.0", &[("acme.dep", "1.2.x")]);
    source.add("acme.dep", "1.2.0", &[]);
    source.add("acme.dep", "1.2.3", &[]);
    source.add("acme.dep", "1.2.4-beta.1", &[]);
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());

    let lock = Resolver::new(&source, &cache)
        .resolve(vec![PackageRequest::parse("acme.root@1.0.0").unwrap()])
        .unwrap();

    assert_eq!(lock.schema, Lockfile::SCHEMA_V2);
    assert_eq!(lock.packages.len(), 2);
    assert_eq!(
        lock.packages
            .iter()
            .find(|package| package.name == "acme.dep")
            .unwrap()
            .version,
        "1.2.3"
    );
    assert_eq!(lock.resolved_dependencies.len(), 1);
    let edge = &lock.resolved_dependencies[0];
    assert_eq!(edge.from_name, "acme.root");
    assert_eq!(edge.from_version, "1.0.0");
    assert_eq!(edge.to_name, "acme.dep");
    assert_eq!(edge.to_version, "1.2.3");
    assert_eq!(edge.declared_constraint, "1.2.x");
    lock.verify_cache(&cache).unwrap();
}

#[test]
fn resolves_branch_local_concrete_versions_of_same_package_with_exact_edges() {
    let mut source = MemorySource::default();
    source.add("acme.left", "1.0.0", &[("acme.dep", "1.0.0")]);
    source.add("acme.right", "1.0.0", &[("acme.dep", "2.0.0")]);
    source.add("acme.dep", "1.0.0", &[]);
    source.add("acme.dep", "2.0.0", &[]);
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());

    let lock = Resolver::new(&source, &cache)
        .resolve(vec![
            PackageRequest::parse("acme.left@1.0.0").unwrap(),
            PackageRequest::parse("acme.right@1.0.0").unwrap(),
        ])
        .unwrap();

    let versions = lock
        .packages
        .iter()
        .filter(|package| package.name == "acme.dep")
        .map(|package| package.version.as_str())
        .collect::<Vec<_>>();
    assert_eq!(versions, vec!["1.0.0", "2.0.0"]);
    assert_eq!(lock.resolved_dependencies.len(), 2);

    let left = &lock.resolved_dependencies[0];
    assert_eq!(
        (
            left.from_name.as_str(),
            left.from_version.as_str(),
            left.to_name.as_str(),
            left.to_version.as_str(),
            left.declared_constraint.as_str(),
        ),
        ("acme.left", "1.0.0", "acme.dep", "1.0.0", "1.0.0")
    );
    let right = &lock.resolved_dependencies[1];
    assert_eq!(
        (
            right.from_name.as_str(),
            right.from_version.as_str(),
            right.to_name.as_str(),
            right.to_version.as_str(),
            right.declared_constraint.as_str(),
        ),
        ("acme.right", "1.0.0", "acme.dep", "2.0.0", "2.0.0")
    );
    lock.verify_cache(&cache).unwrap();
}

#[test]
fn deduplicates_the_same_concrete_identity_across_branches_but_retains_both_edges() {
    let mut source = MemorySource::default();
    source.add("acme.left", "1.0.0", &[("acme.dep", "1.0.0")]);
    source.add("acme.right", "1.0.0", &[("acme.dep", "1.0.0")]);
    source.add("acme.dep", "1.0.0", &[]);
    let dir = tempdir().unwrap();

    let lock = Resolver::new(&source, &PackageCache::new(dir.path()))
        .resolve(vec![
            PackageRequest::parse("acme.left@1.0.0").unwrap(),
            PackageRequest::parse("acme.right@1.0.0").unwrap(),
        ])
        .unwrap();

    assert_eq!(
        lock.packages
            .iter()
            .filter(|package| package.name == "acme.dep" && package.version == "1.0.0")
            .count(),
        1
    );
    assert_eq!(lock.resolved_dependencies.len(), 2);
    assert!(lock
        .resolved_dependencies
        .iter()
        .any(|edge| edge.from_name == "acme.left" && edge.to_name == "acme.dep"));
    assert!(lock
        .resolved_dependencies
        .iter()
        .any(|edge| edge.from_name == "acme.right" && edge.to_name == "acme.dep"));
}

#[test]
fn exact_and_patch_wildcard_requests_can_resolve_to_distinct_versions_deterministically() {
    let mut source = MemorySource::default();
    source.add("acme.dep", "1.2.0", &[]);
    source.add("acme.dep", "1.2.3", &[]);
    source.add("acme.dep", "1.2.4-beta.1", &[]);
    let first_dir = tempdir().unwrap();
    let second_dir = tempdir().unwrap();

    let first = Resolver::new(&source, &PackageCache::new(first_dir.path()))
        .resolve(vec![
            PackageRequest::parse("acme.dep@1.2.0").unwrap(),
            PackageRequest::parse("acme.dep@1.2.x").unwrap(),
        ])
        .unwrap();
    let second = Resolver::new(&source, &PackageCache::new(second_dir.path()))
        .resolve(vec![
            PackageRequest::parse("acme.dep@1.2.x").unwrap(),
            PackageRequest::parse("acme.dep@1.2.0").unwrap(),
        ])
        .unwrap();

    assert_eq!(
        first
            .packages
            .iter()
            .map(|package| (package.name.as_str(), package.version.as_str()))
            .collect::<Vec<_>>(),
        vec![("acme.dep", "1.2.0"), ("acme.dep", "1.2.3")]
    );
    assert!(first.resolved_dependencies.is_empty());
    assert_eq!(first.to_bytes().unwrap(), second.to_bytes().unwrap());
}

#[test]
fn exact_identity_cycle_terminates_by_deduplication_and_retains_closing_edge() {
    let mut source = MemorySource::default();
    source.add("acme.a", "1.0.0", &[("acme.b", "1.0.0")]);
    source.add("acme.b", "1.0.0", &[("acme.a", "1.0.0")]);
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());

    let lock = Resolver::new(&source, &cache)
        .resolve(vec![PackageRequest::parse("acme.a@1.0.0").unwrap()])
        .unwrap();

    assert_eq!(lock.packages.len(), 2);
    assert_eq!(lock.packages[0].name, "acme.a");
    assert_eq!(lock.packages[1].name, "acme.b");
    assert_eq!(lock.resolved_dependencies.len(), 2);
    assert!(lock.resolved_dependencies.iter().any(|edge| {
        edge.from_name == "acme.a"
            && edge.from_version == "1.0.0"
            && edge.to_name == "acme.b"
            && edge.to_version == "1.0.0"
    }));
    assert!(lock.resolved_dependencies.iter().any(|edge| {
        edge.from_name == "acme.b"
            && edge.from_version == "1.0.0"
            && edge.to_name == "acme.a"
            && edge.to_version == "1.0.0"
    }));
    lock.verify_cache(&cache).unwrap();
}

#[test]
fn lockfile_is_byte_stable_for_equivalent_root_sets() {
    let mut source = MemorySource::default();
    source.add("acme.one", "1.0.0", &[]);
    source.add("acme.two", "2.0.0", &[]);
    let first_dir = tempdir().unwrap();
    let second_dir = tempdir().unwrap();

    let first = Resolver::new(&source, &PackageCache::new(first_dir.path()))
        .resolve(vec![
            PackageRequest::parse("acme.two@2.0.0").unwrap(),
            PackageRequest::parse("acme.one@1.0.0").unwrap(),
        ])
        .unwrap()
        .to_bytes()
        .unwrap();
    let second = Resolver::new(&source, &PackageCache::new(second_dir.path()))
        .resolve(vec![
            PackageRequest::parse("acme.one@1.0.0").unwrap(),
            PackageRequest::parse("acme.two@2.0.0").unwrap(),
        ])
        .unwrap()
        .to_bytes()
        .unwrap();

    assert_eq!(first, second);
}

#[test]
fn lockfile_is_byte_stable_for_equivalent_multi_version_dependency_graphs() {
    let mut source = MemorySource::default();
    source.add("acme.left", "1.0.0", &[("acme.dep", "1.0.0")]);
    source.add("acme.right", "1.0.0", &[("acme.dep", "2.0.0")]);
    source.add("acme.dep", "1.0.0", &[]);
    source.add("acme.dep", "2.0.0", &[]);
    let first_dir = tempdir().unwrap();
    let second_dir = tempdir().unwrap();

    let first = Resolver::new(&source, &PackageCache::new(first_dir.path()))
        .resolve(vec![
            PackageRequest::parse("acme.right@1.0.0").unwrap(),
            PackageRequest::parse("acme.left@1.0.0").unwrap(),
        ])
        .unwrap()
        .to_bytes()
        .unwrap();
    let second = Resolver::new(&source, &PackageCache::new(second_dir.path()))
        .resolve(vec![
            PackageRequest::parse("acme.left@1.0.0").unwrap(),
            PackageRequest::parse("acme.right@1.0.0").unwrap(),
        ])
        .unwrap()
        .to_bytes()
        .unwrap();

    assert_eq!(first, second);
}

#[test]
fn identical_package_identity_and_dependencies_do_not_alias_distinct_origin_archive_bytes() {
    // Motivated by real dual-origin IPS 2.0.1 drift: the official registries
    // declare the same name/version/dependencies but publish different TAR
    // members and manifests. This test is fully synthetic and offline.
    let make_archive = |published: bool| {
        let mut manifest = serde_json::json!({
            "name": "acme.root",
            "version": "1.0.0",
            "dependencies": {"acme.dep": "1.0.0"},
            "url": if published {
                "https://example.test/official/1.0.0"
            } else {
                "file:///private/build/draft"
            },
        });
        if !published {
            manifest["notForPublication"] = serde_json::json!(true);
        }
        let bytes = serde_json::to_vec(&manifest).expect("manifest JSON");
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut tar = Builder::new(&mut encoder);
            let mut header = Header::new_gnu();
            header
                .set_path("package/package.json")
                .expect("tar manifest path");
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append(&header, Cursor::new(bytes))
                .expect("tar manifest");
            if !published {
                let publication_request = b"offline private publisher request";
                let mut extra = Header::new_gnu();
                extra
                    .set_path("package/other/publication-request.json")
                    .expect("tar extra path");
                extra.set_size(publication_request.len() as u64);
                extra.set_mode(0o644);
                extra.set_cksum();
                tar.append(&extra, Cursor::new(publication_request))
                    .expect("tar extra member");
            }
            tar.finish().expect("finalize tar");
        }
        encoder.finish().expect("gzip archive")
    };
    let mut first = MemorySource::default();
    let mut second = MemorySource::default();
    for source in [&mut first, &mut second] {
        source.add("acme.dep", "1.0.0", &[]);
    }
    first.packages.insert(
        ("acme.root".to_owned(), "1.0.0".to_owned()),
        make_archive(false),
    );
    second.packages.insert(
        ("acme.root".to_owned(), "1.0.0".to_owned()),
        make_archive(true),
    );
    let first_dir = tempdir().expect("first isolated cache");
    let second_dir = tempdir().expect("second isolated cache");
    let first_cache = PackageCache::new(first_dir.path());
    let second_cache = PackageCache::new(second_dir.path());
    let request = PackageRequest::parse("acme.root@1.0.0").expect("root request");
    let first_lock = Resolver::new(&first, &first_cache)
        .resolve(vec![request.clone()])
        .expect("resolve first archive");
    let second_lock = Resolver::new(&second, &second_cache)
        .resolve(vec![request])
        .expect("resolve second archive");
    first_lock
        .verify_cache(&first_cache)
        .expect("first exact bytes");
    second_lock
        .verify_cache(&second_cache)
        .expect("second exact bytes");

    let first_root = first_lock
        .packages
        .iter()
        .find(|item| item.name == "acme.root")
        .expect("first root");
    let second_root = second_lock
        .packages
        .iter()
        .find(|item| item.name == "acme.root")
        .expect("second root");

    assert_eq!(first_lock.roots, second_lock.roots);
    assert_eq!(
        first_lock.resolved_dependencies,
        second_lock.resolved_dependencies
    );
    assert_eq!(first_root.name, second_root.name);
    assert_eq!(first_root.version, second_root.version);
    assert_eq!(first_root.dependencies, second_root.dependencies);
    assert_eq!(first_root.source, second_root.source);
    assert_ne!(
        first_root.sha256, second_root.sha256,
        "different TAR member bytes must not collapse to the same digest"
    );
    assert_ne!(
        first_lock.to_bytes().unwrap(),
        second_lock.to_bytes().unwrap()
    );
    assert!(
        matches!(
            first_lock.verify_cache(&second_cache),
            Err(PackageError::CacheMissing(_))
        ),
        "the other cache does not contain the expected digest object"
    );
    assert!(
        matches!(
            second_lock.verify_cache(&first_cache),
            Err(PackageError::CacheMissing(_))
        ),
        "the missing-object rejection is symmetric"
    );

    // Distinguish a missing cross-origin object from actual content tampering:
    // write the other origin bytes under the digest expected by this lock.
    let first_object = first_cache
        .root()
        .join("sha256")
        .join(format!("{}.tgz", first_root.sha256));
    let second_object = second_cache
        .root()
        .join("sha256")
        .join(format!("{}.tgz", second_root.sha256));
    let other_bytes = std::fs::read(&second_object).expect("verified second cache bytes");
    std::fs::write(&first_object, other_bytes).expect("tamper first cache digest object");
    let error = first_lock
        .verify_cache(&first_cache)
        .expect_err("changed bytes under expected digest must fail closed");
    assert!(
        matches!(
            error,
            PackageError::CacheDigestMismatch { expected, found, .. }
                if expected == first_root.sha256 && found == second_root.sha256
        ),
        "wrong bytes at a present digest path must report a digest mismatch"
    );
}
