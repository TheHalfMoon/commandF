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
- [ ] **T005** Planning gate: exact final planning head passes all path-applicable CI, independent CodeRabbit/Qodo review truth is recorded without invented PASS, zero unresolved substantive planning findings remain, and planning PR is merged to canonical `main`.

## Phase 1 / Stack A — workflow trust audit and baseline hardening

Depends on T005.

- [ ] **T010** Inventory every tracked `.github/workflows/*.yml|*.yaml`, repository composite Action, external `uses:` reference, runner label, workflow/job permission, checkout credential setting, job/service container image identity, and cargo lockfile-consuming command on canonical planning main.
- [ ] **T011** Define a minimal checked-in AF-01 workflow-trust policy format that makes allowed workflow/job permissions and proof-container identity modes machine-checkable, including any narrowly scoped exception schema with reason/revisit condition.
- [ ] **T012** Implement repository-owned deterministic workflow-trust audit with complete tracked-workflow discovery, local-action allowance, full-40-hex external action requirement, checkout credential check, effective workflow/job permission normalization plus allowlist enforcement, proof-critical job/service container digest enforcement, and proof-runner policy.
- [ ] **T013** Add positive and counterexample tests for T012, including tag/branch/short-SHA rejection, missing `persist-credentials: false`, overbroad permission rejection, unresolved inherited/default permission rejection, proof-critical mutable job/service container rejection, new-workflow coverage, malformed input fail-closed behavior, and deterministic repeat output.
- [ ] **T014** Harden `.github/workflows/ci.yml` to full-SHA external Actions, credentialless checkout, explicit machine-checkable least permissions, fixed supported runner label, bounded timeout, and preserved existing semantic/test steps.
- [ ] **T015** Reconcile every other existing workflow to the AF-01 baseline, including permission declarations and proof-critical container digest identity, without changing its product/oracle/proof semantics or path-filter authority.
- [ ] **T016** Add a regression that scans all tracked workflow/action files and fails if a future workflow, permission grant, external Action ref, checkout credential setting, or proof-critical container identity escapes AF-01 trust auditing.
- [ ] **T017** Run mandatory workspace gates and every path-applicable existing proof/oracle workflow on the exact Stack A head.
- [ ] **T018** Request CodeRabbit and Qodo on exact Stack A head; disposition every substantive returned finding and require zero unresolved material review threads.
- [ ] **T019** Merge Stack A only from its exact qualified head and record canonical merge/main/tree.

## Phase 2 / Stack B — dependency and CI security gates

Depends on canonical T019.

- [ ] **T020** Inspect the exact current Cargo dependency graph and license/source metadata; document intended direct/transitive source and license policy before generating `deny.toml`.
- [ ] **T021** Add checked-in `deny.toml` covering licenses, bans/duplicates, advisories, and sources with narrow reviewed exceptions only.
- [ ] **T022** Add pinned `cargo-deny` execution in an independently diagnosable CI job; retain machine-readable or complete textual evidence.
- [ ] **T023** Add pinned RustSec `cargo-audit` execution against exact `Cargo.lock`; retain advisory database/tool identity where available.
- [ ] **T024** Define waiver documentation requirements for any advisory/security exception: identity, rationale, scope, compensating evidence, and revisit/removal condition.
- [ ] **T025** Add pinned `zizmor` audit over all repository workflows/actions; freeze initial severity policy from observed baseline rather than guessing around findings.
- [ ] **T026** Fix valid high/medium workflow findings or amend the plan/tasks with explicit reviewed disposition; do not lower the gate silently.
- [ ] **T027** Add regressions proving dependency/workflow security configurations are included in relevant workflow path filters so policy mutations cannot bypass gates.
- [ ] **T028** Run mandatory workspace gates plus all path-applicable existing proof/oracle workflows on exact Stack B head.
- [ ] **T029** Obtain and disposition CodeRabbit/Qodo review on exact Stack B head, merge only from exact qualified head, and record canonical merge/main/tree.

## Phase 3 / Stack C — posture evidence, AF-01 proof, and main enforcement

Depends on canonical T029.

- [ ] **T030** Add pinned OpenSSF Scorecard integration in least-authority mode appropriate for this public repository; retain per-check evidence and do not use aggregate score as commandF correctness authority. Any required write/id-token permission must be scoped to the exact Scorecard job and added to the checked-in permission policy.
- [ ] **T031** Inspect Scorecard results for at least Branch-Protection, Dangerous-Workflow, Pinned-Dependencies, Token-Permissions, Security-Policy where applicable, and Vulnerabilities; disposition material findings.
- [ ] **T032** Implement `.github/workflows/af01-assurance-proof.yml` with complete AF-01 path coverage and immutable/pinned execution inputs consistent with commandF proof policy, including digest-pinned proof-critical job/service containers where containers are used.
- [ ] **T033** Define stable `assurance-summary.json` schema and deterministic `AF01_ASSURANCE_SHA256`, binding exact source/tree, policy/config blobs, workflow audit, dependency audit, RustSec audit, zizmor evidence, and tool/action/container identities.
- [ ] **T034** Add proof tests for repeated summary equality, source/tree mismatch, missing required evidence, malformed evidence, permission-policy mismatch, mutable proof-container identity, and dirty/unexpected source where applicable.
- [ ] **T035** Determine final required status-check names from canonical implementation workflows; do not guess names before they exist.
- [ ] **T036** Prepare exact `main` ruleset configuration: PR required, at least one review, required conversations resolved, stale/latest-push review protection, selected status checks required, branch deletion/force-push blocked, and narrowly documented bypass actors only.
- [ ] **T037** Apply T036 through an authorized GitHub administrator path. Current connector read capability does not count as mutation authority.
- [ ] **T038** Query live GitHub after T037 and retain evidence proving the active ruleset/branch policy actually applies to `refs/heads/main` with intended enforcement.
- [ ] **T039** Negative governance proof: demonstrate or otherwise verify from authoritative GitHub configuration that direct/force/deletion/stale-head bypasses are blocked according to T036 without destructively rewriting repository history.
- [ ] **T040** Run exact-head AF-01 proof, mandatory workspace gates, and every path-applicable existing product proof/oracle workflow; retain artifact IDs/digests and tool/source identities.
- [ ] **T041** Obtain exact-head CodeRabbit/Qodo review; require zero unresolved substantive findings.
- [ ] **T042** Merge Stack C only from exact qualified head and verify post-merge `main`, tree, proof applicability, and live ruleset state.

## Phase 4 — convergence

Depends on T042.

- [ ] **T050** Re-read `spec.md`, `plan.md`, `tasks.md`, assurance-program document, constitution, AGENTS, live GitHub policy, and implementation tree; reconcile any drift.
- [ ] **T051** Create `convergence.md` recording planning/Stack A/B/C identities, workflow run/job/artifact/digest evidence, dependency/security tool identities, reviewer dispositions, live ruleset evidence, limits, and deferrals.
- [ ] **T052** Confirm product-semantic diff from pre-AF-01 canonical base contains no unauthorized CF semantic change; any incidental product source mutation requires separate task/justification and full semantic qualification.
- [ ] **T053** Record remaining assurance work under AF-02/AF-03/AF-04 rather than falsely claiming fuzz/mutation/portability/release/performance completion.
- [ ] **T054** Exact convergence head receives path-applicable CI/review truth with zero unresolved substantive findings.
- [ ] **T055** Merge convergence PR and verify canonical post-merge main/tree plus live source-control policy.
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
