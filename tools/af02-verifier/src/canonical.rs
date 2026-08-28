use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CanonicalError {
    #[error("floating-point JSON numbers are prohibited")]
    FloatNumber,
    #[error("failed to encode JSON string: {0}")]
    StringEncoding(#[from] serde_json::Error),
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
        let value: Value = serde_json::from_str(
            r#"{"z":{"b":2,"a":1},"a":[{"y":2,"x":1},0]}"#,
        )
        .unwrap();
        assert_eq!(
            canonical_json_bytes(&value).unwrap(),
            br#"{"a":[{"x":1,"y":2},0],"z":{"a":1,"b":2}}"#
        );
    }

    #[test]
    fn rejects_floats() {
        let value: Value = serde_json::from_str(r#"{"n":1.5}"#).unwrap();
        assert!(matches!(
            canonical_json_bytes(&value),
            Err(CanonicalError::FloatNumber)
        ));
    }
}
