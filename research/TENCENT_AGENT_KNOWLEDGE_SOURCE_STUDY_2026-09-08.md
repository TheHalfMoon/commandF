# Tencent Agent and Knowledge Source Study — 2026-09-08

Status: **STUDY_COMPLETE / PLAN_INTEGRATED / NO_CODE_ADOPTION**

This document records a bounded source qualification and architecture study for four public Tencent repositories supplied as commandF source candidates:

- `Tencent/WeKnora`
- `Tencent/RoMem`
- `Tencent/LoopForge`
- `Tencent/SkillHone`

The study is now integrated into `docs/COMMAND_F_AGENT_EVIDENCE_AND_SOURCE_ADOPTION_PLAN_2026-09-08.md` and the authoritative plan-set index. It creates no current product implementation authority, no AF-02 implementation authority, and no source-code adoption by itself.

The current AF-02 contracts and dependency order remain unchanged.

## Guardrails

- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`, `docs/COMMAND_F_PLAN_INDEX.md`, `docs/PROVENANCE_AND_DONOR_POLICY.md`, `AGENTS.md`, and the active AF-02 package remain authoritative for current execution.
- `docs/COMMAND_F_AGENT_EVIDENCE_AND_SOURCE_ADOPTION_PLAN_2026-09-08.md` is future planning coverage only. It does not create a new canonical CF/AF identity or bypass a Spec Kit activation gate.
- A source listed here remains study material unless a future authorized CF, AF, or research unit activates exact source paths through the provenance/adoption process.
- No source in this study becomes semantic authority.
- No source is allowed to weaken exact-head CI, independent review, hidden-evaluation separation, fail-closed behavior, deterministic evidence, or provenance requirements.
- Repository-level licensing is not treated as sufficient for later copying. Any future source adoption requires exact file/path and third-party review.
- The Founder has authorized commandF to pursue source reuse from supplied project sources. This is project authorization, not a substitute for upstream license, third-party, data, model-weight, or redistribution-rights evidence required by `docs/PROVENANCE_AND_DONOR_POLICY.md`.

## Qualified study set

| Source | Immutable study pin | License evidence at pin | commandF relevance | Current disposition |
|---|---|---|---|---|
| `Tencent/WeKnora` | `647848f3954dae34473b8a8d0e0eef5e0fb3a58e` | Root `LICENSE`: MIT for the project with separately licensed third-party components | Sandbox lifecycle, scoped capabilities, approval, bounded durable memory, forgetting/redaction, recall observability | `STUDY` |
| `Tencent/RoMem` | `39ac1417b4db41ea729e5c3be71ac20de54da993` | No root `LICENSE` file present at the pinned repository state | Temporal contradiction, history preservation, temporal-ranking research | `STUDY_ONLY_NO_COPY` |
| `Tencent/LoopForge` | `09c765286f549624dd95434e1e6ef2249657cbeb` | Root `LICENSE`: MIT with third-party attribution for included prior art | Persisted task state, staged delivery, interruption/resume, role separation, durable handoff evidence | `STUDY` |
| `Tencent/SkillHone` | `7d565839fb4dc74f9c77f09ace660e1c0484e048` | Root `LICENSE`: MIT | Candidate/evaluator isolation, held-out split discipline, score provenance, redacted diagnosis, one-change attribution | `STUDY` |

The exact source paths retained for each repository are recorded in `donors/tencent-agent-knowledge-sources-2026-09-08.yaml`.

## 1. WeKnora

### Relevant architecture patterns

The pinned WeKnora source exposes useful patterns for an optional future commandF capability/memory plane without requiring commandF to adopt the complete WeKnora product:

- provider-neutral session sandbox lifecycle rather than application logic directly owning one sandbox provider;
- task/session workspace boundaries with explicit input/output roots;
- script validation before execution;
- failure-preserving workspace preparation rather than deleting/replacing existing state to make an operation appear successful;
- principal/workspace-scoped durable memory;
- independent workspace, agent, and user memory enablement;
- user deletion/tombstone behavior that blocks deleted information from being silently re-derived from the same source event;
- sensitive-content redaction before durable memory storage, including rejection where redaction leaves no useful content;
- bounded recall and prompt budgets;
- explicit lexical/vector ranking diagnostics and skip reasons;
- distinction between memory injected as standing context and memory genuinely recalled as relevant to the current question;
- memory failure degrading to an empty contextual enhancement rather than changing deterministic product correctness;
- auditable recall and tool/model observability;
- curated minimum tool exposure as a defense-in-depth control.

These patterns align with commandF's existing requirements for explicit guard stages, scoped capabilities, separate sandbox/approval policy, durable replayable evidence, and reconstructable model-visible context.

### commandF-specific lessons

1. **Memory is context, not authority.** Current repository, package, standard, validator, and evidence truth must be re-read from authoritative sources when freshness matters.
2. **Forgetting is a security/privacy invariant.** Deleting a memory is incomplete if background extraction can silently recreate it from the same source event.
3. **Memory provenance and time belong in the record.** A retrieved sentence without origin and validity context is unsafe for interoperability work.
4. **Injection and relevance are separate facts.** Standing context must not be presented to users or evaluators as though it was evidence selected by the current task.
5. **Failure must be diagnosable.** Lexical/vector/disabled/empty paths should be observable without turning model telemetry into semantic evidence.

### Non-adoptions

This study does **not** justify:

- adopting WeKnora's complete RAG/knowledge-base product surface;
- introducing its Go application stack into commandF core;
- making long-term memory part of FHIR package/diff/check/graph/impact semantics;
- treating recalled or model-generated material as interoperability authority;
- exposing a host Docker socket or equivalent privileged runtime without a commandF-specific threat model and separate authorization;
- copying any exact source path until file-scoped license and third-party inheritance are rechecked at the activation pin.

## 2. RoMem

### Relevant research pattern

RoMem studies temporal memory using continuous-time ranking, a learned relation-dependent temporal gate, and non-destructive ranking of changing facts. Its central commandF relevance is **research methodology**, not immediate runtime architecture.

The useful ideas to test independently are:

- time-sensitive facts should carry explicit temporal meaning rather than an optional display timestamp;
- relations differ in expected volatility;
- a changed fact should not require erasing the historical observation that preceded it;
- contradiction handling can preserve history while changing retrieval preference by query time;
- temporal retrieval/ranking can be evaluated as one method against deterministic validity-interval and supersession baselines.

The commandF research charter already treats temporal preservation as a first-class measurement. Future `commandF Bench` work may therefore compare explicit temporal-preservation assertions and contradiction cases across multiple methods.

### Licensing and evidence boundary

No root `LICENSE` file was present at the pinned repository state. Consequently, absent separate verifiable rights evidence:

- no RoMem source code is qualified for copy, port, dependency, or vendoring;
- bundled checkpoints, model weights, datasets, and baseline implementations are not qualified for use or redistribution;
- repository or README publication claims are not commandF evidence;
- paper metadata, publication status, datasets, and experimental claims require independent qualification before any research protocol freeze;
- learned temporal ranking is never semantic or policy authority.

The Founder project authorization does not remove these external-rights requirements.

## 3. LoopForge

### Relevant workflow patterns

LoopForge persists task-scoped workflow state and delivery artifacts across requirement, design, implementation, independent review, testing, and delivery. Its resume path reads durable workflow state and routes from the first unfinished stage rather than reconstructing progress from conversation memory.

The strongest commandF-relevant patterns are:

- one task owns one explicit durable workflow state;
- stage status, next target, and last event are persisted;
- completed artifacts survive interruption;
- failed stages are not rewritten as completed;
- resume starts at the first unfinished stage;
- role-specific work receives bounded allowed inputs and required outputs;
- implementation, review, and testing remain separable responsibilities;
- unrelated task state is not scanned or adopted during resume;
- workflow artifacts preserve why work advanced, stopped, or failed.

### commandF-specific correction

commandF needs one additional invariant beyond the generic LoopForge pattern: **source-changing commits invalidate exact-head qualification evidence**. Therefore a future commandF workflow-state layer must be able to reopen CI/review/provenance stages when the candidate SHA changes. Durable workflow state is navigation evidence; it never overrides live GitHub repository truth.

### Non-adoptions

LoopForge does not replace:

- GitHub Spec Kit;
- commandF's canonical Spec Kit packages;
- `AGENTS.md`;
- GitHub PR/branch/commit truth;
- exact-head CI;
- independent review;
- repository rulesets or protected evaluator authority.

Host-specific agent bundles are not commandF execution authority.

## 4. SkillHone

### Relevant evaluation patterns

SkillHone's strongest commandF-relevant idea is the separation between behavior being changed and the authority measuring it. Its evaluation model distinguishes:

- the public candidate/skill repository;
- the protected evaluation repository and verifier contract;
- isolated solver workdirs;
- a redacted observation surface used for diagnosis;
- iteration/probe evidence;
- PR-validation evidence;
- final held-out test evidence.

It also requires score provenance to identify the exact run/split/output source and prohibits exposing hidden gold/test material to the optimizer.

### Failure-layer diagnosis

A pass/fail or aggregate score alone is insufficient. The useful diagnostic surface distinguishes at least:

1. candidate/product behavior failure;
2. verifier or rubric failure;
3. compiler/parser/validator failure of a produced artifact;
4. tool/runtime failure;
5. infrastructure failure;
6. policy/approval failure.

This distinction avoids changing product behavior to compensate for broken infrastructure and avoids dismissing real candidate failures as infrastructure noise.

For compiler-like artifacts, standard public compilers/parsers/validators should be invoked directly where possible instead of creating duplicate validators that can drift from the real artifact contract.

### Optimization discipline

Retain these principles for any future commandF agent/recipe/prompt optimization:

- diagnose before fixing;
- use only redacted iteration evidence;
- never expose final held-out material during optimization;
- bind every score to exact candidate and evaluator identities;
- keep one focused source change per optimization cycle where attribution matters;
- source-changing commits invalidate prior final evaluation;
- optimizer-selected changes still require ordinary commandF CI, review, provenance, and product-evidence gates.

### AF-02 boundary

AF-02 is already planning-frozen. This study does not add new T022-T028 algorithms, policies, fixtures, workflow topology, schemas, tool identifiers, or evidence requirements. SkillHone-derived patterns are future architecture evidence and may inform later work only through separately authorized governance.

## 5. Cross-source synthesis

The four repositories are most valuable when their useful patterns are **composed around commandF's deterministic core rather than copied as one agent stack**.

The resulting future architecture is split into six independently governable trust domains:

1. **Evidence substrate** — append-only replayable execution/model-context evidence before any LLM dependency.
2. **Capability and sandbox plane** — typed, deny-by-default, resource-bounded optional tool execution.
3. **Resumable workflow orchestration** — task-scoped stage state that can be invalidated by repository truth.
4. **Bounded durable memory** — scoped, erasable, sensitive-data-aware, temporally qualified context that is never semantic authority.
5. **Evaluation and optimization authority** — protected evaluator/candidate separation, held-out data isolation, exact score provenance, and failure-layer diagnosis.
6. **Copilot/model experience** — proposal/explanation/retrieval/triage/drafting after the prior layers are proven.

This synthesis is recorded in `docs/COMMAND_F_AGENT_EVIDENCE_AND_SOURCE_ADOPTION_PLAN_2026-09-08.md`.

## 6. Gaps found during this pass

The review found future-agent/source-adoption gaps that were not explicit enough in the prior plan:

- donor inventory lacked a unified portfolio decision/requalification lifecycle;
- the future agent plane was too broad and mixed independent trust domains;
- resumability did not explicitly model source-change evidence invalidation;
- durable memory lacked a commandF-specific trust/provenance/forgetting model;
- temporal contradiction and validity were not explicit memory contracts;
- evaluator/candidate separation was not generalized as a future optimization rule;
- pass/fail evidence lacked a reusable failure-layer taxonomy;
- visible iteration data could overfit future optimizers without protected split discipline;
- sandbox/network/secret/filesystem/resource/approval policies were not detailed enough;
- `model-visible means logged` lacked a concrete reconstruction/evidence-envelope plan;
- source availability could encourage architecture-by-copying without a donor portfolio decision protocol;
- upstream pin/path/license drift lacked an explicit requalification trigger;
- agent/tool/memory quality lacked a future multidimensional benchmark lane.

These gaps are now retained explicitly in the future plan rather than being inserted into the healthcare-interoperability gap ledger as though they were domain claims.

## 7. Source-portfolio conclusion across commandF's retained inventory

The broader commandF discovery annex already retains many standards, validators, mapping engines, terminology systems, policy engines, data/query systems, testing tools, review tools, provenance systems, runtime candidates, and research corpora.

The plan correction is to treat those sources as a **portfolio**, not a shopping list:

- official standards/artifacts → `IMPORT`/authoritative reference;
- validators and independent implementations → `ORACLE`/process boundary;
- stable libraries/services → `DEPEND` when measured need exists;
- declarative mappings/schemas/tests → `IMPORT` where rights permit;
- mature policy/provenance/security plumbing → adapters/dependencies rather than reinvention;
- architecture/process patterns → `STUDY` or narrow `PORT`;
- `COPY` only when a destination trust boundary materially benefits and exact provenance, rights, notices, tests, security, and SBOM impact are recorded;
- research code/data → separate research qualification, never silent product authority.

Every future donor activation should answer:

1. What exact commandF gap/capability does it serve?
2. What existing source or implementation does it supersede or complement?
3. Why is the chosen adoption mode the least coupled option?
4. What exact paths/blobs and rights are involved?
5. What is the destination trust boundary?
6. What tests prove intended and failure behavior?
7. What event forces requalification or replacement?

## 8. commandF-specific decisions

1. **No current core adoption.** The deterministic FHIR package, inspect, diff, check, graph, and impact path remains independent of these agent/memory/workflow runtimes.
2. **No current AF-02 contract change.** The study is comparative/future-planning evidence only while AF-02 executes its canonical plan.
3. **Evidence before agents.** Future agent work should first establish a model-independent execution/evidence substrate.
4. **Capability boundaries before model autonomy.** Sandbox, network, secrets, resources, and approvals must be independent policy surfaces.
5. **Workflow state is not repository truth.** Resume state must revalidate live GitHub state and invalidate stale qualification evidence after source changes.
6. **Memory is optional context.** WeKnora-derived scope/forgetting/redaction/recall patterns are retained; deterministic commands must behave correctly with memory disabled.
7. **Temporal memory is research-first.** RoMem-derived temporal concepts remain benchmark/research hypotheses until rights and evidence are independently qualified.
8. **Protected evaluation precedes optimization.** SkillHone-derived evaluator/candidate isolation and held-out split discipline are retained for future optimization work.
9. **Copilot is last, not first.** Models may propose/explain/retrieve/triage/draft only after evidence, capability, workflow, memory, and evaluation foundations are separately proven.
10. **Reuse is encouraged but bounded.** The Founder authorizes pursuit of lawful source reuse; commandF still records exact upstream rights and provenance before public adoption.

## 9. Evidence limitations

- This pass inspected pinned repository source and documentation; it did not execute the upstream systems.
- No upstream security audit or reproducibility replication was performed.
- RoMem licensing remains unresolved at the pinned repository root.
- No paper result, benchmark score, model checkpoint, or dataset claim from these sources has been reproduced by commandF.
- No copied/ported source has been introduced by this study.
- A later pin/path/license/third-party change invalidates source-level adoption conclusions until the changed material is re-reviewed.

## 10. Current execution impact

None.

PR #82 and AF-02 T022 remain the active execution frontier. This planning/source-study stack must not be merged ahead of the current exact-base T022 qualification if moving canonical `main` would invalidate active evidence.

The next implementation authority after this study must still come from live canonical repository governance, not from this document or conversation memory.
