use commandf_pkg::{Lockfile, PackageError};

#[test]
fn unsupported_lock_schema_fails_closed() {
    let bytes = br#"{"schema":2,"roots":[],"packages":[]}"#;
    let error = Lockfile::from_slice(bytes).unwrap_err();
    assert!(matches!(error, PackageError::UnsupportedLockSchema { .. }));
}
