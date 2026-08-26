# CF-12 Consistency Analysis — Deterministic Impact Analysis

Status: CONSISTENT / PLANNING_REVIEW_CLOSED — final PR-head requalification still required before merge.

## Inputs checked

This analysis reconciles `AGENTS.md`, the commandF constitution, Master Architecture V2, canonical CF-11G specification/convergence, CF-12 `spec.md`, `plan.md`, `tasks.md`, and current `diff` / `classify` / `check` / `context` CLI conventions.

## Result

No blocking internal contradiction is known.

The first complete planning head `1bee6f3651fa686f03902f3d86761736d4844513` passed:

```text
ci            32928525763 SUCCESS
cf06-oracle   32928525784 SUCCESS
CodeRabbit                 SUCCESS / no review thread returned
Qodo                       unavailable/not observed; no PASS claimed
```

Updating T004 and this close record moves the PR head, so implementation MUST remain blocked until the final planning head repeats applicable CI/review and the planning PR merges.

## Consistency checks

### Roadmap and vertical slice

`CF-12 = commandf impact` depends on canonical CF-11G. The plan ends in a shipped CLI plus deterministic proof, not a scaffold. CF-06/CF-10 upstream governance is not imported as an independent graph-plane dependency.

Result: CONSISTENT.

### CLI shape and evidence identity

CF-12 reuses the existing selected-package + explicit before/after lock/cache convention. It introduces no branch, mutable registry, or implicit network evidence.

Result: CONSISTENT.

### Determinism

The specification requires byte-identical JSON for identical pinned inputs. Canonical sorting, shortest-path normalization, lexicographic equal-length tie-breaking, and exact-identity visited state make traversal/reporting independent of hash/traversal order.

Result: CONSISTENT.

### Fail-closed graph semantics

CF-11G `resolved` edges alone are traversable. `external` and `ambiguous` remain explicit unresolved boundaries; no preferred candidate or network completion is allowed.

Result: CONSISTENT.

### Compatibility authority

Impact is reachability/exposure evidence, not BREAKING/RISKY/ADDITIVE severity. CF-12 does not recreate CF-03/04/05 authority or infer runtime/clinical breakage from graph reachability.

Result: CONSISTENT.

### Multi-version exactness

Package traversal consumes schema-v2 exact parent/child identities and never collapses same-name concrete versions.

Result: CONSISTENT.

### Side-aware change evidence

Before and after graphs are analyzed independently so removed-before and added-after dependency evidence cannot disappear. `both` is only a normalization for exactly identical evidence.

Result: CONSISTENT.

### Path reporting

One canonical shortest path per exact `(impacted, seed, side)` relation gives deterministic actionable evidence. Equal-length ties use stable lexicographic identity. Unresolved boundaries remain separately retained.

Result: CONSISTENT.

### Coverage and trust boundaries

CF-12 carries CF-11G extraction coverage forward and does not claim exhaustive artifact impact beyond supported relations. Existing bounded archive inspection and verified cache reads remain authoritative; no second archive reader, PHI path, graph database, model, or network resolver is planned.

Result: CONSISTENT.

## Reviewer-risk areas retained for implementation review

Implementation review must challenge:

1. structural-diff seed identity for add/remove/modify cases;
2. separation of package exposure from artifact exposure;
3. shortest-path evidence sufficiency;
4. unresolved-boundary collection semantics;
5. before/after normalization and provenance retention;
6. accidental presentation of reachability as compatibility severity.

A substantive implementation finding reopens the corresponding task; this planning close does not waive future findings.

## Explicit V1 deferrals

SQL-on-FHIR, CQL, SearchParameter expressions, FHIRPath invariants, persistent graph storage, graph databases, network canonical completion, AI/model impact claims, clinical/runtime breakage claims, CF-06 production-pin changes, and frozen CF-10 corpus changes remain outside CF-12 V1.

## Final planning merge rule

The planning package is eligible to merge only if the final exact PR head passes all applicable configured gates, CodeRabbit/reviewer truth remains free of unresolved substantive findings, and no content mutation occurs after that qualification.

Only after that merge may T010 implementation begin.
