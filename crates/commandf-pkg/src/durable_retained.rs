use std::collections::BTreeSet;
use std::fmt;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::Serialize;
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const DURABLE_SCHEMA: &str = "commandf.durable-retained-authority/v1";
pub const PROJECTION_SCHEMA: &str = "commandf.durable-retained-projection/v1";

const IDENTITY_KNOWN: &str = "HISTORICAL_ARTIFACT_IDENTITY_KNOWN";
const BYTES_VERIFIED: &str = "HISTORICAL_ARTIFACT_BYTES_VERIFIED";
const BYTES_UNAVAILABLE: &str = "HISTORICAL_ARTIFACT_BYTES_UNAVAILABLE";
const PACKET_VERIFIED: &str = "DURABLE_PACKET_VERIFIED";
const REPLAY_VERIFIED: &str = "OFFLINE_REPLAY_VERIFIED";

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DurableRetainedError {
    #[error("malformed durable retained packet: {0}")]
    Malformed(String),
    #[error("partial durable retained packet: {0}")]
    Partial(String),
    #[error("durable retained authority mismatch: {0}")]
    Mismatch(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedRetainedBinding {
    pub artifact_id: u64,
    pub artifact_name: String,
    pub workflow_run_id: u64,
    pub recorded_sha256: String,
    pub retained_head: String,
    pub retained_base: String,
    pub manifest_git_blob_sha1: String,
    pub donor_git_blob_sha1: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiveArtifactObservation {
    NotConsulted,
    AbsentEmpty,
    PresentExact,
    IdentityMismatch,
    DigestMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HistoricalByteState {
    Verified,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegenerationClass {
    HistoricalBytesVerified,
    ReconstructedSemanticEquivalentCandidate,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DurableProjection {
    pub schema: &'static str,
    pub historical_artifact_identity_state: String,
    pub historical_artifact_byte_state: String,
    pub durable_packet_state: String,
    pub offline_replay_state: String,
    pub live_github_artifact_state: String,
    pub recorded_sha256: String,
    pub byte_digest_verified: bool,
}

pub fn classify_regenerated_archive(bytes: &[u8], recorded_sha256: &str) -> RegenerationClass {
    if sha256_hex(bytes) == recorded_sha256 {
        RegenerationClass::HistoricalBytesVerified
    } else {
        RegenerationClass::ReconstructedSemanticEquivalentCandidate
    }
}

pub fn evaluate_durable_packet(
    packet_bytes: &[u8],
    trusted: &TrustedRetainedBinding,
    artifact_bytes: Option<&[u8]>,
    manifest_git_blob_sha1: &str,
    donor_git_blob_sha1: &str,
    live: LiveArtifactObservation,
) -> Result<DurableProjection, DurableRetainedError> {
    let packet = parse_json_no_duplicates(packet_bytes).map_err(|error| {
        DurableRetainedError::Malformed(format!("packet JSON is not a closed object: {error}"))
    })?;
    let object = packet.as_object().ok_or_else(|| {
        DurableRetainedError::Malformed("durable packet must be a JSON object".to_owned())
    })?;
    require_exact_keys(
        object,
        &[
            "artifact_id",
            "artifact_name",
            "donor_git_blob_sha1",
            "durable_bytes_sha256",
            "historical_artifact_byte_state",
            "historical_artifact_identity_state",
            "manifest_git_blob_sha1",
            "recorded_sha256",
            "retained_base",
            "retained_head",
            "schema",
            "workflow_run_id",
        ],
    )?;

    let schema = require_str(object, "schema")?;
    if schema != DURABLE_SCHEMA {
        return Err(DurableRetainedError::Malformed(format!(
            "unexpected schema {schema}"
        )));
    }
    if require_str(object, "historical_artifact_identity_state")? != IDENTITY_KNOWN {
        return Err(DurableRetainedError::Malformed(
            "identity state must be HISTORICAL_ARTIFACT_IDENTITY_KNOWN".to_owned(),
        ));
    }
    let byte_state = match require_str(object, "historical_artifact_byte_state")? {
        state if state == BYTES_VERIFIED => HistoricalByteState::Verified,
        state if state == BYTES_UNAVAILABLE => HistoricalByteState::Unavailable,
        other => {
            return Err(DurableRetainedError::Malformed(format!(
                "unsupported byte state {other}"
            )))
        }
    };

    expect_u64(object, "artifact_id", trusted.artifact_id, "artifact id")?;
    expect_str_eq(
        object,
        "artifact_name",
        &trusted.artifact_name,
        "artifact name",
    )?;
    expect_u64(
        object,
        "workflow_run_id",
        trusted.workflow_run_id,
        "workflow run",
    )?;
    expect_str_eq(
        object,
        "recorded_sha256",
        &trusted.recorded_sha256,
        "recorded sha256",
    )?;
    expect_str_eq(
        object,
        "retained_head",
        &trusted.retained_head,
        "retained head",
    )?;
    expect_str_eq(
        object,
        "retained_base",
        &trusted.retained_base,
        "retained base",
    )?;
    expect_str_eq(
        object,
        "manifest_git_blob_sha1",
        &trusted.manifest_git_blob_sha1,
        "manifest git blob",
    )?;
    expect_str_eq(
        object,
        "donor_git_blob_sha1",
        &trusted.donor_git_blob_sha1,
        "donor git blob",
    )?;

    if manifest_git_blob_sha1 != trusted.manifest_git_blob_sha1 {
        return Err(DurableRetainedError::Mismatch(
            "retained manifest Git blob mismatch".to_owned(),
        ));
    }
    if donor_git_blob_sha1 != trusted.donor_git_blob_sha1 {
        return Err(DurableRetainedError::Mismatch(
            "retained donor Git blob mismatch".to_owned(),
        ));
    }

    let declared_digest = object.get("durable_bytes_sha256").ok_or_else(|| {
        DurableRetainedError::Partial("durable_bytes_sha256 is missing".to_owned())
    })?;
    let byte_digest_verified = match (byte_state, artifact_bytes, declared_digest) {
        (HistoricalByteState::Verified, Some(bytes), Value::String(digest)) => {
            if digest != &trusted.recorded_sha256 {
                return Err(DurableRetainedError::Mismatch(
                    "durable packet digest mismatch".to_owned(),
                ));
            }
            if sha256_hex(bytes) != trusted.recorded_sha256 {
                return Err(DurableRetainedError::Mismatch(
                    "durable packet byte mismatch".to_owned(),
                ));
            }
            true
        }
        (HistoricalByteState::Verified, None, _) => {
            return Err(DurableRetainedError::Partial(
                "verified byte state requires the historical bytes".to_owned(),
            ));
        }
        (HistoricalByteState::Verified, Some(_), Value::Null) => {
            return Err(DurableRetainedError::Partial(
                "verified byte state requires durable_bytes_sha256".to_owned(),
            ));
        }
        (HistoricalByteState::Unavailable, None, Value::Null) => false,
        (HistoricalByteState::Unavailable, Some(_), _) => {
            return Err(DurableRetainedError::Mismatch(
                "unavailable byte state cannot carry artifact bytes".to_owned(),
            ));
        }
        (HistoricalByteState::Unavailable, None, Value::String(_)) => {
            return Err(DurableRetainedError::Mismatch(
                "unavailable byte state cannot claim a durable digest".to_owned(),
            ));
        }
        _ => {
            return Err(DurableRetainedError::Malformed(
                "durable_bytes_sha256 must be a string or null".to_owned(),
            ))
        }
    };

    let live_state = match live {
        LiveArtifactObservation::NotConsulted => "NOT_CONSULTED",
        LiveArtifactObservation::AbsentEmpty => "ABSENT",
        LiveArtifactObservation::PresentExact => "PRESENT_EXACT",
        LiveArtifactObservation::IdentityMismatch => {
            return Err(DurableRetainedError::Mismatch(
                "live artifact identity mismatch".to_owned(),
            ))
        }
        LiveArtifactObservation::DigestMismatch => {
            return Err(DurableRetainedError::Mismatch(
                "live artifact digest mismatch".to_owned(),
            ))
        }
    };

    Ok(DurableProjection {
        schema: PROJECTION_SCHEMA,
        historical_artifact_identity_state: IDENTITY_KNOWN.to_owned(),
        historical_artifact_byte_state: match byte_state {
            HistoricalByteState::Verified => BYTES_VERIFIED,
            HistoricalByteState::Unavailable => BYTES_UNAVAILABLE,
        }
        .to_owned(),
        durable_packet_state: PACKET_VERIFIED.to_owned(),
        offline_replay_state: REPLAY_VERIFIED.to_owned(),
        live_github_artifact_state: live_state.to_owned(),
        recorded_sha256: trusted.recorded_sha256.clone(),
        byte_digest_verified,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn require_exact_keys(
    object: &Map<String, Value>,
    expected: &[&str],
) -> Result<(), DurableRetainedError> {
    let actual: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    let expected_set: BTreeSet<&str> = expected.iter().copied().collect();
    if actual != expected_set {
        let missing: Vec<&str> = expected_set.difference(&actual).copied().collect();
        let extra: Vec<&str> = actual.difference(&expected_set).copied().collect();
        if !missing.is_empty() {
            return Err(DurableRetainedError::Partial(format!(
                "missing fields {}",
                missing.join(",")
            )));
        }
        return Err(DurableRetainedError::Malformed(format!(
            "unexpected fields {}",
            extra.join(",")
        )));
    }
    Ok(())
}

fn require_str<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a str, DurableRetainedError> {
    object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| DurableRetainedError::Partial(format!("{key} is missing")))
}

fn expect_str_eq(
    object: &Map<String, Value>,
    key: &str,
    expected: &str,
    label: &str,
) -> Result<(), DurableRetainedError> {
    let actual = require_str(object, key)?;
    if actual != expected {
        return Err(DurableRetainedError::Mismatch(format!("wrong {label}")));
    }
    Ok(())
}

fn expect_u64(
    object: &Map<String, Value>,
    key: &str,
    expected: u64,
    label: &str,
) -> Result<(), DurableRetainedError> {
    let actual = object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| DurableRetainedError::Partial(format!("{key} is missing")))?;
    if actual != expected {
        return Err(DurableRetainedError::Mismatch(format!("wrong {label}")));
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct NoDuplicateValue;

struct NoDuplicateVisitor;

impl<'de> DeserializeSeed<'de> for NoDuplicateValue {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(NoDuplicateVisitor)
    }
}

impl<'de> Visitor<'de> for NoDuplicateVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Err(E::custom("floating-point JSON numbers are prohibited"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(NoDuplicateValue)? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut keys = BTreeSet::new();
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!(
                    "duplicate JSON object key {key:?}"
                )));
            }
            let value = map.next_value_seed(NoDuplicateValue)?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

fn parse_json_no_duplicates(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = NoDuplicateValue.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}
