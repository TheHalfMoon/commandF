# CF-17 Snapshot Identity Plan

Status: SPEC_CANDIDATE

## Why this unit

Wave -1 verified-byte and bounded-input issues #35, #36, #37, #38, and #40 are closed on canonical main. CF-17's first package needs frozen snapshot identity. It does not need the unavailable historical Actions artifact bytes from issue #100, and it does not need the CF-06 pin change blocked by issue #15.

## Scope in

- Deterministic snapshot projection and published-authority rejection of mutable CI.

## Scope out

- Live registry ingestion.
- Package and canonical-reference graph construction.
- Incremental cache measurement.
- Issue #100 byte recovery and issue #15 pin qualification.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is separate package and canonical-reference closures. Issue #100 and issue #15 stay open.
