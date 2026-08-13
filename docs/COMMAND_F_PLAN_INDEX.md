# commandF Plan Index

Status: **authoritative plan-set index**

commandF is intentionally planned as a small execution spine plus a larger preserved product/discovery/research envelope. No single document should mix immediate implementation authority with every candidate donor, product capability, and research hypothesis.

The commandF plan therefore consists of the following layers.

## A. Execution authority

`docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`

Defines product boundary, architecture planes, first execution stack, trust boundaries, and mandatory gates. When it conflicts with older bootstrap planning, V2 controls execution order.

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

Every `CF-*` slice is a Spec Kit-style feature unit:

```text
spec.md -> plan.md -> tasks.md -> implementation -> deterministic validation -> convergence.md
```

CF-01 is the current feature unit under `specs/001-cf-01-package-resolution/`.

## Coverage rule

Before a future architecture supersedes V2, its review must reconcile this entire plan set. A candidate, product capability, gap, or research track may be:

- adopted
- retained for later
- explicitly rejected with rationale
- superseded by better evidence

It may **not** disappear silently.

## Build-order rule

Preserving a candidate in the plan does not allow it to bypass the V2 execution sequence. A donor/tool/capability is activated only when a concrete slice requires it and its provenance/adoption gate is satisfied.
