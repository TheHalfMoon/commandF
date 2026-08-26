use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use semver::Version;

use crate::{PackageError, PackageName};

pub(crate) const MAX_PACKAGE_ARCHIVE_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageArchive {
    pub bytes: Vec<u8>,
    pub source: String,
}

pub trait PackageSource {
    fn source_id(&self) -> String;
    fn available_versions(&self, name: &PackageName) -> Result<Vec<Version>, PackageError>;
    fn archive(&self, name: &PackageName, version: &Version) -> Result<Vec<u8>, PackageError>;

    fn archive_with_source(
        &self,
        name: &PackageName,
        version: &Version,
    ) -> Result<PackageArchive, PackageError> {
        Ok(PackageArchive {
            bytes: self.archive(name, version)?,
            source: self.source_id(),
        })
    }
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
        "local-mirror".to_owned()
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
        let file = fs::File::open(path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                PackageError::PackageNotFound {
                    name: name.to_string(),
                    version: version.to_string(),
                }
            } else {
                PackageError::Io(error)
            }
        })?;
        read_bounded_archive(file, MAX_PACKAGE_ARCHIVE_BYTES)
    }
}

fn read_bounded_archive<R: Read>(reader: R, max_bytes: u64) -> Result<Vec<u8>, PackageError> {
    let mut bytes = Vec::new();
    reader
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(PackageError::InvalidRequest(format!(
            "package archive exceeds the maximum supported compressed size of {max_bytes} bytes"
        )));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn bounded_archive_reader_accepts_exact_limit() {
        let bytes = read_bounded_archive(Cursor::new(b"abcd"), 4).expect("exact bound");
        assert_eq!(bytes, b"abcd");
    }

    #[test]
    fn bounded_archive_reader_rejects_limit_plus_one() {
        let error = read_bounded_archive(Cursor::new(b"abcde"), 4)
            .expect_err("limit plus one must fail closed");
        assert!(matches!(error, PackageError::InvalidRequest(_)));
    }
}
