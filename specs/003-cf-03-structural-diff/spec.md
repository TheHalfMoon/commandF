# CF-03 Specification — Deterministic Structural Diff

Status: Draft execution specification

## Purpose

CF-03 adds a deterministic structural-diff stage above CF-02 inspection. It answers **what changed** between two separately locked versions of the same FHIR package. It does not decide whether a change is breaking, risky, additive, safe, producer-facing, or consumer-facing; those judgments belong to CF-04.

## User-visible command

```text
commandf diff <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --format json
```

Both sides are exact CF-01 lock/cache states. Diff performs no acquisition.

## Required behavior

1. Parse one FHIR package name and find exactly one selected version of that package in each supplied lockfile.
2. Verify both content-addressed cache objects before reading and independently recheck each archive digest before structural parsing.
3. Rebuild each resource inventory from package-root FHIR JSON rather than trusting `.index.json`.
4. Emit deterministic package/version/archive evidence for both sides.
5. Match canonical resources primarily by logical canonical URL when that URL is unique on both sides. If a URL occurs multiple times on either side, use exact `url|version` identities for that URL group. Canonical URL changes therefore appear as remove/add unless an exact identity remains.
6. Match non-canonical resources by unique `(resourceType,id)` when available, otherwise by filename. Ambiguous non-canonical match keys fail explicitly.
7. Emit typed resource additions/removals, filename/version/content-hash changes, and StructureDefinition structural changes.
8. For StructureDefinition resources, compare `snapshot` and `differential` as distinct views and match elements by exact `ElementDefinition.id`.
9. Compare a versioned structural field set rather than editorial prose. CF-03 v1 includes path/slicing/cardinality/type/contentReference/representation/fixed-pattern/default/min-max/maxLength/conditions/constraints/mustSupport/modifier/summary/binding/extension-related structural fields and selected StructureDefinition metadata.
10. Normalize known set-like arrays deterministically so irrelevant ordering does not produce a delta; preserve ordering where FHIR semantics may depend on order.
11. Sort every emitted change by a stable deterministic key and serialize byte-identically for identical inputs.
12. Fail closed on malformed structural fields, ambiguous resource matching, duplicate element ids, unsupported inspection schema, cache corruption, or archive-bound violations.

## Change kinds

CF-03 v1 may emit only structural facts such as:

- `resource_added`
- `resource_removed`
- `resource_filename_changed`
- `resource_version_changed`
- `resource_bytes_changed`
- `structure_field_changed`
- `view_added`
- `view_removed`
- `element_added`
- `element_removed`
- `element_field_changed`

No severity or compatibility label is permitted in CF-03 output.

## Determinism

Identical before/after bytes, lockfiles, and tool version must produce byte-identical JSON. Comparing a package state with itself must produce an empty change list.

## Acceptance

- synthetic tests cover resource matching, canonical-version groups, ambiguous fallback keys, element additions/removals, cardinality/type/binding/slicing/fixed-pattern changes, view changes, editorial-field exclusion, deterministic ordering, and no-op diff;
- CI runs with the committed `Cargo.lock` and `--locked`;
- a real smoke resolves/verifies `hl7.fhir.r4.core@4.0.1` into two independent cache/lock states and proves self-diff is empty;
- no PHI fixtures;
- no CF-04 classification behavior.

## Non-goals

- BREAKING/RISKY/ADDITIVE classification;
- producer/consumer direction;
- FHIR Validator judgments or snapshot generation;
- terminology semantic expansion/diff;
- FSH source mapping;
- ecosystem blast radius;
- mapping execution, CSIR, or AI authority.
