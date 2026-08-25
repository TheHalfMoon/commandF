# CF-11G Convergence Evidence — Ecosystem Context Graph

Status: convergence candidate; final exact-head regression and independent-review gate still open

Decision: `CF11G_IMPLEMENTATION_PROVEN_PENDING_FINAL_CONVERGENCE_GATES`

This document records the evidence accumulated for CF-11G before the final convergence-head qualification. It does not make the branch canonical, does not authorize merge by itself, and does not authorize CF-12 implementation.

## 1. Authority boundary

CF-11G remains an evidence-only, offline dependency graph slice.

It does not:

- classify compatibility, safety, clinical meaning, or semantic equivalence;
- perform package acquisition or network canonical resolution during `commandf context`;
- change CF-03/04/05 compatibility semantics;
- change the CF-06 production HL7 pin or oracle failure behavior;
- modify the frozen CF-10 corpus;
- introduce a graph database, vector store, RAG system, model authority, or agent authority;
- start CF-12 `commandf impact`.

CF-12 remains blocked until T043 is closed on a final exact head.

## 2. Canonical base and stacked implementation identity

Canonical `main` read during convergence preparation:

```text
main commit = 5bafce4f63537e0507e9b0708e1ebd8e22e3c463
main tree   = f0f009fe8abcab99b3a1d339d400b63af4a7b486
```

Stack snapshot before this convergence-document commit:

| Stack | PR | Head | Tree | Purpose |
|---|---:|---|---|---|
| Planning | #20 | `190945b1b7665e8b01a2c3ce9f93cf2c5e23dd45` | `f9e274e743964ca4ad0b92e622fce6c901078fb2` | restore CF-11G prerequisite and freeze Spec 012 |
| Stack A | #21 | `40983be3bbd6aed098c18cae8f381cab0ed1826e` | `b3d9cac69f74a6f41ed77250aad3006aab98b035` | lock schema v2 and exact resolved dependency edges |
| Stack B | #22 | `1dce479f7e2b548109e8d2b99a928353639be4b6` | `67fb1cb71e9259d34ceaa989c798f9c239560f3f` | deterministic Context Graph library |
| Stack C | #23 | `fc1fd27e17720d449e7d2358acb620be7e301d47` | `1bb0198f32ecff9d404d8536a2e595e3e596f76a` | shipped offline `commandf context` and deterministic proof |

The stacks were advanced without force-push or rebase. A reviewer-discovered fail-closed repair was propagated to the downstream stack so the current library and CLI heads contain the same invariant.

## 3. Lock schema migration evidence

Stack A establishes schema v2 as the resolver output contract while retaining existing schema-v1 read compatibility for existing commands.

Proven properties include:

- schema v2 carries exact parent identity, child identity, and declared dependency constraint evidence;
- exact dependency edges are sorted and deduplicated deterministically;
- shared child identities retain multiple parent edges;
- cycle-closing edges are retained while exact-identity expansion remains bounded;
- malformed or unsupported lock schema states fail closed;
- schema-v1 locks remain readable by existing commands;
- `commandf context` rejects schema v1 because exact resolved-edge evidence is unavailable;
- schema-v1 serialization now fails closed if a caller constructs a v1 `Lockfile` with non-empty `resolved_dependencies`, preventing silent evidence loss.

The final point was added after independent review. Regression coverage is named:

```text
schema_v1_refuses_to_serialize_resolved_dependency_evidence
```

## 4. Context Graph evidence

Stack B implements deterministic graph evidence through the existing bounded package-inspection and package-cache trust boundaries.

The report contains deterministic evidence for:

- exact package nodes and exact resolved package dependency edges;
- artifact nodes tied to exact owning package identity, archive digest, filename, resource type, canonical URL/version when present, and resource SHA-256;
- StructureDefinition `baseDefinition`, profile, targetProfile, and binding ValueSet references;
- ValueSet include/exclude system and imported ValueSet references;
- CodeSystem `supplements` references;
- explicit extraction coverage for supported and present-but-unsupported resource types;
- canonical target states frozen by Spec 012: `resolved`, `external`, and `ambiguous`.

For canonical matching, `url|version` uses exact in-closure URL + version eligibility. Zero eligible matches are intentionally `external` / unresolved-in-closure under Spec 012 §6; no fourth version-mismatch state is introduced.

## 5. Shipped CLI evidence

Stack C ships:

```text
commandf context --lock commandf.lock --cache .commandf/cache --format json
```

The command:

- reads explicit lock/cache inputs;
- builds the library-owned deterministic Context Graph;
- writes canonical JSON to stdout;
- performs no package acquisition or registry lookup;
- fails closed for schema-v1 context requests, missing archives, corrupt archive digests, and malformed graph-required supported inputs.

End-to-end fixtures cover multi-version package edges, profile and Extension references, terminology references, all three frozen target states, and unsupported resource coverage.

## 6. Determinism proof

Exact Stack C pre-convergence head `fc1fd27e17720d449e7d2358acb620be7e301d47` passed `cf11g-context-proof` run `32843775848`.

The proof environment used:

```text
Rust = 1.97.1
container = docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f
checkout = actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09
```

The proof ran `commandf context` twice from identical pinned fixture inputs and compared stdout bytes exactly.

Observed output identity:

```text
CF11G_CONTEXT_SHA256=cbc08088a858ca12af0a2a773be5f4b02a03bc099442e59f7300f5edaca069c0
repeat equality = PASS
repository-clean assertion = PASS
```

The proof also uploaded the retained checksum evidence artifact successfully.

## 7. Pre-convergence regression evidence

These are the exact-head runs that were green immediately before this convergence-document commit. Because this document changes the candidate head, T040/T041 remain open until the final convergence head reruns the applicable gates.

### Planning head — PR #20

```text
head = 190945b1b7665e8b01a2c3ce9f93cf2c5e23dd45
ci = 32832888034 / SUCCESS
cf06-oracle = 32832888010 / SUCCESS
```

### Stack A — PR #21

```text
head = 40983be3bbd6aed098c18cae8f381cab0ed1826e
ci = 32843594591 / SUCCESS
cf06-oracle = 32843594634 / SUCCESS
cf11-multi-version-proof = 32843594609 / SUCCESS
```

The `ci` job includes successful format, clippy, workspace tests, CF-08/CF-09 security regressions, real FHIR smoke, terminology smoke, and local GitHub Action smoke.

### Stack B — PR #22

```text
head = 1dce479f7e2b548109e8d2b99a928353639be4b6
ci = 32843690946 / SUCCESS
cf06-oracle = 32843690704 / SUCCESS
cf11-multi-version-proof = 32843690836 / SUCCESS
```

### Stack C — PR #23

```text
head = fc1fd27e17720d449e7d2358acb620be7e301d47
ci = 32843775881 / SUCCESS
cf06-oracle = 32843775904 / SUCCESS
cf11-multi-version-proof = 32843775886 / SUCCESS
cf11g-context-proof = 32843775848 / SUCCESS
```

## 8. Independent-review truth

Independent review has produced substantive findings, and each known substantive finding has an explicit disposition.

### PR #21 — schema-v1 serialization evidence loss

Finding: a public schema-v1 `Lockfile` state with populated `resolved_dependencies` could serialize through the v1 shape and silently drop edge evidence.

Disposition: **FIXED**.

The write path now rejects the state with the same fail-closed diagnostic used by the read path, and a regression test covers it. CodeRabbit confirmed the repair in-thread and the thread is resolved. The same invariant was propagated to #22 and #23.

### PR #22 — proposed fourth canonical-resolution state

Finding: distinguish explicit-version mismatch from `External` with a fourth target status.

Disposition: **NOT ADOPTED — conflicts with frozen Spec 012 V1 contract**.

Spec 012 §6 defines exact `url|version` eligibility and zero matches as `external` / unresolved-in-closure. Acceptance criterion H freezes the three V1 states `resolved`, `external`, and `ambiguous`. The exact source canonical string retains the explicit version evidence. The review thread is resolved with this rationale.

### PR #23 — proof-container provenance

Finding: the job container used an implicit Docker Hub registry/namespace even though its digest was pinned.

Disposition: **FIXED**.

The workflow now uses the fully qualified digest reference:

```text
docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f
```

CodeRabbit confirmed the repair and resolved the thread.

### Review availability that remains open

T042 is not yet closed in this convergence candidate.

- PR #20 CodeRabbit review was triggered for the exact planning range and remained in processing at the last live read.
- Incremental review attempts after the propagated #21 repair on #22 and #23 hit CodeRabbit's included-review rate limit. These attempts must not be represented as exact-head independent-review PASS.
- Qodo was not observed as connected/available on these PRs during this convergence pass and no Qodo PASS is claimed.
- CodeRabbit's docstring-coverage notices are reviewer advisories, not repository acceptance gates for this slice; no behavior is weakened to satisfy them.

## 9. Remaining gaps / explicit deferrals

### Blocking convergence gap — T042-R1

Complete or explicitly disposition the still-running PR #20 independent review, and obtain the final available independent review coverage required by T042. Any new substantive finding reopens the affected implementation task and must be repaired or rejected against the frozen contract with evidence.

### Final-head proof gap — T043-R1

After the convergence documentation/task-state commit, rerun and inspect on the new exact Stack C head:

```text
ci
cf06-oracle
cf11-multi-version-proof
cf11g-context-proof
```

Record the final exact head/tree/run identities in PR metadata after the runs settle. A failing or missing required gate prevents T043 closure.

### Reviewer availability deferral — T042-D1

If CodeRabbit remains rate-limited for an incremental re-review, record the limitation exactly rather than converting it into PASS. Previously completed findings and their dispositions remain evidence, but reviewer unavailability is not itself a positive review result.

Qodo remains an explicit availability-based deferral unless a connected review becomes observable before closure.

## 10. CF-12 eligibility

Current decision:

```text
CF-12 = BLOCKED_PENDING_T042_T043
```

CF-12 `commandf impact` MUST NOT begin implementation until:

1. T042 has a defensible independent-review disposition with every substantive finding closed;
2. the final convergence head passes the required exact-head workflows;
3. T043 is marked complete without unresolved CF-11G implementation gaps;
4. the stacked CF-11G changes are merged through the repository's normal PR lifecycle.

No statement in this document grants compatibility, safety, model, runtime, source, dependency, or clinical authority beyond the deterministic evidence recorded above.
