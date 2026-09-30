# CF-17 Closure Query

Status: SPEC_CANDIDATE

Slice: third CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- A query binds one published snapshot to one closure projection.
- The query SHA-256 covers schema, snapshot SHA-256, package-closure SHA-256, and canonical-closure SHA-256. It does not include itself.
- Repeating the same snapshot and closures returns the same query identity.
- A legitimate change to either closure identity changes the query identity.
- The closure schema must be `commandf.ecosystem-closure/v1`.
- A closure whose snapshot SHA-256 differs from the snapshot fails closed.
- Package members plus unrelated packages must be exactly the snapshot package set, including archive digests. A package in both lists fails closed.
- Resolved, unresolved, and ambiguous canonical lists are disjoint.
- Every unresolved canonical on the snapshot remains in the closure unresolved list.
- Stored closure digests must match a fresh digest of the same closure documents.
- `MUTABLE_CI` cannot be queried as published authority.
- Query input sizes over 10000 edges, packages, or canonicals per list fail closed.
- This slice does not download a registry or add a CLI command.
- It does not close issue #100 or issue #15, and it does not change the CF-06 production pin or `verify_artifacts`.
