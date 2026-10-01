# commandF Plan Index

Status: **authoritative plan-set index**

commandF is intentionally planned as a small execution spine plus a larger preserved product/discovery/research envelope. No single document should mix immediate implementation authority with every candidate donor, product capability, and research hypothesis.

The commandF plan therefore consists of the following layers.

## A. Execution authority

`docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`

Defines product boundary, architecture planes, first execution stack, trust boundaries, mandatory gates, and the relationship between cross-cutting Assurance Foundation work and CF product slices. When it conflicts with older bootstrap planning, V2 controls execution order.

## B. Discovery coverage authority

`docs/COMMAND_F_DISCOVERY_COVERAGE_2026-08-13.md`

Preserves the open-source projects, standards, tools, product inspirations, runtime candidates, validators/oracles, test frameworks, provenance/supply-chain tooling, benchmarks, and research directions discovered before and during V2.

A candidate appearing there is **retained in the plan**, not automatically adopted.

Coverage corrections explicitly retained in addition to the named annex entries:

- **MLIR** — architecture-study donor for multi-dialect typed IR, verification passes, canonicalization, and lowering. It does not force an LLVM dependency.
- **GoFSH** — FSH ecosystem tooling/reference alongside SUSHI and IG Publisher.
- **Open Concept Lab (OCL)** — terminology-service/reference candidate alongside Snowstorm, TermX, Hades, and OHDSI vocabulary tooling.

## C. Product-family authority

`docs/COMMAND_F_PRODUCT_FAMILY.md`

Preserves the long-term commandF capability family discussed during discovery:

- commandF Core
- commandF Studio
- commandF Registry
- commandF Verify
- commandF Gateway
- commandF Query
- commandF Bench
- commandF Copilot
- commandF Trust

These are capability groupings, not authorization to create separate services or repositories now. The V2 execution sequence still decides what is built and when.

## D. Problem/gap authority

`docs/COMMAND_F_GAP_LEDGER_2026-08-13.md`

Preserves the 35 interoperability gap hypotheses that motivate the product and research program, and maps them to the commandF response.

## E. Donor/provenance authority

`docs/PROVENANCE_AND_DONOR_POLICY.md`

Defines adoption modes and the pin/license/permission/source-path requirements that must be satisfied before candidate prior art becomes adopted commandF code/data/mappings.

Current slice-specific donor records remain under `donors/`, including:

- `donors/cf-01-package-resolution.yaml`
- `donors/agent-harness-2026-08-13.yaml`

Future slice plans must add or update donor records rather than relying on conversation memory.

## F. Research authority

`research/RESEARCH_CHARTER.md`

Preserves the candidate master's thesis, core research question, H1–H3, initial standards/models, baseline families, measurement framework, experiment sequence, reproducibility artifact, and evidence/data governance.

`research/BENCHMARK_ADMISSION_LIFECYCLE.md` orders source evidence, labels, candidate membership, split assignment, and the final freeze so those steps do not wait on each other in a circle. It admits no item by itself. `research/CANDIDATE_CORPUS.md` records candidate membership. Membership is not a protocol freeze. `research/CASE_CLASS_MATRIX.md` records which remaining classes are blocked. It does not admit an item. `research/SPLIT_POLICY.md` version `sp-1` specifies the assignment function. It does not assign the current corpus. `research/BASELINE_IDENTITIES.md` names which planned baseline families have a command. It does not run them. `research/STATISTICAL_ANALYSIS_PLAN.md` version `sap-1` specifies the calculations. It does not compute them. `research/manuscript/commandfbench-methods.md` states the admission, split, baseline, and analysis records. It reports no measurement. `research/manuscript/problem-definition.md` defines the decision problem. It does not implement the planned decision envelope. `research/manuscript/compatibility-model.md` separates the shipped `Breaking`/`Risky`/`Additive` findings from the planned truth classes. `research/manuscript/consumer-contract-model.md` separates the shipped graph edges from the planned consumer contract. `research/manuscript/evidence-and-decision-assurance.md` records the shipped cache digest check and leaves the decision receipt unimplemented. `research/manuscript/selective-oracle-escalation.md` records the caller-invoked oracle and leaves automatic escalation unimplemented. `research/manuscript/experimental-design.md` states the unfrozen design. It does not run an experiment. `research/manuscript/threats-to-validity.md` records the pre-measurement threats. It does not estimate bias. `research/manuscript/limitations.md` records the current limits, including the unassigned split and the blocked paid Jev path. `research/manuscript/reproducibility.md` records the pinned identities and states that no reproduction package exists.

