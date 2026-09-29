use std::collections::BTreeMap;

use serde_json::Value;

use crate::{
    archive::read_manifest, artifact_scan::scan_package_resources, inspect_package, Lockfile,
    PackageCache, PackageError, TerminologyError,
};

const TERMINOLOGY_TYPES: [&str; 2] = ["CodeSystem", "ValueSet"];

#[derive(Clone, Debug)]
pub(crate) struct TerminologyResource {
    pub package_name: String,
    pub package_version: String,
    pub filename: String,
    pub resource_type: String,
    pub url: String,
    pub version: Option<String>,
    pub value: Value,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TerminologyClosure {
    by_url: BTreeMap<String, Vec<TerminologyResource>>,
    exact: BTreeMap<String, TerminologyResource>,
}

impl TerminologyClosure {
    pub(crate) fn load(
        lockfile: &Lockfile,
        cache: &PackageCache,
    ) -> Result<Self, TerminologyError> {
        Self::load_consuming(lockfile, |digest| cache.read_verified(digest))
    }

    fn load_consuming<F>(
        lockfile: &Lockfile,
        mut read_verified: F,
    ) -> Result<Self, TerminologyError>
    where
        F: FnMut(&str) -> Result<Vec<u8>, PackageError>,
    {
        let mut closure = Self::default();

        for package in &lockfile.packages {
            let bytes = read_verified(&package.sha256)?;
            let manifest = read_manifest(&bytes)?;
            if manifest.name != package.name || manifest.version != package.version {
                return Err(TerminologyError::InvalidField {
                    resource: format!("{}@{}", package.name, package.version),
                    field: "package/package.json".to_owned(),
                    message: format!(
                        "lock identity does not match archive manifest {}@{}",
                        manifest.name, manifest.version
                    ),
                });
            }

            let inspection =
                inspect_package(&package.name, &package.version, &package.sha256, &bytes)?;
            let mut raw = BTreeMap::new();
            for resource in scan_package_resources(&bytes)? {
                let filename = resource.filename;
                let value = serde_json::from_slice(&resource.bytes).map_err(|source| {
                    TerminologyError::Json {
                        file: filename.clone(),
                        source,
                    }
                })?;
                if raw.insert(filename.clone(), value).is_some() {
                    return Err(TerminologyError::InvalidField {
                        resource: format!("{}@{}", package.name, package.version),
                        field: filename,
                        message: "duplicate package resource filename".to_owned(),
                    });
                }
            }

            for resource in inspection.resources {
                if !TERMINOLOGY_TYPES.contains(&resource.resource_type.as_str()) {
                    continue;
                }
                let Some(url) = resource.canonical_url else {
                    continue;
                };
                let value = raw.get(&resource.filename).cloned().ok_or_else(|| {
                    TerminologyError::InvalidField {
                        resource: resource.filename.clone(),
                        field: "resource".to_owned(),
                        message: "inspected terminology resource is missing from scanned archive"
                            .to_owned(),
                    }
                })?;
                closure.insert(TerminologyResource {
                    package_name: package.name.clone(),
                    package_version: package.version.clone(),
                    filename: resource.filename,
                    resource_type: resource.resource_type,
                    url,
                    version: resource.canonical_version,
                    value,
                })?;
            }
        }

        for resources in closure.by_url.values_mut() {
            resources.sort_by(|left, right| {
                left.version
                    .cmp(&right.version)
                    .then_with(|| left.package_name.cmp(&right.package_name))
                    .then_with(|| left.package_version.cmp(&right.package_version))
                    .then_with(|| left.filename.cmp(&right.filename))
            });
        }
        Ok(closure)
    }

    fn insert(&mut self, resource: TerminologyResource) -> Result<(), TerminologyError> {
        let exact = exact_identity(&resource.url, resource.version.as_deref());
        if let Some(first) = self.exact.get(&exact) {
            return Err(TerminologyError::DuplicateCanonical {
                canonical: exact,
                first: location(first),
                second: location(&resource),
            });
        }
        self.exact.insert(exact, resource.clone());
        self.by_url
            .entry(resource.url.clone())
            .or_default()
            .push(resource);
        Ok(())
    }

    pub(crate) fn resolve_value_set(
        &self,
        reference: &str,
    ) -> Result<Option<&TerminologyResource>, TerminologyError> {
        let (url, version) = parse_canonical_reference(reference)?;
        if let Some(version) = version {
            let key = exact_identity(url, Some(version));
            return Ok(self
                .exact
                .get(&key)
                .filter(|resource| resource.resource_type == "ValueSet"));
        }

        let Some(resources) = self.by_url.get(url) else {
            return Ok(None);
        };
        let matches = resources
            .iter()
            .filter(|resource| resource.resource_type == "ValueSet")
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [] => Ok(None),
            [single] => Ok(Some(*single)),
            _ => Err(TerminologyError::AmbiguousCanonical {
                canonical: url.to_owned(),
                matches: matches.len(),
            }),
        }
    }
}

