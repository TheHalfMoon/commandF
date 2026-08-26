use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use crate::artifact_scan::scan_package_resources;
use crate::{
    inspect_package, ArtifactError, CanonicalReferenceRelation, CanonicalResolutionStatus,
    ContextArtifactIdentity, ContextArtifactNode, ContextCanonicalReferenceEdge, ContextCoverage,
    ContextGraphError, ContextGraphReport, ContextPackageDependencyEdge, ContextPackageIdentity,
    ContextPackageNode, Lockfile, PackageCache,
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PendingCanonicalReference {
    source: ContextArtifactIdentity,
    relation: CanonicalReferenceRelation,
    source_path: String,
    source_element_id: Option<String>,
    canonical: String,
}

pub fn build_context_graph(
    lock: &Lockfile,
    cache: &PackageCache,
) -> Result<ContextGraphReport, ContextGraphError> {
    if lock.schema != Lockfile::SCHEMA_V2 {
        return Err(ContextGraphError::RequiresLockV2 { found: lock.schema });
    }
    lock.validate_v2()?;

    let mut packages = Vec::new();
    let mut package_identities = BTreeMap::new();
    for package in &lock.packages {
        let identity = ContextPackageIdentity {
            name: package.name.clone(),
            version: package.version.clone(),
            sha256: package.sha256.clone(),
        };
        package_identities.insert(
            (package.name.clone(), package.version.clone()),
            identity.clone(),
        );
        packages.push(ContextPackageNode {
            identity,
            source: package.source.clone(),
        });
    }
    packages.sort();

    let mut package_dependency_edges = Vec::new();
    for edge in &lock.resolved_dependencies {
        let from = package_identities
            .get(&(edge.from_name.clone(), edge.from_version.clone()))
            .cloned()
            .ok_or_else(|| {
                crate::PackageError::InvalidLockfile(format!(
                    "context graph source package {}@{} disappeared after lock validation",
                    edge.from_name, edge.from_version
                ))
            })?;
        let to = package_identities
            .get(&(edge.to_name.clone(), edge.to_version.clone()))
            .cloned()
            .ok_or_else(|| {
                crate::PackageError::InvalidLockfile(format!(
                    "context graph target package {}@{} disappeared after lock validation",
                    edge.to_name, edge.to_version
                ))
            })?;
        package_dependency_edges.push(ContextPackageDependencyEdge {
            from,
            to,
            declared_constraint: edge.declared_constraint.clone(),
        });
    }
    package_dependency_edges.sort();
    package_dependency_edges.dedup();

    let mut artifacts = Vec::new();
    let mut pending_references = Vec::new();
    let mut present_resource_types = BTreeSet::new();

    for package in &lock.packages {
        let archive_bytes = cache.read_verified(&package.sha256)?;
        let inspection = inspect_package(
            package.name.clone(),
            package.version.clone(),
            package.sha256.clone(),
            &archive_bytes,
        )?;
        let scanned_resources = scan_package_resources(&archive_bytes)?;
        let inspected_by_filename = inspection
            .resources
            .iter()
            .map(|resource| (resource.filename.as_str(), resource))
            .collect::<BTreeMap<_, _>>();

        for scanned in scanned_resources {
            let inspected = inspected_by_filename
                .get(scanned.filename.as_str())
                .ok_or_else(|| ContextGraphError::ArtifactInventoryMismatch {
                    file: scanned.filename.clone(),
                })?;
            present_resource_types.insert(inspected.resource_type.clone());

            let package_identity = package_identities
                .get(&(package.name.clone(), package.version.clone()))
                .cloned()
                .ok_or_else(|| {
                    crate::PackageError::InvalidLockfile(format!(
                        "context graph package {}@{} disappeared after lock validation",
                        package.name, package.version
                    ))
                })?;
            let artifact_identity = ContextArtifactIdentity {
                package: package_identity,
                filename: inspected.filename.clone(),
                sha256: inspected.sha256.clone(),
            };
            artifacts.push(ContextArtifactNode {
                identity: artifact_identity.clone(),
                resource_type: inspected.resource_type.clone(),
                id: inspected.id.clone(),
                canonical_url: inspected.canonical_url.clone(),
                canonical_version: inspected.canonical_version.clone(),
            });

            let value: Value = serde_json::from_slice(&scanned.bytes).map_err(|source| {
                ContextGraphError::Artifact(ArtifactError::Json {
                    file: scanned.filename.clone(),
                    source,
                })
            })?;
            let object = value
                .as_object()
                .ok_or_else(|| invalid_field(&scanned.filename, "$", "a JSON object"))?;
            extract_references(
                &artifact_identity,
                &scanned.filename,
                &inspected.resource_type,
                object,
                &mut pending_references,
            )?;
        }
    }

    artifacts.sort();
    artifacts.dedup();
    pending_references.sort();
    pending_references.dedup();

    let canonical_index = build_canonical_index(&artifacts);
    let mut canonical_reference_edges = Vec::new();
    for pending in pending_references {
        canonical_reference_edges.push(resolve_reference(pending, &canonical_index)?);
    }
    canonical_reference_edges.sort();
    canonical_reference_edges.dedup();

    let supported_source_resource_types = vec![
        "CodeSystem".to_owned(),
        "StructureDefinition".to_owned(),
        "ValueSet".to_owned(),
    ];
    let supported = supported_source_resource_types
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let unsupported_source_resource_types = present_resource_types
        .into_iter()
        .filter(|resource_type| !supported.contains(resource_type.as_str()))
        .collect::<Vec<_>>();

    Ok(ContextGraphReport {
        schema: ContextGraphReport::SCHEMA_V1,
        lock_schema: lock.schema,
        root_requests: lock.roots.clone(),
        packages,
        artifacts,
        package_dependency_edges,
        canonical_reference_edges,
        coverage: ContextCoverage {
            extractor_schema: 1,
            supported_source_resource_types,
            unsupported_source_resource_types,
        },
    })
}

fn build_canonical_index(
    artifacts: &[ContextArtifactNode],
) -> BTreeMap<String, Vec<(Option<String>, ContextArtifactIdentity)>> {
    let mut index = BTreeMap::<String, Vec<(Option<String>, ContextArtifactIdentity)>>::new();
    for artifact in artifacts {
        let Some(url) = &artifact.canonical_url else {
            continue;
        };
        index.entry(url.clone()).or_default().push((
            artifact.canonical_version.clone(),
            artifact.identity.clone(),
        ));
    }
    for candidates in index.values_mut() {
        candidates.sort();
        candidates.dedup();
    }
    index
}

fn resolve_reference(
    pending: PendingCanonicalReference,
    index: &BTreeMap<String, Vec<(Option<String>, ContextArtifactIdentity)>>,
) -> Result<ContextCanonicalReferenceEdge, ContextGraphError> {
    let (target_url, explicit_version) = parse_canonical_target(
        &pending.canonical,
        &pending.source.filename,
        &pending.source_path,
    )?;
    let mut candidates = index
        .get(target_url)
        .into_iter()
        .flatten()
        .filter(|(candidate_version, _)| match explicit_version {
            Some(version) => candidate_version.as_deref() == Some(version),
            None => true,
        })
        .map(|(_, identity)| identity.clone())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();

    let resolution = match candidates.len() {
        0 => CanonicalResolutionStatus::External,
        1 => CanonicalResolutionStatus::Resolved,
        _ => CanonicalResolutionStatus::Ambiguous,
    };

    Ok(ContextCanonicalReferenceEdge {
        source: pending.source,
        relation: pending.relation,
        source_path: pending.source_path,
        source_element_id: pending.source_element_id,
        canonical: pending.canonical,
        resolution,
        candidates,
    })
}

fn parse_canonical_target<'a>(
    canonical: &'a str,
    file: &str,
    path: &str,
) -> Result<(&'a str, Option<&'a str>), ContextGraphError> {
    let without_fragment = canonical
        .split_once('#')
        .map(|(target, _)| target)
        .unwrap_or(canonical);
    if without_fragment.is_empty() {
        return Err(ContextGraphError::EmptyCanonicalTarget {
            file: file.to_owned(),
            path: path.to_owned(),
        });
    }
    if let Some((url, version)) = without_fragment.rsplit_once('|') {
        if url.is_empty() {
            return Err(ContextGraphError::EmptyCanonicalTarget {
                file: file.to_owned(),
                path: path.to_owned(),
            });
        }
        if version.is_empty() {
            return Err(ContextGraphError::EmptyCanonicalVersion {
                file: file.to_owned(),
                path: path.to_owned(),
            });
        }
        Ok((url, Some(version)))
    } else {
        Ok((without_fragment, None))
    }
}

