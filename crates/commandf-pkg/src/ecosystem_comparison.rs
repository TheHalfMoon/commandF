use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use thiserror::Error;

use crate::{
    query_closures, verify_lifecycle_record, verify_snapshot_identity, ClosureError,
    EcosystemClosures, EcosystemSnapshot, LifecycleError, LifecycleRecord, PackageCache,
    PackageError, SnapshotError, SnapshotPackage, CANONICAL_AMBIGUOUS, CANONICAL_RESOLVED,
    CANONICAL_UNRESOLVED,
};

pub const ECOSYSTEM_COMPARISON_SCHEMA: &str = "commandf.ecosystem-snapshot-comparison/v1";
pub const EVIDENCE_PRESENT: &str = "PRESENT";
pub const EVIDENCE_ABSENT: &str = "ABSENT";
pub const STATUS_ABSENT: &str = "ABSENT";
pub const MAX_COMPARISON_RECORDS: usize = 10_000;
pub const MAX_COMPARISON_ENGINE_CHARS: usize = 256;
pub const MAX_COMPARISON_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PackageMembership {
    pub archive_sha256: String,
    pub mutability: String,
    pub name: String,
    pub source_id: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PackageChange {
    pub after_archive_sha256: String,
    pub after_mutability: String,
    pub after_source_id: String,
    pub before_archive_sha256: String,
    pub before_mutability: String,
    pub before_source_id: String,
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResolutionChange {
    pub after_status: String,
    pub before_status: String,
    pub canonical: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LifecycleStateChange {
    pub after_state: String,
    pub before_state: String,
    pub source_id: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ComparisonWitnesses<'a> {
    pub before_closures: Option<&'a EcosystemClosures>,
    pub after_closures: Option<&'a EcosystemClosures>,
    pub before_lifecycle: Option<&'a LifecycleRecord>,
    pub after_lifecycle: Option<&'a LifecycleRecord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnapshotComparison {
    pub schema: String,
    pub engine_schema: String,
    pub before_snapshot_sha256: String,
    pub after_snapshot_sha256: String,
    pub added_packages: Vec<PackageMembership>,
    pub removed_packages: Vec<PackageMembership>,
    pub changed_packages: Vec<PackageChange>,
    pub unchanged_packages: Vec<PackageMembership>,
    pub introduced_unresolved_canonicals: Vec<String>,
    pub removed_unresolved_canonicals: Vec<String>,
    pub retained_unresolved_canonicals: Vec<String>,
    pub closure_evidence: String,
    pub before_package_closure_sha256: String,
    pub after_package_closure_sha256: String,
    pub before_canonical_closure_sha256: String,
    pub after_canonical_closure_sha256: String,
    pub resolution_evidence: String,
    pub resolution_changes: Vec<ResolutionChange>,
    pub lifecycle_evidence: String,
    pub lifecycle_states: Vec<LifecycleStateChange>,
    pub comparison_sha256: String,
}

#[derive(Debug, Error)]
pub enum ComparisonError {
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Closure(#[from] ClosureError),

    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),

    #[error(transparent)]
    Package(#[from] PackageError),

    #[error("comparison engine schema must be a non-empty token without whitespace")]
    InvalidEngineSchema,

    #[error("comparison engine schema exceeds {MAX_COMPARISON_ENGINE_CHARS} characters")]
    EngineSchemaTooLong,

    #[error("comparison record count exceeds {MAX_COMPARISON_RECORDS}")]
    TooManyRecords,

    #[error("comparison output exceeds {MAX_COMPARISON_OUTPUT_BYTES} bytes")]
    OutputTooLarge,

    #[error("closure evidence must be supplied for both snapshots or neither")]
    OneSidedClosures,

    #[error("lifecycle evidence must be supplied for both snapshots or neither")]
    OneSidedLifecycle,

    #[error("stored comparison does not match the recomputed comparison")]
    ComparisonMismatch,
}

pub fn project_snapshot_comparison(
    before: &EcosystemSnapshot,
    after: &EcosystemSnapshot,
    engine_schema: &str,
    witnesses: ComparisonWitnesses<'_>,
) -> Result<SnapshotComparison, ComparisonError> {
    validate_engine_schema(engine_schema)?;
    if exceeds_record_bound(before)
        || exceeds_record_bound(after)
        || option_exceeds(witnesses.before_closures)
        || option_exceeds(witnesses.after_closures)
        || lifecycle_exceeds(witnesses.before_lifecycle)
        || lifecycle_exceeds(witnesses.after_lifecycle)
    {
        return Err(ComparisonError::TooManyRecords);
    }

    verify_snapshot_identity(before)?;
    verify_snapshot_identity(after)?;
    crate::require_published_authority(before)?;
    crate::require_published_authority(after)?;

    let closure = closure_delta(
        before,
        after,
        witnesses.before_closures,
        witnesses.after_closures,
    )?;
    let closure_evidence = closure.evidence;
    let before_package = closure.before_package_closure_sha256;
    let after_package = closure.after_package_closure_sha256;
    let before_canonical = closure.before_canonical_closure_sha256;
    let after_canonical = closure.after_canonical_closure_sha256;
    let resolution_evidence = closure.resolution_evidence;
    let resolution_changes = closure.resolution_changes;
    let (lifecycle_evidence, lifecycle_states) = lifecycle_delta(
        before,
        after,
        witnesses.before_lifecycle,
        witnesses.after_lifecycle,
    )?;

    let before_packages = index_packages(&before.packages);
    let after_packages = index_packages(&after.packages);
    let mut added_packages = Vec::new();
    let mut removed_packages = Vec::new();
    let mut changed_packages = Vec::new();
    let mut unchanged_packages = Vec::new();
    for (identity, before_package) in &before_packages {
        match after_packages.get(identity) {
            None => removed_packages.push(membership(before_package)),
            Some(after_package) if same_package(before_package, after_package) => {
                unchanged_packages.push(membership(before_package));
            }
            Some(after_package) => changed_packages.push(PackageChange {
                after_archive_sha256: after_package.archive_sha256.clone(),
                after_mutability: after_package.mutability.clone(),
                after_source_id: after_package.source_id.clone(),
                before_archive_sha256: before_package.archive_sha256.clone(),
                before_mutability: before_package.mutability.clone(),
                before_source_id: before_package.source_id.clone(),
                name: before_package.name.clone(),
                version: before_package.version.clone(),
            }),
        }
    }
    for (identity, after_package) in &after_packages {
        if !before_packages.contains_key(identity) {
            added_packages.push(membership(after_package));
        }
    }

    let before_unresolved: BTreeSet<&str> = before
        .unresolved_canonicals
        .iter()
        .map(String::as_str)
        .collect();
    let after_unresolved: BTreeSet<&str> = after
        .unresolved_canonicals
        .iter()
        .map(String::as_str)
        .collect();
    let introduced_unresolved_canonicals: Vec<String> = after_unresolved
        .difference(&before_unresolved)
        .map(|value| (*value).to_owned())
        .collect();
    let removed_unresolved_canonicals: Vec<String> = before_unresolved
        .difference(&after_unresolved)
        .map(|value| (*value).to_owned())
        .collect();
    let retained_unresolved_canonicals: Vec<String> = before_unresolved
        .intersection(&after_unresolved)
        .map(|value| (*value).to_owned())
        .collect();

    let body = ComparisonBody {
        added_packages: &added_packages,
        after_canonical_closure_sha256: &after_canonical,
        after_package_closure_sha256: &after_package,
        after_snapshot_sha256: &after.snapshot_sha256,
        before_canonical_closure_sha256: &before_canonical,
        before_package_closure_sha256: &before_package,
        before_snapshot_sha256: &before.snapshot_sha256,
        changed_packages: &changed_packages,
        closure_evidence: &closure_evidence,
        engine_schema,
        introduced_unresolved_canonicals: &introduced_unresolved_canonicals,
        lifecycle_evidence: &lifecycle_evidence,
        lifecycle_states: &lifecycle_states,
        removed_packages: &removed_packages,
        removed_unresolved_canonicals: &removed_unresolved_canonicals,
        resolution_changes: &resolution_changes,
        resolution_evidence: &resolution_evidence,
        retained_unresolved_canonicals: &retained_unresolved_canonicals,
        schema: ECOSYSTEM_COMPARISON_SCHEMA,
        unchanged_packages: &unchanged_packages,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    if bytes.len() > MAX_COMPARISON_OUTPUT_BYTES {
        return Err(ComparisonError::OutputTooLarge);
    }
    Ok(SnapshotComparison {
        schema: ECOSYSTEM_COMPARISON_SCHEMA.to_owned(),
        engine_schema: engine_schema.to_owned(),
        before_snapshot_sha256: before.snapshot_sha256.clone(),
        after_snapshot_sha256: after.snapshot_sha256.clone(),
        added_packages,
        removed_packages,
        changed_packages,
        unchanged_packages,
        introduced_unresolved_canonicals,
        removed_unresolved_canonicals,
        retained_unresolved_canonicals,
        closure_evidence,
        before_package_closure_sha256: before_package,
        after_package_closure_sha256: after_package,
        before_canonical_closure_sha256: before_canonical,
        after_canonical_closure_sha256: after_canonical,
        resolution_evidence,
        resolution_changes,
        lifecycle_evidence,
        lifecycle_states,
        comparison_sha256: PackageCache::digest(&bytes),
    })
}

pub fn require_comparison_replay(
    before: &EcosystemSnapshot,
    after: &EcosystemSnapshot,
    engine_schema: &str,
    witnesses: ComparisonWitnesses<'_>,
    comparison: &SnapshotComparison,
) -> Result<(), ComparisonError> {
    let projected = project_snapshot_comparison(before, after, engine_schema, witnesses)?;
    if &projected != comparison {
        return Err(ComparisonError::ComparisonMismatch);
    }
    Ok(())
}

#[derive(Serialize)]
struct ComparisonBody<'a> {
    added_packages: &'a [PackageMembership],
    after_canonical_closure_sha256: &'a str,
    after_package_closure_sha256: &'a str,
    after_snapshot_sha256: &'a str,
    before_canonical_closure_sha256: &'a str,
    before_package_closure_sha256: &'a str,
    before_snapshot_sha256: &'a str,
    changed_packages: &'a [PackageChange],
    closure_evidence: &'a str,
    engine_schema: &'a str,
    introduced_unresolved_canonicals: &'a [String],
    lifecycle_evidence: &'a str,
    lifecycle_states: &'a [LifecycleStateChange],
    removed_packages: &'a [PackageMembership],
    removed_unresolved_canonicals: &'a [String],
    resolution_changes: &'a [ResolutionChange],
    resolution_evidence: &'a str,
    retained_unresolved_canonicals: &'a [String],
    schema: &'a str,
    unchanged_packages: &'a [PackageMembership],
}

struct ClosureFacts {
    evidence: String,
    before_package_closure_sha256: String,
    after_package_closure_sha256: String,
    before_canonical_closure_sha256: String,
    after_canonical_closure_sha256: String,
    resolution_evidence: String,
    resolution_changes: Vec<ResolutionChange>,
}

fn closure_delta(
    before: &EcosystemSnapshot,
    after: &EcosystemSnapshot,
    before_closures: Option<&EcosystemClosures>,
    after_closures: Option<&EcosystemClosures>,
) -> Result<ClosureFacts, ComparisonError> {
    match (before_closures, after_closures) {
        (None, None) => Ok(ClosureFacts {
            evidence: EVIDENCE_ABSENT.to_owned(),
            before_package_closure_sha256: String::new(),
            after_package_closure_sha256: String::new(),
            before_canonical_closure_sha256: String::new(),
            after_canonical_closure_sha256: String::new(),
            resolution_evidence: EVIDENCE_ABSENT.to_owned(),
            resolution_changes: Vec::new(),
        }),
        (Some(before_closures), Some(after_closures)) => {
            query_closures(before, before_closures)?;
            query_closures(after, after_closures)?;
            let mut changes = Vec::new();
            let mut canonicals = BTreeSet::new();
            collect_resolution_targets(before_closures, &mut canonicals);
            collect_resolution_targets(after_closures, &mut canonicals);
            for canonical in canonicals {
                let before_status = resolution_status(before_closures, canonical);
                let after_status = resolution_status(after_closures, canonical);
                if before_status != after_status {
                    changes.push(ResolutionChange {
                        after_status: after_status.to_owned(),
                        before_status: before_status.to_owned(),
                        canonical: canonical.to_owned(),
                    });
                }
            }
            Ok(ClosureFacts {
                evidence: EVIDENCE_PRESENT.to_owned(),
                before_package_closure_sha256: before_closures.package_closure_sha256.clone(),
                after_package_closure_sha256: after_closures.package_closure_sha256.clone(),
                before_canonical_closure_sha256: before_closures.canonical_closure_sha256.clone(),
                after_canonical_closure_sha256: after_closures.canonical_closure_sha256.clone(),
                resolution_evidence: EVIDENCE_PRESENT.to_owned(),
                resolution_changes: changes,
            })
        }
        _ => Err(ComparisonError::OneSidedClosures),
    }
}

fn lifecycle_delta(
    before: &EcosystemSnapshot,
    after: &EcosystemSnapshot,
    before_lifecycle: Option<&LifecycleRecord>,
    after_lifecycle: Option<&LifecycleRecord>,
) -> Result<(String, Vec<LifecycleStateChange>), ComparisonError> {
    match (before_lifecycle, after_lifecycle) {
        (None, None) => Ok((EVIDENCE_ABSENT.to_owned(), Vec::new())),
        (Some(before_lifecycle), Some(after_lifecycle)) => {
            verify_lifecycle_record(before, before_lifecycle)?;
            verify_lifecycle_record(after, after_lifecycle)?;
            let mut sources = BTreeSet::new();
            for source in before_lifecycle
                .sources
                .iter()
                .chain(after_lifecycle.sources.iter())
            {
                sources.insert(source.source_id.as_str());
            }
            let states = sources
                .into_iter()
                .map(|source_id| LifecycleStateChange {
                    after_state: lifecycle_state(after_lifecycle, source_id).to_owned(),
                    before_state: lifecycle_state(before_lifecycle, source_id).to_owned(),
                    source_id: source_id.to_owned(),
                })
                .collect();
            Ok((EVIDENCE_PRESENT.to_owned(), states))
        }
        _ => Err(ComparisonError::OneSidedLifecycle),
    }
}

fn index_packages(packages: &[SnapshotPackage]) -> BTreeMap<(&str, &str), &SnapshotPackage> {
    let mut index = BTreeMap::new();
    for package in packages {
        index.insert((package.name.as_str(), package.version.as_str()), package);
    }
    index
}

fn membership(package: &SnapshotPackage) -> PackageMembership {
    PackageMembership {
        archive_sha256: package.archive_sha256.clone(),
        mutability: package.mutability.clone(),
        name: package.name.clone(),
        source_id: package.source_id.clone(),
        version: package.version.clone(),
    }
}

fn same_package(left: &SnapshotPackage, right: &SnapshotPackage) -> bool {
    left.archive_sha256 == right.archive_sha256
        && left.source_id == right.source_id
        && left.mutability == right.mutability
}

fn collect_resolution_targets<'a>(
    closures: &'a EcosystemClosures,
    targets: &mut BTreeSet<&'a str>,
) {
    for canonical in closures
        .resolved_canonicals
        .iter()
        .chain(closures.unresolved_canonicals.iter())
        .chain(closures.ambiguous_canonicals.iter())
    {
        targets.insert(canonical.as_str());
    }
}

fn resolution_status(closures: &EcosystemClosures, canonical: &str) -> &'static str {
    if closures
        .resolved_canonicals
        .iter()
        .any(|value| value == canonical)
    {
        CANONICAL_RESOLVED
    } else if closures
        .unresolved_canonicals
        .iter()
        .any(|value| value == canonical)
    {
        CANONICAL_UNRESOLVED
    } else if closures
        .ambiguous_canonicals
        .iter()
        .any(|value| value == canonical)
    {
        CANONICAL_AMBIGUOUS
    } else {
        STATUS_ABSENT
    }
}

fn lifecycle_state<'a>(record: &'a LifecycleRecord, source_id: &str) -> &'a str {
    record
        .sources
        .iter()
        .find(|source| source.source_id == source_id)
        .map(|source| source.state.as_str())
        .unwrap_or(STATUS_ABSENT)
}

