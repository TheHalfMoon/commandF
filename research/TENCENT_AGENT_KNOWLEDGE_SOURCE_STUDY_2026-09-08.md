# Tencent Agent and Knowledge Source Study — 2026-09-08

Status: **STUDY_ONLY**

This document records a bounded source qualification pass for four public Tencent repositories. It creates no commandF product authority, no AF-02 implementation authority, and no source-code adoption. The current AF-02 contracts and dependency order remain unchanged.

## Guardrails

- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`, `docs/COMMAND_F_PLAN_INDEX.md`, `docs/PROVENANCE_AND_DONOR_POLICY.md`, `AGENTS.md`, and the active AF-02 package remain authoritative.
- A source listed here is retained for study only unless a future authorized CF, AF, or research unit activates it through a separate provenance/adoption gate.
- No source in this study becomes semantic authority.
- No source is allowed to weaken exact-head CI, independent review, hidden-evaluation separation, fail-closed behavior, deterministic evidence, or provenance requirements.
- Repository-level licensing is not treated as sufficient for later copying. Any future source adoption requires exact file/path and third-party review.

## Qualified study set

| Source | Immutable study pin | License evidence at pin | commandF relevance | Current disposition |
|---|---|---|---|---|
| `Tencent/WeKnora` | `647848f3954dae34473b8a8d0e0eef5e0fb3a58e` | Root `LICENSE`: MIT for the project with separately licensed third-party components | Sandbox boundaries, explicit tool approval, scoped durable memory, minimum tool surfaces, recall/tool observability | `STUDY` |
| `Tencent/RoMem` | `39ac1417b4db41ea729e5c3be71ac20de54da993` | No root `LICENSE` file present at the pinned repository state | Temporal contradiction and temporal-preservation research ideas | `STUDY_ONLY_NO_COPY` |
| `Tencent/LoopForge` | `09c765286f549624dd95434e1e6ef2249657cbeb` | Root `LICENSE`: MIT with third-party attribution for included prior art | Persisted staged workflow state, role separation, resume/audit artifacts | `STUDY` |
| `Tencent/SkillHone` | `7d565839fb4dc74f9c77f09ace660e1c0484e048` | Root `LICENSE`: MIT | Candidate/evaluator isolation, held-out split discipline, score provenance, failure-layer diagnosis, focused PR attribution | `STUDY` |

The exact source paths retained for each repository are recorded in `donors/tencent-agent-knowledge-sources-2026-09-08.yaml`.

## 1. WeKnora

### Relevant patterns

The pinned WeKnora source exposes a mature agent-runtime separation that is useful as comparative architecture material for commandF's optional future agent plane:

- session-persistent sandbox execution separated from the host application;
- per-configuration network policy rather than unconstrained tool egress;
- explicit pending human approval for selected tool operations;
- approval-required states treated as policy decisions rather than transient transport failures;
- principal/workspace-scoped cross-session memory;
- distinct memory search and recall-observability surfaces;
- curated tool exposure as a defense-in-depth control.

These patterns align with commandF's existing requirements for explicit guard stages, scoped capabilities, separate sandbox/approval policy, durable replayable facts, and reconstructable model-visible evidence.

### Non-adoptions

This study does **not** justify:

- adopting WeKnora's RAG/knowledge-base product surface;
- introducing its Go application stack into commandF;
- making long-term memory part of FHIR package/diff/check semantics;
- treating recalled or model-generated material as interoperability authority;
- exposing a host Docker socket or equivalent privileged runtime without a commandF-specific threat model and separate authorization.

## 2. RoMem

### Relevant research pattern

RoMem studies temporal memory using continuous-time ranking and a learned relation-dependent temporal gate. Its standalone reranker accepts timestamped triples and query time, then combines temporal and optional semantic scores.

The commandF research charter already treats temporal preservation as a first-class measurement. The useful connection is therefore **research methodology**, not agent memory infrastructure: future `commandF Bench` work may compare explicit temporal-preservation assertions or contradiction cases against multiple independent methods.

### Licensing and evidence boundary

No root `LICENSE` file was present at the pinned repository state. Consequently:

- no RoMem source code is qualified for copy, port, dependency, or vendoring;
- bundled checkpoints, model weights, datasets, and baseline implementations are not qualified for use or redistribution;
- repository or README publication claims are not commandF evidence;
- paper metadata, publication status, datasets, and experimental claims require independent qualification before any research protocol freeze.

## 3. LoopForge

### Relevant patterns

LoopForge persists workflow state and delivery artifacts across requirement, design, implementation, independent review, and testing stages. Its resume path reads durable workflow state and routes from the first unfinished stage rather than reconstructing progress from conversation memory.

This is useful for commandF's development and future agent-plane design because it reinforces several existing commandF principles:

- workflow state should be explicit rather than conversationally assumed;
- implementation and review roles should remain separable;
- interruption should not erase evidence or cause completed stages to be silently reinterpreted;
- handoff artifacts should preserve why work advanced or stopped.

### Non-adoptions

LoopForge does not replace GitHub Spec Kit, commandF's canonical Spec Kit packages, `AGENTS.md`, exact-head CI, external review, or GitHub repository truth. Host-specific agent bundles are not commandF execution authority.

## 4. SkillHone

### Relevant patterns

SkillHone's strongest commandF-relevant idea is the separation between the behavior being changed and the evaluation authority measuring it. Its evaluation workflow distinguishes:

- the public skill/candidate repository;
- the private evaluation repository and verifier contract;
- isolated solver workdirs;
- a redacted observation surface used for diagnosis.

It also distinguishes probe, PR-validation, and final test splits, requires score provenance to identify the exact run/split/output source, and prohibits exposing hidden gold/test material to the optimizer.

This strongly matches commandF's anti-self-forgery direction: the candidate under test must not be able to rewrite or learn hidden authority in order to make itself green. It also reinforces one focused change per PR and diagnosis of infrastructure failures separately from behavioral failures.

### AF-02 boundary

AF-02 is already planning-frozen. This study does not add new T022-T028 algorithms, policies, fixtures, workflow topology, schemas, tool identifiers, or evidence requirements. Any later use of SkillHone-derived patterns requires a separately authorized governance change or future unit.

## commandF-specific decisions

1. **No current core adoption.** The deterministic FHIR package, inspect, diff, check, graph, and impact path remains independent of these agent/memory/workflow runtimes.
2. **No current AF-02 contract change.** The study is comparative evidence only while AF-02 is executing its canonical plan.
3. **Future agent plane candidate patterns.** WeKnora's sandbox/approval/tool-surface ideas, LoopForge's resumable role-bounded state, and SkillHone's evaluator/candidate separation are retained for later planning.
4. **Research-only temporal candidate.** RoMem is retained only as a research reference for temporal-preservation/contradiction methodology until licensing and experimental evidence are separately qualified.
5. **Clean-room preference.** Where a pattern is eventually useful, commandF should prefer independently specified narrow contracts over copying large upstream runtime surfaces.

## Possible future activation points

These are candidate activation points, not authorization:

- a future commandF Copilot/agent-plane Spec Kit unit;
- future sandbox/tool-approval capability planning;
- agent or skill evaluation methodology with hidden-authority separation;
- `commandF Bench` temporal-preservation research protocols;
- resumable development-workflow evidence after the current Assurance Foundation sequence is closed.

## Evidence limitations

- This pass inspected pinned repository source and documentation; it did not execute the upstream systems.
- No upstream security audit or reproducibility replication was performed.
- RoMem licensing is unresolved at the pinned repository root.
- No paper result, benchmark score, model checkpoint, or dataset claim from these sources has been reproduced by commandF.
- A later pin change invalidates source-level conclusions until the changed paths and rights are re-reviewed.

## Current execution impact

None. PR #82 and AF-02 T022 remain the active execution frontier. This source study must not be merged ahead of the current exact-base T022 qualification if doing so would change the canonical base and invalidate its evidence.
