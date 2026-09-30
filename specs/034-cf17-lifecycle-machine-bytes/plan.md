# CF-17 Lifecycle Machine Bytes Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/033-cf17-snapshot-machine-bytes/` is merged as `60221fe0de7c0ee003a0fb9a724e9a0cfd7af959`. Its plan names canonical machine bytes for one published source-lifecycle record as the next CF-17 package. The bytes are the lifecycle document that package already hashes.

## Scope in

- Canonical encode and decode of one lifecycle record bound to one published snapshot.

## Scope out

- Live registry ingestion.
- Compatibility classification.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is canonical machine bytes for one published closure pair, using the same verified-byte rule. It still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
