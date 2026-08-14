use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::{
    artifact_scan::scan_package_resources, ArtifactError, ElementAddress, ElementView,
    PackageCache, PackageInspection, ResourceArtifact,
};

pub fn inspect_package(
    package_name: impl Into<String>,
    package_version: impl Into<String>,
    expected_archive_sha256: impl Into<String>,
    archive_bytes: &[u8],
) -> Result<PackageInspection, ArtifactError> {
    let package_name = package_name.into();
    let package_version = package_version.into();
    let expected = expected_archive_sha256.into();
    let found = PackageCache::digest(archive_bytes);
    if found != expected {
        return Err(ArtifactError::ArchiveDigestMismatch { expected, found });
    }

    let mut resources = Vec::new();
    for scanned in scan_package_resources(archive_bytes)? {
        resources.push(parse_resource(&scanned.filename, &scanned.bytes)?);
    }

    reject_duplicate_canonicals(&resources)?;

    Ok(PackageInspection {
        schema: PackageInspection::SCHEMA_V1,
        package_name,
        package_version,
        archive_sha256: expected,
        resources,
    })
}

fn parse_resource(filename: &str, bytes: &[u8]) -> Result<ResourceArtifact, ArtifactError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|source| ArtifactError::Json {
        file: filename.to_owned(),
        source,
    })?;
    let object = value
        .as_object()
        .ok_or_else(|| ArtifactError::InvalidResourceType(filename.to_owned()))?;
    let resource_type = required_string(object, "resourceType", filename)
        .map_err(|_| ArtifactError::InvalidResourceType(filename.to_owned()))?;
    let id = optional_string(object, "id", filename)?;
    let canonical_url = optional_string(object, "url", filename)?;
    let canonical_version = optional_string(object, "version", filename)?;
    let elements = if resource_type == "StructureDefinition" {
        structure_elements(object, filename)?
    } else {
        Vec::new()
    };

    Ok(ResourceArtifact {
        filename: filename.to_owned(),
        resource_type,
        id,
        canonical_url,
        canonical_version,
        sha256: PackageCache::digest(bytes),
        elements,
    })
}

fn structure_elements(
    object: &Map<String, Value>,
    filename: &str,
) -> Result<Vec<ElementAddress>, ArtifactError> {
    let mut output = Vec::new();
    for (name, view) in [
        ("snapshot", ElementView::Snapshot),
        ("differential", ElementView::Differential),
    ] {
        let Some(container) = object.get(name) else {
            continue;
        };
        let Some(elements) = container.get("element").and_then(Value::as_array) else {
            return Err(ArtifactError::InvalidElementArray {
                file: filename.to_owned(),
                view: name.to_owned(),
            });
        };
        let mut seen = BTreeSet::new();
        for (index, element) in elements.iter().enumerate() {
            let Some(element) = element.as_object() else {
                return Err(ArtifactError::MissingElementId {
                    file: filename.to_owned(),
                    view: name.to_owned(),
                    index,
                });
            };
            let element_id = required_string(element, "id", filename).map_err(|_| {
                ArtifactError::MissingElementId {
                    file: filename.to_owned(),
                    view: name.to_owned(),
                    index,
                }
            })?;
            if !seen.insert(element_id.clone()) {
                return Err(ArtifactError::DuplicateElementId {
                    file: filename.to_owned(),
                    view: name.to_owned(),
                    id: element_id,
                });
            }
            output.push(ElementAddress {
                view,
                element_id,
                path: optional_string(element, "path", filename)?,
                slice_name: optional_string(element, "sliceName", filename)?,
            });
        }
    }
    Ok(output)
}

fn reject_duplicate_canonicals(resources: &[ResourceArtifact]) -> Result<(), ArtifactError> {
    let mut seen = BTreeMap::<String, String>::new();
    for resource in resources {
        let Some(url) = &resource.canonical_url else {
            continue;
        };
        let identity = match &resource.canonical_version {
            Some(version) => format!("{url}|{version}"),
            None => url.clone(),
        };
        if let Some(first) = seen.insert(identity.clone(), resource.filename.clone()) {
            return Err(ArtifactError::DuplicateCanonical {
                identity,
                first,
                second: resource.filename.clone(),
            });
        }
    }
    Ok(())
}

fn required_string(
    object: &Map<String, Value>,
    field: &str,
    file: &str,
) -> Result<String, ArtifactError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ArtifactError::InvalidStringField {
            file: file.to_owned(),
            field: field.to_owned(),
        })
}

fn optional_string(
    object: &Map<String, Value>,
    field: &str,
    file: &str,
) -> Result<Option<String>, ArtifactError> {
    match object.get(field) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(ArtifactError::InvalidStringField {
            file: file.to_owned(),
            field: field.to_owned(),
        }),
    }
}
