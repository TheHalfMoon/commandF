# CF-17 Cache Identity

Status: SPEC_CANDIDATE

Slice: fifth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- A cache identity is the SHA-256 of schema, snapshot SHA-256, and engine schema. It does not include itself.
- The same snapshot and engine schema replay to the same cache identity and may be reused.
- A different snapshot digest or a different engine schema changes the cache identity and fails reuse.
- A stored cache digest that does not match the recomputed identity fails reuse.
- Engine schema is a non-empty token without whitespace, at most 256 characters.
- `MUTABLE_CI` cannot receive a published cache identity.
- This slice does not write a cache, measure scale, time a workload, or download a registry.
- It does not close issue #100 or issue #15, and it does not change the CF-06 production pin or `verify_artifacts`.
