use std::collections::BTreeMap;
use std::time::Duration;

use semver::Version;
use serde::Deserialize;
use ureq::Agent;

use crate::{PackageArchive, PackageError, PackageName, PackageSource};

const PRIMARY: &str = "https://packages.fhir.org";
const SECONDARY: &str = "https://packages2.fhir.org/packages";
const METADATA_LIMIT: usize = 4 * 1024 * 1024;
const ARCHIVE_LIMIT: usize = 128 * 1024 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

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

    fn metadata_from(&self, endpoint: &str, name: &PackageName) -> Result<RegistryMetadata, String> {
        let url = format!("{endpoint}/{name}");
        let mut response = self.agent.get(&url).call().map_err(|error| error.to_string())?;
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
        let mut response = self.agent.get(&url).call().map_err(|error| error.to_string())?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(ARCHIVE_LIMIT)
            .read_to_vec()
            .map_err(|error| error.to_string())?;
        Ok(PackageArchive { bytes, source: url })
    }
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
                    let mut versions = metadata
                        .versions
                        .keys()
                        .filter_map(|raw| Version::parse(raw).ok())
                        .collect::<Vec<_>>();
                    versions.sort();
                    return Ok(versions);
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
