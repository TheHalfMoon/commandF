use commandf_pkg::{PackageCache, PackageError, PackageRequest};
use tempfile::tempdir;

#[test]
fn cache_corruption_is_detected() {
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());
    let digest = cache.put(b"original").unwrap();
    let object = cache.root().join("sha256").join(format!("{digest}.tgz"));
    std::fs::write(object, b"corrupt").unwrap();

    let error = cache.verify(&digest).unwrap_err();
    assert!(matches!(error, PackageError::CacheDigestMismatch { .. }));
}

#[test]
fn invalid_digest_is_rejected_before_cache_access() {
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());
    let error = cache.verify("not-a-sha256-digest").unwrap_err();
    assert!(matches!(error, PackageError::InvalidDigest(_)));
}

#[test]
fn unsupported_broad_constraint_is_rejected() {
    let error = PackageRequest::parse("acme.root@^1.0.0").unwrap_err();
    assert!(matches!(error, PackageError::UnsupportedConstraint { .. }));
}
