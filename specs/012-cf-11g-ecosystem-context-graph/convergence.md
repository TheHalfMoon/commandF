# CF-11G Convergence Evidence — Ecosystem Context Graph

Status: CLOSED_CANONICAL — valid once this closeout PR passes its exact-head documentation-path gates and review, then merges without content changes.

Decision:

```text
CF11G_CLOSED_CANONICAL
CF12_ELIGIBLE_AFTER_CLOSEOUT_MERGE
```

CF-11G is the deterministic, offline ecosystem Context Graph slice. It records evidence only; it does not classify compatibility, safety, clinical meaning, or semantic equivalence, and it does not change CF-06 oracle identity, the frozen CF-10 corpus, or any HL7 production pin.

## Canonical implementation chain

```text
Planning PR #20
head:  190945b1b7665e8b01a2c3ce9f93cf2c5e23dd45
merge: ffbcdaea9bcbcb31271d38dea83419a014558904

Stack A / PR #21 — lock schema v2 + exact resolved dependency edges
head:  40983be3bbd6aed098c18cae8f381cab0ed1826e
merge: 4bc5df2cfcb9437e1d4d84b19bc7ddca556d6996

Stack B / PR #22 — deterministic Context Graph library
head:  1dce479f7e2b548109e8d2b99a928353639be4b6
merge: 8b93a04d573f182e902d21d42ad54180d36d5223

Stack C / PR #23 — shipped commandf context CLI
final reviewed head: 1475a9d117f11dd5af3de6b118cd72fc8ccdfebd
merge:               7579309d6298998a0bad47bac080be156b4d80df
canonical tree:      160df0b89642e2588fddc17b5daed517c3689857
```

No force-push or rebase was used to advance the stack.

## Final implementation-head regression evidence

Exact head `1475a9d117f11dd5af3de6b118cd72fc8ccdfebd` passed the complete required gate set:

```text
ci                         32844511038  SUCCESS
cf06-oracle                32844511085  SUCCESS
cf11-multi-version-proof   32844511014  SUCCESS
cf11g-context-proof        32844511055  SUCCESS
CodeRabbit status                       SUCCESS
```

The `ci` workflow covers formatting, Clippy with warnings denied, workspace tests, CF-08/CF-09 security regressions, real FHIR smoke, terminology smoke, and local GitHub Action smoke.

## Lock schema migration evidence

CF-11G makes schema v2 the output contract for new package resolutions while retaining valid schema-v1 read/verify compatibility for existing commands.

Schema v2 records deterministic exact parent→child dependency evidence including the original declared constraint. Shared children retain all parent edges; cycle-closing edges are retained while exact-identity expansion remains bounded. `commandf context` refuses schema v1 rather than reconstructing ambiguous multi-version edges.

Independent review found one evidence-loss edge case in the v1 serialization path: a programmatically constructed schema-v1 `Lockfile` with non-empty `resolved_dependencies` could have discarded that evidence. The write path was repaired to reject the state fail-closed, and regression `schema_v1_refuses_to_serialize_resolved_dependency_evidence` covers the invariant. CodeRabbit confirmed the repair and resolved the thread.

## Context Graph contract

The canonical graph implementation preserves:

- exact package identity `(name, version, sha256)` and package source provenance;
- exact resolved package dependency edges;
- artifact ownership, archive digest, source path, resource type, canonical URL/version, and resource SHA-256;
- bounded StructureDefinition, ValueSet, and CodeSystem V1 canonical-reference extraction;
- deterministic extraction-coverage metadata for present unsupported resource types;
- exactly the frozen V1 target states `resolved`, `external`, and `ambiguous`;
- exact source canonical strings even when an explicit version has no eligible in-closure match;
- cache-only/offline graph construction with verified archive bytes.

A CodeRabbit suggestion to add a fourth explicit-version-mismatch target status was not adopted because Spec 012 freezes zero exact eligible matches as `external` / unresolved-in-closure and Acceptance Criterion H freezes the three V1 states above. CodeRabbit rechecked the contract, agreed the original finding did not apply, and resolved the thread.

## Shipped CLI and deterministic proof

Canonical user-visible command:

```text
commandf context --lock commandf.lock --cache .commandf/cache --format json
```

It performs no registry acquisition or network canonical resolution and fails closed for unsupported lock schema, missing cache archives, corrupt archive digests, and malformed graph-required supported inputs.

Exact proof run `32844511055`, job `97791094780`, ran the CLI determinism proof in:

```text
Rust:      1.97.1
container: docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f
checkout:  actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09
```

Observed canonical output identity:

```text
CF11G_CONTEXT_SHA256=cbc08088a858ca12af0a2a773be5f4b02a03bc099442e59f7300f5edaca069c0
repeat byte equality: PASS
repository-clean assertion: PASS
```

Retained workflow artifact identity:

```text
artifact name: cf11g-context-proof
artifact id:   9561791650
artifact digest: sha256:f4134aef0acfd3dff671d6219ee3a2d80dd73a5dbb33312c2ff97a9135f5b421
workflow head: 1475a9d117f11dd5af3de6b118cd72fc8ccdfebd
```

Artifact expiration does not change the immutable GitHub-recorded run/head/digest identity.

## Independent-review truth

Every substantive finding actually returned for the implementation stack has a disposition:

- **PR #21:** schema-v1 serialization evidence loss — FIXED, regression added, CodeRabbit confirmed, thread resolved.
- **PR #22:** proposed fourth canonical target state — NOT ADOPTED because it contradicts frozen Spec 012 V1; CodeRabbit confirmed the contract and resolved the thread.
- **PR #23:** implicit container registry/namespace — FIXED by using the fully qualified digest-pinned image; CodeRabbit confirmed and resolved the thread.
- **PR #20:** CodeRabbit entered processing, then an explicit re-review attempt was rate-limited. No substantive planning finding was returned. This is recorded as reviewer unavailability, not PASS.
- **Qodo:** no connected/available review was observed for this stack; no Qodo PASS is claimed.

Reviewer absence is recorded rather than replaced with invented approval. There is no unresolved substantive returned finding.

## T040–T043 convergence disposition

```text
T040 mandatory workspace gates: COMPLETE
T041 preserved workflow gates:   COMPLETE
T042 independent review:         COMPLETE WITH AVAILABILITY TRUTH RECORDED
T043 convergence pass:           COMPLETE, subject to this closeout PR exact-head validation and merge
```

The final closeout PR MUST remain unmerged if any exact-head required workflow fails, the head changes without requalification, or a new substantive review finding remains unresolved.

## CF-12 boundary

After this closeout record itself passes its exact-head configured gates/review and merges without content changes, CF-11G is canonically closed and `CF-12 commandf impact` becomes eligible for implementation.

CF-12 must consume the explicit graph evidence and preserve `external` and `ambiguous` states. This closeout grants no new compatibility, clinical, model, dependency-source, network, or HL7-oracle authority.
