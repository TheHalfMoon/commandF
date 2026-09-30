# CF-17 Cache Identity Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/028-cf17-source-lifecycle/` is merged as `2384be6483943dd8ba3311bc27c63aefeb98e266`. Its plan names content-addressed cache identity for a frozen snapshot as the next CF-17 package. Identity is required before any scale measurement. This slice does not measure scale and does not need historical Actions artifact bytes or the CF-06 pin change.

## Scope in

- Deterministic cache identity and fail-closed reuse.

## Scope out

- Filesystem cache storage.
- Scale, runtime, or memory measurement.
- Live registry ingestion.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is a deterministic comparison of two published snapshot identities. It records added, removed, and digest-changed packages and does not ingest a registry. Issue #100 and issue #15 stay open.
