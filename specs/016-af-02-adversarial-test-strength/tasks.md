# AF-02 Tasks — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Task-state rules

- `[ ]` means not proven complete on canonical repository evidence.
- No task inherits stale-head CI/reviewer evidence.
- No force-push, rebase, or destructive history rewrite.
- No PASS/MERGED/CLOSED_CANONICAL claim without exact-head and post-merge evidence.
- `verification-protocol.md`, checked-in policies, and schemas are normative. `evidence-contracts.md` applies only where not superseded.
- `commandf.af02-authority-baseline/v1` is prohibited; use v2.
- Final proof schema remains `commandf.af02-adversarial-proof/v1`, now an envelope over the preserved proof-core schema plus 17 extension contract roles (42 total contract files: 25 core + 17 extension); the preserved core already contains `enforcement_inventory`.

## Planning gate

- [ ] **T001** Re-read canonical `main` SHA/tree and confirm AF-01 is `CLOSED_CANONICAL`.
- [ ] **T002** Confirm planning diff changes no product source, workflow, Cargo input, dependency, live ruleset, CF-06 pin, CF-10 corpus, or product behavior.
- [ ] **T003** Reconcile `spec.md`, `verification-protocol.md`, machine policies/schemas, `evidence-contracts.md`, `plan.md`, `tasks.md`, `consistency.md`, and donor provenance under one precedence rule.
- [ ] **T004** Reconstruct AF-01/CF-06/CF-10 expected authority from live/canonical/retained sources; validate `retained-authority-sources.json` against its closed schema.
- [ ] **T005** Qualify one exact final planning head: all path-applicable workflows green; required contexts unique and proven through `required-check-policy.json` plus `af02-required-check-provenance-v1`; fresh Qodo/CodeRabbit truth; zero unresolved substantive findings.
- [ ] **T006** Merge with expected-head guard; re-read canonical post-merge `main`/tree and both live AF-01 rulesets. Only then set `AF-02 PLANNING: CANONICAL` and authorize Stack A0 only.

## Stack A0 — design freeze and base-controlled verifier

A0 contains policy/schema/verifier infrastructure and tests only. New fuzz/property/coverage/mutation outcomes cannot prove A0 itself.

- [ ] **T010** Add authority baseline v2 and verifier reconstruction for AF-01 live rulesets, CF-06 canonical-base sources, and CF-10 retained sources.
- [ ] **T011** Validate retained CF-10 locators with `af02-retained-authority-sources-v1.schema.json`; reconstruct URLs/objects rather than trusting candidate locator strings.
- [ ] **T012** Add `commandf.af02-surface-policy/v1` validated by `af02-surface-policy-v1.schema.json` and deterministic `syn=3.0.3` AST discovery over both Rust source roots.
- [ ] **T013** Prove exact source-universe membership and every discovery has one critical-surface or reviewed-exclusion disposition; zero stale/unclassified entries.
- [ ] **T014** Add `commandf.af02-resource-policy/v1` validated by `af02-resource-policy-v1.schema.json`.
- [ ] **T015** Implement the digest-pinned Linux OCI runner with network none, read-only source/root, bounded CPU/memory/PID/tmpfs/output and negative network/write probes.
- [ ] **T016** Freeze immutable tool acquisition/lock evidence; no registry package activates with a null checksum.
- [ ] **T017** Freeze independent property/model registry.
- [ ] **T018** Add `commandf.af02-corpus/v1`, assertion registry and no-PHI/provenance rules; validate corpus with `af02-corpus-v1.schema.json`.
- [ ] **T019** Add `waiver-policy.json` parser and ancestry verifier; start with zero waivers; same-candidate waiver cannot self-green.
- [ ] **T020** Add `required-check-policy.json` and exact GitHub provenance verifier using `af02-required-check-provenance-v1.schema.json`; app id alone is insufficient.
- [ ] **T021** Implement every algorithm and negative fixture named by `semantic-contract.json`; own tests must map algorithm/fixture ids to verifier code.
- [ ] **T022** Enforce `verifier-input-policy.json` before parsing untrusted candidate authority; bounded file/aggregate/depth/record/time/memory; YAML aliases/tags/merge keys prohibited.
- [ ] **T023** Add enforcement inventory covering projector/scanner/policy parsers/runners/check provenance/waiver/retained locator/input-limit/proof/workflow/schema authority.
- [ ] **T024** Add canonical-base `pull_request_target` verifier gate with read-only permissions, separate base/candidate trees, candidate-as-data-only and no candidate code execution.
- [ ] **T025** Prove base workflow/verifier/schema/inventory blob identity, path-trigger universality and fail-closed behavior for unparseable/unknown authority.
- [ ] **T026** Add anti-forgery negative tests: skip/rename/base-ref swap/candidate verifier substitution/candidate execution/parser exhaustion/symlink/path escape.
- [ ] **T027** Exact-head A0 CI/security/oracle/assurance + fresh Qodo/CodeRabbit + zero substantive threads.
- [ ] **T028** Guarded merge A0; re-read main/rulesets/base-verifier topology; authorize A1 only.

## Stack A1 — fuzz/property/replay

