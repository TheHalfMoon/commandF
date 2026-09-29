# Terminology Verified Cache Bytes Plan

Status: SPEC_CANDIDATE

## Scope in

- Bind `TerminologyClosure::load` to `PackageCache::read_verified`.
- Add the two deterministic regressions named in `spec.md`.
- Record this slice in the plan index as a candidate, not as CF-17 authority.

## Scope out

- Impact and oracle verified-byte staging (issue #36).
- Lockfile size bounds (issue #37).
- Local mirror archive bounds (issue #38).
- Root CLI archive consumers (issue #40).
- Retained historical artifact bytes (issue #100).
- AF-02 `verify_artifacts` and authority-path edits.

## Exit

Normal merge of this candidate after exact-head required checks pass. Issue #35 stays open until post-merge evidence is recorded and the issue body is still satisfied on the merge commit. Issues #36, #37, #38, #40, and #100 stay open. CF-17 stays unauthorized.
