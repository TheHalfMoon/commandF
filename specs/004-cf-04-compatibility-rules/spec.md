# CF-04 Specification — Directional Compatibility Rules

Status: Implemented — founder review candidate

## Purpose

CF-04 converts deterministic CF-03 structural facts into deterministic compatibility findings. It answers **who is affected and how strongly** when a FHIR conformance package changes.

CF-04 is rule-based. It does not ask an AI model to decide compatibility, does not replace the FHIR Validator or IG Publisher, and does not perform terminology expansion or implication solving.

## User-visible command

```text
commandf classify <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --format json
```

The command reuses the exact CF-03 two-state loading and structural-diff path and performs no acquisition.

## Direction semantics

CF-04 uses two explicit directions:

- **producer** — whether output valid under the before contract can still be produced under the after contract;
- **consumer** — whether an implementation prepared to consume all before-valid output can safely consume all after-valid output.

A structural change may emit findings in one or both directions.

## Severity semantics

CF-04 v1 emits only:

- `BREAKING` — deterministic evidence establishes incompatibility for the stated direction;
- `RISKY` — compatibility may be affected but CF-04 cannot prove a safe subset/superset relation with its authorized semantics;
- `ADDITIVE` — the rule adds a contract artifact without invalidating an existing contract under that finding.

There is no implicit `SAFE` state and no AI-authored severity.

## Versioned rule corpus

The public ruleset identifier is `cf04-rules-v1`. Every finding carries a stable rule id, direction, severity, structural evidence, and deterministic message.

### Cardinality and maximum length

- `min` increase -> producer `BREAKING`;
- `min` decrease -> consumer `BREAKING`;
- `max` decrease -> producer `BREAKING`;
- `max` increase -> consumer `BREAKING`;
- `maxLength` tightening/relaxation follows the same producer/consumer variance;
- `*` is treated as an unbounded maximum.

### Type choices

CF-04 separates primitive/resource **type codes** from profile qualifiers:

- narrowing the allowed type-code set -> producer `BREAKING`;
- widening the allowed type-code set -> consumer `BREAKING`;
- replacing the type-code set with an incomparable set -> `BREAKING` in both directions;
- when the type-code set is unchanged but `profile`, `targetProfile`, or `aggregation` qualifiers change -> `RISKY` in both directions;
- when direct type-code comparison is unavailable -> `RISKY` rather than an invented `BREAKING` claim.

### Fixed values, patterns, and value bounds

- adding a fixed value -> producer `BREAKING`;
- removing a fixed value -> consumer `BREAKING`;
- replacing one fixed value with another -> `BREAKING` both;
- adding/removing a pattern follows the same directional constraint logic;
- replacing a pattern -> `RISKY` both because generic pattern implication is not proven;
- adding/removing a `minValue[x]`/`maxValue[x]` bound follows directional constraint logic;
- changing an existing generic value bound -> `RISKY` both unless a later typed semantic layer proves ordering.

### Terminology bindings

FHIR R4 binding strengths are ordered:

```text
example < preferred < extensible < required
```

- strengthening binding strength -> producer `BREAKING`;
- weakening binding strength -> consumer `BREAKING`;
- changing a bound ValueSet without proven set inclusion -> `RISKY` both;
- a present non-string or unrecognized `binding.strength` fails closed instead of degrading to `RISKY`.

ValueSet expansion/subset semantics are deferred to CF-07.

### Constraints

Constraints are compared by stable `constraint.key`:

- adding an error-level invariant -> producer `BREAKING`;
- removing an error-level invariant -> consumer `BREAKING`;
- warning-level additions/removals -> directional `RISKY`;
- warning -> error -> producer `BREAKING`;
- error -> warning -> consumer `BREAKING`;
- changing the expression/metadata of the same keyed invariant without a severity ordering proof -> `RISKY` both;
- duplicate constraint keys fail closed rather than allowing map overwrite;
- generic FHIRPath implication/equivalence solving is out of scope.

