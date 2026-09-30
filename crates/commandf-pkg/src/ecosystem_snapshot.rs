use semver::Version;
use serde::Serialize;
use thiserror::Error;

use crate::{PackageCache, PackageError, PackageName};

pub const ECOSYSTEM_SNAPSHOT_SCHEMA: &str = "commandf.ecosystem-snapshot/v1";
pub const IMMUTABLE_RELEASE: &str = "IMMUTABLE_RELEASE";
pub const MUTABLE_CI: &str = "MUTABLE_CI";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotPackage {
    pub name: String,
    pub version: String,
    pub archive_sha256: String,
    pub source_id: String,
    pub mutability: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EcosystemSnapshot {
    pub schema: String,
    pub packages: Vec<SnapshotPackage>,
    pub unresolved_canonicals: Vec<String>,
    pub snapshot_sha256: String,
}

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error(transparent)]
    Package(#[from] PackageError),

    #[error("duplicate package identity {name}@{version}")]
    DuplicatePackage { name: String, version: String },

    #[error("archive sha256 must be 64 lowercase hex characters")]
    InvalidDigest,

    #[error("source mutability must be IMMUTABLE_RELEASE or MUTABLE_CI")]
    InvalidMutability,

    #[error("source identity must be non-empty")]
    EmptySource,

    #[error("duplicate unresolved canonical {canonical}")]
    DuplicateCanonical { canonical: String },

    #[error("unresolved canonical must be a non-empty token without whitespace")]
    InvalidCanonical,

    #[error("mutable CI source cannot be published authority: {source_id}")]
    MutableSourceIsNotPublished { source_id: String },
}

pub fn project_snapshot(
    packages: Vec<SnapshotPackage>,
    unresolved_canonicals: Vec<String>,
) -> Result<EcosystemSnapshot, SnapshotError> {
    let mut packages = packages;
    for package in &packages {
        PackageName::parse(&package.name)?;
        Version::parse(&package.version).map_err(|error| {
            PackageError::InvalidRequest(format!(
                "snapshot package {} has invalid version {}: {error}",
                package.name, package.version
            ))
        })?;
        if !is_sha256(&package.archive_sha256) {
            return Err(SnapshotError::InvalidDigest);
        }
        if package.source_id.trim() != package.source_id.as_str() || package.source_id.is_empty() {
            return Err(SnapshotError::EmptySource);
        }
        if package.mutability != IMMUTABLE_RELEASE && package.mutability != MUTABLE_CI {
            return Err(SnapshotError::InvalidMutability);
        }
    }
    packages.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
            .then_with(|| left.archive_sha256.cmp(&right.archive_sha256))
            .then_with(|| left.source_id.cmp(&right.source_id))
            .then_with(|| left.mutability.cmp(&right.mutability))
    });
    for pair in packages.windows(2) {
        if pair[0].name == pair[1].name && pair[0].version == pair[1].version {
            return Err(SnapshotError::DuplicatePackage {
                name: pair[0].name.clone(),
                version: pair[0].version.clone(),
            });
        }
    }

    let mut unresolved_canonicals = unresolved_canonicals;
    for canonical in &unresolved_canonicals {
        if canonical.is_empty()
            || canonical.trim() != canonical.as_str()
            || canonical.chars().any(char::is_whitespace)
        {
            return Err(SnapshotError::InvalidCanonical);
        }
    }
    unresolved_canonicals.sort();
    for pair in unresolved_canonicals.windows(2) {
        if pair[0] == pair[1] {
            return Err(SnapshotError::DuplicateCanonical {
                canonical: pair[0].clone(),
            });
        }
    }

    let body = SnapshotBody {
        schema: ECOSYSTEM_SNAPSHOT_SCHEMA,
        packages: &packages,
        unresolved_canonicals: &unresolved_canonicals,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    Ok(EcosystemSnapshot {
        schema: ECOSYSTEM_SNAPSHOT_SCHEMA.to_owned(),
        packages,
        unresolved_canonicals,
        snapshot_sha256: PackageCache::digest(&bytes),
    })
}