fn extract_references(
    source: &ContextArtifactIdentity,
    file: &str,
    resource_type: &str,
    object: &Map<String, Value>,
    output: &mut Vec<PendingCanonicalReference>,
) -> Result<(), ContextGraphError> {
    match resource_type {
        "StructureDefinition" => extract_structure_definition(source, file, object, output),
        "ValueSet" => extract_value_set(source, file, object, output),
        "CodeSystem" => extract_code_system(source, file, object, output),
        _ => Ok(()),
    }
}

fn extract_structure_definition(
    source: &ContextArtifactIdentity,
    file: &str,
    object: &Map<String, Value>,
    output: &mut Vec<PendingCanonicalReference>,
) -> Result<(), ContextGraphError> {
    if let Some(canonical) = optional_string(object, "baseDefinition", file, "baseDefinition")? {
        push_reference(
            output,
            source,
            CanonicalReferenceRelation::StructureBaseDefinition,
            "baseDefinition".to_owned(),
            None,
            canonical,
        );
    }

    let Some(differential) = optional_object(object, "differential", file, "differential")? else {
        return Ok(());
    };
    let elements = required_array(differential, "element", file, "differential.element")?;
    for (element_index, element) in elements.iter().enumerate() {
        let element_path = format!("differential.element[{element_index}]");
        let element = element
            .as_object()
            .ok_or_else(|| invalid_field(file, &element_path, "a JSON object"))?;
        let element_id = required_string(element, "id", file, &format!("{element_path}.id"))?;

        if let Some(types) = optional_array(element, "type", file, &format!("{element_path}.type"))?
        {
            for (type_index, type_value) in types.iter().enumerate() {
                let type_path = format!("{element_path}.type[{type_index}]");
                let type_object = type_value
                    .as_object()
                    .ok_or_else(|| invalid_field(file, &type_path, "a JSON object"))?;
                extract_canonical_array(
                    output,
                    source,
                    type_object,
                    "profile",
                    file,
                    &type_path,
                    &element_id,
                    CanonicalReferenceRelation::StructureTypeProfile,
                )?;
                extract_canonical_array(
                    output,
                    source,
                    type_object,
                    "targetProfile",
                    file,
                    &type_path,
                    &element_id,
                    CanonicalReferenceRelation::StructureTypeTargetProfile,
                )?;
            }
        }

        if let Some(binding) =
            optional_object(element, "binding", file, &format!("{element_path}.binding"))?
        {
            if let Some(canonical) = optional_string(
                binding,
                "valueSet",
                file,
                &format!("{element_path}.binding.valueSet"),
            )? {
                push_reference(
                    output,
                    source,
                    CanonicalReferenceRelation::StructureBindingValueSet,
                    format!("{element_path}.binding.valueSet"),
                    Some(element_id.clone()),
                    canonical,
                );
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn extract_canonical_array(
    output: &mut Vec<PendingCanonicalReference>,
    source: &ContextArtifactIdentity,
    object: &Map<String, Value>,
    field: &str,
    file: &str,
    parent_path: &str,
    element_id: &str,
    relation: CanonicalReferenceRelation,
) -> Result<(), ContextGraphError> {
    let field_path = format!("{parent_path}.{field}");
    let Some(values) = optional_array(object, field, file, &field_path)? else {
        return Ok(());
    };
    for (index, value) in values.iter().enumerate() {
        let source_path = format!("{field_path}[{index}]");
        let canonical = value
            .as_str()
            .ok_or_else(|| invalid_field(file, &source_path, "a string"))?;
        push_reference(
            output,
            source,
            relation,
            source_path,
            Some(element_id.to_owned()),
            canonical.to_owned(),
        );
    }
    Ok(())
}

fn extract_value_set(
    source: &ContextArtifactIdentity,
    file: &str,
    object: &Map<String, Value>,
    output: &mut Vec<PendingCanonicalReference>,
) -> Result<(), ContextGraphError> {
    let Some(compose) = optional_object(object, "compose", file, "compose")? else {
        return Ok(());
    };
    for (field, system_relation, value_set_relation) in [
        (
            "include",
            CanonicalReferenceRelation::ValueSetIncludeSystem,
            CanonicalReferenceRelation::ValueSetIncludeValueSet,
        ),
        (
            "exclude",
            CanonicalReferenceRelation::ValueSetExcludeSystem,
            CanonicalReferenceRelation::ValueSetExcludeValueSet,
        ),
    ] {
        let field_path = format!("compose.{field}");
        let Some(clauses) = optional_array(compose, field, file, &field_path)? else {
            continue;
        };
        for (clause_index, clause) in clauses.iter().enumerate() {
            let clause_path = format!("{field_path}[{clause_index}]");
            let clause = clause
                .as_object()
                .ok_or_else(|| invalid_field(file, &clause_path, "a JSON object"))?;
            if let Some(system) =
                optional_string(clause, "system", file, &format!("{clause_path}.system"))?
            {
                push_reference(
                    output,
                    source,
                    system_relation,
                    format!("{clause_path}.system"),
                    None,
                    system,
                );
            }
            let value_set_path = format!("{clause_path}.valueSet");
            if let Some(imports) = optional_array(clause, "valueSet", file, &value_set_path)? {
                for (import_index, import) in imports.iter().enumerate() {
                    let source_path = format!("{value_set_path}[{import_index}]");
                    let canonical = import
                        .as_str()
                        .ok_or_else(|| invalid_field(file, &source_path, "a string"))?;
                    push_reference(
                        output,
                        source,
                        value_set_relation,
                        source_path,
                        None,
                        canonical.to_owned(),
                    );
                }
            }
        }
    }
    Ok(())
}

fn extract_code_system(
    source: &ContextArtifactIdentity,
    file: &str,
    object: &Map<String, Value>,
    output: &mut Vec<PendingCanonicalReference>,
) -> Result<(), ContextGraphError> {
    if let Some(canonical) = optional_string(object, "supplements", file, "supplements")? {
        push_reference(
            output,
            source,
            CanonicalReferenceRelation::CodeSystemSupplements,
            "supplements".to_owned(),
            None,
            canonical,
        );
    }
    Ok(())
}

fn push_reference(
    output: &mut Vec<PendingCanonicalReference>,
    source: &ContextArtifactIdentity,
    relation: CanonicalReferenceRelation,
    source_path: String,
    source_element_id: Option<String>,
    canonical: String,
) {
    output.push(PendingCanonicalReference {
        source: source.clone(),
        relation,
        source_path,
        source_element_id,
        canonical,
    });
}

fn optional_string(
    object: &Map<String, Value>,
    field: &str,
    file: &str,
    path: &str,
) -> Result<Option<String>, ContextGraphError> {
    match object.get(field) {
        None => Ok(None),
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(invalid_field(file, path, "a string")),
    }
}

fn required_string(
    object: &Map<String, Value>,
    field: &str,
    file: &str,
    path: &str,
) -> Result<String, ContextGraphError> {
    optional_string(object, field, file, path)?.ok_or_else(|| invalid_field(file, path, "a string"))
}

fn optional_array<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    file: &str,
    path: &str,
) -> Result<Option<&'a Vec<Value>>, ContextGraphError> {
    match object.get(field) {
        None => Ok(None),
        Some(Value::Array(values)) => Ok(Some(values)),
        Some(_) => Err(invalid_field(file, path, "an array")),
    }
}

fn required_array<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    file: &str,
    path: &str,
) -> Result<&'a Vec<Value>, ContextGraphError> {
    optional_array(object, field, file, path)?.ok_or_else(|| invalid_field(file, path, "an array"))
}

fn optional_object<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    file: &str,
    path: &str,
) -> Result<Option<&'a Map<String, Value>>, ContextGraphError> {
    match object.get(field) {
        None => Ok(None),
        Some(Value::Object(value)) => Ok(Some(value)),
        Some(_) => Err(invalid_field(file, path, "a JSON object")),
    }
}

fn invalid_field(file: &str, path: &str, expected: &'static str) -> ContextGraphError {
    ContextGraphError::InvalidResourceField {
        file: file.to_owned(),
        path: path.to_owned(),
        expected,
    }
}