### Must Support and modifiers

FHIR defines Must Support separately from cardinality and leaves the concrete support obligation profile/context dependent. Any `mustSupport` change is therefore `RISKY` in both directions in CF-04 v1.

A change that newly marks an element as a modifier is consumer `BREAKING` because modifier semantics cannot be safely ignored; producer impact is `RISKY`. Other modifier-semantic changes are `RISKY` both.

### Slicing

CF-04 uses the R4 ordering:

```text
open < openAtEnd < closed
```

- increasing restrictiveness -> producer `BREAKING`;
- relaxing slicing -> consumer `RISKY`;
- unordered -> ordered -> producer `BREAKING`;
- ordered -> unordered -> consumer `RISKY`;
- discriminator/other slicing payload changes -> `RISKY` both;
- a present non-string or unrecognized `slicing.rules` fails closed.

### Resource, view, element, and residual facts

- resource addition -> `ADDITIVE` both;
- resource removal -> `BREAKING` both at the package-contract level;
- resourceType or selected StructureDefinition identity/target changes with direct contract impact -> `BREAKING` as defined by the rule corpus;
- filename/id/version-only facts -> `RISKY`;
- view addition -> `ADDITIVE`; view removal -> `RISKY`;
- element addition -> `RISKY` because CF-03 add evidence does not carry enough cardinality/modifier context to prove a directional subset;
- snapshot element removal -> producer `BREAKING`; differential element removal -> `RISKY` both;
- residual byte-only change -> `RISKY` both **only when no more precise structural fact exists for the same resource**;
- element/StructureDefinition fields whose subset relation is not established by CF-04 are `RISKY`, not guessed safe.

## Fail-closed coverage

CF-04 rejects:

- unsupported CF-03 report schemas;
- malformed evidence needed by a rule;
- unknown future CF-03 structural fields without a CF-04 rule;
- duplicate constraint keys;
- present unrecognized binding strengths;
- present unrecognized slicing rules.

This prevents silent under-classification when CF-03 evolves or malformed evidence reaches CF-04.

## Determinism and deduplication

Equivalent snapshot/differential element-field facts are deduplicated deterministically with snapshot evidence winning. Snapshot identities are pre-indexed before classification, avoiding a per-change nested scan.

Findings are stable-sorted. Identical structural-diff input produces byte-identical compatibility JSON. A no-op CF-03 report produces an empty finding list.

## Standards rationale

CF-04 v1 is grounded in published FHIR R4 profiling semantics, including:

- cardinality and profiling rules: `https://hl7.org/fhir/R4/profiling.html`;
- binding-strength meanings: `https://hl7.org/fhir/R4/terminologies.html`;
- modifier-extension safety semantics: `https://hl7.org/fhir/R4/extensibility.html`.

These sources constrain the rule rationale; HL7 does not publish commandF's `BREAKING/RISKY/ADDITIVE` labels.

## Acceptance

- public report schema and `cf04-rules-v1` are deterministic;
- producer/consumer direction is explicit on every finding;
- cardinality, maxLength, type-code, qualifier, fixed/pattern/value-bound, binding, constraint, Must Support, modifier, slicing, resource, view, element, residual, and fail-closed rules have regression coverage;
- corrupted cache behavior is proven at the CLI boundary;
- repeated classification is byte-identical;
- real independently resolved `hl7.fhir.r4.core@4.0.1` self-classification produces zero findings;
- CI uses committed `Cargo.lock` with `--locked`;
- no PHI fixtures;
- no CF-05+ behavior.

## Non-goals

- SARIF or quality-gate exit policy — CF-05;
- HL7/FHIR Validator differential oracle — CF-06;
- ValueSet/CodeSystem membership and set inclusion — CF-07;
- GitHub annotations — CF-08;
- FSH source mapping — CF-09;
- ecosystem blast radius — CF-11/12;
- mapping execution or semantic-loss runtime;
- AI-authored compatibility judgments.