- [ ] **T030** Create isolated fuzz workspace pinned to cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13, arbitrary 1.4.2 and nightly-2026-08-25 without product-toolchain leakage.
- [ ] **T031** Implement bounded archive/package raw fuzzing through existing product seams.
- [ ] **T032** Implement Lockfile raw/structured fuzz/property targets and independent model checks.
- [ ] **T033** Implement report/check/gate, graph/reference and source-map/path adversarial properties.
- [ ] **T034** Run deterministic property configurations; retain model/config/case counts and minimized counterexamples.
- [ ] **T035** Enforce A0 resource/offline policy for build/replay/property paths; resource/harness failures remain separate.
- [ ] **T036** Promote every discovered defect/counterexample to minimized deterministic corpus + assertion entry before closure.
- [ ] **T037** Prove corpus/assertion/replay bijection, no PHI/provenance violation, per-fixture <=256 KiB and aggregate <=8 MiB.
- [ ] **T038** Exact-head A1 qualification/review; guarded merge; re-read main before B0.

## Stack B0 — nextest/coverage design freeze

- [ ] **T050** Freeze cargo-nextest 0.9.143, retries=2, flaky-result=fail and deterministic isolated retry-pass fixture/JUnit process envelope.
- [ ] **T051** Freeze nextest state-file/no-follow/output-mount parser predicates and negative fixtures.
- [ ] **T052** Freeze `commandf.af02-coverage-policy/v1` validated by `af02-coverage-policy-v1.schema.json` before observing percentages.
- [ ] **T053** Freeze coverage source universe equal to surface universe, exact command/Cargo/replay/property inputs and canonical-base exclusions.
- [ ] **T054** Add policy-schema/semantic negative tests for units/ranges/duplicate paths/missing paths/unknown scope/same-candidate weakening.
- [ ] **T055** Exact-head B0 qualification/review; guarded merge; re-read main before B1.

## Stack B1 — nextest/coverage execution

- [ ] **T060** Run canonical `cargo test --workspace --all-features --locked`.
- [ ] **T061** Run ordinary nextest and isolated retry-pass fixture; retry-pass remains non-zero failed AF-02 evidence.
- [ ] **T062** Run frozen cargo-llvm-cov descriptor; parse every production source exactly once including zero-hit files.
- [ ] **T063** Derive workspace/critical-surface floors with checked integer arithmetic; no missing/unknown/duplicate/out-of-scope source.
- [ ] **T064** Prove anti-gaming: command/test/scope/exclusion/floor/policy weakening cannot self-green.
- [ ] **T065** Exact-head B1 qualification/review; guarded merge; re-read main before C0.

## Stack C0 — mutation/proof design freeze

- [ ] **T070** Freeze `commandf.af02-mutation-policy/v1` validated by `af02-mutation-policy-v1.schema.json` before mutant listing.
- [ ] **T071** Freeze target paths, cargo-mutants identity/argv/test/timeout/retry/diagnosis, exclusion-policy digest and waiver-policy digest.
- [ ] **T072** Derive required set as all listed in-target mutants minus exact pre-frozen exclusions; no post-list selection.
- [ ] **T073** Freeze mutation result classes; TIMEOUT/UNVIABLE require retry+diagnosis and remain non-green.
- [ ] **T074** Validate waiver-policy schema and canonical-ancestry/mutant-binding negative tests.
- [ ] **T075** Validate final proof envelope, preserved proof-core, 17 extension roles (42 total contract files; `enforcement_inventory` remains a core role), required-check provenance and final deterministic hashing.
- [ ] **T076** Prove every proof-critical policy instance validates against its planning-frozen schema before dependent evidence.
- [ ] **T077** Exact-head C0 qualification/review; guarded merge; re-read main before C1.

## Stack C1 — mutation execution and final proof

- [ ] **T080** Execute every required mutant from frozen inventory.
- [ ] **T081** Close every required result as KILLED or a previously canonical exact waiver; zero survivor/timeout/unviable/unclassified required outcomes.
- [ ] **T082** Run bounded stochastic fuzz campaigns only as stochastic observations; no-crash is not correctness PASS.
- [ ] **T083** Reconstruct AF-01/CF-06/CF-10 authority and compare with baseline v2.
- [ ] **T084** Reconstruct proof from raw evidence; validate proof-core + extension schemas, semantic contract and cross-object invariants; independently compute final `AF02_ADVERSARIAL_SHA256`.
- [ ] **T085** Prove exact-head required-check GitHub provenance from API truth and canonical-base workflow blobs.
- [ ] **T086** Full existing CI/security/oracle/assurance/non-regression gates + fresh Qodo/CodeRabbit.
- [ ] **T087** Guarded merge C1 and post-merge main/live-policy read-back.

## Final convergence

- [ ] **T090** Reconcile repository/CI/settings/process/document drift.
- [ ] **T091** Prove AF-01 invariants unchanged or separately strengthened; CF-06 identity unchanged; CF-10 failed production interpretation unchanged.
- [ ] **T092** Prove no PHI, no unresolved mutation result, no coverage/source omission, no retry-pass green, no verifier self-forgery and no parser-limit bypass.
- [ ] **T093** Record final source/tree, runs/checks/artifacts, required-check provenance, proof digest, tool locks, corpus, coverage, mutation/waivers, stochastic observations, reviewer dispositions and live ruleset read-back.
- [ ] **T094** Final exact-head external review and existing CI reconciliation.
- [ ] **T095** Guarded convergence merge and post-merge main/tree/live-ruleset read-back.
- [ ] **T096** Mark `AF02_CLOSED_CANONICAL` only when every dependency is proven; otherwise record the precise blocker without weakening the contract.

## Continuation rule

When AF-02 becomes `CLOSED_CANONICAL`, immediately re-read canonical roadmap/specs/tasks and begin the next genuinely eligible project unit.
