# CF-17 Workspace Identity

Status: SPEC_CANDIDATE

Slice: fifteenth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- A workspace names member packages by `name` and `version` that already exist in one published snapshot.
- Membership is sorted by name, then version. Input order does not change the digest.
- An empty member list is an explicit empty workspace.
- A duplicate member, a member absent from the snapshot, a mutable snapshot, or more than 10000 members fails closed.
- The document field order is `members`, `schema`, `snapshot_sha256`. Member field order is `name`, `version`.
- The schema is `commandf.ecosystem-workspace/v1`. `workspace_sha256` is the digest of that document and is stored beside it.
- A branch name, filesystem path, clock, or compatibility label is not part of the identity.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
