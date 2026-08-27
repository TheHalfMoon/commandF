# AF-02 Tasks — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Task-state rules

- `[ ]` means not yet proven complete on canonical repository evidence.
- A task is complete only when its implementation/evidence is on the exact reviewed head required by its stack and survives canonicalization.
- `verification-protocol.md` and the machine-readable schemas under `schemas/` are normative AF-02 authority. `evidence-contracts.md` is normative only where it is not superseded by that closed protocol/schema set.
- The illustrative `commandf.af02-authority-baseline/v1` in `evidence-contracts.md` is deprecated and MUST NOT be implemented; use `commandf.af02-authority-baseline/v2`.
- No stale-head CI/reviewer result qualifies a later head.
- No force-push, rebase, or destructive history rewrite.
- No PASS/MERGED/CLOSED_CANONICAL claim without exact-head/post-merge evidence.

## Planning gate

- [ ] **T001** Re-read canonical `main` SHA/tree and confirm AF-01 is `CLOSED_CANONICAL` before planning merge.
- [ ] **T002** Confirm PR diff is planning/provenance only and changes no product source, workflow, Cargo input, dependency, live ruleset, CF-06 pin, CF-10 corpus, or behavior.
- [ ] **T003** Reconcile `spec.md`, `verification-protocol.md`, machine schemas, `evidence-contracts.md`, `plan.md`, this task list, `consistency.md`, and donor/provenance metadata under one explicit precedence rule.
- [ ] **T004** Verify retained authority source locators and exact AF-01/CF-06/CF-10 expected identities from live/canonical/retained sources; do not infer CF-10 success from its retained failed run.
- [ ] **T005** Qualify one exact final planning head: all path-applicable workflows terminal green, required contexts unique and GitHub-Actions-app bound, fresh Qodo and CodeRabbit review truth, zero unresolved substantive threads.
- [ ] **T006** Merge planning with expected-head guard; re-read canonical post-merge `main`/tree and both live AF-01 rulesets. Only then set `AF-02 PLANNING: CANONICAL` and authorize **Stack A0 only**.

## Stack A0 — design freeze and independently anchored verification

A0 contains policy/schema/verifier infrastructure. It does not use new fuzz/property discoveries as evidence that A0 itself is correct.

- [ ] **T010** Add `commandf.af02-authority-baseline/v2` validated by `schemas/af02-authority-baseline-v2.schema.json`.
- [ ] **T011** Reconstruct AF-01 ruleset projections live and CF-06 from canonical-base source files; compare with baseline v2.
- [ ] **T012** Fetch CF-10 manifest/donor from the exact retained commit/blob locators in `retained-authority-sources.json`; prove exactly 3 deltas/6 states and retained PR/head/base/run/conclusion/artifact id/name/digest.
- [ ] **T013** Add `commandf.af02-surface-policy/v1` and one deterministic `syn=3.0.3` AST scanner over tracked Rust under both `crates/**/src/**` and `tools/**/src/**`.
- [ ] **T014** Prove scanner completeness: exact Git source universe, alias/macro/cfg/dead-code semantics, every finding single-dispositioned, zero stale/unclassified entries, scanner/tool/policy digests retained.
- [ ] **T015** Add `commandf.af02-resource-policy/v1` with campaign/execution/input/memory/CPU/PID/tmpfs/generated/decompressed/temp-file/subprocess/artifact/corpus/retention limits.
- [ ] **T016** Implement canonical digest-pinned Linux OCI runner: network none, read-only root/source, dedicated writable output, CPU/memory/PID/tmpfs constraints, runtime inspection and negative network/write probes.
- [ ] **T017** Add immutable tool-lock acquisition verification for cargo-fuzz, cargo-nextest, cargo-llvm-cov, cargo-mutants and scanner/parser dependencies; retain package/source/asset and installed executable identities.
- [ ] **T018** Freeze independent property/model registry for archive, Lockfile, source paths, canonical references, graph order, gate/fingerprint/suppression semantics.
- [ ] **T019** Add deterministic corpus policy and `commandf.af02-assertion-registry/v1`; enforce scenario/assertion bijection, exact argv/target/parser/source/config binding and no-PHI/provenance rules.
- [ ] **T020** Add machine-readable proof/authority schemas and repository semantic validator; include schema SHA-256 in proof contract files.
- [ ] **T021** Add `commandf.af02-enforcement-inventory/v1` covering every authority projector/scanner/parser/runner/result/proof/workflow path.
- [ ] **T022** Implement base-controlled `pull_request_target` AF-02 verifier gate from canonical-base workflow code with read-only permissions; separate base/candidate checkouts, candidate-as-data-only, no candidate execution.
- [ ] **T023** Prove the base-verifier gate records base workflow/verifier/schema/inventory blob SHAs, exact base/head, fixed command/input identities, and fails closed if base verifier cannot run/parse.
- [ ] **T024** Prove base-verifier path trigger coverage for every AF-02 policy/schema/verifier/scanner/parser/result/workflow/enforcement-inventory change.
- [ ] **T025** Prove the new base-verifier check has universal terminal topology. If it will become a main required context, apply/re-read the live AF-01 ruleset only after the check exists successfully on the exact A0 head.
- [ ] **T026** Add negative tests for candidate workflow trying to skip/rename/base-ref-swap/replace the base verifier and for candidate verifier falsely labeled as base.
- [ ] **T027** Exact-head A0 qualification: canonical `cargo test --workspace --all-features --locked`, existing CI/security/oracle/assurance gates, fresh Qodo/CodeRabbit, zero substantive unresolved findings.
- [ ] **T028** Merge A0 with expected-head guard, re-read canonical main/rulesets/base-verifier topology; authorize A1 only.

