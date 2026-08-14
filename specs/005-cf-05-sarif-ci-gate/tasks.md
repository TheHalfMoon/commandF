# CF-05 Tasks

Status: Implemented — convergence candidate

- [x] **T001 — Gate model.** Add versioned `CheckPolicy`, `CheckDecision`, and `CheckReport` types with deterministic JSON serialization.
- [x] **T002 — Policy evaluator.** Validate CF-04 schema/ruleset and evaluate producer/consumer/both plus breaking/risky/none thresholds without mutating CF-04 evidence.
- [x] **T003 — SARIF 2.1.0 model.** Add deterministic SARIF serialization using CF-04 rule ids, severity-to-level mapping, messages, and commandF evidence properties.
- [x] **T004 — No-fake-location boundary.** Ensure SARIF contains no invented physical repository locations; document CF-09 source-mapping dependency for GitHub annotations.
- [x] **T005 — Stable exit contract.** Reserve exit 0 for pass, exit 1 for check usage/operational/classification failure, and exit 2 only for completed policy failure while preserving existing non-check parse behavior.
- [x] **T006 — `commandf check` CLI.** Reuse the two-state CF-03 loader and CF-04 classifier; add direction, fail-on, format, and optional output path.
- [x] **T007 — Atomic output.** Write stdout or publish a synced same-directory temporary file by rename over an explicit output path, with complete output emitted before exit 2 and stale output replaceable.
- [x] **T008 — Package regressions.** Cover threshold/direction/count logic, unsupported CF-04 authority, repeated-evaluation deterministic JSON/SARIF, severity mapping, rule ordering, messages, evidence preservation, and no locations.
- [x] **T009 — CLI regressions.** Cover help, JSON/SARIF output, exit 2 with complete output, operational exit 1, invalid policy syntax, existing-output replacement, missing-parent failure, and no acquisition/network dependency.
- [x] **T010 — Real R4 smoke.** Extend exact CI with independent `hl7.fhir.r4.core@4.0.1` JSON and SARIF `commandf check` self-equivalence assertions.
- [x] **T011 — Review reconciliation.** Request CodeRabbit/Qodo, verify findings against current code, fix the valid atomic-replacement finding, resolve its thread, and record that the later incremental CodeRabbit review was rate-limited and Qodo returned no review result.
- [x] **T012 — Convergence.** Reconcile spec/plan/tasks, add `convergence.md`, require fresh exact-final-head Format/Clippy/Test/real smoke after the docs-only convergence commit, keep PR Draft, and do not begin CF-06.
