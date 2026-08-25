use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::PackageError;

#[derive(Clone, Debug)]
pub struct PackageCache {
    root: PathBuf,
}

impl PackageCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn digest(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }

    fn object_path(&self, digest: &str) -> PathBuf {
        self.root.join("sha256").join(format!("{digest}.tgz"))
    }

    pub fn put(&self, bytes: &[u8]) -> Result<String, PackageError> {
        let digest = Self::digest(bytes);
        let path = self.object_path(&digest);
        if path.exists() {
            self.verify(&digest)?;
            return Ok(digest);
        }

        let parent = path
            .parent()
            .expect("cache object path always has a sha256 parent");
        fs::create_dir_all(parent)?;

        let mut temporary = NamedTempFile::new_in(parent)?;
        temporary.write_all(bytes)?;
        temporary.as_file().sync_all()?;

        if let Err(error) = temporary.persist(&path) {
            if path.exists() {
                self.verify(&digest)?;
                return Ok(digest);
            }
            return Err(PackageError::Io(error.into()));
        }

        self.verify(&digest)?;
        Ok(digest)
    }

    pub fn verify(&self, digest: &str) -> Result<(), PackageError> {
        self.read_verified(digest).map(|_| ())
    }

    pub fn read_verified(&self, digest: &str) -> Result<Vec<u8>, PackageError> {
        validate_digest(digest)?;
        let path = self.object_path(digest);
        let bytes = fs::read(&path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                PackageError::CacheMissing(digest.to_owned())
            } else {
                PackageError::Io(error)
            }
        })?;
        let found = Self::digest(&bytes);
        if found != digest {
            return Err(PackageError::CacheDigestMismatch {
                path,
                expected: digest.to_owned(),
                found,
            });
        }
        Ok(bytes)
    }
}

fn validate_digest(digest: &str) -> Result<(), PackageError> {
    let valid = digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid {
        Ok(())
    } else {
        Err(PackageError::InvalidDigest(digest.to_owned()))
    }
}
