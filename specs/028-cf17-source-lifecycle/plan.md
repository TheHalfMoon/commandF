# CF-17 Source Lifecycle Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/027-cf17-closure-query/` is merged as `296641b021269adb35506867b186a6b79cfa799a`. Its plan names source lifecycle recorded on the snapshot as the next CF-17 package. The record is bound to an existing snapshot identity. It does not need historical Actions artifact bytes or the CF-06 pin change.

## Scope in

- Deterministic source lifecycle projection and adoption refusal for stale or withdrawn sources.

## Scope out

- Network refresh.
- Live registry ingestion.
- Incremental cache measurement.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package remains inside the observatory: content-addressed cache identity for a frozen snapshot, still without registry ingest and still without a scale claim. Issue #100 and issue #15 stay open.
