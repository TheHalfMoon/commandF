# CF-08 Tasks

Status: Implemented — exact final documentation-head certification pending

- [x] **T001 — GitHub projection module.** Added deterministic bounded workflow-command annotation rendering from a validated CF-05 `CheckReport`.
- [x] **T002 — Shared direction selection.** CF-05 policy evaluation and CF-08 projection reuse the same crate-private direction predicate.
- [x] **T003 — Workflow-command escaping.** Data/property escaping and injected-command regressions prevent finding-controlled workflow-command/property injection.
- [x] **T004 — Annotation bounds.** Enforced 10 error / 10 warning / 10 total notice commands with reserved overflow notice, plus deterministic omission counts.
- [x] **T005 — No-location boundary.** CF-08 emits no explicit file/line/column/end-position properties and labels findings artifact-level pending CF-09.
- [x] **T006 — `github-annotations` CLI.** Added 64 MiB bounded report parsing, persisted-decision validation, deterministic rendering, valid policy-failure rendering, and fail-closed malformed/unsupported behavior.
- [x] **T007 — Root composite Action.** Added repository-root `action.yml` with package/lock/cache/policy inputs and report/exit/passed outputs.
- [x] **T008 — Action runner.** Added pinned source build wrapper plus pure quoted-argv run wrapper preserving CF-05 exit `0/1/2`, rendering before exit `2`, and not silently creating caller report parents.
- [x] **T009 — Action security regressions.** Covered shell metacharacters, paths with spaces, policy-fail rendering, operational failure, renderer failure, stale-report output avoidance, and caller-parent semantics.
- [x] **T010 — Projection regression matrix.** Covered severity mapping, direction selection, fail-on independence, limits, overflow, title/message bounds, deterministic bytes, invalid/inconsistent reports, and no-source-location behavior.
- [x] **T011 — CLI regressions.** Covered help, required input, empty report, policy-failed report renderer success, malformed JSON, and oversized input.
- [x] **T012 — Existing stack regression.** Exact implementation run `31830175341` preserved Format, locked Clippy, full tests, and real CF-01..05 R4 self-check behavior.
- [x] **T013 — Real local Action smoke.** Exact implementation run `31830175341` invoked `uses: ./` against independently resolved real R4 states and verified report-path / exit-code / passed outputs plus report truth.
- [x] **T014 — Review reconciliation.** CodeRabbit substantive review completed on exact implementation head `a5c24bc5fa9ee0360a3f6822eb7d8a97f8fe06a7` with no blocking issues and no actionable inline threads. Qodo `/review` returned no substantive result; no Qodo PASS is claimed.
- [x] **T015 — Convergence.** Spec/plan/tasks/convergence are reconciled. Exact final documentation-head CI and final PR/governance evidence are intentionally recorded in PR metadata after this docs commit to avoid self-referential commit churn. CF-09 must remain unstarted until that final gate is complete.
