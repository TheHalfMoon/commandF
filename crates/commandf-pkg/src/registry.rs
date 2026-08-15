use std::collections::BTreeMap;
use std::time::Duration;

use semver::Version;
use serde::Deserialize;
use ureq::Agent;

use crate::{PackageArchive, PackageError, PackageName, PackageSource};

const PRIMARY: &str = "https://packages.fhir.org";
const SECONDARY: &str = "https://packages2.fhir.org/packages";
const SECONDARY_TARBALL_BASE: &str = "https://packages2.fhir.org/web";
const METADATA_LIMIT: u64 = 4 * 1024 * 1024;
const ARCHIVE_LIMIT: u64 = 128 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);
const GZIP_MAGIC: [u8; 2] = [0x1f, 0x8b];

#[derive(Debug, Deserialize)]
struct RegistryMetadata {
    #[serde(default)]
    versions: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone)]
pub struct FhirRegistrySource {
    agent: Agent,
}

impl Default for FhirRegistrySource {
    fn default() -> Self {
        let config = Agent::config_builder()
            .timeout_global(Some(REQUEST_TIMEOUT))
            .max_redirects(0)
            .build();
        Self {
            agent: config.into(),
        }
    }
}

impl FhirRegistrySource {
    pub fn new() -> Self {
        Self::default()
    }

    fn endpoints() -> [&'static str; 2] {
        [PRIMARY, SECONDARY]
    }

    fn metadata_from(
        &self,
        endpoint: &str,
        name: &PackageName,
    ) -> Result<RegistryMetadata, String> {
        let url = format!("{endpoint}/{name}");
        let mut response = self
            .agent
            .get(&url)
            .call()
            .map_err(|error| error.to_string())?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(METADATA_LIMIT)
            .read_to_vec()
            .map_err(|error| error.to_string())?;
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())
    }

    fn archive_from(
        &self,
        endpoint: &str,
        name: &PackageName,
        version: &Version,
    ) -> Result<PackageArchive, String> {
        let url = format!("{endpoint}/{name}/{version}");
        let mut response = self
            .agent
            .get(&url)
            .call()
            .map_err(|error| error.to_string())?;

        let status = response.status().as_u16();
        if (300..400).contains(&status) {
            let location = response
                .headers()
                .get("location")
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| format!("registry redirect from {url} omitted a valid Location header"))?;
            let target = validated_secondary_redirect(endpoint, status, name, version, location)?;
            return self.direct_archive_from_url(&target);
        }

        if !(200..300).contains(&status) {
            return Err(format!("registry download from {url} returned HTTP {status}"));
        }

        let bytes = read_archive_body(&mut response, &url)?;
        validate_gzip_archive(&bytes, &url)?;
        Ok(PackageArchive { bytes, source: url })
    }

    fn direct_archive_from_url(&self, url: &str) -> Result<PackageArchive, String> {
        let mut response = self
            .agent
            .get(url)
            .call()
            .map_err(|error| error.to_string())?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            return Err(format!(
                "registry tarball download from {url} returned HTTP {status}; redirects are not followed recursively"
            ));
        }
        let bytes = read_archive_body(&mut response, url)?;
        validate_gzip_archive(&bytes, url)?;
        Ok(PackageArchive {
            bytes,
            source: url.to_owned(),
        })
    }
}

fn read_archive_body(
    response: &mut ureq::http::Response<ureq::Body>,
    url: &str,
) -> Result<Vec<u8>, String> {
    response
        .body_mut()
        .with_config()
        .limit(ARCHIVE_LIMIT)
        .read_to_vec()
        .map_err(|error| format!("registry archive body from {url} failed: {error}"))
}

fn validate_gzip_archive(bytes: &[u8], url: &str) -> Result<(), String> {
    if !bytes.starts_with(&GZIP_MAGIC) {
        return Err(format!(
            "registry response from {url} is not a gzip package archive"
        ));
    }
    Ok(())
}

fn expected_secondary_tarball(name: &PackageName, version: &Version) -> String {
    format!("{SECONDARY_TARBALL_BASE}/{name}-{version}.tgz")
}

