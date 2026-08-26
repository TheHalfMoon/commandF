# CF-13 Tasks — Baselines, Suppressions, and Quality Gates

Status: CONVERGENCE_CANDIDATE

Tasks are dependency ordered. A task is complete only with executable evidence on the exact candidate state.

This task ledger records completed planning and implementation work through canonical PRs #30, #31, and #32. CF-13 is **not** `CLOSED_CANONICAL` until the docs-only convergence closeout containing this ledger qualifies on its exact head and merges to `main`.

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
  - exact unique SHA-256 finding fingerprints with explicit persisted fingerprint schema `1`;
  - valid same-package/same-ruleset CF-05 baseline;
  - recursive canonical JSON object-key normalization for semantic digests/fingerprints while preserving array order;
  - baseline evidence retains exact before/after package identities plus complete sorted fingerprint membership;
  - suppression evidence retains complete normalized membership with mandatory rationale;
  - exact non-wildcard suppressions only;
  - `suppressed > baseline > new` disposition precedence;
  - full current CF-05 evidence preserved;
  - canonical digests bind retained evidence but never substitute for membership needed to revalidate dispositions.

- [x] T004 — Complete planning consistency, exact-head CI, and independent planning review; merge planning before implementation.
  - planning PR #30 final head `33a0536d745d67ac6a094ce891293efa7e2204b9`;
  - planning merge `cb3d0824d795b06d40bd121798030be15bba507c` became the canonical Stack A base.

## Stack A — deterministic quality-gate library

- [x] T010 — Add CF-13 V1 public models.
  - explicit-version `FindingFingerprint { schema, digest }`;
  - quality-gate report/decision/finding/disposition;
  - membership-bearing baseline/suppression evidence;
  - suppression input schema v1;
  - deterministic JSON serialization.

- [x] T011 — Implement deterministic finding fingerprint V1.
  - fixed semantic key fields from `spec.md`;
  - recursive JSON object-key canonicalization;
  - SHA-256 `sha256:<lowercase hex>` digest inside explicit schema `1` identity;
  - message excluded;
  - positive and counterexample fingerprint tests;
  - unsupported/cross-version fingerprint identities rejected before matching.

- [x] T012 — Validate and normalize CF-05 baselines.
  - existing `validate_check_report` authority;
  - exact package/ruleset compatibility;
  - duplicate fingerprint rejection;
  - recursively canonical baseline digest;
  - retain exact baseline before/after `PackageEvidence`;
  - retain complete lexicographically sorted unique baseline fingerprint membership.

- [x] T013 — Validate and normalize suppression files.
  - suppression schema and fingerprint schema validation;
  - digest syntax, bounds, non-empty rationale;
  - duplicate rejection;
  - deterministic order and recursively canonical digest;
  - retain complete normalized suppression membership;
  - unmatched suppression retention.

- [x] T014 — Implement deterministic finding disposition.
  - same-version suppression precedence;
  - baseline membership matching;
  - new finding classification;
  - original current finding order preserved.

- [x] T015 — Implement quality-gate decision by composing CF-05 policy semantics.
  - direction parity;
  - breaking/risky/none parity;
  - only selected `new` findings can block;
  - baseline/suppressed evidence retained but non-blocking.

- [x] T016 — Implement CF-13 persisted-report validation/invariant checks.
  - current CF-05 report validity;
  - supported report/suppression/fingerprint schema validation;
  - recomputed current fingerprints;
  - baseline membership/count/package/ruleset/before/after identity validation;
  - suppression membership/count/rationale/reference validation;
  - every `baseline` disposition must match retained baseline membership;
  - every `suppressed` disposition must match retained suppression membership and metadata;
  - recomputed unused suppressions, disposition counts, and decision;
  - unique identities;
  - fail closed on unknown/inconsistent/insufficient evidence.

- [x] T017 — Prove library determinism, tamper resistance, and bounds.
  - legitimate serialized report validates successfully;
  - repeated report/validation results are deterministic;
  - semantic top-level and nested JSON object-key invariance;
  - meaningful array-order differences remain identity-bearing;
  - suppression order invariance;
  - duplicate/invalid/bounded-input failures;
  - unsupported fingerprint schema rejected;
  - forged `baseline` disposition without retained membership rejected;
  - forged `suppressed` disposition or altered waiver metadata rejected;
  - altered current fingerprint rejected;
  - baseline/suppression membership count mismatch rejected;
  - decision/count mismatch rejected;
  - unknown disposition/report/fingerprint schema values rejected;
  - no silent evidence deletion.

- [x] T018 — Prove CF-05 behavior remains unchanged by Stack A refactors.
  - existing check model/evaluator/SARIF tests unchanged and green;
  - shared policy helpers retain exact CF-05 behavior under regression tests.
  - Stack A merged through PR #31 at canonical merge `82bf9d69c8b574ba7f302296e08b416d7566a351` from final head `8bdca1bc66539058310249f5841ece9fca2a437a`.

