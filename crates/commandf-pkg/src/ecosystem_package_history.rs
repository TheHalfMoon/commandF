use semver::Version;
use serde::Serialize;
use thiserror::Error;

use crate::{
    project_snapshot, PackageCache, PackageError, PackageName, SnapshotError, SnapshotPackage,
    IMMUTABLE_RELEASE, PRIMARY_HOST, SECONDARY_HOST,
};

pub const ECOSYSTEM_PACKAGE_HISTORY_SCHEMA: &str = "commandf.ecosystem-package-history/v1";
pub const SUPPLIED_VERSIONS_ONLY: &str = "SUPPLIED_VERSIONS_ONLY";
pub const MAX_PACKAGE_HISTORY_VERSIONS: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageVersionRecord {
    pub version: String,
    pub archive_sha256: String,
    pub source_host: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PackageHistoryNode {
    pub archive_sha256: String,
    pub snapshot_sha256: String,
    pub source_host: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageHistory {
    pub coverage: String,
    pub name: String,
    pub schema: String,
    pub versions: Vec<PackageHistoryNode>,
    pub graph_sha256: String,
}

#[derive(Debug, Error)]
pub enum PackageHistoryError {
    #[error("package history requires at least one exact version")]
    EmptyHistory,

    #[error("package version must be an exact semantic version")]
    InvalidVersion,

    #[error("registry host is not an authorized official source")]
    UnauthorizedHost,

    #[error("duplicate package version {version}")]
    DuplicateVersion { version: String },

    #[error("package history exceeds {MAX_PACKAGE_HISTORY_VERSIONS} versions")]
    TooManyVersions,

    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Package(#[from] PackageError),
}

#[derive(Serialize)]
struct HistoryBody<'a> {
    coverage: &'a str,
    name: &'a str,
    schema: &'a str,
    versions: &'a [PackageHistoryNode],
}

pub fn project_package_history(
    name: &str,
    versions: &[PackageVersionRecord],
) -> Result<PackageHistory, PackageHistoryError> {
    PackageName::parse(name)?;
    if versions.is_empty() {
        return Err(PackageHistoryError::EmptyHistory);
    }
    if versions.len() > MAX_PACKAGE_HISTORY_VERSIONS {
        return Err(PackageHistoryError::TooManyVersions);
    }
    let mut ordered = Vec::with_capacity(versions.len());
    for record in versions {
        let parsed =
            Version::parse(&record.version).map_err(|_| PackageHistoryError::InvalidVersion)?;
        if record.source_host != PRIMARY_HOST && record.source_host != SECONDARY_HOST {
            return Err(PackageHistoryError::UnauthorizedHost);
        }
        ordered.push((parsed, record.clone()));
    }
    ordered.sort_by(|left, right| left.0.cmp(&right.0));
    for pair in ordered.windows(2) {
        if pair[0].1.version == pair[1].1.version {
            return Err(PackageHistoryError::DuplicateVersion {
                version: pair[0].1.version.clone(),
            });
        }
    }
    let mut nodes = Vec::with_capacity(ordered.len());
    for (_, record) in ordered {
        let snapshot = project_snapshot(
            vec![SnapshotPackage {
                name: name.to_owned(),
                version: record.version.clone(),
                archive_sha256: record.archive_sha256.clone(),
                source_id: record.source_host.clone(),
                mutability: IMMUTABLE_RELEASE.to_owned(),
            }],
            Vec::new(),
        )?;
        nodes.push(PackageHistoryNode {
            archive_sha256: record.archive_sha256,
            snapshot_sha256: snapshot.snapshot_sha256,
            source_host: record.source_host,
            version: record.version,
        });
    }
    let body = HistoryBody {
        coverage: SUPPLIED_VERSIONS_ONLY,
        name,
        schema: ECOSYSTEM_PACKAGE_HISTORY_SCHEMA,
        versions: &nodes,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    Ok(PackageHistory {
        coverage: SUPPLIED_VERSIONS_ONLY.to_owned(),
        name: name.to_owned(),
        schema: ECOSYSTEM_PACKAGE_HISTORY_SCHEMA.to_owned(),
        versions: nodes,
        graph_sha256: PackageCache::digest(&bytes),
    })
}

pub fn replay_package_history(
    name: &str,
    versions: &[PackageVersionRecord],
) -> Result<PackageHistory, PackageHistoryError> {
    project_package_history(name, versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: u8) -> String {
        format!("{byte:02x}").repeat(32)
    }

    fn record(version: &str, byte: u8) -> PackageVersionRecord {
        PackageVersionRecord {
            version: version.to_owned(),
            archive_sha256: digest(byte),
            source_host: PRIMARY_HOST.to_owned(),
        }
    }

    #[test]
    fn order_is_semantic_and_stable() {
        let left = project_package_history(
            "hl7.fhir.r4.core",
            &[record("1.10.0", 1), record("1.9.0", 2)],
        )
        .expect("history");
        let right = replay_package_history(
            "hl7.fhir.r4.core",
            &[record("1.9.0", 2), record("1.10.0", 1)],
        )
        .expect("replay");
        assert_eq!(left.versions[0].version, "1.9.0");
        assert_eq!(left.versions[1].version, "1.10.0");
        assert_eq!(left.graph_sha256, right.graph_sha256);
        assert_eq!(left.coverage, SUPPLIED_VERSIONS_ONLY);
        let encoded = serde_json::to_vec(&HistoryBody {
            coverage: &left.coverage,
            name: &left.name,
            schema: &left.schema,
            versions: &left.versions,
        })
        .expect("json");
        let text = String::from_utf8(encoded).expect("utf8");
        assert!(text.contains(SUPPLIED_VERSIONS_ONLY));
        assert!(!text.contains("retrieved_at"));
        assert!(!text.contains("PROVEN_COMPATIBLE"));
        assert!(!text.contains("PROVEN_BREAKING"));
    }

    #[test]
    fn archive_change_changes_the_recomputed_snapshot() {
        let first =
            project_package_history("hl7.fhir.r4.core", &[record("1.0.0", 1)]).expect("first");
        let second =
            project_package_history("hl7.fhir.r4.core", &[record("1.0.0", 2)]).expect("second");
        assert_ne!(
            first.versions[0].snapshot_sha256,
            second.versions[0].snapshot_sha256
        );
        assert_ne!(first.graph_sha256, second.graph_sha256);
    }

    #[test]
    fn closed_failures() {
        let duplicate = project_package_history(
            "hl7.fhir.r4.core",
            &[record("1.0.0", 1), record("1.0.0", 2)],
        )
        .unwrap_err();
        assert!(matches!(
            duplicate,
            PackageHistoryError::DuplicateVersion { .. }
        ));

        let latest = PackageVersionRecord {
            version: "latest".to_owned(),
            archive_sha256: digest(1),
            source_host: PRIMARY_HOST.to_owned(),
        };
        let error = project_package_history("hl7.fhir.r4.core", &[latest]).unwrap_err();
        assert!(matches!(error, PackageHistoryError::InvalidVersion));

        let evil = PackageVersionRecord {
            version: "1.0.0".to_owned(),
            archive_sha256: digest(1),
            source_host: "evil.example".to_owned(),
        };
        let error = project_package_history("hl7.fhir.r4.core", &[evil]).unwrap_err();
        assert!(matches!(error, PackageHistoryError::UnauthorizedHost));

        let many = (1..=257)
            .map(|index| record(&format!("1.0.{index}"), 1))
            .collect::<Vec<_>>();
        let error = project_package_history("hl7.fhir.r4.core", &many).unwrap_err();
        assert!(matches!(error, PackageHistoryError::TooManyVersions));
    }
}
