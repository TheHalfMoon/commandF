use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

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

    pub fn object_path(&self, digest: &str) -> PathBuf {
        self.root.join("sha256").join(format!("{digest}.tgz"))
    }

    pub fn put(&self, bytes: &[u8]) -> Result<String, PackageError> {
        let digest = Self::digest(bytes);
        let path = self.object_path(&digest);
        if path.exists() {
            self.verify(&digest)?;
            return Ok(digest);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, bytes)?;
        self.verify(&digest)?;
        Ok(digest)
    }

    pub fn verify(&self, digest: &str) -> Result<(), PackageError> {
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
        Ok(())
    }
}
