# AF-02 Tasks — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Task-state rules

- `[ ]` means not yet proven complete on canonical repository evidence.
- A task is complete only when its implementation/evidence is on the exact reviewed head required by its stack and survives canonicalization.
- `verification-protocol.md`, `tool-policy.json`, `exclusion-policy.json`, and machine-readable schemas under `schemas/` are normative AF-02 authority. `evidence-contracts.md` is normative only where not superseded by that closed set.
- `commandf.af02-authority-baseline/v1` is deprecated and MUST NOT be implemented; use v2 only.
- No stale-head CI/reviewer result qualifies a later head.
- No force-push, rebase, or destructive history rewrite.
- No PASS/MERGED/CLOSED_CANONICAL claim without exact-head/post-merge evidence.

## Planning gate

- [ ] **T001** Re-read canonical `main` SHA/tree and confirm AF-01 is `CLOSED_CANONICAL` before planning merge.
- [ ] **T002** Confirm PR diff is planning/provenance only and changes no product source, workflow, Cargo input, dependency, live ruleset, CF-06 pin, CF-10 corpus, or behavior.
- [ ] **T003** Reconcile spec/protocol/policies/schemas/evidence-contracts/plan/tasks/consistency/donor metadata under the explicit precedence rule.
- [ ] **T004** Verify retained authority source locators and exact AF-01/CF-06/CF-10 identities from live/canonical/retained sources; do not infer CF-10 success from its retained failed run.
- [ ] **T005** Qualify one exact final planning head: all path-applicable workflows terminal green, required contexts unique and GitHub-Actions-app bound, fresh Qodo and CodeRabbit review truth, zero unresolved substantive threads.
- [ ] **T006** Merge planning with expected-head guard; re-read canonical post-merge `main`/tree and both live AF-01 rulesets. Only then set `AF-02 PLANNING: CANONICAL` and authorize **Stack A0 only**.

## Stack A0 — design freeze and independently anchored verification

A0 contains policy/schema/verifier infrastructure. It does not use new fuzz/property/coverage/mutation discoveries as evidence that A0 itself is correct.

- [ ] **T010** Add `commandf.af02-authority-baseline/v2` validated by `schemas/af02-authority-baseline-v2.schema.json`.
- [ ] **T011** Reconstruct AF-01 ruleset projections live and CF-06 from canonical-base source files; compare with baseline v2.
- [ ] **T012** Fetch CF-10 manifest/donor from exact retained commit/blob locators; prove exactly 3 deltas/6 states and retained PR/head/base/run/conclusion/artifact id/name/digest.
- [ ] **T013** Validate canonical `tool-policy.json` against `schemas/af02-tool-policy-v1.schema.json`; freeze A0 active set to exactly `syn-af02-scanner` with `syn=3.0.3`, features `[full,visit]`, checksum `53e9bae58849f64dfa4f5d5ae372c8341f7305f82a3868709269343628b659a3`.
- [ ] **T014** Validate canonical `exclusion-policy.json` against `schemas/af02-exclusion-policy-v1.schema.json`; initial production-source and mutation exclusion arrays are exactly empty. Any future addition must be canonical before dependent evidence.
- [ ] **T015** Add `commandf.af02-surface-policy/v1` and one deterministic syn AST scanner over the Git-derived source universe under both `crates/**/src/**` and `tools/**/src/**`, minus only canonical-base production source exclusions.
- [ ] **T016** Prove scanner completeness: exact source-universe schema/digest, alias/macro/cfg/dead semantics, every finding single-dispositioned, zero stale/unclassified entries, scanner/tool/policy/exclusion digests retained.
- [ ] **T017** Add `commandf.af02-resource-policy/v1` with campaign/execution/input/memory/CPU/PID/tmpfs/generated/decompressed/temp-file/subprocess/artifact/corpus/retention limits.
- [ ] **T018** Implement canonical digest-pinned Linux OCI runner: network none, read-only root/source, dedicated writable output, CPU/memory/PID/tmpfs constraints, runtime inspection and negative probes.
- [ ] **T019** Implement `commandf.af02-tool-lock/v1` validated by `schemas/af02-tool-lock-v1.schema.json`; base verifier derives mandatory active members from canonical-base tool policy and rejects missing/unexpected/substituted entries.
- [ ] **T020** Add/validate closed machine schemas for source universe, assertion registry, replay results, coverage inventory, mutation inventory, corpus fixtures, property counterexamples and enforcement inventory using `schemas/af02-evidence-inventories-v1.schema.json`.
- [ ] **T021** Freeze independent property/model registry plus deterministic corpus/assertion policy; enforce no-PHI/provenance, scenario/assertion bijection, exact argv/target/parser/source/config binding.
- [ ] **T022** Add `commandf.af02-enforcement-inventory/v1` validated by the evidence-inventory schema bundle, covering every authority projector/scanner/parser/runner/result/proof/workflow/schema path.
- [ ] **T023** Implement base-controlled `pull_request_target` AF-02 verifier gate from canonical-base workflow code with read-only permissions; separate base/candidate checkouts, candidate-as-data-only, no candidate execution.
- [ ] **T024** Prove the base gate records base workflow/verifier/schema/policy/inventory blob SHAs, exact base/head, fixed command/input identities, and fails closed if base verifier cannot run/parse.
- [ ] **T025** Prove base-gate trigger coverage for every AF-02 policy/schema/verifier/scanner/parser/result/workflow/enforcement-inventory change; candidate path filters cannot disable it.
- [ ] **T026** Prove universal terminal topology. If the base-verifier check will become a main required context, apply/re-read live AF-01 ruleset only after the check exists successfully on the exact A0 head.
- [ ] **T027** Add negative tests for empty/omitted/substituted tool members, wrong digest/checksum/features, unlisted/same-candidate exclusions, malformed evidence inventories, candidate workflow skip/rename/base-ref-swap/replace attempts, and candidate verifier falsely labeled as base.
- [ ] **T028** Exact-head A0 qualification: canonical cargo test, existing CI/security/oracle/assurance gates, fresh Qodo/CodeRabbit, zero substantive unresolved findings.
- [ ] **T029** Merge A0 with expected-head guard, re-read canonical main/rulesets/base-verifier topology; authorize A1 only.

