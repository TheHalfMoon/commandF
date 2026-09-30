# CF-17 Cache Machine Bytes Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/037-cf17-comparison-machine-bytes/` is merged as `145a3c9b5f590e60fbbfc627dfdb09020421143c`. Its plan names canonical machine bytes for one cache identity as the next CF-17 package.

## Scope in

- Canonical encode and decode of one cache identity bound to one published snapshot.

## Scope out

- Filesystem cache storage and scale measurement.
- Live registry ingestion.
- Compatibility classification.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. Re-read the CF-17 playbook before naming another observatory package. This slice still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