## Stack B — shipped `commandf gate`

- [x] T020 — Add the `commandf gate` CLI surface.
  - exact two-state package inputs;
  - direction/fail-on;
  - optional baseline/suppressions;
  - JSON/output arguments.

- [x] T021 — Add bounded baseline/suppression file loading and fail-closed CLI errors.
  - explicit byte limits;
  - bounded diagnostic behavior;
  - unsupported fingerprint versions fail closed;
  - no network acquisition added;
  - exact CLI regressions cover both oversized baseline and oversized suppression inputs.

- [x] T022 — Preserve atomic output and gate exit semantics.
  - complete output before exit 2;
  - atomic replacement on pass/fail;
  - gate parse failures normalized to 1;
  - non-check/non-gate Clap behavior unchanged.

- [x] T023 — Add end-to-end CLI fixtures/regressions.
  - new blocker;
  - baseline pass;
  - suppression pass;
  - stale suppression cannot hide blocker;
  - malformed/mismatched/version-incompatible inputs exit 1;
  - deterministic repeated bytes.

- [x] T024 — Add dedicated `cf13-quality-gate-proof` workflow.
  - pinned toolchain/actions;
  - complete CF-13 path filters;
  - baseline + new + suppression proof;
  - persisted-report validation;
  - repeated byte equality;
  - clean repository;
  - retained deterministic evidence artifact.

- [x] T025 — Record exact CF-13 deterministic proof identity and immutable provenance.
  - exact implementation head `06da4f3f61b47afe11525b2c33306b5952cd680e`;
  - exact implementation tree `6707735a3d3521380ab22a31d4a0865982fadd6a`;
  - proof run `32978131520`, job `98207812843`, artifact `9610321732`;
  - artifact digest `sha256:4e1f8e0cf4167e77153e2d5ff8749d146881a1a6c20608f743c2c44a71c5a8fe`;
  - `CF13_GATE_SHA256=118fdd9e7606394d4abcbb39b51e0af81d303c95e3a513886acb1bedb95e93cf`;
  - proof artifact retains repository, dependency, toolchain, fixture, package/archive, baseline, suppression, and governing-contract identities required by `spec.md`.

- [x] T026 — Prove existing user-visible command behavior remains unchanged.
  - `commandf check` JSON/SARIF/exit semantics;
  - CF-01 through CF-12 applicable regressions;
  - no CF-06 identity, CF-10 corpus, dependency, or lock-schema mutation.

## Regression, review, and convergence

- [x] T040 — Run mandatory workspace gates on the exact final implementation head.
  - `ci` run `32978131562` succeeded on `06da4f3f61b47afe11525b2c33306b5952cd680e`, including format, Clippy with `-D warnings`, full workspace tests, and configured regressions.

- [x] T041 — Run every path-applicable repository workflow and configured real-FHIR/security regression.
  - exact final implementation head succeeded in `ci`, `cf06-oracle`, `cf11-multi-version-proof`, `cf11g-context-proof`, `cf12-impact-proof`, and `cf13-quality-gate-proof`.
  - exact workflow identities are recorded in `convergence.md`.

- [x] T042 — Independent implementation review.
  - Qodo re-reviewed the exact final implementation head after the oversized-input regression was added and reported no substantive issues;
  - CodeRabbit completed the exact final incremental review with no actionable comments;
  - every returned substantive inline thread is resolved;
  - reviewer warnings/operational notes are retained in `convergence.md` without inventing approval beyond returned evidence.

- [x] T043 — Run CF-13 convergence.
  - this docs-only closeout records exact planning/implementation/merge/proof/review evidence, V1 limits, and deferrals;
  - the closeout itself must pass its exact-head path-applicable workflows and independent review before merge;
  - only the canonical closeout merge may classify CF-13 as `CLOSED_CANONICAL`.

## Hard sequencing rules

1. T004 completed through canonical planning PR #30 before Stack A implementation.
2. T010 preceded T011-T016; fingerprint/baseline/suppression/disposition/decision/report-validation dependencies were implemented in Stack A and merged through PR #31.
3. Stack A became canonical before Stack B PR #32 was based on it.
4. `commandf check` remains CF-05 authority and was not silently changed to new-change-first semantics.
5. Baseline/suppression matching remains exact and fingerprint-version-aware; no wildcard or inferred waiver authority is allowed.
6. A persisted baseline/suppressed disposition is not authoritative unless the report retains membership evidence sufficient to revalidate it.
7. No time/network/model authority is introduced.
8. No CF-06 production pin, frozen CF-10 corpus, or lock-schema mutation is authorized by CF-13.
9. The implementation merges do not alone close CF-13; canonical closure requires the qualified docs-only convergence merge.