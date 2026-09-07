# commandF Agent Evidence and Source Adoption Plan — 2026-09-08

Status: **FUTURE_ARCHITECTURE_PLAN / NO_CURRENT_IMPLEMENTATION_AUTHORITY**

This plan deepens commandF's future agent/Copilot architecture and donor-adoption strategy after a repository-wide plan review and a focused study of `Tencent/WeKnora`, `Tencent/RoMem`, `Tencent/LoopForge`, and `Tencent/SkillHone` alongside the source families already retained by commandF.

It does not alter the active AF-02 dependency order, does not authorize source-changing work outside an approved CF/AF/research unit, and does not make any model, memory system, optimizer, donor implementation, or external score semantic authority.

The deterministic artifact/diff/check/graph/impact spine remains the product trust anchor.

## 1. Planning correction

The existing V2 architecture correctly keeps the agent plane optional and replaceable, but the future agent plane is still too coarse. Treating "agent runtime" as one subsystem would couple concerns that must remain independently governable:

1. capability and sandbox execution;
2. durable execution evidence;
3. workflow orchestration and resumability;
4. durable memory and retrieval;
5. evaluation authority and optimization;
6. model adapters and user-facing Copilot behavior.

The corrected plan splits those concerns and orders them so commandF can gain useful infrastructure without making an LLM a prerequisite.

## 2. Design synthesis from retained sources

### DeepSeek Harness — capability seams

Retain the already-qualified patterns:

- replaceable service/provider/consumer capability seams;
- typed tool contracts;
- guarded pre-execute/execute/post-execute lifecycle;
- per-agent capability restrictions;
- sandbox and approval as separate policy seams;
- append-only execution events;
- reversible plugin effects;
- model-visible-means-logged.

### WeKnora — bounded sandbox and memory controls

Retain selected patterns, not the complete product runtime:

- provider-neutral session sandbox lifecycle;
- explicit workspace boundaries and failure-preserving behavior;
- script validation before execution;
- principal/workspace-scoped memory;
- independent workspace, agent, and user memory enablement;
- user forgetting/tombstone behavior that prevents deleted information from being silently re-derived;
- sensitive-content redaction before durable memory storage;
- bounded recall with explicit ranking-mode/skip diagnostics;
- distinction between memory injected into a model and memory reported as relevant;
- memory failure degrading to "no memory" rather than rewriting product semantics.

### LoopForge — resumable workflow state

Retain selected workflow patterns:

- explicit task-scoped persistent state rather than conversation-state inference;
- requirement, design, implementation, review, test, and delivery stages;
- independent role boundaries for implementation, review, and test;
- resume from the first unfinished stage;
- preserve previous artifacts on interruption;
- never mark a failed stage complete;
- do not scan or recover unrelated task state;
- retain decisions and verification artifacts for handoff and retrospective analysis.

GitHub Spec Kit remains commandF's specification process authority. LoopForge-derived state patterns complement it; they do not replace it.

### SkillHone — evaluator/candidate isolation

Retain selected evaluation and optimization patterns:

- separate candidate behavior from evaluator authority;
- isolated per-item execution workdirs;
- separate probe, PR-validation, and final-test visibility;
- never expose final-test/gold material to the optimizing candidate;
- score provenance tied to exact split, output, workdir/run, and candidate identity;
- redacted observation surfaces for diagnosis;
- diagnose infrastructure, tool, compiler/validator, verifier, and candidate failures separately;
- one focused change per optimization cycle so improvement/regression attribution remains interpretable;
- use standard compilers/validators as direct evidence instead of inventing duplicate validators.

### RoMem — temporal research, not authority

RoMem is useful as research material for time-sensitive memory and contradiction handling. The current pinned repository state lacks root license evidence, so its source, checkpoints, datasets, and bundled baselines remain unqualified for copy or redistribution.

The useful commandF lesson is architectural and experimental:

- time must be part of a memory fact's identity/evidence, not merely an optional display label;
- static and rapidly changing claims should not be treated as equally persistent;
- contradiction handling should preserve history rather than silently overwrite evidence;
- learned temporal ranking can be compared in `commandF Bench`, but cannot become semantic or policy authority without independent evidence.

## 3. New architecture: six independent future capabilities

### A. Evidence substrate — build before any agent runtime

Purpose: make every future agent action replayable and auditable without requiring a model.

Required contracts:

- immutable execution/session/task/run identifiers;
- append-only ordered events;
- exact repository/base/head/tree identity where code is involved;
- exact tool/provider/version/blob or digest identity;
- canonical typed tool input/output envelopes;
- content digests for material inputs and outputs;
- actor/principal/capability identity;
- timestamps separated into `observed_at`, `effective_at`/validity where relevant, and recorded-at time;
- approval/denial events as durable facts;
- deterministic redaction metadata when sensitive content is excluded;
- reconstructable model-visible context projection;
- explicit references to authoritative evidence instead of copying authority into model memory.

A model-visible context that cannot be reconstructed from retained evidence is not eligible for authoritative workflow claims.

### B. Capability and sandbox plane

Purpose: execute optional agent tools behind a deny-by-default capability boundary.

Required properties:

- typed tool schemas and canonical validated outputs;
- explicit capability grants by role/task;
- separate execution, network-egress, secret-access, filesystem, approval, and resource policies;
- path containment and no silent traversal outside the task workspace;
- no host Docker socket or equivalent privileged host control by default;
- explicit remote/local sandbox provider boundary;
- CPU, memory, wall-time, output-size, and operation-count budgets;
- network allow/deny policy that is independently reviewable;
- secrets never included in generic logs, prompts, or replay artifacts;
- destructive or externally visible operations can enter durable `approval_required`, `approved`, `denied`, or `expired` states;
- failure to obtain required policy/approval is a fail-closed outcome, not an implicit retry with more authority.

### C. Resumable workflow orchestration

Purpose: make long-running work recoverable without trusting chat history.

Required state machine properties:

- one task has one canonical workflow-state record;
- stage transitions have explicit preconditions and evidence references;
- stages cannot become complete when required output/evidence is missing;
- source-changing commits invalidate exact-head CI/review evidence and return the workflow to the required qualification stage;
- implementation, independent review, and qualification/test roles remain separable;
- resume starts at the first incomplete/invalidated stage;
- abort is explicit and preserves artifacts;
- resumption cannot scan or adopt unrelated task state;
- task state references repository truth rather than replacing repository truth;
- all stage artifacts are bounded, versioned, and inspectable.

This orchestration layer may support development workflows and future product workflows, but it must not replace `AGENTS.md`, Spec Kit packages, GitHub PR state, required checks, or canonical repository governance.

### D. Bounded durable memory

Purpose: improve continuity and retrieval without turning memory into truth.

Memory record minimum fields:

- principal/workspace scope;
- source/evidence reference;
- origin category (explicit user statement, extracted observation, tool result, workflow fact, research note, etc.);
- created/observed time;
- validity interval or explicit unknown validity when temporally meaningful;
- supersedes/superseded-by relation where applicable;
- sensitivity/redaction state;
- user-visible deletion/tombstone state;
- retrieval/ranking metadata separated from the stored fact;
- optional confidence as a ranking hint only, never as proof.

Memory policy:

- disabled means no memory use, not degraded correctness;
- user deletion must block silent re-derivation from the same source event unless the user explicitly re-authorizes it;
- credentials and highly sensitive material are rejected/redacted before storage;
- recall has hard item/token/byte/time budgets;
- lexical/vector/temporal retrieval failures are observable and may fall back safely;
- memory output never replaces exact GitHub, validator, standard, package, or evidence lookup when current truth matters;
- recalled material is labelled as memory/context, not as verified interoperability evidence.

### E. Evaluation and optimization authority

