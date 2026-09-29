# Impact and Oracle Verified Bytes Plan

Status: SPEC_CANDIDATE

## Scope in

- Bind impact archive consumption to `PackageCache::read_verified`.
- Stage oracle inputs from verified bytes into a private temporary directory.
- Prove replacement of the original cache object cannot change the staged bytes.

## Scope out

- Lockfile allocation bounds (issue #37).
- Local mirror archive bounds (issue #38).
- Historical retained artifact bytes (issue #100).
- CF-06 production oracle pin changes.
- CF-17 implementation.

## Exit

Normal merge after exact-head required checks. Issue #36 may close only after post-merge assurance evidence. Issues #37, #38, and #100 stay open. CF-17 stays unauthorized until the remaining Wave -1 blockers for that slice are closed or explicitly deferred.
