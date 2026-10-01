# Compatibility Model

Status: RESEARCH_PLANNING. Model description only. Not an accuracy claim. Not a protocol freeze. Not execution authority.

The shipped classifier is `classify_structural_diff` in `crates/commandf-pkg/src/compatibility.rs`. It accepts a structural diff whose schema is `StructuralDiffReport::SCHEMA_V1`. Any other schema returns `CompatibilityError::UnsupportedDiffSchema`. It emits `CompatibilityReport` schema 1 with ruleset `cf04-rules-v1`. The shipped policy is `evaluate_compatibility_policy` in `crates/commandf-pkg/src/check.rs`. The planned truth classes are section 4 of `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md`. They are not values of `CompatibilitySeverity`.

## What a structural difference is

An artifact difference is one `StructuralChange` on a locked package pair. The kinds are `ResourceAdded`, `ResourceRemoved`, `ResourceFilenameChanged`, `ResourceVersionChanged`, `ResourceTypeChanged`, `ResourceIdChanged`, `ResourceBytesChanged`, `StructureFieldChanged`, `ViewAdded`, `ViewRemoved`, `ElementAdded`, `ElementRemoved`, and `ElementFieldChanged`.

The classifier turns those changes into findings. Each finding has a rule id, a severity, and a direction. A byte change that already has a more precise change on the same resource is not classified again. A field with no CF-04 rule returns `UnsupportedStructuralField`. An invalid field value returns `InvalidChangeValue`. Neither error becomes an empty finding list.

## What the shipped severity is

`CompatibilitySeverity` has three values: `Breaking`, `Risky`, and `Additive`. `CompatibilityDirection` has two values: `Producer` and `Consumer`. Those names are not `PROVEN_BREAKING` and `PROVEN_COMPATIBLE`.

`CheckPolicy` selects a direction of `Both`, `Producer`, or `Consumer`, and a fail threshold of `Breaking`, `Risky`, or `None`. The default is both directions and fail on `Breaking`. `CheckDecision.passed` is true when the selected blocking count is zero. A zero blocking count can still sit beside risky or additive findings. An empty selected set is not a planned `PROVEN_COMPATIBLE` result.

`Risky` is not the planned class `RISK_DETECTED`. `Breaking` is not the planned class `PROVEN_BREAKING`. `Additive` is not compatibility proof.

## What the model does not collapse

A behavioral difference is a change in what a consumer or a runtime does with the artifacts. The structural classifier does not observe that behavior. A finding is a rule applied to a recorded structural change.

Consumer impact is whether a protected consumer uses the changed surface. The shipped direction axis records producer-side and consumer-side rules. It does not name a consumer, and it does not read a consumer contract. `commandf context` builds a graph from one lock and one cache. That graph is not an input to `evaluate_compatibility_policy`.

The planned classes that are not in the shipped severity enum are `PROVEN_COMPATIBLE`, `PROVEN_BREAKING`, `CONDITIONALLY_COMPATIBLE`, `RISK_DETECTED`, `CONFLICTING_EVIDENCE`, `INSUFFICIENT_EVIDENCE`, `UNSUPPORTED`, and `INDETERMINATE`. Their meanings are the planning text in `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` section 4.1, as already distinguished in `research/manuscript/problem-definition.md`. This section does not assign any current package pair to those classes.

Conditional compatibility, conflicting evidence, insufficient evidence, unsupported coverage, and an indeterminate execution failure are planned states. The shipped classifier expresses an out-of-coverage structural field as an error. It does not emit those planned names.

## What this section does not claim

It does not report precision, recall, or a comparison with an oracle. It does not say every removal is breaking for every consumer. It does not authorize CF-18.
