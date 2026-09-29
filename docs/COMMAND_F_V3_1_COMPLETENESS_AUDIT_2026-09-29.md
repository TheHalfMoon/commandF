# commandF V3.1 Planning Completeness Audit — 2026-09-29

Status: **READY_FOR_SPEC_KIT_SHAPING / NOT EXECUTION AUTHORITY**

This audit challenges the combined V3 + V3.1 planning set for silent gaps before future implementation. It does not prove product behavior, close any CF/AF unit, supersede V2, or authorize code changes outside the live canonical frontier.

Reviewed planning set:

- `COMMAND_F_MASTER_ARCHITECTURE_V3_CANDIDATE.md`
- `COMMAND_F_PLAN_GAP_REVIEW_2026-09-12.md`
- `COMMAND_F_V3_EXECUTION_PLAYBOOK.md`
- `COMMAND_F_OPEN_SOURCE_QUALIFICATION_2026-09-12.md`
- `COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md`
- `COMMAND_F_V3_1_IMPLEMENTATION_RUNBOOK.md`
- `COMMAND_F_PLAN_INDEX.md`
- `docs/commandf-v3_1-internal-pattern-sources-2026-09-29.yaml`

Historical execution authority remains `COMMAND_F_MASTER_ARCHITECTURE_V2.md` plus live canonical Spec Kits.

## 1. Audit method

The plan was challenged across these dimensions:

1. authority and build-order ambiguity;
2. semantic decision ambiguity;
3. unsupported/unknown/error behavior;
4. evidence sufficiency and oracle failure;
5. consumer/version identity;
6. schema and upgrade compatibility;
7. provenance and source rights;
8. privacy/PHI boundary;
9. resource and integrity boundaries;
10. offline/replay behavior;
11. optional model/plugin authority;
12. benchmark gaming and calibration;
13. counterfactual robustness;
14. protocol coverage beyond resource schemas;
15. cross-standard scope creep;
16. public evidence correction/freshness;
17. assurance duplication and product velocity;
18. implementation handoff clarity.

The audit rule is fail-closed: an applicable unanswered question blocks `READY_FOR_IMPLEMENTATION` for the owning slice and must become an explicit gap or Spec Kit task.

## 2. Result

The combined planning set is **ready for Spec Kit shaping** because every material planning concern found in this review has one of:

- an existing G01-G40 owner;
- a new G41-G50 owner;
- an explicit boundary/non-goal;
- a migration prerequisite;
- a research-only disposition.

No claim is made that no future gap can ever be discovered. Instead, the planning system now forbids silent gaps: any newly discovered material concern with no existing owner must receive a new gap identity before implementation proceeds.

## 3. Gap accounting

### Existing V3 gaps

- G01-G40 remain retained exactly.
- No V3.1 document renumbers or deletes them.
- Existing closure contracts remain in `COMMAND_F_V3_EXECUTION_PLAYBOOK.md`.

### V3.1 gaps

- G41 — proof/confidence/action separation
- G42 — evidence sufficiency and abstention
- G43 — Decision Receipt completeness
- G44 — portable Consumer Contract schema
- G45 — unsafe-allow-at-coverage benchmark
- G46 — counterfactual sensitivity / irrelevant-edit stability
- G47 — explicit oracle escalation policy
- G48 — optional model authority/calibration boundary
- G49 — known evidence/input integrity blockers before trust migration
- G50 — shared assurance primitives / anti-duplication

Total explicit V3 planning gaps after this overlay: **50**.

A gap is not closed by being listed.

## 4. Authority and dependency audit

### Finding: PASS at planning level

The plan preserves:

```text
V2 execution authority
  -> live CF/AF Spec Kits
  -> future migration/reconciliation gate
  -> V3/V3.1 candidate work
```

V3.1 does not introduce a competing slice numbering family. It extends existing owners:

- CF-18 decision semantics;
- CF-19 consumer contracts;
- CF-20 oracle/probe escalation;
- CF-21 receipts/replay;
- CF-22 evidence-derived tests;
- CF-23 external stable surfaces;
- CF-24 optional capability/model boundary;
- CF-25/26 evidence providers;
- CF-27 public evidence;
- CF-28 decision benchmark/research.

No future slice may use evidence from a downstream unmet dependency to close itself.

## 5. Decision-semantics audit

### Prior weakness

The V3 finding contract retained uncertainty/unsupported state but did not fully freeze the distinction among:

- semantic proof;
- model/statistical confidence;
- evidence completeness;
- policy action;
- abstention/escalation.

### V3.1 correction

The plan now requires separate machine concepts:

```text
truth_class
recommended_action
completeness
sufficiency
model_advisory (optional)
```

Invalid combinations must fail validation.

Planning result: **covered by G41/G42/CF-18**.

## 6. Consumer-contract audit

### Prior weakness

Consumer scanning could remain an implementation detail without a portable exact consumer/version artifact.

### V3.1 correction

`commandf.consumer-contract/v1` is required with protected-set semantics, provenance, source/extractor identity, version context, dependency families, and privacy classification.

Popularity/graph centrality is explicitly forbidden as a replacement for consumer evidence.

Planning result: **covered by G44/CF-19**.

## 7. Oracle and partial-result audit

### Risks challenged

- required oracle not run;
- oracle unavailable;
- timeout;
- disagreement;
- runtime probe completion confused with verification;
- expensive oracle overuse.

### V3.1 correction

Explicit states and deterministic escalation policy are required. Missing required oracle evidence cannot become `PROVEN_COMPATIBLE`.

Planning result: **covered by G42/G47/CF-20 plus existing G33**.