The broader research inventory remains in Sections 21–23 of `docs/COMMAND_F_DISCOVERY_COVERAGE_2026-08-13.md` and includes sixteen retained tracks:

1. Semantic Conservation across FHIR/openEHR/OMOP
2. Semantic Conservation measurement
3. cross-standard round-trip benchmark
4. commandF Bench
5. IG/Profile conflict detection and harmonization
6. empirical FHIR server compatibility
7. differential FHIRPath semantics
8. constrained AI mapping
9. terminology gap registry
10. provenance-complete transformations
11. Transformation Certificates
12. cross-model Clinical Query IR
13. AI-oriented clinical serialization
14. concurrency/transaction safety
15. imaging semantic bridge
16. data quality / AI readiness

Master's priority remains R1–R4. Research hypotheses never become product guarantees without executed evidence.

## G. Feature execution units

Every `CF-*` product slice is a Spec Kit-style feature unit:

```text
spec.md -> plan.md -> tasks.md -> implementation -> deterministic validation -> convergence.md
```

The same process is used for an `AF-*` Assurance Foundation unit when it creates independently executable verification authority around the repository rather than product semantics.

Current canonical execution truth at the creation of the Assurance Program:

```text
CF-13: CLOSED_CANONICAL
main: 8a45857bf31c4acae57fdfb1e3cdde3d0f7d0361
next product identity: CF-14
current cross-cutting planning unit: AF-01
```

## H. Assurance-program authority

`docs/COMMAND_F_ASSURANCE_PROGRAM_2026-08-26.md`

Preserves and sequences the cross-cutting work required to make commandF's own development/release evidence as rigorous as its interoperability evidence.

Assurance units use `AF-*` identities and **do not renumber product CF slices**.

Program units retained:

1. **AF-01 Trusted Development Baseline** — source-control enforcement, immutable workflow references, least authority, dependency/license/source/advisory policy, CI/CD static analysis, exact-head assurance proof.
2. **AF-02 Adversarial Test Strength** — structure-aware/differential fuzzing, property tests, mutation adequacy, coverage diagnostics/floors, flaky-as-failure execution, minimized regression corpus.
3. **AF-03 Portability and Release Evidence** — Linux/Windows/macOS, MSRV, public API/SemVer guard, SBOM, SLSA-compatible provenance, artifact/signature verification.
4. **AF-04 Performance and Reliability Evidence** — measured benchmark/resource budgets, large-input stress, external-sentinel separation, retained trends reusable by future commandF Bench.

Immediate authorized planning package once this index update is canonical:

`specs/015-af-01-trusted-development-baseline/`

Ordering rule:

- AF-01 must close before a new post-CF-13 product implementation is merged.
- CF-14 planning may proceed in parallel under its own Spec Kit authority.
- AF-02/03/04 remain retained program units and require their own planning packages before implementation. `specs/049-af02-review-gate-block/` records that a review-text edit of `specs/016-af-02-adversarial-test-strength/` was rejected by the AF-02 base gate. `specs/050-af02-precanonical-path-gap/` records that every absent inventory path is unexecuted data, so none is a strengthening. AF-02 planning stays a candidate. Stack A0 stays unauthorized.

## Coverage rule

Before a future architecture supersedes V2, its review must reconcile this entire plan set. A candidate, product capability, gap, assurance unit, or research track may be:

- adopted
- retained for later
- explicitly rejected with rationale
- superseded by better evidence

It may **not** disappear silently.

## Build-order rule

Preserving a candidate in the plan does not allow it to bypass the V2 execution sequence. A donor/tool/capability is activated only when a concrete CF or AF unit requires it and its provenance/adoption gate is satisfied.

Assurance tooling is not exempt from this rule: naming cargo-fuzz, SLSA, Sigstore, Scorecard, cargo-deny, cargo-audit, zizmor, or any other tool in discovery/program documents is not adoption until the relevant AF plan pins the exact implementation identity and acceptance boundary.

