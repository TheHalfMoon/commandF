use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

use crate::{
    artifact_diff_change::{base_change, push_scalar_change, sort_changes},
    artifact_diff_error::StructuralDiffError,
    artifact_diff_model::{
        PackageEvidence, ResourceKey, ResourceKeyKind, StructuralChange, StructuralChangeKind,
        StructuralDiffReport,
    },
    artifact_diff_structure::compare_structure_definition,
    artifact_scan::scan_package_resources,
    inspect_package, PackageInspection, ResourceArtifact,
};

struct Side {
    inspection: PackageInspection,
    raw: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatchedStructureDefinitionPair {
    pub resource: ResourceKey,
    pub before_filename: String,
    pub after_filename: String,
    pub before_json: Vec<u8>,
    pub after_json: Vec<u8>,
}

pub fn diff_package_archives(
    package_name: impl Into<String>,
    before_version: impl Into<String>,
    before_digest: impl Into<String>,
    before_bytes: &[u8],
    after_version: impl Into<String>,
    after_digest: impl Into<String>,
    after_bytes: &[u8],
) -> Result<StructuralDiffReport, StructuralDiffError> {
    let package_name = package_name.into();
    let before_version = before_version.into();
    let before_digest = before_digest.into();
    let after_version = after_version.into();
    let after_digest = after_digest.into();

    let before = load_side(&package_name, &before_version, &before_digest, before_bytes)?;
    let after = load_side(&package_name, &after_version, &after_digest, after_bytes)?;

    let before_counts = canonical_counts(&before.inspection);
    let after_counts = canonical_counts(&after.inspection);
    let before_index = build_resource_index(&before.inspection, &before_counts, &after_counts)?;
    let after_index = build_resource_index(&after.inspection, &before_counts, &after_counts)?;
    let keys = before_index
        .keys()
        .chain(after_index.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut changes = Vec::new();

    for key in keys {
        match (before_index.get(&key), after_index.get(&key)) {
            (Some(before_index), Some(after_index)) => compare_matched_resource(
                &key,
                &before.inspection.resources[*before_index],
                &after.inspection.resources[*after_index],
                &before.raw,
                &after.raw,
                &mut changes,
            )?,
            (Some(before_index), None) => {
                let resource = &before.inspection.resources[*before_index];
                changes.push(base_change(
                    StructuralChangeKind::ResourceRemoved,
                    &key,
                    Some(&resource.filename),
                    None,
                ));
            }
            (None, Some(after_index)) => {
                let resource = &after.inspection.resources[*after_index];
                changes.push(base_change(
                    StructuralChangeKind::ResourceAdded,
                    &key,
                    None,
                    Some(&resource.filename),
                ));
            }
            (None, None) => unreachable!("union key must exist on at least one side"),
        }
    }

    sort_changes(&mut changes);
    Ok(StructuralDiffReport {
        schema: StructuralDiffReport::SCHEMA_V1,
        package_name,
        before: PackageEvidence {
            version: before_version,
            archive_sha256: before_digest,
        },
        after: PackageEvidence {
            version: after_version,
            archive_sha256: after_digest,
        },
        changes,
    })
}

pub fn matched_structure_definition_pairs(
    package_name: &str,
    before_version: &str,
    before_digest: &str,
    before_bytes: &[u8],
    after_version: &str,
    after_digest: &str,
    after_bytes: &[u8],
) -> Result<Vec<MatchedStructureDefinitionPair>, StructuralDiffError> {
    let before = load_side(package_name, before_version, before_digest, before_bytes)?;
    let after = load_side(package_name, after_version, after_digest, after_bytes)?;
    let before_counts = canonical_counts(&before.inspection);
    let after_counts = canonical_counts(&after.inspection);
    let before_index = build_resource_index(&before.inspection, &before_counts, &after_counts)?;
    let after_index = build_resource_index(&after.inspection, &before_counts, &after_counts)?;
    let mut pairs = Vec::new();

    for (resource, before_index) in &before_index {
        let Some(after_index) = after_index.get(resource) else {
            continue;
        };
        let before_artifact = &before.inspection.resources[*before_index];
        let after_artifact = &after.inspection.resources[*after_index];
        if before_artifact.resource_type != "StructureDefinition"
            || after_artifact.resource_type != "StructureDefinition"
        {
            continue;
        }
        let before_value = before.raw.get(&before_artifact.filename).ok_or_else(|| {
            StructuralDiffError::MissingScannedResource {
                file: before_artifact.filename.clone(),
            }
        })?;
        let after_value = after.raw.get(&after_artifact.filename).ok_or_else(|| {
            StructuralDiffError::MissingScannedResource {
                file: after_artifact.filename.clone(),
            }
        })?;
        let before_json = serde_json::to_vec(before_value).map_err(|source| {
            StructuralDiffError::Json {
                file: before_artifact.filename.clone(),
                source,
            }
        })?;
        let after_json = serde_json::to_vec(after_value).map_err(|source| {
            StructuralDiffError::Json {
                file: after_artifact.filename.clone(),
                source,
            }
        })?;
        pairs.push(MatchedStructureDefinitionPair {
            resource: resource.clone(),
            before_filename: before_artifact.filename.clone(),
            after_filename: after_artifact.filename.clone(),
            before_json,
            after_json,
        });
    }
    Ok(pairs)
}

fn load_side(
    package_name: &str,
    version: &str,
    digest: &str,
    bytes: &[u8],
) -> Result<Side, StructuralDiffError> {
    let inspection = inspect_package(package_name, version, digest, bytes)?;
    let mut raw = BTreeMap::new();
    for resource in scan_package_resources(bytes)? {
        let value = serde_json::from_slice(&resource.bytes).map_err(|source| {
            StructuralDiffError::Json {
                file: resource.filename.clone(),
                source,
            }
        })?;
        if raw.insert(resource.filename.clone(), value).is_some() {
            return Err(StructuralDiffError::DuplicateResourceFilename {
                file: resource.filename,
            });
        }
    }
    Ok(Side { inspection, raw })
}

fn canonical_counts(inspection: &PackageInspection) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for resource in &inspection.resources {
        if let Some(url) = &resource.canonical_url {
            *counts.entry(url.clone()).or_insert(0) += 1;
        }
    }
    counts
}

fn build_resource_index(
    inspection: &PackageInspection,
    before_counts: &BTreeMap<String, usize>,
    after_counts: &BTreeMap<String, usize>,
) -> Result<BTreeMap<ResourceKey, usize>, StructuralDiffError> {
    let mut index = BTreeMap::new();
    for (position, resource) in inspection.resources.iter().enumerate() {
        let key = resource_key(resource, before_counts, after_counts)?;
        if let Some(first_position) = index.insert(key.clone(), position) {
            return Err(StructuralDiffError::AmbiguousResourceKey {
                key: format!("{:?}:{}", key.kind, key.value),
                first: inspection.resources[first_position].filename.clone(),
                second: resource.filename.clone(),
            });
        }
    }
    Ok(index)
}

fn resource_key(
    resource: &ResourceArtifact,
    before_counts: &BTreeMap<String, usize>,
    after_counts: &BTreeMap<String, usize>,
) -> Result<ResourceKey, StructuralDiffError> {
    if let Some(url) = &resource.canonical_url {
        let before_count = before_counts.get(url).copied().unwrap_or(0);
        let after_count = after_counts.get(url).copied().unwrap_or(0);
        let value = if before_count <= 1 && after_count <= 1 {
            url.clone()
        } else {
            let version = resource
                .canonical_version
                .as_deref()
                .filter(|version| !version.trim().is_empty())
                .ok_or_else(
                    || StructuralDiffError::CanonicalMultiplicityMissingVersion {
                        url: url.clone(),
                        file: resource.filename.clone(),
                    },
                )?;
            format!("{url}|{version}")
        };
        return Ok(ResourceKey {
            kind: ResourceKeyKind::Canonical,
            value,
        });
    }
    if let Some(id) = &resource.id {
        return Ok(ResourceKey {
            kind: ResourceKeyKind::ResourceId,
            value: format!("{}/{}", resource.resource_type, id),
        });
    }
    Ok(ResourceKey {
        kind: ResourceKeyKind::Filename,
        value: resource.filename.clone(),
    })
}

fn compare_matched_resource(
    key: &ResourceKey,
    before: &ResourceArtifact,
    after: &ResourceArtifact,
    before_raw: &BTreeMap<String, Value>,
    after_raw: &BTreeMap<String, Value>,
    changes: &mut Vec<StructuralChange>,
) -> Result<(), StructuralDiffError> {
    push_scalar_change(
        changes,
        StructuralChangeKind::ResourceFilenameChanged,
        key,
        before,
        after,
        "filename",
        Some(Value::String(before.filename.clone())),
        Some(Value::String(after.filename.clone())),
        before.filename != after.filename,
    );
    push_scalar_change(
        changes,
        StructuralChangeKind::ResourceVersionChanged,
        key,
        before,
        after,
        "version",
        optional_string_value(&before.canonical_version),
        optional_string_value(&after.canonical_version),
        before.canonical_version != after.canonical_version,
    );
    push_scalar_change(
        changes,
        StructuralChangeKind::ResourceTypeChanged,
        key,
        before,
        after,
        "resourceType",
        Some(Value::String(before.resource_type.clone())),
        Some(Value::String(after.resource_type.clone())),
        before.resource_type != after.resource_type,
    );
    push_scalar_change(
        changes,
        StructuralChangeKind::ResourceIdChanged,
        key,
        before,
        after,
        "id",
        optional_string_value(&before.id),
        optional_string_value(&after.id),
        before.id != after.id,
    );
    push_scalar_change(
        changes,
        StructuralChangeKind::ResourceBytesChanged,
        key,
        before,
        after,
        "sha256",
        Some(Value::String(before.sha256.clone())),
        Some(Value::String(after.sha256.clone())),
        before.sha256 != after.sha256,
    );

    if before.resource_type == "StructureDefinition" && after.resource_type == "StructureDefinition"
    {
        let before_value = before_raw.get(&before.filename).ok_or_else(|| {
            StructuralDiffError::MissingScannedResource {
                file: before.filename.clone(),
            }
        })?;
        let after_value = after_raw.get(&after.filename).ok_or_else(|| {
            StructuralDiffError::MissingScannedResource {
                file: after.filename.clone(),
            }
        })?;
        compare_structure_definition(key, before, after, before_value, after_value, changes)?;
    }
    Ok(())
}

fn optional_string_value(value: &Option<String>) -> Option<Value> {
    value.as_ref().map(|value| Value::String(value.clone()))
}
