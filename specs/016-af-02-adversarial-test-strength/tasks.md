# AF-02 Tasks — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Task-state rules

- A checkbox is complete only with exact repository/GitHub evidence.
- Head mutation invalidates prior exact-head CI/review qualification unless the evidence is explicitly content-independent.
- `evidence-contracts.md` is normative for schemas, algorithms, resource limits, authority snapshots, tool provenance, anti-forgery, and design-freeze ordering.
- Canonical `cargo test --workspace --all-features --locked` remains independently mandatory.
- AF-01 live source-control policy, CF-06 production oracle authority, CF-10 frozen corpus, and CF product semantics remain frozen unless separately authorized work has already changed them canonically.
- Stochastic fuzz discovery is never a deterministic PASS signal.
- No implementation task begins until T006 is canonical.
- Every Stack design-freeze PR must merge before dependent implementation code.
- No public product API may be added solely for fuzzing convenience.
- Synthetic/public redistributable non-PHI fixture provenance only.

## Phase 0 — planning and authority

- [x] **T001** Record canonical planning entry: `main=2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1`, tree `804ce63c15edb501574bd4aba9a9aadc5bfb07f3`, AF-01 `CLOSED_CANONICAL`.
- [x] **T002** Inventory current critical boundaries: package acquisition/cache/archive/manifest/resource ingestion; Lockfile V1/V2; source-map/path/filesystem containment; context graph/canonical references; compatibility/check/gate/fingerprint/suppression retained evidence; deterministic serializers.
- [x] **T003** Research and pin initial donor/tool source identities for cargo-fuzz, cargo-mutants, cargo-llvm-cov, cargo-nextest and proptest; record test-only/fuzz-only versions and donor licensing.
- [x] **T004** Author AF-02 Spec Kit planning package plus donor record without Rust/workflow/dependency/live-policy mutation.
- [ ] **T005** Resolve exact-head planning-review findings by making `evidence-contracts.md` normative and reconciling `spec.md`, `plan.md`, `tasks.md`, `consistency.md`, and donor provenance. Obtain fresh reviewer confirmation that no material design choice remains hidden before implementation.
- [ ] **T006** Planning gate: the exact final planning head passes every path-applicable existing workflow; fresh Qodo and CodeRabbit exact-head review truth is obtained; every substantive finding is dispositioned; zero unresolved substantive review threads remain; PR #54 merges only from the exact qualified head using an expected-head guard; canonical post-merge main/tree and both live AF-01 rulesets are re-read. Only then is AF-02 implementation authority granted.

## Stack A0 — design freeze: surface, resource, tool, property, corpus, no-PHI

Depends on canonical T006. This Stack is a planning/contract implementation candidate only; dependent fuzz/property code waits for T019.

- [ ] **T010** Re-read canonical main and independently derive `commandf.af02-authority-baseline/v1`: live AF-01 ruleset semantic projections; CF-06 production oracle identity; CF-10 unchanged case membership and retained evidence identity. Candidate-edited baseline is not self-authorizing.
- [ ] **T011** Implement `commandf.af02-surface-policy/v1` with frozen source roots, boundary categories, critical surfaces, accepted outcome classes, evidence modes, resource profiles, corpus namespaces, mutation/coverage scopes, and independent model/oracle fields.
- [ ] **T012** Implement deterministic production-boundary discovery for parser/deserializer, archive/compression, filesystem/path, network/acquisition, cache/persistence, and subprocess seams. Fail on any newly discovered unclassified boundary.
- [ ] **T013** Add surface-policy regressions proving duplicate/missing IDs fail, stale source/seam/corpus references fail, unclassified discovered boundaries fail, and reviewed exclusions require complete narrow metadata. Include package acquisition/cache boundaries explicitly.
- [ ] **T014** Implement `commandf.af02-resource-policy/v1` from the normative contract, including campaign/time/execution/input/memory/CPU/PID/tmpfs/decompressed-work/temp-file/subprocess/artifact/corpus/retention/offline fields and validation.
- [ ] **T015** Prove effective deterministic offline execution: immutable acquisition phase first, then `CARGO_NET_OFFLINE=true` plus OS/container network denial with explicit resource controls. Missing effective offline enforcement when required fails.
- [ ] **T016** Implement `commandf.af02-tool-lock/v1` acquisition procedure. Executable tools may use only `LOCKED_GIT_REV_SOURCE_BUILD` or `IMMUTABLE_RELEASE_ASSET_WITH_SHA256`; retain exact source/release digest, install command, executable SHA-256, version output, compiler/cargo/target/features. Registry packages retain exact crates.io checksums.
- [ ] **T017** Freeze property/generator schemas and the independent archive, Lockfile, source-map/path, context-graph and quality-gate/fingerprint test-owned models defined by `evidence-contracts.md`.
- [ ] **T018** Implement `commandf.af02-corpus/v1`, stable scenario namespace, raw-byte SHA-256, <=256 KiB default fixture and <=8 MiB aggregate policy, assertion/replay registry binding, allowed provenance classes, no-PHI provenance validation, and safe opaque fuzz-artifact retention rules.
- [ ] **T019** Qualify Stack A0 exact head with mandatory existing workflows, authority read-back, fresh Qodo/CodeRabbit and zero unresolved substantive threads; merge only from exact qualified head and verify post-merge authority. Only then may Stack A implementation begin.

