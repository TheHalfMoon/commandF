# CF-17 Query Machine Bytes

Status: SPEC_CANDIDATE

Slice: twelfth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- Machine bytes are the compact UTF-8 JSON document already hashed as `query_sha256`.
- Field order is `canonical_closure_sha256`, `package_closure_sha256`, `schema`, `snapshot_sha256`.
- The encoder identity is `commandf.ecosystem-query-bytes/v1`. It is recorded beside the bytes and is not a second hashed wrapper.
- The query binds one published snapshot and both closure digests. A different closure pair fails.
- Parsing those bytes and writing them again produces the same bytes.
- Whitespace, an extra field, an unknown schema, or a document larger than 1 MiB fails closed.
- The document has no clock, path, locale, or compatibility field.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
