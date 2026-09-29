# commandF V3.1 — Decision Assurance and Product Hardening Plan

Status: **PLANNING_CANDIDATE / V3.1 OVERLAY**

This document strengthens the existing V3 candidate. It does **not** supersede `COMMAND_F_MASTER_ARCHITECTURE_V2.md`, does not authorize production implementation by itself, and does not change any active CF/AF Spec Kit. V2 remains execution authority until a separately qualified migration/reconciliation gate says otherwise.

The purpose of V3.1 is to remove the remaining planning ambiguity around decision accuracy, evidence sufficiency, abstention, consumer-specific impact, machine-verifiable receipts, and model-assisted reasoning without weakening commandF's deterministic trust boundary.

## 1. Product thesis

The target product sentence is:

> **commandF tells you what an interoperability change will break, which protected consumers it can break, why, how complete the evidence is, what remains unknown, and what reproducible evidence supports the decision — before the change ships.**

The critical distinction is:

- **proof is not probability**;
- **confidence is not semantic authority**;
- **absence of evidence is not compatibility**;
- **unknown/unsupported is not PASS**;
- **abstention is a valid high-quality result when evidence is insufficient**.

commandF should become the evidence-bound decision layer for healthcare interoperability engineering, not another general FHIR validator, agent runtime, terminology server, integration engine, or LLM wrapper.

## 2. Authority hierarchy

V3.1 preserves this order:

1. exact immutable input bytes and identities;
2. commandF deterministic normalization/diff/graph/rule semantics;
3. explicit policy and protected-consumer contracts;
4. pinned authoritative/independent oracle evidence where required;
5. measured runtime evidence where static evidence is insufficient;
6. human authority where policy requires it;
7. optional model assistance only as advisory ranking/proposal/evidence-sufficiency input.

A model may never upgrade an unsupported or incomplete deterministic state into `PROVEN_COMPATIBLE` or `PROVEN_BREAKING`.

## 3. Decision Assurance Layer

V3.1 adds an explicit logical **Decision Assurance Layer** between evidence assembly and user-facing gating.

```text
Immutable Inputs
      |
      v
Artifact Truth
      |
      v
Context / Dependency Graph
      |
      v
Compatibility Semantics
      |
      v
Consumer Contracts
      |
      v
Evidence Assembly
      |
      v
Decision Assurance
  | deterministic rules
  | protected-consumer witnesses
  | authoritative/differential oracles
  | runtime probes when authorized
  | evidence completeness/sufficiency
  | optional calibrated advisory scorer
      |
      v
Policy Gate
      |
      v
Decision Receipt
      |
      +--> CLI / JSON / SARIF / PR / LSP / API / Studio
```

This is a logical architecture. It does not require a new service, database, model runtime, or crate before a slice proves that a new boundary is justified.

## 4. Decision vocabulary

### 4.1 Truth class

Every final decision carries exactly one truth class:

- `PROVEN_COMPATIBLE`
- `PROVEN_BREAKING`
- `CONDITIONALLY_COMPATIBLE`
- `RISK_DETECTED`
- `CONFLICTING_EVIDENCE`
- `INSUFFICIENT_EVIDENCE`
- `UNSUPPORTED`
- `INDETERMINATE`

Meanings:

- `PROVEN_COMPATIBLE` requires the declared protected scope to be fully covered by the applicable compatibility model and required evidence. It is never inferred from "no finding" alone.
- `PROVEN_BREAKING` requires a deterministic witness, protected-consumer witness, or qualified empirical/oracle witness that satisfies the governing rule/policy.
- `CONDITIONALLY_COMPATIBLE` means compatibility holds only under explicit version/consumer/configuration/terminology/runtime conditions retained in evidence.
- `RISK_DETECTED` means a material risk is identified but evidence does not justify a stronger proof class.
- `CONFLICTING_EVIDENCE` means qualified sources disagree and the disagreement cannot be resolved by the current authority policy.
- `INSUFFICIENT_EVIDENCE` means the requested conclusion is in-scope but required evidence is absent/incomplete.
- `UNSUPPORTED` means the artifact/version/change/protocol surface is outside declared commandF semantic coverage.
- `INDETERMINATE` means execution/control failure prevents a trustworthy conclusion.

### 4.2 Recommended action

Truth class is separate from the requested next action. Actions are:

- `ALLOW`
- `WARN`
- `BLOCK`
- `REQUEST_ORACLE`
- `REQUEST_RUNTIME_PROBE`
- `REQUEST_HUMAN`
- `ABSTAIN`

Policy maps truth/evidence state to an action. A model does not choose a privileged action directly.

### 4.3 Completeness

Each decision carries one evidence-completeness state:

- `COMPLETE_FOR_DECLARED_SCOPE`
- `MATERIALLY_INCOMPLETE`
- `UNKNOWN_DUE_TO_ERROR`

This is intentionally separate from both truth class and action.

## 5. Decision Envelope v1

CF-18 must freeze a versioned machine contract named conceptually `commandf.decision-envelope/v1` before implementation.

Minimum fields:

```text
schema_version
engine_identity
comparison_identity
protected_scope
truth_class
recommended_action
compatibility_dimensions[]
classification_state
completeness
sufficiency
unsupported_witnesses[]
conflicting_evidence[]
rule_refs[]
consumer_refs[]
evidence_refs[]
oracle_refs[]
runtime_probe_refs[]
policy_identity
configuration_identity
model_advisory (optional)
reason_codes[]
```

`model_advisory`, when present, is subordinate metadata and must include exact model/tokenizer/calibration/policy identity. It cannot alter the deterministic truth class without a deterministic/oracle/policy witness that independently justifies the transition.

## 6. Evidence sufficiency

V3.1 makes sufficiency explicit rather than treating it as narrative explanation.

For every requested decision commandF must determine:

1. what evidence classes are required;
2. which required evidence is present;
3. which required evidence is unavailable;
4. whether the missing evidence can change the requested verdict;
5. whether an oracle/runtime probe can resolve the gap;
6. whether policy requires human review;
7. whether the correct action is abstention.

Sufficiency must be deterministic for the declared policy whenever the required inputs are deterministic.

An optional model may estimate/rank likely useful next evidence, but the acceptance of evidence and final sufficiency state are deterministic/policy controlled.

## 7. Selective oracle escalation

Running every expensive oracle for every change is unnecessary; skipping a required oracle is unsafe. V3.1 therefore requires explicit escalation semantics.

Candidate deterministic triggers include:

- rule class requires authoritative validator evidence;
- snapshot/differential ambiguity affects classification;
- consumer contract depends on runtime behavior not represented in static metadata;
- qualified independent oracles disagree;
- protocol behavior requires executable conformance evidence;
- protected policy marks a decision class as oracle-required;
- evidence completeness is otherwise insufficient for `PROVEN_*`.

Oracle outcomes must include at least:

- `NOT_REQUIRED`
- `REQUIRED_NOT_RUN`
- `RUN_AGREEMENT`
- `RUN_DIVERGENCE`
- `UNAVAILABLE`
- `FAILED`
- `TIMED_OUT`
- `UNCOMPARABLE`

`REQUIRED_NOT_RUN`, `UNAVAILABLE`, `FAILED`, or `TIMED_OUT` may not silently become compatibility.

## 8. Consumer Contract v1

CF-19 must make consumer evidence a versioned artifact, not only an internal scanner result.

Conceptual schema: `commandf.consumer-contract/v1`.

Minimum identity fields:

- contract id/version;
- producer package/version scope;
- consumer application/service identity;
- deployment/version/environment label;
- source evidence identity;
- extraction method/version;
- explicit protected/unprotected status;
- capture time only where time is semantically relevant;
- provenance and privacy classification.

Minimum dependency families:

- FHIR packages/profiles/extensions;
- canonical resources;
- element/path dependencies;
- terminology systems/versions/bindings;
- SearchParameters and search interactions;
- FHIRPath expressions;
- CQL/ELM/Library dependencies;
- SQL-on-FHIR `ViewDefinition` dependencies;
- CapabilityStatement expectations;
- REST interactions and operations;
- SMART scopes/backend-service expectations;
- Bulk Data expectations;
- subscriptions/eventing contracts;
- selected TestScript/Inferno or other executable contract references;
- declared migration constraints.

Rules:

- popularity, registry presence, or graph centrality may never stand in for an observed/declared consumer contract;
- missing contract information must remain missing, not guessed;
- contracts that touch instance data require the separately approved privacy boundary;
- protected consumer/version sets are exact inputs to a certification decision.