fn exact_identity(url: &str, version: Option<&str>) -> String {
    match version {
        Some(version) => format!("{url}|{version}"),
        None => url.to_owned(),
    }
}

fn parse_canonical_reference(reference: &str) -> Result<(&str, Option<&str>), TerminologyError> {
    if reference.is_empty()
        || reference.trim() != reference
        || reference.chars().any(char::is_whitespace)
    {
        return Err(TerminologyError::MalformedCanonical {
            reference: reference.to_owned(),
        });
    }
    let mut parts = reference.split('|');
    let url = parts.next().unwrap_or_default();
    let version = parts.next();
    if url.is_empty() || version.is_some_and(str::is_empty) || parts.next().is_some() {
        return Err(TerminologyError::MalformedCanonical {
            reference: reference.to_owned(),
        });
    }
    Ok((url, version))
}

fn location(resource: &TerminologyResource) -> String {
    format!(
        "{}@{}:{}",
        resource.package_name, resource.package_version, resource.filename
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_reference_parser_is_exact_and_fail_closed() {
        assert_eq!(
            parse_canonical_reference("http://example.org/ValueSet/test").unwrap(),
            ("http://example.org/ValueSet/test", None)
        );
        assert_eq!(
            parse_canonical_reference("http://example.org/ValueSet/test|1").unwrap(),
            ("http://example.org/ValueSet/test", Some("1"))
        );
        for malformed in ["", " x", "x ", "x|", "x|1|2", "x y"] {
            assert!(matches!(
                parse_canonical_reference(malformed),
                Err(TerminologyError::MalformedCanonical { .. })
            ));
        }
    }

    #[test]
    fn terminology_closure_uses_the_verified_reader_once_per_package() {
        let bytes = package_archive(
            "acme.codes",
            "1.0.0",
            "http://example.org/CodeSystem/verified-reader",
        );
        let digest = PackageCache::digest(&bytes);
        let lockfile = sample_lock(&digest);
        let mut reads = 0_usize;
        let closure = TerminologyClosure::load_consuming(&lockfile, |observed| {
            reads += 1;
            assert_eq!(observed, digest);
            Ok(bytes.clone())
        })
        .expect("verified reader bytes");

        assert_eq!(reads, 1);
        assert!(closure
            .by_url
            .contains_key("http://example.org/CodeSystem/verified-reader"));
    }

    #[test]
    fn replaced_cache_object_cannot_supply_terminology_bytes() {
        let directory = tempfile::tempdir().expect("temp directory");
        let cache = PackageCache::new(directory.path());
        let bytes = package_archive(
            "acme.codes",
            "1.0.0",
            "http://example.org/CodeSystem/original",
        );
        let digest = cache.put(&bytes).expect("cache object");
        let replacement = package_archive(
            "acme.codes",
            "1.0.0",
            "http://example.org/CodeSystem/replaced",
        );
        std::fs::write(
            cache.root().join("sha256").join(format!("{digest}.tgz")),
            replacement,
        )
        .expect("replace cache object");

        let error = TerminologyClosure::load(&sample_lock(&digest), &cache)
            .expect_err("replaced cache object must fail closed");
        assert!(matches!(
            error,
            TerminologyError::Package(PackageError::CacheDigestMismatch { .. })
        ));
    }

    fn sample_lock(digest: &str) -> Lockfile {
        Lockfile::new(
            vec!["acme.codes@1.0.0".to_owned()],
            vec![crate::LockedPackage {
                name: "acme.codes".to_owned(),
                version: "1.0.0".to_owned(),
                sha256: digest.to_owned(),
                source: "memory".to_owned(),
                dependencies: Default::default(),
            }],
        )
    }

    fn package_archive(name: &str, version: &str, url: &str) -> Vec<u8> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::Builder;

        let manifest = serde_json::json!({
            "name": name,
            "version": version
        });
        let code_system = serde_json::json!({
            "resourceType": "CodeSystem",
            "url": url,
            "version": "1"
        });
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut builder = Builder::new(&mut encoder);
            append_entry(
                &mut builder,
                "package/package.json",
                &serde_json::to_vec(&manifest).expect("manifest"),
            );
            append_entry(
                &mut builder,
                "package/CodeSystem-test.json",
                &serde_json::to_vec(&code_system).expect("code system"),
            );
            builder.finish().expect("archive");
        }
        encoder.finish().expect("gzip")
    }

    fn append_entry(
        builder: &mut tar::Builder<&mut flate2::write::GzEncoder<Vec<u8>>>,
        path: &str,
        body: &[u8],
    ) {
        use std::io::Cursor;
        use tar::Header;

        let mut header = Header::new_gnu();
        header.set_path(path).expect("path");
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, Cursor::new(body)).expect("entry");
    }
}
