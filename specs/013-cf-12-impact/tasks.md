# CF-12 Tasks — Deterministic Impact Analysis

Status: PLANNING_READY_FOR_MERGE — T004 is closed by the first exact-head planning qualification; implementation remains blocked until this planning PR itself passes final-head requalification and merges.

Tasks are dependency ordered. A task is complete only with executable evidence on the exact candidate state.

## Planning and contract freeze

- [x] T001 — Confirm CF-12 entry eligibility from canonical CF-11G closure.
  - CF-11G closeout main: `8f2ce65de3565a81968bb127c96b451f617593c4`.
  - CF-12 remains `commandf impact`.
  - No CF-06/CF-10 production-oracle dependency is introduced.

- [x] T002 — Freeze the V1 CLI input shape.
  - positional selected package;
  - explicit before/after lock/cache inputs matching existing diff/check conventions;
  - JSON-only output in V1.

- [x] T003 — Freeze the authority boundary.
  - impact = deterministic dependency exposure evidence;
  - impact != BREAKING/RISKY/ADDITIVE classification;
  - no network canonical resolution;
  - no PHI/instance data;
  - no graph database/model/agent authority.

- [x] T004 — Close planning consistency and independent review.
  - `spec.md`, `plan.md`, `tasks.md`, and `consistency.md` contain no known unresolved contradiction;
  - first exact planning head `1bee6f3651fa686f03902f3d86761736d4844513` passed `ci` run `32928525763` and `cf06-oracle` run `32928525784`;
  - CodeRabbit status was `success` and no review thread/substantive finding was returned on that planning head;
  - Qodo was not observed connected/available; no Qodo PASS is claimed;
  - this T004 state change moves the head, so the planning PR MUST rerun applicable exact-head gates/review before merge.

## Stack A — library model and deterministic traversal

- [ ] T010 — Add library-owned CF-12 impact report schema v1.
  - subject/evidence identity, seeds, artifact impacts, package impacts, unresolved boundaries, coverage, canonical JSON.

- [ ] T011 — Build deterministic change seeds from the existing package structural-diff pipeline.
  - added, removed, and modified canonical artifacts with exact side-specific identity;
  - no second diff engine.

- [ ] T012 — Build side-specific reverse indexes over resolved CF-11G canonical-reference edges.
  - only `resolved` edges are traversable;
  - exact artifact identities remain version-aware.

- [ ] T013 — Implement deterministic transitive reverse artifact traversal.
  - direct/transitive dependents, cycle termination, exact visited state, canonical shortest path.

- [ ] T014 — Implement equal-length path tie-breaking.
  - minimum edge count, then lexicographically smallest stable exact path.

- [ ] T015 — Implement exact reverse package-dependency exposure.
  - schema-v2 exact edges, exact version/digest identity, declared constraints, no name-only collapse.

- [ ] T016 — Collect unresolved impact boundaries.
  - preserve `external` and `ambiguous` edges/candidates;
  - no network lookup or preferred-candidate heuristic.

- [ ] T017 — Normalize before/after evidence without losing side-only state.
  - preserve removed-before and added-after evidence;
  - use `both` only for exact normalized identical evidence.

- [ ] T018 — Prove library invariants and byte determinism.
  - direct/transitive, cycles, add/remove, multi-version, ambiguous/external, tie-breaking, permutations, repeat bytes.

## Stack B — shipped `commandf impact`

- [ ] T020 — Add the `commandf impact` CLI surface.
  - package + explicit before/after lock/cache + JSON output.

- [ ] T021 — Enforce CLI fail-closed boundaries.
  - schema-v1/unsupported context refusal, missing/corrupt cache refusal, bounded malformed-input handling, sanitized diagnostics.

- [ ] T022 — Add end-to-end impact fixtures.
  - direct/transitive, removed/added target, multi-version package exposure, ambiguous/external boundaries, no invented severity.

- [ ] T023 — Add dedicated `cf12-impact-proof` workflow.
  - digest-pinned Rust 1.97.1 container, immutable action SHAs, complete path filters, repeat-byte comparison, clean tree, retained artifact.

- [ ] T024 — Record deterministic CLI proof identity.
  - `CF12_IMPACT_SHA256=<sha256>`, exact head/tree/run/job, artifact id/digest.

- [ ] T025 — Prove existing command behavior remains unchanged.
  - no regression for diff/classify/check/context/terminology/oracle/source-map/annotations;
  - no compatibility-authority or lock-schema change.

## Regression, review, and convergence

- [ ] T040 — Run mandatory workspace gates on the exact final implementation head.
- [ ] T041 — Preserve applicable repository workflows including `ci`, `cf06-oracle`, path-triggered CF-11/CF-11G proofs, `cf12-impact-proof`, real FHIR, and security regressions.
- [ ] T042 — Independent implementation review; disposition every substantive returned finding and record reviewer unavailability without invented PASS.
- [ ] T043 — Run CF-12 convergence; record final heads/runs, `CF12_IMPACT_SHA256`, artifact digest, coverage limits, and every remaining gap/deferral.

## Hard sequencing rules

1. This planning PR MUST merge cleanly before T010 implementation starts.
2. T010 precedes traversal tasks; T011/T012 precede T013; T013 precedes T014; T015/T016/T017 precede T018.
3. T010–T018 precede user-visible CLI shipping; T020–T025 precede final convergence.
4. No task may traverse CF-11G `external` or `ambiguous` evidence as resolved.
5. No task may convert reachability into compatibility severity without an existing explicit CF-04/CF-05 authority contract.
6. No CF-06 production pin, frozen CF-10 case, graph database, AI/model authority, or network resolution is authorized by CF-12 V1.
