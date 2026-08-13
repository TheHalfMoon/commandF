# commandF Constitution

Derived from GitHub Spec Kit's constitution workflow and specialized for healthcare interoperability change intelligence.

## Core Principles

### I. Ship a vertical capability, not a scaffold
Every feature MUST produce a user-visible command, report, annotation, or independently executable verification result. A new crate without an immediate shipped consumer is prohibited.

### II. Determinism before intelligence
Identical pinned inputs MUST produce byte-identical machine-readable outputs wherever the domain permits. AI may propose; deterministic code, authoritative oracles, tests, or explicit human approval establish accepted evidence.

### III. Fail closed on unknown interoperability state
commandF MUST NOT silently classify an unknown delta as compatible, silently resolve conflicting package versions, invent clinical facts, hide precision loss, or infer authority that was not supplied.

### IV. Standards and mature infrastructure are dependencies or oracles
commandF MUST NOT reimplement authoritative FHIR validation merely to remove a JVM dependency, create a new FHIR authoring language, terminology server, EMPI, package registry, universal query language, or executable mapping language where mature standards/projects already exist.

### V. Review the authored source
A finding MUST point back to the artifact a human maintains whenever possible. For FSH-authored IGs, generated StructureDefinition JSON is not sufficient review UX; FSH/SUSHI source mapping is a required product capability.

### VI. Evidence is explicit and reproducible
Package identity, exact version, content digest, rule-pack version, oracle identity/version, and relevant source provenance MUST be retained. Mutable aliases alone are insufficient evidence.

### VII. Precision over review noise
False-positive breaking findings are an existential product risk. BREAKING classifications require a documented rationale, positive example, counterexample/negative coverage, and version applicability. Review summaries MUST be bounded and actionable.

### VIII. Product and research remain separate
Research hypotheses such as semantic-conservation metrics do not become product guarantees until supported by reproducible evidence and appropriate peer/expert review.

## Scope and Trust Boundary

The initial product analyzes FHIR conformance metadata and does not require patient data. Repository tests use synthetic or publicly redistributable conformance artifacts. A future instance-data profiler must be separately isolated, on-premises by default, and aggregate-only at its external boundary.

The long-term vision may include mapping analysis, explicit transformation-loss vocabulary, openEHR/OMOP correspondence, round-trip experiments, and evidence bundles. None may become a dependency of `commandf check` until earned by a prior working vertical slice.

## Spec-Driven Development Workflow

Every CF slice MUST have, before implementation is considered complete:

1. `spec.md` — user problem, outcomes, functional requirements, non-goals, edge cases.
2. `plan.md` — architecture, dependencies/oracles, security/trust boundaries, tests, migration impact.
3. `tasks.md` — independently verifiable tasks ordered by dependency.
4. consistency analysis — spec/plan/tasks contradictions resolved before merge.
5. implementation evidence — tests and CI against the exact PR head.
6. convergence pass — remaining gaps appended to tasks or explicitly deferred.

Graphite-style PR stacks are the delivery unit. CodeRabbit and Qodo are independent reviewers when available; neither substitutes for deterministic gates.

## Mandatory Gates

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- deterministic repeat-run checks where applicable
- no unclassified compatibility state in a shipped checker
- no PHI fixtures
- explicit licensing/provenance for adopted donor material
- authoritative-oracle differential testing for overlapping judgments
- no merge until the exact candidate state is green

## Governance

This constitution outranks feature plans and implementation convenience. Any exception MUST be explicit in the relevant `plan.md`, justified, bounded, reviewed, and include a removal or revisit condition.

Spec Kit process donor: `github/spec-kit`, pinned initially to release `v0.16.2` (MIT). commandF adopts the workflow and template concepts; healthcare-specific requirements remain commandF-owned.

**Version**: 1.0.0 | **Ratified**: 2026-08-13 | **Last Amended**: 2026-08-13
