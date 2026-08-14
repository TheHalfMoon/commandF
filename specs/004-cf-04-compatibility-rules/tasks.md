# CF-04 Tasks

Status: Approved for implementation

- [ ] **T001 — Compatibility report model.** Add deterministic schema-v1 compatibility report, severity/direction enums, stable ruleset id, findings, and byte-stable JSON serialization.
- [ ] **T002 — Fail-closed classifier boundary.** Add `CompatibilityError` and `classify_structural_diff`, reject unsupported CF-03 schemas, and exhaustively dispatch every structural change kind.
- [ ] **T003 — Cardinality and maxLength rules.** Implement producer/consumer variance rules for `min`, `max`, and `maxLength`, including `*` handling and malformed-value rejection.
- [ ] **T004 — Type-choice rules.** Compare normalized CF-03 type entries as deterministic sets and classify narrowing, widening, and incomparable replacement.
- [ ] **T005 — Fixed/pattern/value-bound rules.** Implement fixed and pattern add/remove/change behavior and conservative RISKY treatment for value-bound changes whose ordering cannot be proved generically.
- [ ] **T006 — Terminology binding rules.** Classify R4 binding-strength changes directionally; emit RISKY both for ValueSet changes without CF-07 membership semantics.
- [ ] **T007 — Constraint/support/modifier/slicing rules.** Implement error/warning constraint changes, Must Support RISKY behavior, new modifier consumer BREAKING behavior, and slicing restriction/relaxation rules.
- [ ] **T008 — Resource/view/element/residual rules.** Cover all remaining CF-03 change kinds and structural fields without silent fallback; unknown future fields fail closed.
- [ ] **T009 — Snapshot/differential deduplication.** Prefer equivalent snapshot evidence over duplicate differential field evidence while preserving distinct view changes.
- [ ] **T010 — CLI.** Add `commandf classify` with the same explicit two-state lock/cache contract as `commandf diff`, no acquisition, and JSON output.
- [ ] **T011 — Synthetic/CLI/real smoke validation.** Add rule positive/counterexample tests, CLI contracts, deterministic serialization checks, and real independent R4 self-classification with zero findings.
- [ ] **T012 — Review and convergence.** Run exact-head Format/Clippy/Test/real smoke, request CodeRabbit/Qodo when available, disposition all valid findings, write `convergence.md`, keep the PR Draft, and do not begin CF-05.
