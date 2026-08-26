use std::fs;
use std::io::{Read, Write};
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
        let bytes = fs::read(&path).map_err(|error| map_cache_read_error(error, digest))?;
        verify_cache_bytes(path, digest, bytes)
    }

    pub fn read_verified_bounded(
        &self,
        digest: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, PackageError> {
        validate_digest(digest)?;
        let path = self.object_path(digest);
        let file = fs::File::open(&path).map_err(|error| map_cache_read_error(error, digest))?;
        let read_limit = max_bytes.saturating_add(1);
        let mut bytes = Vec::new();
        file.take(read_limit).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > max_bytes {
            return Err(PackageError::InvalidRequest(format!(
                "cache object exceeds the maximum supported size of {max_bytes} bytes"
            )));
        }
        verify_cache_bytes(path, digest, bytes)
    }
}

fn map_cache_read_error(error: std::io::Error, digest: &str) -> PackageError {
    if error.kind() == std::io::ErrorKind::NotFound {
        PackageError::CacheMissing(digest.to_owned())
    } else {
        PackageError::Io(error)
    }
}

fn verify_cache_bytes(
    path: PathBuf,
    expected: &str,
    bytes: Vec<u8>,
) -> Result<Vec<u8>, PackageError> {
    let found = PackageCache::digest(&bytes);
    if found != expected {
        return Err(PackageError::CacheDigestMismatch {
            path,
            expected: expected.to_owned(),
            found,
        });
    }
    Ok(bytes)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_verified_read_rejects_oversized_cache_object() {
        let directory = tempfile::tempdir().expect("temp directory");
        let cache = PackageCache::new(directory.path());
        let digest = cache.put(b"abcd").expect("cache object");

        let error = cache
            .read_verified_bounded(&digest, 3)
            .expect_err("oversized cache object must fail closed");
        assert!(matches!(error, PackageError::InvalidRequest(_)));
    }

    #[test]
    fn bounded_verified_read_returns_the_bytes_it_verified() {
        let directory = tempfile::tempdir().expect("temp directory");
        let cache = PackageCache::new(directory.path());
        let digest = cache.put(b"abcd").expect("cache object");

        let bytes = cache
            .read_verified_bounded(&digest, 4)
            .expect("bounded verified bytes");
        assert_eq!(bytes, b"abcd");
    }
}
