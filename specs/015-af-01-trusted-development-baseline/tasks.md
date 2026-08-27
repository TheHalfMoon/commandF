# AF-01 Tasks — Trusted Development Baseline

Status: PLANNING_CANDIDATE

## Task-state rules

- A checkbox is completed only by exact repository/GitHub evidence, not by intent.
- Any head mutation invalidates prior exact-head implementation/review qualification unless the affected gate explicitly proves it is content-independent.
- Product semantics remain frozen throughout AF-01.
- No implementation task begins until T005 is canonical.
- AF-01 cannot close while the live `main` ruleset/branch-policy requirement remains unproven.

## Phase 0 — planning and authority

- [x] **T001** Record canonical entry identity: `main=8a45857bf31c4acae57fdfb1e3cdde3d0f7d0361`, tree `ffaa14fdc7a738a771ac872e566ad1609eedf2cc`, CF-13 `CLOSED_CANONICAL`.
- [x] **T002** Audit current repository assurance gaps: unprotected `main`; mixed mutable/immutable workflow references; missing cargo-deny/cargo-audit/zizmor/Scorecard; no fuzz/mutation/coverage/portability/release assurance program; stale README capability surface.
- [x] **T003** Research current primary guidance for GitHub Actions full-SHA pinning, SLSA v1.2, Sigstore bundles, Rust fuzzing/mutation/coverage/security tooling, and HL7 FHIR release status.
- [x] **T004** Add `docs/COMMAND_F_ASSURANCE_PROGRAM_2026-08-26.md` and AF-01 Spec Kit planning package; preserve CF-14/15/16 identities.
- [x] **T005** Planning gate: exact final planning head passes all path-applicable CI, independent CodeRabbit/Qodo review truth is recorded without invented PASS, zero unresolved substantive planning findings remain, and planning PR is merged to canonical `main`.

## Phase 1 / Stack A — workflow trust audit and baseline hardening

Depends on T005.

- [x] **T010** Inventory every tracked `.github/workflows/*.yml|*.yaml`, every tracked Action metadata file named `action.yml` or `action.yaml` at any repository depth, every external `uses:` reference, runner label, workflow/job permission, checkout credential setting, job/service container image identity, and cargo lockfile-consuming command on canonical planning main.
- [x] **T011** Define a minimal checked-in AF-01 workflow-trust policy format that makes allowed workflow/job permissions and proof-container identity modes machine-checkable, including any narrowly scoped exception schema with reason/revisit condition.
- [x] **T012** Implement repository-owned deterministic workflow-trust audit with complete workflow plus `action.yml`/`action.yaml` discovery, local-action allowance, full-40-hex external action/reusable-workflow requirement, checkout credential check, effective workflow/job permission normalization plus allowlist enforcement, proof-critical job/service container digest enforcement, and proof-runner policy.
- [x] **T013** Add positive and counterexample tests for T012, including mutable external `uses:` in `action.yaml`, tag/branch/short-SHA rejection, missing `persist-credentials: false`, overbroad permission rejection, unresolved inherited/default permission rejection, proof-critical mutable job/service container rejection, new-workflow/action-metadata coverage, malformed input fail-closed behavior, and deterministic repeat output.
- [x] **T014** Harden `.github/workflows/ci.yml` to full-SHA external Actions, credentialless checkout, explicit machine-checkable least permissions, fixed supported runner label, bounded timeout, and preserved existing semantic/test steps.
- [x] **T015** Reconcile every other existing workflow and repository Action metadata file to the AF-01 baseline, including permission declarations and proof-critical container digest identity, without changing its product/oracle/proof semantics or path-filter authority except where later universal required-check aggregation is explicitly introduced.
- [x] **T016** Add a regression that discovers both `action.yml` and `action.yaml` anywhere in the tracked tree and fails if a future workflow, Action metadata file, permission grant, external Action ref, checkout credential setting, or proof-critical container identity escapes AF-01 trust auditing.
- [x] **T017** Run mandatory workspace gates and every path-applicable existing proof/oracle workflow on the exact Stack A head.
- [x] **T018** Request CodeRabbit and Qodo on exact Stack A head; disposition every substantive returned finding and require zero unresolved material review threads.
- [x] **T019** Merge Stack A only from its exact qualified head and record canonical merge/main/tree.

## Phase 2 / Stack B — dependency and CI security gates

Depends on canonical T019.

- [x] **T020** Inspect the exact current Cargo dependency graph and license/source metadata; document intended direct/transitive source and license policy before generating `deny.toml`.
- [x] **T021** Add checked-in `deny.toml` covering licenses, bans/duplicates, advisories, and sources with narrow reviewed exceptions only.
- [x] **T022** Add pinned `cargo-deny` execution in an independently diagnosable CI job; retain machine-readable or complete textual evidence.
- [x] **T023** Add pinned RustSec `cargo-audit` execution against exact `Cargo.lock`; retain advisory database/tool identity where available.
- [x] **T024** Define waiver documentation requirements for any advisory/security exception: identity, rationale, scope, compensating evidence, and revisit/removal condition.
- [x] **T025** Add pinned `zizmor` audit over all repository workflows/actions; freeze initial severity policy from observed baseline rather than guessing around findings.
- [x] **T026** Fix valid high/medium workflow findings or amend the plan/tasks with explicit reviewed disposition; do not lower the gate silently.
- [x] **T027** Add regressions proving dependency/workflow security configurations and both Action metadata filename forms are included in relevant workflow path/coverage logic so policy mutations cannot bypass gates.
- [ ] **T028** Run mandatory workspace gates plus all path-applicable existing proof/oracle workflows on exact Stack B head.
- [ ] **T029** Obtain and disposition CodeRabbit/Qodo review on exact Stack B head, merge only from exact qualified head, and record canonical merge/main/tree.

