# CF-03 Implementation Plan

Status: Implemented — convergence candidate

## Architecture

CF-03 stays inside the existing `commandf-pkg` trust boundary and `commandf` CLI. No new workspace crate was required.

The bounded package-root resource scanner from CF-02 is now a reusable internal helper consumed by both inspection and diff, preserving the same archive limits and exclusion rules. This is a behavior-preserving CF-02 refactor, not a new acquisition path.

## Inputs

The CLI receives one validated package name plus explicit before/after lock and cache paths. Each lock must contain exactly one selected version of that package. Both selected cache objects are verified through CF-01, and both archives are independently rehashed by the structural parser before comparison.

`commandf diff` performs no acquisition.

## Resource matching

Deterministic match keys are built in this order:

1. canonical URL group;
2. exact `url|version` identity when one URL has multiplicity on either side;
3. unique `resourceType/id` for non-canonical resources;
4. filename fallback.

Any residual ambiguous non-canonical key fails rather than guessing. Duplicate package-root resource filenames also fail explicitly so the inspection inventory and raw structural inventory cannot diverge.

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

## Normalization

Canonicalize JSON objects recursively. Normalize fields known to behave as sets:

- `representation` and `condition`: sort scalar values;
- `type`: sort by canonical content and sort `profile`, `targetProfile`, and `aggregation` lists inside each type;
- `constraint`: sort by constraint key plus canonical content.

Preserve array order for fields where order may be semantically meaningful, especially slicing structures and arbitrary fixed/pattern values.

## Output model

`StructuralDiffReport` contains schema, package name, before/after package evidence, and ordered `StructuralChange` entries.

Each change contains:

- typed change kind;
- stable resource key;
- before/after filenames where relevant;
- optional StructureDefinition view;
- optional element id;
- optional changed field;
- optional normalized before/after JSON values.

Resource-level structural facts include add/remove plus filename, canonical version, resourceType, id, and exact resource-byte SHA-256 changes. CF-03 contains no severity or compatibility field.

## CLI

```text
commandf diff <package-name> \
  --before-lock before.lock --before-cache before-cache \
  --after-lock after.lock --after-cache after-cache \
  --format json
```

All four before/after path options are required in v1 to avoid hidden workspace assumptions.

## Validation

Synthetic tests use generated package archives and prove:

- byte-stable no-op self-diff;
- unique canonical matching across canonical-version changes;
- `url|version` matching for multi-version canonical groups;
- fail-closed ambiguous non-canonical ids and duplicate archive resource filenames;
- view and element additions/removals;
- cardinality/type/binding/slicing/fixed structural changes;
- editorial-field exclusion;
- set-like normalization for representation, condition, constraint, type/profile/targetProfile/aggregation ordering.

Real CI resolves and verifies published `hl7.fhir.r4.core@4.0.1` into two independent lock/cache states, runs `commandf diff` through the CLI, and requires an empty change list.

No external FHIR validator is required for CF-03 because this slice computes structural facts rather than conformance judgments. Differential oracle work begins in CF-06.
