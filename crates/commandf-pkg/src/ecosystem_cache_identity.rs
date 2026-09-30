use serde::Serialize;
use thiserror::Error;

use crate::{
    require_published_authority, EcosystemSnapshot, PackageCache, PackageError, SnapshotError,
};

pub const ECOSYSTEM_CACHE_IDENTITY_SCHEMA: &str = "commandf.ecosystem-cache-identity/v1";
pub const MAX_ENGINE_SCHEMA_CHARS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheIdentity {
    pub schema: String,
    pub snapshot_sha256: String,
    pub engine_schema: String,
    pub cache_sha256: String,
}

#[derive(Debug, Error)]
pub enum CacheIdentityError {
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Package(#[from] PackageError),

    #[error("engine schema must be a non-empty token without whitespace")]
    InvalidEngineSchema,

    #[error("engine schema exceeds {MAX_ENGINE_SCHEMA_CHARS} characters")]
    EngineSchemaTooLong,

    #[error("cache identity does not match the snapshot and engine schema")]
    CacheInvalid,
}

pub fn project_cache_identity(
    snapshot: &EcosystemSnapshot,
    engine_schema: &str,
) -> Result<CacheIdentity, CacheIdentityError> {
    require_published_authority(snapshot)?;
    validate_engine_schema(engine_schema)?;
    let body = CacheIdentityBody {
        engine_schema,
        schema: ECOSYSTEM_CACHE_IDENTITY_SCHEMA,
        snapshot_sha256: &snapshot.snapshot_sha256,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    Ok(CacheIdentity {
        schema: ECOSYSTEM_CACHE_IDENTITY_SCHEMA.to_owned(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        engine_schema: engine_schema.to_owned(),
        cache_sha256: PackageCache::digest(&bytes),
    })
}

pub fn require_cache_reuse(
    snapshot: &EcosystemSnapshot,
    engine_schema: &str,
    cached: &CacheIdentity,
) -> Result<(), CacheIdentityError> {
    let projected = project_cache_identity(snapshot, engine_schema)?;
    if &projected != cached {
        return Err(CacheIdentityError::CacheInvalid);
    }
    Ok(())
}

fn validate_engine_schema(value: &str) -> Result<(), CacheIdentityError> {
    if value.chars().count() > MAX_ENGINE_SCHEMA_CHARS {
        return Err(CacheIdentityError::EngineSchemaTooLong);
    }
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_whitespace) {
        return Err(CacheIdentityError::InvalidEngineSchema);
    }
    Ok(())
}

#[derive(Serialize)]
struct CacheIdentityBody<'a> {
    engine_schema: &'a str,
    schema: &'a str,
    snapshot_sha256: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{project_snapshot, SnapshotPackage, IMMUTABLE_RELEASE, MUTABLE_CI};

    fn snapshot(digest_byte: u8) -> EcosystemSnapshot {
        project_snapshot(
            vec![SnapshotPackage {
                name: "acme.subject".to_owned(),
                version: "1.0.0".to_owned(),
                archive_sha256: format!("{digest_byte:02x}").repeat(32),
                source_id: "local-mirror".to_owned(),
                mutability: IMMUTABLE_RELEASE.to_owned(),
            }],
            Vec::new(),
        )
        .expect("snapshot")
    }

    #[test]
    fn cache_identity_tracks_snapshot_and_engine_schema() {
        let published = snapshot(0x11);
        let left = project_cache_identity(&published, "commandf.closure/v1").expect("left");
        let right = project_cache_identity(&published, "commandf.closure/v1").expect("right");
        assert_eq!(left, right);
        require_cache_reuse(&published, "commandf.closure/v1", &left).expect("reuse");

        let other_engine =
            project_cache_identity(&published, "commandf.closure/v2").expect("engine");
        assert_ne!(left.cache_sha256, other_engine.cache_sha256);
        assert!(matches!(
            require_cache_reuse(&published, "commandf.closure/v2", &left),
            Err(CacheIdentityError::CacheInvalid)
        ));

        let other_snapshot = project_cache_identity(&snapshot(0x12), "commandf.closure/v1")
            .expect("snapshot change");
        assert_ne!(left.cache_sha256, other_snapshot.cache_sha256);
    }

    #[test]
    fn tampered_digest_mutable_and_bounds_fail_closed() {
        let published = snapshot(0x11);
        let mut cached = project_cache_identity(&published, "commandf.closure/v1").expect("cache");
        cached.cache_sha256 = "cd".repeat(32);
        assert!(matches!(
            require_cache_reuse(&published, "commandf.closure/v1", &cached),
            Err(CacheIdentityError::CacheInvalid)
        ));

        let mut mutable = published.clone();
        mutable.packages[0].mutability = MUTABLE_CI.to_owned();
        assert!(matches!(
            project_cache_identity(&mutable, "commandf.closure/v1"),
            Err(CacheIdentityError::Snapshot(
                SnapshotError::MutableSourceIsNotPublished { .. }
            ))
        ));
        assert!(matches!(
            project_cache_identity(&published, " "),
            Err(CacheIdentityError::InvalidEngineSchema)
        ));
        let long = "a".repeat(MAX_ENGINE_SCHEMA_CHARS + 1);
        assert!(matches!(
            project_cache_identity(&published, &long),
            Err(CacheIdentityError::EngineSchemaTooLong)
        ));
    }
}
