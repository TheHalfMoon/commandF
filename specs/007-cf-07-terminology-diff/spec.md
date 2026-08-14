# CF-07 — Closed-World Terminology Diff

Status: Approved for implementation

## Purpose

CF-07 adds deterministic **ValueSet, CodeSystem, and terminology-binding membership evidence** above CF-04.

It closes the specific CF-04 deferral represented by `CF04-BIND-005`: a bound ValueSet change is currently `RISKY` in both directions because CF-04 cannot prove a membership subset/superset relation.

CF-07 does not build a terminology server and does not claim to solve arbitrary FHIR terminology semantics. It proves set relations only when the repository inputs establish a finite, locally closed membership universe. Every other case is explicit `indeterminate` evidence.

## Stack boundary

CF-07 depends on converged CF-04 only.

```text
base branch: feat/cf-04-compatibility-rules
base SHA: ae33586a925023d92b4d58db01663bf26f3bd9a3
```

CF-05 SARIF/policy and CF-06 HL7 StructureDefinition oracle behavior are not dependencies and MUST NOT leak into this slice.

## Direction authority

CF-07 preserves CF-04 direction semantics exactly:

- **producer** — whether output valid under the before contract remains valid under the after contract;
- **consumer** — whether a consumer prepared for all before-valid output can consume all after-valid output.

For a proven allowed-membership set relation:

- after set is a strict subset of before -> **narrowed** -> producer impact;
- after set is a strict superset of before -> **widened** -> consumer impact;
- both added and removed members -> **incomparable** -> potentially both directions;
- identical members -> **equal**;
- no complete proof -> **indeterminate**.

No alternative producer/consumer vocabulary is introduced.

## FHIR terminology boundary

FHIR R4 distinguishes:

- `ValueSet.compose` — the intensional definition of intended members;
- `ValueSet.expansion` — the actual list of members under recorded expansion conditions;
- `CodeSystem.content=complete` — complete CodeSystem content;
- `fragment`, `example`, `not-present`, and `supplement` — representations that are not a complete enumerated universe for membership proof.

A ValueSet definition may contain filters, imports, implicit code-system version choices, excludes, and other semantics that require terminology resolution. CF-07 v1 MUST NOT infer set inclusion from arbitrary `compose` structure.

The official HL7 `ValueSetComparer` and `CodeSystemComparer` are useful differential evidence but do not replace this boundary: the current 6.10.2 ValueSet comparison path compares compose structures and does not establish a general expansion subset proof.

## Membership identity

### CodeSystem concept identity

Within a matched CodeSystem canonical URL, membership identity is the exact concept `code` string. The report retains the before/after CodeSystem business versions as evidence.

Code comparison is not used for proof if `caseSensitive` changes across the two resources because code-equivalence semantics changed.

### ValueSet expansion concept identity

ValueSet expansion uniqueness follows the FHIR expansion identity tuple:

```text
system + version + code
```

`version` is nullable and is part of identity when present.

Nested `expansion.contains` is flattened only for membership collection. The hierarchy itself has no logical subset meaning and MUST NOT be used for inferencing.

Entries without `code` are navigation/grouping entries and are not members. A coded entry without `system` fails closed as malformed expansion evidence.

## Proof modes

CF-07 v1 exposes the proof mode on every relation.

### `code_system_complete`

A CodeSystem is eligible for finite membership proof only when:

1. `resourceType == CodeSystem`;
2. `content == complete`;
3. `compositional` is absent or false;
4. canonical URL is present;
5. all nested concept codes are non-empty and globally unique within the resource;
6. when `count` is present, it equals the flattened concept count;
7. `caseSensitive` does not change across the compared pair.

Otherwise the CodeSystem membership relation is `indeterminate` with a stable reason.

### `value_set_expansion`

A ValueSet is eligible for finite membership proof only when both compared resources contain a complete local expansion meeting all of these conditions:

1. `resourceType == ValueSet`;
2. expansion is present;
3. `offset` is absent or zero;
4. `total` is present and equals the number of unique flattened coded members;
5. each coded member has `system` and non-empty `code`;
6. duplicate `system|version|code` identities fail closed;
7. the normalized expansion parameter multiset is identical on both sides;
8. no unsupported parameter value shape is present.

`identifier` and `timestamp` are provenance of an expansion instance and are not membership identity. They are not required to match.

An expansion containing coded entries marked `abstract=true` can still be reported as artifact membership evidence, but CF-07 MUST NOT use that expansion for a hard binding-compatibility refinement in v1; the relation is retained with `binding_proof_eligible=false`.

A ValueSet with compose only, a paged/incomplete expansion, incompatible expansion parameters, or malformed expansion evidence is `indeterminate`. CF-07 does not call `$expand` and does not acquire terminology data.

## Relation vocabulary

```text
equal
narrowed
widened
incomparable
indeterminate
```

`narrowed`, `widened`, and `incomparable` require a complete finite set proof. `indeterminate` always carries a stable reason code.

Suggested reason vocabulary:

```text
missing_expansion
incomplete_or_paged_expansion
expansion_context_mismatch
malformed_expansion
unsupported_expansion_parameter
abstract_member_present
code_system_not_complete
code_system_compositional
code_system_case_sensitivity_changed
code_system_count_mismatch
malformed_code_system
unresolved_value_set
ambiguous_canonical
unsupported_binding_interaction
```

The implementation may add narrowly scoped reason codes only when tests and this specification are updated together.

## Canonical resolution boundary

CF-07 executes only against explicit verified CF-01 lock/cache states and performs no acquisition.

Terminology canonicals may live in the root package or any package already present in the explicit lock closure. CF-07 builds a deterministic read-only canonical index from those verified archives.

Resolution rules:

- exact `url|version` reference -> exact matching canonical/version required;
- bare canonical URL -> exactly one matching resource across the lock closure required;
- zero matches -> unresolved / `indeterminate` where allowed by the report contract;
- multiple matches -> ambiguous canonical and fail closed for any attempted hard compatibility proof.

CF-07 MUST NOT query a remote terminology server or user-level package cache.

## User-visible command

Add:

```text
commandf terminology <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --format json
```

The command performs no acquisition and verifies every cache object that it consumes.

## Public report

Suggested schema:

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

`compatibility` is the complete unmodified CF-04 `CompatibilityReport` for the same structural diff.

A set delta contains:

```text
TerminologySetDelta {
  resource,
  resource_type,
  proof_mode,
  relation,
  binding_proof_eligible,
  reason,
  before_count,
  after_count,
  added,
  removed
}
```

Member lists are stable-sorted. Public output must remain bounded; exact limits are defined in the implementation plan and tests.

## Binding refinement contract

CF-07 never rewrites or deletes the embedded CF-04 finding. It emits a separate refinement tied to the relevant CF-04 binding evidence.

Hard compatibility refinement is authorized only when:

1. before and after binding strengths are both exactly `required`;
2. the ValueSet membership relation is proven by an eligible finite proof;
3. no simultaneous binding-strength change makes the interaction ambiguous.

Then:

- `narrowed` -> producer `BREAKING` refinement;
- `widened` -> consumer `BREAKING` refinement;
- `incomparable` -> `BREAKING` in both producer and consumer directions;
- `equal` -> membership-equivalent evidence only; it does **not** create an implicit global `SAFE` claim.

For `extensible`, `preferred`, or `example`, CF-07 may attach the proven membership relation as evidence but MUST NOT upgrade that relation alone into a hard `BREAKING` judgment. Extensible applicability can depend on whether an in-set concept applies to the concept being communicated, which is not pure set mathematics.

If proof is `indeterminate`, the CF-04 `CF04-BIND-005` `RISKY` finding remains the operative compatibility statement.

## CodeSystem diff contract

For eligible complete CodeSystems, CF-07 reports deterministic added/removed code identities and the resulting set relation.

