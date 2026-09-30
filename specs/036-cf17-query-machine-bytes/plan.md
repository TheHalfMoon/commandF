# CF-17 Query Machine Bytes Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/035-cf17-closure-machine-bytes/` is merged as `ef10a306d0b42a772ad8f69e9d4f3bae83db1a3e`. Its plan names canonical machine bytes for one closure query as the next CF-17 package. The query digest already exists. This slice exposes those bytes.

## Scope in

- Canonical encode and decode of one closure query.

## Scope out

- Live registry ingestion.
- Compatibility classification.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is canonical machine bytes for one published snapshot comparison, using the same verified-byte rule. It still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
