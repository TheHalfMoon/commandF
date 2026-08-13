# commandF Research Agenda

This file is intentionally separated from product architecture and donor adoption decisions.

Status: candidate research program. Nothing here is a clinical claim or an assertion that a hypothesis will succeed.

## Research principle

The master's project should not be “build a FHIR converter.” The scientifically interesting question is whether healthcare transformations can be made **semantics-preserving, measurable, reproducible, and verifiable across models with different purposes**.

---

# R1 — Master's core: Semantic Conservation across FHIR, openEHR, and OMOP

## Candidate title

**Semantic Conservation in Cross-Standard Clinical Data Transformation: A Typed Intermediate Representation and Verifiable Interoperability Compiler for FHIR, openEHR, and OMOP**

## Core question

Can a typed neutral clinical intermediate representation with explicit information-loss accounting improve fidelity and verifiability compared with direct pairwise transformations?

## Hypotheses

H1. CSIR-mediated transformations preserve a greater proportion of predefined clinical facts than direct pairwise baselines.

H2. Explicit loss accounting detects clinically relevant degradations that target-format validators alone do not detect.

H3. Reusable intermediate mappings reduce authoring effort when the number of source/target systems grows.

## Initial standards
- FHIR R4/R5
- openEHR RM/archetypes/templates
- OMOP CDM 5.4

## Baselines
- FHIRconnect/openFHIR
- Eos/OMOCL
- HL7 FHIR→OMOP IG implementations where executable
- direct expert-authored transformations
- existing FHIR transformation tooling

## Measurements
- target syntactic validity
- profile/template/CDM conformance
- clinical fact coverage
- terminology fidelity
- cardinality preservation
- temporal preservation
- relationship/context preservation
- unit/precision preservation
- round-trip recoverability
- irreversible loss count/severity
- mapping reuse
- authoring/review effort
- transformation throughput

## Test data
Prefer synthetic/permitted datasets:
- Synthea
- curated openEHR archetype fixtures
- OMOP synthetic/test datasets
- MIMIC-derived FHIR resources only where access/license terms permit the experiment

---

# R2 — A formal Semantic Conservation Score

## Question

Can semantic preservation be represented by a reproducible multidimensional metric rather than a binary “valid/invalid” outcome?

## Candidate dimensions
- factual coverage
- terminology equivalence
- temporal equivalence
- numerical/unit equivalence
- relationship graph preservation
- provenance preservation
- reversibility
- unsupported-target loss

## Important methodological constraint

Do not invent arbitrary weights and call the result validated. First publish the vector of dimensions; derive a scalar score only after expert elicitation/sensitivity analysis demonstrates that aggregation is defensible.

Potential output: metric-methodology paper and reference implementation in commandF Bench.

---

# R3 — Cross-standard round-trip benchmark

## Question

How much information survives repeated transformations?

Experiment families:

```text
FHIR -> openEHR -> FHIR
FHIR -> OMOP -> FHIR
openEHR -> OMOP -> openEHR
FHIR R4 -> R5 -> R4
A -> CSIR -> B -> CSIR -> A
```

Classify failures into structural, semantic, terminology, temporal, precision, reference, and provenance loss.

Potential output: open benchmark/data paper.

---

# R4 — commandF Bench: a universal healthcare transformation benchmark

## Gap

Current conformance and AI benchmarks cover valuable subsets, but there is no single benchmark focused on semantic preservation across heterogeneous health-data models and mapping engines.

## Proposed benchmark

Producers:
- Synthea
- FHIR fixtures
- openEHR fixtures
- OMOP fixtures
- later HL7v2/CDA/DICOM fixtures

Transformation engines:
- commandF
- FHIRconnect/openFHIR
- Eos/OMOCL
- Microsoft FHIR Converter where applicable
- other executable baselines

Oracles:
- official HL7 validator
- HAPI
- Firely
- openEHR validator/oracle
- OHDSI DataQualityDashboard
- terminology servers

Potential output: benchmark paper + reusable public harness.

---

# R5 — IG/Profile Conflict Detection and Harmonization

## Question

Can incompatible FHIR Implementation Guide constraints be detected and explained automatically using normalized constraint graphs?

## Candidate method

Convert StructureDefinition snapshot/differential constraints into a constraint graph and compare:
- cardinalities
- allowed types/profiles
- slicing
- fixed/pattern values
- terminology bindings
- invariants
- extensions

Produce a minimal conflict witness and candidate harmonized profile where a safe intersection exists.

Potential outputs:
- algorithm paper
- commandF `profile diff/harmonize` feature

---

# R6 — Empirical FHIR Server Compatibility

## Question

How often do server behaviors diverge despite compatible or similar CapabilityStatements?

## Method

Run the same generated conformance probes against multiple open implementations and versions. Separate claimed capability from empirically observed capability.

Metrics:
- search semantics
- include/revinclude
- conditional interactions
- transaction behavior
- FHIRPath differences
- validation differences
- terminology behavior

Potential output: longitudinal compatibility dataset / reproducibility paper.

---

# R7 — Differential FHIRPath Semantics

