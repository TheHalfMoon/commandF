# CF-03 Implementation Plan

Status: Implemented — founder review candidate

## Architecture

CF-03 stays inside the existing `commandf-pkg` trust boundary and `commandf` CLI. No new workspace crate was required.

The bounded package-root resource scanner from CF-02 is one reusable internal helper consumed by both inspection and diff, preserving the same archive limits and exclusion rules. This is a behavior-preserving CF-02 refactor, not a new acquisition path.

## Inputs

The CLI receives one validated package name plus explicit before/after lock and cache paths. Each lock must contain exactly one selected version of that package. Both selected cache objects are verified through CF-01, and both archives are independently rehashed by the structural parser before comparison.

`commandf diff` performs no acquisition.

## Resource matching

Deterministic match keys are built in this order:

1. canonical URL group;
2. exact `url|version` identity when one URL has multiplicity on either side;
3. unique `resourceType/id` for non-canonical resources;
4. filename fallback.

If canonical multiplicity exists, every member of that URL group must have a non-empty explicit canonical version; otherwise matching fails closed instead of mixing bare and version-qualified identities. Any residual ambiguous non-canonical key also fails. Duplicate package-root resource filenames fail explicitly so inspection and raw structural inventories cannot diverge.

## StructureDefinition comparison

For a matched StructureDefinition:

- compare selected resource-level structural metadata: `kind`, `abstract`, `type`, `baseDefinition`, `derivation`, `fhirVersion`, `context`, and `contextInvariant` when present;
- treat `snapshot` and `differential` as separate named views;
- match elements by exact `ElementDefinition.id`;
- emit add/remove when an id is present on only one side;
- compare only the CF-03 structural-field set for matched ids.

Structural element fields include:

- `path`, `sliceName`, `sliceIsConstraining`, `representation`, `slicing`;
- `min`, `max`, `contentReference`, `type`;
- `defaultValue[x]`, `meaningWhenMissing`, `orderMeaning`;
- `fixed[x]`, `pattern[x]`, `minValue[x]`, `maxValue[x]`, `maxLength`;
- `condition`, `constraint`;
- `mustSupport`, `isModifier`, `isModifierReason`, `isSummary`;
- `binding`;
- `extension` when present on the ElementDefinition.

Editorial-only fields such as `short`, `definition`, `comment`, `requirements`, `alias`, `example`, and `mapping` are not emitted as StructureDefinition structural-field changes in CF-03. Exact resource-byte hash changes remain visible separately as `resource_bytes_changed`.

## Shape validation and FHIR primitive metadata

CF-03 validates the shape of fields it interprets **before** normalization. It is not a general FHIR validator; the purpose is to stop malformed values from being canonicalized into valid-looking structural deltas.

Enforced CF-03-owned shapes include selected StructureDefinition metadata, cardinality values, boolean flags, `slicing`, `binding`, `type`, `constraint`, `extension`, and `maxLength`.

Required primitive string fields such as `ElementDefinition.type.code` and `constraint.key` accept either a normal non-empty string value or meaningful FHIR JSON `_field` primitive metadata when the primitive value is absent. Empty/malformed metadata fails closed. This preserves official R4 extension-only primitive shapes such as `_code` while retaining malformed-input rejection.

## Normalization

Canonicalize JSON objects recursively. Normalize fields known to behave as sets in CF-03:

- `representation`, `condition`, and `contextInvariant`: sort values;
- `type`: sort by canonical content and sort `profile`, `targetProfile`, and `aggregation` lists inside each type;
- `constraint`: sort by constraint key plus canonical content.

Preserve array order where semantics may depend on order. In particular, CF-03 intentionally preserves `extension[]` ordering; it does not globally treat every extension collection as a set. A future profile-aware layer may apply narrower unordered semantics only when the governing profile/slicing contract proves them.

## Output model

`StructuralDiffReport` contains schema, package name, before/after package evidence, and ordered `StructuralChange` entries. Each change contains a typed change kind, stable resource key, before/after filenames where relevant, optional StructureDefinition view, optional element id, optional changed field, and optional normalized before/after JSON values.

Resource-level facts include add/remove plus filename, canonical version, resourceType, id, and exact resource-byte SHA-256 changes. CF-03 contains no severity or compatibility field.

## CLI

```text
commandf diff <package-name> \
  --before-lock before.lock --before-cache before-cache \
  --after-lock after.lock --after-cache after-cache \
  --format json
```

All four before/after path options are required in v1 to avoid hidden workspace assumptions.

## Validation

Synthetic and CLI tests prove:

- byte-stable no-op self-diff;
- unique canonical matching across canonical-version changes;
- `url|version` matching for multi-version canonical groups and fail-closed missing group versions;
- fail-closed ambiguous non-canonical ids and duplicate archive filenames;
- view and element additions/removals;
- cardinality/type/binding/slicing/fixed structural changes;
- malformed interpreted structural shapes fail closed;
- valid primitive `_field` metadata forms remain accepted;
- editorial-field exclusion;
- set-like normalization for representation, condition, contextInvariant, constraint, and type/profile/targetProfile/aggregation ordering;
- extension ordering remains structural;
- CLI usage errors, absent packages, corrupt before/after caches, and successful offline diff.

Real CI independently resolves and verifies published `hl7.fhir.r4.core@4.0.1` into two distinct cache/lock states, inspects the before state, then runs the real CLI self-diff and requires an empty change list. The second independent resolution is intentional reproducibility evidence, even though it makes the real-package gate depend on registry availability.

No external FHIR validator is required for CF-03 because this slice computes structural facts rather than conformance judgments. Differential oracle work begins in CF-06.