## Stack A1 — fuzz/property/replay implementation

- [ ] **T030** Create isolated fuzz workspace pinned to cargo-fuzz 0.13.2, libfuzzer-sys 0.4.13, arbitrary 1.4.2, nightly-2026-08-25; no nightly leakage into normal workspace authority.
- [ ] **T031** Implement bounded raw package/archive fuzz target through an existing product seam.
- [ ] **T032** Implement Lockfile raw/structured fuzz and property targets.
- [ ] **T033** Implement retained-report/check/gate fuzz/property targets with independent validation model where available.
- [ ] **T034** Implement context-graph/canonical-reference structured targets and order/ambiguity properties.
- [ ] **T035** Implement source-map/portable-path adversarial properties through narrow existing/internal test seams without public API expansion.
- [ ] **T036** Run all deterministic property configurations; retain exact model/config/case counts and minimized counterexamples.
- [ ] **T037** Enforce OCI offline/resource limits for every deterministic replay/property/fuzz-build path; classify harness/resource failures separately from valid rejection.
- [ ] **T038** Promote every discovered crash/invariant failure/unexpected acceptance/property counterexample to a minimized deterministic corpus fixture with stable scenario id/digest and assertion registry entry.
- [ ] **T039** Prove no orphan corpus/assertion entries, no PHI/provenance violations, <=256 KiB default fixture, <=8 MiB aggregate corpus.
- [ ] **T040** Add corpus replay gate and prove byte/invariant-stable results from fixed corpus/config.
- [ ] **T041** Exact-head A1 qualification and external review; merge with expected-head guard; re-read canonical main before B0.

## Stack B0 — flaky/coverage design freeze

- [ ] **T050** Freeze nextest 0.9.143 tool lock and `.config/nextest.toml` profile `ci` with `retries=2`, `flaky-result="fail"`, bounded slow timeout and deterministic JUnit path.
- [ ] **T051** Freeze exact retry-pass fixture path/manifest/target/test and argv with explicit `--retries 2 --flaky-result fail`.
- [ ] **T052** Freeze deterministic atomic state protocol and dedicated empty output mount protocol: parent mode/owner, target absent/non-symlink before run, JUnit regular/non-symlink/owner/link-count/output-mount checks after run.
- [ ] **T053** Freeze nextest result parser: exactly one selected testcase, flaky retry history, fixed state transition, non-zero process exit, captured stdout/stderr/JUnit hashes in one base-controlled runner envelope.
- [ ] **T054** Freeze coverage descriptor before observing percentages: exact source/tree/platform/Rust/tool/command/Cargo/replay/property/exclusion identities.
- [ ] **T055** Freeze Git-derived coverage universe over tracked Rust under **both** `crates/**/src/**` and `tools/**/src/**`; exact previous exclusions only; missing/unknown/duplicate normalized paths fail.
- [ ] **T056** Add machine-schema and semantic-validator negative tests for malformed types, formats, ranges, enums, cardinality, conditional shapes, counter relations, path containment, digest relationships and unknown fields.
- [ ] **T057** Exact-head B0 qualification/review and guarded merge; re-read canonical main before B1.

## Stack B1 — flaky/coverage execution

- [ ] **T060** Run canonical `cargo test --workspace --all-features --locked`; retain exact raw/count evidence.
- [ ] **T061** Run ordinary nextest no-flake evidence with frozen configuration and forced flaky-result failure semantics.
- [ ] **T062** Run isolated retry-pass fixture from a clean dedicated output mount; prove JUnit is created by the same waited-for nextest process envelope and process exit remains non-zero.
- [ ] **T063** Run frozen cargo-llvm-cov command against exact inputs and parse each production source exactly once, including zero-hit files.
- [ ] **T064** Derive workspace and each critical-surface line floor independently with checked integer arithmetic; freeze descriptor/raw report/source universe/file metrics.
- [ ] **T065** Prove anti-gaming: missing path, duplicate path, unknown path, test-selection drift, command drift, exclusion drift, floor reduction, malformed report, or same-candidate policy weakening fails.
- [ ] **T066** Require any future coverage floor/scope/exclusion change to be a dedicated policy-only PR under prior policy, with no product/tests/measurement-command edits; later PR adopts new baseline.
- [ ] **T067** Exact-head B1 qualification/review and guarded merge; re-read canonical main before C0.