## Stack A1 — fuzz/property foundation and deterministic regression corpus

Depends on canonical T019.

- [ ] **T020** Create isolated `fuzz/` workspace using cargo-fuzz 0.13.2 at commit `984c861c8dfea28055254c5f1d2659ab2cd63f76`, `libfuzzer-sys =0.4.13`, `arbitrary =1.4.2`, and `nightly-2026-08-25`; verify/retain exact tool-lock and registry checksums; product Rust remains 1.97.1.
- [ ] **T021** Implement bounded raw archive/package fuzz target through an existing product entrypoint. Cover malformed/truncated compression/tar/JSON, path/header/boundary cases and expected normalized result classes without network or unbounded product maxima.
- [ ] **T022** Implement Lockfile raw and structured adversarial coverage using the independent test-owned model: schema/canonical round-trip/order/dedup/dependency-edge completeness/constraint satisfaction and hostile persisted-evidence rejection.
- [ ] **T023** Implement source-map/path adversarial coverage through the narrowest existing/internal test seam: portable traversal/prefix/empty-component rejection, duplicate outputs, line ranges, source containment, report/index bounds and independent path model.
- [ ] **T024** Implement context-graph/canonical-reference structured/property coverage against the independent synthetic canonical-index model for permutation invariance, external/resolved/ambiguous outcomes, version/fragment/empty rejection, and bounded verified-cache fixtures.
- [ ] **T025** Implement compatibility/check/gate/fingerprint/suppression adversarial properties against independent set/truth-table and canonical-key models: evaluate-then-validate, tamper rejection, baseline/suppression order equivalence, decision predicates, serializer repeatability.
- [ ] **T026** Implement deterministic corpus promotion pipeline: discovery/counterexample -> minimize/shrink -> safe raw fixture -> manifest/digest -> assertion/replay binding -> deterministic normalized outcome. A fix cannot close without the promoted reproducer.
- [ ] **T027** Add AF-02 Stack A CI: verified tool acquisition, deterministic offline surface/corpus/property qualification, fuzz-target build, canonical cargo test, bounded artifact retention, and scheduled/manual stochastic discovery whose no-crash result is only `NO_CRASH_OBSERVED_WITHIN_BOUND`.
- [ ] **T028** Run exact-head Stack A deterministic gates plus every path-applicable product/oracle/AF-01 workflow; prove product public API and semantics unchanged except any reviewed behavior-preserving internal test seam.
- [ ] **T029** Obtain fresh Qodo/CodeRabbit with zero unresolved substantive findings; merge Stack A only from exact qualified head and re-read main/tree/live AF-01/CF-06/CF-10 authority before Stack B0.

## Stack B0 — design freeze: nextest and coverage

Depends on canonical T029. No nextest/coverage enforcement implementation until T036 is canonical.

- [ ] **T030** Pin/verify cargo-nextest 0.9.143 at commit `60fa45f638ffc3f35e74afa65737f45fcd32db2a` through `commandf.af02-tool-lock/v1`.
- [ ] **T031** Freeze `.config/nextest.toml` CI semantics: `retries=2`, `flaky-result="fail"`, `slow-timeout={period="60s", terminate-after=2}` and invocation authority `--retries 2 --flaky-result fail` so weaker per-test overrides cannot make AF-02 green.
- [ ] **T032** Freeze isolated non-workspace deterministic retry fixture using an AF-02-owned state file: first attempt fails after atomic create, retry passes, normalized result `FLAKY_RETRY_PASS`, overall nextest process must remain non-zero; no wall clock/RNG/network dependency.
- [ ] **T033** Pin/verify cargo-llvm-cov 0.9.0 at commit `be59056988acd54c7f984b7c85643daea3711b29` and freeze pre-measurement descriptor: Linux/x86_64, Rust 1.97.1 + llvm-tools-preview, `cargo llvm-cov --workspace --all-features --locked --json`, production `crates/*/src/**`, explicit non-product exclusions only.
- [ ] **T034** Freeze normalized coverage descriptor schema, raw-report retention, workspace/critical-surface metric rules, missing-source fail-closed behavior and dedicated rebaseline/floor-reduction policy. A descriptor/floor weakening cannot make the same candidate green.
- [ ] **T035** Add design-policy self-tests: per-test flaky override cannot defeat CLI fail semantics; zero/incomplete coverage source fails; source-exclusion broadening/floor reduction is detected against canonical base policy; cancelled/incomplete test/coverage runs fail.
- [ ] **T036** Qualify and merge Stack B0 exact head with all applicable gates and fresh Qodo/CodeRabbit, then re-read canonical authority. Only then measure coverage or add dependent enforcement.

