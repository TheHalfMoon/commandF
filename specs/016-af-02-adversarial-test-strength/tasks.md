# AF-02 Tasks — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Task-state rules

- A checkbox is completed only by exact repository/GitHub evidence, not by intent.
- Any head mutation invalidates prior exact-head qualification unless the affected evidence is explicitly content-independent.
- AF-02 adds test-strength evidence; it does not replace canonical `cargo test` or existing CF/AF authority.
- Product semantics, CF-06 production oracle identity, CF-10 frozen corpus, and AF-01 live source-control policy remain frozen unless a separately authorized unit changes them.
- Stochastic fuzz discovery is not a deterministic PASS signal. Deterministic replay/property/mutation/coverage policy evidence and stochastic observations are recorded separately.
- No implementation task begins until T006 is canonical.
- No public product API may be added solely for fuzz harness convenience.
- No PHI or real patient-instance fixture is permitted.

## Phase 0 — planning and authority

- [ ] **T001** Record canonical AF-02 planning entry identity: `main=2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1`, tree `804ce63c15edb501574bd4aba9a9aadc5bfb07f3`, AF-01 `CLOSED_CANONICAL`.
- [ ] **T002** Inventory current high-risk input/evidence boundaries in canonical `commandf-pkg`, including archive/package ingestion, Lockfile V1/V2, source mapping/path validation, context graph/canonical references, compatibility/check/gate retained evidence, and deterministic serializers.
- [ ] **T003** Verify and record exact initial upstream tool identities/licensing/MSRV for cargo-fuzz, libfuzzer-sys/arbitrary, proptest, cargo-nextest, cargo-llvm-cov, and cargo-mutants; add/update commandF donor/provenance record.
- [ ] **T004** Author AF-02 `spec.md`, `plan.md`, `tasks.md`, and `consistency.md` preserving CF-14/15/16 identities, CF-06/CF-10 authority, AF-01 required-check/live-policy authority, no-PHI boundary, and deterministic-vs-stochastic evidence separation.
- [ ] **T005** Run consistency analysis across constitution, AGENTS, master architecture, plan index, assurance program, AF-01 closeout/handoff, AF-02 package, current source tree, donor policy, and live GitHub policy; resolve contradictions before planning merge.
- [ ] **T006** Planning gate: exact final planning head passes every path-applicable existing workflow, Qodo/CodeRabbit exact-head review truth is obtained and every substantive finding is dispositioned, zero unresolved substantive review threads remain, planning PR merges from exact qualified head, and post-merge canonical main/tree plus AF-01 live rulesets are re-read. Only then is AF-02 implementation authorized.

## Phase 1 / Stack A — surface policy, fuzz/property foundation, regression corpus

Depends on canonical T006.

