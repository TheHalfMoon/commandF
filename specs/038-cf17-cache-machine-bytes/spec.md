# CF-17 Cache Machine Bytes

Status: SPEC_CANDIDATE

Slice: fourteenth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- Machine bytes are the compact UTF-8 JSON document already hashed as `cache_sha256`.
- Field order is `engine_schema`, `schema`, `snapshot_sha256`.
- The encoder identity is `commandf.ecosystem-cache-bytes/v1`. It is recorded beside the bytes and is not a second hashed wrapper.
- The same snapshot and engine schema replay to identical bytes.
- A different snapshot or engine schema changes the digest and cannot reuse the bytes.
- Whitespace, an extra field, or a document larger than 64 KiB fails closed.
- This slice does not store a filesystem cache, measure scale, or classify compatibility.
- It does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
