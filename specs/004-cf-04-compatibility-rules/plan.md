# CF-04 Implementation Plan

Status: Implemented — founder review candidate

## Architecture

CF-04 stays inside the existing `commandf-pkg` crate and `commandf` CLI. No new workspace crate was required.

The public classifier consumes the in-memory `StructuralDiffReport` produced by CF-03. It does not reopen archives or duplicate structural parsing. CF-03 remains the single authority for deterministic resource matching and structural facts.

The public path is layered as:

1. validate the CF-03 schema and CF-04-owned evidence codes;
2. pre-index snapshot field identities and resources with precise structural facts;
3. suppress duplicate differential evidence and subsumed byte-only facts;
4. dispatch each remaining structural fact through the versioned rule corpus;
5. stable-sort the resulting findings.

## Public model

`CompatibilityReport` contains:

- schema version `1`;
- ruleset `cf04-rules-v1`;
- package name;
- before/after package evidence copied from CF-03;
- ordered compatibility findings.

Each finding contains:

- stable `rule_id`;
- severity (`BREAKING`, `RISKY`, `ADDITIVE`);
- direction (`producer`, `consumer`);
- source `StructuralChangeKind`;
- resource key and filenames;
- optional view, element id, and field;
- normalized before/after evidence;
- deterministic explanatory message.

## Classifier boundary

Public API:

```text
classify_structural_diff(&StructuralDiffReport)
    -> Result<CompatibilityReport, CompatibilityError>
```

The classifier accepts only CF-03 schema v1. Unsupported schema or unknown future structural fields fail closed.

Before rule dispatch it also fails closed on duplicate `constraint.key` values and on present non-string or unknown `binding.strength` / `slicing.rules` values.

## Directional variance

Producer compatibility detects tightening of what may be emitted. Consumer compatibility detects widening or relaxation that may allow after-valid output outside the assumptions of a before-consumer.

Cardinality and maximum-length rules compare ordered bounds, including `*` as unbounded maximum.

## Type rules

CF-03 already normalizes type arrays. CF-04 separates:

- direct type-code set variance, which can support `BREAKING` claims;
- profile/targetProfile/aggregation qualifier changes, which remain `RISKY` when the type-code set itself is unchanged.

This avoids false `BREAKING` claims where qualifier subset/overlap has not been proven.

## Binding rules

Use the FHIR R4 strength order:

```text
example < preferred < extensible < required
```

Strengthening is producer-breaking and weakening is consumer-breaking.

A bound ValueSet change emits `RISKY` findings for both directions because terminology set inclusion is CF-07 work. Unknown present binding strengths fail closed.

## Constraint rules

Constraint arrays are keyed by `constraint.key`. Duplicate keys fail closed.

- add error invariant -> producer `BREAKING`;
- remove error invariant -> consumer `BREAKING`;
- add/remove warning -> directional `RISKY`;
- warning -> error -> producer `BREAKING`;
- error -> warning -> consumer `BREAKING`;
- same-key expression/metadata rewrite -> `RISKY` both unless a later oracle proves implication/equivalence.

No FHIRPath implication solver is introduced.

## Slicing rules

Use `open < openAtEnd < closed` as increasing restrictiveness and inspect `ordered` when present.

- more restrictive -> producer `BREAKING`;
- relaxed -> consumer `RISKY`;
- false -> true ordering -> producer `BREAKING`;
- true -> false -> consumer `RISKY`;
- discriminator/residual slicing change -> `RISKY` both;
- unknown present `slicing.rules` -> fail closed.

## Must Support and modifiers

Any Must Support change is `RISKY` both because R4 support obligations are context-dependent and distinct from cardinality.

New `isModifier=true` emits consumer `BREAKING` plus producer `RISKY`. Other modifier-semantic changes remain `RISKY` both.

## Residual structural facts

CF-04 does not silently drop CF-03 facts:

- resource addition/removal and direct identity/target rules are classified explicitly;
- filename/id/version, view, element, default, representation, extension-order, and other non-proven relations remain explicit `RISKY`/`ADDITIVE`/`BREAKING` according to the stable rule;
- resource byte changes emit residual `RISKY` findings **only if the same resource has no more precise structural fact**. A precise structural fact subsumes the generic byte-hash fact.

## Snapshot/differential deduplication

The public classifier precomputes a `BTreeSet` of snapshot element-field identities before the classification loop. Equivalent differential facts are removed through indexed membership checks and snapshot evidence wins.

This replaces public-path nested per-change scanning and avoids O(n²) deduplication behavior on large real diffs.

## CLI

```text
commandf classify <package-name> \
  --before-lock before.lock --before-cache before-cache \
  --after-lock after.lock --after-cache after-cache \
  --format json
```

`diff` and `classify` share the same internal `build_diff_report` two-state loader. `classify` performs no package acquisition.

CF-04 intentionally does **not** change the process exit code based on findings. Policy gates and SARIF belong to CF-05.

The repeated CLI path arguments remain explicit in CF-04; consolidating Clap argument structs was reviewed as optional cleanup and not adopted because it adds refactor churn without changing the public contract or correctness.

## Validation

Synthetic tests cover:

- empty report and repeated byte-identical classification;
- min/max/maxLength directionality;
- type-code narrowing/widening/incomparable replacement and qualifier-only RISKY behavior;
- fixed, pattern, and value-bound add/remove/change;
- binding-strength direction and ValueSet-change RISKY behavior;
- constraint additions/removals/severity/rewrite behavior;
- Must Support, modifier, and slicing behavior;
- resource/view/element/residual rules;
- unknown field/schema failure;
- duplicate constraint-key failure;
- unknown/malformed binding-strength and slicing-rule failure;
- snapshot/differential deduplication;
- residual-byte subsumption;
- corrupted-cache failure reason at the CLI boundary;
- offline classify success.

Real CI independently resolves and verifies published `hl7.fhir.r4.core@4.0.1` into explicit before/after states, runs CF-02 inspect, CF-03 self-diff, and CF-04 self-classify, and requires both changes and findings to be empty.

## Reviewer reconciliation

CodeRabbit returned two actionable implementation findings:

1. unknown binding/slicing code values could degrade to `RISKY` instead of failing closed — fixed with public pre-dispatch validation and regressions;
2. differential deduplication used a nested scan — fixed with a precomputed snapshot-identity index on the public path.

Reviewer nitpicks were also considered: corrupted-cache reason assertion, rule-family/determinism coverage, and removal of the temporary CF-04 push trigger were adopted; byte-change subsumption is documented here; sharing the repeated Clap args was intentionally not adopted as non-functional churn.

## Deferred semantics

Do not add terminology expansion, validator-oracle judgments, SARIF, source mapping, baselines, ecosystem graph impact, mapping execution, or AI authority in CF-04.