- [ ] **T010** Re-read canonical main and perform an implementation-time reachable-entrypoint inventory for every FR-001 critical surface. For each surface record exact product source paths, existing public/internal test seam, raw-vs-structured fuzz suitability, property candidates, mutation scope, coverage-critical scope, and corpus-replay path.
- [ ] **T011** Add a machine-readable AF-02 critical-surface policy with stable schema, unique IDs, evidence modes, source paths, entrypoints/test seams, input/case bounds, corpus paths, mutation scope, and coverage-critical classification.
- [ ] **T012** Add repository-owned validation/tests for the AF-02 surface policy: duplicate/missing surface IDs fail; missing source/target/corpus references fail; every initial critical surface has at least one required adversarial evidence mode; later parser/validator surfaces cannot silently evade classification according to the frozen discovery rule.
- [ ] **T013** Create isolated `fuzz/` workspace using `cargo-fuzz 0.13.2` / upstream commit `984c861c8dfea28055254c5f1d2659ab2cd63f76`, exact fuzz-only crate versions, and dated `nightly-2026-08-25`; retain crates.io checksums/toolchain identity without changing normal workspace Rust `1.97.1`.
- [ ] **T014** Implement bounded raw package/archive fuzz target through an existing product entrypoint. Cover malformed/truncated gzip/tar/JSON, manifest/path/header/boundary cases, no panic, no network, bounded input size/work, and deterministic corpus replay classification.
- [ ] **T015** Implement Lockfile adversarial coverage: raw JSON fuzzing plus structure-aware generation/property tests for V1/V2 schema, canonical round-trip bytes, sorted/dedup invariants, dependency-edge completeness/constraint matching, and rejection of hostile non-canonical persisted V2 evidence.
- [ ] **T016** Implement source-map adversarial coverage using the narrowest existing reachable seam: portable-path traversal/prefix/empty-component rejection, duplicate output identities, line-range validation, source-root containment, report-size/index bounds, and isolated temporary filesystem behavior. Use colocated private-property tests or a non-public internal seam if necessary; do not add public API only for testing.
- [ ] **T017** Implement context-graph/canonical-reference structure-aware/property coverage: equivalent input-order permutations, sorted/dedup outputs, external/resolved/ambiguous status, empty target/version rejection, and bounded synthetic verified-cache/package fixtures.
- [ ] **T018** Implement compatibility/check/gate property/adversarial coverage: fingerprint JSON key-order invariance, evaluate-then-validate consistency, tampered retained-evidence rejection, baseline/suppression order/set equivalence, deterministic serializer repeatability, and false-PASS-sensitive decision predicates.
- [ ] **T019** Add minimized regression corpus policy/manifest with stable scenario IDs, SHA-256 digests, provenance classification, affected surface, expected result, default `256 KiB` per promoted fixture limit, orphan/missing/digest mismatch checks, and deterministic replay tests. Discovery corpora remain generated artifacts, not automatically committed.
- [ ] **T020** Add AF-02 corpus-promotion self-tests demonstrating discovery/counterexample -> minimize/shrink -> manifest/digest -> deterministic regression. A fix cannot close without the promoted deterministic reproducer.
- [ ] **T021** Add Stack A CI/workflow coverage using AF-01-compliant full-SHA Actions, least permissions, credentialless checkout, fixed runner policy, bounded timeouts, exact fuzz toolchain installation/identity, fuzz-target build, deterministic committed-corpus replay, and property tests. Stochastic no-crash runs are not reported as correctness PASS.
- [ ] **T022** Run mandatory workspace gates and every path-applicable existing product/oracle/AF-01 workflow on exact Stack A head. Confirm no product semantic diff beyond any minimal reviewed internal test-seam refactor.
- [ ] **T023** Request CodeRabbit and Qodo on exact Stack A head; disposition every substantive finding and require zero unresolved material review threads.
- [ ] **T024** Merge Stack A only from its exact qualified head; record merge SHA/tree, post-merge applicable proof results, and re-read live AF-01 rulesets before beginning Stack B.

## Phase 2 / Stack B — flaky-as-failure and measured coverage floors

Depends on canonical T024.

