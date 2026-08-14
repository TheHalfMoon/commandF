# CF-07 Tasks

Status: Implementation complete; convergence validation pending exact final-head CI

- [x] **T001 — Terminology models/errors.** Schema-v1 deterministic terminology relation, proof-mode, member, set-delta, binding-refinement, report, and fail-closed error types implemented.
- [x] **T002 — Verified terminology closure.** Deterministic ValueSet/CodeSystem canonical indexes load every verified package in explicit before/after lock closures with no acquisition.
- [x] **T003 — Canonical resolution.** Exact `url|version`, unique bare URL, unresolved, ambiguity, duplicate exact identity, and no implicit latest selection implemented.
- [x] **T004 — Complete CodeSystem proof.** Finite set extraction/relation covers completeness, compositional, case-sensitive, count, duplicate, nesting, and member bounds.
- [x] **T005 — ValueSet expansion proof.** Complete expansion extraction/relation covers paging/total, logical tuple identity, hierarchy flatten/de-duplication, parameter context, abstract-member eligibility, and hard bounds.
- [x] **T006 — Root terminology delta discovery.** CF-03 matching authority is reused for direct root-package CodeSystem/ValueSet pairs; no second resource matcher exists.
- [x] **T007 — Binding resolution/reconciliation.** Matched CF-03 StructureDefinition/element bindings resolve through verified closures; dependency terminology drift is detected even when CF-03/CF-04 evidence is empty; the embedded CF-04 report is preserved.
- [x] **T008 — Required-binding hard refinements.** `CF07-BIND-001..004` implement required narrowing/widening/incomparable producer/consumer BREAKING refinements only; equal has no SAFE claim and non-required strengths remain evidence-only.
- [x] **T009 — `commandf terminology` CLI.** Explicit before/after lock/cache inputs, JSON format, zero acquisition, deterministic execution, and existing two-state package selection are wired.
- [x] **T010 — CodeSystem regression matrix.** Equal/narrowed/widened/incomparable, nesting, duplicates, content modes, compositional, count, case sensitivity, and non-membership metadata boundaries are covered.
- [x] **T011 — ValueSet regression matrix.** Expansion relations, nesting/navigation, hierarchy logical de-duplication, tuple validation, paging/total, parameter ordering/context mismatch, timestamp/identifier exclusion, abstract eligibility, and compose-only indeterminate cases are covered.
- [x] **T012 — Binding regression matrix.** Required directional refinements, incomparable both directions, equal no SAFE, non-required no hard break, strength interaction, unresolved/ambiguous ValueSets, self-equivalent unresolved suppression, dependency drift, and unchanged embedded CF-04 evidence are covered.
- [x] **T013 — Closure/CLI regressions.** Root/dependency canonical resolution, dependency cache corruption, offline behavior, deterministic JSON, bounds, and CLI UX are covered.
- [x] **T014 — Real R4 terminology smoke.** Green implementation candidate `06894d853a0ed9abdb73a622a9bb0a0d818ac3d4` passed run `31823710644`, including independently resolved/verified `hl7.fhir.r4.core@4.0.1` self-terminology with no false terminology delta. Exact final documentation head must re-pass the same gate before convergence certification.
- [x] **T015 — Review reconciliation.** CodeRabbit was requested twice on the green implementation candidate and rate-limited with zero substantive review threads; Qodo `/review` returned no substantive result. No reviewer PASS is invented. Cubic summaries are informational only.
- [x] **T016 — Convergence documentation.** `spec.md`, `plan.md`, `tasks.md`, and `convergence.md` are reconciled to implementation truth. Final convergence certification remains contingent on exact final-head CI plus Draft/open/unmerged/auto-merge/CF-08 governance checks.
