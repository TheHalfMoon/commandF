# CF-12 Tasks — Deterministic Impact Analysis

Status: CONVERGENCE_CANDIDATE — implementation and exact-head evidence are complete; canonical closure requires this docs-only closeout PR to qualify and merge.

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
  - canonical planning merge: `cefa5e4a56041bf88e833844a318b170e7e7ae83`.

## Stack A — library model and deterministic traversal

Stack A merged through PR `#26` at merge commit `d46591f0f7224d49fda0d89a6a79cc418fba534e` from exact qualified head `9fa948cb2ad0110cd4288c330a5bc8b977472418`.

- [x] T010 — Add library-owned CF-12 impact report schema v1.
  - subject/evidence identity, seeds, artifact impacts, package impacts, unresolved boundaries, coverage, canonical JSON.

- [x] T011 — Build deterministic change seeds from the existing package structural-diff pipeline.
  - added, removed, and modified canonical artifacts with exact side-specific identity;
  - no second diff engine.

- [x] T012 — Build side-specific reverse indexes over resolved CF-11G canonical-reference edges.
  - only `resolved` edges are traversable;
  - exact artifact identities remain version-aware.

- [x] T013 — Implement deterministic transitive reverse artifact traversal.
  - direct/transitive dependents, cycle termination, exact visited state, canonical shortest path.

- [x] T014 — Implement equal-length path tie-breaking.
  - minimum edge count, then lexicographically smallest stable exact path.

- [x] T015 — Implement exact reverse package-dependency exposure.
  - schema-v2 exact edges, exact version/digest identity, declared constraints, no name-only collapse.

- [x] T016 — Collect unresolved impact boundaries.
  - preserve `external` and `ambiguous` edges/candidates;
  - no network lookup or preferred-candidate heuristic.

- [x] T017 — Normalize before/after evidence without losing side-only state.
  - preserve removed-before and added-after evidence;
  - use `both` only for exact normalized identical evidence.

- [x] T018 — Prove library invariants and byte determinism.
  - direct/transitive, cycles, add/remove, multi-version, ambiguous/external, tie-breaking, permutations, repeat bytes.

## Stack B — shipped `commandf impact`

Stack B merged through PR `#27` at merge commit `9e462cbb5c0bd05cf2219e2283f09bfbc8a51720` from exact qualified head `6d8e22b1d8c999256692052d473ba3c27effc972`.

- [x] T020 — Add the `commandf impact` CLI surface.
  - package + explicit before/after lock/cache + JSON output.

- [x] T021 — Enforce CLI fail-closed boundaries.
  - schema-v1/unsupported context refusal, missing/corrupt cache refusal, bounded malformed-input handling, sanitized diagnostics.

- [x] T022 — Add end-to-end impact fixtures.
  - direct/transitive, removed/added target, multi-version package exposure, ambiguous/external boundaries, no invented severity;
  - final CLI-level reverse package-exposure evidence merged through PR `#28` at `71c5c4372a829ca6b26846acad0a8ded44f1e1ba`.

- [x] T023 — Add dedicated `cf12-impact-proof` workflow.
  - digest-pinned Rust 1.97.1 container, immutable action SHAs, complete path filters, repeat-byte comparison, clean tree, retained artifact.

- [x] T024 — Record deterministic CLI proof identity.
  - final implementation/evidence head: `c874c8c665a053d3022b6592a6dcf2a9f9c88349`;
  - tree: `0ab82f0d8fb19d88ddcb0af1fbc5a4cd8535b765`;
  - `cf12-impact-proof` run: `32942924956`;
  - job: `98097504274`;
  - artifact: `9597183002`;
  - artifact digest: `sha256:1cf1fc14c84f35a84c00c01ed2cc475a0c310e374dd14f3105ae4ac08bb79c1f`;
  - `CF12_IMPACT_SHA256=e75f54cefc9af93819fb11b437418c04f6fe8036bef3e4be1ccf6523170c84b1`.

- [x] T025 — Prove existing command behavior remains unchanged.
  - exact-head `ci` run `32942924918` passed format, Clippy, workspace tests, CF-08/CF-09 security regressions, real FHIR command smoke, terminology smoke, and Action source-map smoke;
  - exact-head `cf06-oracle` run `32942924969`, `cf11-multi-version-proof` run `32942924926`, and `cf11g-context-proof` run `32942924928` all succeeded;
  - no compatibility-authority, CF-06 identity, frozen CF-10 corpus, dependency, or lock-schema change was introduced.

## Regression, review, and convergence

- [x] T040 — Run mandatory workspace gates on the exact final implementation head.
  - `ci` run `32942924918`, job `98097504281`, exact head `c874c8c665a053d3022b6592a6dcf2a9f9c88349`: SUCCESS.

- [x] T041 — Preserve applicable repository workflows including `ci`, `cf06-oracle`, path-triggered CF-11/CF-11G proofs, `cf12-impact-proof`, real FHIR, and security regressions.
  - all five configured workflows triggered on the final implementation/evidence head and succeeded;
  - integrated real-FHIR and security regressions inside `ci` also succeeded.

- [x] T042 — Independent implementation review; disposition every substantive returned finding and record reviewer unavailability without invented PASS.
  - Qodo returned one substantive correctness finding on PR `#28`; it was fixed in `6210db22c08a5e9a0b6e9f9b5c7653b771da0795` and the thread is resolved/outdated on the final head;
  - CodeRabbit commit status on the final head is `success` with description `Review rate limited`; no stronger CodeRabbit review claim is made;
  - unresolved substantive findings: `0`.

- [x] T043 — Run CF-12 convergence; record final heads/runs, `CF12_IMPACT_SHA256`, artifact digest, coverage limits, and every remaining gap/deferral.
  - immutable evidence and V1 coverage/deferral boundaries are recorded in `convergence.md`;
  - this task record remains a convergence candidate until the docs-only closeout PR itself passes its path-applicable exact-head gates/review and merges.

## Hard sequencing rules

1. The planning PR merged before T010 implementation started.
2. T010 preceded traversal tasks; T011/T012 preceded T013; T013 preceded T014; T015/T016/T017 preceded T018.
3. T010–T018 preceded user-visible CLI shipping; T020–T025 preceded final convergence.
4. CF-12 does not traverse CF-11G `external` or `ambiguous` evidence as resolved.
5. CF-12 does not convert reachability into compatibility severity without existing explicit CF-04/CF-05 authority.
6. CF-12 V1 does not authorize a CF-06 production pin change, frozen CF-10 case mutation, graph database, AI/model authority, or network resolution.
