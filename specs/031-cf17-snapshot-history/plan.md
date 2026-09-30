# CF-17 Snapshot History Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/030-cf17-snapshot-comparison/` is merged as `4ca49836e7f1419f7555eb3504ee3694af494025`. Its plan names an ordered history of published snapshot comparisons as the next CF-17 package. History consumes comparison identities. It does not need historical Actions artifact bytes or the CF-06 pin change.

## Scope in

- Deterministic chain of published snapshot comparisons.

## Scope out

- Live registry ingestion.
- Compatibility classification.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is deterministic machine bytes for one history document, still without registry ingest or compatibility classification. Issue #100 and issue #15 stay open.
