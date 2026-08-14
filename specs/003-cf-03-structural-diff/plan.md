# CF-03 Implementation Plan

Status: Draft implementation plan

## Architecture

Keep CF-03 inside the existing `commandf-pkg` trust boundary and `commandf` CLI. Do not add a workspace crate unless an independently exercised boundary becomes necessary.

Refactor the bounded package-root resource scanner from CF-02 into a reusable internal helper so both inspection and diff consume the same archive limits and exclusion rules. This is a behavior-preserving CF-02 refactor, not a new acquisition path.

## Inputs

The CLI receives one package name plus explicit before/after lock and cache paths. Each lock selects one exact version of the package. Both archives are verified through CF-01 and rehashed by the structural parser.

## Resource matching

Build deterministic match keys in this order:

1. canonical URL group;
2. exact `url|version` identity when one URL has multiplicity on either side;
3. unique `resourceType/id` for non-canonical resources;
4. filename fallback.

Any residual ambiguous non-canonical key fails rather than guessing.

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

Editorial-only fields such as `short`, `definition`, `comment`, `requirements`, `alias`, `example`, and `mapping` are not structural changes in CF-03.

## Normalization

Canonicalize JSON objects recursively. Normalize fields known to behave as sets:

- `representation` and `condition`: sort scalar values;
- `type`: sort by type code plus canonical content, and sort profile/targetProfile/aggregation lists inside each type;
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

CF-03 contains no severity field.

## CLI

```text
commandf diff <package-name> \
  --before-lock before.lock --before-cache before-cache \
  --after-lock after.lock --after-cache after-cache \
  --format json
```

All four before/after path options are required in v1 to avoid hidden workspace assumptions.

## Validation

Synthetic tests use permitted generated package archives. Real CI resolves the same published R4 core package into two isolated lock/cache states and proves an empty self-diff.

No external FHIR validator is required for CF-03 because the slice computes structural deltas rather than conformance judgments. Differential oracle work begins in CF-06.