## Stack A1 — fuzz/property/replay implementation

- [ ] **T030** Before dependent execution, replace A1 registry checksum nulls in canonical tool policy with exact crates.io checksums in a design-freeze policy change; require active set `arbitrary`, `cargo-fuzz`, `libfuzzer-sys`, `proptest`, `syn-af02-scanner` and no others for A1 qualification.
- [ ] **T031** Create isolated fuzz workspace pinned to cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13, arbitrary 1.4.2 with derive, nightly-2026-08-25; no nightly leakage into normal workspace authority.
- [ ] **T032** Implement bounded raw package/archive fuzz target through an existing product seam.
- [ ] **T033** Implement Lockfile raw/structured fuzz and property targets.
- [ ] **T034** Implement retained-report/check/gate fuzz/property targets with independent validation model where available.
- [ ] **T035** Implement context-graph/canonical-reference structured targets and order/ambiguity properties.
- [ ] **T036** Implement source-map/portable-path adversarial properties through narrow existing/internal test seams without public API expansion.
- [ ] **T037** Run deterministic property configurations; retain exact model/config/case counts and minimized counterexample inventory under the closed schema.
- [ ] **T038** Enforce OCI offline/resource limits for every deterministic replay/property/fuzz-build path; classify harness/resource failures separately from valid rejection.
- [ ] **T039** Promote every discovered crash/invariant failure/unexpected acceptance/property counterexample to a minimized deterministic corpus fixture with stable scenario id/digest and assertion entry.
- [ ] **T040** Prove no orphan corpus/assertion/replay entries, no PHI/provenance violations, <=256 KiB default fixture, <=8 MiB aggregate corpus; validate closed fixture/replay schemas.
- [ ] **T041** Add deterministic corpus replay gate and prove byte/invariant-stable results from fixed corpus/config.
- [ ] **T042** Exact-head A1 qualification/review and guarded merge; re-read canonical main before B0.

## Stack B0 — flaky/coverage design freeze

- [ ] **T050** Activate exact cargo-nextest 0.9.143 and cargo-llvm-cov 0.9.0 tool-policy members and require their complete tool-lock evidence; freeze nextest `profile.ci` with retries=2, flaky-result=fail, bounded slow timeout and deterministic JUnit path.
- [ ] **T051** Freeze exact retry-pass fixture path/manifest/target/test and argv with explicit `--retries 2 --flaky-result fail`.
- [ ] **T052** Freeze deterministic atomic state protocol and dedicated empty output mount protocol: parent mode/owner, target absent/non-symlink before run, JUnit regular/non-symlink/owner/link-count/output-mount checks after run.
- [ ] **T053** Freeze nextest result parser: exactly one selected testcase, retry history, fixed state transition, non-zero exit, stdout/stderr/JUnit hashes in one base-controlled process envelope.
- [ ] **T054** Freeze coverage descriptor before observing percentages: exact source/tree/platform/Rust/tool/command/Cargo/replay/property/exclusion identities.
- [ ] **T055** Freeze Git-derived coverage universe using the same `commandf.af02-source-universe/v1` and canonical-base exclusion-policy digest as surface discovery; missing/unknown/duplicate paths fail.
- [ ] **T056** Validate `commandf.af02-coverage-inventory/v1`; add machine-schema and semantic-validator negative tests for malformed types/formats/ranges/enums/cardinality/order/uniqueness/counter/path/digest/exclusion relations.
- [ ] **T057** Exact-head B0 qualification/review and guarded merge; re-read canonical main before B1.

## Stack B1 — flaky/coverage execution

