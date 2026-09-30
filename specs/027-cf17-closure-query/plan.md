# CF-17 Closure Query Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/026-cf17-separate-closures/` is merged as `5846a9742ba21b6bbd45138dd7c31ff25792fe60`. Its plan names a deterministic machine query over one frozen snapshot and the two closure identities as the next CF-17 package. The query reads those identities. It does not need historical Actions artifact bytes or the CF-06 pin change.

## Scope in

- Fail-closed binding query and its own digest.

## Scope out

- CLI surface.
- Live registry ingestion.
- Incremental cache measurement.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is source lifecycle recorded on the snapshot, still without registry ingest. Issue #100 and issue #15 stay open.