Use multiple FHIRPath engines (official/HAPI/Firely/JS implementations) against generated and real conformance expressions.

Research questions:
- where do implementations disagree?
- which disagreements stem from spec ambiguity versus defects?
- can differential testing automatically minimize counterexamples?

Potential output: conformance/testing paper.

---

# R8 — Constrained AI Mapping vs unconstrained LLM mapping

## Question

Does an LLM constrained by schemas, terminology services, a typed Mapping IR, compiler checks, and semantic verification outperform direct prompt-based FHIR mapping in correctness and reviewer effort?

Arms:
1. direct LLM transformation
2. LLM generates target JSON + validator feedback
3. LLM generates Mapping IR + compiler/terminology feedback
4. retrieval from prior verified mappings + Mapping IR + human review

Important outcomes:
- hallucinated elements/codes
- terminology correctness
- valid-but-semantically-wrong outputs
- human correction time
- reproducibility

Potential output: AI/data-harmonization paper.

---

# R9 — Terminology Gap Registry

## Question

Can unresolved local/vendor concepts be characterized systematically rather than silently coerced into approximate codes?

Domains worth testing:
- wearables
- nursing
- genomics
- patient-reported outcomes
- local laboratory catalogs

Classify:
- exact
- equivalent
- broader
- narrower
- compositional/post-coordinated
- no acceptable map

Potential output: terminology coverage dataset and gap-analysis paper.

---

# R10 — Provenance-complete transformations

## Question

Can every target clinical assertion carry reproducible evidence linking it to source assertions, mapping rules, terminology decisions, conversions, and generated/default values without making the system impractically expensive?

Evaluate:
- storage overhead
- runtime overhead
- ability to reconstruct transformation reasoning
- usefulness for audit/debug/research reproducibility

Potential output: provenance architecture paper.

---

# R11 — Transformation Certificates and trust

## Question

Can a machine-verifiable transformation certificate improve interoperability auditability compared with conventional ETL logs?

Certificate candidates:
- content hashes
- schema/profile package closure
- mapping hashes
- terminology versions
- validator identities
- semantic-loss vector
- provenance root
- reproducibility data
- optional signatures/attestations

Potential output: methods/standards proposal after commandF has empirical evidence.

---

# R12 — Clinical Query IR

Longer-term, not recommended as the master's core.

Question: can clinically equivalent read-only queries be compiled across FHIR Search/FHIRPath, openEHR AQL, SQL-on-FHIR, and OMOP SQL with explicit proof of unsupported semantic differences?

Potential output: systems/query-language paper.

---

# R13 — AI-oriented clinical serialization

Build on FHIRBench and FHIR-AgentBench.

Question: can commandF choose or generate an information-preserving representation optimized for a model/task while retaining a verifiable link to source FHIR/openEHR/OMOP facts?

Do not make this the first paper; it depends on mature CSIR/provenance.

---

# R14 — Concurrency and transaction safety across healthcare systems

Investigate race conditions when EHR, pharmacy, lab, CDS, patient apps, and AI agents concurrently modify interoperable data.

Possible commandF contribution: transaction-safety analyzer that detects unsafe read-modify-write and policy TOCTOU patterns across adapter workflows.

Longer-term research only.

---

# R15 — Imaging semantic bridge

Question: how much clinical/imaging metadata is preserved when DICOM SR/SEG/study metadata are represented through FHIR, openEHR, and analytics models?

Potential future paper after the core structured-data pipeline is mature.

---

## Priority

### Master's priority
1. R1 Semantic Conservation
2. R2 measurement method
3. R3 round-trip experiment
4. R4 benchmark as the reproducibility artifact

### Strong follow-on papers
5. R5 IG conflict/harmonization
6. R8 constrained AI mapping
7. R9 terminology gaps
8. R6/R7 differential compatibility

### Later systems research
9. R10/R11 provenance and certificates
10. R12 Clinical Query IR
11. R13 AI serialization
12. R14 concurrency
13. R15 imaging

---

## Key literature/datasets to keep in the permanent bibliography

- FHIRconnect: Towards a Seamless Integration of openEHR and FHIR (2025).
- Bridging openEHR and OMOP: Expanded Mappings and Systematic Analysis of Semantic and Structural Limitations in the OMOP CDM (2026).
- Challenges of health data standard adoption and usage: a systematic review (2026).
- Interoperability-driven digital-twin work comparing FHIR, openEHR, and OMOP (2026).
- Modular FHIR transformation pipeline research (2025).
- LLM-assisted clinical data harmonization for OMOP (2026).
- Large Language Models for Automating Clinical Data Standardization: HL7 FHIR (2025).
- Infherno: End-to-end Agent-based FHIR Resource Synthesis from Free-form Clinical Notes (2025/2026 publication cycle).
- FHIR-AgentBench (2025/2026).
- FHIRBench (2026).
- FHIRPath-QA (2026).
- recent FHIR/OMOP bidirectional transformation studies.

A formal bibliography with DOI/URL, access date, evidence classification, and which hypothesis each source supports should be added before the research protocol is frozen.