## 8. Receipt/reproducibility audit

### Risk challenged

A final JSON finding could look complete even if a required stage was blocked, omitted, or failed.

### V3.1 correction

Decision Receipt v1 requires an explicit stage manifest with:

```text
PASS / FAIL / BLOCKED / ERROR / NOT_APPLICABLE / NOT_RUN
```

plus exact source/comparison, policy/config, protected scope, evidence, Decision Envelope, Interoperability BOM/replay references, and redaction/truncation facts.

Planning result: **covered by G43/CF-21/23**.

## 9. Integrity/resource-boundary audit

Live issue review identified unresolved issue families describing:

- verify-then-reopen cache generation gaps;
- external oracle consumption not necessarily bound to verified bytes;
- unbounded lockfile allocation before parse;
- unbounded local-mirror archive allocation;
- repeated root archive consumer verified-byte gaps.

V3.1 does not pretend planning fixes these defects. It makes their invariants migration blockers for a production-trust V3 claim and requires an equivalent-pattern audit rather than mechanical issue closure.

Planning result: **covered by G49 + Wave -1**.

## 10. Model/Jev-alternative audit

The selected internal pattern source is GAX because it explicitly separates finite action choice, sufficiency, abstention, calibration, evidence support, and counterfactual behavior.

The selected adoption is **PATTERN_ONLY**.

Rejected architecture:

```text
model confidence -> commandF semantic truth
```

Required architecture:

```text
deterministic/oracle evidence -> truth class
optional model observation -> advisory metadata / evidence-priority proposal
policy -> allowed next action
```

A deterministic-only deployment remains a first-class supported mode. A negative benchmark result may keep it as the default.

Planning result: **covered by G48/CF-24/28**.

## 11. Benchmark-gaming audit

### Risks challenged

- always-act looks good on accuracy;
- always-abstain looks good on error rate;
- aggregate score hides catastrophic slice;
- model tuned against final labels;
- expensive oracle use hides operational cost.

### V3.1 correction

Required joint reporting includes:

- unsafe allow;
- missed breaking;
- false block;
- coverage/selective risk;
- abstention quality;
- consumer impact precision/recall;
- unsupported detection;
- oracle escalation precision/recall;
- calibration if probabilistic;
- per-slice failures and ops cost.

Primary deployment framing: **unsafe auto-allow risk at declared coverage**.

Planning result: **covered by G45/CF-28**.

## 12. Counterfactual audit

Both of these are mandatory:

- material edit should move the decision/evidence in the expected direction;
- irrelevant edit should preserve semantic decision identity.

This prevents superficial serialization sensitivity from being mistaken for interoperability intelligence.

Planning result: **covered by G46/CF-18/22/28**.

## 13. Privacy and rights audit

The plan preserves:

- no PHI required for core qualification;
- instance-touching contract capture requires a separate approved/on-prem boundary;
- software license and terminology/content/data/model rights are separate;
- donor pinning alone does not authorize copying/redistribution;
- public observatory excludes private/sensitive consumer evidence by default.

Planning result: **covered by existing G29/G34 plus donor policy and V3.1 runbook**.

## 14. Assurance-velocity audit

### Risk challenged

A project with strong proof culture can still slow product delivery if every feature reinvents bounded reads, evidence IDs, status vocabularies, replay manifests, external process handling, and canonical serialization.

### V3.1 correction

G50 requires shared assurance primitives or explicit justified strengthening with compatibility tests.

Planning result: **covered by G50**.

## 15. Implementation-surface audit

Current workspace has a deliberately small crate boundary. V3.1 therefore does not force premature micro-crate expansion.

Implementation guidance:

- prefer existing semantic library boundaries first;
- add new modules/contracts under the owning Spec Kit;
- create a new crate only when ownership/API/performance evidence justifies it;
- no privileged semantic logic may live only in CLI/UI/MCP/model adapter surfaces.

Planning result: **implementation path explicit without premature source-layout lock-in**.

## 16. Residual risks intentionally not claimed solved

The plan explicitly retains these as boundaries/research rather than pretending closure:

- universal semantic equivalence across FHIR/openEHR/OMOP;
- universal safe profile harmonization;
- universal patient matching;
- universal authorization/consent policy semantics;
- universally correct terminology mapping;
- transaction/concurrency safety inferred from static artifacts;
- model superiority before executed benchmark evidence;
- semantic-loss metric universality before research validation.

These are not planning gaps because the product claim boundary is explicit.

## 17. Ready-to-implement handoff rule

A future owning slice may be marked `READY_FOR_IMPLEMENTATION` only when its Spec Kit has answered the runbook checklist for:

- exact inputs/outputs;
- authority;
- schema/version;
- invalid/unsupported/error/partial states;
- bounds/capabilities;
- privacy/rights;
- determinism/cache identity;
- upgrade behavior;
- oracle disagreement;
- consumer scope;
- offline/replay;
- tests/counterexamples;
- performance claim boundary;
- evidence retention;
- review/merge proof.

An unanswered applicable item blocks readiness.

## 18. Audit conclusion

**Planning conclusion: READY_FOR_SPEC_KIT_SHAPING.**

The V3 + V3.1 plan now has explicit ownership for all 50 identified planning gaps, preserves prior 35 legacy interoperability hypotheses, retains execution authority boundaries, pins internal pattern sources, formalizes decision sufficiency/abstention/receipt semantics, and defines migration blockers plus executable handoff rules.

This conclusion means the plan is sufficiently specified to create the next dependency-eligible Spec Kits without inventing missing architecture. It does **not** mean the planned features are implemented, proven, merged, or production-ready.