## Stack B1 — flaky-as-failure and measured coverage

Depends on canonical T036.

- [ ] **T040** Add nextest lane using the frozen profile and mandatory CLI flags. Keep canonical `cargo test --workspace --all-features --locked` separate and mandatory.
- [ ] **T041** Execute deterministic retry fixture and prove first fail/later pass still yields non-zero AF-02 result and cannot be overridden to green.
- [ ] **T042** Measure first canonical AF-02 coverage baseline under the frozen descriptor; retain source/tree/compiler/tool/lock/manifests/command/platform/raw-report/corpus/property identities and exact covered/total integer pairs.
- [ ] **T043** Mechanically freeze floors: workspace production line floor = integer floor of measured `covered/total*100`; each `COVERAGE_CRITICAL` surface independently receives its own integer floor; no averaging. Function/region values remain diagnostics initially.
- [ ] **T044** Implement coverage/base-policy validation: floor breach, missing critical source, unknown source, descriptor drift, same-candidate floor reduction or source exclusion weakening fail closed.
- [ ] **T045** Run exact-head Stack B canonical cargo test, nextest, coverage, Stack A adversarial replay/property, authority preservation and every path-applicable product/oracle/AF-01 workflow.
- [ ] **T046** Obtain fresh Qodo/CodeRabbit and require zero unresolved substantive findings.
- [ ] **T047** Merge Stack B only from exact qualified head; record post-merge baseline/floors/tool identities and re-read authority before Stack C0.

## Stack C0 — design freeze: mutation and exact-head proof

Depends on canonical T047. No required mutation/proof implementation until T057 is canonical.

- [ ] **T050** Pin/verify cargo-mutants 27.1.0 at commit `8ab1dc786a1f61a4e370416cc6c68b81a704e917` through the exact tool-lock procedure.
- [ ] **T051** Verify exact pinned-version cargo-mutants flags/config and freeze command, build profile, test command, parallelism (default max 2), baseline required, minimum test timeout 20s, max test timeout <=120s, max build timeout <=180s, targeted source paths and exclusions.
- [ ] **T052** Freeze JSON mutant inventory procedure using `--list --json` or exact-version equivalent, retained `mutants.json` SHA-256, stable mutant-ID derivation bound to source/tool/config, and required-mutant selection criteria prioritizing false-PASS/security boundaries.
- [ ] **T053** Freeze mutation result schema: `KILLED`, `SURVIVED`, `TIMEOUT`, `UNVIABLE_OR_BUILD_FAILURE`, `WAIVED_EQUIVALENT_OR_OUT_OF_SCOPE`; every TIMEOUT/UNVIABLE gets bounded retry and diagnosis; unresolved results need the exact waiver standard and never count as killed.
- [ ] **T054** Freeze waiver governance fields and anti-gaming: a newly added waiver or reduced required set cannot make the same candidate green; dedicated reviewed policy PR evaluated against prior canonical policy is required.
- [ ] **T055** Freeze `commandf.af02-adversarial-proof/v1` canonicalization and independent verifier: no floats in deterministic object, recursive UTF-8 key order, schema-defined arrays, compact UTF-8 JSON, independent raw evidence recomputation, stochastic observations excluded from deterministic digest.
- [ ] **T056** Freeze final CI/artifact topology and base-policy anti-forgery algorithm, including candidate/base SHA/tree comparison, weakening classification, incomplete-run semantics and always-run AF-01/CF-06/CF-10 identity verification.
- [ ] **T057** Qualify and merge Stack C0 exact head with all existing/AF-02 design gates and fresh Qodo/CodeRabbit. Only then run required mutation qualification or implement final proof workflow.

## Stack C1 — mutation adequacy and AF-02 proof

