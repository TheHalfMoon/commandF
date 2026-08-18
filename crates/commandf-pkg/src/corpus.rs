use std::collections::BTreeSet;

use semver::Version;

use crate::corpus_error::CorpusError;
use crate::corpus_model::{
    CorpusPackageAttestation, CorpusPackageSide, CorpusPackageState, RealIgCase, RealIgCorpus,
};
use crate::{Lockfile, PackageCache, PackageName};

pub const MAX_CORPUS_MANIFEST_BYTES: usize = 256 * 1024;
pub const MAX_CORPUS_CASES: usize = 64;
pub const MAX_CORPUS_ARCHIVE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_EVIDENCE_URL_BYTES: usize = 4096;
const MAX_PUBLISHER_BYTES: usize = 256;

pub fn parse_corpus_manifest(bytes: &[u8]) -> Result<RealIgCorpus, CorpusError> {
    if bytes.len() > MAX_CORPUS_MANIFEST_BYTES {
        return Err(CorpusError::ManifestTooLarge {
            actual: bytes.len(),
            maximum: MAX_CORPUS_MANIFEST_BYTES,
        });
    }

    let corpus: RealIgCorpus = serde_json::from_slice(bytes)
        .map_err(|error| CorpusError::InvalidJson(error.to_string()))?;
    validate_corpus_manifest(&corpus)?;
    Ok(corpus)
}

pub fn validate_corpus_manifest(corpus: &RealIgCorpus) -> Result<(), CorpusError> {
    if corpus.schema != 1 {
        return Err(CorpusError::UnsupportedSchema(corpus.schema));
    }
    if corpus.cases.is_empty() {
        return Err(CorpusError::EmptyCorpus);
    }
    if corpus.cases.len() > MAX_CORPUS_CASES {
        return Err(CorpusError::TooManyCases {
            actual: corpus.cases.len(),
            maximum: MAX_CORPUS_CASES,
        });
    }

    let mut ids = BTreeSet::new();
    let mut previous_id: Option<&str> = None;
    for case in &corpus.cases {
        validate_case_id(&case.id)?;
        if !ids.insert(case.id.as_str()) {
            return Err(CorpusError::DuplicateCaseId(case.id.clone()));
        }
        if let Some(previous) = previous_id {
            if previous > case.id.as_str() {
                return Err(CorpusError::NonCanonicalCaseOrder {
                    previous: previous.to_owned(),
                    current: case.id.clone(),
                });
            }
        }
        previous_id = Some(case.id.as_str());
        validate_case(case)?;
    }

    Ok(())
}

