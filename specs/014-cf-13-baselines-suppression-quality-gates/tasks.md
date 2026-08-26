# CF-13 Tasks — Baselines, Suppressions, and Quality Gates

Status: PLANNING_CANDIDATE

Tasks are dependency ordered. A task is complete only with executable evidence on the exact candidate state.

## Planning and contract freeze

- [x] T001 — Confirm CF-13 roadmap authority and dependency eligibility.
  - Master Architecture V2 defines CF-13 as `baselines/suppression/quality gates` depending on CF-05.
  - CF-13 does not depend on the externally blocked CF-06/CF-10 production-oracle path.

- [x] T002 — Freeze the V1 user-visible command and exit contract.
  - `commandf gate <package>` with exact before/after lock/cache inputs;
  - optional CF-05 baseline and exact suppression file;
  - existing CF-05 direction/fail-on semantics;
  - JSON-only V1;
  - exit 0 pass / 1 operational-or-input / 2 completed gate failure.

- [x] T003 — Freeze baseline, fingerprint, suppression, and evidence semantics.
  - exact unique SHA-256 finding fingerprints;
  - valid same-package/same-ruleset CF-05 baseline;
  - exact non-wildcard suppressions with mandatory rationale;
  - `suppressed > baseline > new` disposition precedence;
  - full current CF-05 evidence preserved.

- [ ] T004 — Complete planning consistency, exact-head CI, and independent planning review; merge planning before implementation.

## Stack A — deterministic quality-gate library

- [ ] T010 — Add CF-13 V1 public models.
  - quality-gate report/decision/finding/disposition;
  - baseline/suppression evidence;
  - suppression input schema v1;
  - deterministic JSON serialization.

- [ ] T011 — Implement deterministic finding fingerprint V1.
  - fixed semantic key fields from `spec.md`;
  - recursive JSON object-key canonicalization;
  - SHA-256 `sha256:<lowercase hex>` output;
  - message excluded;
  - positive and counterexample fingerprint tests.

- [ ] T012 — Validate and normalize CF-05 baselines.
  - existing `validate_check_report` authority;
  - exact package/ruleset compatibility;
  - duplicate fingerprint rejection;
  - canonical baseline digest/evidence.

- [ ] T013 — Validate and normalize suppression files.
  - schema, syntax, bounds, non-empty rationale;
  - duplicate rejection;
  - deterministic order and canonical digest;
  - unmatched suppression retention.

- [ ] T014 — Implement deterministic finding disposition.
  - suppression precedence;
  - baseline matching;
  - new finding classification;
  - original current finding order preserved.

- [ ] T015 — Implement quality-gate decision by composing CF-05 policy semantics.
  - direction parity;
  - breaking/risky/none parity;
  - only selected `new` findings can block;
  - baseline/suppressed evidence retained but non-blocking.

- [ ] T016 — Implement CF-13 report validation/invariant checks.
  - current CF-05 report validity;
  - fingerprint/disposition/count consistency;
  - unique identities;
  - fail closed on unknown/inconsistent state.

- [ ] T017 — Prove library determinism and bounds.
  - repeated bytes;
  - semantic JSON key-order invariance;
  - suppression order invariance;
  - duplicate/invalid/bounded-input failures;
  - no silent evidence deletion.

- [ ] T018 — Prove CF-05 behavior remains unchanged by Stack A refactors.
  - existing check model/evaluator/SARIF tests unchanged and green;
  - any shared helper extraction covered by exact parity tests.

## Stack B — shipped `commandf gate`

- [ ] T020 — Add the `commandf gate` CLI surface.
  - exact two-state package inputs;
  - direction/fail-on;
  - optional baseline/suppressions;
  - JSON/output arguments.

- [ ] T021 — Add bounded baseline/suppression file loading and fail-closed CLI errors.
  - explicit byte limits;
  - bounded diagnostic behavior;
  - no network acquisition added.

- [ ] T022 — Preserve atomic output and gate exit semantics.
  - complete output before exit 2;
  - atomic replacement on pass/fail;
  - gate parse failures normalized to 1;
  - non-check/non-gate Clap behavior unchanged.

- [ ] T023 — Add end-to-end CLI fixtures/regressions.
  - new blocker;
  - baseline pass;
  - suppression pass;
  - stale suppression cannot hide blocker;
  - malformed/mismatched inputs exit 1;
  - deterministic repeated bytes.

- [ ] T024 — Add dedicated `cf13-quality-gate-proof` workflow.
  - pinned toolchain/actions;
  - complete CF-13 path filters;
  - baseline + new + suppression proof;
  - repeated byte equality;
  - clean repository;
  - retained digest artifact.

- [ ] T025 — Record exact CF-13 deterministic proof identity.
  - exact head/tree;
  - run/job/artifact ids;
  - artifact digest;
  - `CF13_GATE_SHA256`.

- [ ] T026 — Prove existing user-visible command behavior remains unchanged.
  - `commandf check` JSON/SARIF/exit semantics;
  - CF-01 through CF-12 applicable regressions;
  - no CF-06 identity, CF-10 corpus, dependency, or lock-schema mutation.

## Regression, review, and convergence

- [ ] T040 — Run mandatory workspace gates on the exact final implementation head.
  - format;
  - Clippy with `-D warnings`;
  - full workspace tests.

- [ ] T041 — Run every path-applicable repository workflow and configured real-FHIR/security regression.
  - do not invent workflows that path filters do not trigger.

- [ ] T042 — Independent implementation review.
  - CodeRabbit when available;
  - Qodo when connected/available;
  - disposition every substantive finding;
  - record rate limits/unavailability without invented PASS.

- [ ] T043 — Run CF-13 convergence.
  - record exact final heads/trees/runs/jobs/artifacts/digests;
  - record `CF13_GATE_SHA256`;
  - verify unresolved substantive findings = 0;
  - record explicit V1 coverage limits/deferrals;
  - create docs-only closeout stack if needed.

## Hard sequencing rules

1. T004 must be complete and the planning PR merged before T010 implementation starts.
2. T010 precedes T011-T016; T011 precedes baseline/suppression matching; T012/T013 precede T014; T014 precedes T015; T015 precedes T016/T017.
3. Stack A must merge before Stack B starts unless the canonical planning merge explicitly authorizes a stacked implementation base.
4. `commandf check` remains CF-05 authority and must not be silently changed to new-change-first semantics.
5. Baseline/suppression matching is exact; no wildcard or inferred waiver authority is allowed.
6. No time/network/model authority is introduced.
7. No CF-06 production pin, frozen CF-10 corpus, or lock-schema mutation is authorized by CF-13.