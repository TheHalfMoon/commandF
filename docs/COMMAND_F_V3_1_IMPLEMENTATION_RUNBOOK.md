# commandF V3.1 — Implementation Runbook

Status: **PLANNING_CANDIDATE / IMPLEMENTATION HANDOFF**

This runbook is an additive implementation handoff for `COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` and the existing `COMMAND_F_V3_EXECUTION_PLAYBOOK.md`.

It does not supersede `COMMAND_F_MASTER_ARCHITECTURE_V2.md`, does not activate any CF/AF implementation by itself, and does not mutate an active Spec Kit. V2 remains execution authority until a separately qualified migration/reconciliation gate is canonical.

The purpose of this file is operational: a future implementation agent should be able to determine the exact next planning/implementation work without inventing product semantics, authority, schema fields, evaluation claims, or donor adoption.

## 1. Authority order

When sources disagree, use this order:

1. live canonical `main`, live rulesets, active canonical Spec Kit, and exact current task authority;
2. `COMMAND_F_MASTER_ARCHITECTURE_V2.md` until a V3 migration gate is canonical;
3. canonical Assurance Program units;
4. canonical CF slice specs/plans/tasks;
5. V3 candidate playbook;
6. this V3.1 overlay/runbook for future V3 planning requirements;
7. donor/pattern/research sources.

No planning file can bypass a live canonical task boundary.

## 2. Current planning truth captured for this pass

At review time:

```text
repository: TheHalfMoon/commandF
main: 18819c7fdaee618f4aa86c6c3279b1acb70ec9f2
open planning PR: #91
planning branch: docs/commandf-v3-plan-research
V3/V3.1 status: planning candidate only
```

Any implementation attempt must re-read live truth. These identities are historical review context, not permanent execution pins.

## 3. Migration gate before V3 implementation authority

A future migration/reconciliation Spec Kit must be created before V3/V3.1 product implementation becomes execution authority.

The migration gate must prove at minimum:

- existing canonical CF-01..CF-16 and AF history is preserved and not renumbered;
- no active CF/AF frontier is bypassed;
- open issue state is re-read rather than assumed from this document;
- V3 G01-G40 and V3.1 G41-G50 each have exactly one closure owner or explicit shared ownership;
- all public schema/version identities are collision-free;
- dependency graph has no cycle that allows a slice to use future evidence to prove itself;
- known verified-byte/input-boundary defects are dispositioned according to Section 4;
- source qualification and donor policy remain mandatory;
- exact model/plugin boundaries are optional and do not become core authority;
- release/performance claims remain gated by AF-03/AF-04 evidence.

The migration gate must fail closed if the live repository has diverged materially from the reviewed plan.

## 4. Wave -1 — close evidence-boundary blockers

Before any V3 migration claims production-trust readiness, re-audit and close the invariants represented by live issues #35, #36, #37, #38, and #40 if they remain applicable.

Do not mechanically close issue numbers. Prove the invariants:

### Verified-byte invariant

For every cache-backed semantic consumer:

```text
bytes_consumed == bytes_digest_verified
```

No path may:

```text
verify(path) -> reopen(path) -> consume different generation
```

External process/oracle boundaries must receive staged bytes bound to the verified generation or another stronger equivalent construction.

### Bounded-input invariant

Any caller-selected persisted input that can materially allocate memory must be bounded before full allocation/parsing. The bound must be evidence-driven and shared where semantics are the same.

### Offline symmetry invariant

Local/offline acquisition may not silently have weaker resource/integrity guarantees than network acquisition unless the difference is explicit, justified, tested, and surfaced.

### Exit evidence

- deterministic regressions for every repaired boundary;
- repository-wide search/audit for equivalent patterns;
- exact-head CI/review/proof evidence;
- no semantic reinterpretation hidden inside integrity repair;
- reusable helper where multiple call sites share the invariant.

## 5. V3.1 build dependency amendments

The existing V3 dependency graph remains. V3.1 adds mandatory evidence flow:

```text
CF-17 ecosystem evidence
        |
        v
CF-18 compatibility + Decision Envelope v1
        |
        +--------------------+
        |                    |
        v                    v
CF-19 Consumer Contract v1   CF-20 oracle/probe escalation
        |                    |
        +----------+---------+
                   v
          Decision Assurance
                   |
                   v
CF-21 Decision Receipt v1 + Interop BOM/replay
                   |
                   v
CF-22 deterministic tests/counterfactual regressions
                   |
        +----------+----------+
        |                     |
        v                     v
CF-23 public developer       CF-28 decision benchmark /
platform contracts          optional model qualification
```

CF-24 controls capability/plugin/model adapter authority. CF-25/26 feed terminology/transformation evidence into the same envelope/receipt contracts; they do not invent parallel decision semantics.

## 6. CF-18 amendment — Decision Contract Foundation

CF-18 already owns compatibility coverage. V3.1 extends its must-ship contract.

### 6.1 Required public/internal schemas

Freeze before semantic implementation:

```text
schemas/commandf-decision-envelope-v1.schema.json
schemas/commandf-decision-policy-v1.schema.json
```

Schema exact locations may be adjusted by the Spec Kit after source-layout audit, but the semantic identities must remain stable once frozen.

### 6.2 Required enums

Truth class:

```text
PROVEN_COMPATIBLE
PROVEN_BREAKING
CONDITIONALLY_COMPATIBLE
RISK_DETECTED
CONFLICTING_EVIDENCE
INSUFFICIENT_EVIDENCE
UNSUPPORTED
INDETERMINATE
```

Recommended action:

```text
ALLOW
WARN
BLOCK
REQUEST_ORACLE
REQUEST_RUNTIME_PROBE
REQUEST_HUMAN
ABSTAIN
```

Completeness:

```text
COMPLETE_FOR_DECLARED_SCOPE
MATERIALLY_INCOMPLETE
UNKNOWN_DUE_TO_ERROR
```

### 6.3 Required invalid-state matrix

The schema/semantic validator must reject at least:

- `PROVEN_COMPATIBLE` with material unsupported witnesses inside protected scope;
- `PROVEN_COMPATIBLE` with required oracle marked not-run/failed/unavailable;
- `ALLOW` when governing policy forbids allow for the truth/completeness combination;
- any `PROVEN_*` produced solely from model advisory output;
- `COMPLETE_FOR_DECLARED_SCOPE` with required stages omitted;
- unknown enum/state combinations;
- evidence reference to an unknown/unbound identity;
- model confidence without exact model/calibration/policy identity;
- protected consumer references that are not part of the input decision scope.

### 6.4 Deterministic decision policy

Policy evaluation must be a pure function over versioned inputs where feasible:

```text
Decision = f(
  compatibility_findings,
  change_space_coverage,
  protected_consumers,
  evidence_state,
  oracle_state,
  runtime_state,
  policy,
  configuration
)
```

The model advisory, if enabled, is not an argument capable of directly granting `PROVEN_*` or privileged `ALLOW/BLOCK`; it can only populate advisory evidence consumed by explicit policy fields that cannot override proof requirements.

### 6.5 CF-18 test families

- exhaustive enum/cross-field schema tests;
- truth-table/property tests for deterministic policy;
- unknown/unsupported fail-closed tests;
- material-counterfactual fixtures;
- irrelevant-edit stability fixtures;
- configuration precedence determinism;
- serialization determinism;
- cross-version schema rejection/migration tests;
- no-model and model-unavailable equivalence for deterministic core semantics.

### 6.6 CF-18 exit gate

CF-18 cannot close V3.1 ownership unless G02/G03/G08/G09/G13/G14/G30/G31/G32/G33 plus G41/G42 and its share of G46/G48 are executable and evidenced.

## 7. CF-19 amendment — Consumer Contract v1

### 7.1 Schema

Freeze:

```text
schemas/commandf-consumer-contract-v1.schema.json
```

### 7.2 Required identity

A contract must bind:

```text
contract_id
contract_version
consumer_id
consumer_version
producer_scope
protected_state
source_identity
extractor_identity
configuration_identity
provenance_class
privacy_class
```

### 7.3 Dependency record shape

