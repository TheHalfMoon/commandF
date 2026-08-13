use std::fs;
use std::path::{Path, PathBuf};

use semver::Version;

use crate::{PackageError, PackageName};

pub trait PackageSource {
    fn source_id(&self) -> String;
    fn available_versions(&self, name: &PackageName) -> Result<Vec<Version>, PackageError>;
    fn archive(&self, name: &PackageName, version: &Version) -> Result<Vec<u8>, PackageError>;
}

#[derive(Clone, Debug)]
pub struct LocalMirrorSource {
    root: PathBuf,
}

impl LocalMirrorSource {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}

impl PackageSource for LocalMirrorSource {
    fn source_id(&self) -> String {
        format!("local-mirror:{}", self.root.display())
    }

    fn available_versions(&self, name: &PackageName) -> Result<Vec<Version>, PackageError> {
        let package_dir = self.root.join(name.as_str());
        let mut versions = Vec::new();
        let entries = fs::read_dir(&package_dir).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                PackageError::PackageNotFound {
                    name: name.to_string(),
                    version: "*".to_owned(),
                }
            } else {
                PackageError::Io(error)
            }
        })?;

        for entry in entries {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            if let Some(raw) = entry.file_name().to_str() {
                if let Ok(version) = Version::parse(raw) {
                    versions.push(version);
                }
            }
        }
        versions.sort();
        Ok(versions)
    }

    fn archive(&self, name: &PackageName, version: &Version) -> Result<Vec<u8>, PackageError> {
        let path = self
            .root
            .join(name.as_str())
            .join(version.to_string())
            .join("package.tgz");
        fs::read(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                PackageError::PackageNotFound {
                    name: name.to_string(),
                    version: version.to_string(),
                }
            } else {
                PackageError::Io(error)
            }
        })
    }
}