- [ ] **T060** Run canonical `cargo test --workspace --all-features --locked`; retain exact raw/count evidence.
- [ ] **T061** Run ordinary nextest no-flake evidence with frozen configuration and forced flaky-result failure semantics.
- [ ] **T062** Run isolated retry-pass fixture from clean dedicated output; prove JUnit created by same waited-for process envelope and exit remains non-zero.
- [ ] **T063** Run frozen cargo-llvm-cov command and parse each production source exactly once, including zero-hit files.
- [ ] **T064** Derive workspace and each critical-surface line floor independently with checked integer arithmetic; freeze descriptor/raw report/source universe/coverage inventory.
- [ ] **T065** Prove anti-gaming: missing/duplicate/unknown path, test-selection drift, command drift, exclusion drift, floor reduction, malformed report, or same-candidate policy weakening fails.
- [ ] **T066** Any future coverage floor/scope/exclusion change is a prior dedicated policy-only PR under prior policy, with no dependent product/tests/measurement result in the same candidate.
- [ ] **T067** Exact-head B1 qualification/review and guarded merge; re-read canonical main before C0.

## Stack C0 — mutation/proof design freeze

- [ ] **T070** Activate exact cargo-mutants 27.1.0 tool-policy member; freeze command/config, target source paths, test command, timeout policy and canonical-base mutation exclusions **before listing/execution**.
- [ ] **T071** Generate pinned JSON mutant inventory under `commandf.af02-mutation-inventory/v1` and stable IDs.
- [ ] **T072** Derive required set only as every listed mutant in frozen target paths minus exact canonical-base exclusions. No post-list selection/top-N/percentage/operator preference/manual priority subset.
- [ ] **T073** Freeze mutation result classes and mandatory retry/diagnosis for TIMEOUT/UNVIABLE_OR_BUILD_FAILURE; no such result is closure-green.
- [ ] **T074** Freeze exact waiver schema; new waiver cannot make same implementation candidate green.
- [ ] **T075** Validate final `schemas/af02-adversarial-proof-v1.schema.json`, exact 8-member tool lock, 25 contract-file roles, and semantic invariants against positive/negative fixtures.
- [ ] **T076** Bind proof/authority/tool-policy/tool-lock/exclusion/evidence-inventory schemas plus policy instances and retained authority into `contract_files[]`.
- [ ] **T077** Freeze final proof-builder/verifier separation and raw-evidence reconstruction order.
- [ ] **T078** Exact-head C0 qualification/review and guarded merge; re-read canonical main before C1.

## Stack C1 — mutation execution, discovery observations, final proof

- [ ] **T080** Execute every required mutant from frozen inventory with exact test identity.
- [ ] **T081** Retain KILLED/SURVIVED/TIMEOUT/UNVIABLE_OR_BUILD_FAILURE/WAIVED counts separately; retry/diagnose non-terminal required outcomes.
- [ ] **T082** Require every required mutant to close as KILLED or previously canonical exact waiver; zero survivor/timeout/unviable/unclassified required outcomes.
- [ ] **T083** Run bounded stochastic fuzz campaigns under frozen resource/offline policy; no-crash is observation only.
- [ ] **T084** Reconstruct AF-01/CF-06/CF-10 authority and canonical-base tool/exclusion policies; compare with proof bindings.
- [ ] **T085** Validate every raw inventory against closed schema, build deterministic proof from raw evidence, validate proof schema/semantics, independently recompute `AF02_ADVERSARIAL_SHA256`, reject producer mismatch.
- [ ] **T086** Prove exact-head live required contexts unique/success/exact-head/GitHub-App-bound; include any canonical AF-02 required context only if live policy adopted it through prior governance.
- [ ] **T087** Run full existing CI/security/oracle/assurance/non-regression gates.
- [ ] **T088** Fresh Qodo/CodeRabbit exact-head review; zero unresolved substantive findings.
- [ ] **T089** Guarded merge C1 and post-merge canonical/live-policy read-back.

## Final convergence

- [ ] **T090** Re-read canonical specs/plan/tasks/contracts/policies/schemas/enforcement inventory and reconcile repository/CI/settings/process/document drift.
- [ ] **T091** Prove AF-01 workflow/dependency/source-control invariants unchanged or intentionally strengthened through separately qualified policy.
- [ ] **T092** Prove CF-06 production oracle identity unchanged and CF-10 frozen corpus/failed retained-run interpretation unchanged.
- [ ] **T093** Prove no PHI, prohibited corpus payload, unresolved mutation result, coverage/source-universe omission, retry-pass green, unlisted exclusion, incomplete tool lock, or authority/verifier self-forgery.
- [ ] **T094** Record exact final source/tree, workflow/check runs, required-context provenance, proof artifact/digest, tool policies/locks, exclusion policy, corpus digests, coverage floors, mutation inventories/results/waivers, stochastic observations, reviewer dispositions and live ruleset read-back in `convergence.md`.
- [ ] **T095** Final exact-head external review and existing CI gate reconciliation.
- [ ] **T096** Merge final convergence with expected-head guard and re-read canonical main/tree/live rulesets.
- [ ] **T097** Mark `AF02_CLOSED_CANONICAL` only if every dependency above is proven; otherwise record the precise blocker without weakening the contract.

## Continuation rule

When AF-02 becomes `CLOSED_CANONICAL`, immediately re-read canonical roadmap/specs/tasks and begin the next genuinely eligible project unit. Do not stop solely because AF-02 closed.