fn validated_secondary_redirect(
    endpoint: &str,
    status: u16,
    name: &PackageName,
    version: &Version,
    location: &str,
) -> Result<String, String> {
    if endpoint != SECONDARY || status != 302 {
        return Err(format!(
            "unexpected registry redirect for {name}@{version}: endpoint={endpoint} status={status}"
        ));
    }
    let expected = expected_secondary_tarball(name, version);
    if location != expected {
        return Err(format!(
            "unexpected secondary registry redirect for {name}@{version}: expected {expected}, found {location}"
        ));
    }
    Ok(expected)
}

impl PackageSource for FhirRegistrySource {
    fn source_id(&self) -> String {
        "fhir-package-registry".to_owned()
    }

    fn available_versions(&self, name: &PackageName) -> Result<Vec<Version>, PackageError> {
        let mut errors = Vec::new();
        for endpoint in Self::endpoints() {
            match self.metadata_from(endpoint, name) {
                Ok(metadata) => {
                    let parsed = metadata
                        .versions
                        .keys()
                        .map(|raw| Version::parse(raw))
                        .collect::<Result<Vec<_>, _>>();
                    match parsed {
                        Ok(mut versions) => {
                            versions.sort();
                            return Ok(versions);
                        }
                        Err(error) => errors.push(format!("{endpoint}: {error}")),
                    }
                }
                Err(error) => errors.push(format!("{endpoint}: {error}")),
            }
        }
        Err(PackageError::Registry(format!(
            "metadata lookup for {name} failed; {}",
            errors.join("; ")
        )))
    }

    fn archive(&self, name: &PackageName, version: &Version) -> Result<Vec<u8>, PackageError> {
        Ok(self.archive_with_source(name, version)?.bytes)
    }

    fn archive_with_source(
        &self,
        name: &PackageName,
        version: &Version,
    ) -> Result<PackageArchive, PackageError> {
        let mut errors = Vec::new();
        for endpoint in Self::endpoints() {
            match self.archive_from(endpoint, name, version) {
                Ok(archive) => return Ok(archive),
                Err(error) => errors.push(format!("{endpoint}: {error}")),
            }
        }
        Err(PackageError::Registry(format!(
            "download for {name}@{version} failed; {}",
            errors.join("; ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package() -> PackageName {
        PackageName::parse("hl7.fhir.us.core").unwrap()
    }

    fn version() -> Version {
        Version::parse("8.0.1").unwrap()
    }

    #[test]
    fn accepts_expected_secondary_redirect_only() {
        let name = package();
        let version = version();
        let expected = "https://packages2.fhir.org/web/hl7.fhir.us.core-8.0.1.tgz";
        assert_eq!(
            validated_secondary_redirect(SECONDARY, 302, &name, &version, expected).unwrap(),
            expected
        );
    }

    #[test]
    fn rejects_redirect_from_primary_endpoint() {
        let error = validated_secondary_redirect(
            PRIMARY,
            302,
            &package(),
            &version(),
            "https://packages2.fhir.org/web/hl7.fhir.us.core-8.0.1.tgz",
        )
        .unwrap_err();
        assert!(error.contains("unexpected registry redirect"));
    }

    #[test]
    fn rejects_unexpected_secondary_redirect_target() {
        let error = validated_secondary_redirect(
            SECONDARY,
            302,
            &package(),
            &version(),
            "https://example.invalid/hl7.fhir.us.core-8.0.1.tgz",
        )
        .unwrap_err();
        assert!(error.contains("unexpected secondary registry redirect"));
    }

    #[test]
    fn rejects_non_302_secondary_redirect() {
        let error = validated_secondary_redirect(
            SECONDARY,
            307,
            &package(),
            &version(),
            "https://packages2.fhir.org/web/hl7.fhir.us.core-8.0.1.tgz",
        )
        .unwrap_err();
        assert!(error.contains("unexpected registry redirect"));
    }

    #[test]
    fn rejects_non_gzip_registry_body() {
        let error = validate_gzip_archive(b"Found. Redirecting", "https://example.test/package")
            .unwrap_err();
        assert!(error.contains("not a gzip package archive"));
    }

    #[test]
    fn accepts_gzip_magic() {
        validate_gzip_archive(&[0x1f, 0x8b, 0x08, 0x00], "https://example.test/package")
            .unwrap();
    }
}