## I. 2026-09-12 V3 strategic planning overlay

The following planning artifacts were produced from a whole-plan review against canonical `main` at `c56d9619758ea3e1075e751bec83ddea6706a81e`.

The V3 candidate documents are:

- `docs/COMMAND_F_MASTER_ARCHITECTURE_V3_CANDIDATE.md`
- `docs/COMMAND_F_PLAN_GAP_REVIEW_2026-09-12.md`
- `docs/COMMAND_F_OPEN_SOURCE_QUALIFICATION_2026-09-12.md`
- `docs/COMMAND_F_V3_EXECUTION_PLAYBOOK.md`

They are **planning candidates only**. They do not supersede V2 execution authority, mutate an active AF/CF task frontier, or authorize production implementation by themselves.

The V3 candidate preserves CF-01..CF-16 and AF-01..AF-04 identities and proposes post-CF-16 candidate units for:

- ecosystem observation and historical standards/package drift;
- machine-verifiable compatibility change-space coverage;
- consumer contract scanning and protected consumer/version matrices;
- differential Compatibility Lab execution;
- Interoperability BOM and reproducible source-to-package build evidence;
- deterministic TestGen;
- stable developer platform/API/LSP surfaces;
- capability-scoped Wasm plugin/policy extensions;
- terminology federation/gap intelligence;
- transformation evidence/certificates;
- public compatibility evidence; and
- benchmark/research convergence.

No candidate in this overlay may bypass the existing build-order rule. `COMMAND_F_V3_EXECUTION_PLAYBOOK.md` remains the base implementation-facing navigation document for the V3 candidate roadmap: it defines the dependency DAG, closure contract for every G01-G40 gap, per-slice work packages, acceptance evidence, and agent operating loop. A future V3 execution migration requires its own reconciliation/qualification gate after currently governed prerequisites are satisfied.

## J. 2026-09-29 V3.1 decision-assurance hardening overlay

The following artifacts strengthen V3 without renumbering existing CF/AF work:

- `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md`
- `docs/COMMAND_F_V3_1_IMPLEMENTATION_RUNBOOK.md`
- `docs/COMMAND_F_V3_1_COMPLETENESS_AUDIT_2026-09-29.md`
- `docs/COMMAND_F_V3_1_REVIEW_GOVERNANCE.md`
- `docs/commandf-v3_1-internal-pattern-sources-2026-09-29.yaml`

The V3.1 overlay adds ten explicit gaps (`G41..G50`) and binds them to existing candidate slice ownership rather than inventing a parallel roadmap. It formalizes:

- proof versus probability/confidence;
- deterministic truth class versus policy action;
- evidence completeness and sufficiency;
- first-class abstention/insufficient/unsupported/indeterminate outcomes;
- a versioned Decision Envelope;
- a portable Consumer Contract;
- selective oracle/runtime escalation;
- a source/evidence-bound Decision Receipt;
- counterfactual sensitivity and irrelevant-edit stability;
- decision-quality evaluation centered on unsafe auto-allow risk at declared coverage;
- optional provider-neutral model assistance that is technically non-authoritative;
- evidence-authority/reconciliation patterns where missing or unsupported coverage can never become a clean result by absence; and
- shared assurance primitives so future product slices reuse evidence/integrity controls instead of forking them.

The V3.1 donor record pins founder-controlled GAX, Ascout, Sentrdel, Kernux, and Cotra revisions as `PATTERN_ONLY` sources. Pinning does not authorize a runtime dependency, model authority, or code copy without the normal slice-specific provenance/license/rights gate.

The completeness audit marks the combined V3 + V3.1 planning package `READY_FOR_SPEC_KIT_SHAPING / NOT EXECUTION AUTHORITY`. This means the plan is detailed enough to shape dependency-eligible Spec Kits without inventing missing architecture; it does not mean any future capability is implemented or proven.

The review-governance candidate records the intended future V3/V3.1 independent-review stack: repository-owned deterministic/assurance gates, Jev, Alibaba Open Code Review, and human disposition/merge authority. CodeRabbit, Qodo, Cubic, and similar hosted reviewer scores/checkmarks are not future V3 qualification evidence unless a later canonical governance change explicitly re-authorizes one. Current canonical governance still controls until the V3 migration gate reconciles it.

