use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    require_published_authority, EcosystemSnapshot, PackageCache, PackageError, SnapshotError,
};

pub const ECOSYSTEM_WORKSPACE_SCHEMA: &str = "commandf.ecosystem-workspace/v1";
pub const ECOSYSTEM_WORKSPACE_BYTES_SCHEMA: &str = "commandf.ecosystem-workspace-bytes/v1";
pub const MAX_WORKSPACE_MEMBERS: usize = 10_000;
pub const MAX_WORKSPACE_STRING_BYTES: usize = 2_048;
pub const MAX_WORKSPACE_MACHINE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceMachineBytes {
    pub serialization_schema: String,
    pub bytes: Vec<u8>,
    pub machine_sha256: String,
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

    #[error("workspace string exceeds {MAX_WORKSPACE_STRING_BYTES} bytes")]
    WorkspaceStringTooLong,

    #[error("workspace machine document exceeds {MAX_WORKSPACE_MACHINE_BYTES} bytes")]
    WorkspaceMachineTooLarge,

    #[error("workspace snapshot digest is not 64 lowercase hex characters")]
    InvalidWorkspaceDigest,

    #[error("workspace bytes are not the canonical encoding")]
    NoncanonicalWorkspace,

    #[error("workspace JSON could not be read as the canonical document")]
    MalformedWorkspace,
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

pub fn encode_workspace_machine(
    snapshot: &EcosystemSnapshot,
    workspace: &EcosystemWorkspace,
) -> Result<WorkspaceMachineBytes, WorkspaceError> {
    reject_workspace_shape(workspace)?;
    verify_workspace_identity(snapshot, workspace)?;
    let bytes = workspace_document_bytes(workspace)?;
    let machine_sha256 = PackageCache::digest(&bytes);
    if machine_sha256 != workspace.workspace_sha256 {
        return Err(WorkspaceError::WorkspaceDigestMismatch);
    }
    Ok(WorkspaceMachineBytes {
        serialization_schema: ECOSYSTEM_WORKSPACE_BYTES_SCHEMA.to_owned(),
        bytes,
        machine_sha256,
    })
}

pub fn decode_workspace_machine(
    snapshot: &EcosystemSnapshot,
    bytes: &[u8],
) -> Result<EcosystemWorkspace, WorkspaceError> {
    if bytes.len() > MAX_WORKSPACE_MACHINE_BYTES {
        return Err(WorkspaceError::WorkspaceMachineTooLarge);
    }
    let document: WorkspaceDocument =
        serde_json::from_slice(bytes).map_err(|_| WorkspaceError::MalformedWorkspace)?;
    if document.members.len() > MAX_WORKSPACE_MEMBERS {
        return Err(WorkspaceError::TooManyMembers);
    }
    let canonical = serde_json::to_vec(&document).map_err(PackageError::Json)?;
    if canonical.as_slice() != bytes {
        return Err(WorkspaceError::NoncanonicalWorkspace);
    }
    let workspace = EcosystemWorkspace {
        schema: document.schema,
        snapshot_sha256: document.snapshot_sha256,
        members: document.members,
        workspace_sha256: PackageCache::digest(&canonical),
    };
    reject_workspace_shape(&workspace)?;
    verify_workspace_identity(snapshot, &workspace)?;
    Ok(workspace)
}

fn reject_workspace_shape(workspace: &EcosystemWorkspace) -> Result<(), WorkspaceError> {
    if workspace.members.len() > MAX_WORKSPACE_MEMBERS {
        return Err(WorkspaceError::TooManyMembers);
    }
    if !is_sha256(&workspace.snapshot_sha256) {
        return Err(WorkspaceError::InvalidWorkspaceDigest);
    }
    let mut upper_bound = 128usize;
    for member in &workspace.members {
        if member.name.len() > MAX_WORKSPACE_STRING_BYTES
            || member.version.len() > MAX_WORKSPACE_STRING_BYTES
        {
            return Err(WorkspaceError::WorkspaceStringTooLong);
        }
        upper_bound = upper_bound
            .saturating_add(64)
            .saturating_add(member.name.len().saturating_mul(6))
            .saturating_add(member.version.len().saturating_mul(6));
    }
    if upper_bound > MAX_WORKSPACE_MACHINE_BYTES {
        return Err(WorkspaceError::WorkspaceMachineTooLarge);
    }
    for pair in workspace.members.windows(2) {
        if pair[0] == pair[1] {
            return Err(WorkspaceError::DuplicateMember {
                name: pair[0].name.clone(),
                version: pair[0].version.clone(),
            });
        }
        if pair[0] > pair[1] {
            return Err(WorkspaceError::NoncanonicalWorkspace);
        }
    }
    Ok(())
}

fn workspace_document_bytes(workspace: &EcosystemWorkspace) -> Result<Vec<u8>, WorkspaceError> {
    let body = WorkspaceBody {
        members: &workspace.members,
        schema: ECOSYSTEM_WORKSPACE_SCHEMA,
        snapshot_sha256: &workspace.snapshot_sha256,
    };
    let bytes = serde_json::to_vec(&body).map_err(PackageError::Json)?;
    if bytes.len() > MAX_WORKSPACE_MACHINE_BYTES {
        return Err(WorkspaceError::WorkspaceMachineTooLarge);
    }
    Ok(bytes)
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Serialize)]
struct WorkspaceBody<'a> {
    members: &'a [WorkspaceMember],
    schema: &'a str,
    snapshot_sha256: &'a str,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct WorkspaceDocument {
    members: Vec<WorkspaceMember>,
    schema: String,
    snapshot_sha256: String,
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

    #[test]
    fn workspace_machine_bytes_replay_and_reject_noncanonical_input() {
        let mut first = package("acme.a", "1.0.0");
        first.archive_sha256 = "11".repeat(32);
        let mut second = package("acme.b", "1.0.0");
        second.archive_sha256 = "22".repeat(32);
        let snapshot =
            project_snapshot(vec![second.clone(), first.clone()], Vec::new()).expect("snapshot");
        let left = project_workspace(
            &snapshot,
            vec![member("acme.b", "1.0.0"), member("acme.a", "1.0.0")],
        )
        .expect("left");
        let right = project_workspace(
            &snapshot,
            vec![member("acme.a", "1.0.0"), member("acme.b", "1.0.0")],
        )
        .expect("right");
        let encoded = encode_workspace_machine(&snapshot, &left).expect("encode");
        let again = encode_workspace_machine(&snapshot, &right).expect("again");
        assert_eq!(encoded.bytes, again.bytes);
        assert_eq!(encoded.machine_sha256, left.workspace_sha256);
        assert_eq!(
            encoded.serialization_schema,
            ECOSYSTEM_WORKSPACE_BYTES_SCHEMA
        );
        assert!(!encoded.bytes.windows(6).any(|window| window == b"branch"));
        assert!(!encoded.bytes.windows(7).any(|window| window == b"PROVEN_"));
        let decoded = decode_workspace_machine(&snapshot, &encoded.bytes).expect("decode");
        assert_eq!(decoded, left);
        let round = encode_workspace_machine(&snapshot, &decoded).expect("round");
        assert_eq!(round.bytes, encoded.bytes);

        let subset = project_workspace(&snapshot, vec![member("acme.a", "1.0.0")]).expect("subset");
        assert_ne!(
            encode_workspace_machine(&snapshot, &subset)
                .expect("subset bytes")
                .machine_sha256,
            encoded.machine_sha256
        );
        let mut other_package = first.clone();
        other_package.archive_sha256 = "33".repeat(32);
        let other = project_snapshot(vec![other_package, second], Vec::new()).expect("other");
        let rebound = project_workspace(
            &other,
            vec![member("acme.a", "1.0.0"), member("acme.b", "1.0.0")],
        )
        .expect("rebound");
        assert_ne!(
            encode_workspace_machine(&other, &rebound)
                .expect("rebound bytes")
                .machine_sha256,
            encoded.machine_sha256
        );
        assert!(matches!(
            decode_workspace_machine(&other, &encoded.bytes),
            Err(WorkspaceError::WorkspaceDigestMismatch)
        ));

        let mut spaced = encoded.bytes.clone();
        spaced.insert(1, b' ');
        assert!(matches!(
            decode_workspace_machine(&snapshot, &spaced),
            Err(WorkspaceError::NoncanonicalWorkspace)
        ));
        let mut trailing = encoded.bytes.clone();
        trailing.push(b'x');
        assert!(matches!(
            decode_workspace_machine(&snapshot, &trailing),
            Err(WorkspaceError::MalformedWorkspace)
        ));
        assert!(matches!(
            decode_workspace_machine(
                &snapshot,
                br#"{"members":[],"schema":"commandf.ecosystem-workspace/v1","snapshot_sha256":"aa","branch":"main"}"#
            ),
            Err(WorkspaceError::MalformedWorkspace)
        ));
        assert!(matches!(
            decode_workspace_machine(&snapshot, &vec![0; MAX_WORKSPACE_MACHINE_BYTES + 1]),
            Err(WorkspaceError::WorkspaceMachineTooLarge)
        ));

        let mut tampered = left.clone();
        tampered.workspace_sha256 = "ab".repeat(32);
        assert!(matches!(
            encode_workspace_machine(&snapshot, &tampered),
            Err(WorkspaceError::WorkspaceDigestMismatch)
        ));
        tampered = left.clone();
        tampered.schema = "commandf.ecosystem-workspace/v2".to_owned();
        assert!(matches!(
            encode_workspace_machine(&snapshot, &tampered),
            Err(WorkspaceError::UnexpectedWorkspaceSchema)
        ));
        tampered = left.clone();
        tampered.snapshot_sha256 = "zz".repeat(32);
        assert!(matches!(
            encode_workspace_machine(&snapshot, &tampered),
            Err(WorkspaceError::InvalidWorkspaceDigest)
        ));
        tampered = left.clone();
        tampered.members.reverse();
        assert!(matches!(
            encode_workspace_machine(&snapshot, &tampered),
            Err(WorkspaceError::NoncanonicalWorkspace)
        ));
        tampered = left.clone();
        tampered
            .members
            .push(tampered.members.last().expect("member").clone());
        assert!(matches!(
            encode_workspace_machine(&snapshot, &tampered),
            Err(WorkspaceError::DuplicateMember { .. })
        ));
    }
}
