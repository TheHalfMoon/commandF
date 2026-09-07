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

Future agent/evidence/source-adoption control gaps are retained separately in `docs/COMMAND_F_AGENT_EVIDENCE_AND_SOURCE_ADOPTION_PLAN_2026-09-08.md` and `docs/COMMAND_F_AGENT_SECURITY_AND_CONTROL_GAPS_2026-09-08.md` so they do not silently become healthcare-interoperability claims.

## E. Donor/provenance authority

`docs/PROVENANCE_AND_DONOR_POLICY.md`

Defines adoption modes and the pin/license/permission/source-path requirements that must be satisfied before candidate prior art becomes adopted commandF code/data/mappings.

Current slice-specific and architecture-study donor records remain under `donors/`, including:

- `donors/cf-01-package-resolution.yaml`
- `donors/agent-harness-2026-08-13.yaml`
- `donors/tencent-agent-knowledge-sources-2026-09-08.yaml`

Future slice plans must add or update donor records rather than relying on conversation memory. A broad Founder authorization to pursue source reuse does not replace the file/artifact-level upstream rights evidence required by the donor policy.

## F. Research authority

`research/RESEARCH_CHARTER.md`

Preserves the candidate master's thesis, core research question, H1–H3, initial standards/models, baseline families, measurement framework, experiment sequence, reproducibility artifact, and evidence/data governance.

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

Historical execution snapshot at the creation of the Assurance Program:

```text
CF-13: CLOSED_CANONICAL
main: 8a45857bf31c4acae57fdfb1e3cdde3d0f7d0361
next product identity: CF-14
then-current cross-cutting planning unit: AF-01
```

This block is historical context only. It is not the current execution frontier.

### Current execution frontier — verified 2026-09-08

Live repository truth at this planning PR's latest reconciliation:

```text
canonical main: 83b270893a6ccba7ce911b8d31e186f5c9fb122b
AF-01: CLOSED_CANONICAL
AF-02 T021: CLOSED_CANONICAL on current main
active implementation frontier: AF-02 T022
active implementation PR: #82
PR #82 exact base: 83b270893a6ccba7ce911b8d31e186f5c9fb122b
PR #82 current candidate head at reconciliation: 90481e22e629ed81c5cc61aaea646e0f269f5125
```

The hashes above are a dated reconciliation record, not a substitute for live verification. Before acting, re-read current `main`, the active AF-02 task ledger, PR #82, and exact-head CI/review/provenance state. Live repository truth overrides this dated block if any identity has moved.

## H. Assurance-program authority

`docs/COMMAND_F_ASSURANCE_PROGRAM_2026-08-26.md`

Preserves and sequences the cross-cutting work required to make commandF's own development/release evidence as rigorous as its interoperability evidence.

Assurance units use `AF-*` identities and **do not renumber product CF slices**.

Program units retained:

1. **AF-01 Trusted Development Baseline** — source-control enforcement, immutable workflow references, least authority, dependency/license/source/advisory policy, CI/CD static analysis, exact-head assurance proof.
2. **AF-02 Adversarial Test Strength** — structure-aware/differential fuzzing, property tests, mutation adequacy, coverage diagnostics/floors, flaky-as-failure execution, minimized regression corpus.
3. **AF-03 Portability and Release Evidence** — Linux/Windows/macOS, MSRV, public API/SemVer guard, SBOM, SLSA-compatible provenance, artifact/signature verification.
4. **AF-04 Performance and Reliability Evidence** — measured benchmark/resource budgets, large-input stress, external-sentinel separation, retained trends reusable by future commandF Bench.

Historical initial AF-01 planning package:

`specs/015-af-01-trusted-development-baseline/`

AF-01 is now `CLOSED_CANONICAL` according to its canonical task ledger. Its original prerequisite text is therefore historical rather than current authorization. AF-02 is the active assurance execution family at this reconciliation point; AF-03/AF-04 remain retained program units and require their own canonical activation/planning authority before implementation.

Current ordering rule:

- complete the active AF-02 dependency-ordered frontier under its canonical package before merging unrelated planning that would invalidate an exact-base qualification;
- future product/assurance implementation remains subject to the live V2/Spec Kit/Assurance ordering and must be re-read rather than inferred from this index;
- AF-03/AF-04 naming in this index is retained coverage, not implementation authorization.

## I. Future agent, evidence, and source-adoption planning coverage

`docs/COMMAND_F_AGENT_EVIDENCE_AND_SOURCE_ADOPTION_PLAN_2026-09-08.md`

Refines the optional future agent/Copilot plane without changing current execution authority. It separates six future trust domains:

1. deterministic evidence substrate;
2. capability/sandbox execution;
3. resumable workflow orchestration;
4. bounded durable memory;
5. protected evaluation/optimization authority;
6. Copilot/model experience.

The plan also adds a donor-portfolio decision strategy across the repository's existing source inventory and records future-agent/source-adoption gaps discovered through the Tencent source study.

`docs/COMMAND_F_AGENT_SECURITY_AND_CONTROL_GAPS_2026-09-08.md`

Adds the second-pass cross-layer threat-model constraints that are easy to miss when each capability is reviewed separately, including:

- untrusted-context/prompt-injection provenance and trust labels;
- exact-action approval binding and TOCTOU resistance;
- replay/idempotency/effect reconciliation;
- delegated capability attenuation;
- tenant/workspace/principal isolation;
- tamper-evident audit evidence;
- skill/plugin/tool supply-chain controls;
- retention/deletion/export/privacy lifecycle;
- concurrency leases/fencing;
- model/provider routing and fallback identity;
- cumulative autonomy circuit breakers;
- human-approval fatigue and semantic previews.

Important ordering rule:

- these future planning documents do **not** alter the active AF-02 contracts or dependency order;
- no new canonical CF/AF identity is created by these documents;
- implementation still requires a separately authorized Spec Kit unit and provenance/adoption gate;
- the deterministic commandF core remains independent of all agent/memory/evaluation runtimes.

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
