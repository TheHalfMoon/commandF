# CF-12 Tasks — Deterministic Impact Analysis

Status: PLANNING_CANDIDATE — no CF-12 implementation is authorized by task text alone; planning must pass exact-head review and merge first.

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

- [ ] T004 — Close planning consistency and independent review.
  - `spec.md`, `plan.md`, `tasks.md`, and `consistency.md` contain no unresolved contradiction;
  - CodeRabbit reviewed when available;
  - Qodo reviewed when connected/available;
  - every substantive planning finding dispositioned before implementation branch creation.

## Stack A — library model and deterministic traversal

- [ ] T010 — Add library-owned CF-12 impact report schema v1.
  - subject package identity;
  - before/after evidence identities;
  - seeds;
  - artifact impacts;
  - package impacts;
  - unresolved boundaries;
  - extraction coverage;
  - stable canonical JSON serialization.

- [ ] T011 — Build deterministic change seeds from the existing package structural-diff pipeline.
  - added canonical artifacts;
  - removed canonical artifacts;
  - modified canonical artifacts with non-empty structural delta;
  - preserve exact side-specific artifact/package/digest identity;
  - do not implement a second diff engine.

- [ ] T012 — Build side-specific reverse indexes over resolved CF-11G canonical-reference edges.
  - only `resolved` edges are traversable;
  - exact artifact identities remain version-aware;
  - external/ambiguous states excluded from traversal.

- [ ] T013 — Implement deterministic transitive reverse artifact traversal.
  - direct dependents;
  - transitive dependents;
  - cycle termination;
  - exact `(side, impacted identity, seed identity)` visited state;
  - canonical shortest evidence path.

- [ ] T014 — Implement equal-length path tie-breaking.
  - minimum edge count first;
  - lexicographically smallest stable exact path among equal lengths;
  - traversal/hash-map order cannot affect output.

- [ ] T015 — Implement exact reverse package-dependency exposure.
  - consume schema-v2 resolved dependency edges;
  - preserve exact package name/version/digest identity;
  - preserve declared dependency constraints;
  - never collapse same-name package versions.

- [ ] T016 — Collect unresolved impact boundaries.
  - retain `external` edges originating from seeds/impacted artifacts;
  - retain `ambiguous` edges and all sorted candidates;
  - no network lookup;
  - no preferred-candidate heuristic.

- [ ] T017 — Normalize before/after evidence without losing side-only state.
  - removed before-only dependencies remain visible;
  - added after-only dependencies remain visible;
  - use `both` only for exact normalized identical evidence.

- [ ] T018 — Prove library invariants and byte determinism.
  - direct + transitive fixtures;
  - cycles;
  - added/removed target fixtures;
  - multi-version package fixture;
  - ambiguous/external boundaries;
  - equal-length path tie fixture;
  - input-order permutation fixture;
  - repeat serialization byte identity.

## Stack B — shipped `commandf impact`

- [ ] T020 — Add the `commandf impact` CLI surface.
  - `package` positional argument;
  - `--before-lock`;
  - `--before-cache`;
  - `--after-lock`;
  - `--after-cache`;
  - `--format json`;
  - canonical JSON to stdout.

- [ ] T021 — Enforce CLI fail-closed boundaries.
  - unsupported/schema-v1 Context Graph input refuses safely;
  - missing cache archive refuses;
  - corrupt archive digest refuses;
  - malformed required artifact input refuses under existing bounded policy;
  - runtime diagnostics remain sanitized/bounded.

- [ ] T022 — Add end-to-end impact fixtures.
  - direct artifact impact;
  - transitive artifact impact;
  - removed-target before-side evidence;
  - added-target after-side evidence;
  - exact multi-version package exposure;
  - ambiguous and external boundaries;
  - reachability without invented compatibility severity.

- [ ] T023 — Add a dedicated `cf12-impact-proof` workflow.
  - immutable digest-pinned Rust 1.97.1 container;
  - immutable action SHAs;
  - `persist-credentials: false`;
  - complete relevant path filters;
  - repeated CLI output byte comparison;
  - clean-tree assertion;
  - retained checksum artifact.

- [ ] T024 — Record deterministic CLI proof identity.
  - emit `CF12_IMPACT_SHA256=<sha256>`;
  - record exact head/tree/run/job;
  - record artifact id and GitHub artifact digest.

- [ ] T025 — Prove existing command behavior remains unchanged.
  - no CLI regression for `diff`, `classify`, `check`, `context`, terminology, oracle, source-map, annotations;
  - no new compatibility authority;
  - no lock schema change.

## Regression, review, and convergence

- [ ] T040 — Run mandatory workspace gates on the exact final implementation head.
  - `cargo fmt --all -- --check`;
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  - `cargo test --workspace --all-features`.

- [ ] T041 — Preserve applicable repository workflows.
  - `ci`;
  - `cf06-oracle`;
  - `cf11-multi-version-proof` where path-triggered;
  - `cf11g-context-proof` where path-triggered;
  - `cf12-impact-proof`;
  - real FHIR and CF-08/CF-09 security regressions through `ci`.

- [ ] T042 — Independent implementation review.
  - CodeRabbit when available;
  - Qodo when connected/available;
  - every substantive finding fixed or rejected against the frozen contract with evidence;
  - reviewer unavailability recorded without invented PASS.

- [ ] T043 — Run CF-12 convergence pass.
  - record final implementation head/tree/run identities;
  - record `CF12_IMPACT_SHA256` and artifact digest;
  - record unresolved-boundary behavior and coverage limits;
  - append any remaining gap as a task or explicit deferral;
  - merge only when exact final candidate state is green and review-clean.

## Hard sequencing rules

1. T004 MUST close planning before T010 implementation starts.
2. T010 precedes traversal tasks because output semantics must be frozen before algorithms.
3. T011/T012 precede T013 because traversal consumes deterministic seeds and reverse indexes.
4. T013 precedes T014 because tie-breaking normalizes proven paths.
5. T015/T016/T017 precede T018 full library proof.
6. T010–T018 precede user-visible CLI shipping.
7. T020–T025 precede final regression/review/convergence.
8. No task may traverse CF-11G `external` or `ambiguous` evidence as a resolved edge.
9. No task may convert reachability into compatibility severity without consuming an existing explicit CF-04/CF-05 authority contract.
10. No CF-06 production pin, frozen CF-10 case, graph database, AI/model authority, or network resolution is authorized by CF-12 V1.
