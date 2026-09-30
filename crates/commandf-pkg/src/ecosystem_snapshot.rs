use std::collections::BTreeSet;

use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{PackageCache, PackageError, PackageName};

pub const ECOSYSTEM_SNAPSHOT_SCHEMA: &str = "commandf.ecosystem-snapshot/v1";
pub const ECOSYSTEM_SNAPSHOT_BYTES_SCHEMA: &str = "commandf.ecosystem-snapshot-bytes/v1";
pub const MAX_SNAPSHOT_RECORDS: usize = 10_000;
pub const MAX_SNAPSHOT_STRING_BYTES: usize = 8_192;
pub const MAX_SNAPSHOT_MACHINE_BYTES: usize = 16 * 1024 * 1024;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotMachineBytes {
    pub serialization_schema: String,
    pub bytes: Vec<u8>,
    pub machine_sha256: String,
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

    #[error("snapshot schema is not {ECOSYSTEM_SNAPSHOT_SCHEMA}")]
    UnexpectedSnapshotSchema,

    #[error("stored snapshot digest does not match the snapshot document")]
    SnapshotDigestMismatch,

    #[error("snapshot record count exceeds {MAX_SNAPSHOT_RECORDS}")]
    TooManySnapshotRecords,

    #[error("snapshot string exceeds {MAX_SNAPSHOT_STRING_BYTES} bytes")]
    SnapshotStringTooLong,

    #[error("snapshot machine document exceeds {MAX_SNAPSHOT_MACHINE_BYTES} bytes")]
    SnapshotMachineTooLarge,

    #[error("snapshot bytes are not the canonical encoding")]
    NoncanonicalSnapshot,

    #[error("snapshot JSON could not be read as the canonical document")]
    MalformedSnapshot,
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

pub fn verify_snapshot_identity(snapshot: &EcosystemSnapshot) -> Result<(), SnapshotError> {
    if snapshot.schema != ECOSYSTEM_SNAPSHOT_SCHEMA {
        return Err(SnapshotError::UnexpectedSnapshotSchema);
    }
    let mut package_ids = BTreeSet::new();
    for package in &snapshot.packages {
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
        if !package_ids.insert((package.name.as_str(), package.version.as_str())) {
            return Err(SnapshotError::DuplicatePackage {
                name: package.name.clone(),
                version: package.version.clone(),
            });
        }
    }
    let mut canonicals = BTreeSet::new();
    for canonical in &snapshot.unresolved_canonicals {
        if canonical.is_empty()
            || canonical.trim() != canonical.as_str()
            || canonical.chars().any(char::is_whitespace)
        {
            return Err(SnapshotError::InvalidCanonical);
        }
        if !canonicals.insert(canonical.as_str()) {
            return Err(SnapshotError::DuplicateCanonical {
                canonical: canonical.clone(),
            });
        }
    }
    let body = SnapshotBody {
        schema: ECOSYSTEM_SNAPSHOT_SCHEMA,
        packages: &snapshot.packages,
        unresolved_canonicals: &snapshot.unresolved_canonicals,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    if PackageCache::digest(&bytes) != snapshot.snapshot_sha256 {
        return Err(SnapshotError::SnapshotDigestMismatch);
    }
    Ok(())
}

pub fn encode_snapshot_machine(
    snapshot: &EcosystemSnapshot,
) -> Result<SnapshotMachineBytes, SnapshotError> {
    if snapshot.packages.len() > MAX_SNAPSHOT_RECORDS
        || snapshot.unresolved_canonicals.len() > MAX_SNAPSHOT_RECORDS
    {
        return Err(SnapshotError::TooManySnapshotRecords);
    }
    reject_long_snapshot_strings(snapshot)?;
    verify_snapshot_identity(snapshot)?;
    require_published_authority(snapshot)?;
    let body = SnapshotBody {
        schema: ECOSYSTEM_SNAPSHOT_SCHEMA,
        packages: &snapshot.packages,
        unresolved_canonicals: &snapshot.unresolved_canonicals,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    if bytes.len() > MAX_SNAPSHOT_MACHINE_BYTES {
        return Err(SnapshotError::SnapshotMachineTooLarge);
    }
    let machine_sha256 = PackageCache::digest(&bytes);
    if machine_sha256 != snapshot.snapshot_sha256 {
        return Err(SnapshotError::SnapshotDigestMismatch);
    }
    Ok(SnapshotMachineBytes {
        serialization_schema: ECOSYSTEM_SNAPSHOT_BYTES_SCHEMA.to_owned(),
        bytes,
        machine_sha256,
    })
}

pub fn decode_snapshot_machine(bytes: &[u8]) -> Result<EcosystemSnapshot, SnapshotError> {
    if bytes.len() > MAX_SNAPSHOT_MACHINE_BYTES {
        return Err(SnapshotError::SnapshotMachineTooLarge);
    }
    let document: SnapshotDocument =
        serde_json::from_slice(bytes).map_err(|_| SnapshotError::MalformedSnapshot)?;
    if document.packages.len() > MAX_SNAPSHOT_RECORDS
        || document.unresolved_canonicals.len() > MAX_SNAPSHOT_RECORDS
    {
        return Err(SnapshotError::TooManySnapshotRecords);
    }
    let canonical = serde_json::to_vec(&document).map_err(PackageError::Json)?;
    if canonical.as_slice() != bytes {
        return Err(SnapshotError::NoncanonicalSnapshot);
    }
    let snapshot = EcosystemSnapshot {
        schema: document.schema,
        packages: document
            .packages
            .into_iter()
            .map(|package| SnapshotPackage {
                name: package.name,
                version: package.version,
                archive_sha256: package.archive_sha256,
                source_id: package.source_id,
                mutability: package.mutability,
            })
            .collect(),
        unresolved_canonicals: document.unresolved_canonicals,
        snapshot_sha256: PackageCache::digest(&canonical),
    };
    reject_long_snapshot_strings(&snapshot)?;
    verify_snapshot_identity(&snapshot)?;
    require_published_authority(&snapshot)?;
    Ok(snapshot)
}

fn reject_long_snapshot_strings(snapshot: &EcosystemSnapshot) -> Result<(), SnapshotError> {
    for package in &snapshot.packages {
        for value in [
            package.name.as_str(),
            package.version.as_str(),
            package.archive_sha256.as_str(),
            package.source_id.as_str(),
            package.mutability.as_str(),
        ] {
            if value.len() > MAX_SNAPSHOT_STRING_BYTES {
                return Err(SnapshotError::SnapshotStringTooLong);
            }
        }
    }
    for canonical in &snapshot.unresolved_canonicals {
        if canonical.len() > MAX_SNAPSHOT_STRING_BYTES {
            return Err(SnapshotError::SnapshotStringTooLong);
        }
    }
    Ok(())
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

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SnapshotDocument {
    schema: String,
    packages: Vec<SnapshotPackageDocument>,
    unresolved_canonicals: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SnapshotPackageDocument {
    archive_sha256: String,
    mutability: String,
    name: String,
    source_id: String,
    version: String,
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

    #[test]
    fn snapshot_machine_bytes_replay_and_reject_noncanonical_input() {
        let published = project_snapshot(
            vec![
                package("1.1.0", 0x22, IMMUTABLE_RELEASE),
                package("1.0.0", 0x11, IMMUTABLE_RELEASE),
            ],
            vec![
                "http://example.org/b".to_owned(),
                "http://example.org/a".to_owned(),
            ],
        )
        .expect("published");
        let reordered = project_snapshot(
            vec![
                package("1.0.0", 0x11, IMMUTABLE_RELEASE),
                package("1.1.0", 0x22, IMMUTABLE_RELEASE),
            ],
            vec![
                "http://example.org/a".to_owned(),
                "http://example.org/b".to_owned(),
            ],
        )
        .expect("reordered");
        let left = encode_snapshot_machine(&published).expect("left");
        let right = encode_snapshot_machine(&reordered).expect("right");
        assert_eq!(left.bytes, right.bytes);
        assert_eq!(left.machine_sha256, published.snapshot_sha256);
        assert_eq!(left.serialization_schema, ECOSYSTEM_SNAPSHOT_BYTES_SCHEMA);
        let decoded = decode_snapshot_machine(&left.bytes).expect("decode");
        assert_eq!(decoded, published);
        let again = encode_snapshot_machine(&decoded).expect("again");
        assert_eq!(again.bytes, left.bytes);

        let changed = project_snapshot(
            vec![package("1.0.0", 0x12, IMMUTABLE_RELEASE)],
            vec!["http://example.org/a".to_owned()],
        )
        .expect("changed");
        let changed_bytes = encode_snapshot_machine(&changed).expect("changed bytes");
        assert_ne!(left.machine_sha256, changed_bytes.machine_sha256);

        let mut spaced = left.bytes.clone();
        spaced.insert(1, b' ');
        assert!(matches!(
            decode_snapshot_machine(&spaced),
            Err(SnapshotError::NoncanonicalSnapshot)
        ));
        assert!(matches!(
            decode_snapshot_machine(
                br#"{"schema":"x","packages":[],"unresolved_canonicals":[],"extra":1}"#
            ),
            Err(SnapshotError::MalformedSnapshot)
        ));
        assert!(matches!(
            decode_snapshot_machine(&vec![0; MAX_SNAPSHOT_MACHINE_BYTES + 1]),
            Err(SnapshotError::SnapshotMachineTooLarge)
        ));

        let mut tampered = published.clone();
        tampered.snapshot_sha256 = "ab".repeat(32);
        assert!(matches!(
            encode_snapshot_machine(&tampered),
            Err(SnapshotError::SnapshotDigestMismatch)
        ));
        tampered = published.clone();
        tampered.schema = "commandf.ecosystem-snapshot/v2".to_owned();
        assert!(matches!(
            encode_snapshot_machine(&tampered),
            Err(SnapshotError::UnexpectedSnapshotSchema)
        ));
        tampered = published.clone();
        tampered.packages.push(tampered.packages[0].clone());
        assert!(matches!(
            encode_snapshot_machine(&tampered),
            Err(SnapshotError::DuplicatePackage { .. })
        ));
        let mutable = project_snapshot(vec![package("1.0.0", 0x11, MUTABLE_CI)], Vec::new())
            .expect("mutable");
        assert!(matches!(
            encode_snapshot_machine(&mutable),
            Err(SnapshotError::MutableSourceIsNotPublished { .. })
        ));
    }
}
