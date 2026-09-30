# CF-17 Workspace Machine Bytes

Status: SPEC_CANDIDATE

Slice: sixteenth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- Machine bytes are the compact UTF-8 JSON document already hashed as `workspace_sha256`.
- Document field order is `members`, `schema`, `snapshot_sha256`.
- Member field order is `name`, `version`.
- Members are strictly ascending by name then version. A duplicate or a descending pair fails closed.
- The encoder identity is `commandf.ecosystem-workspace-bytes/v1`. It is recorded beside the bytes and is not a second hashed wrapper.
- The machine SHA-256 equals the stored workspace digest.
- The document binds the published snapshot digest. A different snapshot or a non-64-hex digest fails.
- Parsing those bytes and writing them again produces the same bytes.
- Reordering the same members before projection does not change the bytes.
- A changed member set changes the bytes.
- Whitespace, reordered keys, an extra field, or trailing bytes fail.
- An unknown workspace schema fails closed.
- More than 10000 members, a name or version longer than 2048 bytes, or a document larger than 1 MiB fails before unbounded acceptance.
- The document has no clock, path, locale, branch, or compatibility field.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
