# commandF Agent Security and Control Gap Addendum — 2026-09-08

Status: **FUTURE_ARCHITECTURE_PLAN_COMPANION / NO_CURRENT_IMPLEMENTATION_AUTHORITY**

Companion to:

- `docs/COMMAND_F_AGENT_EVIDENCE_AND_SOURCE_ADOPTION_PLAN_2026-09-08.md`
- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`
- `docs/PROVENANCE_AND_DONOR_POLICY.md`

This addendum records a second-pass threat-model review performed after the Tencent source synthesis. The items below are commandF-specific design inferences from the combined architecture, existing repository governance, and retained source families. They are **not claims that any one donor implements these controls completely**.

The active AF-02 contracts and dependency order remain unchanged.

## Why this addendum exists

The first synthesis correctly separated evidence, sandbox/capabilities, orchestration, memory, evaluation, and model experience. A second threat-model pass found cross-layer failure modes that do not fit cleanly inside one of those six capability descriptions.

These controls are required before a future commandF agent plane can be considered safe for high-authority or externally visible actions.

## G-A15 — untrusted-context and prompt-injection provenance

### Gap

Tool output, retrieved documents, memory records, web content, repository text, issue comments, and model-generated text can contain instructions. A model may confuse untrusted data with commandF policy or execution authority.

### Plan correction

Future model-context projection must carry trust/provenance labels and preserve instruction/data separation.

Required properties:

- every context fragment has an origin category and evidence reference;
- untrusted retrieved content cannot create or enlarge capabilities;
- tool output and external text are data unless an independently authorized policy explicitly interprets them otherwise;
- prompt/model text cannot override repository governance, tool policy, approval policy, evaluator authority, or hidden-test isolation;
- context assembly records which fragments were system/policy instructions versus user input versus untrusted retrieved data;
- injection-resistant tests include malicious repository text, issue comments, tool output, memory content, and retrieved documents;
- a context fragment's trust class is never upgraded solely because a model repeated or summarized it.

## G-A16 — approval binding and TOCTOU

### Gap

A generic approval such as "allow this tool" can be reused after arguments, target, repository head, payload, or side effect has changed.

### Plan correction

Approval must bind the exact proposed action.

Required approval identity:

- actor/principal;
- task/run identity;
- capability/tool identity and version;
- canonical argument digest;
- target/resource identity;
- repository/base/head identity when code or repository state is involved;
- requested side-effect class;
- policy version;
- approval timestamp and expiration;
- optional bounded repetition count where explicitly permitted.

Any material action change invalidates approval and returns to `approval_required`.

Approval records must distinguish previewed intent from executed effect.

## G-A17 — replay, idempotency, and side-effect reconciliation

### Gap

Resumable workflows can accidentally repeat emails, deployments, mutations, payments, issue edits, merges, deletes, or other externally visible actions after interruption or retry.

### Plan correction

Every mutating capability must declare replay semantics before activation.

Required classes:

- read-only/replay-safe;
- idempotent with stable idempotency key;
- conditionally idempotent with precondition/expected-state binding;
- non-idempotent and approval-required per execution;
- compensatable with an explicit compensation action;
- irreversible/high-risk and not eligible for autonomous replay.

The evidence substrate records intent, idempotency key, precondition, provider response, observed resulting state, and reconciliation outcome.

A workflow may mark a mutating stage complete only after the externally observed state is reconciled with the intended effect.

## G-A18 — delegated authority and capability attenuation

### Gap

A parent agent or workflow can accidentally create a child/subagent with more authority than the parent, or a child can infer permissions from role names rather than explicit grants.

### Plan correction

Delegation is capability attenuation, never capability amplification.

Required rules:

- child capability set must be a subset of the delegating principal's effective grant;
- delegation records parent, child, task, scope, expiry, and grant digest;
- no implicit inheritance of secrets, network access, filesystem scope, approval authority, or protected evaluator access;
- reviewer/evaluator roles cannot inherit candidate mutation authority merely because they run in the same workflow;
- nested delegation depth and fan-out are resource-bounded;
- revocation propagates to active descendants or blocks their next privileged operation.

## G-A19 — multi-tenant, workspace, and principal isolation

### Gap

Scoped memory and session sandboxes are insufficient if caches, artifact stores, logs, vector indexes, task state, or provider handles can cross tenant/workspace/principal boundaries.

### Plan correction

Isolation must be end-to-end across all stateful layers.

Required controls:

- explicit tenant/workspace/principal namespace on durable state;
- authorization checked on both write and read/replay paths;
- provider handles and sandbox bindings are scoped and non-enumerable across tenants;
- cache keys include the security boundary, not only content identity;
- memory/vector retrieval cannot search another tenant/workspace unless a separately authorized sharing contract exists;
- exported evidence is filtered/redacted according to the destination principal;
- isolation tests include confused-deputy, cache-key collision, stale-handle reuse, and cross-workspace resume cases.

## G-A20 — tamper-evident evidence and audit integrity

### Gap

Append-only events are useful but an attacker or buggy component could alter, reorder, truncate, or replace retained evidence unless integrity is independently verifiable.

### Plan correction

The future evidence substrate must be tamper-evident where evidence is used for qualification, audit, or high-authority decisions.

Candidate controls to evaluate:

- canonical event serialization;
- per-record content digest;
- ordered sequence/fencing identity;
- hash chaining or Merkle-style batch commitments where justified;
- signed attestations for release/qualification evidence where existing commandF supply-chain tooling can be reused;
- immutable/content-addressed artifact references;
- explicit gap/truncation detection;
- retention of verifier/workflow/tool identity required to reproduce the commitment.

Cryptographic integrity is evidence integrity, not semantic correctness.

## G-A21 — skill/plugin/tool supply-chain authority

### Gap

A future agent plane can be secure at runtime yet become unsafe through mutable skills, plugins, tool schemas, model adapters, container images, or remote tool providers.

### Plan correction

Treat executable agent extensions as supply-chain inputs.

Required properties:

- immutable version/digest pin for executable extensions in authoritative runs;
- provenance/license/SBOM/security review appropriate to the artifact;
- declared capability manifest before execution;
- no silent capability expansion on update;
- update diff/requalification before a changed extension is trusted;
- model-visible tool description is bound to the same tool implementation/schema identity used for execution;
- container/template/runtime image identity is retained in evidence;
- remote provider trust and outage/fallback policy are explicit.

This should reuse AF-01/AF-03 supply-chain machinery rather than create a parallel trust system.

## G-A22 — retention, deletion, export, and privacy lifecycle

### Gap

"Model-visible means logged" conflicts with privacy if logs retain every prompt, attachment, memory item, secret-like string, or sensitive clinical/business context indefinitely.

### Plan correction

Evidence completeness and data minimization must be designed together.

Required lifecycle fields/policies:

- data classification;
- purpose;
- retention class/expiry;
- redaction/transformation applied;
- source deletion relationship;
- export eligibility;
- legal/policy hold where applicable;
- cryptographic or provider-side deletion limits where exact deletion cannot be proven.

Model-context reconstruction should normally rely on content-addressed/redacted evidence references rather than duplicating sensitive payloads across logs.

Deletion of contextual memory and deletion of compliance/audit evidence are separate policies and must not be conflated.

## G-A23 — concurrency, leases, and fencing

### Gap

Two workers can resume the same task, execute the same mutation, race an approval, overwrite workflow state, or create conflicting evidence.

### Plan correction

Workflow and mutating capability state require concurrency semantics.

Required controls:

- optimistic expected-version or lease/fencing token on state transition;
- monotonic event sequence per task/run;
- compare-and-swap or equivalent protection for canonical workflow-state updates;
- one active mutating executor per protected stage unless the stage is explicitly parallel-safe;
- approval consumption is atomic for single-use approvals;
- stale workers cannot commit completion after losing their lease/fence;
- parallel read-only work may fan out but converges through deterministic evidence references.

## G-A24 — model/provider identity, routing, and fallback semantics

### Gap

A routing layer may silently switch model, provider, prompt template, tool availability, sampling policy, or context window after an error. The resulting output can no longer be compared or replayed as though it came from the original configuration.

### Plan correction

Model execution identity is part of evidence, and fallback is an explicit state transition.

Required evidence:

- provider and model identifier;
- model/version snapshot or provider revision when available;
- adapter implementation/version;
- prompt/policy/template digest;
- tool-set/schema digest;
- sampling/configuration parameters that materially affect behavior;
- context projection digest;
- routing decision and fallback reason;
- retry attempt identity;
- response/tool-call identity.

A fallback result may be useful, but it cannot inherit the qualification identity of a different model/provider run.

## G-A25 — kill switch, circuit breaker, and bounded autonomy

### Gap

Resource budgets limit one call, but a faulty workflow can still repeatedly call a failing provider, generate repeated proposals, or create a long chain of low-risk actions that becomes high-risk in aggregate.

### Plan correction

Future orchestration requires task- and principal-level autonomy budgets and circuit breakers.

Candidate controls:

- maximum privileged operations per task/run;
- maximum delegated-agent fan-out/depth;
- cumulative wall-time/cost/token/tool budgets;
- repeated-failure circuit breaker by provider/tool/failure class;
- manual/global kill switch that blocks new privileged operations while preserving evidence;
- escalation to explicit human review after bounded retries or repeated policy denials;
- no hidden "try a more powerful tool/model" fallback after policy failure.

## G-A26 — human-approval fatigue and semantic preview

### Gap

A system can technically require approval while presenting opaque requests so frequently that users approve without understanding the effect.

### Plan correction

Approval UX is part of the security contract.

Required preview properties:

- human-readable action summary;
- exact target;
- material arguments and redacted secret handling;
- expected side effects;
- irreversible/destructive classification;
- diff/preview when a deterministic preview is possible;
- reason the action is required;
- what changes invalidate the approval;
- clear distinction between approving a proposal and approving execution.

Batch approvals are allowed only for an explicitly bounded homogeneous action set with a stable policy and argument envelope.

## Cross-layer invariants

The future architecture should preserve these invariants regardless of implementation donor:

1. **Untrusted data cannot grant authority.**
2. **A child cannot have more authority than its delegator.**
3. **Approval binds exact intent; execution proves exact effect.**
4. **Retries do not duplicate side effects.**
5. **Repository/source changes invalidate source-bound qualification evidence.**
6. **Hidden evaluation authority cannot leak through memory, logs, tools, or descendants.**
7. **Tenant/workspace boundaries apply to every cache, store, sandbox, log, and retrieval path.**
8. **Model/provider fallback is observable and changes execution identity.**
9. **Audit integrity does not imply semantic correctness.**
10. **Deleting memory is not the same as deleting required audit evidence.**
11. **No agent extension gains authority merely because it is installed.**
12. **Autonomy is bounded cumulatively, not only per tool call.**

## Activation impact

These gaps refine candidate exit criteria for future stages in `docs/COMMAND_F_AGENT_EVIDENCE_AND_SOURCE_ADOPTION_PLAN_2026-09-08.md`:

- Stage 1 evidence planning must cover trust labels, tamper evidence, retention, model identity, and concurrency identity.
- Stage 2 capability/sandbox planning must cover exact-action approval binding, delegation attenuation, extension supply chain, tenant isolation, idempotency, and circuit breakers.
- Stage 3 resumable workflow planning must cover leases/fencing, replay semantics, effect reconciliation, and source-change invalidation.
- Stage 4 bounded-memory planning must cover untrusted-memory injection tests, privacy lifecycle, tenant isolation, and hidden-evaluation leakage prevention.
- Stage 5 protected evaluation must prove final-test material cannot leak through memory, shared workspaces, logs, caches, or delegated agents.
- Stage 6 Copilot activation must prove model/provider routing and fallback identities are reconstructable and that high-authority actions present a semantically meaningful preview.

No current implementation unit is created or modified by this addendum.