Purpose: improve agents, prompts, recipes, and optional automation without allowing the candidate to grade itself.

Required isolation:

- candidate code/config/prompt is separate from evaluation authority;
- evaluator/verifier contracts are base-controlled or otherwise independently protected;
- final held-out material is never visible during optimization;
- probe/iteration evidence is redacted to the smallest useful diagnostic surface;
- each evaluation binds exact candidate identity, evaluator identity, environment/tool identity, split, run attempt, outputs, and score provenance;
- infrastructure failures are not silently counted as candidate-quality failures;
- candidate-quality failures are not excused as infrastructure without evidence;
- one focused source change per optimization cycle where attribution matters;
- no optimizer-selected change becomes trusted until ordinary commandF CI/review/provenance gates also pass;
- final evaluation cannot be reused after a source-changing commit.

AF-02's anti-forgery and exact-head work is a foundational precursor, not something this plan modifies retroactively.

### F. Copilot/model experience

Purpose: expose optional AI assistance after the evidence/capability/evaluation foundations exist.

Initial allowed behaviors:

- explain deterministic findings;
- retrieve cited evidence;
- triage likely causes;
- propose mappings, tests, policies, recipes, or remediation;
- prepare review summaries;
- draft changes in isolated workspaces.

Initial prohibited authority:

- no model-only breaking-change judgment;
- no model-only terminology equivalence;
- no model-only mapping correctness claim;
- no direct bypass of validators, policy gates, human approval, or protected evaluator authority;
- no production mutation merely because an optimizer score improved.

## 4. Gap analysis found by this review

The existing plan preserves strong product and assurance coverage, but the following future-agent/source-adoption gaps were under-specified.

### G-A1 — donor inventory without a portfolio decision lifecycle

The repository preserves many sources, but candidate retention is not yet a portfolio that answers: what capability does this source serve, what gap does it address, what is the preferred adoption mode, what source supersedes it, and what event reopens qualification?

**Plan correction:** add a source-portfolio layer that maps donor → capability → gap → adoption mode → activation unit → rights evidence → supersession/requalification rule.

### G-A2 — one broad future-agent concept instead of independent trust domains

**Plan correction:** split evidence, capabilities, orchestration, memory, evaluation, and model UX as independently replaceable layers.

### G-A3 — resumability and invalidation semantics were not explicit

**Plan correction:** persist task-scoped workflow state; exact-head source changes invalidate affected evidence and reopen qualification stages.

### G-A4 — durable memory had no commandF-specific trust model

**Plan correction:** memory is contextual, scoped, provenance-bearing, erasable, temporally qualified, and never semantic authority.

### G-A5 — no explicit forgetting/sensitive-memory behavior

**Plan correction:** add deletion tombstones, re-derivation guards, pre-storage redaction, and user/workspace/agent enablement boundaries.

### G-A6 — temporal contradiction policy was missing

**Plan correction:** preserve observation history, validity intervals, and supersession rather than destructive overwrite; compare learned ranking only through research evidence.

### G-A7 — evaluator/candidate authority separation was not generalized beyond current AF work

**Plan correction:** make protected evaluator authority, held-out split isolation, and exact candidate binding mandatory for future agent optimization.

### G-A8 — pass/fail evidence lacked a general failure-layer taxonomy

**Plan correction:** future observation surfaces distinguish product/candidate, verifier, compiler/validator, tool/runtime, infrastructure, and policy/approval failures.

### G-A9 — optimization could overfit visible evaluation data

**Plan correction:** separate `probe`, `pr_validation`, and final held-out evaluation; final data is never an iteration signal.

### G-A10 — sandbox/network/resource policy was not detailed enough

**Plan correction:** deny-by-default capability grants, separate network and approval policy, explicit resource budgets, path containment, and no privileged host control by default.

### G-A11 — model-context reproducibility stopped at a principle

**Plan correction:** define an evidence projection contract so anything sent to a model can be reconstructed from canonical events and content hashes.

