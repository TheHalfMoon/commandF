# CF-17 History Machine Bytes Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/031-cf17-snapshot-history/` is merged as `87676ac4d9b584d20be2948d9ac2339cd8dfe2b3`. Its plan names deterministic machine bytes for one history document as the next CF-17 package. The bytes are the history document that package already hashes. This slice does not need historical Actions artifact bytes or the CF-06 pin change.

## Scope in

- Canonical encode and decode of one qualified snapshot history.

## Scope out

- Live registry ingestion.
- Compatibility classification.
- A second document whose digest disagrees with `history_sha256`.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is canonical machine bytes for one published snapshot, using the same verified-byte rule. It still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
