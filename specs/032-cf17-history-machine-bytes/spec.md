# CF-17 History Machine Bytes

Status: SPEC_CANDIDATE

Slice: eighth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- Machine bytes are the canonical JSON document already hashed as `history_sha256`.
- Field order is `comparison_sha256s`, `engine_schema`, `first_before_snapshot_sha256`, `last_after_snapshot_sha256`, `schema`.
- JSON is compact UTF-8 with no extra whitespace. The schema inside the bytes is `commandf.ecosystem-snapshot-history/v1`.
- The encoder identity is `commandf.ecosystem-history-bytes/v1`. It is not a second hashed wrapper.
- The machine SHA-256 is the digest of those exact bytes and must equal the stored history digest.
- Parsing those bytes and writing them again produces the same bytes.
- A space, a reordered key, or a trailing token is not canonical and fails.
- An unsupported history schema, a short digest, a duplicate comparison digest, or an empty history that still names an endpoint fails closed.
- More than 1000 comparison identities, or more than 1 MiB of machine bytes, fails closed before the document is accepted.
- A broken comparison chain still fails in `project_snapshot_history` and therefore has no machine bytes.
- The document has no clock, path, locale, or compatibility field.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
