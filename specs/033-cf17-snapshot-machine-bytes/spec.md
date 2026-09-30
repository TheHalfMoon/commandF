# CF-17 Snapshot Machine Bytes

Status: SPEC_CANDIDATE

Slice: ninth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- Machine bytes are the compact UTF-8 JSON document already hashed as `snapshot_sha256`.
- Document field order is `schema`, `packages`, `unresolved_canonicals`.
- Package field order is `archive_sha256`, `mutability`, `name`, `source_id`, `version`.
- The encoder identity is `commandf.ecosystem-snapshot-bytes/v1`. It is recorded beside the bytes and is not a second hashed wrapper.
- The machine SHA-256 equals the stored snapshot digest.
- Parsing those bytes and writing them again produces the same bytes.
- Reordering the same packages or unresolved canonicals before projection does not change the bytes.
- A changed archive digest changes the bytes.
- Whitespace, reordered keys, an extra field, or trailing bytes fail.
- An unknown snapshot schema, a duplicate package, a duplicate unresolved canonical, a digest mismatch, or mutable CI fails closed.
- More than 10000 packages or unresolved canonicals, a string longer than 8192 bytes, or a document larger than 16 MiB fails closed.
- The document has no clock, path, locale, or compatibility field.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