For any future V3 implementation planning, agents must read **both** `COMMAND_F_V3_EXECUTION_PLAYBOOK.md` and `COMMAND_F_V3_1_IMPLEMENTATION_RUNBOOK.md`. The latter is the mandatory amendment for G41-G50 and decision-assurance semantics. If the two planning candidates conflict, the narrower V3.1 rule controls only the decision-assurance subject it explicitly amends; neither document supersedes V2 until a canonical migration gate says so.

Known verified-byte and unbounded-input issue families are treated as migration blockers for production-trust claims, not silently ignored debt. Their issue state and applicability must be re-read at migration time rather than assumed from this planning snapshot.

The migration candidate is `specs/018-v3-1-authority-migration/`. It is a Spec Kit candidate. It does not make CF-17 through CF-28 executable, and it does not close issue #100 or issues #35, #36, #37, #38, and #40.

Issue #100's successor candidate is `specs/019-durable-offline-retained-authority/`. The historical artifact identity is known. The historical artifact bytes are unavailable. That candidate does not authorize CF-17.

The terminology verified-byte candidate is `specs/020-terminology-verified-cache-bytes/`. It covers issue #35 only. It does not close issues #36, #37, #38, #40, or #100, and it does not authorize CF-17.

The root-consumer verified-byte candidate is `specs/021-root-verified-cache-bytes/`. It covers issue #40 only. It does not close issues #36, #37, #38, or #100, and it does not authorize CF-17.

The impact and oracle verified-byte candidate is `specs/022-impact-oracle-verified-bytes/`. It covers issue #36 only. It does not close issues #37, #38, or #100, and it does not authorize CF-17.

The bounded lockfile candidate is `specs/023-bounded-lockfile-reads/`. It covers issue #37 only. It does not close issues #38 or #100, and it does not authorize CF-17.

The local-mirror archive candidate is `specs/024-local-mirror-archive-bound/`. It covers issue #38 only. It does not close issue #100.

