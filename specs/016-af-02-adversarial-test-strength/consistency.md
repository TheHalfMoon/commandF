# AF-02 Consistency Analysis

Status: PLANNING_CANDIDATE

## Scope

This analysis reconciles:

- `.specify/memory/constitution.md`;
- `AGENTS.md`;
- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`;
- `docs/COMMAND_F_PLAN_INDEX.md`;
- `docs/COMMAND_F_ASSURANCE_PROGRAM_2026-08-26.md`;
- canonical AF-01 `spec.md`, `plan.md`, `tasks.md`, convergence/closeout state, and live rulesets;
- current `commandf-pkg` source boundaries at canonical AF-02 planning base;
- `donors/af-02-adversarial-testing.yaml`;
- AF-02 `spec.md`, `plan.md`, and `tasks.md`;
- exact upstream tool identities inspected during planning.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

## Resolved consistency questions

### 1. Does `016-af-02-*` rename or consume CF-16?

No.

The directory number is only the next Spec Kit package sequence. Product identities remain:

```text
CF-14 on-prem aggregate-only source profiler
CF-15 verified dry-run recipes
CF-16 mapping analysis IR, parse-only
```

`AF-02` is the second Assurance Foundation unit and is orthogonal to CF numbering.

### 2. Is AF-02 authorized to begin after AF-01?

Planning is authorized by canonical architecture, assurance program, and AF-01 handoff. AF-01 explicitly retained a separate AF-02 unit for fuzzing, property tests, mutation adequacy, coverage floors, nextest flaky-as-failure, and minimized regression corpus promotion.

Implementation is **not** authorized merely because AF-01 closed. This AF-02 planning package must first become canonical through T006.

### 3. Does AF-02 violate the constitution's vertical-capability rule because it does not ship a product command?

No.

The constitution explicitly permits an independently executable verification result. AF-02 ships adversarial-test qualification and a retained exact-head proof artifact. It is not an empty testing scaffold.

### 4. Does AF-02 replace deterministic product authority with fuzzing?

No.

Fuzz discovery is stochastic and is intentionally classified as discovery evidence. No-crash duration/execution count is never called a product correctness proof.

Canonical deterministic authority remains in product validators, exact regression/property evidence, authoritative oracles where already defined, and existing CF policies. AF-02 strengthens the test evidence around them.

### 5. Is stochastic fuzzing compatible with the constitution's determinism principle?

Yes, because the plan separates discovery from qualification.

Deterministic AF-02 identity covers exact source/policy/tool identities, target build, committed corpus replay, property assertions, nextest result, fixed coverage inputs, classified mutation evidence, and normalized summary construction.

Stochastic campaigns retain seed/budget/corpus/tool metadata when available, but their exploration path and wall-clock timing are not placed into a semantic deterministic digest. A no-crash campaign is a bounded observation only.

### 6. Does cargo-fuzz require commandF's stable workspace to move to nightly?

No.

The product workspace remains on Rust `1.97.1`. AF-02 uses an isolated fuzz-only toolchain pinned to `nightly-2026-08-25`. No stable-release/MSRV claim is derived from the fuzz compiler.

### 7. Does adding fuzz-only crates violate the no-new-crate rule?

No new commandF product crate is planned.

`libfuzzer-sys`, `arbitrary`, and related fuzz dependencies live in the dedicated fuzz harness and execute immediately as an assurance capability. They are not product runtime dependencies. `proptest` is test/dev-only and immediately backs executable property tests.

### 8. Can private parser helpers simply be made public for fuzzing?

No.

The plan requires existing public product paths where possible, colocated unit/property tests for private pure helpers, or a behavior-preserving internal test seam. Adding public API solely for harness convenience is explicitly prohibited.

This preserves future AF-03 public API/SemVer authority and avoids testing architecture dictating product API.

### 9. Why target archive/package parsing through a public path rather than `read_manifest` directly?

`read_manifest` is currently private and its limits are implementation details behind package behavior. Fuzzing through public package inspection/acquisition paths exercises the real product boundary and avoids API pollution.

A private colocated property/unit test may still directly verify internal limit/canonicalization helpers when that is stronger evidence.

### 10. Does source-map filesystem fuzzing create a path-escape risk?

The harness itself must be fail-closed and isolated.

Generated filesystem cases use a dedicated temporary root, no network, bounded sizes, and explicit checks that generated paths never escape the harness root. The product's canonicalization/source-root containment logic remains the subject under test, not a reason to permit arbitrary host filesystem access.

### 11. Does property testing make example-based tests redundant?

No.

Property tests broaden input families and shrink counterexamples. Canonical example/regression tests remain required because they document specific contracts and reproduce known defects exactly.

Every AF-02 discovered defect must end as a deterministic minimized regression, not only as a property seed hidden in a test runner cache.

### 12. Is mutation score a correctness score?

No.

AF-02 explicitly rejects aggregate mutation percentage as sole authority. The stronger closure contract is classification of the required mutation target set: every required survivor is killed or narrowly waived, while timeout/build-failure/equivalent classifications remain separate.

This prevents a high aggregate score from hiding one false-PASS-sensitive surviving mutation.

### 13. Can cargo-mutants timeout/build failures count as killed mutants?

Not silently.

`KILLED`, `SURVIVED`, `TIMEOUT`, `UNVIABLE_OR_BUILD_FAILURE`, and reviewed waiver states are distinct. Tests only prove adequacy when they actually detect a viable behavior mutation.

### 14. Is a zero-survivor policy realistic in the presence of equivalent mutants?

The policy is zero **unclassified required survivors**, not zero raw tool survivors.

Equivalent/trivial/out-of-scope mutations can be waived only with exact source/mutation identity, rationale, compensating evidence, revisit condition, and removal condition. Broad exclusions for critical modules are prohibited.

### 15. Is coverage percentage a correctness gate?

Coverage is a regression diagnostic, not semantic authority.

Floors are derived from the first measured canonical baseline rather than guessed. They prevent unexplained test exercise regression; they do not prove behavior is correct. Function/region evidence is retained alongside line coverage, and critical-module coverage is preferred over one vanity workspace number.

### 16. Why not set 90%/100% coverage in planning?

Because that would be an unmeasured vanity target and could incentivize meaningless tests or broad exclusions.

The plan requires measuring exact current coverage first, freezing a reproducible floor from that evidence, then improving coverage through meaningful properties/regressions over time. A floor cannot be lowered in the same change merely to regain green.

### 17. Does nextest replace `cargo test`?

No.

`cargo test --workspace --all-features` remains mandatory. Nextest adds bounded retry diagnostics and explicit flaky-as-failure behavior. It must not hide doctest or harness differences.

### 18. Why allow retries if retry-pass is failure?

Retries are diagnostic evidence: they distinguish a reproducible failure from an intermittent one. `flaky-result = "fail"` prevents the retry from manufacturing green.

The AF-02 self-test must prove this behavior without committing a permanently flaky product test.

### 19. Does AF-02 alter the AF-01 live ruleset?

No by default.

Current required contexts remain `rust`, `assurance-proof`, and `scorecard`. AF-02 heavy/path-applicable workflows can be non-required.

Any later proposal to require an AF-02 aggregation check must separately prove universal terminal topology, update checked-in ruleset intent, receive review, be applied by authorized GitHub administration, and be live-read back. AF-02 closure cannot assume such a change.

### 20. Does AF-02 weaken AF-01 workflow-trust rules for third-party testing tools?

No.

AF-02 workflows/actions must satisfy canonical AF-01 full-SHA external Action pinning, least permissions, credentialless checkout, fixed runner policy, bounded timeout, and proof identity rules. AF-02 adds its workflows/config paths to existing assurance coverage rather than bypassing it.

Tool installation/version pins are separate from GitHub Action `uses:` pins; both identities are retained where applicable.

### 21. Are the planned tool identities concrete rather than mutable aliases?

Yes.

Planning resolved:

```text
cargo-fuzz 0.13.2
  commit 984c861c8dfea28055254c5f1d2659ab2cd63f76

