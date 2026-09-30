# CF-17 Snapshot Machine Bytes Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/032-cf17-history-machine-bytes/` is merged as `be863ef1ae7e8fa9fde4a734f3430cc307887874`. Its plan names canonical machine bytes for one published snapshot as the next CF-17 package. The bytes are the snapshot document that package already hashes. This slice does not need historical Actions artifact bytes or the CF-06 pin change.

## Scope in

- Canonical encode and decode of one published snapshot.

## Scope out

- Live registry ingestion.
- Compatibility classification.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is canonical machine bytes for one published source-lifecycle record, using the same verified-byte rule. It still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
