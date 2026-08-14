# CF-03 Tasks

Status: Authorized implementation sequence

- [ ] **T001 — Shared bounded resource scanner.** Factor CF-02 package-root JSON scanning into one internal helper used by inspect and diff without changing CF-02 behavior or limits.
- [ ] **T002 — Diff models.** Add deterministic `StructuralDiffReport`, package evidence, resource key, change-kind, and change records; no severity fields.
- [ ] **T003 — Two-state package loading.** Load one exact selected package from each explicit before/after lock+cache pair, verify cache objects, and recheck archive digests.
- [ ] **T004 — Deterministic resource matcher.** Implement canonical URL grouping, exact `url|version` fallback for multi-version groups, unique non-canonical `resourceType/id`, filename fallback, and fail-closed ambiguity.
- [ ] **T005 — StructureDefinition shape parser.** Parse selected structural metadata plus snapshot/differential element objects keyed by exact `ElementDefinition.id`.
- [ ] **T006 — Structural normalization.** Canonicalize objects and known set-like arrays while preserving potentially meaningful order.
- [ ] **T007 — Typed diff engine.** Emit resource/view/element additions/removals and field changes without compatibility classification.
- [ ] **T008 — Deterministic serialization.** Stable-sort changes and prove byte-identical JSON for identical inputs.
- [ ] **T009 — CLI.** Implement `commandf diff <package-name>` with explicit before/after lock/cache paths and `--format json`; no acquisition.
- [ ] **T010 — Synthetic contract tests.** Cover no-op, resource matching, multi-version canonical groups, ambiguity failure, view/element changes, structural fields, editorial exclusion, and ordering.
- [ ] **T011 — Real self-diff smoke.** Resolve/verify the same `hl7.fhir.r4.core@4.0.1` into two independent states and prove `changes` is empty.
- [ ] **T012 — Review/convergence.** Run locked CI, record reviewer availability truth, reconcile spec/plan/tasks, keep stacked PR Draft, and do not introduce CF-04 severity logic.