cargo-mutants 27.1.0
  signed tag resolves to commit 8ab1dc786a1f61a4e370416cc6c68b81a704e917

cargo-llvm-cov 0.9.0
  commit be59056988acd54c7f984b7c85643daea3711b29

cargo-nextest 0.9.143
  annotated tag resolves to commit 60fa45f638ffc3f35e74afa65737f45fcd32db2a

proptest =1.11.0
  upstream manifest rust-version 1.86

libfuzzer-sys =0.4.13
arbitrary =1.4.2
fuzz nightly = nightly-2026-08-25
```

Implementation still records exact package/binary/checksum identities actually installed. A release tag name alone is not sufficient retained evidence when a commit/checksum is available.

### 22. Does AF-02 authorize CF-14 implementation or patient-instance fuzzing?

No.

CF-14 has its own required Spec Kit. AF-02 uses synthetic/public conformance artifacts only and introduces no real patient-instance/PHI data.

Once CF-14 later introduces an instance-data parser/boundary, that boundary must enter the canonical AF-02 surface/property/fuzz inventory before CF-14 can close canonically. This is a future closure dependency, not present implementation authority.

### 23. Does AF-02 change CF-06 production oracle identity or CF-10 frozen corpus?

No.

Any differential/cross-path testing involving existing oracle contracts uses the already canonical CF-06 identity. AF-02 cannot repin the production validator or change CF-10 frozen corpus membership under test-strength work.

### 24. Could fuzzing be called an authoritative oracle?

No.

Fuzzing generates/explores inputs. It does not establish clinical/FHIR semantic truth. Where a cross-path comparison exists, the plan names the actual authority. Two paths backed by the same implementation are not falsely labeled independent oracles.

### 25. Does AF-02 own performance/stress testing?

Only enough execution bounds to keep adversarial testing safe.

Measured product performance/resource budgets, large-package/large-graph stress, and trend regression belong to AF-04. AF-02 must not turn fuzz timeouts into benchmark claims.

### 26. Does AF-02 own portability/release assurance?

No.

Windows/macOS/Linux qualification, MSRV release proof, SBOM, SLSA/provenance/signing, public API/SemVer, and stable release verification remain AF-03.

### 27. Can a fuzz/property crash be fixed without adding a committed regression?

Not for AF-02 closure.

The failure must be reproduced and minimized/shrunk, classified, given stable scenario/digest/provenance, and executed through deterministic regression/corpus replay before the fix task closes. This prevents rediscovery-only evidence from disappearing with local fuzz artifacts.

### 28. What if the minimized reproducer exceeds the default 256 KiB policy?

The limit is a reviewability default, not a semantic rejection threshold. A larger fixture is allowed only with explicit rationale/provenance and bounded aggregate-corpus review. AF-04 remains the preferred home for generic large-scale stress fixtures.

### 29. Could the first coverage measurement be manipulated by choosing a favorable test subset?

The baseline scope is part of checked-in policy and exact evidence. It must include the canonical workspace test suite plus the exact AF-02 replay/property inputs selected by the implementation plan. Exclusions are explicit/path-specific and independently reviewed.

The exact source/compiler/tool/test/corpus identities are retained so a later result cannot silently compare a different scope.

### 30. Can planning tasks that depend on future CI/review be marked complete in this authored commit?

No.

T001-T005 are only completed when their exact evidence exists on the final planning candidate. T006 includes exact-head CI/review, merge, and post-merge re-read, so its completion is inherently temporal and cannot be truthfully embedded before the merge without creating circular/stale evidence.

Planning qualification identifiers therefore belong in the PR conversation/checkpoint. After the planning merge, the first AF-02 implementation branch must reconcile T001-T006 task state in a docs-only leading commit/checkpoint before product/test implementation changes begin, or use an equivalent dedicated state-reconciliation PR. No implementation may rely on an unproven planning merge.

### 31. Does the donor record mean commandF copies these tools' source code?

No.

The donor manifest classifies them primarily as development-tool dependencies and design references. No source vendoring is planned. If implementation later copies code/patterns beyond ordinary API/config use, it must add exact source paths, preserve license notices, and update provenance before merge.

## Requirement-to-task trace

| Requirement | Tasks |
|---|---|
| FR-001 surface inventory | T002, T010-T012, T058, T070 |
| FR-002 isolated exact fuzz tool identity | T003, T013, T021, T060 |
| FR-003 raw/structured fuzzing | T014-T018, T021, T055 |
| FR-004 differential/cross-path invariants | T015-T018, T060 |
| FR-005 property tests | T015-T018, T021, T060 |
| FR-006 corpus promotion | T019-T020, T021, T060, T073 |
| FR-007 mutation adequacy | T050-T054, T056-T057, T060, T073 |
| FR-008 coverage floors | T034-T038, T056-T057, T060 |
| FR-009 flaky-as-failure | T030-T033, T038, T056-T057, T060 |
| FR-010 deterministic vs stochastic evidence | T011-T021, T055-T057, T060, T071 |
| FR-011 bounded CI topology | T021, T038, T055-T060 |
| FR-012 exact-head AF-02 proof | T056-T057, T060-T062, T071-T078 |
| FR-013 product/assurance non-regression | T010, T022, T039, T058-T062, T072 |
| FR-014 reviewer truth | T006, T023, T040, T061, T075, T077 |
| NFR-001 determinism | T012-T020, T035-T037, T056-T057 |
| NFR-002 fail closed | all validation/gate tasks |
| NFR-003 bounded resources | T013-T021, T030-T038, T050-T060 |
| NFR-004 no hidden retries | T031-T033, T056-T057 |
| NFR-005 no PHI | all tasks |
| NFR-006 exact provenance | T003, T013, T030, T034, T050, T056-T060 |
| NFR-007 stackability | phase ordering T010-T078 |
| NFR-008 no vanity metric | T035-T037, T051-T054, T071 |

## Known planning risks retained explicitly

1. **Private helper reachability.** Some high-value source-map/archive parser functions are private. Stack A must use real public boundaries or internal test seams without creating public API solely for fuzzing.
2. **Fuzz nightly drift.** A dated nightly is pinned, but libFuzzer/compiler behavior differs from stable product execution. Fuzz evidence therefore never becomes an MSRV/stable-release claim.
3. **CI cost.** Mutation and fuzz discovery can be expensive. The plan deliberately separates PR deterministic qualification from scheduled discovery and targeted mutation rather than silently skipping evidence.
4. **Coverage instrumentation drift.** Tool/compiler identity is part of the baseline. Floor changes caused by instrumentation updates require explicit re-baselining, not automatic tolerance.
5. **Equivalent mutants.** Zero raw survivors is unrealistic in some code. The policy requires zero unclassified required survivors with narrow reviewable waivers.
6. **False fuzz confidence.** No-crash duration/execution count is explicitly non-authoritative.
7. **Corpus growth.** Only minimized stable regressions are committed; generic discovery corpora/artifacts remain generated.
8. **Required-check deadlock.** AF-02 does not add a live required check by default. Any future addition repeats AF-01 universal-terminal topology/live-read-back discipline.
9. **Future CF-14 boundary.** AF-02 can close before CF-14 exists, but CF-14 later cannot close without adding its new input parser boundary to AF-02 adversarial coverage.

## Final planning consistency result

No unresolved architecture contradiction is known in the authored AF-02 package.

The remaining conditions are temporal qualification, not hidden design assumptions:

```text
AF-02 PLANNING CONSISTENCY: CANDIDATE / REQUIRES EXACT-HEAD CI + INDEPENDENT REVIEW
AF-01: CLOSED_CANONICAL / AUTHORITY PRESERVED
PRODUCT IDENTITIES CF-14/15/16: PRESERVED
CF-06/CF-10 AUTHORITY: UNCHANGED
REAL PATIENT DATA / PHI: OUT OF SCOPE
IMPLEMENTATION AUTHORITY: NOT YET GRANTED
```
