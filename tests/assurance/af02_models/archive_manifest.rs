use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogicalEntry {
    pub path: String,
    pub body: Vec<u8>,
    pub decompressed_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_entries: usize,
    pub max_manifest_bytes: u64,
    pub max_decompressed_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestView {
    pub name: String,
    pub version: String,
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Outcome {
    AcceptManifest {
        entry_index: usize,
        manifest_sha256: String,
        manifest: ManifestView,
    },
    MissingManifest,
    ManifestTooLarge,
    EntryLimit,
    DecompressedLimit,
    InvalidManifestJson,
}

pub const INVALIDITY_CLASSES: &[&str] = &[
    "DUPLICATE_MANIFEST",
    "MANIFEST_TOO_LARGE",
    "MISSING_MANIFEST",
    "PREFIX_VARIANT",
    "ENTRY_COUNT_OVER_LIMIT",
    "DECOMPRESSED_BUDGET_OVER_LIMIT",
    "INVALID_MANIFEST_JSON",
];

pub fn evaluate(entries: &[LogicalEntry], limits: Limits) -> Outcome {
    let mut decompressed = 0_u64;
    for (entry_index, entry) in entries.iter().enumerate() {
        if entry_index + 1 > limits.max_entries {
            return Outcome::EntryLimit;
        }

        decompressed = decompressed.saturating_add(entry.decompressed_bytes);
        if decompressed > limits.max_decompressed_bytes {
            return Outcome::DecompressedLimit;
        }

        let normalized = entry.path.strip_prefix("./").unwrap_or(&entry.path);
        if normalized != "package/package.json" {
            continue;
        }
        if entry.body.len() as u64 > limits.max_manifest_bytes {
            return Outcome::ManifestTooLarge;
        }

        let Ok(value) = serde_json::from_slice::<Value>(&entry.body) else {
            return Outcome::InvalidManifestJson;
        };
        let Some(object) = value.as_object() else {
            return Outcome::InvalidManifestJson;
        };
        let Some(name) = object.get("name").and_then(Value::as_str) else {
            return Outcome::InvalidManifestJson;
        };
        let Some(version) = object.get("version").and_then(Value::as_str) else {
            return Outcome::InvalidManifestJson;
        };

        let mut dependencies = BTreeMap::new();
        if let Some(raw_dependencies) = object.get("dependencies") {
            let Some(map) = raw_dependencies.as_object() else {
                return Outcome::InvalidManifestJson;
            };
            for (dependency, constraint) in map {
                let Some(constraint) = constraint.as_str() else {
                    return Outcome::InvalidManifestJson;
                };
                dependencies.insert(dependency.clone(), constraint.to_owned());
            }
        }

        return Outcome::AcceptManifest {
            entry_index,
            manifest_sha256: format!("{:x}", Sha256::digest(&entry.body)),
            manifest: ManifestView {
                name: name.to_owned(),
                version: version.to_owned(),
                dependencies,
            },
        };
    }

    Outcome::MissingManifest
}
