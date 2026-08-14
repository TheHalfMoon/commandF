# CF-05 Tasks

Status: Approved for implementation

- [ ] **T001 — Gate model.** Add versioned `CheckPolicy`, `CheckDecision`, and `CheckReport` types with deterministic JSON serialization.
- [ ] **T002 — Policy evaluator.** Validate CF-04 schema/ruleset and evaluate producer/consumer/both plus breaking/risky/none thresholds without mutating CF-04 evidence.
- [ ] **T003 — SARIF 2.1.0 model.** Add deterministic SARIF serialization using CF-04 rule ids, severity-to-level mapping, messages, and commandF evidence properties.
- [ ] **T004 — No-fake-location boundary.** Ensure SARIF contains no invented physical repository locations; document CF-09 source-mapping dependency for GitHub annotations.
- [ ] **T005 — Stable exit contract.** Introduce check-specific process outcome: exit 0 pass, exit 1 operational failure, exit 2 completed policy failure; preserve existing commands.
- [ ] **T006 — `commandf check` CLI.** Reuse the two-state CF-03 loader and CF-04 classifier; add direction, fail-on, format, and optional output path.
- [ ] **T007 — Atomic output.** Write stdout or atomically replace an explicit output path, with complete output emitted before exit 2.
- [ ] **T008 — Package regressions.** Cover threshold/direction/count logic, unsupported CF-04 authority, deterministic JSON/SARIF, severity mapping, rule ordering, evidence preservation, and no timestamps/locations.
- [ ] **T009 — CLI regressions.** Cover help, stdout JSON/SARIF, exit 2 with output, operational exit 1, output-file behavior, and no acquisition/network dependency.
- [ ] **T010 — Real R4 smoke.** Extend exact CI with independent `hl7.fhir.r4.core@4.0.1` JSON and SARIF `commandf check` self-equivalence assertions.
- [ ] **T011 — Review reconciliation.** Request CodeRabbit/Qodo when available, verify all findings against current code, fix only valid issues, and record exact reviewer truth.
- [ ] **T012 — Convergence.** Reconcile spec/plan/tasks, add `convergence.md`, run exact-final-head Format/Clippy/Test/real smoke, keep PR Draft, and do not begin CF-06.