fn exceeds_record_bound(snapshot: &EcosystemSnapshot) -> bool {
    snapshot.packages.len() > MAX_COMPARISON_RECORDS
        || snapshot.unresolved_canonicals.len() > MAX_COMPARISON_RECORDS
}

fn option_exceeds(closures: Option<&EcosystemClosures>) -> bool {
    closures.is_some_and(|closures| {
        closures.package_members.len() > MAX_COMPARISON_RECORDS
            || closures.unrelated_packages.len() > MAX_COMPARISON_RECORDS
            || closures.package_edges.len() > MAX_COMPARISON_RECORDS
            || closures.canonical_edges.len() > MAX_COMPARISON_RECORDS
            || closures.resolved_canonicals.len() > MAX_COMPARISON_RECORDS
            || closures.unresolved_canonicals.len() > MAX_COMPARISON_RECORDS
            || closures.ambiguous_canonicals.len() > MAX_COMPARISON_RECORDS
    })
}

fn lifecycle_exceeds(record: Option<&LifecycleRecord>) -> bool {
    record.is_some_and(|record| record.sources.len() > MAX_COMPARISON_RECORDS)
}

fn validate_engine_schema(value: &str) -> Result<(), ComparisonError> {
    if value.chars().count() > MAX_COMPARISON_ENGINE_CHARS {
        return Err(ComparisonError::EngineSchemaTooLong);
    }
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_whitespace) {
        return Err(ComparisonError::InvalidEngineSchema);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        project_closures, project_snapshot, project_source_lifecycle, CanonicalReferenceEdge,
        PackageDependencyEdge, SnapshotPackage, CANONICAL_RESOLVED, CANONICAL_UNRESOLVED,
        IMMUTABLE_RELEASE, LIFECYCLE_CURRENT, LIFECYCLE_STALE, MUTABLE_CI,
    };

    fn package(name: &str, version: &str, digest_byte: u8, source_id: &str) -> SnapshotPackage {
        SnapshotPackage {
            name: name.to_owned(),
            version: version.to_owned(),
            archive_sha256: format!("{digest_byte:02x}").repeat(32),
            source_id: source_id.to_owned(),
            mutability: IMMUTABLE_RELEASE.to_owned(),
        }
    }

    fn snapshot(packages: Vec<SnapshotPackage>, unresolved: Vec<&str>) -> EcosystemSnapshot {
        project_snapshot(
            packages,
            unresolved.into_iter().map(str::to_owned).collect(),
        )
        .expect("snapshot")
    }

    fn before() -> EcosystemSnapshot {
        snapshot(
            vec![
                package("acme.kept", "1.0.0", 0x11, "mirror-a"),
                package("acme.gone", "1.0.0", 0x22, "mirror-a"),
            ],
            vec!["http://example.org/kept", "http://example.org/gone"],
        )
    }

    fn after() -> EcosystemSnapshot {
        snapshot(
            vec![
                package("acme.kept", "1.0.0", 0x11, "mirror-a"),
                package("acme.new", "1.0.0", 0x33, "mirror-a"),
            ],
            vec!["http://example.org/kept", "http://example.org/new"],
        )
    }

    #[test]
    fn replay_is_byte_identical_and_direction_sensitive() {
        let left = project_snapshot_comparison(
            &before(),
            &after(),
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
        )
        .expect("left");
        let right = project_snapshot_comparison(
            &before(),
            &after(),
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
        )
        .expect("right");
        assert_eq!(left, right);
        assert_eq!(left.comparison_sha256, right.comparison_sha256);
        let reversed = project_snapshot_comparison(
            &after(),
            &before(),
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
        )
        .expect("reversed");
        assert_ne!(left.comparison_sha256, reversed.comparison_sha256);
        assert_eq!(left.added_packages[0].name, "acme.new");
        assert_eq!(left.removed_packages[0].name, "acme.gone");
        assert_eq!(reversed.added_packages[0].name, "acme.gone");
        assert_eq!(reversed.removed_packages[0].name, "acme.new");
    }

    #[test]
    fn self_comparison_has_no_membership_changes() {
        let published = before();
        let comparison = project_snapshot_comparison(
            &published,
            &published,
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
        )
        .expect("self");
        assert!(comparison.added_packages.is_empty());
        assert!(comparison.removed_packages.is_empty());
        assert!(comparison.changed_packages.is_empty());
        assert_eq!(comparison.unchanged_packages.len(), 2);
        assert_eq!(
            comparison.retained_unresolved_canonicals,
            vec![
                "http://example.org/gone".to_owned(),
                "http://example.org/kept".to_owned()
            ]
        );
        assert_eq!(comparison.resolution_evidence, EVIDENCE_ABSENT);
        assert!(comparison.resolution_changes.is_empty());
    }

    #[test]
    fn package_and_canonical_membership_are_recorded_once() {
        let mut changed_after = after();
        changed_after.packages[0].archive_sha256 = "aa".repeat(32);
        changed_after =
            project_snapshot(changed_after.packages, changed_after.unresolved_canonicals)
                .expect("changed after");
        let comparison = project_snapshot_comparison(
            &before(),
            &changed_after,
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
        )
        .expect("comparison");
        assert_eq!(comparison.added_packages.len(), 1);
        assert_eq!(comparison.removed_packages.len(), 1);
        assert_eq!(comparison.changed_packages.len(), 1);
        assert_eq!(comparison.changed_packages[0].name, "acme.kept");
        assert_eq!(comparison.introduced_unresolved_canonicals.len(), 1);
        assert_eq!(comparison.removed_unresolved_canonicals.len(), 1);
        assert_eq!(comparison.retained_unresolved_canonicals.len(), 1);
        let other_engine = project_snapshot_comparison(
            &before(),
            &changed_after,
            "commandf.snapshot-comparison/v2",
            ComparisonWitnesses::default(),
        )
        .expect("other engine");
        assert_ne!(comparison.comparison_sha256, other_engine.comparison_sha256);
    }

    #[test]
    fn ordering_does_not_change_output_and_tamper_fails() {
        let mut reordered = before();
        reordered.packages.reverse();
        reordered.unresolved_canonicals.reverse();
        reordered = project_snapshot(reordered.packages, reordered.unresolved_canonicals)
            .expect("reordered");
        let forward = project_snapshot_comparison(
            &before(),
            &after(),
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
        )
        .expect("forward");
        let reordered_comparison = project_snapshot_comparison(
            &reordered,
            &after(),
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
        )
        .expect("reordered comparison");
        assert_eq!(forward, reordered_comparison);

        let mut tampered_before = before();
        tampered_before.snapshot_sha256 = "ab".repeat(32);
        assert!(matches!(
            project_snapshot_comparison(
                &tampered_before,
                &after(),
                "commandf.snapshot-comparison/v1",
                ComparisonWitnesses::default()
            ),
            Err(ComparisonError::Snapshot(
                SnapshotError::SnapshotDigestMismatch
            ))
        ));
        let mut tampered_after = after();
        tampered_after.snapshot_sha256 = "cd".repeat(32);
        assert!(matches!(
            project_snapshot_comparison(
                &before(),
                &tampered_after,
                "commandf.snapshot-comparison/v1",
                ComparisonWitnesses::default()
            ),
            Err(ComparisonError::Snapshot(
                SnapshotError::SnapshotDigestMismatch
            ))
        ));
        let mut tampered_comparison = forward.clone();
        tampered_comparison.comparison_sha256 = "ef".repeat(32);
        assert!(require_comparison_replay(
            &before(),
            &after(),
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses::default(),
            &tampered_comparison
        )
        .is_err());
    }

    #[test]
    fn wrong_closure_binding_duplicate_and_bounds_fail_closed() {
        let earlier = before();
        let later = after();
        let earlier_closures = project_closures(
            &earlier,
            Vec::new(),
            vec![CanonicalReferenceEdge {
                source_canonical: "http://example.org/src".to_owned(),
                status: CANONICAL_UNRESOLVED.to_owned(),
                target_canonical: "http://example.org/gone".to_owned(),
            }],
        )
        .expect("earlier closures");
        let later_closures = project_closures(
            &later,
            vec![PackageDependencyEdge {
                from_name: "acme.kept".to_owned(),
                from_version: "1.0.0".to_owned(),
                to_name: "acme.new".to_owned(),
                to_version: "1.0.0".to_owned(),
            }],
            vec![CanonicalReferenceEdge {
                source_canonical: "http://example.org/src".to_owned(),
                status: CANONICAL_RESOLVED.to_owned(),
                target_canonical: "http://example.org/found".to_owned(),
            }],
        )
        .expect("later closures");
        assert!(matches!(
            project_snapshot_comparison(
                &earlier,
                &later,
                "commandf.snapshot-comparison/v1",
                ComparisonWitnesses {
                    before_closures: Some(&later_closures),
                    after_closures: Some(&earlier_closures),
                    ..ComparisonWitnesses::default()
                }
            ),
            Err(ComparisonError::Closure(_))
        ));
        let bound = project_snapshot_comparison(
            &earlier,
            &later,
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses {
                before_closures: Some(&earlier_closures),
                after_closures: Some(&later_closures),
                ..ComparisonWitnesses::default()
            },
        )
        .expect("bound closures");
        assert_eq!(bound.closure_evidence, EVIDENCE_PRESENT);
        assert_ne!(
            bound.before_package_closure_sha256,
            bound.after_package_closure_sha256
        );
        assert!(bound
            .resolution_changes
            .iter()
            .any(|change| change.canonical == "http://example.org/gone"));

        assert!(matches!(
            project_snapshot_comparison(
                &earlier,
                &later,
                "commandf.snapshot-comparison/v1",
                ComparisonWitnesses {
                    before_closures: Some(&earlier_closures),
                    ..ComparisonWitnesses::default()
                }
            ),
            Err(ComparisonError::OneSidedClosures)
        ));

        let mut duplicate = earlier.clone();
        duplicate.packages.push(duplicate.packages[0].clone());
        assert!(matches!(
            project_snapshot_comparison(
                &duplicate,
                &later,
                "commandf.snapshot-comparison/v1",
                ComparisonWitnesses::default()
            ),
            Err(ComparisonError::Snapshot(
                SnapshotError::DuplicatePackage { .. }
            ))
        ));

        let mut huge = earlier.clone();
        huge.packages = vec![huge.packages[0].clone(); MAX_COMPARISON_RECORDS + 1];
        assert!(matches!(
            project_snapshot_comparison(
                &huge,
                &later,
                "commandf.snapshot-comparison/v1",
                ComparisonWitnesses::default()
            ),
            Err(ComparisonError::TooManyRecords)
        ));
    }

    #[test]
    fn lifecycle_facts_are_recorded_without_inferring_time() {
        let earlier = before();
        let later = snapshot(
            vec![
                package("acme.kept", "1.0.0", 0x11, "mirror-b"),
                package("acme.new", "1.0.0", 0x33, "mirror-b"),
            ],
            vec!["http://example.org/kept"],
        );
        let earlier_life = project_source_lifecycle(
            &earlier,
            vec![crate::SourceLifecycle {
                source_id: "mirror-a".to_owned(),
                state: LIFECYCLE_CURRENT.to_owned(),
            }],
        )
        .expect("earlier life");
        let later_life = project_source_lifecycle(
            &later,
            vec![crate::SourceLifecycle {
                source_id: "mirror-b".to_owned(),
                state: LIFECYCLE_STALE.to_owned(),
            }],
        )
        .expect("later life");
        let comparison = project_snapshot_comparison(
            &earlier,
            &later,
            "commandf.snapshot-comparison/v1",
            ComparisonWitnesses {
                before_lifecycle: Some(&earlier_life),
                after_lifecycle: Some(&later_life),
                ..ComparisonWitnesses::default()
            },
        )
        .expect("lifecycle comparison");
        assert_eq!(comparison.lifecycle_evidence, EVIDENCE_PRESENT);
        assert_eq!(comparison.lifecycle_states.len(), 2);
        assert!(comparison.lifecycle_states.iter().any(|state| {
            state.source_id == "mirror-a" && state.before_state == LIFECYCLE_CURRENT
        }));
        assert!(comparison.lifecycle_states.iter().any(|state| {
            state.source_id == "mirror-b" && state.after_state == LIFECYCLE_STALE
        }));
        let mut mutable_package = package("acme.kept", "1.0.0", 0x11, "mirror-a");
        mutable_package.mutability = MUTABLE_CI.to_owned();
        let mutable =
            project_snapshot(vec![mutable_package], Vec::new()).expect("mutable snapshot");
        assert!(matches!(
            project_snapshot_comparison(
                &mutable,
                &later,
                "commandf.snapshot-comparison/v1",
                ComparisonWitnesses::default()
            ),
            Err(ComparisonError::Snapshot(
                SnapshotError::MutableSourceIsNotPublished { .. }
            ))
        ));
    }
}
