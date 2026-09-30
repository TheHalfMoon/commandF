# CF-17 Lifecycle Machine Bytes

Status: SPEC_CANDIDATE

Slice: tenth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- Machine bytes are the compact UTF-8 JSON document already hashed as `lifecycle_sha256`.
- Document field order is `schema`, `snapshot_sha256`, `sources`.
- Source field order is `source_id`, `state`.
- Source records are ordered by `source_id` ascending. A descending or duplicate source order is rejected.
- Allowed states remain `CURRENT`, `STALE`, and `WITHDRAWN`. Stale and withdrawn records stay observable. Adoption still requires `CURRENT`.
- The encoder identity is `commandf.ecosystem-lifecycle-bytes/v1`. It is recorded beside the bytes and is not a second hashed wrapper.
- The machine SHA-256 equals the stored lifecycle digest.
- The bytes bind the published snapshot digest. A different snapshot identity fails.
- Parsing those bytes and writing them again produces the same bytes.
- Reordering the same sources before projection does not change the bytes.
- A changed lifecycle state changes the bytes.
- Whitespace, reordered keys, an extra field, or trailing bytes fail.
- An unknown schema, a duplicate source, a digest mismatch, or a mutable snapshot fails closed.
- More than 10000 sources, a string longer than 8192 bytes, or a document larger than 16 MiB fails closed.
- The document has no clock, path, locale, or compatibility field.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
