use thiserror::Error;

use crate::{
    project_cache_identity, project_closures, project_snapshot, project_source_lifecycle,
    project_workspace, require_published_authority, CacheIdentityError, CanonicalReferenceEdge,
    ClosureError, EcosystemSnapshot, LifecycleError, PackageDependencyEdge, SnapshotError,
    SnapshotPackage, SourceLifecycle, WorkspaceError, WorkspaceMember,
};

pub const ECOSYSTEM_REPLAY_SCHEMA: &str = "commandf.ecosystem-replay/v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayInputs {
    pub packages: Vec<SnapshotPackage>,
    pub unresolved_canonicals: Vec<String>,
    pub package_edges: Vec<PackageDependencyEdge>,
    pub canonical_edges: Vec<CanonicalReferenceEdge>,
    pub sources: Vec<SourceLifecycle>,
    pub members: Vec<WorkspaceMember>,
    pub engine_schema: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayIdentity {
    pub schema: String,
    pub snapshot_sha256: String,
    pub package_closure_sha256: String,
    pub canonical_closure_sha256: String,
    pub lifecycle_sha256: String,
    pub workspace_sha256: String,
    pub cache_sha256: String,
}

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Closure(#[from] ClosureError),

    #[error(transparent)]
    Lifecycle(#[from] LifecycleError),

    #[error(transparent)]
    Workspace(#[from] WorkspaceError),

    #[error(transparent)]
    Cache(#[from] CacheIdentityError),
}

pub fn replay_frozen_observation(inputs: &ReplayInputs) -> Result<ReplayIdentity, ReplayError> {
    let snapshot = project_snapshot(
        inputs.packages.clone(),
        inputs.unresolved_canonicals.clone(),
    )?;
    require_published_authority(&snapshot)?;
    replay_from_snapshot(&snapshot, inputs)
}

fn replay_from_snapshot(
    snapshot: &EcosystemSnapshot,
    inputs: &ReplayInputs,
) -> Result<ReplayIdentity, ReplayError> {
    let closures = project_closures(
        snapshot,
        inputs.package_edges.clone(),
        inputs.canonical_edges.clone(),
    )?;
    let lifecycle = project_source_lifecycle(snapshot, inputs.sources.clone())?;
    let workspace = project_workspace(snapshot, inputs.members.clone())?;
    let cache = project_cache_identity(snapshot, &inputs.engine_schema)?;
    Ok(ReplayIdentity {
        schema: ECOSYSTEM_REPLAY_SCHEMA.to_owned(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        package_closure_sha256: closures.package_closure_sha256,
        canonical_closure_sha256: closures.canonical_closure_sha256,
        lifecycle_sha256: lifecycle.lifecycle_sha256,
        workspace_sha256: workspace.workspace_sha256,
        cache_sha256: cache.cache_sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IMMUTABLE_RELEASE, LIFECYCLE_CURRENT, MUTABLE_CI};

    fn inputs() -> ReplayInputs {
        ReplayInputs {
            packages: vec![
                package("acme.b", "1.0.0", 0x22),
                package("acme.a", "1.0.0", 0x11),
            ],
            unresolved_canonicals: vec!["http://example.org/missing".to_owned()],
            package_edges: vec![PackageDependencyEdge {
                from_name: "acme.b".to_owned(),
                from_version: "1.0.0".to_owned(),
                to_name: "acme.a".to_owned(),
                to_version: "1.0.0".to_owned(),
            }],
            canonical_edges: Vec::new(),
            sources: vec![
                source("mirror-b", LIFECYCLE_CURRENT),
                source("mirror-a", LIFECYCLE_CURRENT),
            ],
            members: vec![
                WorkspaceMember {
                    name: "acme.b".to_owned(),
                    version: "1.0.0".to_owned(),
                },
                WorkspaceMember {
                    name: "acme.a".to_owned(),
                    version: "1.0.0".to_owned(),
                },
            ],
            engine_schema: "commandf.replay/v1".to_owned(),
        }
    }

    fn package(name: &str, version: &str, digest_byte: u8) -> SnapshotPackage {
        SnapshotPackage {
            name: name.to_owned(),
            version: version.to_owned(),
            archive_sha256: format!("{digest_byte:02x}").repeat(32),
            source_id: format!("mirror-{}", name.rsplit('.').next().unwrap_or("x")),
            mutability: IMMUTABLE_RELEASE.to_owned(),
        }
    }

    fn source(source_id: &str, state: &str) -> SourceLifecycle {
        SourceLifecycle {
            source_id: source_id.to_owned(),
            state: state.to_owned(),
        }
    }

    #[test]
    fn two_replays_of_the_same_frozen_inputs_match() {
        let first = replay_frozen_observation(&inputs()).expect("first");
        let mut reordered = inputs();
        reordered.packages.reverse();
        reordered.sources.reverse();
        reordered.members.reverse();
        let second = replay_frozen_observation(&reordered).expect("second");
        assert_eq!(first, second);
        assert_eq!(first.schema, ECOSYSTEM_REPLAY_SCHEMA);
        let mut changed = inputs();
        changed.packages[0].archive_sha256 = "33".repeat(32);
        let third = replay_frozen_observation(&changed).expect("changed");
        assert_ne!(first.snapshot_sha256, third.snapshot_sha256);
    }

    #[test]
    fn mutable_ci_cannot_be_replayed_as_published() {
        let mut mutable = inputs();
        mutable.packages[0].mutability = MUTABLE_CI.to_owned();
        assert!(matches!(
            replay_frozen_observation(&mutable),
            Err(ReplayError::Snapshot(
                SnapshotError::MutableSourceIsNotPublished { .. }
            ))
        ));
    }
}
