use serde::Serialize;
use thiserror::Error;

use crate::{
    PackageCache, PackageError, PackageName, IMMUTABLE_RELEASE, MUTABLE_CI, PRIMARY_HOST,
    SECONDARY_HOST,
};

pub const EXTENSION_TELEMETRY_SCHEMA: &str = "commandf.ecosystem-extension-telemetry/v1";
pub const TERMINOLOGY_TELEMETRY_SCHEMA: &str = "commandf.ecosystem-terminology-telemetry/v1";
pub const AVAILABILITY_TELEMETRY_SCHEMA: &str = "commandf.ecosystem-availability-telemetry/v1";
pub const TELEMETRY_OBSERVED: &str = "OBSERVED";
pub const TELEMETRY_MISSING: &str = "MISSING";
pub const TELEMETRY_UNSUPPORTED: &str = "UNSUPPORTED";
pub const MAX_TELEMETRY_ENTRIES: usize = 256;
pub const MAX_TELEMETRY_IDENTIFIER_BYTES: usize = 2048;
pub const MAX_TELEMETRY_DOCUMENT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryClass {
    Extension,
    Terminology,
    Availability,
}

impl TelemetryClass {
    fn schema(self) -> &'static str {
        match self {
            Self::Extension => EXTENSION_TELEMETRY_SCHEMA,
            Self::Terminology => TERMINOLOGY_TELEMETRY_SCHEMA,
            Self::Availability => AVAILABILITY_TELEMETRY_SCHEMA,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemetryEntry {
    pub identifier: String,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct TelemetryEntryBody {
    identifier: String,
    state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TelemetryDocument {
    pub schema: String,
    pub name: String,
    pub version: String,
    pub archive_sha256: String,
    pub source_host: String,
    pub mutability: String,
    pub evidence_state: String,
    pub entries: Vec<TelemetryEntry>,
    pub document_sha256: String,
}

#[derive(Serialize)]
struct TelemetryBody<'a> {
    archive_sha256: &'a str,
    entries: &'a [TelemetryEntryBody],
    evidence_state: &'a str,
    mutability: &'a str,
    name: &'a str,
    schema: &'a str,
    source_host: &'a str,
    version: &'a str,
}

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("telemetry class does not match the document schema")]
    ClassMismatch,

    #[error("telemetry evidence state must be OBSERVED, MISSING, or UNSUPPORTED")]
    InvalidState,

    #[error("telemetry entry state does not match the document state")]
    EntryStateMismatch,

    #[error("source mutability must be IMMUTABLE_RELEASE or MUTABLE_CI")]
    InvalidMutability,

    #[error("mutable CI telemetry cannot be published authority")]
    MutableSourceIsNotPublished,

    #[error("registry host is not an authorized official source")]
    UnauthorizedHost,

    #[error("archive sha256 must be 64 lowercase hex characters")]
    InvalidDigest,

    #[error("duplicate telemetry identifier")]
    DuplicateEntry,

    #[error("telemetry entry count exceeds {MAX_TELEMETRY_ENTRIES}")]
    TooManyEntries,

    #[error("telemetry identifier exceeds {MAX_TELEMETRY_IDENTIFIER_BYTES} bytes")]
    IdentifierTooLong,

    #[error("telemetry document exceeds {MAX_TELEMETRY_DOCUMENT_BYTES} bytes")]
    DocumentTooLarge,

    #[error(transparent)]
    Package(#[from] PackageError),
}

pub struct TelemetryInput<'a> {
    pub class: TelemetryClass,
    pub name: &'a str,
    pub version: &'a str,
    pub archive_sha256: &'a str,
    pub source_host: &'a str,
    pub mutability: &'a str,
    pub evidence_state: &'a str,
    pub entries: &'a [TelemetryEntry],
}

pub fn project_telemetry(input: &TelemetryInput<'_>) -> Result<TelemetryDocument, TelemetryError> {
    let class = input.class;
    let name = input.name;
    let version = input.version;
    let archive_sha256 = input.archive_sha256;
    let source_host = input.source_host;
    let mutability = input.mutability;
    let evidence_state = input.evidence_state;
    let entries = input.entries;
    PackageName::parse(name)?;
    if semver::Version::parse(version).is_err() {
        return Err(TelemetryError::Package(PackageError::InvalidRequest(
            "telemetry version must be an exact semantic version".to_owned(),
        )));
    }
    if archive_sha256.len() != 64
        || !archive_sha256
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        return Err(TelemetryError::InvalidDigest);
    }
    if source_host != PRIMARY_HOST && source_host != SECONDARY_HOST {
        return Err(TelemetryError::UnauthorizedHost);
    }
    if mutability != IMMUTABLE_RELEASE && mutability != MUTABLE_CI {
        return Err(TelemetryError::InvalidMutability);
    }
    if evidence_state != TELEMETRY_OBSERVED
        && evidence_state != TELEMETRY_MISSING
        && evidence_state != TELEMETRY_UNSUPPORTED
    {
        return Err(TelemetryError::InvalidState);
    }
    if evidence_state != TELEMETRY_OBSERVED && !entries.is_empty() {
        return Err(TelemetryError::EntryStateMismatch);
    }
    if entries.len() > MAX_TELEMETRY_ENTRIES {
        return Err(TelemetryError::TooManyEntries);
    }
    let mut stored = Vec::with_capacity(entries.len());
    for entry in entries {
        if entry.identifier.len() > MAX_TELEMETRY_IDENTIFIER_BYTES {
            return Err(TelemetryError::IdentifierTooLong);
        }
        if entry.state != evidence_state {
            return Err(TelemetryError::EntryStateMismatch);
        }
        stored.push(TelemetryEntryBody {
            identifier: entry.identifier.clone(),
            state: entry.state.clone(),
        });
    }
    stored.sort_by(|left, right| left.identifier.cmp(&right.identifier));
    for pair in stored.windows(2) {
        if pair[0].identifier == pair[1].identifier {
            return Err(TelemetryError::DuplicateEntry);
        }
    }
    let schema = class.schema();
    let body = TelemetryBody {
        archive_sha256,
        entries: &stored,
        evidence_state,
        mutability,
        name,
        schema,
        source_host,
        version,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    if bytes.len() > MAX_TELEMETRY_DOCUMENT_BYTES {
        return Err(TelemetryError::DocumentTooLarge);
    }
    Ok(TelemetryDocument {
        schema: schema.to_owned(),
        name: name.to_owned(),
        version: version.to_owned(),
        archive_sha256: archive_sha256.to_owned(),
        source_host: source_host.to_owned(),
        mutability: mutability.to_owned(),
        evidence_state: evidence_state.to_owned(),
        entries: stored
            .into_iter()
            .map(|entry| TelemetryEntry {
                identifier: entry.identifier,
                state: entry.state,
            })
            .collect(),
        document_sha256: PackageCache::digest(&bytes),
    })
}

pub fn replay_telemetry(input: &TelemetryInput<'_>) -> Result<TelemetryDocument, TelemetryError> {
    project_telemetry(input)
}

pub fn require_published_telemetry(document: &TelemetryDocument) -> Result<(), TelemetryError> {
    if document.mutability == MUTABLE_CI {
        return Err(TelemetryError::MutableSourceIsNotPublished);
    }
    if document.mutability != IMMUTABLE_RELEASE {
        return Err(TelemetryError::InvalidMutability);
    }
    Ok(())
}

pub fn require_telemetry_schema(
    document: &TelemetryDocument,
    class: TelemetryClass,
) -> Result<(), TelemetryError> {
    if document.schema != class.schema() {
        return Err(TelemetryError::ClassMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> String {
        "ab".repeat(32)
    }

    fn entry(identifier: &str) -> TelemetryEntry {
        TelemetryEntry {
            identifier: identifier.to_owned(),
            state: TELEMETRY_OBSERVED.to_owned(),
        }
    }

    fn project(
        class: TelemetryClass,
        state: &str,
        mutability: &str,
        entries: &[TelemetryEntry],
    ) -> TelemetryDocument {
        project_telemetry(&TelemetryInput {
            class,
            name: "hl7.fhir.r4.core",
            version: "4.0.1",
            archive_sha256: &digest(),
            source_host: PRIMARY_HOST,
            mutability,
            evidence_state: state,
            entries,
        })
        .expect("project")
    }

    #[test]
    fn classes_stay_separate_and_order_is_stable() {
        let extension = project(
            TelemetryClass::Extension,
            TELEMETRY_OBSERVED,
            IMMUTABLE_RELEASE,
            &[entry("b"), entry("a")],
        );
        let terminology = project(
            TelemetryClass::Terminology,
            TELEMETRY_OBSERVED,
            IMMUTABLE_RELEASE,
            &[entry("a")],
        );
        let replayed = replay_telemetry(&TelemetryInput {
            class: TelemetryClass::Extension,
            name: "hl7.fhir.r4.core",
            version: "4.0.1",
            archive_sha256: &digest(),
            source_host: PRIMARY_HOST,
            mutability: IMMUTABLE_RELEASE,
            evidence_state: TELEMETRY_OBSERVED,
            entries: &[entry("a"), entry("b")],
        })
        .expect("replay");
        assert_eq!(extension.entries[0].identifier, "a");
        assert_eq!(extension.document_sha256, replayed.document_sha256);
        assert!(require_telemetry_schema(&extension, TelemetryClass::Terminology).is_err());
        assert!(require_telemetry_schema(&terminology, TelemetryClass::Extension).is_err());
        assert_ne!(extension.schema, terminology.schema);
        assert_ne!(
            extension.schema,
            project(
                TelemetryClass::Availability,
                TELEMETRY_OBSERVED,
                IMMUTABLE_RELEASE,
                &[entry("a")],
            )
            .schema
        );
    }

    #[test]
    fn states_mutability_and_bounds_are_explicit() {
        let missing = project(
            TelemetryClass::Extension,
            TELEMETRY_MISSING,
            IMMUTABLE_RELEASE,
            &[],
        );
        let unsupported = project(
            TelemetryClass::Extension,
            TELEMETRY_UNSUPPORTED,
            IMMUTABLE_RELEASE,
            &[],
        );
        assert_ne!(missing.document_sha256, unsupported.document_sha256);
        let mutable = project(
            TelemetryClass::Availability,
            TELEMETRY_OBSERVED,
            MUTABLE_CI,
            &[entry("up")],
        );
        assert_eq!(mutable.mutability, MUTABLE_CI);
        assert!(require_published_telemetry(&mutable).is_err());
        assert!(require_published_telemetry(&missing).is_ok());

        let too_many = (0..257)
            .map(|index| entry(&format!("id-{index}")))
            .collect::<Vec<_>>();
        let error = project_telemetry(&TelemetryInput {
            class: TelemetryClass::Extension,
            name: "hl7.fhir.r4.core",
            version: "4.0.1",
            archive_sha256: &digest(),
            source_host: PRIMARY_HOST,
            mutability: IMMUTABLE_RELEASE,
            evidence_state: TELEMETRY_OBSERVED,
            entries: &too_many,
        })
        .unwrap_err();
        assert!(matches!(error, TelemetryError::TooManyEntries));

        let long = entry(&"x".repeat(MAX_TELEMETRY_IDENTIFIER_BYTES + 1));
        let error = project_telemetry(&TelemetryInput {
            class: TelemetryClass::Terminology,
            name: "hl7.fhir.r4.core",
            version: "4.0.1",
            archive_sha256: &digest(),
            source_host: PRIMARY_HOST,
            mutability: IMMUTABLE_RELEASE,
            evidence_state: TELEMETRY_OBSERVED,
            entries: &[long],
        })
        .unwrap_err();
        assert!(matches!(error, TelemetryError::IdentifierTooLong));

        let encoded = serde_json::to_string(&TelemetryBody {
            archive_sha256: &missing.archive_sha256,
            entries: &[],
            evidence_state: &missing.evidence_state,
            mutability: &missing.mutability,
            name: &missing.name,
            schema: &missing.schema,
            source_host: &missing.source_host,
            version: &missing.version,
        })
        .expect("json");
        assert!(!encoded.contains("PROVEN_COMPATIBLE"));
        assert!(!encoded.contains("ALLOW"));
        assert!(!encoded.contains("retrieved_at"));
    }
}