### G-A12 — copying prior art could become the architecture

**Plan correction:** donor code is an implementation accelerator, not design authority. Prefer narrow `DEPEND`, `IMPORT`, `ORACLE`, or small `PORT` boundaries; use `COPY` only when the destination boundary materially benefits and provenance/tests/notices are preserved.

### G-A13 — source changes and upstream drift lacked explicit requalification triggers

**Plan correction:** pin exact upstream commit and path/blob. Any pin/path/license/third-party change reopens qualification before further adoption.

### G-A14 — future-agent quality had no dedicated benchmark lane

**Plan correction:** `commandF Bench` may later host agent/tool/memory evaluation, but it must keep final held-out data and semantic/product benchmarks separately governed.

## 5. Source portfolio strategy for the repository's retained inventory

The discovery annex remains the inventory authority. This plan adds a decision strategy across that inventory.

| Source family | Default commandF role | Preferred adoption mode | Never silently becomes |
|---|---|---|---|
| HL7/openEHR/OMOP standards and official artifacts | semantic/specification input | `IMPORT` / authoritative reference | model memory or copied interpretation |
| HL7 Validator / IG Publisher and independent FHIR implementations | validation/differential evidence | `ORACLE` / process boundary | single hidden correctness authority |
| mapping engines and mapping corpora | parse/analyze/compare prior art | `IMPORT`, `ORACLE`, selected `PORT` | proprietary universal mapping language |
| terminology services | terminology evidence | `DEPEND`/service adapter/`ORACLE` | license to redistribute terminology content |
| query/data-plane systems | query semantics/reference | `ORACLE`, adapter, selected `DEPEND` | proof that all query dialects are equivalent |
| provenance/supply-chain systems | mature evidence plumbing | `DEPEND`/`IMPORT` | semantic correctness claim |
| policy/security systems | bounded policy/identity adapters | `DEPEND`/adapter | universal healthcare authorization model |
| testing/fuzzing/review tooling | assurance evidence | pinned tool `DEPEND` / CI execution | unexplained trust score |
| GitHub Spec Kit / LoopForge patterns | development process and resumability | process dependency / template adaptation | repository truth or reviewer authority |
| DeepSeek Harness / WeKnora patterns | optional capability/sandbox/memory architecture | selected `PORT`/adapter after activation | artifact critical-path dependency |
| SkillHone patterns | protected evaluation and optimization | selected `PORT`/process adaptation | self-grading model authority |
| RoMem and other memory research | benchmark/research hypothesis | `STUDY`, later research implementation if rights permit | runtime semantic authority |
| research datasets/benchmarks | reproducible measurement | governed `IMPORT`/reference | unrestricted redistribution right |

## 6. Donor adoption decision protocol

For every future donor-code activation:

1. identify the exact commandF gap/capability it serves;
2. prove the source is not redundant with an already selected implementation;
3. pin exact repository commit and source blob/path;
4. classify rights at file/artifact level, including third-party material;
5. choose the least-coupled adoption mode (`DEPEND` → `IMPORT` → `ORACLE` → small `PORT` → `COPY` only when justified);
6. define the destination trust boundary before code is copied;
7. retain donor-origin metadata and required notices;
8. add focused positive, negative, conflict, and failure-path tests;
9. assess security, dependency, release, and SBOM impact;
10. bind implementation to an authorized CF/AF/research unit;
11. qualify exact candidate head through canonical CI/review/provenance;
12. record why the source was selected over alternatives and what would trigger replacement/requalification.

A broad permission statement is project authorization to pursue lawful adoption; it is not a substitute for upstream license/rights evidence required by `docs/PROVENANCE_AND_DONOR_POLICY.md`.

## 7. Candidate activation sequence

These stages are planning candidates, not new canonical CF/AF identifiers and not implementation authorization.

### Stage 0 — current canonical work remains first