CF-07 does not claim that membership equality proves semantic equivalence of:

- displays/designations;
- definitions;
- properties;
- hierarchy meaning;
- concept status/inactivity;
- supplements;
- compositional grammar.

Those remain separate artifact semantics. A complete CodeSystem membership delta is code-set evidence only.

## ValueSet diff contract

For eligible complete expansions, CF-07 reports deterministic added/removed `(system, version, code)` identities and the set relation.

CF-07 does not infer set inclusion from:

- arbitrary compose filters;
- imported ValueSets without a complete eligible expansion;
- remote code systems;
- hidden terminology-server state;
- differing expansion contexts.

## Safety / rights boundary

No PHI is involved.

Repository tests and CI MUST use synthetic terminology or terminology content already distributable in the public FHIR R4 core package. CF-07 MUST NOT commit or redistribute licensed proprietary terminology content merely to test set inclusion.

No remote terminology service credentials or patient data are accepted by the v1 command.

## Determinism

For identical verified lock/cache inputs, output is byte-identical.

No clock, random id, temporary path, host path, environment value, network response, or unordered map iteration may enter public output.

## Fail-closed behavior

CF-07 fails rather than guessing on:

- unsupported CF-03 or CF-04 schema/ruleset;
- malformed CodeSystem or ValueSet fields that CF-07 interprets;
- duplicate complete-CodeSystem concept codes;
- duplicate expansion membership identities;
- malformed canonical references;
- ambiguous canonical resolution when a hard proof is attempted;
- corrupted consumed cache objects;
- unknown future proof/relation state.

An unsupported but well-formed terminology semantic case is represented as deterministic `indeterminate`, not as a parser/runtime failure.

## Acceptance

CF-07 is complete only when the exact final head proves:

1. exact CF-04 base and no CF-05/06 behavior leakage;
2. no package acquisition or terminology-server call during `commandf terminology`;
3. complete CodeSystem equal/narrowed/widened/incomparable relations;
4. `fragment`, `example`, `not-present`, `supplement`, compositional, count-mismatch, and case-sensitivity-change cases do not produce false complete-set proofs;
5. complete ValueSet expansion equal/narrowed/widened/incomparable relations;
6. compose-only/filter/import cases remain explicit `indeterminate` without remote expansion;
7. paged/incomplete expansion cannot produce a complete proof;
8. expansion context mismatch cannot produce a complete proof;
9. hierarchical expansion nesting is flattened for identity only and not used for logical inference;
10. duplicate/malformed membership evidence fails closed;
11. canonical terminology resolution across explicit verified lock closure is deterministic;
12. required-binding narrowing -> producer `BREAKING` refinement;
13. required-binding widening -> consumer `BREAKING` refinement;
14. required-binding incomparable replacement -> both-direction `BREAKING` refinements;
15. equal membership never creates an implicit global `SAFE` finding;
16. extensible/preferred/example membership changes do not become hard breaking from set math alone;
17. embedded CF-04 report is preserved unmodified;
18. unresolved/indeterminate binding leaves CF04-BIND-005 risk evidence intact;
19. repeated JSON is byte-identical and bounded;
20. existing CF-01..04 gates remain green;
21. real public R4 package self-terminology smoke produces no false set delta;
22. no proprietary terminology content is added to fixtures;
23. CodeRabbit/Qodo truth is reconciled without invented PASSes;
24. spec/plan/tasks/convergence match exact implementation truth;
25. PR remains Draft/open/unmerged with auto-merge disabled and CF-08 does not start before convergence.

## Explicit deferrals

CF-07 does not add:

- a terminology server or remote `$expand` execution;
- SNOMED/LOINC/RxNorm proprietary test corpus redistribution;
- general compose/filter implication solving;
- concept semantic-equivalence judgments;
- CF-08 GitHub annotations;
- CF-09 FSH source mapping;
- CF-10 public real-IG delta corpus;
- graph blast radius;
- mapping execution;
- AI/agent terminology authority.
