use commandf_pkg::{PackageCache, PackageError, PackageRequest};
use tempfile::tempdir;

#[test]
fn cache_corruption_is_detected() {
    let dir = tempdir().unwrap();
    let cache = PackageCache::new(dir.path());
    let digest = cache.put(b"original").unwrap();
    std::fs::write(cache.object_path(&digest), b"corrupt").unwrap();

    let error = cache.verify(&digest).unwrap_err();
    assert!(matches!(error, PackageError::CacheDigestMismatch { .. }));
}

#[test]
fn unsupported_broad_constraint_is_rejected() {
    let error = PackageRequest::parse("acme.root@^1.0.0").unwrap_err();
    assert!(matches!(error, PackageError::UnsupportedConstraint { .. }));
}