pub fn require_published_authority(snapshot: &EcosystemSnapshot) -> Result<(), SnapshotError> {
    for package in &snapshot.packages {
        if package.mutability == MUTABLE_CI {
            return Err(SnapshotError::MutableSourceIsNotPublished {
                source_id: package.source_id.clone(),
            });
        }
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Serialize)]
struct SnapshotBody<'a> {
    schema: &'a str,
    packages: &'a [SnapshotPackage],
    unresolved_canonicals: &'a [String],
}

impl Serialize for SnapshotPackage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SnapshotPackage", 5)?;
        state.serialize_field("archive_sha256", &self.archive_sha256)?;
        state.serialize_field("mutability", &self.mutability)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("source_id", &self.source_id)?;
        state.serialize_field("version", &self.version)?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(version: &str, digest_byte: u8, mutability: &str) -> SnapshotPackage {
        SnapshotPackage {
            name: "acme.subject".to_owned(),
            version: version.to_owned(),
            archive_sha256: format!("{digest_byte:02x}").repeat(32),
            source_id: "local-mirror".to_owned(),
            mutability: mutability.to_owned(),
        }
    }

    #[test]
    fn replay_is_order_independent_and_deterministic() {
        let first = vec![
            package("1.0.0", 0xab, IMMUTABLE_RELEASE),
            package("1.1.0", 0xcd, IMMUTABLE_RELEASE),
        ];
        let second = vec![
            package("1.1.0", 0xcd, IMMUTABLE_RELEASE),
            package("1.0.0", 0xab, IMMUTABLE_RELEASE),
        ];
        let unresolved = vec![
            "http://example.org/StructureDefinition/b".to_owned(),
            "http://example.org/StructureDefinition/a".to_owned(),
        ];
        let left = project_snapshot(first, unresolved.clone()).expect("left snapshot");
        let right = project_snapshot(second, unresolved).expect("right snapshot");
        assert_eq!(left.snapshot_sha256, right.snapshot_sha256);
        assert_eq!(left, right);
        require_published_authority(&left).expect("immutable snapshot");
    }

    #[test]
    fn one_byte_digest_change_changes_snapshot_identity() {
        let baseline =
            project_snapshot(vec![package("1.0.0", 0x11, IMMUTABLE_RELEASE)], Vec::new())
                .expect("baseline");
        let changed = project_snapshot(vec![package("1.0.0", 0x12, IMMUTABLE_RELEASE)], Vec::new())
            .expect("changed");
        assert_ne!(baseline.snapshot_sha256, changed.snapshot_sha256);
    }

    #[test]
    fn mutable_ci_cannot_be_published_authority() {
        let snapshot = project_snapshot(vec![package("1.0.0", 0x11, MUTABLE_CI)], Vec::new())
            .expect("mutable snapshot is retained");
        let error = require_published_authority(&snapshot).expect_err("mutable CI");
        assert!(matches!(
            error,
            SnapshotError::MutableSourceIsNotPublished { .. }
        ));
    }

    #[test]
    fn duplicate_package_and_partial_digest_fail_closed() {
        let duplicate = project_snapshot(
            vec![
                package("1.0.0", 0x11, IMMUTABLE_RELEASE),
                package("1.0.0", 0x22, IMMUTABLE_RELEASE),
            ],
            Vec::new(),
        );
        assert!(matches!(
            duplicate,
            Err(SnapshotError::DuplicatePackage { .. })
        ));
        let mut broken = package("1.0.0", 0x11, IMMUTABLE_RELEASE);
        broken.archive_sha256 = "abcd".to_owned();
        assert!(matches!(
            project_snapshot(vec![broken], Vec::new()),
            Err(SnapshotError::InvalidDigest)
        ));
    }

    #[test]
    fn unresolved_canonicals_are_retained_and_duplicates_fail() {
        let snapshot = project_snapshot(
            Vec::new(),
            vec!["http://example.org/StructureDefinition/missing".to_owned()],
        )
        .expect("unresolved witness");
        assert_eq!(
            snapshot.unresolved_canonicals,
            vec!["http://example.org/StructureDefinition/missing".to_owned()]
        );
        let error = project_snapshot(
            Vec::new(),
            vec![
                "http://example.org/x".to_owned(),
                "http://example.org/x".to_owned(),
            ],
        );
        assert!(matches!(
            error,
            Err(SnapshotError::DuplicateCanonical { .. })
        ));
    }
}
