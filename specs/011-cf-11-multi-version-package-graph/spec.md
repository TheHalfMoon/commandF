# CF-11 Specification — Multi-Version Package Graph

Status: foundation correction candidate

## Problem

CF-01 currently flattens the selected dependency closure by package **name**. A second request for the same name at a different concrete version fails with `VersionConflict`, even when the requests originate from different dependency branches.

CF-10's frozen real-IG eligibility sweep (`31856586654`) proved this model cannot represent 5 of 6 selected public package states. No semantic CF-10 result was executed before this foundation gap was identified.

## Goal

Represent the resolved package closure using exact package identities:

```text
(package name, concrete version)
```

while preserving deterministic selection, provenance, content digests, bounded acquisition, and fail-closed downstream ambiguity handling.

## Normative behavior

1. Every dependency request is resolved independently under its declared constraint.
2. Exact constraints select exactly that version.
3. Patch wildcards retain CF-01 behavior: select the highest stable matching patch from the source metadata.
4. The selected concrete identity is `(name, version)`.
5. A concrete identity already present in the closure is deduplicated and is not downloaded/expanded again.
6. A different concrete version of the same package name is a distinct closure node and MUST NOT be rejected solely because another version of that name is already present.
7. Resolution MUST NOT silently replace one selected version with another, apply last-writer-wins, or globally coerce two branch-local requests to one version.
8. Lockfile package ordering remains deterministic by `(name, version)`.
9. Lock schema v1 remains unchanged in CF-11. `LockedPackage.dependencies` continues to record the package manifest's declared dependency constraints; CF-11 does not claim that schema v1 is an explicit resolved-edge graph.
10. Cache verification remains digest-based for every locked concrete package identity.
11. Existing commands that select a locked package by name only MUST remain fail-closed if multiple locked versions make that selection ambiguous. CF-11 does not invent a semantic/root-selection rule for those commands.
12. `inspect name@exact-version` MUST continue to select one exact locked identity.
13. Existing terminology canonical ambiguity/duplicate protections remain unchanged.

## Explicit non-goals

CF-11 does not:

- change CF-03/04/05 compatibility semantics;
- choose benchmark cases or alter CF-10's frozen corpus;
- add npm-style hoisting or dedupe optimization;
- solve peer/optional dependencies;
- add arbitrary semver ranges beyond existing exact + patch wildcard support;
- change registry trust, archive bounds, cache layout, or package identity validation;
- change terminology canonical resolution;
- change CLI diff/check/oracle package-selection syntax;
- introduce AI authority.

## Acceptance criteria

### A. Multi-version graph

A synthetic graph where one branch requires `acme.dep@1.0.0` and another requires `acme.dep@2.0.0` MUST resolve successfully and lock both exact identities.

### B. Same identity deduplication

If multiple branches resolve to the same concrete `(name, version)`, that package appears exactly once in the lock.

### C. Request-local wildcard determinism

An exact request and a patch-wildcard request for the same name may resolve to different concrete versions; both identities are retained. Equivalent root-order permutations produce byte-identical lockfiles.

### D. Cycles

A dependency cycle that returns to an already-selected exact identity terminates by identity deduplication rather than repeatedly expanding the package.

### E. Downstream ambiguity remains fail-closed

A name-only locked-package selector encountering multiple versions MUST error rather than choose one silently.

### F. Regression

All existing workspace tests, CF-08/CF-09 security regressions, real FHIR smoke, and CF-06 oracle gates remain green.

### G. Real foundation proof

Before CF-11 convergence, at least one package state that failed CF-10 solely due to the old name-level `VersionConflict` must complete `pkg resolve` + `pkg verify` with multiple same-name concrete versions visible in the lock. CF-10 itself remains frozen until CF-11 is canonical.

## Authority boundary

A green CF-11 proves only that commandF can represent a multi-version transitive package closure deterministically. It does not prove semantic compatibility, terminology correctness, or that every real FHIR package graph is supported.