pub fn canonical_corpus_manifest_bytes(corpus: &RealIgCorpus) -> Result<Vec<u8>, CorpusError> {
    validate_corpus_manifest(corpus)?;
    let mut bytes = serde_json::to_vec_pretty(corpus)
        .map_err(|error| CorpusError::Serialization(error.to_string()))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn attest_corpus_package_state(
    case: &RealIgCase,
    side: CorpusPackageSide,
    lockfile: &Lockfile,
    cache: &PackageCache,
) -> Result<CorpusPackageAttestation, CorpusError> {
    validate_case(case)?;
    lockfile
        .verify_cache(cache)
        .map_err(|error| CorpusError::CacheVerification {
            case_id: case.id.clone(),
            message: error.to_string(),
        })?;

    let (side_name, expected) = expected_state(case, side);
    let mut matches = lockfile
        .packages
        .iter()
        .filter(|package| package.name == case.package && package.version == expected.version);
    let locked = matches
        .next()
        .ok_or_else(|| CorpusError::LockedPackageMissing {
            case_id: case.id.clone(),
            package: case.package.clone(),
            version: expected.version.clone(),
        })?;
    if matches.next().is_some() {
        return Err(CorpusError::LockedPackageAmbiguous {
            case_id: case.id.clone(),
            package: case.package.clone(),
            version: expected.version.clone(),
        });
    }

    if locked.sha256 != expected.archive_sha256 {
        return Err(CorpusError::LockedPackageDigestMismatch {
            case_id: case.id.clone(),
            package: case.package.clone(),
            version: expected.version.clone(),
            expected: expected.archive_sha256.clone(),
            found: locked.sha256.clone(),
        });
    }

    let bytes =
        cache
            .read_verified(&locked.sha256)
            .map_err(|error| CorpusError::CacheVerification {
                case_id: case.id.clone(),
                message: error.to_string(),
            })?;
    let actual_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if actual_bytes != expected.archive_bytes {
        return Err(CorpusError::ArchiveSizeMismatch {
            case_id: case.id.clone(),
            side: side_name,
            expected: expected.archive_bytes,
            found: actual_bytes,
        });
    }

    let actual_sha256 = PackageCache::digest(&bytes);
    if actual_sha256 != expected.archive_sha256 {
        return Err(CorpusError::ArchiveDigestMismatch {
            case_id: case.id.clone(),
            side: side_name,
            expected: expected.archive_sha256.clone(),
            found: actual_sha256,
        });
    }

    Ok(CorpusPackageAttestation {
        case_id: case.id.clone(),
        package: case.package.clone(),
        side,
        version: expected.version.clone(),
        sha256: expected.archive_sha256.clone(),
        archive_bytes: expected.archive_bytes,
    })
}

fn expected_state(
    case: &RealIgCase,
    side: CorpusPackageSide,
) -> (&'static str, &CorpusPackageState) {
    match side {
        CorpusPackageSide::Before => ("before", &case.before),
        CorpusPackageSide::After => ("after", &case.after),
    }
}

fn validate_case(case: &RealIgCase) -> Result<(), CorpusError> {
    if PackageName::parse(case.package.clone()).is_err() {
        return Err(CorpusError::InvalidPackageName {
            case_id: case.id.clone(),
            package: case.package.clone(),
        });
    }

    let before = validate_version(&case.id, "before", &case.before.version)?;
    let after = validate_version(&case.id, "after", &case.after.version)?;
    if before == after {
        return Err(CorpusError::SameVersion(case.id.clone()));
    }

    if case.fhir_version != "4.0.1" {
        return Err(CorpusError::UnsupportedFhirVersion {
            case_id: case.id.clone(),
            version: case.fhir_version.clone(),
        });
    }

    validate_state(&case.id, "before", &case.before)?;
    validate_state(&case.id, "after", &case.after)?;
    validate_text(&case.id, "publisher", &case.publisher, MAX_PUBLISHER_BYTES)?;
    validate_https_url(&case.id, "change_evidence_url", &case.change_evidence_url)?;
    validate_https_url(&case.id, "rights_evidence_url", &case.rights_evidence_url)?;

    Ok(())
}

fn validate_case_id(id: &str) -> Result<(), CorpusError> {
    let bytes = id.as_bytes();
    if bytes.len() != 4 || bytes[0] != b'C' || !bytes[1..].iter().all(u8::is_ascii_digit) {
        return Err(CorpusError::InvalidCaseId(id.to_owned()));
    }
    Ok(())
}

fn validate_version(case_id: &str, side: &'static str, raw: &str) -> Result<Version, CorpusError> {
    Version::parse(raw).map_err(|_| CorpusError::InvalidVersion {
        case_id: case_id.to_owned(),
        side,
        version: raw.to_owned(),
    })
}

fn validate_state(
    case_id: &str,
    side: &'static str,
    state: &CorpusPackageState,
) -> Result<(), CorpusError> {
    if !is_lower_sha256(&state.archive_sha256) {
        return Err(CorpusError::InvalidArchiveSha256 {
            case_id: case_id.to_owned(),
            side,
            sha256: state.archive_sha256.clone(),
        });
    }
    if state.archive_bytes == 0 || state.archive_bytes > MAX_CORPUS_ARCHIVE_BYTES {
        return Err(CorpusError::InvalidArchiveSize {
            case_id: case_id.to_owned(),
            side,
            bytes: state.archive_bytes,
            maximum: MAX_CORPUS_ARCHIVE_BYTES,
        });
    }
    validate_https_url(case_id, "publication_url", &state.publication_url)
}

fn is_lower_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_https_url(case_id: &str, field: &'static str, value: &str) -> Result<(), CorpusError> {
    if value.len() > MAX_EVIDENCE_URL_BYTES
        || !value.starts_with("https://")
        || value.len() <= "https://".len()
        || value
            .chars()
            .any(|ch| ch.is_whitespace() || ch.is_control())
    {
        return Err(CorpusError::InvalidEvidence {
            case_id: case_id.to_owned(),
            field,
        });
    }
    Ok(())
}

fn validate_text(
    case_id: &str,
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), CorpusError> {
    if value.trim().is_empty() || value.len() > maximum || value.chars().any(char::is_control) {
        return Err(CorpusError::InvalidEvidence {
            case_id: case_id.to_owned(),
            field,
        });
    }
    Ok(())
}
