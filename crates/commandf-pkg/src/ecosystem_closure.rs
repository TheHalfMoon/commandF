use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use thiserror::Error;

use crate::{
    require_published_authority, EcosystemSnapshot, PackageCache, PackageError, SnapshotError,
};

pub const ECOSYSTEM_CLOSURE_SCHEMA: &str = "commandf.ecosystem-closure/v1";
pub const CANONICAL_RESOLVED: &str = "RESOLVED";
pub const CANONICAL_UNRESOLVED: &str = "UNRESOLVED";
pub const CANONICAL_AMBIGUOUS: &str = "AMBIGUOUS";
pub const MAX_CLOSURE_EDGES: usize = 10_000;
pub const MAX_CANONICAL_CHARS: usize = 2_048;

const PACKAGE_KIND: &str = "package-dependency";
const CANONICAL_KIND: &str = "canonical-reference";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PackageDependencyEdge {
    pub from_name: String,
    pub from_version: String,
    pub to_name: String,
    pub to_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CanonicalReferenceEdge {
    pub source_canonical: String,
    pub status: String,
    pub target_canonical: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ClosurePackage {
    pub archive_sha256: String,
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EcosystemClosures {
    pub schema: String,
    pub snapshot_sha256: String,
    pub package_members: Vec<ClosurePackage>,
    pub package_edges: Vec<PackageDependencyEdge>,
    pub unrelated_packages: Vec<ClosurePackage>,
    pub resolved_canonicals: Vec<String>,
    pub unresolved_canonicals: Vec<String>,
    pub ambiguous_canonicals: Vec<String>,
    pub canonical_edges: Vec<CanonicalReferenceEdge>,
    pub package_closure_sha256: String,
    pub canonical_closure_sha256: String,
}

#[derive(Debug, Error)]
pub enum ClosureError {
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Package(#[from] PackageError),

    #[error("package dependency endpoint {name}@{version} is not in the snapshot")]
    UnknownPackage { name: String, version: String },

    #[error("package {name}@{version} cannot depend on itself")]
    SelfDependency { name: String, version: String },

    #[error("duplicate package dependency edge")]
    DuplicatePackageEdge,

    #[error("duplicate canonical reference edge")]
    DuplicateCanonicalEdge,

    #[error("canonical {canonical} has conflicting resolution status")]
    ConflictingCanonicalStatus { canonical: String },

    #[error("canonical must be a non-empty token without whitespace")]
    InvalidCanonical,

    #[error("canonical reference status must be RESOLVED, UNRESOLVED, or AMBIGUOUS")]
    InvalidStatus,

    #[error("closure edge count exceeds {MAX_CLOSURE_EDGES}")]
    TooManyEdges,

    #[error("canonical exceeds {MAX_CANONICAL_CHARS} characters")]
    CanonicalTooLong,
}

pub fn project_closures(
    snapshot: &EcosystemSnapshot,
    package_edges: Vec<PackageDependencyEdge>,
    canonical_edges: Vec<CanonicalReferenceEdge>,
) -> Result<EcosystemClosures, ClosureError> {
    if package_edges.len() > MAX_CLOSURE_EDGES || canonical_edges.len() > MAX_CLOSURE_EDGES {
        return Err(ClosureError::TooManyEdges);
    }
    require_published_authority(snapshot)?;

    let mut package_index = BTreeMap::new();
    for package in &snapshot.packages {
        package_index.insert(
            (package.name.as_str(), package.version.as_str()),
            package.archive_sha256.as_str(),
        );
    }

    let mut package_edges = package_edges;
    for edge in &package_edges {
        if !package_index.contains_key(&(edge.from_name.as_str(), edge.from_version.as_str())) {
            return Err(ClosureError::UnknownPackage {
                name: edge.from_name.clone(),
                version: edge.from_version.clone(),
            });
        }
        if !package_index.contains_key(&(edge.to_name.as_str(), edge.to_version.as_str())) {
            return Err(ClosureError::UnknownPackage {
                name: edge.to_name.clone(),
                version: edge.to_version.clone(),
            });
        }
        if edge.from_name == edge.to_name && edge.from_version == edge.to_version {
            return Err(ClosureError::SelfDependency {
                name: edge.from_name.clone(),
                version: edge.from_version.clone(),
            });
        }
    }
    package_edges.sort_by(|left, right| {
        left.from_name
            .cmp(&right.from_name)
            .then_with(|| left.from_version.cmp(&right.from_version))
            .then_with(|| left.to_name.cmp(&right.to_name))
            .then_with(|| left.to_version.cmp(&right.to_version))
    });
    for pair in package_edges.windows(2) {
        if pair[0] == pair[1] {
            return Err(ClosureError::DuplicatePackageEdge);
        }
    }

    let mut members = BTreeSet::new();
    for edge in &package_edges {
        members.insert((edge.from_name.as_str(), edge.from_version.as_str()));
        members.insert((edge.to_name.as_str(), edge.to_version.as_str()));
    }
    let mut package_members = Vec::new();
    let mut unrelated_packages = Vec::new();
    for package in &snapshot.packages {
        let identity = (package.name.as_str(), package.version.as_str());
        let record = ClosurePackage {
            archive_sha256: package.archive_sha256.clone(),
            name: package.name.clone(),
            version: package.version.clone(),
        };
        if members.contains(&identity) {
            package_members.push(record);
        } else {
            unrelated_packages.push(record);
        }
    }

    let mut canonical_edges = canonical_edges;
    for edge in &canonical_edges {
        validate_canonical(&edge.source_canonical)?;
        validate_canonical(&edge.target_canonical)?;
        if edge.status != CANONICAL_RESOLVED
            && edge.status != CANONICAL_UNRESOLVED
            && edge.status != CANONICAL_AMBIGUOUS
        {
            return Err(ClosureError::InvalidStatus);
        }
    }
    canonical_edges.sort_by(|left, right| {
        left.source_canonical
            .cmp(&right.source_canonical)
            .then_with(|| left.target_canonical.cmp(&right.target_canonical))
            .then_with(|| left.status.cmp(&right.status))
    });
    for pair in canonical_edges.windows(2) {
        if pair[0] == pair[1] {
            return Err(ClosureError::DuplicateCanonicalEdge);
        }
    }

    let mut status_by_target: BTreeMap<String, String> = BTreeMap::new();
    for canonical in &snapshot.unresolved_canonicals {
        observe_status(
            &mut status_by_target,
            canonical.clone(),
            CANONICAL_UNRESOLVED,
        )?;
    }
    for edge in &canonical_edges {
        observe_status(
            &mut status_by_target,
            edge.target_canonical.clone(),
            &edge.status,
        )?;
    }
    let mut resolved_canonicals = Vec::new();
    let mut unresolved_canonicals = Vec::new();
    let mut ambiguous_canonicals = Vec::new();
    for (canonical, status) in status_by_target {
        match status.as_str() {
            CANONICAL_RESOLVED => resolved_canonicals.push(canonical),
            CANONICAL_UNRESOLVED => unresolved_canonicals.push(canonical),
            CANONICAL_AMBIGUOUS => ambiguous_canonicals.push(canonical),
            _ => return Err(ClosureError::InvalidStatus),
        }
    }

    let package_body = PackageClosureBody {
        edges: &package_edges,
        kind: PACKAGE_KIND,
        members: &package_members,
        schema: ECOSYSTEM_CLOSURE_SCHEMA,
        snapshot_sha256: &snapshot.snapshot_sha256,
        unrelated_packages: &unrelated_packages,
    };
    let canonical_body = CanonicalClosureBody {
        ambiguous_canonicals: &ambiguous_canonicals,
        edges: &canonical_edges,
        kind: CANONICAL_KIND,
        resolved_canonicals: &resolved_canonicals,
        schema: ECOSYSTEM_CLOSURE_SCHEMA,
        snapshot_sha256: &snapshot.snapshot_sha256,
        unresolved_canonicals: &unresolved_canonicals,
    };
    let package_bytes = serde_json::to_vec(&package_body).map_err(PackageError::Json)?;
    let canonical_bytes = serde_json::to_vec(&canonical_body).map_err(PackageError::Json)?;

    Ok(EcosystemClosures {
        schema: ECOSYSTEM_CLOSURE_SCHEMA.to_owned(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        package_members,
        package_edges,
        unrelated_packages,
        resolved_canonicals,
        unresolved_canonicals,
        ambiguous_canonicals,
        canonical_edges,
        package_closure_sha256: PackageCache::digest(&package_bytes),
        canonical_closure_sha256: PackageCache::digest(&canonical_bytes),
    })
}

fn observe_status(
    statuses: &mut BTreeMap<String, String>,
    canonical: String,
    status: &str,
) -> Result<(), ClosureError> {
    if let Some(existing) = statuses.get(&canonical) {
        if existing != status {
            return Err(ClosureError::ConflictingCanonicalStatus { canonical });
        }
        return Ok(());
    }
    statuses.insert(canonical, status.to_owned());
    Ok(())
}

fn validate_canonical(value: &str) -> Result<(), ClosureError> {
    if value.chars().count() > MAX_CANONICAL_CHARS {
        return Err(ClosureError::CanonicalTooLong);
    }
    if value.is_empty() || value.trim() != value || value.chars().any(char::is_whitespace) {
        return Err(ClosureError::InvalidCanonical);
    }
    Ok(())
}

#[derive(Serialize)]
struct PackageClosureBody<'a> {
    edges: &'a [PackageDependencyEdge],
    kind: &'a str,
    members: &'a [ClosurePackage],
    schema: &'a str,
    snapshot_sha256: &'a str,
    unrelated_packages: &'a [ClosurePackage],
}

#[derive(Serialize)]
struct CanonicalClosureBody<'a> {
    ambiguous_canonicals: &'a [String],
    edges: &'a [CanonicalReferenceEdge],
    kind: &'a str,
    resolved_canonicals: &'a [String],
    schema: &'a str,
    snapshot_sha256: &'a str,
    unresolved_canonicals: &'a [String],
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{project_snapshot, SnapshotPackage, IMMUTABLE_RELEASE, MUTABLE_CI};

    fn package(name: &str, version: &str, digest_byte: u8) -> SnapshotPackage {
        SnapshotPackage {
            name: name.to_owned(),
            version: version.to_owned(),
            archive_sha256: format!("{digest_byte:02x}").repeat(32),
            source_id: "local-mirror".to_owned(),
            mutability: IMMUTABLE_RELEASE.to_owned(),
        }
    }

    fn snapshot() -> EcosystemSnapshot {
        project_snapshot(
            vec![
                package("acme.a", "1.0.0", 0x11),
                package("acme.b", "1.0.0", 0x22),
                package("acme.c", "1.0.0", 0x33),
            ],
            vec!["http://example.org/StructureDefinition/missing".to_owned()],
        )
        .expect("snapshot")
    }

    fn dep(from: &str, to: &str) -> PackageDependencyEdge {
        PackageDependencyEdge {
            from_name: from.to_owned(),
            from_version: "1.0.0".to_owned(),
            to_name: to.to_owned(),
            to_version: "1.0.0".to_owned(),
        }
    }

    fn canonical(source: &str, target: &str, status: &str) -> CanonicalReferenceEdge {
        CanonicalReferenceEdge {
            source_canonical: source.to_owned(),
            status: status.to_owned(),
            target_canonical: target.to_owned(),
        }
    }

    #[test]
    fn closures_are_order_independent_and_partition_the_snapshot() {
        let left = project_closures(
            &snapshot(),
            vec![dep("acme.b", "acme.a"), dep("acme.a", "acme.b")],
            vec![
                canonical(
                    "http://example.org/StructureDefinition/src",
                    "http://example.org/StructureDefinition/found",
                    CANONICAL_RESOLVED,
                ),
                canonical(
                    "http://example.org/StructureDefinition/src",
                    "http://example.org/StructureDefinition/maybe",
                    CANONICAL_AMBIGUOUS,
                ),
            ],
        )
        .expect("left");
        let right = project_closures(
            &snapshot(),
            vec![dep("acme.a", "acme.b"), dep("acme.b", "acme.a")],
            vec![
                canonical(
                    "http://example.org/StructureDefinition/src",
                    "http://example.org/StructureDefinition/maybe",
                    CANONICAL_AMBIGUOUS,
                ),
                canonical(
                    "http://example.org/StructureDefinition/src",
                    "http://example.org/StructureDefinition/found",
                    CANONICAL_RESOLVED,
                ),
            ],
        )
        .expect("right");
        assert_eq!(left, right);
        assert_eq!(
            left.package_members.len() + left.unrelated_packages.len(),
            3
        );
        assert_eq!(left.unrelated_packages[0].name, "acme.c");
        assert_eq!(
            left.unresolved_canonicals,
            vec!["http://example.org/StructureDefinition/missing".to_owned()]
        );
        assert_eq!(
            left.resolved_canonicals,
            vec!["http://example.org/StructureDefinition/found".to_owned()]
        );
        assert_eq!(
            left.ambiguous_canonicals,
            vec!["http://example.org/StructureDefinition/maybe".to_owned()]
        );
    }

    #[test]
    fn package_and_canonical_edges_change_independent_identities() {
        let baseline = project_closures(
            &snapshot(),
            vec![dep("acme.a", "acme.b")],
            vec![canonical(
                "http://example.org/StructureDefinition/src",
                "http://example.org/StructureDefinition/other",
                CANONICAL_UNRESOLVED,
            )],
        )
        .expect("baseline");
        let package_changed = project_closures(
            &snapshot(),
            vec![dep("acme.a", "acme.c")],
            vec![canonical(
                "http://example.org/StructureDefinition/src",
                "http://example.org/StructureDefinition/other",
                CANONICAL_UNRESOLVED,
            )],
        )
        .expect("package change");
        assert_ne!(
            baseline.package_closure_sha256,
            package_changed.package_closure_sha256
        );
        assert_eq!(
            baseline.canonical_closure_sha256,
            package_changed.canonical_closure_sha256
        );

        let canonical_changed = project_closures(
            &snapshot(),
            vec![dep("acme.a", "acme.b")],
            vec![canonical(
                "http://example.org/StructureDefinition/src",
                "http://example.org/StructureDefinition/changed",
                CANONICAL_UNRESOLVED,
            )],
        )
        .expect("canonical change");
        assert_eq!(
            baseline.package_closure_sha256,
            canonical_changed.package_closure_sha256
        );
        assert_ne!(
            baseline.canonical_closure_sha256,
            canonical_changed.canonical_closure_sha256
        );
    }

    #[test]
    fn canonical_resolution_does_not_create_a_package_member() {
        let closures = project_closures(
            &snapshot(),
            Vec::new(),
            vec![canonical(
                "http://example.org/StructureDefinition/src",
                "http://example.org/StructureDefinition/found",
                CANONICAL_RESOLVED,
            )],
        )
        .expect("canonical only");
        assert!(closures.package_members.is_empty());
        assert_eq!(closures.unrelated_packages.len(), 3);
        assert!(closures.package_edges.is_empty());
    }

    #[test]
    fn mutable_unknown_self_duplicate_and_conflict_fail_closed() {
        let mut mutable = snapshot();
        mutable.packages[0].mutability = MUTABLE_CI.to_owned();
        assert!(matches!(
            project_closures(&mutable, Vec::new(), Vec::new()),
            Err(ClosureError::Snapshot(
                SnapshotError::MutableSourceIsNotPublished { .. }
            ))
        ));

        let unknown =
            project_closures(&snapshot(), vec![dep("acme.a", "acme.missing")], Vec::new());
        assert!(matches!(unknown, Err(ClosureError::UnknownPackage { .. })));
        assert!(matches!(
            project_closures(&snapshot(), vec![dep("acme.a", "acme.a")], Vec::new()),
            Err(ClosureError::SelfDependency { .. })
        ));
        assert!(matches!(
            project_closures(
                &snapshot(),
                vec![dep("acme.a", "acme.b"), dep("acme.a", "acme.b")],
                Vec::new()
            ),
            Err(ClosureError::DuplicatePackageEdge)
        ));
        assert!(matches!(
            project_closures(
                &snapshot(),
                Vec::new(),
                vec![canonical(
                    "http://example.org/StructureDefinition/src",
                    "http://example.org/StructureDefinition/missing",
                    CANONICAL_RESOLVED,
                )]
            ),
            Err(ClosureError::ConflictingCanonicalStatus { .. })
        ));
        assert!(matches!(
            project_closures(
                &snapshot(),
                Vec::new(),
                vec![
                    canonical(
                        "http://example.org/StructureDefinition/src",
                        "http://example.org/StructureDefinition/found",
                        CANONICAL_RESOLVED,
                    ),
                    canonical(
                        "http://example.org/StructureDefinition/src",
                        "http://example.org/StructureDefinition/found",
                        CANONICAL_RESOLVED,
                    ),
                ]
            ),
            Err(ClosureError::DuplicateCanonicalEdge)
        ));
        assert!(matches!(
            project_closures(
                &snapshot(),
                Vec::new(),
                vec![canonical(
                    "http://example.org/StructureDefinition/src",
                    "http://example.org/StructureDefinition/found",
                    "PROBABLY",
                )]
            ),
            Err(ClosureError::InvalidStatus)
        ));
    }

    #[test]
    fn edge_and_canonical_bounds_fail_closed() {
        let edges = vec![
            PackageDependencyEdge {
                from_name: "x".to_owned(),
                from_version: "1".to_owned(),
                to_name: "y".to_owned(),
                to_version: "1".to_owned(),
            };
            MAX_CLOSURE_EDGES + 1
        ];
        assert!(matches!(
            project_closures(&snapshot(), edges, Vec::new()),
            Err(ClosureError::TooManyEdges)
        ));
        let long = "a".repeat(MAX_CANONICAL_CHARS + 1);
        assert!(matches!(
            project_closures(
                &snapshot(),
                Vec::new(),
                vec![canonical(
                    "http://example.org/src",
                    &long,
                    CANONICAL_UNRESOLVED
                )]
            ),
            Err(ClosureError::CanonicalTooLong)
        ));
    }
}