- [ ] **T030** Install/pin `cargo-nextest 0.9.143` resolved to upstream commit `60fa45f638ffc3f35e74afa65737f45fcd32db2a`; retain exact binary/install identity used by CI.
- [ ] **T031** Add `.config/nextest.toml` CI profile with `retries = 2` and `flaky-result = "fail"`; measure current suite durations before freezing slow-timeout/termination policy, then commit explicit bounded timeout semantics.
- [ ] **T032** Add an isolated AF-02 nextest self-test proving a test that fails initially and passes on retry still makes the AF-02 gate fail, without introducing a permanently flaky test to canonical product `cargo test`.
- [ ] **T033** Keep `cargo test --workspace --all-features` independently mandatory and document/test the authority split: nextest is additive flaky/retry evidence and does not substitute for cargo-test-only/doctest behavior.
- [ ] **T034** Install/pin `cargo-llvm-cov 0.9.0` resolved to upstream commit `be59056988acd54c7f984b7c85643daea3711b29`; retain exact compiler/tool/source identities.
- [ ] **T035** Measure the first exact canonical AF-02 coverage baseline before selecting thresholds. Retain workspace line/function/region observations and critical-module line observations where reliable, plus explicit path-specific exclusions.
- [ ] **T036** Freeze initial coverage floors from measured baseline using the canonical rule defined by the plan (integer floor of exact measured percentage unless measurement proves a more stable reviewed rule is needed). Do not choose vanity targets or exclude low-coverage product code to raise scores.
- [ ] **T037** Implement coverage-policy validation: floor breach fails; missing/unknown critical source coverage fails closed; exclusions are explicit/bounded; same-candidate floor lowering solely to regain green is prohibited and must require an explicit reviewed policy-change path/rationale.
- [ ] **T038** Add Stack B CI lanes for nextest flaky-as-failure and coverage, with bounded timeouts and retained machine-readable evidence. Keep expensive/external product oracle behavior appropriately separated while preserving all existing path-applicable gates.
- [ ] **T039** Run mandatory workspace gates, Stack A adversarial replay/property evidence, and every path-applicable existing product/oracle/AF-01 workflow on exact Stack B head.
- [ ] **T040** Request CodeRabbit and Qodo on exact Stack B head; disposition every substantive finding and require zero unresolved material review threads.
- [ ] **T041** Merge Stack B only from exact qualified head; record canonical merge/main/tree, coverage baseline/floors, nextest configuration/identity, and post-merge proof applicability before beginning Stack C.

## Phase 3 / Stack C — mutation adequacy and exact-head AF-02 proof

Depends on canonical T041.

- [ ] **T050** Install/pin `cargo-mutants 27.1.0` resolved to upstream commit `8ab1dc786a1f61a4e370416cc6c68b81a704e917`; retain exact binary/install identity.
- [ ] **T051** Inventory candidate mutations on canonical Stack B source and freeze the required targeted mutation scope, prioritizing false-PASS decision predicates, Lockfile validation, source-map escape/line validation, context resolution/order, archive limits/acceptance, and retained-evidence validators.
- [ ] **T052** Define checked-in mutation result/waiver schema distinguishing `KILLED`, `SURVIVED`, `TIMEOUT`, `UNVIABLE_OR_BUILD_FAILURE`, and `WAIVED_EQUIVALENT_OR_OUT_OF_SCOPE`. Waivers require exact tool/source/mutation identity, rationale, compensating evidence, revisit condition, and removal condition.
- [ ] **T053** Add mutation-policy self-tests: a deliberately under-tested mutation fixture remains `SURVIVED` and fails the gate; strengthening the fixture test kills it; timeout/build-failure are not counted as killed; waiver with missing required metadata fails closed.
- [ ] **T054** Run the required mutation target set and strengthen product tests/property/corpus cases until every required survivor is killed or narrowly reviewed/waived. No broad critical-module exclusion and no unclassified survivor may remain.
- [ ] **T055** Add/complete bounded fuzz discovery workflow lane using exact source/tool/toolchain/corpus identities. Retain crashes and campaign bounds; classify no-crash outcome as `NO_CRASH_OBSERVED_WITHIN_BOUND`, never as timeless correctness proof.
- [ ] **T056** Implement `.github/workflows/af02-adversarial-proof.yml` or equivalent retained proof. Stable evidence includes exact source/tree, AF-02 policy/spec hashes, fuzz target/toolchain identities, corpus manifest/digests/replay, property configuration, nextest no-flake truth, coverage baseline/floors/results, mutation classifications/waivers, stochastic observations, and deterministic `AF02_ADVERSARIAL_SHA256`.
- [ ] **T057** Add proof tests for repeated normalized deterministic summary equality; source/tree/policy mismatch; missing corpus/property/nextest/coverage/mutation evidence; stochastic fields excluded from deterministic identity; floor breach; retry-pass; corpus replay failure; and unclassified mutation survivor.
- [ ] **T058** Reconcile AF-02 workflow paths/configs with AF-01 workflow-trust/assurance coverage so future AF-02 policy/tool/workflow mutation cannot bypass existing security gates.
- [ ] **T059** Decide whether any AF-02 check should become a live `main` required check. Default is **no live ruleset change**. If a change is proposed, add a separate universal-terminal topology proof, checked-in ruleset intent update, negative docs-only/nonmatching PR proof, administrator application, and live read-back before closure relies on it.
- [ ] **T060** Run exact-head AF-02 proof, mandatory workspace gates, Stack A/B gates, and every path-applicable existing product/oracle/AF-01 workflow; retain run/job/artifact IDs, digests, tool identities, corpus identities, coverage/mutation results, and source SHA/tree.
- [ ] **T061** Obtain exact-head Qodo/CodeRabbit review; require zero unresolved substantive findings and explicitly review false-PASS risk, stochastic-evidence wording, mutation waivers, coverage floors, and AF-01 authority preservation.
- [ ] **T062** Merge Stack C only from exact qualified head; verify post-merge canonical main/tree, AF-02 proof applicability, no AF-01 ruleset drift, and no product/oracle semantic authority drift.

