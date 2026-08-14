use std::collections::{BTreeMap, BTreeSet};

use crate::{
    Hl7OracleReport, OracleDivergenceReport, OracleError, OracleIdentity, OracleResourceIdentity,
    OracleResourceResult, OracleResourceStatus, ResourceKey, ResourceKeyKind, StructuralChangeKind,
    StructuralDiffReport, HL7_ORACLE_PROJECT, HL7_ORACLE_RELEASE, HL7_ORACLE_SOURCE_COMMIT,
};

const MAX_MESSAGES: usize = 100_000;
const MAX_STRING_BYTES: usize = 64 * 1024;

pub fn parse_hl7_oracle_report(bytes: &[u8]) -> Result<Hl7OracleReport, OracleError> {
    let mut report: Hl7OracleReport = serde_json::from_slice(bytes)?;
    canonicalize_messages(&mut report);
    validate_hl7_oracle_report(&report)?;
    Ok(report)
}

pub fn validate_hl7_oracle_report(report: &Hl7OracleReport) -> Result<(), OracleError> {
    if report.schema != Hl7OracleReport::SCHEMA_V1 {
        return Err(OracleError::UnsupportedSchema {
            actual: report.schema,
            expected: Hl7OracleReport::SCHEMA_V1,
        });
    }
    validate_identity(&report.oracle)?;
    validate_resource_identity("left", &report.left)?;
    validate_resource_identity("right", &report.right)?;
    if report.messages.len() > MAX_MESSAGES {
        return Err(OracleError::EvidenceLimit {
            field: "messages",
            actual: report.messages.len(),
            limit: MAX_MESSAGES,
        });
    }
    for message in &report.messages {
        validate_string("message.location", &message.location)?;
        validate_string("message.message", &message.message)?;
    }
    Ok(())
}

pub fn reconcile_hl7_oracle(
    structural_diff: StructuralDiffReport,
    observations: Vec<(ResourceKey, Hl7OracleReport)>,
) -> Result<OracleDivergenceReport, OracleError> {
    let mut changes_by_resource = BTreeMap::<ResourceKey, BTreeSet<StructuralChangeKind>>::new();
    let mut one_sided = BTreeSet::<ResourceKey>::new();
    for change in &structural_diff.changes {
        changes_by_resource
            .entry(change.resource.clone())
            .or_default()
            .insert(change.kind);
        if matches!(
            change.kind,
            StructuralChangeKind::ResourceAdded | StructuralChangeKind::ResourceRemoved
        ) {
            one_sided.insert(change.resource.clone());
        }
    }

    let mut observation_map = BTreeMap::<ResourceKey, Hl7OracleReport>::new();
    for (resource, mut observation) in observations {
        canonicalize_messages(&mut observation);
        validate_hl7_oracle_report(&observation)?;
        validate_observation_resource(&resource, &observation)?;
        if observation_map.insert(resource.clone(), observation).is_some() {
            return Err(OracleError::DuplicateObservation {
                resource: resource.value,
            });
        }
    }

    let keys = changes_by_resource
        .keys()
        .chain(observation_map.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut resources = Vec::with_capacity(keys.len());

    for resource in keys {
        let kinds = changes_by_resource
            .get(&resource)
            .map(|kinds| kinds.iter().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        let observation = observation_map.remove(&resource);
        let status = if one_sided.contains(&resource) {
            OracleResourceStatus::Uncomparable
        } else if let Some(oracle) = observation.as_ref() {
            let commandf_changed = !kinds.is_empty();
            let oracle_changed = oracle.has_change_signal();
            match (commandf_changed, oracle_changed) {
                (false, false) => OracleResourceStatus::Agreement,
                (true, false) => OracleResourceStatus::CommandfOnly,
                (false, true) => OracleResourceStatus::AuthorityOnly,
                (true, true) => OracleResourceStatus::BothChanged,
            }
        } else {
            OracleResourceStatus::Uncomparable
        };
        resources.push(OracleResourceResult {
            resource,
            status,
            oracle: observation,
            commandf_change_kinds: kinds,
        });
    }

    Ok(OracleDivergenceReport {
        schema: OracleDivergenceReport::SCHEMA_V1,
        oracle: OracleIdentity::pinned_hl7(),
        package_name: structural_diff.package_name.clone(),
        structural_diff,
        resources,
    })
}

fn validate_observation_resource(
    resource: &ResourceKey,
    observation: &Hl7OracleReport,
) -> Result<(), OracleError> {
    let left = display_identity(&observation.left);
    let right = display_identity(&observation.right);
    let valid = resource.kind == ResourceKeyKind::Canonical
        && identity_matches_key(&observation.left, resource)
        && identity_matches_key(&observation.right, resource);
    if !valid {
        return Err(OracleError::ObservationIdentityMismatch {
            resource: resource.value.clone(),
            left,
            right,
        });
    }
    Ok(())
}

fn identity_matches_key(identity: &OracleResourceIdentity, resource: &ResourceKey) -> bool {
    let Some(url) = identity.url.as_deref().map(str::trim).filter(|value| !value.is_empty()) else {
        return false;
    };
    if url == resource.value {
        return true;
    }
    identity
        .canonical_identity()
        .as_deref()
        .is_some_and(|canonical| canonical == resource.value)
}

fn display_identity(identity: &OracleResourceIdentity) -> String {
    identity
        .canonical_identity()
        .or_else(|| identity.id.clone())
        .unwrap_or_else(|| "<missing>".to_owned())
}

fn canonicalize_messages(report: &mut Hl7OracleReport) {
    report.messages.sort();
    report.messages.dedup();
}

fn validate_identity(identity: &OracleIdentity) -> Result<(), OracleError> {
    for (field, expected, actual) in [
        ("project", HL7_ORACLE_PROJECT, identity.project.as_str()),
        ("release", HL7_ORACLE_RELEASE, identity.release.as_str()),
        (
            "source_commit",
            HL7_ORACLE_SOURCE_COMMIT,
            identity.source_commit.as_str(),
        ),
    ] {
        if actual != expected {
            return Err(OracleError::IdentityMismatch {
                field,
                expected,
                actual: actual.to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_resource_identity(
    side: &'static str,
    identity: &OracleResourceIdentity,
) -> Result<(), OracleError> {
    let url = identity
        .url
        .as_deref()
        .ok_or(OracleError::EmptyField { field: side })?;
    if url.trim().is_empty() {
        return Err(OracleError::EmptyField { field: side });
    }
    validate_string(side, url)?;
    for value in [
        identity.version.as_deref(),
        identity.id.as_deref(),
        identity.resource_type.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        validate_string(side, value)?;
    }
    Ok(())
}

fn validate_string(field: &'static str, value: &str) -> Result<(), OracleError> {
    if value.len() > MAX_STRING_BYTES {
        return Err(OracleError::EvidenceLimit {
            field,
            actual: value.len(),
            limit: MAX_STRING_BYTES,
        });
    }
    Ok(())
}
