# CF-07 Convergence Record

Status: Candidate — exact final-head CI required before founder-review certification

## Exact stack identity

```text
repository: TheHalfMoon/commandF
PR: #8
branch: feat/cf-07-terminology-diff
base branch: feat/cf-04-compatibility-rules
base SHA: ae33586a925023d92b4d58db01663bf26f3bd9a3
```

CF-05 and CF-06 are not dependencies. CF-08 is not authorized by this convergence record.

## Implemented product boundary

CF-07 provides deterministic closed-world terminology change evidence above CF-04:

- complete finite CodeSystem code-set comparison;
- complete compatible local ValueSet expansion comparison;
- verified terminology canonical resolution across the complete explicit CF-01 lock closure;
- direct root terminology deltas through CF-03 resource matching authority;
- binding discovery through CF-03 matched StructureDefinition/element identity authority;
- dependency ValueSet drift detection even when the root profile/binding is unchanged and CF-03/CF-04 produce no finding;
- required-binding hard refinements `CF07-BIND-001..004`;
- evidence-only behavior for non-required strengths;
- equal membership without implicit SAFE;
- explicit indeterminate behavior for unsupported semantic cases;
- `commandf terminology` with explicit two-state lock/cache inputs and no acquisition.

No terminology server, `$expand`, general compose/filter solver, proprietary terminology corpus, graph blast radius, mapping execution, or AI/agent authority is added.

## Membership and FHIR handling truth

ValueSet expansion logical membership identity is `(system, version?, code)`. Nested hierarchy is flattened only for membership collection and never used for subset inference. Repeated logical members caused by hierarchy/display repetition are de-duplicated. Entries without code are navigation/grouping nodes. Coded entries require a system. Abstract coded members disable hard binding proof while retaining artifact evidence.

CodeSystem proof is limited to complete, non-compositional, count-consistent sets with stable case-sensitivity semantics and valid unique concept codes.

Paging/incompleteness, expansion-context mismatch, compose-only terminology, non-complete CodeSystems, unresolved references, and other well-formed semantics outside the proof boundary remain deterministic `indeterminate` rather than guessed compatibility.

## Closure and determinism truth

Every consumed cache object in each explicit lockfile is digest-verified before terminology indexing. Package manifest name/version is checked against lock identity. Canonical resolution supports exact `url|version` or unique bare URL only; ambiguity and duplicate exact identities fail closed. No implicit latest selection or network acquisition exists on the terminology command path.

Public output is deterministically sorted, bounded, and serialized without clocks, random identifiers, host paths, or network-derived state.

## Binding refinement truth

The complete CF-04 compatibility report is embedded unchanged.

Hard refinements are authorized only when both strengths are exactly `required`, membership proof is finite and eligible, and no simultaneous strength interaction makes the result ambiguous.

```text
CF07-BIND-001 narrowed      -> producer BREAKING
CF07-BIND-002 widened       -> consumer BREAKING
CF07-BIND-003 incomparable  -> producer BREAKING
CF07-BIND-004 incomparable  -> consumer BREAKING
```

Identical unresolved references on both sides are suppressed as no-op self-diff noise. Real changed/unbalanced unresolved interactions remain indeterminate evidence.

## Regression evidence

Synthetic and CLI tests cover:

- CodeSystem equal/narrowed/widened/incomparable and fail-closed completeness boundaries;
- ValueSet expansion relation math, hierarchy de-duplication, paging/total, context mismatch, malformed members, and abstract-member eligibility;
- exact/bare canonical resolution, ambiguity, duplicate identity, root/dependency lookup, and corrupted dependency cache;
- unchanged-reference dependency ValueSet narrowing with `CF-03 changes=[]` and `CF-04 findings=[]` yielding `CF07-BIND-001`;
- required widening and incomparable directions;
- non-required evidence-only behavior;
- equal/no-SAFE behavior;
- strength interaction;
- unresolved and self-equivalent unresolved behavior;
- deterministic JSON;
- `commandf terminology` help, usage error, offline self-equivalence, and corruption failure.

## Green implementation candidate

The exact implementation candidate below passed all repository gates before this documentation reconciliation:

```text
head: 06894d853a0ed9abdb73a622a9bb0a0d818ac3d4
Actions run: 31823710644
Format: PASS
Clippy --locked --workspace --all-targets --all-features -- -D warnings: PASS
Test --locked --workspace --all-features: PASS
Real independently resolved/verified hl7.fhir.r4.core@4.0.1 inspect + self-diff + self-classify + self-terminology smoke: PASS
```

This evidence proves implementation behavior but does not substitute for exact final documentation-head CI.

## Reviewer truth

CodeRabbit substantive review was requested twice on the green implementation candidate and both attempts were rate-limited. No substantive inline review thread was returned. A status context may report success, but it is not treated as a substantive review PASS.

```text
CodeRabbit substantive review: NOT RETURNED / RATE LIMITED
unresolved inline threads observed: 0
CodeRabbit PASS claimed: NO
```

Qodo `/review` was requested and no substantive result was observed.

```text
Qodo result: NOT RETURNED
Qodo PASS claimed: NO
```

Cubic-generated summaries are informational and not certification.

## Final certification gate

CF-07 may be reported as `CF-07_COMPLETE_READY_FOR_FOUNDER_REVIEW` only when the exact PR head containing this record proves:

1. Format PASS;
2. locked full-workspace Clippy with `-D warnings` PASS;
3. locked full-workspace tests PASS;
4. real independently resolved/verified R4 self-terminology smoke PASS;
5. PR #8 remains open, Draft, unmerged;
6. auto-merge remains disabled;
7. base SHA remains `ae33586a925023d92b4d58db01663bf26f3bd9a3`;
8. no unresolved actionable review threads exist;
9. CodeRabbit/Qodo truth is not overstated;
10. no CF-08 branch/PR has started.

Until all ten are revalidated on the exact final head, the correct state is `CF-07_IMPLEMENTATION_COMPLETE_FINAL_CERTIFICATION_PENDING`.
