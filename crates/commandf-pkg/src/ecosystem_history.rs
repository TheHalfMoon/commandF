use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    verify_comparison_identity, ComparisonError, PackageCache, PackageError, SnapshotComparison,
};

pub const ECOSYSTEM_HISTORY_SCHEMA: &str = "commandf.ecosystem-snapshot-history/v1";
pub const ECOSYSTEM_HISTORY_BYTES_SCHEMA: &str = "commandf.ecosystem-history-bytes/v1";
pub const MAX_HISTORY_STEPS: usize = 1_000;
pub const MAX_HISTORY_MACHINE_BYTES: usize = 1024 * 1024;
const MAX_ENGINE_CHARS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotHistory {
    pub schema: String,
    pub engine_schema: String,
    pub comparison_sha256s: Vec<String>,
    pub first_before_snapshot_sha256: String,
    pub last_after_snapshot_sha256: String,
    pub history_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoryMachineBytes {
    pub serialization_schema: String,
    pub bytes: Vec<u8>,
    pub machine_sha256: String,
}

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error(transparent)]
    Comparison(#[from] ComparisonError),

    #[error(transparent)]
    Package(#[from] PackageError),

    #[error("history engine schema must be a non-empty token without whitespace")]
    InvalidEngineSchema,

    #[error("history engine schema exceeds {MAX_ENGINE_CHARS} characters")]
    EngineSchemaTooLong,

    #[error("history step count exceeds {MAX_HISTORY_STEPS}")]
    TooManySteps,

    #[error("comparison step {step} does not continue the previous snapshot identity")]
    BrokenChain { step: usize },

    #[error("history machine document exceeds {MAX_HISTORY_MACHINE_BYTES} bytes")]
    MachineTooLarge,

    #[error("history schema is not {ECOSYSTEM_HISTORY_SCHEMA}")]
    UnsupportedHistorySchema,

    #[error("history digest or snapshot identity is not 64 lowercase hex characters")]
    InvalidIdentity,

    #[error("stored history digest does not match the history document")]
    HistoryDigestMismatch,

    #[error("history bytes are not the canonical encoding")]
    NoncanonicalEncoding,

    #[error("duplicate comparison identity {digest}")]
    DuplicateComparison { digest: String },

    #[error("empty history must not name endpoint snapshots")]
    EndpointMismatch,

    #[error("history JSON could not be read as the canonical document")]
    MalformedHistory,
}

pub fn project_snapshot_history(
    engine_schema: &str,
    comparisons: &[SnapshotComparison],
) -> Result<SnapshotHistory, HistoryError> {
    if comparisons.len() > MAX_HISTORY_STEPS {
        return Err(HistoryError::TooManySteps);
    }
    validate_engine(engine_schema)?;
    for comparison in comparisons {
        verify_comparison_identity(comparison)?;
    }
    for (index, pair) in comparisons.windows(2).enumerate() {
        if pair[0].after_snapshot_sha256 != pair[1].before_snapshot_sha256 {
            return Err(HistoryError::BrokenChain { step: index + 1 });
        }
    }
    let comparison_sha256s: Vec<String> = comparisons
        .iter()
        .map(|comparison| comparison.comparison_sha256.clone())
        .collect();
    let first_before = comparisons
        .first()
        .map(|comparison| comparison.before_snapshot_sha256.clone())
        .unwrap_or_default();
    let last_after = comparisons
        .last()
        .map(|comparison| comparison.after_snapshot_sha256.clone())
        .unwrap_or_default();
    let document = HistoryDocument {
        comparison_sha256s: comparison_sha256s.clone(),
        engine_schema: engine_schema.to_owned(),
        first_before_snapshot_sha256: first_before.clone(),
        last_after_snapshot_sha256: last_after.clone(),
        schema: ECOSYSTEM_HISTORY_SCHEMA.to_owned(),
    };
    let bytes = serde_json::to_vec(&document).map_err(PackageError::Json)?;
    Ok(SnapshotHistory {
        schema: ECOSYSTEM_HISTORY_SCHEMA.to_owned(),
        engine_schema: engine_schema.to_owned(),
        comparison_sha256s,
        first_before_snapshot_sha256: first_before,
        last_after_snapshot_sha256: last_after,
        history_sha256: PackageCache::digest(&bytes),
    })
}

pub fn encode_history_machine(
    history: &SnapshotHistory,
) -> Result<HistoryMachineBytes, HistoryError> {
    if history.comparison_sha256s.len() > MAX_HISTORY_STEPS {
        return Err(HistoryError::TooManySteps);
    }
    validate_history_shape(history)?;
    let document = HistoryDocument::from_history(history);
    let bytes = serde_json::to_vec(&document).map_err(PackageError::Json)?;
    if bytes.len() > MAX_HISTORY_MACHINE_BYTES {
        return Err(HistoryError::MachineTooLarge);
    }
    let machine_sha256 = PackageCache::digest(&bytes);
    if machine_sha256 != history.history_sha256 {
        return Err(HistoryError::HistoryDigestMismatch);
    }
    Ok(HistoryMachineBytes {
        serialization_schema: ECOSYSTEM_HISTORY_BYTES_SCHEMA.to_owned(),
        bytes,
        machine_sha256,
    })
}

pub fn decode_history_machine(bytes: &[u8]) -> Result<SnapshotHistory, HistoryError> {
    if bytes.len() > MAX_HISTORY_MACHINE_BYTES {
        return Err(HistoryError::MachineTooLarge);
    }
    let document: HistoryDocument =
        serde_json::from_slice(bytes).map_err(|_| HistoryError::MalformedHistory)?;
    let canonical = serde_json::to_vec(&document).map_err(PackageError::Json)?;
    if canonical.as_slice() != bytes {
        return Err(HistoryError::NoncanonicalEncoding);
    }
    if document.comparison_sha256s.len() > MAX_HISTORY_STEPS
        || canonical.len() > MAX_HISTORY_MACHINE_BYTES
    {
        return Err(HistoryError::TooManySteps);
    }
    let history = SnapshotHistory {
        schema: document.schema,
        engine_schema: document.engine_schema,
        comparison_sha256s: document.comparison_sha256s,
        first_before_snapshot_sha256: document.first_before_snapshot_sha256,
        last_after_snapshot_sha256: document.last_after_snapshot_sha256,
        history_sha256: PackageCache::digest(&canonical),
    };
    validate_history_shape(&history)?;
    Ok(history)
}

fn validate_engine(value: &str) -> Result<(), HistoryError> {
    if value.chars().count() > MAX_ENGINE_CHARS {
        return Err(HistoryError::EngineSchemaTooLong);
    }
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_whitespace) {
        return Err(HistoryError::InvalidEngineSchema);
    }
    Ok(())
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct HistoryDocument {
    comparison_sha256s: Vec<String>,
    engine_schema: String,
    first_before_snapshot_sha256: String,
    last_after_snapshot_sha256: String,
    schema: String,
}

impl HistoryDocument {
    fn from_history(history: &SnapshotHistory) -> Self {
        Self {
            comparison_sha256s: history.comparison_sha256s.clone(),
            engine_schema: history.engine_schema.clone(),
            first_before_snapshot_sha256: history.first_before_snapshot_sha256.clone(),
            last_after_snapshot_sha256: history.last_after_snapshot_sha256.clone(),
            schema: history.schema.clone(),
        }
    }
}

fn validate_history_shape(history: &SnapshotHistory) -> Result<(), HistoryError> {
    if history.schema != ECOSYSTEM_HISTORY_SCHEMA {
        return Err(HistoryError::UnsupportedHistorySchema);
    }
    validate_engine(&history.engine_schema)?;
    let mut seen = std::collections::BTreeSet::new();
    for digest in &history.comparison_sha256s {
        if !is_sha256(digest) {
            return Err(HistoryError::InvalidIdentity);
        }
        if !seen.insert(digest.as_str()) {
            return Err(HistoryError::DuplicateComparison {
                digest: digest.clone(),
            });
        }
    }
    let empty = history.comparison_sha256s.is_empty();
    let first_ok = if empty {
        history.first_before_snapshot_sha256.is_empty()
    } else {
        is_sha256(&history.first_before_snapshot_sha256)
    };
    let last_ok = if empty {
        history.last_after_snapshot_sha256.is_empty()
    } else {
        is_sha256(&history.last_after_snapshot_sha256)
    };
    if !first_ok || !last_ok {
        return Err(HistoryError::EndpointMismatch);
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        project_snapshot, project_snapshot_comparison, ComparisonWitnesses, SnapshotPackage,
        IMMUTABLE_RELEASE,
    };

    fn package(name: &str, digest_byte: u8) -> SnapshotPackage {
        SnapshotPackage {
            name: name.to_owned(),
            version: "1.0.0".to_owned(),
            archive_sha256: format!("{digest_byte:02x}").repeat(32),
            source_id: "local-mirror".to_owned(),
            mutability: IMMUTABLE_RELEASE.to_owned(),
        }
    }

    fn snap(name: &str, digest_byte: u8) -> crate::EcosystemSnapshot {
        project_snapshot(vec![package(name, digest_byte)], Vec::new()).expect("snapshot")
    }

    fn step(
        before: &crate::EcosystemSnapshot,
        after: &crate::EcosystemSnapshot,
    ) -> SnapshotComparison {
        project_snapshot_comparison(
            before,
            after,
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
        )
        .expect("comparison")
    }

    #[test]
    fn history_replays_and_order_changes_identity() {
        let first = snap("acme.a", 0x11);
        let second = snap("acme.b", 0x22);
        let third = snap("acme.c", 0x33);
        let forward = vec![step(&first, &second), step(&second, &third)];
        let left =
            project_snapshot_history("commandf.snapshot-history/v1", &forward).expect("left");
        let right =
            project_snapshot_history("commandf.snapshot-history/v1", &forward).expect("right");
        assert_eq!(left, right);
        assert_eq!(left.first_before_snapshot_sha256, first.snapshot_sha256);
        assert_eq!(left.last_after_snapshot_sha256, third.snapshot_sha256);

        let reverse = vec![step(&third, &second), step(&second, &first)];
        let reversed =
            project_snapshot_history("commandf.snapshot-history/v1", &reverse).expect("reverse");
        assert_ne!(left.history_sha256, reversed.history_sha256);
        let other_engine =
            project_snapshot_history("commandf.snapshot-history/v2", &forward).expect("engine");
        assert_ne!(left.history_sha256, other_engine.history_sha256);
    }

    #[test]
    fn broken_chain_tamper_and_bounds_fail_closed() {
        let first = snap("acme.a", 0x11);
        let second = snap("acme.b", 0x22);
        let repeated = vec![step(&first, &second), step(&first, &second)];
        assert!(matches!(
            project_snapshot_history("commandf.snapshot-history/v1", &repeated),
            Err(HistoryError::BrokenChain { step: 1 })
        ));
        let mut tampered = step(&first, &second);
        tampered.comparison_sha256 = "ab".repeat(32);
        assert!(matches!(
            project_snapshot_history("commandf.snapshot-history/v1", &[tampered]),
            Err(HistoryError::Comparison(
                ComparisonError::ComparisonMismatch
            ))
        ));
        let huge = vec![step(&first, &second); MAX_HISTORY_STEPS + 1];
        assert!(matches!(
            project_snapshot_history("commandf.snapshot-history/v1", &huge),
            Err(HistoryError::TooManySteps)
        ));
        let empty = project_snapshot_history("commandf.snapshot-history/v1", &[]).expect("empty");
        assert!(empty.comparison_sha256s.is_empty());
        assert!(empty.first_before_snapshot_sha256.is_empty());
    }

    #[test]
    fn history_machine_bytes_replay_and_reject_noncanonical_input() {
        let first = snap("acme.a", 0x11);
        let second = snap("acme.b", 0x22);
        let third = snap("acme.c", 0x33);
        let history = project_snapshot_history(
            "commandf.snapshot-history/v1",
            &[step(&first, &second), step(&second, &third)],
        )
        .expect("history");
        let left = encode_history_machine(&history).expect("left");
        let right = encode_history_machine(&history).expect("right");
        assert_eq!(left.bytes, right.bytes);
        assert_eq!(left.machine_sha256, right.machine_sha256);
        assert_eq!(left.machine_sha256, history.history_sha256);
        assert_eq!(left.serialization_schema, ECOSYSTEM_HISTORY_BYTES_SCHEMA);
        let decoded = decode_history_machine(&left.bytes).expect("decode");
        assert_eq!(decoded, history);
        let again = encode_history_machine(&decoded).expect("again");
        assert_eq!(again.bytes, left.bytes);

        let other = project_snapshot_history(
            "commandf.snapshot-history/v1",
            &[step(&third, &second), step(&second, &first)],
        )
        .expect("other");
        let other_bytes = encode_history_machine(&other).expect("other bytes");
        assert_ne!(left.machine_sha256, other_bytes.machine_sha256);

        let mut spaced = left.bytes.clone();
        spaced.insert(1, b' ');
        assert!(matches!(
            decode_history_machine(&spaced),
            Err(HistoryError::NoncanonicalEncoding)
        ));
        assert!(matches!(
            decode_history_machine(b"{"),
            Err(HistoryError::MalformedHistory)
        ));
        assert!(matches!(
            decode_history_machine(&vec![0; MAX_HISTORY_MACHINE_BYTES + 1]),
            Err(HistoryError::MachineTooLarge)
        ));

        let mut tampered = history.clone();
        tampered.history_sha256 = "ab".repeat(32);
        assert!(matches!(
            encode_history_machine(&tampered),
            Err(HistoryError::HistoryDigestMismatch)
        ));
        tampered = history.clone();
        tampered.schema = "commandf.ecosystem-snapshot-history/v2".to_owned();
        assert!(matches!(
            encode_history_machine(&tampered),
            Err(HistoryError::UnsupportedHistorySchema)
        ));
        tampered = history.clone();
        tampered
            .comparison_sha256s
            .push(tampered.comparison_sha256s[0].clone());
        assert!(matches!(
            encode_history_machine(&tampered),
            Err(HistoryError::DuplicateComparison { .. })
        ));
        assert!(matches!(
            project_snapshot_history(
                "commandf.snapshot-history/v1",
                &[step(&first, &second), step(&first, &second)]
            ),
            Err(HistoryError::BrokenChain { step: 1 })
        ));
    }
}