## 9. Decision Receipt v1

CF-21/23 must expose a source/evidence-bound receipt patterned after commandF's own proof discipline and the useful receipt separation studied in Ascout.

Conceptual schema: `commandf.decision-receipt/v1`.

Required sections:

### Source/comparison

- exact old/new package/artifact/source identities;
- exact lock/cache/input digests;
- workspace/comparison semantics;
- source stability/drift state where applicable.

### Execution

Each required stage records:

- `PASS`
- `FAIL`
- `BLOCKED`
- `ERROR`
- `NOT_APPLICABLE`
- `NOT_RUN`

Non-success/non-execution requires stable reason codes. A missing stage cannot be omitted to make the receipt look green.

### Evidence

- evidence truth class;
- producer identity;
- exact digest/artifact reference;
- redaction/truncation facts;
- offline/replay availability;
- tool/model/oracle/config identity where applicable.

### Decision

- Decision Envelope v1;
- protected consumer set;
- completeness;
- policy gate result;
- migration/remediation references;
- reproducibility/bundle identity.

Receipt identity must change when any semantic input, rule, protected-consumer set, required oracle identity, configuration, or policy identity changes.

## 10. Optional GAX-derived advisory scorer

The founder-controlled GAX project is retained as a pinned pattern source, not semantic authority.

V3.1 adopts these concepts only:

- explicit finite action set;
- probability over advisory actions;
- evidence-support scoring;
- sufficiency separate from top-1 confidence;
- native abstention;
- calibration separate from threshold/policy;
- counterfactual and irrelevant-edit testing;
- risk-coverage evaluation.

The optional adapter may propose among advisory actions such as:

- `LIKELY_ALLOW`
- `LIKELY_WARN`
- `LIKELY_BLOCK`
- `NEED_ORACLE`
- `NEED_PROBE`
- `NEED_HUMAN`
- `ABSTAIN`

These values are never persisted as proof classes. They are model observations subject to policy.

### Mandatory adapter boundary

- default deterministic core has no model dependency;
- model adapter is optional and replaceable;
- no hidden network fallback;
- exact model/tokenizer/calibration/policy identity retained;
- bounded input/output;
- no PHI required for product qualification;
- model failure or absence leaves deterministic semantics available;
- model cannot activate rules, suppress findings, approve migrations, or grant protected actions;
- no model claim is promoted without benchmark evidence.

## 11. CommandFBench-Decision

CF-28 must add an evidence-bound decision benchmark. It should not optimize for a single aggregate score.

Required metrics:

- missed-breaking-change rate;
- unsafe `ALLOW` rate;
- false `BLOCK` rate;
- precision/recall for protected-consumer impact;
- risk-coverage curve;
- AURC or an equivalent preregistered selective-risk metric;
- abstention precision/recall;
- evidence-sufficiency calibration;
- Brier/NLL/ECE where probabilistic advisory outputs exist;
- oracle-escalation precision/recall and avoided unnecessary oracle cost;
- contradictory-evidence handling;
- counterfactual sensitivity for material changes;
- irrelevant-edit decision stability;
- unsupported-surface detection rate;
- decision-receipt reproducibility;
- p50/p95 latency and peak memory by path;
- oracle/runtime-probe cost separately from deterministic core cost.

### Primary safety metric

A candidate deployment profile must report:

> **unsafe auto-allow rate at declared coverage**

Coverage without risk is misleading; risk without coverage rewards trivial abstention. Both must be reported together.

### Dataset/benchmark discipline

- frozen train/calibration/test or discovery/validation/final partitions as applicable;
- no tuning on final-test labels;
- exact corpus/version/provenance;
- leakage/contamination analysis for any model-assisted evaluation;
- confidence intervals and failed-inference counts;
- per-artifact/per-protocol/per-version/per-consumer slices;
- catastrophic slices cannot be hidden by aggregate averages.

## 12. Counterfactual decision testing

V3.1 adds two mandatory test families for decision behavior:

### Material-edit sensitivity

A controlled compatibility-relevant edit expected to change the decision must change the applicable evidence/classification/action in the expected direction.

Examples:

- cardinality restriction;
- required binding change;
- search parameter removal;
- SMART scope requirement change;
- operation parameter change;
- subscription/event contract change;
- protected consumer adds/removes dependency.

