# Root Verified Cache Bytes Plan

Status: SPEC_CANDIDATE

## Scope in

- Replace verify-then-reopen in `inspect`, `build_diff_report`, and `build_terminology_report`.
- Prove the shared reader returns the verified bytes and rejects a replaced object.

## Scope out

- `commandf impact` and the HL7 JVM path (issue #36).
- Lockfile allocation bounds (issue #37).
- Local mirror archive bounds (issue #38).
- Historical retained artifact bytes (issue #100).
- CF-17 implementation.

## Exit

Normal merge after exact-head required checks. Issue #40 may close only after post-merge assurance evidence. Issues #36, #37, #38, and #100 stay open.
