# CF-17 Separate Closures Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/025-cf17-snapshot-identity/` is merged as `1f4e45405505406d3d57d47d4773bbb23559f366`. Its plan names separate package and canonical-reference closures as the next CF-17 package. This slice depends on frozen snapshot identity. It does not depend on historical Actions artifact bytes or the CF-06 pin change.

## Scope in

- Deterministic projection of two closure identities from one published snapshot and caller-supplied edges.

## Scope out

- Live registry ingestion.
- Archive traversal and CF-11 context extraction.
- Incremental cache measurement.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is a deterministic machine query over one frozen snapshot and these two closure identities. It still does not ingest a registry. Issue #100 and issue #15 stay open.
