use std::collections::BTreeSet;
use std::fmt;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonicalError {
    #[error("floating-point JSON numbers are prohibited")]
    FloatNumber,
    #[error("failed to encode JSON string: {0}")]
    StringEncoding(#[from] serde_json::Error),
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

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("non-finite JSON number"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        NoDuplicateValue.deserialize(deserializer)
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

pub fn parse_json_no_duplicates(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = NoDuplicateValue.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

pub fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, CanonicalError> {
    let mut out = Vec::new();
    write_value(value, &mut out)?;
    Ok(out)
}

pub fn canonical_sha256(value: &Value) -> Result<String, CanonicalError> {
    Ok(sha256_hex(&canonical_json_bytes(value)?))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

pub fn git_blob_sha1_hex(bytes: &[u8]) -> String {
    let header = format!("blob {}\0", bytes.len());
    let mut object = Vec::with_capacity(header.len() + bytes.len());
    object.extend_from_slice(header.as_bytes());
    object.extend_from_slice(bytes);
    sha1_hex(&object)
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut message = bytes.to_vec();
    let bit_len = (message.len() as u64).wrapping_mul(8);
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    let mut h0 = 0x6745_2301u32;
    let mut h1 = 0xefcd_ab89u32;
    let mut h2 = 0x98ba_dcfeu32;
    let mut h3 = 0x1032_5476u32;
    let mut h4 = 0xc3d2_e1f0u32;

    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 80];
        for (index, word) in words[..16].iter_mut().enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for (index, word) in words.iter().enumerate() {
            let (function, constant) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5a82_7999),
                20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
                _ => (b ^ c ^ d, 0xca62_c1d6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(function)
                .wrapping_add(e)
                .wrapping_add(constant)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    format!("{h0:08x}{h1:08x}{h2:08x}{h3:08x}{h4:08x}")
}

fn write_value(value: &Value, out: &mut Vec<u8>) -> Result<(), CanonicalError> {
    match value {
        Value::Null => out.extend_from_slice(b"null"),
        Value::Bool(value) => {
            out.extend_from_slice(if *value { b"true" } else { b"false" });
        }
        Value::Number(number) => {
            if number.is_i64() || number.is_u64() {
                out.extend_from_slice(number.to_string().as_bytes());
            } else {
                return Err(CanonicalError::FloatNumber);
            }
        }
        Value::String(value) => {
            out.extend_from_slice(serde_json::to_string(value)?.as_bytes());
        }
        Value::Array(values) => {
            out.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    out.push(b',');
                }
                write_value(value, out)?;
            }
            out.push(b']');
        }
        Value::Object(values) => {
            out.push(b'{');
            let mut keys: Vec<&String> = values.keys().collect();
            keys.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
            for (index, key) in keys.into_iter().enumerate() {
                if index != 0 {
                    out.push(b',');
                }
                out.extend_from_slice(serde_json::to_string(key)?.as_bytes());
                out.push(b':');
                write_value(&values[key], out)?;
            }
            out.push(b'}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_recursive_object_keys_and_preserves_array_order() {
        let value: Value =
            parse_json_no_duplicates(br#"{"z":{"b":2,"a":1},"a":[{"y":2,"x":1},0]}"#).unwrap();
        assert_eq!(
            canonical_json_bytes(&value).unwrap(),
            br#"{"a":[{"x":1,"y":2},0],"z":{"a":1,"b":2}}"#
        );
    }

    #[test]
    fn rejects_floats() {
        let value: Value = parse_json_no_duplicates(br#"{"n":1.5}"#).unwrap();
        assert!(matches!(
            canonical_json_bytes(&value),
            Err(CanonicalError::FloatNumber)
        ));
    }

    #[test]
    fn rejects_duplicate_object_keys_at_any_depth() {
        let error = parse_json_no_duplicates(br#"{"outer":{"id":1,"id":2}}"#).unwrap_err();
        assert!(error
            .to_string()
            .contains("duplicate JSON object key \"id\""));
    }

    #[test]
    fn computes_git_blob_identity() {
        assert_eq!(
            git_blob_sha1_hex(b"test content\n"),
            "d670460b4b4aece5915caf5c68d12f560a9fe3e4"
        );
    }
}
