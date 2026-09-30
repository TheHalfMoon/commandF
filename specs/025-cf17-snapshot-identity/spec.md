# CF-17 Snapshot Identity

Status: SPEC_CANDIDATE

Slice: first CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- A snapshot records package name, version, archive SHA-256, source identity, and source mutability.
- Package closure and unresolved canonical witnesses are separate fields.
- Canonical JSON is deterministic. The snapshot SHA-256 is the digest of that JSON and does not include itself.
- Reordering the same inputs does not change the snapshot identity.
- Changing an archive digest changes the snapshot identity.
- `MUTABLE_CI` may be retained as an observation. It cannot pass `require_published_authority`.
- Duplicate package identity, a short digest, or a duplicate unresolved canonical fails closed.
- This slice does not download a registry, host packages, or measure scale.
- It does not close issue #100 or issue #15, and it does not change the CF-06 production pin or `verify_artifacts`.