The first CF-17 package is `specs/025-cf17-snapshot-identity/`. It freezes snapshot identity only. It merged as `1f4e45405505406d3d57d47d4773bbb23559f366`. It does not finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The second CF-17 package is `specs/026-cf17-separate-closures/`. It projects separate package-dependency and canonical-reference closures from an already frozen snapshot. It merged as `5846a9742ba21b6bbd45138dd7c31ff25792fe60`. It does not ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The third CF-17 package is `specs/027-cf17-closure-query/`. It binds one published snapshot to the two closure digests and returns a query identity. It merged as `296641b021269adb35506867b186a6b79cfa799a`. It does not ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The fourth CF-17 package is `specs/028-cf17-source-lifecycle/`. It records source lifecycle against a published snapshot and refuses stale or withdrawn sources at adoption. It merged as `2384be6483943dd8ba3311bc27c63aefeb98e266`. It does not ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The fifth CF-17 package is `specs/029-cf17-cache-identity/`. It keys cache reuse to the snapshot digest and an explicit engine schema. It merged as `11e3bf80bf8ac168cd1c85b9ffce5958c8fbe87d`. It does not measure scale, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The sixth CF-17 package is `specs/030-cf17-snapshot-comparison/`. It compares two published snapshot identities. It records membership and identity facts only. It merged as `4ca49836e7f1419f7555eb3504ee3694af494025`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The seventh CF-17 package is `specs/031-cf17-snapshot-history/`. It chains published snapshot comparisons in order. It merged as `87676ac4d9b584d20be2948d9ac2339cd8dfe2b3`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The eighth CF-17 package is `specs/032-cf17-history-machine-bytes/`. It emits canonical bytes for one qualified history. It merged as `be863ef1ae7e8fa9fde4a734f3430cc307887874`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The ninth CF-17 package is `specs/033-cf17-snapshot-machine-bytes/`. It emits canonical bytes for one published snapshot. It merged as `60221fe0de7c0ee003a0fb9a724e9a0cfd7af959`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The tenth CF-17 package is `specs/034-cf17-lifecycle-machine-bytes/`. It emits canonical bytes for one published source-lifecycle record. It merged as `f53b13616be9b4074a720aa30b997def248e687f`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The eleventh CF-17 package is `specs/035-cf17-closure-machine-bytes/`. It emits separate canonical bytes for one published package-dependency closure and one canonical-reference closure. It merged as `ef10a306d0b42a772ad8f69e9d4f3bae83db1a3e`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The twelfth CF-17 package is `specs/036-cf17-query-machine-bytes/`. It emits canonical bytes for one closure query. It merged as `17aed3d7c3d97fc2b8987e9f066e470bc5424d97`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The thirteenth CF-17 package is `specs/037-cf17-comparison-machine-bytes/`. It emits canonical bytes for one published snapshot comparison. It merged as `145a3c9b5f590e60fbbfc627dfdb09020421143c`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The fourteenth CF-17 package is `specs/038-cf17-cache-machine-bytes/`. It emits canonical bytes for one cache identity. It merged as `dc479b0806979710999b0fdb119dc264aa795ea4`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The fifteenth CF-17 package is `specs/039-cf17-workspace-identity/`. It records which snapshot packages belong to one workspace. It merged as `33ac488018c36b538482ddbf8d513d216f0b795a`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The sixteenth CF-17 package is `specs/040-cf17-workspace-machine-bytes/`. It emits canonical bytes for one workspace identity. It merged as `7d38d073286bd72277bb165f42ec30ac0b7f925c`. It does not classify compatibility, ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The seventeenth CF-17 package is `specs/041-cf17-offline-replay/`. It replays one frozen published observation and shows the resulting identities match. It merged as `eb83dce4a7e4430c8adc2a9ae37a9ecf8ba84648`. It does not ingest a registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The eighteenth CF-17 record is `specs/042-cf17-exit-gap-record/`. It records why official registry acquisition is must-ship and not executable. It is not execution authority. It does not implement acquisition, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The nineteenth CF-17 package is `specs/043-cf17-registry-acquisition/`. Its authorization is `fffb09c27179e52105ddc654248ae0fa00adeb02`. The implementation acquires one exact official package through an injected transport, hashes those response bytes, and replays them offline. It merged as `9adc556fdc02faf8e99661a23da78cc37ddec526`. It does not crawl a catalog, measure scale, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The twentieth CF-17 package is `specs/044-cf17-historical-package-graph/`. Its authorization is `abcc767559a556c6fb77ae2943f809d152c870b0`. The implementation orders exact supplied versions, recomputes each snapshot digest, and records `SUPPLIED_VERSIONS_ONLY`. It merged as `684eba49258e46d809d68259cd8029bd70659dd2`. It does not download a catalog, claim a complete registry listing, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The twenty-first CF-17 package is `specs/045-cf17-advertised-versions/`. Its authorization is `e15e4ce8b3ec31704e3b5c7a6955a283ce868983`. The implementation reads one official listing, verifies an archive for every advertised exact version, and records `ADVERTISED_BY_OFFICIAL_SOURCE`. It merged as `e48f3c119bda127765cb3e648a76f29b3f202d99`. It does not enumerate the registry, finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The twenty-second CF-17 package is `specs/046-cf17-telemetry-partition/`. Its authorization is `305231f2a3ec794240772d2dc267c9f42d202ec3`. The implementation keeps extension, terminology, and availability evidence in separate documents and refuses to treat mutable CI telemetry as published authority. It merged as `93a182428533ae8c7b2ab5569fcb07d445f880cd`. It does not finish the observatory, close issue #100 or issue #15, or authorize CF-18.

The twenty-third CF-17 record is `specs/047-cf17-exit-frontier/`. It recomputes the CF-17 requirement table after telemetry. It is not execution authority. It does not authorize a registry catalog, a scale claim, CF-18, issue #100 recovery, or an issue #15 pin change.

The twenty-fourth CF-17 record is `specs/048-cf17-catalog-source-gap/`. It records that a bounded catalog Spec Kit cannot be shaped yet, because the authorized hosts do not name a catalog document and `package-feeds.json` has no pinned commit or rights record. It is not execution authority. It does not authorize a catalog client, AF-03, AF-04, CF-18, issue #100 recovery, or an issue #15 pin change.
