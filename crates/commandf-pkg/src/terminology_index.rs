use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use serde_json::{Map, Value};

use crate::{
    archive::read_manifest, artifact_scan::scan_package_resources, compare_value_set_expansions,
    Lockfile, PackageCache, PackageError, ResourceKey, ResourceKeyKind, TerminologyError,
    TerminologyProofMode, TerminologyRelation,
};

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

            let mut seen_filenames = BTreeSet::new();
            for scanned in scan_package_resources(&bytes)? {
                let filename = scanned.filename;
                if !seen_filenames.insert(filename.clone()) {
                    return Err(TerminologyError::InvalidField {
                        resource: format!("{}@{}", package.name, package.version),
                        field: filename,
                        message: "duplicate package resource filename".to_owned(),
                    });
                }

                let value: Value = serde_json::from_slice(&scanned.bytes).map_err(|source| {
                    TerminologyError::Json {
                        file: filename.clone(),
                        source,
                    }
                })?;
                let object = value
                    .as_object()
                    .ok_or_else(|| TerminologyError::InvalidField {
                        resource: filename.clone(),
                        field: "resourceType".to_owned(),
                        message: "FHIR package resource must be a JSON object".to_owned(),
                    })?;
                let resource_type = required_string(object, "resourceType", &filename)?;

                // The lock-closure index exists only to resolve StructureDefinition binding
                // references. Direct root CodeSystem/ValueSet deltas are handled by CF-03
                // matched-resource authority, so dependency CodeSystem canonicals must not create
                // unrelated ambiguity here.
                if resource_type != "ValueSet" {
                    continue;
                }

                let Some(url) = optional_string(object, "url", &filename)? else {
                    continue;
                };
                let version = optional_string(object, "version", &filename)?;
                closure.insert(TerminologyResource {
                    package_name: package.name.clone(),
                    package_version: package.version.clone(),
                    filename,
                    resource_type,
                    url,
                    version,
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
            // Multi-version package graphs and companion packages may repeat the same canonical
            // ValueSet with byte/JSON differences in metadata that CF-07 never uses for binding
            // proof. Reuse CF-07's own normalized ValueSet-expansion comparison as the authority:
            // only identical binding evidence is safely deduplicated. Conflicting evidence remains
            // fail-closed.
            if value_set_binding_evidence_equivalent(first, &resource)? {
                return Ok(());
            }
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

fn value_set_binding_evidence_equivalent(
    first: &TerminologyResource,
    second: &TerminologyResource,
) -> Result<bool, TerminologyError> {
    let resource = ResourceKey {
        kind: ResourceKeyKind::Canonical,
        value: exact_identity(&first.url, first.version.as_deref()),
    };
    let first_self = compare_value_set_expansions(resource.clone(), &first.value, &first.value)?;
    let second_self = compare_value_set_expansions(resource.clone(), &second.value, &second.value)?;
    if first_self != second_self {
        return Ok(false);
    }

    let cross = compare_value_set_expansions(resource, &first.value, &second.value)?;
    match cross.proof_mode {
        Some(TerminologyProofMode::ValueSetExpansion) => {
            Ok(cross.relation == TerminologyRelation::Equal)
        }
        Some(_) => Ok(false),
        None => Ok(cross.relation == TerminologyRelation::Indeterminate),
    }
}

fn required_string(
    object: &Map<String, Value>,
    field: &str,
    filename: &str,
) -> Result<String, TerminologyError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| TerminologyError::InvalidField {
            resource: filename.to_owned(),
            field: field.to_owned(),
            message: "must be a string".to_owned(),
        })
}

fn optional_string(
    object: &Map<String, Value>,
    field: &str,
    filename: &str,
) -> Result<Option<String>, TerminologyError> {
    match object.get(field) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(TerminologyError::InvalidField {
            resource: filename.to_owned(),
            field: field.to_owned(),
            message: "must be a string when present".to_owned(),
        }),
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
    use std::collections::BTreeMap;
    use std::io::Cursor;

    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::{Builder, Header};
    use tempfile::TempDir;

    use super::*;
    use crate::LockedPackage;

    fn package_archive(resources: &[(&str, &str)]) -> Vec<u8> {
        let manifest = br#"{"name":"example.pkg","version":"1.0.0","dependencies":{}}"#;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        {
            let mut builder = Builder::new(&mut encoder);
            append(&mut builder, "package/package.json", manifest);
            for (filename, body) in resources {
                append(
                    &mut builder,
                    &format!("package/{filename}"),
                    body.as_bytes(),
                );
            }
            builder.finish().unwrap();
        }
        encoder.finish().unwrap()
    }

    fn append(builder: &mut Builder<&mut GzEncoder<Vec<u8>>>, path: &str, body: &[u8]) {
        let mut header = Header::new_gnu();
        header.set_path(path).unwrap();
        header.set_size(body.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, Cursor::new(body)).unwrap();
    }

    fn closure_for(resources: &[(&str, &str)]) -> Result<TerminologyClosure, TerminologyError> {
        let temp = TempDir::new().unwrap();
        let cache = PackageCache::new(temp.path());
        let bytes = package_archive(resources);
        let sha256 = cache.put(&bytes).unwrap();
        let lockfile = Lockfile::new(
            vec!["example.pkg@1.0.0".to_owned()],
            vec![LockedPackage {
                name: "example.pkg".to_owned(),
                version: "1.0.0".to_owned(),
                sha256,
                source: "https://packages.example.org/example.pkg/1.0.0".to_owned(),
                dependencies: BTreeMap::new(),
            }],
        );
        TerminologyClosure::load(&lockfile, &cache)
    }

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
    fn non_binding_canonical_collisions_do_not_block_value_set_closure() {
        let closure = closure_for(&[
            (
                "CapabilityStatement-example.json",
                r#"{"resourceType":"CapabilityStatement","url":"urn:uuid:shared","version":"1"}"#,
            ),
            (
                "TerminologyCapabilities-example.json",
                r#"{"resourceType":"TerminologyCapabilities","url":"urn:uuid:shared","version":"1"}"#,
            ),
            (
                "CodeSystem-a.json",
                r#"{"resourceType":"CodeSystem","url":"http://example.org/CodeSystem/shared","version":"1"}"#,
            ),
            (
                "CodeSystem-b.json",
                r#"{"resourceType":"CodeSystem","url":"http://example.org/CodeSystem/shared","version":"1"}"#,
            ),
            (
                "ValueSet-test.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1"}"#,
            ),
        ])
        .unwrap();

        let resolved = closure
            .resolve_value_set("http://example.org/ValueSet/test|1")
            .unwrap()
            .expect("ValueSet should resolve");
        assert_eq!(resolved.filename, "ValueSet-test.json");
    }

    #[test]
    fn identical_value_set_canonical_is_deduplicated() {
        let body = r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1","status":"active"}"#;
        let closure = closure_for(&[("ValueSet-a.json", body), ("ValueSet-b.json", body)]).unwrap();

        let resolved = closure
            .resolve_value_set("http://example.org/ValueSet/test|1")
            .unwrap()
            .expect("identical ValueSet should resolve once");
        assert_eq!(resolved.url, "http://example.org/ValueSet/test");
    }

    #[test]
    fn equivalent_indeterminate_binding_evidence_is_deduplicated() {
        let closure = closure_for(&[
            (
                "ValueSet-a.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1","status":"active","name":"First"}"#,
            ),
            (
                "ValueSet-b.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1","status":"draft","name":"Second"}"#,
            ),
        ])
        .unwrap();

        assert!(closure
            .resolve_value_set("http://example.org/ValueSet/test|1")
            .unwrap()
            .is_some());
    }

    #[test]
    fn equivalent_finite_binding_evidence_is_deduplicated() {
        let closure = closure_for(&[
            (
                "ValueSet-a.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1","status":"active","expansion":{"total":1,"parameter":[{"name":"includeDesignations","valueBoolean":false}],"contains":[{"system":"http://example.org/system","code":"A"}]}}"#,
            ),
            (
                "ValueSet-b.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1","status":"draft","title":"Different metadata","expansion":{"contains":[{"code":"A","system":"http://example.org/system"}],"parameter":[{"valueBoolean":false,"name":"includeDesignations"}],"total":1}}"#,
            ),
        ])
        .unwrap();

        assert!(closure
            .resolve_value_set("http://example.org/ValueSet/test|1")
            .unwrap()
            .is_some());
    }

    #[test]
    fn conflicting_value_set_binding_evidence_still_fails_closed() {
        let result = closure_for(&[
            (
                "ValueSet-a.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1","expansion":{"total":1,"contains":[{"system":"http://example.org/system","code":"A"}]}}"#,
            ),
            (
                "ValueSet-b.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1","expansion":{"total":1,"contains":[{"system":"http://example.org/system","code":"B"}]}}"#,
            ),
        ]);

        assert!(matches!(
            result,
            Err(TerminologyError::DuplicateCanonical { .. })
        ));
    }

    #[test]
    fn different_indeterminate_binding_evidence_still_fails_closed() {
        let result = closure_for(&[
            (
                "ValueSet-a.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1"}"#,
            ),
            (
                "ValueSet-b.json",
                r#"{"resourceType":"ValueSet","url":"http://example.org/ValueSet/test","version":"1","expansion":{"offset":1,"total":0}}"#,
            ),
        ]);

        assert!(matches!(
            result,
            Err(TerminologyError::DuplicateCanonical { .. })
        ));
    }
}
