use std::collections::BTreeSet;

use serde::Serialize;
use thiserror::Error;

use crate::{
    require_published_authority, EcosystemSnapshot, PackageCache, PackageError, SnapshotError,
};

pub const ECOSYSTEM_WORKSPACE_SCHEMA: &str = "commandf.ecosystem-workspace/v1";
pub const MAX_WORKSPACE_MEMBERS: usize = 10_000;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct WorkspaceMember {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EcosystemWorkspace {
    pub schema: String,
    pub snapshot_sha256: String,
    pub members: Vec<WorkspaceMember>,
    pub workspace_sha256: String,
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error(transparent)]
    Snapshot(#[from] SnapshotError),

    #[error(transparent)]
    Package(#[from] PackageError),

    #[error("workspace member count exceeds {MAX_WORKSPACE_MEMBERS}")]
    TooManyMembers,

    #[error("duplicate workspace member {name}@{version}")]
    DuplicateMember { name: String, version: String },

    #[error("workspace member {name}@{version} is not in the snapshot")]
    UnknownMember { name: String, version: String },

    #[error("workspace schema is not {ECOSYSTEM_WORKSPACE_SCHEMA}")]
    UnexpectedWorkspaceSchema,

    #[error("stored workspace digest does not match the workspace document")]
    WorkspaceDigestMismatch,
}

pub fn project_workspace(
    snapshot: &EcosystemSnapshot,
    members: Vec<WorkspaceMember>,
) -> Result<EcosystemWorkspace, WorkspaceError> {
    if members.len() > MAX_WORKSPACE_MEMBERS {
        return Err(WorkspaceError::TooManyMembers);
    }
    require_published_authority(snapshot)?;
    let mut members = members;
    members.sort();
    for pair in members.windows(2) {
        if pair[0] == pair[1] {
            return Err(WorkspaceError::DuplicateMember {
                name: pair[0].name.clone(),
                version: pair[0].version.clone(),
            });
        }
    }
    let snapshot_members: BTreeSet<(&str, &str)> = snapshot
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package.version.as_str()))
        .collect();
    for member in &members {
        if !snapshot_members.contains(&(member.name.as_str(), member.version.as_str())) {
            return Err(WorkspaceError::UnknownMember {
                name: member.name.clone(),
                version: member.version.clone(),
            });
        }
    }
    let body = WorkspaceBody {
        members: &members,
        schema: ECOSYSTEM_WORKSPACE_SCHEMA,
        snapshot_sha256: &snapshot.snapshot_sha256,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    Ok(EcosystemWorkspace {
        schema: ECOSYSTEM_WORKSPACE_SCHEMA.to_owned(),
        snapshot_sha256: snapshot.snapshot_sha256.clone(),
        members,
        workspace_sha256: PackageCache::digest(&bytes),
    })
}

pub fn verify_workspace_identity(
    snapshot: &EcosystemSnapshot,
    workspace: &EcosystemWorkspace,
) -> Result<(), WorkspaceError> {
    if workspace.schema != ECOSYSTEM_WORKSPACE_SCHEMA {
        return Err(WorkspaceError::UnexpectedWorkspaceSchema);
    }
    if workspace.snapshot_sha256 != snapshot.snapshot_sha256 {
        return Err(WorkspaceError::WorkspaceDigestMismatch);
    }
    let projected = project_workspace(snapshot, workspace.members.clone())?;
    if projected != *workspace {
        return Err(WorkspaceError::WorkspaceDigestMismatch);
    }
    Ok(())
}

#[derive(Serialize)]
struct WorkspaceBody<'a> {
    members: &'a [WorkspaceMember],
    schema: &'a str,
    snapshot_sha256: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{project_snapshot, SnapshotPackage, IMMUTABLE_RELEASE, MUTABLE_CI};

    fn package(name: &str, version: &str) -> SnapshotPackage {
        SnapshotPackage {
            name: name.to_owned(),
            version: version.to_owned(),
            archive_sha256: "ab".repeat(32),
            source_id: "local-mirror".to_owned(),
            mutability: IMMUTABLE_RELEASE.to_owned(),
        }
    }

    fn member(name: &str, version: &str) -> WorkspaceMember {
        WorkspaceMember {
            name: name.to_owned(),
            version: version.to_owned(),
        }
    }

    #[test]
    fn workspace_membership_is_order_independent() {
        let snapshot = project_snapshot(
            vec![package("acme.b", "1.0.0"), package("acme.a", "2.0.0")],
            Vec::new(),
        )
        .expect("snapshot");
        let left = project_workspace(
            &snapshot,
            vec![member("acme.b", "1.0.0"), member("acme.a", "2.0.0")],
        )
        .expect("left");
        let right = project_workspace(
            &snapshot,
            vec![member("acme.a", "2.0.0"), member("acme.b", "1.0.0")],
        )
        .expect("right");
        assert_eq!(left, right);
        verify_workspace_identity(&snapshot, &left).expect("verify");
        let subset = project_workspace(&snapshot, vec![member("acme.a", "2.0.0")]).expect("subset");
        assert_ne!(left.workspace_sha256, subset.workspace_sha256);
    }

    #[test]
    fn unknown_duplicate_mutable_and_bounds_fail_closed() {
        let snapshot =
            project_snapshot(vec![package("acme.a", "1.0.0")], Vec::new()).expect("snapshot");
        assert!(matches!(
            project_workspace(&snapshot, vec![member("acme.missing", "1.0.0")]),
            Err(WorkspaceError::UnknownMember { .. })
        ));
        assert!(matches!(
            project_workspace(
                &snapshot,
                vec![member("acme.a", "1.0.0"), member("acme.a", "1.0.0")]
            ),
            Err(WorkspaceError::DuplicateMember { .. })
        ));
        let mut mutable = snapshot.clone();
        mutable.packages[0].mutability = MUTABLE_CI.to_owned();
        assert!(matches!(
            project_workspace(&mutable, vec![member("acme.a", "1.0.0")]),
            Err(WorkspaceError::Snapshot(
                SnapshotError::MutableSourceIsNotPublished { .. }
            ))
        ));
        let too_many = vec![member("acme.a", "1.0.0"); MAX_WORKSPACE_MEMBERS + 1];
        assert!(matches!(
            project_workspace(&snapshot, too_many),
            Err(WorkspaceError::TooManyMembers)
        ));
    }
}
