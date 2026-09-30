# CF-17 Comparison Machine Bytes Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/036-cf17-query-machine-bytes/` is merged as `17aed3d7c3d97fc2b8987e9f066e470bc5424d97`. Its plan names canonical machine bytes for one published snapshot comparison as the next CF-17 package.

## Scope in

- Canonical encode and decode of one snapshot comparison.

## Scope out

- Live registry ingestion.
- Compatibility classification.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is canonical machine bytes for one cache identity, using the same verified-byte rule. It still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
