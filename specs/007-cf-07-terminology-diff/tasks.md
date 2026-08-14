# CF-07 Tasks

Status: Implementation authorized

- [ ] **T001 — Terminology models/errors.** Add schema-v1 deterministic terminology relation, proof-mode, member, set-delta, binding-refinement, report, and fail-closed error types.
- [ ] **T002 — Verified terminology closure.** Build deterministic ValueSet/CodeSystem canonical indexes from every verified package in explicit before/after lock closures with no acquisition.
- [ ] **T003 — Canonical resolution.** Implement exact `url|version`, unique bare URL, unresolved, and ambiguity behavior without implicit latest selection.
- [ ] **T004 — Complete CodeSystem proof.** Implement finite set extraction/relation with complete/content, compositional, case-sensitive, count, duplicate, nesting, and hard member bounds.
- [ ] **T005 — ValueSet expansion proof.** Implement complete expansion extraction/relation with paging/total, tuple uniqueness, hierarchy flattening, parameter context normalization, abstract-member binding eligibility, and hard bounds.
- [ ] **T006 — Root terminology delta discovery.** Reuse CF-03 matching authority for direct root-package CodeSystem/ValueSet before/after pairs and emit deterministic set evidence without a second matcher.
- [ ] **T007 — Binding resolution/reconciliation.** Resolve before/after bound ValueSets through the verified closures, preserve the complete CF-04 report, and emit relation evidence tied to binding changes.
- [ ] **T008 — Required-binding hard refinements.** Add `CF07-BIND-001..004` for proven required-binding narrowing/widening/incomparable relations only; equal has no SAFE claim and non-required strengths do not become hard breaks from set math alone.
- [ ] **T009 — `commandf terminology` CLI.** Wire explicit before/after lock/cache inputs, JSON format, zero acquisition, deterministic execution, and existing two-state package-selection behavior.
- [ ] **T010 — CodeSystem regression matrix.** Cover equal/narrowed/widened/incomparable, nesting, duplicates, content modes, compositional, count, case sensitivity, and non-membership metadata changes.
- [ ] **T011 — ValueSet regression matrix.** Cover complete expansion relations, nesting/navigation, tuple validation, paging/total, parameter ordering/context mismatch, timestamp/identifier exclusion, abstract binding eligibility, and compose-only indeterminate cases.
- [ ] **T012 — Binding regression matrix.** Cover required directional refinements, incomparable both directions, equal no SAFE, non-required no hard break, strength interaction, unresolved/ambiguous ValueSets, and unchanged embedded CF-04 evidence.
- [ ] **T013 — Closure/CLI regressions.** Cover root/dependency canonical resolution, cache corruption, offline behavior, deterministic JSON, bounds, and CLI UX.
- [ ] **T014 — Real R4 terminology smoke.** Preserve all CF-01..04 gates and prove independently resolved `hl7.fhir.r4.core@4.0.1` self-terminology produces no false terminology delta.
- [ ] **T015 — Review reconciliation.** Trigger/reconcile CodeRabbit and Qodo when available; fix valid findings, resolve actionable threads, and record reviewer truth without invented PASSes.
- [ ] **T016 — Convergence.** Reconcile spec/plan/tasks, add `convergence.md`, validate exact final head, keep PR Draft/open/unmerged and auto-merge disabled, and do not start CF-08.
