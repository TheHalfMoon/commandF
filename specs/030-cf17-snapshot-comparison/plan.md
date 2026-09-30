# CF-17 Snapshot Comparison Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/029-cf17-cache-identity/` is merged as `11e3bf80bf8ac168cd1c85b9ffce5958c8fbe87d`. Its plan names a deterministic comparison of two published snapshot identities as the next CF-17 package. The comparison reads already projected snapshots. It does not need historical Actions artifact bytes or the CF-06 pin change.

## Scope in

- Deterministic directional comparison of two published snapshots, with optional bound closure and lifecycle witnesses.

## Scope out

- Live registry ingestion.
- Compatibility classification, consumer impact, and oracle decisions.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is an ordered history of published snapshot comparisons. It still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
