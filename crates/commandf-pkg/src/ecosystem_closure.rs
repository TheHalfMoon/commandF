use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    require_published_authority, EcosystemSnapshot, PackageCache, PackageError, SnapshotError,
};

pub const ECOSYSTEM_CLOSURE_SCHEMA: &str = "commandf.ecosystem-closure/v1";
pub const ECOSYSTEM_CLOSURE_BYTES_SCHEMA: &str = "commandf.ecosystem-closure-bytes/v1";
pub const MAX_CLOSURE_MACHINE_BYTES: usize = 16 * 1024 * 1024;
pub const ECOSYSTEM_QUERY_SCHEMA: &str = "commandf.ecosystem-query/v1";
pub const CANONICAL_RESOLVED: &str = "RESOLVED";
pub const CANONICAL_UNRESOLVED: &str = "UNRESOLVED";
pub const CANONICAL_AMBIGUOUS: &str = "AMBIGUOUS";
pub const MAX_CLOSURE_EDGES: usize = 10_000;
pub const MAX_CANONICAL_CHARS: usize = 2_048;

const PACKAGE_KIND: &str = "package-dependency";
const CANONICAL_KIND: &str = "canonical-reference";

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageDependencyEdge {
    pub from_name: String,
    pub from_version: String,
    pub to_name: String,
    pub to_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CanonicalReferenceEdge {
    pub source_canonical: String,
    pub status: String,
    pub target_canonical: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureMachineBytes {
    pub serialization_schema: String,
    pub package_bytes: Vec<u8>,
    pub canonical_bytes: Vec<u8>,
    pub package_machine_sha256: String,
    pub canonical_machine_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClosureQuery {
    pub schema: String,
    pub snapshot_sha256: String,
    pub package_closure_sha256: String,
    pub canonical_closure_sha256: String,
    pub query_sha256: String,
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

    #[error("closure schema is not {ECOSYSTEM_CLOSURE_SCHEMA}")]
    UnexpectedClosureSchema,

    #[error("closure snapshot identity does not match the published snapshot")]
    UnboundSnapshot,

    #[error("closure package records do not partition the snapshot")]
    PackagePartitionMismatch,

    #[error("canonical witness lists overlap or omit a snapshot unresolved canonical")]
    CanonicalWitnessMismatch,

    #[error("stored closure digest does not match the closure document")]
    ClosureDigestMismatch,

    #[error("closure machine document exceeds {MAX_CLOSURE_MACHINE_BYTES} bytes")]
    ClosureMachineTooLarge,

    #[error("closure bytes are not the canonical encoding")]
    NoncanonicalClosure,

    #[error("closure JSON could not be read as the canonical document")]
    MalformedClosure,
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

    let package_closure_sha256 = digest_package_closure(
        &snapshot.snapshot_sha256,
        &package_members,
        &package_edges,
        &unrelated_packages,
    )?;
    let canonical_closure_sha256 = digest_canonical_closure(
        &snapshot.snapshot_sha256,
        &resolved_canonicals,
        &unresolved_canonicals,
        &ambiguous_canonicals,
        &canonical_edges,
    )?;

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
        package_closure_sha256,
        canonical_closure_sha256,
    })
}

pub fn query_closures(
    snapshot: &EcosystemSnapshot,
    closures: &EcosystemClosures,
) -> Result<ClosureQuery, ClosureError> {
    if closures.package_edges.len() > MAX_CLOSURE_EDGES
        || closures.canonical_edges.len() > MAX_CLOSURE_EDGES
        || closures
            .package_members
            .len()
            .saturating_add(closures.unrelated_packages.len())
            > MAX_CLOSURE_EDGES
        || closures.resolved_canonicals.len() > MAX_CLOSURE_EDGES
        || closures.unresolved_canonicals.len() > MAX_CLOSURE_EDGES
        || closures.ambiguous_canonicals.len() > MAX_CLOSURE_EDGES
    {
        return Err(ClosureError::TooManyEdges);
    }
    require_published_authority(snapshot)?;
    if closures.schema != ECOSYSTEM_CLOSURE_SCHEMA {
        return Err(ClosureError::UnexpectedClosureSchema);
    }
    if closures.snapshot_sha256 != snapshot.snapshot_sha256 {
        return Err(ClosureError::UnboundSnapshot);
    }

    let mut snapshot_packages = BTreeMap::new();
    for package in &snapshot.packages {
        snapshot_packages.insert(
            (package.name.as_str(), package.version.as_str()),
            package.archive_sha256.as_str(),
        );
    }
    let mut closure_packages = BTreeMap::new();
    for package in closures
        .package_members
        .iter()
        .chain(closures.unrelated_packages.iter())
    {
        if closure_packages
            .insert(
                (package.name.as_str(), package.version.as_str()),
                package.archive_sha256.as_str(),
            )
            .is_some()
        {
            return Err(ClosureError::PackagePartitionMismatch);
        }
    }
    if snapshot_packages != closure_packages {
        return Err(ClosureError::PackagePartitionMismatch);
    }

    let resolved: BTreeSet<&str> = closures
        .resolved_canonicals
        .iter()
        .map(String::as_str)
        .collect();
    let unresolved: BTreeSet<&str> = closures
        .unresolved_canonicals
        .iter()
        .map(String::as_str)
        .collect();
    let ambiguous: BTreeSet<&str> = closures
        .ambiguous_canonicals
        .iter()
        .map(String::as_str)
        .collect();
    if !resolved.is_disjoint(&unresolved)
        || !resolved.is_disjoint(&ambiguous)
        || !unresolved.is_disjoint(&ambiguous)
    {
        return Err(ClosureError::CanonicalWitnessMismatch);
    }
    if snapshot
        .unresolved_canonicals
        .iter()
        .any(|canonical| !unresolved.contains(canonical.as_str()))
    {
        return Err(ClosureError::CanonicalWitnessMismatch);
    }

    let package_closure_sha256 = digest_package_closure(
        &snapshot.snapshot_sha256,
        &closures.package_members,
        &closures.package_edges,
        &closures.unrelated_packages,
    )?;
    let canonical_closure_sha256 = digest_canonical_closure(
        &snapshot.snapshot_sha256,
        &closures.resolved_canonicals,
        &closures.unresolved_canonicals,
        &closures.ambiguous_canonicals,
        &closures.canonical_edges,
    )?;
    if package_closure_sha256 != closures.package_closure_sha256
        || canonical_closure_sha256 != closures.canonical_closure_sha256
    {
        return Err(ClosureError::ClosureDigestMismatch);
    }

    let body = QueryBody {
        canonical_closure_sha256: &canonical_closure_sha256,
        package_closure_sha256: &package_closure_sha256,
        schema: ECOSYSTEM_QUERY_SCHEMA,
        snapshot_sha256: &snapshot.snapshot_sha256,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    Ok(ClosureQuery {
        schema: ECOSYSTEM_QUERY_SCHEMA.to_owned(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        package_closure_sha256,
        canonical_closure_sha256,
        query_sha256: PackageCache::digest(&bytes),
    })
}

pub fn encode_closure_machine(
    snapshot: &EcosystemSnapshot,
    closures: &EcosystemClosures,
) -> Result<ClosureMachineBytes, ClosureError> {
    query_closures(snapshot, closures)?;
    let package_bytes = package_closure_bytes(
        &snapshot.snapshot_sha256,
        &closures.package_members,
        &closures.package_edges,
        &closures.unrelated_packages,
    )?;
    let canonical_bytes = canonical_closure_bytes(
        &snapshot.snapshot_sha256,
        &closures.resolved_canonicals,
        &closures.unresolved_canonicals,
        &closures.ambiguous_canonicals,
        &closures.canonical_edges,
    )?;
    if package_bytes.len() > MAX_CLOSURE_MACHINE_BYTES
        || canonical_bytes.len() > MAX_CLOSURE_MACHINE_BYTES
    {
        return Err(ClosureError::ClosureMachineTooLarge);
    }
    Ok(ClosureMachineBytes {
        serialization_schema: ECOSYSTEM_CLOSURE_BYTES_SCHEMA.to_owned(),
        package_machine_sha256: PackageCache::digest(&package_bytes),
        canonical_machine_sha256: PackageCache::digest(&canonical_bytes),
        package_bytes,
        canonical_bytes,
    })
}

pub fn decode_closure_machine(
    snapshot: &EcosystemSnapshot,
    package_bytes: &[u8],
    canonical_bytes: &[u8],
) -> Result<EcosystemClosures, ClosureError> {
    if package_bytes.len() > MAX_CLOSURE_MACHINE_BYTES
        || canonical_bytes.len() > MAX_CLOSURE_MACHINE_BYTES
    {
        return Err(ClosureError::ClosureMachineTooLarge);
    }
    let package: PackageClosureDocument =
        serde_json::from_slice(package_bytes).map_err(|_| ClosureError::MalformedClosure)?;
    let canonical: CanonicalClosureDocument =
        serde_json::from_slice(canonical_bytes).map_err(|_| ClosureError::MalformedClosure)?;
    let package_canonical = serde_json::to_vec(&package).map_err(PackageError::Json)?;
    let canonical_canonical = serde_json::to_vec(&canonical).map_err(PackageError::Json)?;
    if package_canonical.as_slice() != package_bytes
        || canonical_canonical.as_slice() != canonical_bytes
    {
        return Err(ClosureError::NoncanonicalClosure);
    }
    if package.kind != PACKAGE_KIND || canonical.kind != CANONICAL_KIND {
        return Err(ClosureError::MalformedClosure);
    }
    let closures = EcosystemClosures {
        schema: package.schema,
        snapshot_sha256: package.snapshot_sha256,
        package_members: package.members,
        package_edges: package.edges,
        unrelated_packages: package.unrelated_packages,
        resolved_canonicals: canonical.resolved_canonicals,
        unresolved_canonicals: canonical.unresolved_canonicals,
        ambiguous_canonicals: canonical.ambiguous_canonicals,
        canonical_edges: canonical.edges,
        package_closure_sha256: PackageCache::digest(&package_canonical),
        canonical_closure_sha256: PackageCache::digest(&canonical_canonical),
    };
    if closures.schema != canonical.schema || closures.snapshot_sha256 != canonical.snapshot_sha256
    {
        return Err(ClosureError::UnboundSnapshot);
    }
    query_closures(snapshot, &closures)?;
    Ok(closures)
}

fn package_closure_bytes(
    snapshot_sha256: &str,
    members: &[ClosurePackage],
    edges: &[PackageDependencyEdge],
    unrelated_packages: &[ClosurePackage],
) -> Result<Vec<u8>, ClosureError> {
    let body = PackageClosureBody {
        edges,
        kind: PACKAGE_KIND,
        members,
        schema: ECOSYSTEM_CLOSURE_SCHEMA,
        snapshot_sha256,
        unrelated_packages,
    };
    Ok(serde_json::to_vec(&body).map_err(PackageError::Json)?)
}

fn canonical_closure_bytes(
    snapshot_sha256: &str,
    resolved_canonicals: &[String],
    unresolved_canonicals: &[String],
    ambiguous_canonicals: &[String],
    edges: &[CanonicalReferenceEdge],
) -> Result<Vec<u8>, ClosureError> {
    let body = CanonicalClosureBody {
        ambiguous_canonicals,
        edges,
        kind: CANONICAL_KIND,
        resolved_canonicals,
        schema: ECOSYSTEM_CLOSURE_SCHEMA,
        snapshot_sha256,
        unresolved_canonicals,
    };
    Ok(serde_json::to_vec(&body).map_err(PackageError::Json)?)
}

fn digest_package_closure(
    snapshot_sha256: &str,
    members: &[ClosurePackage],
    edges: &[PackageDependencyEdge],
    unrelated_packages: &[ClosurePackage],
) -> Result<String, ClosureError> {
    let bytes = package_closure_bytes(snapshot_sha256, members, edges, unrelated_packages)?;
    Ok(PackageCache::digest(&bytes))
}

fn digest_canonical_closure(
    snapshot_sha256: &str,
    resolved_canonicals: &[String],
    unresolved_canonicals: &[String],
    ambiguous_canonicals: &[String],
    edges: &[CanonicalReferenceEdge],
) -> Result<String, ClosureError> {
    let bytes = canonical_closure_bytes(
        snapshot_sha256,
        resolved_canonicals,
        unresolved_canonicals,
        ambiguous_canonicals,
        edges,
    )?;
    Ok(PackageCache::digest(&bytes))
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

#[derive(Serialize)]
struct QueryBody<'a> {
    canonical_closure_sha256: &'a str,
    package_closure_sha256: &'a str,
    schema: &'a str,
    snapshot_sha256: &'a str,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PackageClosureDocument {
    edges: Vec<PackageDependencyEdge>,
    kind: String,
    members: Vec<ClosurePackage>,
    schema: String,
    snapshot_sha256: String,
    unrelated_packages: Vec<ClosurePackage>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CanonicalClosureDocument {
    ambiguous_canonicals: Vec<String>,
    edges: Vec<CanonicalReferenceEdge>,
    kind: String,
    resolved_canonicals: Vec<String>,
    schema: String,
    snapshot_sha256: String,
    unresolved_canonicals: Vec<String>,
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

    #[test]
    fn query_binds_snapshot_and_both_closure_digests() {
        let published = snapshot();
        let closures = project_closures(
            &published,
            vec![dep("acme.a", "acme.b")],
            vec![canonical(
                "http://example.org/StructureDefinition/src",
                "http://example.org/StructureDefinition/found",
                CANONICAL_RESOLVED,
            )],
        )
        .expect("closures");
        let left = query_closures(&published, &closures).expect("left query");
        let right = query_closures(&published, &closures).expect("right query");
        assert_eq!(left, right);
        assert_eq!(left.snapshot_sha256, published.snapshot_sha256);
        assert_eq!(left.package_closure_sha256, closures.package_closure_sha256);
        assert_eq!(
            left.canonical_closure_sha256,
            closures.canonical_closure_sha256
        );

        let changed = project_closures(&published, vec![dep("acme.a", "acme.c")], Vec::new())
            .expect("changed closures");
        let changed_query = query_closures(&published, &changed).expect("changed query");
        assert_ne!(left.query_sha256, changed_query.query_sha256);
    }

    #[test]
    fn query_rejects_unbound_tampered_and_overlapping_witnesses() {
        let published = snapshot();
        let mut closures = project_closures(&published, vec![dep("acme.a", "acme.b")], Vec::new())
            .expect("closures");
        closures.snapshot_sha256 = "ab".repeat(32);
        assert!(matches!(
            query_closures(&published, &closures),
            Err(ClosureError::UnboundSnapshot)
        ));

        closures = project_closures(&published, vec![dep("acme.a", "acme.b")], Vec::new())
            .expect("closures");
        closures.package_closure_sha256 = "cd".repeat(32);
        assert!(matches!(
            query_closures(&published, &closures),
            Err(ClosureError::ClosureDigestMismatch)
        ));

        closures = project_closures(&published, vec![dep("acme.a", "acme.b")], Vec::new())
            .expect("closures");
        closures.unresolved_canonicals.clear();
        assert!(matches!(
            query_closures(&published, &closures),
            Err(ClosureError::CanonicalWitnessMismatch)
        ));

        closures = project_closures(&published, vec![dep("acme.a", "acme.b")], Vec::new())
            .expect("closures");
        closures
            .resolved_canonicals
            .push("http://example.org/StructureDefinition/missing".to_owned());
        assert!(matches!(
            query_closures(&published, &closures),
            Err(ClosureError::CanonicalWitnessMismatch)
        ));

        closures = project_closures(&published, vec![dep("acme.a", "acme.b")], Vec::new())
            .expect("closures");
        closures.package_members[0].archive_sha256 = "ef".repeat(32);
        assert!(matches!(
            query_closures(&published, &closures),
            Err(ClosureError::PackagePartitionMismatch)
        ));

        let mut mutable = published.clone();
        mutable.packages[0].mutability = MUTABLE_CI.to_owned();
        let mutable_closures =
            project_closures(&published, Vec::new(), Vec::new()).expect("immutable projection");
        assert!(matches!(
            query_closures(&mutable, &mutable_closures),
            Err(ClosureError::Snapshot(
                SnapshotError::MutableSourceIsNotPublished { .. }
            ))
        ));
    }

    #[test]
    fn closure_machine_bytes_keep_package_and_canonical_documents_separate() {
        let published = snapshot();
        let left = project_closures(
            &published,
            vec![dep("acme.b", "acme.a"), dep("acme.a", "acme.b")],
            vec![canonical(
                "http://example.org/StructureDefinition/src",
                "http://example.org/StructureDefinition/found",
                CANONICAL_RESOLVED,
            )],
        )
        .expect("left");
        let right = project_closures(
            &published,
            vec![dep("acme.a", "acme.b"), dep("acme.b", "acme.a")],
            vec![canonical(
                "http://example.org/StructureDefinition/src",
                "http://example.org/StructureDefinition/found",
                CANONICAL_RESOLVED,
            )],
        )
        .expect("right");
        let encoded = encode_closure_machine(&published, &left).expect("encode");
        let again = encode_closure_machine(&published, &right).expect("again");
        assert_eq!(encoded.package_bytes, again.package_bytes);
        assert_eq!(encoded.canonical_bytes, again.canonical_bytes);
        assert_eq!(encoded.package_machine_sha256, left.package_closure_sha256);
        assert_eq!(
            encoded.canonical_machine_sha256,
            left.canonical_closure_sha256
        );
        assert_ne!(encoded.package_bytes, encoded.canonical_bytes);
        let decoded =
            decode_closure_machine(&published, &encoded.package_bytes, &encoded.canonical_bytes)
                .expect("decode");
        assert_eq!(decoded, left);

        let package_only = project_closures(&published, vec![dep("acme.a", "acme.c")], Vec::new())
            .expect("package only");
        let package_bytes = encode_closure_machine(&published, &package_only).expect("package");
        assert_ne!(
            package_bytes.package_machine_sha256,
            encoded.package_machine_sha256
        );
        assert_ne!(
            package_bytes.canonical_machine_sha256,
            encoded.canonical_machine_sha256
        );

        let mut spaced = encoded.package_bytes.clone();
        spaced.insert(1, b' ');
        assert!(matches!(
            decode_closure_machine(&published, &spaced, &encoded.canonical_bytes),
            Err(ClosureError::NoncanonicalClosure)
        ));
        assert!(matches!(
            decode_closure_machine(
                &published,
                &vec![0; MAX_CLOSURE_MACHINE_BYTES + 1],
                &encoded.canonical_bytes
            ),
            Err(ClosureError::ClosureMachineTooLarge)
        ));
        let mut tampered = left.clone();
        tampered.package_closure_sha256 = "ab".repeat(32);
        assert!(matches!(
            encode_closure_machine(&published, &tampered),
            Err(ClosureError::ClosureDigestMismatch)
        ));
    }
}
