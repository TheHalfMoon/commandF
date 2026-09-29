# Terminology Verified Cache Bytes

Status: SPEC_CANDIDATE

Issue: #35

## Problem

`TerminologyClosure::load` verified every lockfile cache object and then opened each cache path again with an independent filesystem read. The bytes parsed as terminology were not the bytes returned by the verification boundary.

## Contract

- Terminology loading obtains each locked archive through `PackageCache::read_verified`.
- Manifest identity, package identity, duplicate filename rejection, canonical matching, and fail-closed errors stay in force.
- A cache object whose bytes no longer match the locked digest is rejected as a cache digest mismatch.
- The consumed-byte check is separate from retained-authority historical artifact verification. It does not change `verify_artifacts`.
- This candidate does not close issues #36, #37, #38, #40, or #100.
- This candidate does not authorize CF-17.

## Required evidence

- A reader is invoked once per locked package, and the terminology URL comes from the bytes that reader returned.
- Replacing the cache object after `PackageCache::put` fails closed with a digest mismatch.