## Stack C0 — mutation/proof design freeze

- [ ] **T070** Freeze exact cargo-mutants 27.1.0 tool lock, command/config, target source paths, test command, timeout policy and exact previously reviewed exclusions **before listing/execution**.
- [ ] **T071** Generate pinned JSON mutant inventory and stable IDs.
- [ ] **T072** Derive required set by the only allowed rule: **every listed mutant inside frozen target paths is REQUIRED unless it matches exactly one pre-frozen exact exclusion**. No post-list selection, top-N, percentage, operator preference or manual priority subset.
- [ ] **T073** Freeze mutation result classes and mandatory retry/diagnosis for TIMEOUT/UNVIABLE_OR_BUILD_FAILURE; no such result is closure-green.
- [ ] **T074** Freeze exact waiver schema; new waiver cannot make the same implementation candidate green.
- [ ] **T075** Validate `schemas/af02-adversarial-proof-v1.schema.json` and semantic invariant table against positive fixtures and negative malformed-type/format/range/cardinality/conditional/cross-field/path/digest/counter fixtures.
- [ ] **T076** Bind proof schema file/digest plus authority baseline schema and retained-authority-source manifest into `contract_files[]`.
- [ ] **T077** Freeze final proof-builder/verifier separation and raw-evidence reconstruction order.
- [ ] **T078** Exact-head C0 qualification/review and guarded merge; re-read canonical main before C1.

## Stack C1 — mutation execution, discovery observations, final proof

- [ ] **T080** Execute every required mutant from frozen inventory with exact test identity.
- [ ] **T081** Retain KILLED/SURVIVED/TIMEOUT/UNVIABLE_OR_BUILD_FAILURE/WAIVED counts separately; retry/diagnose non-terminal required outcomes.
- [ ] **T082** Require every required mutant to close as KILLED or a previously canonical exact waiver; zero survivor/timeout/unviable/unclassified required outcomes at qualification.
- [ ] **T083** Run bounded stochastic fuzz campaigns under frozen resource/offline policy and retain them only as stochastic observations; no-crash is not correctness PASS.
- [ ] **T084** Reconstruct AF-01/CF-06/CF-10 authority from live/canonical/retained sources and compare with baseline v2.
- [ ] **T085** Build deterministic proof object from raw evidence, validate JSON Schema and semantic invariants, independently recompute `AF02_ADVERSARIAL_SHA256` and reject producer mismatch.
- [ ] **T086** Prove exact-head required contexts `rust`, `assurance-proof`, `scorecard` are each unique, success, exact head, GitHub Actions app id 15368; include any canonical AF-02 required base-verifier context if live policy has adopted it.
- [ ] **T087** Run full existing CI/security/oracle/assurance/non-regression gates.
- [ ] **T088** Fresh Qodo/CodeRabbit exact-head review; zero unresolved substantive findings.
- [ ] **T089** Guarded merge C1 and post-merge canonical/live-policy read-back.

## Final convergence

- [ ] **T090** Re-read canonical specs/plan/tasks/contracts/schemas/enforcement inventory and reconcile repository/CI/settings/process/document drift.
- [ ] **T091** Prove AF-01 workflow/dependency/source-control invariants unchanged or intentionally strengthened through separately qualified policy.
- [ ] **T092** Prove CF-06 production oracle identity unchanged and CF-10 frozen corpus/blocked production interpretation unchanged.
- [ ] **T093** Prove no PHI, no prohibited corpus payload, no unresolved mutation result, no coverage/source-universe omission, no retry-pass green, no authority/verifier self-forgery.
- [ ] **T094** Record exact final source/tree, workflow/check runs, required-context provenance, proof artifact/digest, tool locks, corpus digests, coverage floors, mutation inventory/results/waivers, stochastic observations, reviewer dispositions and live ruleset read-back in `convergence.md`.
- [ ] **T095** Final exact-head external review and existing CI gate reconciliation.
- [ ] **T096** Merge final convergence with expected-head guard and re-read canonical main/tree/live rulesets.
- [ ] **T097** Mark `AF02_CLOSED_CANONICAL` only if every dependency above is proven. If not, record the precise blocker without weakening the contract.

## Continuation rule

When AF-02 becomes `CLOSED_CANONICAL`, immediately re-read canonical roadmap/specs/tasks and begin the next genuinely eligible project unit. Do not stop solely because AF-02 closed.
