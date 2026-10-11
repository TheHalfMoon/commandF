use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;

use crate::{source::MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES, PackageError};

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
        reject_oversize(bytes.len() as u64, MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES)?;
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

        self.publish_new_object(temporary, &path, &digest)?;
        Ok(digest)
    }

    /// Publish only if the digest path has not already been created.
    ///
    /// A racing writer may publish this exact digest first. Never replace it:
    /// verify those already-published bytes and fail closed if they differ.
    fn publish_new_object(
        &self,
        temporary: NamedTempFile,
        path: &Path,
        digest: &str,
    ) -> Result<(), PackageError> {
        match temporary.persist_noclobber(path) {
            Ok(_) => self.verify(digest),
            Err(error) if error.error.kind() == io::ErrorKind::AlreadyExists => self.verify(digest),
            Err(error) => Err(PackageError::Io(error.error)),
        }
    }

    pub fn verify(&self, digest: &str) -> Result<(), PackageError> {
        self.read_verified(digest).map(|_| ())
    }

    pub fn read_verified(&self, digest: &str) -> Result<Vec<u8>, PackageError> {
        self.read_verified_bounded(digest, MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES)
    }

    pub fn read_verified_bounded(
        &self,
        digest: &str,
        max_bytes: u64,
    ) -> Result<Vec<u8>, PackageError> {
        validate_digest(digest)?;
        let path = self.object_path(digest);
        let file = fs::File::open(&path).map_err(|error| map_cache_read_error(error, digest))?;
        // Callers may request a smaller bound, but never bypass the archive
        // acquisition ceiling with a larger one.
        let max_bytes = max_bytes.min(MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES);
        reject_oversize(file.metadata()?.len(), max_bytes)?;
        let mut bytes = Vec::new();
        file.take(max_bytes + 1).read_to_end(&mut bytes)?;
        // The object can grow between metadata() and read_to_end().
        reject_oversize(bytes.len() as u64, max_bytes)?;
        verify_cache_bytes(path, digest, bytes)
    }
}

fn reject_oversize(size: u64, max_bytes: u64) -> Result<(), PackageError> {
    if size > max_bytes {
        return Err(PackageError::InvalidRequest(format!(
            "cache object exceeds the maximum supported size of {max_bytes} bytes"
        )));
    }
    Ok(())
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
    fn archive_ceiling_accepts_exact_size_and_rejects_one_extra_byte() {
        assert!(reject_oversize(
            MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES,
            MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES
        )
        .is_ok());
        let error = reject_oversize(
            MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES + 1,
            MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES,
        )
        .expect_err("limit plus one must fail before allocation");
        assert!(matches!(error, PackageError::InvalidRequest(_)));
    }

    #[test]
    fn verified_read_rejects_sparse_oversized_object_without_loading_it() {
        let directory = tempfile::tempdir().expect("temp directory");
        let cache = PackageCache::new(directory.path());
        let digest = cache.put(b"valid archive").expect("cache object");
        let object = cache.object_path(&digest);
        std::fs::OpenOptions::new()
            .write(true)
            .open(&object)
            .expect("open cache object")
            .set_len(MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES + 1)
            .expect("create sparse oversize cache object");

        let error = cache
            .read_verified(&digest)
            .expect_err("oversize cache file must fail before allocating it");
        assert!(matches!(error, PackageError::InvalidRequest(_)));
        assert!(error
            .to_string()
            .contains("cache object exceeds the maximum supported size"));
    }

    #[test]
    fn bounded_reader_never_exceeds_global_archive_ceiling() {
        let directory = tempfile::tempdir().expect("temp directory");
        let cache = PackageCache::new(directory.path());
        let digest = cache.put(b"valid archive").expect("cache object");
        let object = cache.object_path(&digest);
        std::fs::OpenOptions::new()
            .write(true)
            .open(&object)
            .expect("open cache object")
            .set_len(MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES + 1)
            .expect("create sparse oversize cache object");

        let error = cache
            .read_verified_bounded(&digest, u64::MAX)
            .expect_err("oversized caller limit must not bypass global bound");
        assert!(matches!(error, PackageError::InvalidRequest(_)));
    }

    #[test]
    fn publication_cannot_replace_corrupted_digest_object_from_racing_writer() {
        let directory = tempfile::tempdir().expect("temp directory");
        let cache = PackageCache::new(directory.path());
        let expected = b"approved archive bytes";
        let digest = PackageCache::digest(expected);
        let parent = cache.root().join("sha256");
        fs::create_dir_all(&parent).unwrap();

        let mut temporary = NamedTempFile::new_in(&parent).unwrap();
        temporary.write_all(expected).unwrap();
        temporary.as_file().sync_all().unwrap();

        let object = cache.object_path(&digest);
        // Deterministically simulate a competitor publishing under this path
        // after the caller has prepared its temp file but before publication.
        fs::write(&object, b"untrusted archive bytes").unwrap();

        let error = cache
            .publish_new_object(temporary, &object, &digest)
            .expect_err("must not overwrite or silently repair corruption");
        assert!(matches!(
            error,
            PackageError::CacheDigestMismatch { expected, .. } if expected == digest
        ));
        assert_eq!(
            fs::read(&object).unwrap(),
            b"untrusted archive bytes",
            "the competing object must not be overwritten"
        );
    }

    #[test]
    fn publication_accepts_verified_object_from_racing_writer() {
        let directory = tempfile::tempdir().expect("temp directory");
        let cache = PackageCache::new(directory.path());
        let expected = b"approved archive bytes";
        let digest = PackageCache::digest(expected);
        let parent = cache.root().join("sha256");
        fs::create_dir_all(&parent).unwrap();

        let mut temporary = NamedTempFile::new_in(&parent).unwrap();
        temporary.write_all(expected).unwrap();
        temporary.as_file().sync_all().unwrap();
        let object = cache.object_path(&digest);
        fs::write(&object, expected).unwrap();

        cache
            .publish_new_object(temporary, &object, &digest)
            .expect("racing writer published the exact verified object");
        assert_eq!(cache.read_verified(&digest).unwrap(), expected);
    }

    #[test]
    fn concurrent_writes_to_the_same_digest_reuse_verified_complete_bytes() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        let directory = tempfile::tempdir().expect("temp directory");
        let cache = Arc::new(PackageCache::new(directory.path()));
        let bytes = Arc::new(vec![0x3Au8; 64 * 1024]);
        let count = 12;
        let barrier = Arc::new(Barrier::new(count));
        let handles: Vec<_> = (0..count)
            .map(|_| {
                let cache = Arc::clone(&cache);
                let bytes = Arc::clone(&bytes);
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    cache.put(&bytes).expect("concurrent publish succeeds")
                })
            })
            .collect();

        let expected = PackageCache::digest(&bytes);
        for handle in handles {
            assert_eq!(handle.join().expect("writer thread"), expected);
        }
        assert_eq!(cache.read_verified(&expected).unwrap(), *bytes);
        let objects = fs::read_dir(cache.root().join("sha256"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(objects.len(), 1, "no leaked temp or duplicate objects");
    }

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