## Phase 4 — convergence and canonical closeout

Depends on canonical T062.

- [ ] **T070** Re-read AF-02 spec/plan/tasks/consistency, constitution, AGENTS, master architecture, plan index, assurance program, donor record, implementation tree, exact tool identities, AF-01 closeout/live rulesets, and active repository/PR state; reconcile drift.
- [ ] **T071** Create `convergence.md` recording planning/Stack A/B/C identities, target/surface inventory, toolchain/tool/checksum identities, property configuration, corpus scenarios/digests, nextest no-flake truth, coverage baseline/floors, mutation results/waivers, stochastic fuzz observations, proof artifact/digest, reviewer dispositions, limits, and deferrals.
- [ ] **T072** Confirm semantic diff from pre-AF-02 canonical base contains no unauthorized CF semantic change. Any incidental product refactor must be proven behavior-preserving with exact regression evidence and public API unchanged unless separately authorized.
- [ ] **T073** Confirm every discovered AF-02 crash/invariant used in closure has a minimized deterministic regression and every required mutation survivor is killed/waived with exact evidence; no flaky retry-pass or coverage-floor breach remains.
- [ ] **T074** Record remaining work under AF-03/AF-04 and future CF units. Do not claim portability/release/performance/CF-14 completion from AF-02 evidence.
- [ ] **T075** Exact convergence head receives all path-applicable CI/AF-02 proof and independent review truth with zero unresolved substantive findings.
- [ ] **T076** Merge convergence only from exact qualified head and verify canonical post-merge main/tree, post-merge AF-02 proof applicability where triggered, and unchanged live AF-01 source-control policy.
- [ ] **T077** Create final docs-only closeout candidate that reconciles task state without circularly embedding future temporal identifiers. Qualify its exact head with required/path-applicable CI and fresh Qodo/CodeRabbit review; merge with expected-head guard.
- [ ] **T078** Mark `AF-02=CLOSED_CANONICAL` only after T077 merge and post-merge canonical main/tree, AF-02 proof, corpus/coverage/mutation/flaky truth, and live AF-01 policy are re-read and all closure criteria remain satisfied.

## AF-03 handoff retained, not authorized by AF-02 implementation

- Linux/Windows/macOS qualification;
- explicit MSRV proof;
- public API/SemVer compatibility;
- deterministic release inventory;
- SBOM;
- SLSA/GitHub artifact provenance;
- Sigstore/offline verification where adopted;
- stable release verification.

## AF-04 handoff retained, not authorized by AF-02 implementation

- performance/resource benchmark corpus;
- measured regression budgets;
- large package/graph stress scenarios;
- external registry/oracle sentinel separation;
- trend evidence reusable by future commandF Bench.

## CF handoff retained

AF-02 does not implement CF-14/15/16. A future CF-14 parser/instance-data boundary must enter the canonical AF-02 surface/property/fuzz inventory before CF-14 can close canonically.
