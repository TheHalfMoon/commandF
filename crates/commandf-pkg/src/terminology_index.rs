use std::collections::BTreeMap;
use std::fs;

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
        lockfile.verify_cache(cache)?;
        let mut closure = Self::default();

        for package in &lockfile.packages {
            let path = cache
                .root()
                .join("sha256")
                .join(format!("{}.tgz", package.sha256));
            let bytes = fs::read(path).map_err(PackageError::Io)?;
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
}
