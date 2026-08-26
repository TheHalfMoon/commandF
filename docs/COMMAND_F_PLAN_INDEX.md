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
- AF-02/03/04 remain retained program units and require their own planning packages before implementation.

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