Depends on canonical T057.

- [ ] **T060** Generate frozen cargo-mutants JSON inventory on exact source and freeze required target set under canonical C0 rules.
- [ ] **T061** Run required mutation set. Strengthen property/regression/tests until every required survivor is killed or has a previously canonical exact waiver. Execute bounded retry+diagnosis for TIMEOUT/UNVIABLE; retain separate counts and no unclassified result.
- [ ] **T062** Add mutation-policy self-tests: intentional survivor fails; stronger test kills it; timeout/build failure is not killed; malformed/missing waiver metadata fails; same-candidate new waiver cannot self-green.
- [ ] **T063** Run bounded stochastic discovery under resource policy. Retain exact campaign config/source/tool/corpus identity. No-crash is bounded observation only; harness/resource/cancellation is incomplete.
- [ ] **T064** Implement exact-head AF-02 proof workflow/verifier. It independently reconstructs deterministic evidence and recomputes `AF02_ADVERSARIAL_SHA256`, refusing producer-supplied forged summaries.
- [ ] **T065** Add proof regressions for source/tree/base-policy mismatch; policy/file/corpus digest mismatch; assertion replay failure; retry-pass; coverage breach/descriptor drift; unknown/unwaived mutation result; malformed authority state; stochastic fields affecting deterministic digest; manually forged producer summary.
- [ ] **T066** Reconcile AF-02 workflows/configs into AF-01 workflow-trust/dependency/security coverage. Default live required contexts remain unchanged; any proposed new required context is a separate universal-terminal/live-ruleset governance unit.
- [ ] **T067** Run exact-head AF-02 proof, canonical cargo test, Stack A/B gates and every path-applicable product/oracle/AF-01 workflow; retain run/job/artifact/digest/tool/corpus/coverage/mutation/source identities.
- [ ] **T068** Obtain fresh Qodo/CodeRabbit, require zero unresolved substantive findings, merge only from exact qualified head, then verify post-merge main/tree/proof applicability and unchanged live/canonical authority.

## Phase 4 — convergence and canonical closeout

Depends on canonical T068.

- [ ] **T070** Re-read AF-02 spec/evidence-contracts/plan/tasks/consistency/donor record, constitution, AGENTS, architecture/plan index/assurance program, implementation tree, live AF-01 rulesets, CF-06 authority, CF-10 frozen evidence and active PR state; reconcile drift.
- [ ] **T071** Create `convergence.md` recording planning/A0/A1/B0/B1/C0/C1 identities, tool-lock/executable/package checksums, surfaces/exclusions, resource/offline evidence, property models/config, corpus/assertion bindings, nextest truth, coverage baseline/floors, mutation inventory/results/waivers, stochastic observations, exact proof artifact/digest, authority read-backs, reviewer dispositions, limits and deferrals.
- [ ] **T072** Confirm diff from pre-AF-02 canonical base contains no unauthorized product semantic/public-API change; any internal test-seam refactor is behavior-preserving and fully regression-qualified.
- [ ] **T073** Confirm every discovered defect used in closure has a minimized deterministic assertion-bound regression; every required mutation result is green under canonical policy; no retry-pass, coverage breach, incomplete run or authority drift remains.
- [ ] **T074** Confirm base-policy anti-forgery across all policy changes and record remaining AF-03/AF-04/CF work without overclaiming portability/release/performance/product completion.
- [ ] **T075** Exact convergence head receives all path-applicable CI/AF-02 proof and fresh Qodo/CodeRabbit with zero unresolved substantive findings.
- [ ] **T076** Merge convergence only from exact qualified head with expected-head guard; verify post-merge main/tree, AF-02 proof and unchanged authority.
- [ ] **T077** Create docs-only closeout candidate that reconciles task state without circular future identifiers; qualify exact head with required/path-applicable CI and fresh independent review; merge with expected-head guard.
- [ ] **T078** Mark `AF-02=CLOSED_CANONICAL` only after T077 merge and post-merge canonical main/tree, AF-02 proof, corpus/coverage/mutation/flaky truth, and AF-01/CF-06/CF-10 authority are re-read and remain valid.

## Retained handoffs

AF-03 remains separate: Linux/Windows/macOS, explicit MSRV, public API/SemVer, SBOM, SLSA-compatible provenance, signatures and stable release verification.

AF-04 remains separate: performance/resource benchmarks, large-input stress, external sentinel separation and retained trends.

AF-02 does not implement CF-14/15/16. Any future CF-14 parser/instance-data boundary must reconcile the canonical AF-02 surface inventory before CF-14 can close.