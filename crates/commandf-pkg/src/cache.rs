use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use crate::PackageError;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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

    fn temp_path(&self, digest: &str) -> PathBuf {
        let sequence = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        self.root
            .join("sha256")
            .join(format!(".{digest}.{}.{}.tmp", std::process::id(), sequence))
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

        let temp_path = self.temp_path(&digest);
        fs::write(&temp_path, bytes)?;

        if let Err(error) = fs::rename(&temp_path, &path) {
            if path.exists() {
                self.verify(&digest)?;
                return Ok(digest);
            }
            return Err(PackageError::Io(error));
        }

        self.verify(&digest)?;
        Ok(digest)
    }

    pub fn verify(&self, digest: &str) -> Result<(), PackageError> {
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
        Ok(())
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
