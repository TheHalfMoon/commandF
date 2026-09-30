use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    require_published_authority, EcosystemSnapshot, PackageCache, PackageError, SnapshotError,
};

pub const ECOSYSTEM_LIFECYCLE_SCHEMA: &str = "commandf.ecosystem-lifecycle/v1";
pub const ECOSYSTEM_LIFECYCLE_BYTES_SCHEMA: &str = "commandf.ecosystem-lifecycle-bytes/v1";
pub const MAX_LIFECYCLE_STRING_BYTES: usize = 8_192;
pub const MAX_LIFECYCLE_MACHINE_BYTES: usize = 16 * 1024 * 1024;
pub const LIFECYCLE_CURRENT: &str = "CURRENT";
pub const LIFECYCLE_STALE: &str = "STALE";
pub const LIFECYCLE_WITHDRAWN: &str = "WITHDRAWN";
pub const MAX_LIFECYCLE_SOURCES: usize = 10_000;

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLifecycle {
    pub source_id: String,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleRecord {
    pub schema: String,
    pub snapshot_sha256: String,
    pub sources: Vec<SourceLifecycle>,
    pub lifecycle_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LifecycleMachineBytes {
    pub serialization_schema: String,
    pub bytes: Vec<u8>,
    pub machine_sha256: String,
}

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Package(#[from] PackageError),

    #[error("source lifecycle count exceeds {MAX_LIFECYCLE_SOURCES}")]
    TooManySources,

    #[error("source identity must be non-empty")]
    EmptySource,

    #[error("source lifecycle state must be CURRENT, STALE, or WITHDRAWN")]
    InvalidLifecycle,

    #[error("duplicate source lifecycle {source_id}")]
    DuplicateSource { source_id: String },

    #[error("snapshot source {source_id} has no lifecycle record")]
    MissingSource { source_id: String },

    #[error("source lifecycle {source_id} is not used by the snapshot")]
    UnreferencedSource { source_id: String },

    #[error("source {source_id} is {state} and cannot be adopted")]
    SourceNotCurrent { source_id: String, state: String },

    #[error("lifecycle schema is not {ECOSYSTEM_LIFECYCLE_SCHEMA}")]
    UnexpectedLifecycleSchema,

    #[error("lifecycle snapshot identity does not match the published snapshot")]
    UnboundLifecycle,

    #[error("lifecycle source set does not match the snapshot")]
    LifecycleSourceMismatch,

    #[error("stored lifecycle digest does not match the lifecycle document")]
    LifecycleDigestMismatch,

    #[error("lifecycle string exceeds {MAX_LIFECYCLE_STRING_BYTES} bytes")]
    LifecycleStringTooLong,

    #[error("lifecycle machine document exceeds {MAX_LIFECYCLE_MACHINE_BYTES} bytes")]
    LifecycleMachineTooLarge,

    #[error("lifecycle bytes are not the canonical encoding")]
    NoncanonicalLifecycle,

    #[error("lifecycle JSON could not be read as the canonical document")]
    MalformedLifecycle,
}

pub fn project_source_lifecycle(
    snapshot: &EcosystemSnapshot,
    sources: Vec<SourceLifecycle>,
) -> Result<LifecycleRecord, LifecycleError> {
    if sources.len() > MAX_LIFECYCLE_SOURCES {
        return Err(LifecycleError::TooManySources);
    }
    require_published_authority(snapshot)?;

    let mut sources = sources;
    for source in &sources {
        if source.source_id.trim() != source.source_id.as_str() || source.source_id.is_empty() {
            return Err(LifecycleError::EmptySource);
        }
        if source.state != LIFECYCLE_CURRENT
            && source.state != LIFECYCLE_STALE
            && source.state != LIFECYCLE_WITHDRAWN
        {
            return Err(LifecycleError::InvalidLifecycle);
        }
    }
    sources.sort_by(|left, right| {
        left.source_id
            .cmp(&right.source_id)
            .then_with(|| left.state.cmp(&right.state))
    });
    for pair in sources.windows(2) {
        if pair[0].source_id == pair[1].source_id {
            return Err(LifecycleError::DuplicateSource {
                source_id: pair[0].source_id.clone(),
            });
        }
    }

    let snapshot_sources: BTreeSet<&str> = snapshot
        .packages
        .iter()
        .map(|package| package.source_id.as_str())
        .collect();
    let recorded_sources: BTreeSet<&str> = sources
        .iter()
        .map(|source| source.source_id.as_str())
        .collect();
    if let Some(source_id) = snapshot_sources.difference(&recorded_sources).next() {
        return Err(LifecycleError::MissingSource {
            source_id: (*source_id).to_owned(),
        });
    }
    if let Some(source_id) = recorded_sources.difference(&snapshot_sources).next() {
        return Err(LifecycleError::UnreferencedSource {
            source_id: (*source_id).to_owned(),
        });
    }

    let body = LifecycleBody {
        schema: ECOSYSTEM_LIFECYCLE_SCHEMA,
        snapshot_sha256: &snapshot.snapshot_sha256,
        sources: &sources,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    Ok(LifecycleRecord {
        schema: ECOSYSTEM_LIFECYCLE_SCHEMA.to_owned(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        sources,
        lifecycle_sha256: PackageCache::digest(&bytes),
    })
}

pub fn verify_lifecycle_record(
    snapshot: &EcosystemSnapshot,
    record: &LifecycleRecord,
) -> Result<(), LifecycleError> {
    if record.schema != ECOSYSTEM_LIFECYCLE_SCHEMA {
        return Err(LifecycleError::UnexpectedLifecycleSchema);
    }
    if record.snapshot_sha256 != snapshot.snapshot_sha256 {
        return Err(LifecycleError::UnboundLifecycle);
    }
    let mut seen_sources = BTreeSet::new();
    for source in &record.sources {
        if !seen_sources.insert(source.source_id.as_str()) {
            return Err(LifecycleError::DuplicateSource {
                source_id: source.source_id.clone(),
            });
        }
    }
    let snapshot_sources: BTreeSet<&str> = snapshot
        .packages
        .iter()
        .map(|package| package.source_id.as_str())
        .collect();
    let recorded_sources: BTreeSet<&str> = record
        .sources
        .iter()
        .map(|source| source.source_id.as_str())
        .collect();
    if recorded_sources != snapshot_sources {
        return Err(LifecycleError::LifecycleSourceMismatch);
    }
    let body = LifecycleBody {
        schema: ECOSYSTEM_LIFECYCLE_SCHEMA,
        snapshot_sha256: &snapshot.snapshot_sha256,
        sources: &record.sources,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    if PackageCache::digest(&bytes) != record.lifecycle_sha256 {
        return Err(LifecycleError::LifecycleDigestMismatch);
    }
    Ok(())
}

pub fn encode_lifecycle_machine(
    snapshot: &EcosystemSnapshot,
    record: &LifecycleRecord,
) -> Result<LifecycleMachineBytes, LifecycleError> {
    if record.sources.len() > MAX_LIFECYCLE_SOURCES {
        return Err(LifecycleError::TooManySources);
    }
    reject_long_lifecycle_strings(record)?;
    require_published_authority(snapshot)?;
    validate_lifecycle_states(record)?;
    require_sorted_sources(record)?;
    verify_lifecycle_record(snapshot, record)?;
    let body = LifecycleBody {
        schema: ECOSYSTEM_LIFECYCLE_SCHEMA,
        snapshot_sha256: &record.snapshot_sha256,
        sources: &record.sources,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    if bytes.len() > MAX_LIFECYCLE_MACHINE_BYTES {
        return Err(LifecycleError::LifecycleMachineTooLarge);
    }
    let machine_sha256 = PackageCache::digest(&bytes);
    if machine_sha256 != record.lifecycle_sha256 {
        return Err(LifecycleError::LifecycleDigestMismatch);
    }
    Ok(LifecycleMachineBytes {
        serialization_schema: ECOSYSTEM_LIFECYCLE_BYTES_SCHEMA.to_owned(),
        bytes,
        machine_sha256,
    })
}

pub fn decode_lifecycle_machine(
    snapshot: &EcosystemSnapshot,
    bytes: &[u8],
) -> Result<LifecycleRecord, LifecycleError> {
    if bytes.len() > MAX_LIFECYCLE_MACHINE_BYTES {
        return Err(LifecycleError::LifecycleMachineTooLarge);
    }
    let document: LifecycleDocument =
        serde_json::from_slice(bytes).map_err(|_| LifecycleError::MalformedLifecycle)?;
    if document.sources.len() > MAX_LIFECYCLE_SOURCES {
        return Err(LifecycleError::TooManySources);
    }
    let canonical = serde_json::to_vec(&document).map_err(PackageError::Json)?;
    if canonical.as_slice() != bytes {
        return Err(LifecycleError::NoncanonicalLifecycle);
    }
    let record = LifecycleRecord {
        schema: document.schema,
        snapshot_sha256: document.snapshot_sha256,
        sources: document.sources,
        lifecycle_sha256: PackageCache::digest(&canonical),
    };
    reject_long_lifecycle_strings(&record)?;
    require_published_authority(snapshot)?;
    validate_lifecycle_states(&record)?;
    require_sorted_sources(&record)?;
    verify_lifecycle_record(snapshot, &record)?;
    Ok(record)
}

fn reject_long_lifecycle_strings(record: &LifecycleRecord) -> Result<(), LifecycleError> {
    if record.snapshot_sha256.len() > MAX_LIFECYCLE_STRING_BYTES {
        return Err(LifecycleError::LifecycleStringTooLong);
    }
    for source in &record.sources {
        if source.source_id.len() > MAX_LIFECYCLE_STRING_BYTES
            || source.state.len() > MAX_LIFECYCLE_STRING_BYTES
        {
            return Err(LifecycleError::LifecycleStringTooLong);
        }
    }
    Ok(())
}

fn validate_lifecycle_states(record: &LifecycleRecord) -> Result<(), LifecycleError> {
    for source in &record.sources {
        if source.source_id.trim() != source.source_id.as_str() || source.source_id.is_empty() {
            return Err(LifecycleError::EmptySource);
        }
        if source.state != LIFECYCLE_CURRENT
            && source.state != LIFECYCLE_STALE
            && source.state != LIFECYCLE_WITHDRAWN
        {
            return Err(LifecycleError::InvalidLifecycle);
        }
    }
    Ok(())
}

fn require_sorted_sources(record: &LifecycleRecord) -> Result<(), LifecycleError> {
    for pair in record.sources.windows(2) {
        if pair[0].source_id >= pair[1].source_id {
            return Err(LifecycleError::NoncanonicalLifecycle);
        }
    }
    Ok(())
}

pub fn require_current_sources(record: &LifecycleRecord) -> Result<(), LifecycleError> {
    for source in &record.sources {
        if source.state != LIFECYCLE_CURRENT {
            return Err(LifecycleError::SourceNotCurrent {
                source_id: source.source_id.clone(),
                state: source.state.clone(),
            });
        }
    }
    Ok(())
}

#[derive(Serialize)]
struct LifecycleBody<'a> {
    schema: &'a str,
    snapshot_sha256: &'a str,
    sources: &'a [SourceLifecycle],
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LifecycleDocument {
    schema: String,
    snapshot_sha256: String,
    sources: Vec<SourceLifecycle>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{project_snapshot, SnapshotPackage, IMMUTABLE_RELEASE, MUTABLE_CI};

    fn package(name: &str, source_id: &str) -> SnapshotPackage {
        SnapshotPackage {
            name: name.to_owned(),
            version: "1.0.0".to_owned(),
            archive_sha256: "ab".repeat(32),
            source_id: source_id.to_owned(),
            mutability: IMMUTABLE_RELEASE.to_owned(),
        }
    }

    fn published() -> EcosystemSnapshot {
        project_snapshot(vec![package("acme.subject", "mirror-a")], Vec::new()).expect("snapshot")
    }

    fn source(source_id: &str, state: &str) -> SourceLifecycle {
        SourceLifecycle {
            source_id: source_id.to_owned(),
            state: state.to_owned(),
        }
    }

    #[test]
    fn lifecycle_is_order_independent_and_current_sources_adopt() {
        let snapshot = project_snapshot(
            vec![package("acme.b", "mirror-b"), package("acme.a", "mirror-a")],
            Vec::new(),
        )
        .expect("snapshot");
        let left = project_source_lifecycle(
            &snapshot,
            vec![
                source("mirror-b", LIFECYCLE_CURRENT),
                source("mirror-a", LIFECYCLE_CURRENT),
            ],
        )
        .expect("left");
        let right = project_source_lifecycle(
            &snapshot,
            vec![
                source("mirror-a", LIFECYCLE_CURRENT),
                source("mirror-b", LIFECYCLE_CURRENT),
            ],
        )
        .expect("right");
        assert_eq!(left, right);
        require_current_sources(&left).expect("current");
    }

    #[test]
    fn stale_and_withdrawn_are_retained_but_not_adopted() {
        let stale =
            project_source_lifecycle(&published(), vec![source("mirror-a", LIFECYCLE_STALE)])
                .expect("stale retained");
        let current =
            project_source_lifecycle(&published(), vec![source("mirror-a", LIFECYCLE_CURRENT)])
                .expect("current");
        assert_ne!(stale.lifecycle_sha256, current.lifecycle_sha256);
        assert!(matches!(
            require_current_sources(&stale),
            Err(LifecycleError::SourceNotCurrent { .. })
        ));
        let withdrawn =
            project_source_lifecycle(&published(), vec![source("mirror-a", LIFECYCLE_WITHDRAWN)])
                .expect("withdrawn retained");
        assert!(matches!(
            require_current_sources(&withdrawn),
            Err(LifecycleError::SourceNotCurrent { .. })
        ));
    }

    #[test]
    fn missing_extra_duplicate_and_mutable_fail_closed() {
        assert!(matches!(
            project_source_lifecycle(&published(), Vec::new()),
            Err(LifecycleError::MissingSource { .. })
        ));
        assert!(matches!(
            project_source_lifecycle(
                &published(),
                vec![
                    source("mirror-a", LIFECYCLE_CURRENT),
                    source("mirror-b", LIFECYCLE_CURRENT),
                ]
            ),
            Err(LifecycleError::UnreferencedSource { .. })
        ));
        assert!(matches!(
            project_source_lifecycle(
                &published(),
                vec![
                    source("mirror-a", LIFECYCLE_CURRENT),
                    source("mirror-a", LIFECYCLE_STALE),
                ]
            ),
            Err(LifecycleError::DuplicateSource { .. })
        ));
        let mut mutable = published();
        mutable.packages[0].mutability = MUTABLE_CI.to_owned();
        assert!(matches!(
            project_source_lifecycle(&mutable, vec![source("mirror-a", LIFECYCLE_CURRENT)]),
            Err(LifecycleError::Snapshot(
                SnapshotError::MutableSourceIsNotPublished { .. }
            ))
        ));
        let too_many = vec![source("mirror-a", LIFECYCLE_CURRENT); MAX_LIFECYCLE_SOURCES + 1];
        assert!(matches!(
            project_source_lifecycle(&published(), too_many),
            Err(LifecycleError::TooManySources)
        ));
        assert!(matches!(
            project_source_lifecycle(&published(), vec![source("mirror-a", "UNKNOWN")]),
            Err(LifecycleError::InvalidLifecycle)
        ));
    }

    #[test]
    fn lifecycle_machine_bytes_replay_and_reject_noncanonical_input() {
        let snapshot = project_snapshot(
            vec![package("acme.b", "mirror-b"), package("acme.a", "mirror-a")],
            Vec::new(),
        )
        .expect("snapshot");
        let left = project_source_lifecycle(
            &snapshot,
            vec![
                source("mirror-b", LIFECYCLE_CURRENT),
                source("mirror-a", LIFECYCLE_STALE),
            ],
        )
        .expect("left");
        let right = project_source_lifecycle(
            &snapshot,
            vec![
                source("mirror-a", LIFECYCLE_STALE),
                source("mirror-b", LIFECYCLE_CURRENT),
            ],
        )
        .expect("right");
        let encoded = encode_lifecycle_machine(&snapshot, &left).expect("encode");
        let encoded_again = encode_lifecycle_machine(&snapshot, &right).expect("again");
        assert_eq!(encoded.bytes, encoded_again.bytes);
        assert_eq!(encoded.machine_sha256, left.lifecycle_sha256);
        assert_eq!(
            encoded.serialization_schema,
            ECOSYSTEM_LIFECYCLE_BYTES_SCHEMA
        );
        let decoded = decode_lifecycle_machine(&snapshot, &encoded.bytes).expect("decode");
        assert_eq!(decoded, left);
        let round = encode_lifecycle_machine(&snapshot, &decoded).expect("round");
        assert_eq!(round.bytes, encoded.bytes);

        let current = project_source_lifecycle(
            &snapshot,
            vec![
                source("mirror-a", LIFECYCLE_CURRENT),
                source("mirror-b", LIFECYCLE_CURRENT),
            ],
        )
        .expect("current");
        assert_ne!(
            encode_lifecycle_machine(&snapshot, &current)
                .expect("current bytes")
                .machine_sha256,
            encoded.machine_sha256
        );

        let mut spaced = encoded.bytes.clone();
        spaced.insert(1, b' ');
        assert!(matches!(
            decode_lifecycle_machine(&snapshot, &spaced),
            Err(LifecycleError::NoncanonicalLifecycle)
        ));
        assert!(matches!(
            decode_lifecycle_machine(
                &snapshot,
                br#"{"schema":"x","snapshot_sha256":"ab","sources":[],"extra":1}"#
            ),
            Err(LifecycleError::MalformedLifecycle)
        ));
        assert!(matches!(
            decode_lifecycle_machine(&snapshot, &vec![0; MAX_LIFECYCLE_MACHINE_BYTES + 1]),
            Err(LifecycleError::LifecycleMachineTooLarge)
        ));

        let mut tampered = left.clone();
        tampered.lifecycle_sha256 = "ab".repeat(32);
        assert!(matches!(
            encode_lifecycle_machine(&snapshot, &tampered),
            Err(LifecycleError::LifecycleDigestMismatch)
        ));
        tampered = left.clone();
        tampered.schema = "commandf.ecosystem-lifecycle/v2".to_owned();
        assert!(matches!(
            encode_lifecycle_machine(&snapshot, &tampered),
            Err(LifecycleError::UnexpectedLifecycleSchema)
        ));
        tampered = left.clone();
        tampered.sources.reverse();
        assert!(matches!(
            encode_lifecycle_machine(&snapshot, &tampered),
            Err(LifecycleError::NoncanonicalLifecycle)
        ));
    }
}
