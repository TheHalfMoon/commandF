# CF-08 Tasks

Status: Implementation authorized

- [ ] **T001 — GitHub projection module.** Add deterministic bounded workflow-command annotation rendering from a validated CF-05 `CheckReport`.
- [ ] **T002 — Shared direction selection.** Reuse one CF-05 direction predicate between policy evaluation and annotation projection; do not duplicate producer/consumer semantics.
- [ ] **T003 — Workflow-command escaping.** Implement and regression-test data/property escaping so finding-controlled text cannot inject commands or properties.
- [ ] **T004 — Annotation bounds.** Enforce 10 error / 10 warning / 10 notice projections and deterministic overflow disclosure without changing report decision truth.
- [ ] **T005 — No-location boundary.** Prove CF-08 emits no explicit file/line/column properties and labels findings artifact-level pending CF-09.
- [ ] **T006 — `github-annotations` CLI.** Add bounded report-file parsing, deterministic rendering, help/usage behavior, and fail-closed schema/ruleset handling.
- [ ] **T007 — Root composite Action.** Add `action.yml` with package/lock/cache/policy inputs and report/exit/passed outputs.
- [ ] **T008 — Action runner.** Add quoted-argv source-backed Linux runner that preserves CF-05 exit 0/1/2, renders policy-fail annotations before exiting 2, and does not silently create caller report parents.
- [ ] **T009 — Action security regressions.** Cover shell metacharacters, workflow-command injection, renderer failure, operational failure, report preservation, and output correctness.
- [ ] **T010 — Projection regression matrix.** Cover severity mapping, direction selection, fail-on independence, limits, overflow, deterministic bytes, invalid reports, and no-source-location behavior.
- [ ] **T011 — CLI regressions.** Cover help, required input, valid empty report, policy-failed report renderer success, malformed JSON, and oversized input.
- [ ] **T012 — Existing stack regression.** Preserve all exact CF-01..05 format/clippy/test and real-R4 self-check behavior.
- [ ] **T013 — Real local Action smoke.** Invoke `uses: ./` against independently resolved real R4 states and assert Action outputs plus full report truth.
- [ ] **T014 — Review reconciliation.** Request/reconcile CodeRabbit and Qodo when available; fix valid findings and record unavailable/rate-limited truth without invented PASSes.
- [ ] **T015 — Convergence.** Reconcile spec/plan/tasks, add `convergence.md`, validate exact final head, keep PR Draft/open/unmerged with auto-merge disabled, and do not start CF-09.
