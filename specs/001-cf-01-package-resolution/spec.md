# CF-01 — FHIR Package Resolution and Deterministic Locking

Status: Draft specification

## Problem

FHIR Implementation Guides depend on NPM-format packages and transitive dependencies. commandF cannot produce trustworthy diffs or breaking-change findings unless every compared artifact is resolved reproducibly to exact package bytes.

## User Outcome

A developer can declare one or more FHIR packages, resolve all transitive dependencies deterministically, cache the exact archives by content digest, and reproduce the same locked package set offline.

Target flow:

```bash
commandf pkg add hl7.fhir.us.core@6.1.0
commandf pkg verify
```

## Functional Requirements

1. Accept a package root in `package.name@version` form.
2. Resolve direct and transitive package dependencies.
3. Record exact selected package name and version.
4. Record a SHA-256 digest of the exact package archive bytes.
5. Produce a deterministic `commandf.lock` with stable ordering and serialization.
6. Cache archives content-addressed by digest.
7. Verify cached objects against the lockfile without network access.
8. Fail explicitly on incompatible requests for multiple concrete versions of the same package.
9. Reject unsupported aliases/local-path dependency constraints rather than guessing.
10. Validate archive package identity against requested/resolved identity before accepting it.
11. Read package metadata from `package/package.json` without extracting the archive into the worktree.
12. Keep acquisition behind a `PackageSource` abstraction so resolution logic is testable without network access.
13. Initial implementation supports exact versions and the FHIR package ecosystem's common `major.minor.x` patch wildcard; any wider version syntax must be added by explicit spec amendment.

## Success Criteria

- Two independent resolutions of the same fixture graph produce byte-identical lockfiles.
- A dependency conflict fails rather than using implicit last-writer-wins behavior.
- A corrupted cache object is detected by `pkg verify`.
- Resolver unit tests perform no network access.
- At least one real published FHIR package can be resolved end-to-end before CF-01 leaves Draft.
- The same real package can then be verified from cache with network unavailable.

## Edge Cases

- duplicate root request;
- transitive dependency requested by multiple roots;
- same package requested at incompatible versions;
- missing version;
- malformed package name;
- unsupported local/file/alias dependency syntax;
- archive missing `package/package.json`;
- archive identity mismatch;
- corrupt cached archive;
- registry metadata identity mismatch;
- prerelease versions when an `x` wildcard is used.

## Non-Goals

- FHIR resource validation;
- snapshot generation;
- canonical resource indexing;
- artifact diffing;
- package publishing;
- authenticated/private registries;
- commandF-owned package registry;
- dependency auto-upgrade or Renovate-like behavior;
- arbitrary npm package compatibility outside FHIR conformance packages.

## Security / Trust Requirements

- Registry metadata and archives are untrusted input.
- Do not extract package archives merely to read `package/package.json`.
- A package is not trusted solely because a registry returned it; archive identity and digest are recorded.
- Resolver conflicts fail closed.
- Cache verification is content-based, not path/name-based.

## Evidence Produced

`commandf.lock` must record enough information for later slices to prove exactly which package bytes were analyzed: root requests, selected package/version, source location, archive digest, and declared dependency constraints.

## Deferred Questions

- authenticated/private FHIR registries;
- stronger upstream integrity metadata beyond local SHA-256;
- CRMI release-manifest interoperability;
- lockfile signing;
- registry mirror policy.
