# Impact and Oracle Verified Bytes

Status: SPEC_CANDIDATE

Issue: #36

## Problem

`commandf impact` verified cache digests and then read the cache paths again. `commandf oracle` did the same, and it passed those mutable cache paths to the external HL7 JVM adapter.

## Contract

- Impact consumes the bytes returned by `PackageCache::read_verified`.
- Oracle reads each required archive once through `PackageCache::read_verified`.
- CommandF structural comparison uses those same byte buffers.
- The JVM receives private staged copies of those buffers, not the original cache paths.
- Replacing a cache object after the verified read does not change the staged bytes.
- The staged directory is removed when the invocation ends.
- CF-06 core package identity stays `hl7.fhir.r4.core` `4.0.1`.
- Lockfile bounds remain issue #37. Local mirror bounds remain issue #38.
- Issue #100 stays open. This candidate does not authorize CF-17 and does not change `verify_artifacts`.
