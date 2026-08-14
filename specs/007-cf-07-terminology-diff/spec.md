# CF-07 — Closed-World Terminology Diff

Status: Implemented; convergence candidate

## Purpose

CF-07 adds deterministic, closed-world `CodeSystem`, `ValueSet`, and binding-membership evidence above CF-04. It does not build a terminology server and does not infer arbitrary FHIR terminology semantics.

## Exact stack

```text
base branch: feat/cf-04-compatibility-rules
base SHA: ae33586a925023d92b4d58db01663bf26f3bd9a3
```

CF-05 and CF-06 are not dependencies. CF-08 is out of scope.

## Direction authority

CF-07 preserves CF-04 vocabulary:

- producer — before-valid output remains valid under after;
- consumer — a consumer prepared for all before-valid output can consume all after-valid output.

For a proven finite allowed-membership relation:

- strict subset after -> `narrowed` -> producer impact;
- strict superset after -> `widened` -> consumer impact;
- add+remove -> `incomparable` -> both directions may break;
- same members -> `equal`;
- no complete proof -> `indeterminate`.

## Proof boundary

### Complete CodeSystem

Finite code-set proof is allowed only when both resources are `CodeSystem`, `content == complete`, `compositional` is absent/false, canonical URL is present, concept codes are well formed and globally unique, optional `count` is consistent, and `caseSensitive` semantics do not change.

Other valid FHIR completeness modes remain `indeterminate`; malformed interpreted fields fail closed.

### ValueSet expansion

Finite ValueSet proof uses local complete `expansion` evidence only. Membership identity is exact:

```text
(system, version?, code)
```

Requirements:

- `offset` absent or zero;
- `total` present and equal to the number of unique logical coded members;
- coded members have non-empty `system` and `code`;
- expansion parameters normalize deterministically and match across sides;
- unsupported parameter shapes do not produce a complete proof.

Nested `expansion.contains` is flattened for logical membership only. Hierarchical display/navigation may repeat the same logical `(system, version?, code)` member; CF-07 de-duplicates those repetitions rather than treating them as malformed duplicate evidence. Hierarchy has no subset meaning.

Entries without `code` are navigation/grouping entries. A coded entry without `system` fails closed. A coded `abstract=true` member may contribute artifact membership evidence but disables hard binding refinement in v1.

Compose-only, filtered/imported definitions without eligible complete local expansion, incomplete/paged expansion, and expansion-context mismatch remain `indeterminate`. CF-07 never calls `$expand`.

## Canonical resolution

`commandf terminology` operates only on explicit verified CF-01 lock/cache states. Every package object consumed from the lock closure is digest-verified before parsing and package manifest identity is checked.

Terminology resources are indexed across the complete explicit lock closure.

Resolution rules:

- `url|version` -> exact canonical/version match;
- bare `url` -> exactly one matching resource across the state;
- no match -> unresolved evidence where the report contract allows it;
- multiple bare matches -> ambiguous canonical and fail closed for attempted hard proof;
- duplicate exact canonical identities -> fail closed;
- never choose “latest”.

No registry, terminology service, or user-level implicit package cache is consulted by `commandf terminology`.

## Package and binding discovery

Direct root-package `CodeSystem`/`ValueSet` deltas reuse CF-03 matched-resource authority.

Binding discovery also reuses CF-03 matched `StructureDefinition`/element identity authority. It is not limited to an existing CF-04 binding finding. Therefore CF-07 can detect terminology drift in a dependency even when:

```text
root StructureDefinition unchanged
binding reference unchanged
CF-03 changes = []
CF-04 findings = []
```

If the referenced dependency ValueSet membership changes, CF-07 can still emit the appropriate membership refinement.

Identical unresolved binding references on both sides are suppressed as self-diff noise. Changed, added/removed, or asymmetrically resolved references remain explicit `indeterminate` evidence.

## Binding refinement contract

The embedded CF-04 `CompatibilityReport` is preserved unmodified.

Hard refinement is allowed only when before/after binding strengths are both exactly `required`, the ValueSet relation is an eligible finite proof, and there is no simultaneous strength interaction.

```text
CF07-BIND-001  narrowed      -> producer BREAKING
CF07-BIND-002  widened       -> consumer BREAKING
CF07-BIND-003  incomparable  -> producer BREAKING
CF07-BIND-004  incomparable  -> consumer BREAKING
```

`equal` never creates an implicit global SAFE finding. `extensible`, `preferred`, and `example` may carry relation evidence but do not become hard breaking from set mathematics alone. Strength changes remain an unsupported interaction for hard membership refinement.

## User-visible command

```text
commandf terminology <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --format json
```

The command performs no acquisition.

## Public report

```text
TerminologyDiffReport {
  schema,
  ruleset,
  package_name,
  before,
  after,
  compatibility,
  code_systems,
  value_sets,
  binding_refinements
}
```

Schema is `1`; ruleset is `cf07-terminology-v1`. Public collections are deterministically ordered and bounded. Repeated serialization for identical verified inputs is byte-identical.

## Fail-closed boundary

CF-07 fails rather than guesses on malformed interpreted fields, corrupted consumed cache objects, malformed canonical references, duplicate exact canonical identities, ambiguous canonical resolution when hard proof is attempted, duplicate complete-CodeSystem codes, unsupported CF-03/CF-04 schema/ruleset, bounds exhaustion, and unknown future relation/proof states.

Well-formed terminology cases outside the authorized proof domain are deterministic `indeterminate` evidence, not guessed compatibility.

## Safety / rights

Tests use synthetic terminology or public FHIR R4 package content. No proprietary SNOMED/LOINC/RxNorm corpus is committed. No PHI or remote terminology credentials are accepted by v1.

## Acceptance truth

CF-07 convergence requires the exact final PR head to prove:

1. exact CF-04 base and no CF-05/06 behavior leakage;
2. no acquisition or terminology-server call during `commandf terminology`;
3. complete CodeSystem equal/narrowed/widened/incomparable behavior and all completeness guards;
4. complete ValueSet expansion equal/narrowed/widened/incomparable behavior;
5. hierarchy flattened and logical repetitions de-duplicated without hierarchy inference;
6. paging/context/compose-only cases do not produce false complete proofs;
7. verified deterministic lock-closure canonical resolution;
8. dependency ValueSet drift can be detected even when CF-03/CF-04 are empty;
9. required narrowing/widening/incomparable directions map to `CF07-BIND-001..004` exactly;
10. equal membership does not invent SAFE and non-required bindings do not get hard breaks from set math alone;
11. embedded CF-04 report remains unmodified;
12. unresolved self-equivalence produces no noise while real unresolved changes remain explicit evidence;
13. repeated JSON is deterministic and bounded;
14. existing CF-01..04 gates stay green;
15. independently resolved public `hl7.fhir.r4.core@4.0.1` self-terminology smoke has no false terminology delta;
16. reviewer truth is recorded without invented PASSes;
17. `spec.md`, `plan.md`, `tasks.md`, and `convergence.md` agree with implementation;
18. PR stays Draft/open/unmerged, auto-merge disabled, and CF-08 unstarted.

## Explicit deferrals

No remote terminology server, general compose/filter implication solver, semantic concept-equivalence engine, proprietary terminology corpus, CF-08 annotations, CF-09 FSH source mapping, CF-10 public real-IG delta corpus, graph blast radius, mapping execution, or AI/agent authority is added here.
