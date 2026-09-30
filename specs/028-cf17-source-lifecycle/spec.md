# CF-17 Source Lifecycle

Status: SPEC_CANDIDATE

Slice: fourth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- Lifecycle is an explicit caller record bound to one published snapshot SHA-256. It does not read a clock.
- The source set is exactly the set of `source_id` values on the snapshot. A missing or extra source fails closed.
- Duplicate source identity fails closed.
- `CURRENT`, `STALE`, and `WITHDRAWN` are the only states.
- `STALE` and `WITHDRAWN` may be retained. `require_current_sources` rejects them.
- Changing a source state changes the lifecycle SHA-256.
- Reordering the same records does not change the lifecycle SHA-256.
- `MUTABLE_CI` cannot receive a lifecycle projection for published authority.
- More than 10000 source records fail closed.
- This slice does not download a registry or refresh a source from the network.
- It does not close issue #100 or issue #15, and it does not change the CF-06 production pin or `verify_artifacts`.
