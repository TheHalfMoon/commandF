# Root Verified Cache Bytes

Status: SPEC_CANDIDATE

Issue: #40

## Problem

`inspect`, the shared diff family (`diff`, `classify`, and `check`), and the terminology root path verified a locked cache digest and then opened the cache path again. The bytes passed to inspection and archive comparison were not the bytes returned by the verification boundary.

## Contract

- Those root consumers obtain archive bytes through `PackageCache::read_verified`.
- A replaced cache object fails closed with a cache digest mismatch.
- Package identity, lock identity, and existing fail-closed behavior stay in force.
- Issue #36 still owns impact and the external oracle staging boundary.
- Issues #37 and #38 still own lockfile and local-mirror bounds.
- Issue #100 stays open. Historical artifact bytes remain unavailable.
- This candidate does not authorize CF-17 and does not change `verify_artifacts`.
