use std::collections::BTreeMap;
use std::io::Cursor;

use commandf_pkg::{PackageCache, PackageError, PackageName, PackageRequest, PackageSource, Resolver};
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
        builder.finish().unwrap();
    }
    encoder.finish().unwrap()
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

    assert_eq!(lock.packages.len(), 2);
    assert_eq!(
        lock.packages
            .iter()
            .find(|package| package.name == "acme.dep")
            .unwrap()
            .version,
        "1.2.3"
    );
    lock.verify_cache(&cache).unwrap();
}

#[test]
fn incompatible_versions_fail_closed() {
    let mut source = MemorySource::default();
    source.add("acme.left", "1.0.0", &[("acme.dep", "1.0.0")]);
    source.add("acme.right", "1.0.0", &[("acme.dep", "2.0.0")]);
    source.add("acme.dep", "1.0.0", &[]);
    source.add("acme.dep", "2.0.0", &[]);
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());

    let error = Resolver::new(&source, &cache)
        .resolve(vec![
            PackageRequest::parse("acme.left@1.0.0").unwrap(),
            PackageRequest::parse("acme.right@1.0.0").unwrap(),
        ])
        .unwrap_err();

    assert!(matches!(error, PackageError::VersionConflict { .. }));
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
