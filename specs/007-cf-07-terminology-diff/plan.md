# CF-07 Implementation Plan

Status: Implemented; convergence candidate

## Exact stack

```text
base branch: feat/cf-04-compatibility-rules
base SHA: ae33586a925023d92b4d58db01663bf26f3bd9a3
```

CF-07 stays inside `commandf-pkg` plus explicit CLI wiring in `commandf`; no new workspace crate is introduced.

## Architecture actually implemented

1. reuse CF-03 package/resource matching authority;
2. reuse CF-03 matched element/binding identity rather than introducing a second matcher;
3. preserve the complete CF-04 report unchanged;
4. load a deterministic verified terminology index from every package already present in each explicit CF-01 lock/cache state;
5. prove only finite locally closed CodeSystem/ValueSet membership;
6. represent unsupported but well-formed terminology semantics as `indeterminate`;
7. emit CF-07 binding refinements separately from CF-04 findings;
8. expose `commandf terminology` with explicit before/after lock/cache inputs and no acquisition.

## Implemented modules

```text
crates/commandf-pkg/src/terminology_model.rs
crates/commandf-pkg/src/terminology_error.rs
crates/commandf-pkg/src/terminology_index.rs
crates/commandf-pkg/src/terminology_set.rs
crates/commandf-pkg/src/terminology.rs
```

CF-03 helper surfaces were extended narrowly to expose matched resource pairs and matched element binding views.

## Public model

The implementation exposes schema-v1 deterministic terminology models including:

```text
TerminologyProofMode
TerminologyRelation
TerminologyMember
TerminologySetDelta
BindingRefinement
TerminologyDiffReport
TerminologyPackageState
```

Public report constants:

```text
schema = 1
ruleset = cf07-terminology-v1
```

Collections are stably ordered before JSON serialization.

## Verified lock-closure index

For each before/after state:

1. parse the explicit lockfile;
2. verify every lockfile cache object before consuming it;
3. read the content-addressed package archive;
4. validate `package/package.json` name/version against lock identity;
5. scan bounded package-root FHIR JSON resources;
6. index `ValueSet` and `CodeSystem` by canonical URL and optional business version;
7. reject duplicate exact canonical identities;
8. preserve all bare-URL matches so ambiguity is explicit.

Canonical resolution supports exact `url|version` and unique bare URL only. No “latest” behavior exists and no registry/source object is constructed on the terminology command path.

## Complete CodeSystem proof

Eligible proof requires complete non-compositional content, stable case-sensitivity semantics, valid unique codes, consistent optional count, and hard bounds. Code-set relation is `equal`, `narrowed`, `widened`, or `incomparable`; valid completeness cases outside that boundary are `indeterminate`.

## ValueSet expansion proof

Eligible proof requires a complete local expansion with zero/absent offset, consistent `total`, valid coded members, deterministic expansion-parameter context, and hard bounds.

Logical membership identity is `(system, version?, code)`.

Nested `expansion.contains` is flattened only for membership. Repeated logical members caused by hierarchy/display repetition are de-duplicated. Hierarchy is never used to infer inclusion. Entries without code are navigation nodes; coded entries require system. `abstract=true` coded members disable hard binding proof while retaining artifact evidence.

Compose-only/filter/import semantics without an eligible expansion, paging/incompleteness, and context mismatch remain `indeterminate` with no remote `$expand`.

## Root delta discovery

Direct CodeSystem/ValueSet deltas are emitted only for matched terminology resources in the requested root package and reuse CF-03 canonical matching.

Dependency terminology is available to binding resolution through the verified lock closure but is not mislabeled as a direct root-package artifact delta.

## Binding discovery and reconciliation

Binding discovery operates on matched StructureDefinitions/elements from CF-03 authority, not only on existing `CF04-BIND-005` findings. This is required to detect dependency terminology drift when the root profile and binding reference remain unchanged.

Therefore a case with:

```text
CF-03 changes = []
CF-04 findings = []
unchanged required binding reference
changed dependency ValueSet membership
```

can still produce a CF-07 refinement.

Identical unresolved references on both sides are suppressed as no-op self-diff evidence. Changed or asymmetrically resolved references remain `indeterminate`.

Hard refinement is authorized only when both strengths are exactly `required`, proof is finite and binding-eligible, and no simultaneous strength change makes the interaction ambiguous.

```text
CF07-BIND-001  narrowed      -> producer BREAKING
CF07-BIND-002  widened       -> consumer BREAKING
CF07-BIND-003  incomparable  -> producer BREAKING
CF07-BIND-004  incomparable  -> consumer BREAKING
```

Equal membership emits no SAFE claim. Extensible/preferred/example membership changes remain evidence-only. The embedded CF-04 report is not rewritten or deleted.

## CLI

```text
commandf terminology <package-name> \
  --before-lock <path> \
  --before-cache <path> \
  --after-lock <path> \
  --after-cache <path> \
  --format json
```

Execution order:

1. load explicit lockfiles and select requested root package;
2. verify/read root archives;
3. run CF-03 structural diff;
4. run CF-04 classification;
5. load verified terminology closures;
6. emit direct root terminology deltas;
7. reconcile matched bindings against before/after closures;
8. serialize deterministic JSON.

No acquisition is performed by this command.

## Bounds

The implementation uses hard limits for terminology resources, flattened members, expansion parameters, and public member deltas. Bounds failure is a hard error; output is never silently truncated while still claiming a set relation.

## Regression coverage

Implemented tests cover:

- complete CodeSystem equal/narrowed/widened/incomparable and completeness guards;
- ValueSet expansion relations, paging/total, parameter context, hierarchy de-duplication, navigation nodes, malformed coded members, and abstract-member eligibility;
- exact and bare canonical resolution, ambiguity, dependency lookup, and corrupted dependency cache;
- unchanged-reference dependency ValueSet drift with empty CF-03/CF-04 evidence;
- required narrowing/widening/incomparable producer/consumer mapping;
- non-required evidence-only behavior;
- equal/no-SAFE behavior;
- simultaneous strength interaction;
- unresolved behavior and self-equivalent unresolved suppression;
- deterministic JSON;
- CLI help, required args, offline self-equivalence, and corrupted-cache failure.

## Real integration gate

CI independently resolves and verifies `hl7.fhir.r4.core@4.0.1` into distinct before/after lock/cache states, then runs inspect, diff, classify, and terminology. Self-comparison must produce no structural, compatibility, terminology, or binding-refinement false positives.

## Reviewer truth

CodeRabbit review was requested on the green implementation candidate but was rate-limited and returned no substantive review threads. Qodo `/review` produced no substantive result. These are recorded as `NOT RETURNED`; no reviewer PASS is invented. Cubic summaries are informational only.

## Convergence rule

CF-07 converges only after the exact final documentation head passes Format, locked workspace Clippy with `-D warnings`, full workspace tests, and the real independently resolved R4 smoke; `spec.md`, `plan.md`, `tasks.md`, and `convergence.md` must match implementation truth; PR #8 must remain Draft/open/unmerged with auto-merge disabled; CF-08 must remain unstarted.