Every dependency record requires:

```text
dependency_id
dependency_family
canonical_target_or_expression
source_location_or_capture_ref
required_semantics
version/context constraints
confidence = observed|declared|derived
```

`derived` is allowed only for deterministic derivation with a retained derivation rule, never popularity/LLM inference.

### 7.4 Minimum supported families

Implementation order follows existing V3 priorities. The contract schema must be extensible without unknown fields silently changing meaning.

Initial families:

- FHIR package/profile/extension;
- element/path;
- terminology binding/system/version;
- SearchParameter/search interaction;
- FHIRPath;
- CQL/ELM/Library;
- SQL-on-FHIR ViewDefinition;
- CapabilityStatement expectation;
- REST interaction/operation;
- SMART scope/backend-service expectation;
- Bulk Data expectation;
- subscriptions/eventing;
- executable test/conformance reference.

### 7.5 Protected-set semantics

A certification/gate request must pass an exact protected set:

```text
protected_consumers = [consumer_contract_identity...]
```

Rules:

- zero protected consumers cannot be silently interpreted as all consumers safe;
- missing requested consumer contract => `INSUFFICIENT_EVIDENCE` or explicit policy result;
- stale contract handling must be explicit;
- duplicate/conflicting contract identities fail closed;
- contract extraction must retain source evidence.

### 7.6 Privacy boundary

Default repository qualification uses public/synthetic/metadata-only evidence. Any contract scanner that touches patient-instance data requires a separate on-premises/approved boundary, minimization rules, retention/redaction policy, and aggregate/statistical output by default.

### 7.7 CF-19 exit gate

CF-19 must close G04/G05 and its G31/G34/G36/G40 ownership plus G44 before any `can-i-certify`-style green result is considered trustworthy.

## 8. CF-20 amendment — Selective Evidence Escalation

### 8.1 Escalation policy

Freeze a deterministic rule table mapping decision/evidence state to:

```text
NOT_REQUIRED
REQUIRED
OPTIONAL_FOR_ADDITIONAL_CONFIDENCE
```

for each configured oracle/probe class.

### 8.2 Oracle/probe observation states

At minimum:

```text
NOT_REQUIRED
REQUIRED_NOT_RUN
RUN_AGREEMENT
RUN_DIVERGENCE
UNAVAILABLE
FAILED
TIMED_OUT
UNCOMPARABLE
```

Runtime probes additionally distinguish:

```text
REQUESTED
STARTED
COMPLETED
VERIFIED
CANCELLED
INDETERMINATE
```

Completion is not verification.

### 8.3 Escalation algorithm requirements

- identical deterministic inputs => identical escalation decision;
- no oracle call when policy says `NOT_REQUIRED` unless explicit user diagnostic mode is requested;
- required oracle absence prevents a proof state that depends on it;
- divergent qualified oracles produce conflict evidence, not majority-vote laundering;
- external availability is separate from semantic result;
- output/resource/time bounds are part of the oracle identity/evidence;
- batching/concurrency optimization must preserve per-resource evidence identity and deterministic ordering.

### 8.4 CF-20 exit gate

Close G06/G07/G10/G23/G31/G34/G39 plus G42/G47 ownership with retained differential/runtime evidence.

## 9. CF-21 amendment — Decision Receipt and Replay

### 9.1 Schema

Freeze:

```text
schemas/commandf-decision-receipt-v1.schema.json
```

### 9.2 Stage model

Every required stage uses:

```text
PASS
FAIL
BLOCKED
ERROR
NOT_APPLICABLE
NOT_RUN
```

Required stages depend on policy/decision request but the stage manifest itself is explicit. A required stage cannot disappear from the receipt.

### 9.3 Receipt sections

```text
receipt_identity
source_and_comparison
configuration_and_policy
protected_scope
stage_manifest
stage_results
evidence_inventory
oracle_runtime_inventory
interoperability_bom_ref
decision_envelope
migration_refs
offline_replay_ref
redaction_truncation_facts
engine_identity
```

### 9.4 Identity invalidation

Receipt semantic identity must change if any of these change:

- input package/artifact/source bytes;
- protected consumer set/version;
- rule/version;
- policy/configuration;
- required oracle/tool identity;
- terminology/mapping dependency relevant to the result;
- engine/parser/normalizer semantics;
- model advisory identity when its observation is retained;
- evidence bundle contents.

### 9.5 Replay

Offline replay must validate:

- bundle manifest/schema;
- all content digests;
- allowed tool/oracle redistribution or external requirement;
- engine compatibility;
- policy/config identity;
- expected stage availability.

Replay incompatibility is classified; it does not silently re-run with a different tool/version.

### 9.6 CF-21 exit gate

Close G11/G12/G19/G29/G35 plus G43 and produce deterministic receipt round-trip/replay evidence.

## 10. CF-22 amendment — Evidence-derived TestGen and Decision Counterexamples

CF-22 generated tests must be traces of existing deterministic evidence, not new authority.

Required flow:

```text
exact evidence witness
  -> deterministic test template selection
  -> generated fixture/assertion
  -> independent validator/test execution
  -> retained result
  -> optional promotion into regression corpus
```

AI may draft candidate test content only if the candidate is treated as untrusted and passes the same deterministic acceptance path.

Required V3.1 generated families:

- breaking witness preservation;
- consumer-specific regression;
- material counterfactual;
- irrelevant-edit stability;
- oracle disagreement reproduction;
- unsupported-surface retention;
- receipt invalid-state negative tests.

Counterexample promotion requires minimized reproducible identity and provenance.

## 11. CF-23 amendment — Stable developer platform

Every external surface must consume the same semantic contracts.

Required outputs:

- Decision Envelope v1;
- Consumer Contract v1 import/export/validation;
- Decision Receipt v1;
- stable error/partial-result taxonomy;
- capability/version support matrix.

Surfaces may include CLI, JSON, library API, daemon, LSP, CI, GitHub/GitLab, optional read-only MCP adapter, and Studio.

Contract test requirement:

> No external surface may reimplement privileged compatibility/decision logic.

The same fixture must produce semantically equivalent Decision Envelope/Receipt content across all applicable surfaces, modulo transport metadata explicitly excluded from semantic identity.

## 12. CF-24 amendment — Optional model/plugin capability boundary

### 12.1 GAX-derived advisory adapter

This adapter is optional. It must implement a provider-neutral interface conceptually equivalent to:

```text
AdvisoryInput {
  decision_state,
  finite_actions,
  evidence_refs,
  protected_scope_summary
}

AdvisoryOutput {
  action_probabilities,
  evidence_support,
  sufficiency_probability,
  abstain_signal,
  model_identity,
  tokenizer_identity,
  calibration_identity,
  policy_identity
}
```

No free-form answer is accepted as a typed decision without schema-constrained parsing and explicit classification as advisory output.

### 12.2 Capability denial

Default model/plugin authority:

```text
network: denied unless explicitly configured
filesystem: denied except declared inputs/outputs
rule_activation: denied
finding_suppression: denied
policy_mutation: denied
approval: denied
protected_action: denied
evidence_reclassification: denied
```

### 12.3 Failure semantics

Model/plugin failure yields explicit unavailable/error advisory evidence and must not alter deterministic core truth.

### 12.4 Upgrade/revocation

Exact adapter/model/plugin revision, ABI/API, calibration revision, and policy revision must be versioned. Unsupported or revoked revisions fail closed according to CF-24 policy.

## 13. CF-25/CF-26 amendments — evidence providers, not parallel decision engines

Terminology and transformation layers must emit evidence consumable by the common Decision Envelope/Receipt contracts.

They may define domain-specific evidence schemas, but cannot invent another top-level `SAFE/BREAKING` authority that bypasses CF-18 decision semantics.

Examples:

- terminology exact/equivalent/broader/narrower/compositional/no-map evidence;
- mapping loss/reversibility witnesses;
- round-trip divergence;
- field-level provenance.

Unknown content rights or unavailable terminology authority must surface as evidence/completeness limitations.

## 14. CF-27 amendment — public evidence view

Public observatory output must expose:

- immutable analyzed input identity;
- commandF engine/rule/policy identity;
- evidence truth classes;
- supported/unsupported scope;
- decision/receipt identity where publication is allowed;
- correction/supersession relationship;
- freshness timestamp for mutable telemetry;
- reproduction instructions/bundle reference where legal.

A corrected analysis supersedes; it does not erase immutable historical evidence.

No private consumer contract or sensitive interaction evidence is public by default.

## 15. CF-28 amendment — CommandFBench-Decision

### 15.1 Frozen benchmark families

At minimum:

- `Decision-KnownBreaking`
- `Decision-KnownCompatible`
- `Decision-InsufficientEvidence`
- `Decision-OracleRequired`
- `Decision-ConflictingEvidence`
- `Decision-Unsupported`
- `Decision-ConsumerImpact`
- `Decision-Counterfactual`
- `Decision-IrrelevantEdit`
- `Decision-Protocol`
- `Decision-Ops`

### 15.2 Required primary metrics

```text
unsafe_allow_rate
missed_breaking_rate
false_block_rate
consumer_impact_precision
consumer_impact_recall
risk_coverage_curve
selective_risk_summary
abstention_precision
abstention_recall
unsupported_detection_rate
oracle_escalation_precision
oracle_escalation_recall
receipt_replay_success_rate
```

Where a probabilistic adapter exists:

```text
NLL
Brier
ECE or preregistered calibration metric
sufficiency calibration
```

### 15.3 Reporting rule

The primary deployment result must report unsafe auto-allow risk **at declared coverage**. Never publish accuracy without coverage/abstention context for a selective decision system.

### 15.4 Model qualification

A model-assisted path may be recommended only if it improves a preregistered decision/efficiency objective without violating deterministic safety gates.

Possible valid result:

```text
No model benefit demonstrated; deterministic-only remains default.
```

Negative/null results are retained.

## 16. Concrete Spec Kit work packages

When each owning slice becomes eligible, its Spec Kit should include the following minimum work-package pattern.

### WP-A — truth capture

- live `main`/ruleset/active task readback;
- exact current source topology;
- relevant open issues/PRs;
- existing schemas and compatibility promises;
- donor/source pins refreshed.

### WP-B — contract freeze

- schema/enums;
- compatibility/migration policy;
- invalid-state matrix;
- privacy/rights boundary;
- non-goals.

No implementation before contract freeze for proof-critical public semantics.

### WP-C — deterministic core

- pure semantic logic where feasible;
- canonical serialization;
- stable reason codes;
- unsupported/error propagation;
- no hidden network/model dependency.

### WP-D — adversarial/property tests

- valid positive fixtures;
- malformed/unknown negative fixtures;
- counterfactuals;
- irrelevant edits;
- duplicate/ordering/path/size limits where applicable;
- fail-closed external failure.

### WP-E — evidence/oracle qualification

- exact oracle/tool identities;
- agreement/divergence/unavailable cases;
- retained raw/normalized evidence;
- deterministic ordering;
- no oracle laundering.

### WP-F — integration and replay

- CLI/API surface;
- stable machine schema;
- offline replay where applicable;
- cross-platform evidence only when claimed;
- source/consumer/policy identity binding.

### WP-G — review and canonicalization

- exact-head CI;
- mandatory security/assurance paths;
- Jev/Alibaba Open Code Review may be used as independent review inputs when the project governance authorizes them, but no external reviewer score becomes semantic authority;
- zero unresolved substantive findings or explicit accepted risk per governance;
- guarded normal merge;
- post-merge canonical readback.

## 17. Shared assurance primitives backlog

Before duplicating implementation, audit for an existing helper. If absent, assign one owning slice/AF unit and reuse it thereafter.

Required reusable primitives:

```text
bounded_read
verified_read
canonical_relative_path
canonical_json
schema_version_validation
evidence_digest
reason_code/status vocabulary
external_process_bounds
offline_bundle_manifest
redaction/truncation facts
exact source/comparison identity
```

Each primitive needs independent negative tests.

## 18. Internal donor application map

Pinned record:

`donors/commandf-v3_1-internal-pattern-sources-2026-09-29.yaml`

Use:

- **GAX**: decision/sufficiency/abstention/calibration/benchmark patterns only;
- **Ascout**: receipt/status/completeness/source-binding patterns only;
- **Kernux**: provider-neutral capability and evidence-before-done patterns only;
- **Cotra**: bounded capability plus requested/started/completed/verified/failed/cancelled/indeterminate execution-state patterns only.

No code copy is authorized by this runbook. Any code port/copy requires the slice-specific provenance gate.

## 19. Planning completeness matrix

The V3.1 overlay is considered mechanically complete only when these questions have an explicit owner and fail-closed behavior:

| Question | Owner |
| --- | --- |
| What exactly changed? | CF-18 over CF-03/04 foundations |
| Is the change-space form understood? | CF-18 |
| Which compatibility dimensions apply? | CF-18 |
| Which consumers are protected? | CF-19 |
| What exact dependencies do they declare/observe? | CF-19 |
| Is static evidence enough? | CF-18/20 |
| Which oracle/probe is required and why? | CF-20 |
| What if the oracle is unavailable/fails/disagrees? | CF-20/Decision Envelope |
| What is proven versus only risky/likely? | CF-18 |
| Is the evidence complete for the declared scope? | CF-18/21 |
| What remains unsupported or unknown? | CF-18/21/23 |
| What decision/action does policy require? | CF-18 |
| Can another machine reproduce it? | CF-21 |
| Can downstream tools rely on stable schema/exit behavior? | CF-23 |
| Can a plugin/model alter authority? | CF-24: no by default |
| Are terminology/content rights explicit? | CF-21/25 |
| Are transformation-loss claims evidence-bound? | CF-26 |
| Can public evidence be corrected without history rewrite? | CF-27 |
| Is decision quality measured at useful coverage? | CF-28 |
| Are material edits detected and irrelevant edits stable? | CF-18/22/28 |
| Are known integrity/input blockers closed before trust migration? | migration gate / repair owners |
| Are assurance primitives reused rather than forked? | AF + every slice |

If a future review finds a question with no owner, it creates a new gap ID; it must not be silently folded into prose.

## 20. No-gap review checklist before each slice implementation

The implementing agent must answer all applicable items in the Spec Kit consistency document:

- inputs and exact identity known;
- outputs and schema/version known;
- trust authority known;
- unsupported state defined;
- malformed state defined;
- partial-result semantics defined;
- resource limits defined;
- network behavior defined;
- filesystem/process capability defined;
- privacy classification defined;
- software/content/model/data rights defined;
- deterministic ordering/serialization defined;
- cache invalidation identity defined;
- error/reason vocabulary defined;
- upgrade/migration/rejection policy defined;
- oracle independence and disagreement semantics defined;
- consumer/version scope defined;
- offline/replay behavior defined;
- test oracle and counterexamples defined;
- performance claim boundary defined;
- evidence retention and redaction defined;
- review/merge/post-merge evidence defined.

Any unanswered applicable item blocks `READY_FOR_IMPLEMENTATION`.

## 21. Product-level definition of done

The V3.1 roadmap may eventually claim best-in-class evidence-bound interoperability change decisions only when:

1. declared artifact/protocol change space is machine-covered or fail-closed unsupported;
2. protected consumer contracts are explicit and version-bound;
3. `PROVEN_*` cannot be produced from missing required evidence;
4. oracle/runtime escalation is explicit and reproducible;
5. Decision Receipt records what ran, failed, was blocked, and was not run;
6. another machine can validate/replay the declared deterministic result with allowed inputs;
7. optional model assistance is independently calibrated/evaluated and non-authoritative;
8. unsafe auto-allow risk is reported at declared coverage;
9. material counterfactuals change decisions when expected and irrelevant edits do not;
10. known integrity/input blockers are not outstanding for the claimed trust boundary;
11. no private/PHI evidence is required for core qualification;
12. public claims are bound to retained evidence and negative/null results are not hidden.

Until those conditions are executed and canonical, they remain roadmap requirements rather than product claims.