### Irrelevant-edit stability

Formatting/order/non-semantic metadata edits declared irrelevant by the change-space model must not change semantic decision identity.

Both families require exact fixtures, deterministic expected behavior, and minimized regressions when failures are found.

## 13. Known integrity blockers before V3 migration

At the time of this planning pass, live repository issues #35, #36, #37, #38, and #40 are open and describe evidence/input-boundary defects around verified-cache byte consumption and bounded local input allocation.

V3.1 classifies them as **migration blockers for any claim that the post-V3 decision layer is production-trust-ready**.

This does not silently change their existing issue scope or authorize implementation in this planning PR.

Required migration-gate rule:

- all applicable verified-byte consumers must consume the exact bytes whose digest was verified;
- external oracle staging must bind the oracle to the verified generation;
- persisted untrusted inputs must be bounded before full allocation/parsing;
- local/offline acquisition must not have weaker undocumented resource bounds than network acquisition;
- the migration gate must re-audit for equivalent patterns rather than merely close issue numbers.

## 14. Assurance reuse rule

commandF's assurance program is a strength, but reusable trust invariants should not be reimplemented independently in every future slice.

V3.1 requires reusable commandF-owned helpers/contracts for at least:

- bounded file reads;
- verified-byte consumption;
- canonical path handling;
- deterministic canonical JSON serialization;
- evidence ID/digest formation;
- stage status/reason semantics;
- versioned schema validation;
- offline bundle identity;
- external process timeout/output/resource handling;
- redaction/truncation facts.

A new slice may strengthen these primitives, but should not fork slightly different semantics without an explicit rationale and compatibility test.

## 15. New planning gaps G41-G50

These extend, not replace, G01-G40.

| ID | Sev | Gap | Owning slice(s) | Closure contract |
| --- | --- | --- | --- | --- |
| G41 | P0 | Proof, confidence, and policy action are not formally separate machine concepts | CF-18/23 | Decision Envelope v1 encodes truth class, action, completeness, policy/config identity, and optional advisory confidence separately; invalid cross-field states fail validation. |
| G42 | P0 | No explicit evidence-sufficiency/abstention contract | CF-18/20 | Required evidence is machine-declared per decision class; insufficient evidence produces a non-PASS truth/action and can trigger oracle/probe/human/abstain. |
| G43 | P0 | No source/evidence-bound Decision Receipt with stage completeness | CF-21/23 | Decision Receipt v1 records exact source/comparison identity, required-stage status, evidence refs, Decision Envelope, and reproducibility identity; omissions cannot serialize as complete. |
| G44 | P0 | Consumer evidence lacks one stable portable contract schema | CF-19/23 | `commandf.consumer-contract/v1` is versioned, validated, provenance-bound, and round-trip tested across supported dependency families. |
| G45 | P0 | No decision-quality benchmark focused on unsafe allow at useful coverage | CF-28 | CommandFBench-Decision reports unsafe-allow and false-block rates together with risk-coverage/selective-risk metrics and fixed evaluation policy. |
| G46 | P1 | Decision behavior lacks material-counterfactual sensitivity and irrelevant-edit stability proof | CF-18/22/28 | Frozen counterfactual groups demonstrate expected decision movement for material edits and stable identity for irrelevant edits. |
| G47 | P1 | Oracle use is not governed by explicit evidence-sufficiency escalation | CF-20 | Deterministic escalation policy records why an oracle/probe was required or skipped and preserves required-not-run/unavailable/failed states. |
| G48 | P1 | Optional model reasoning has no product-specific authority/calibration boundary | CF-18/23/28 | Provider-neutral advisory adapter contract is optional, exact-identity-bound, calibrated/evaluated, and technically unable to self-promote into proof or privileged action. |
| G49 | P0 | Known verified-byte and unbounded-input defects can undermine evidence claims if V3 expands first | V3 migration gate + owning repair issues | Migration cannot claim production trust until issues #35/#36/#37/#38/#40 invariants are closed and an equivalent-pattern audit is green. |
| G50 | P1 | Assurance patterns risk being duplicated slice-by-slice, slowing product execution and creating semantic drift | AF program + all future CF slices | Shared assurance primitives are reused or explicitly strengthened; duplicate semantics require compatibility rationale/tests. |

No G41-G50 item is considered closed because this document exists.