## Phase 3 / Stack C — posture evidence, AF-01 proof, and main enforcement

Depends on canonical T029.

- [ ] **T030** Add pinned OpenSSF Scorecard integration in least-authority mode appropriate for this public repository; retain per-check evidence and do not use aggregate score as commandF correctness authority. Any required write/id-token permission must be scoped to the exact Scorecard job and added to the checked-in permission policy.
- [ ] **T031** Inspect Scorecard results for at least Branch-Protection, Dangerous-Workflow, Pinned-Dependencies, Token-Permissions, Security-Policy where applicable, and Vulnerabilities; disposition material findings.
- [ ] **T032** Implement `.github/workflows/af01-assurance-proof.yml` with complete AF-01 policy coverage, including workflows and both `action.yml`/`action.yaml`, and immutable/pinned execution inputs consistent with commandF proof policy, including digest-pinned proof-critical job/service containers where containers are used.
- [ ] **T033** Define stable `assurance-summary.json` schema and deterministic `AF01_ASSURANCE_SHA256`, binding exact source/tree, policy/config blobs, workflow audit, dependency audit, RustSec audit, zizmor evidence, and tool/action/container identities.
- [ ] **T034** Add proof tests for repeated summary equality, source/tree mismatch, missing required evidence, malformed evidence, permission-policy mismatch, mutable proof-container identity, missing `action.yaml` coverage, and dirty/unexpected source where applicable.
- [ ] **T035** Determine final required status-check names **and trigger topology** from canonical implementation workflows. Prove every selected required check produces a terminal result on every protected-branch PR at the latest head; path-filtered whole workflows that can remain pending are not eligible as direct required checks.
- [ ] **T036** Add a required-check topology regression/counterexample: a docs-only or otherwise path-nonmatching PR must still receive a terminal result for every check selected by T035. Where heavy validation is conditional, add/identify an always-triggered lightweight aggregation gate that reports the applicable heavy-job result without forcing irrelevant heavy work.
- [ ] **T037** Prepare exact `main` ruleset configuration: PR required, at least one review, required conversations resolved, stale/latest-push review protection, only universally terminal selected status checks required, branch deletion/force-push blocked, and narrowly documented bypass actors only.
- [ ] **T038** Apply T037 through an authorized GitHub administrator path. Current connector read capability does not count as mutation authority.
- [ ] **T039** Query live GitHub after T038 and retain evidence proving the active ruleset/branch policy actually applies to `refs/heads/main` with intended enforcement and exact required-check names.
- [ ] **T040** Negative governance proof: demonstrate or otherwise verify from authoritative GitHub configuration/check topology that direct/force/deletion/stale-head bypasses are blocked and no selected required check can remain pending solely because an entire workflow was path/branch/commit-message skipped, without destructively rewriting repository history.
- [ ] **T041** Run exact-head AF-01 proof, mandatory workspace gates, and every path-applicable existing product proof/oracle workflow; retain artifact IDs/digests and tool/source identities.
- [ ] **T042** Obtain exact-head CodeRabbit/Qodo review; require zero unresolved substantive findings.
- [ ] **T043** Merge Stack C only from exact qualified head and verify post-merge `main`, tree, proof applicability, universal required-check topology, and live ruleset state.

## Phase 4 — convergence

Depends on T043.

- [ ] **T050** Re-read `spec.md`, `plan.md`, `tasks.md`, assurance-program document, constitution, AGENTS, live GitHub policy, and implementation tree; reconcile any drift.
- [ ] **T051** Create `convergence.md` recording planning/Stack A/B/C identities, workflow run/job/artifact/digest evidence, dependency/security tool identities, reviewer dispositions, live ruleset evidence, required-check topology, limits, and deferrals.
- [ ] **T052** Confirm product-semantic diff from pre-AF-01 canonical base contains no unauthorized CF semantic change; any incidental product source mutation requires separate task/justification and full semantic qualification.
- [ ] **T053** Record remaining assurance work under AF-02/AF-03/AF-04 rather than falsely claiming fuzz/mutation/portability/release/performance completion.
- [ ] **T054** Exact convergence head receives path-applicable CI/review truth with zero unresolved substantive findings.
- [ ] **T055** Merge convergence PR and verify canonical post-merge main/tree plus live source-control policy and universally terminal required checks.
- [ ] **T056** Mark `AF-01=CLOSED_CANONICAL` only after T055 evidence is complete.

## AF-02 handoff retained, not authorized by AF-01 implementation

After AF-01 closes, create a separate Spec Kit unit for adversarial test strength covering:

- structure-aware/differential `cargo-fuzz`;
- property tests;
- `cargo-mutants` mutation adequacy;
- `cargo-llvm-cov` diagnostic floors;
- `cargo-nextest` flaky-as-failure policy;
- minimized regression corpus promotion.

## AF-03 handoff retained

- Linux/Windows/macOS qualification;
- explicit MSRV proof;
- Rust public API/SemVer guard;
- SBOM;
- SLSA/GitHub artifact provenance;
- Sigstore/offline verification where adopted;
- stable release verification.

## AF-04 handoff retained

- performance/resource benchmark corpus;
- measured regression budgets;
- large package/graph stress;
- external-service sentinel separation;
- trend evidence reusable by future commandF Bench.
