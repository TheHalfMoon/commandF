use serde::Serialize;
use thiserror::Error;

use crate::{
    verify_comparison_identity, ComparisonError, PackageCache, PackageError, SnapshotComparison,
};

pub const ECOSYSTEM_HISTORY_SCHEMA: &str = "commandf.ecosystem-snapshot-history/v1";
pub const MAX_HISTORY_STEPS: usize = 1_000;
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
    let body = HistoryBody {
        comparison_sha256s: &comparison_sha256s,
        engine_schema,
        first_before_snapshot_sha256: &first_before,
        last_after_snapshot_sha256: &last_after,
        schema: ECOSYSTEM_HISTORY_SCHEMA,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    Ok(SnapshotHistory {
        schema: ECOSYSTEM_HISTORY_SCHEMA.to_owned(),
        engine_schema: engine_schema.to_owned(),
        comparison_sha256s,
        first_before_snapshot_sha256: first_before,
        last_after_snapshot_sha256: last_after,
        history_sha256: PackageCache::digest(&bytes),
    })
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

#[derive(Serialize)]
struct HistoryBody<'a> {
    comparison_sha256s: &'a [String],
    engine_schema: &'a str,
    first_before_snapshot_sha256: &'a str,
    last_after_snapshot_sha256: &'a str,
    schema: &'a str,
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
}
