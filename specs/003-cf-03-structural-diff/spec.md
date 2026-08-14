# CF-03 Specification — Deterministic Structural Diff

Status: Implemented — founder review candidate

## Purpose

CF-03 adds a deterministic structural-diff stage above CF-02 inspection. It answers **what changed** between two explicitly supplied locked/cache states of the same FHIR package. It does not decide whether a change is breaking, risky, additive, safe, producer-facing, or consumer-facing; those judgments belong to CF-04.

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
5. Match canonical resources primarily by logical canonical URL when that URL is unique on both sides. If a URL occurs multiple times on either side, every member of that URL group must have a usable explicit canonical version and exact `url|version` identities are used. Missing usable versions in a multiplicity group fail closed.
6. Match non-canonical resources by unique `(resourceType,id)` when available, otherwise by filename. Ambiguous non-canonical match keys and duplicate package-root filenames fail explicitly.
7. Emit typed resource additions/removals, filename/version/resourceType/id/content-hash changes, and StructureDefinition structural changes.
8. For StructureDefinition resources, compare `snapshot` and `differential` as distinct views and match elements by exact `ElementDefinition.id`.
9. Compare a versioned structural field set rather than editorial prose. CF-03 v1 includes path/slicing/cardinality/type/contentReference/representation/fixed-pattern/default/min-max/maxLength/conditions/constraints/mustSupport/modifier/summary/binding/extension-related structural fields and selected StructureDefinition metadata.
10. Validate the JSON shape of fields CF-03 interprets before normalization. Malformed interpreted values fail with an explicit structural-field error rather than becoming valid-looking changes.
11. Preserve valid FHIR JSON primitive metadata forms: a required primitive string may be represented by its ordinary non-empty value or by a meaningful `_field` metadata object when the primitive value itself is absent. Empty or malformed primitive metadata fails closed.
12. Normalize known set-like arrays deterministically so irrelevant ordering does not produce a delta. Preserve ordering where FHIR semantics may depend on order; `extension[]` is order-preserving in CF-03 unless a future profile-aware layer establishes narrower unordered semantics.
13. Sort every emitted change by a stable deterministic key and serialize byte-identically for identical inputs.
14. Fail closed on malformed structural fields, ambiguous resource matching, duplicate element ids, unsupported inspection schema, cache corruption, archive-bound violations, or internal inventory disagreement.

## Change kinds

CF-03 v1 may emit only structural facts such as:

- `resource_added`
- `resource_removed`
- `resource_filename_changed`
- `resource_version_changed`
- `resource_type_changed`
- `resource_id_changed`
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

- synthetic tests cover resource matching, canonical-version groups, ambiguous fallback keys, duplicate filenames, element additions/removals, cardinality/type/binding/slicing/fixed-pattern changes, view changes, interpreted-field shape rejection, primitive-metadata compatibility, editorial-field exclusion, deterministic ordering, and no-op diff;
- CI runs with the committed `Cargo.lock` and `--locked`;
- the real smoke resolves and verifies published `hl7.fhir.r4.core@4.0.1` once, inspects that official state, copies the content-addressed cache and lock into explicit after-state paths, independently verifies the copied state, and proves CLI self-diff is empty;
- no PHI fixtures;
- no CF-04 classification behavior.

## Non-goals

- BREAKING/RISKY/ADDITIVE classification;
- producer/consumer direction;
- FHIR Validator judgments or snapshot generation;
- terminology semantic expansion/diff;
- profile-aware extension ordering semantics;
- FSH source mapping;
- ecosystem blast radius;
- mapping execution, CSIR, or AI authority.