Finish the currently authorized AF-02 sequence under its frozen contracts. This plan must not alter T022–T028 or reuse stale exact-head evidence.

### Stage 1 — source portfolio and evidence schema planning

Plan a machine-readable donor portfolio and execution-evidence envelope. No LLM dependency.

Exit criteria:

- donor lifecycle states and requalification triggers specified;
- exact source/path/license/permission fields specified;
- execution event/evidence envelope specified;
- no current product semantics changed.

### Stage 2 — capability/sandbox foundation

Introduce typed optional capabilities and bounded sandbox execution only through an authorized future unit.

Exit criteria include:

- deny-by-default capability policy;
- network/secret/filesystem/resource policies;
- durable approval state;
- sandbox path/resource tests;
- exact provider/tool evidence;
- no core command dependency.

### Stage 3 — resumable workflow evidence

Add task-scoped workflow state and stage invalidation/replay.

Exit criteria include:

- interruption/resume tests;
- failed-stage non-completion tests;
- source-change invalidation tests;
- role-boundary tests;
- no replacement of GitHub/Spec Kit authority.

### Stage 4 — bounded memory

Introduce optional scoped memory only after evidence and policy foundations exist.

Exit criteria include:

- opt-in/disable behavior;
- delete/tombstone semantics;
- sensitive-content rejection/redaction;
- provenance and temporal metadata;
- recall budgets and diagnostics;
- stale/contradiction tests;
- proof that disabling memory does not change deterministic command results.

### Stage 5 — protected evaluation harness

Introduce isolated agent/recipe/prompt evaluation.

Exit criteria include:

- candidate/evaluator separation;
- probe/PR-validation/final-test split policy;
- held-out anti-leakage tests;
- exact score provenance;
- failure-layer diagnostics;
- one-change attribution policy;
- source-change requalification.

### Stage 6 — commandF Copilot capabilities

Only after prior stages are proven may model-assisted product workflows be activated.

Start read-mostly and proposal-first. Mutating workflows require deterministic acceptance and explicit policy/human authority.

### Stage 7 — temporal-memory research

Use `commandF Bench` to compare deterministic validity/supersession baselines with qualified temporal retrieval/reranking methods. No research score becomes a product guarantee without reproduced evidence and an explicit product decision.

## 8. Benchmark and evidence plan

Future agent-plane evaluation must be multidimensional. Candidate dimensions:

- task success against deterministic acceptance;
- false-positive/false-negative proposal rate;
- tool-call validity and policy violations;
- sandbox/resource-budget compliance;
- reproducibility/replay success;
- context reconstruction completeness;
- interruption/resume correctness;
- memory precision/relevance and stale-memory rate;
- contradiction handling and temporal validity;
- sensitive-memory rejection/deletion correctness;
- evaluator contamination/leakage rate;
- infrastructure-vs-candidate failure classification accuracy;
- human review effort;
- runtime/cost as secondary metrics.

No single scalar "agent quality" or "trust" score is sufficient.

## 9. Explicit non-goals

This plan does not authorize:

- replacing commandF's Rust deterministic core with an agent framework;
- introducing a mandatory vector database or knowledge graph for V1 commands;
- making recalled memory or model output authoritative interoperability evidence;
- importing WeKnora's full RAG product, LoopForge host bundles, SkillHone's Forgejo runtime, or RoMem checkpoints/datasets as default dependencies;
- leaking hidden evaluation data into optimization;
- copying code without exact-path provenance and rights review;
- using source availability as a reason to duplicate mature standards/oracles;
- modifying current AF-02 frozen contracts.

## 10. Completion definition for this planning layer

This planning layer is complete when:

- the source study and donor manifest pin the reviewed Tencent sources;
- this architecture/gap/adoption plan is present in repository history;
- the Plan Index references it as future planning coverage;
- the active AF-02 frontier remains unchanged;
- no source code has been adopted merely because it was studied;
- any future implementation must still obtain its own Spec Kit authority and exact-head qualification.
