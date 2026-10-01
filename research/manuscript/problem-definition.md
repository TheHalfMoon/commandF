# Problem Definition

Status: RESEARCH_PLANNING. Definitions only. Not a measurement. Not a protocol freeze. Not execution authority.

The V3.1 vocabulary cited here is the planning candidate in `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md`. That document does not supersede V2 and does not by itself implement a decision envelope. The shipped commands cited here are the identities in `research/BASELINE_IDENTITIES.md`.

## The decision the product is for

Given an old artifact state and a new artifact state, a consumer context when one is supplied, and the evidence that was actually supplied, the decision is whether a declared interoperability conclusion is warranted and what action follows. A structural difference, a parse success, and that conclusion are different questions.

```text
IMPLEMENTED = structural diff, structural classification, and a compatibility policy over Breaking, Risky, and Additive findings
PLANNED = the V3.1 truth class, recommended action, completeness, and sufficiency fields
NOT_IMPLEMENTED = a decision envelope that binds those planned fields
```

## Distinctions

A structural difference is a recorded change between the before bytes and the after bytes. `commandf diff` emits that report. It does not call `evaluate_compatibility_policy`.

Syntactic validity is whether the bytes parse as the declared artifact. It is not the same as a structural difference, and it is not a compatibility class. The shipped compatibility path does not define a validator pass as `PROVEN_COMPATIBLE`.

Semantic compatibility is whether the change preserves the behavior the governing rules care about. The shipped model records a finding severity of `Breaking`, `Risky`, or `Additive`, and a direction of `Producer` or `Consumer`. `commandf check` applies a policy to those findings. `CheckDecision.passed` is true when the blocking-finding count is zero. That boolean is a policy result. `research/BASELINE_IDENTITIES.md` says it is not a benchmark label, and it is not a V3.1 truth class.

Consumer-specific compatibility asks the question for a named consumer rather than for the artifact alone. The same change can be breaking for one consumer and unused by another. The shipped check can select `Both`, `Producer`, or `Consumer` as the direction of findings. That direction is not a consumer-contract document. `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` section 8 plans `commandf.consumer-contract/v1` under CF-19. That schema is not implemented. Popularity or registry presence is not a substitute for a declared contract. A missing contract stays missing.

Evidence sufficiency asks whether the evidence required for the requested conclusion is present. Evidence completeness asks whether the declared scope is covered. The planned envelope carries those as separate fields from truth class. Neither field exists on `CheckDecision`.

Uncertainty is not a probability standing in for a proof. The quantities stay separate:

```text
truth_class
recommended_action
statistical_confidence
evidence_sufficiency
evidence_completeness
```

`truth_class` and `recommended_action` are the V3.1 names. `evidence_sufficiency` is the planned `sufficiency` field. `evidence_completeness` is the planned `completeness` field. `statistical_confidence` is the interval `sap-1` would report after a measured run. None of the five is a field of the shipped `CheckDecision`. A model probability is advisory input in the V3.1 plan. It cannot create `PROVEN_COMPATIBLE` or `PROVEN_BREAKING`. Proof is not probability.

Unsupported evidence means the surface is outside the declared contract. The planned truth class is `UNSUPPORTED`. Unknown or unsupported is not a pass. The benchmark class of the same name is a corpus class, not this truth class.

Contradictory evidence means qualified sources disagree and the disagreement is retained. The planned truth class is `CONFLICTING_EVIDENCE`. Averaging the disagreement into a single pass is not the planned rule. `research/ADJUDICATION_PROTOCOL.md` stores that state as `DISAGREEMENT` and does not let commandF choose a surviving label.

A decision action is what policy says to do after the truth class and the evidence state are fixed. The planned actions are `ALLOW`, `WARN`, `BLOCK`, `REQUEST_ORACLE`, `REQUEST_RUNTIME_PROBE`, `REQUEST_HUMAN`, and `ABSTAIN`. The shipped check does not emit those names. It returns process status 0 or 2 from `decision.passed`.

## Inputs and outputs

The inputs the problem requires are:

- the old artifact state and the new artifact state, as locked package bytes;
- a consumer contract or context, when the conclusion is consumer-specific;
- the evidence that was supplied, bound to those bytes;
- oracle or runtime evidence only when a later record actually ran it.

`commandf diff`, `commandf classify`, and `commandf check` take the package name and the before and after lock and cache. `commandf context` takes one lock and one cache and is not fed into `check`. `commandf oracle` requires a caller-supplied adapter. Its jar digest and Java runtime are not pinned, so that input is not an identity yet.

The planned output space is one truth class from this list: `PROVEN_COMPATIBLE`, `PROVEN_BREAKING`, `CONDITIONALLY_COMPATIBLE`, `RISK_DETECTED`, `CONFLICTING_EVIDENCE`, `INSUFFICIENT_EVIDENCE`, `UNSUPPORTED`, `INDETERMINATE`. It also carries one recommended action, one completeness state among `COMPLETE_FOR_DECLARED_SCOPE`, `MATERIALLY_INCOMPLETE`, and `UNKNOWN_DUE_TO_ERROR`, and a sufficiency state. The shipped output is a compatibility report and a `CheckDecision`. Those are not that envelope.

## Failure modes the definition keeps visible

Missing evidence is not compatibility. `PROVEN_COMPATIBLE` requires the declared protected scope to be covered. It is not inferred from an empty finding list. `ABSTAIN` is a planned action when the evidence is insufficient. A missing run in `research/STATISTICAL_ANALYSIS_PLAN.md` is also not an abstention.

Unsupported evidence stays visible. The planned invalid states reject `PROVEN_COMPATIBLE` when material unsupported witnesses sit inside the protected scope. They also reject `PROVEN_COMPATIBLE` when a required oracle was not run, failed, or was unavailable.

Conflicting evidence stays visible. The planned class `CONFLICTING_EVIDENCE` exists so a disagreement is not rewritten as a clean result.

## What this section does not claim

It does not report a rate, a coverage, or a comparison. It does not assign a split. It does not freeze the protocol. It does not authorize CF-17, CF-18, CF-19, AF-02, AF-03, or AF-04.
