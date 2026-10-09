use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use semver::Version;

use crate::{PackageError, PackageName};

/// Compressed package archive limit shared with registry acquisition.
pub const MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES: u64 = 128 * 1024 * 1024;

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
        read_limited_archive(&path, MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES).map_err(|error| {
            match error {
                PackageError::Io(source) if source.kind() == std::io::ErrorKind::NotFound => {
                    PackageError::PackageNotFound {
                        name: name.to_string(),
                        version: version.to_string(),
                    }
                }
                other => other,
            }
        })
    }
}

fn read_limited_archive(path: &Path, max_bytes: u64) -> Result<Vec<u8>, PackageError> {
    let file = File::open(path)?;
    // Reject oversized regular files before reading their contents.
    // Still keep the bounded streaming check for growth after metadata().
    let max_bytes = max_bytes.min(MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES);
    reject_archive_size(file.metadata()?.len(), max_bytes)?;
    let mut bytes = Vec::new();
    file.take(max_bytes + 1).read_to_end(&mut bytes)?;
    reject_archive_size(bytes.len() as u64, max_bytes)?;
    Ok(bytes)
}

fn reject_archive_size(size: u64, max_bytes: u64) -> Result<(), PackageError> {
    if size > max_bytes {
        return Err(PackageError::InvalidRequest(format!(
            "package archive exceeds the maximum supported size of {max_bytes} bytes"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_mirror_reads_exact_bound_and_rejects_one_extra_byte() {
        let directory =
            std::env::temp_dir().join(format!("commandf-mirror-bound-{}", std::process::id()));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).expect("temp directory");
        let path = directory.join("package.tgz");
        fs::write(&path, b"abcd").expect("archive");

        let exact = read_limited_archive(&path, 4).expect("exact bound");
        assert_eq!(exact, b"abcd");
        let error = read_limited_archive(&path, 3).expect_err("bound plus one");
        assert!(error
            .to_string()
            .contains("package archive exceeds the maximum supported size of 3 bytes"));
        assert!(!error.to_string().contains(&path.display().to_string()));
        let _ = fs::remove_dir_all(directory);
    }

    #[test]
    fn sparse_oversized_archive_fails_before_streaming() {
        let directory = tempfile::tempdir().expect("temporary mirror root");
        let path = directory.path().join("large.tgz");
        File::create(&path)
            .expect("create sparse archive")
            .set_len(MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES + 1)
            .expect("expand sparse archive");
        let error = read_limited_archive(&path, u64::MAX)
            .expect_err("oversized mirror archive must be refused early");
        assert!(matches!(error, PackageError::InvalidRequest(_)));
        assert!(error.to_string().contains("package archive exceeds"));
        assert!(!error.to_string().contains(&path.display().to_string()));
    }

    #[test]
    fn mirror_bounded_reader_rejects_oversized_limits_and_preserves_exact_bound() {
        let directory = tempfile::tempdir().expect("temporary mirror root");
        let path = directory.path().join("small.tgz");
        fs::write(&path, b"abcd").expect("write archive");
        assert_eq!(
            read_limited_archive(&path, u64::MAX).expect("clamped read"),
            b"abcd"
        );
        assert_eq!(read_limited_archive(&path, 4).expect("exact read"), b"abcd");
        assert!(matches!(
            read_limited_archive(&path, 3),
            Err(PackageError::InvalidRequest(_))
        ));
    }

    #[test]
    fn missing_local_mirror_archive_stays_package_not_found() {
        let directory =
            std::env::temp_dir().join(format!("commandf-mirror-missing-{}", std::process::id()));
        let source = LocalMirrorSource::new(&directory);
        let name = PackageName::parse("acme.subject").expect("name");
        let version = Version::parse("1.0.0").expect("version");
        let error = source
            .archive(&name, &version)
            .expect_err("missing archive");
        assert!(matches!(error, PackageError::PackageNotFound { .. }));
    }

    #[test]
    fn local_mirror_archive_limit_matches_registry_acquisition() {
        assert_eq!(MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES, 128 * 1024 * 1024);
    }
}