## 16. Slice ownership amendments

V3.1 does not create a new product-numbering family.

- **CF-17** keeps ecosystem observatory ownership and must expose immutable evidence useful for decision evaluation.
- **CF-18** owns Decision Envelope v1, truth/action/completeness algebra, sufficiency policy primitives, compatibility coverage, and counterfactual semantic fixtures.
- **CF-19** owns Consumer Contract v1, protected consumer/version sets, dependency extraction, and consumer-specific impact witnesses.
- **CF-20** owns selective oracle/runtime escalation and explicit disagreement/unavailability semantics.
- **CF-21** owns Decision Receipt v1 integration with Interoperability BOM/reproducible bundles and offline replay.
- **CF-22** owns deterministic generation of tests/counterexamples from exact evidence; generated artifacts cannot self-authorize.
- **CF-23** owns stable external exposure of Decision Envelope, Consumer Contract, and Decision Receipt through CLI/API/LSP/daemon surfaces without duplicate semantic logic.
- **CF-24** owns capability-scoped optional adapters/plugins and ensures no model/plugin can grant itself semantic or execution authority.
- **CF-25** supplies terminology evidence to the same decision/evidence contracts.
- **CF-26** supplies transformation/loss evidence without redefining Decision Envelope authority.
- **CF-27** publishes immutable evidence/receipt views with correction/supersession semantics.
- **CF-28** owns CommandFBench-Decision and any empirical claim about model-assisted decision quality, calibration, efficiency, or superiority.

## 17. Proposed implementation seam

Current commandF uses a deliberately small Rust workspace. V3.1 therefore does **not** require new crates by default.

First implementation should prefer commandF-native modules inside the existing `commandf-pkg` semantic library and expose them through `commandf-cli`, unless the relevant Spec Kit demonstrates that a new crate provides a measurable ownership/compatibility benefit.

Candidate module seams to refine during the owning Spec Kits:

```text
crates/commandf-pkg/src/decision_*.rs
crates/commandf-pkg/src/consumer_contract_*.rs
crates/commandf-pkg/src/evidence_*.rs
crates/commandf-pkg/src/decision_receipt_*.rs
crates/commandf-cli/src/... decision/check/certify surfaces
schemas/commandf-decision-envelope-v1.schema.json
schemas/commandf-consumer-contract-v1.schema.json
schemas/commandf-decision-receipt-v1.schema.json
```

Exact filenames are not implementation authority until the owning Spec Kit audits current source layout and freezes the change surface.

## 18. No-go rules

The following are rejected by V3.1 unless a future evidence-backed architecture supersedes this plan:

- an LLM/model as semantic authority;
- a model confidence threshold that directly means `SAFE`;
- auto-allow from absence of findings;
- silent oracle failure fallback;
- consumer dependencies inferred from popularity alone;
- vector/RAG retrieval as evidence authority;
- hidden network fallback for decision inference;
- arbitrary Python/JavaScript in the trusted rule core;
- one universal clinical/interoperability IR as a prerequisite;
- a new graph database merely for architectural fashion;
- a new commandF cloud runtime required for core correctness;
- an aggregate benchmark score that hides catastrophic safety-relevant slices;
- a receipt that omits required-but-unrun stages;
- reusing evidence across changed source/rule/policy/consumer/oracle identities.

## 19. V3.1 planning definition of done

The planning overlay is implementation-ready only when:

1. G01-G40 remain fully preserved by the existing V3 playbook.
2. G41-G50 are mapped to exact owning slices and executable closure contracts.
3. decision truth/action/completeness/sufficiency semantics are explicit.
4. consumer contract scope is explicit and versionable.
5. receipt semantics distinguish execution, evidence, and decision completeness.
6. GAX/Ascout/Kernux/Cotra are pinned as pattern sources with non-authority boundaries.
7. model-assisted reasoning is optional and replaceable.
8. benchmark metrics prevent trivial always-act and always-abstain strategies from looking good.
9. known evidence/input-boundary defects are migration blockers, not silently ignored debt.
10. implementation sequencing and task-level handoff are defined in the companion V3.1 runbook.
11. no planning artifact claims any unexecuted product behavior is already proven.

The companion implementation handoff is `docs/COMMAND_F_V3_1_IMPLEMENTATION_RUNBOOK.md`.
