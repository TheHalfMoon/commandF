# commandF Research Charter

Status: candidate research program; hypotheses are not product claims and no unexecuted result is treated as evidence.

## Candidate master's thesis

**Semantic Conservation in Cross-Standard Clinical Data Transformation: A Typed Intermediate Representation and Verifiable Interoperability Compiler for FHIR, openEHR, and OMOP**

## Core research question

Can a typed neutral clinical intermediate representation with explicit information-loss accounting improve transformation fidelity, verifiability, and mapping reuse compared with direct pairwise transformations between FHIR, openEHR, and OMOP?

## Initial hypotheses

H1. Intermediate-representation-mediated transformations preserve a greater proportion of predefined clinical facts than selected direct pairwise baselines.

H2. Explicit information-loss accounting detects clinically relevant degradation that target-format validators alone do not identify.

H3. Reusable intermediate mappings reduce mapping authoring/review effort as the number of supported source and target models grows.

These hypotheses may be rejected by experiment. A candidate CSIR is therefore a research/long-term architecture hypothesis, not a prerequisite for the V2 first product stack.

## Initial standards and models

- FHIR R4/R5, with version-aware packages and profiles
- openEHR RM, archetypes, and templates
- OMOP CDM 5.4 and terminology conventions

Later research may add HL7v2, CDA/C-CDA, DICOM/DICOMweb, and vendor schemas only after the initial experiment is reproducible.

## Baseline families

Candidate baselines/oracles retained in the discovery plan include:

- FHIRconnect/openFHIR for openEHR↔FHIR
- Eos/OMOCL for openEHR↔OMOP
- executable FHIR↔OMOP implementations where rights and reproducibility permit
- FHIR Mapping Language/StructureMap implementations
- Microsoft FHIR Converter where applicable
- expert-authored direct transformations
- official and independent validators/terminology services

## Primary measurements

- target structural/conformance validity
- predefined clinical fact coverage
- terminology fidelity
- cardinality preservation
- temporal preservation
- numerical/unit/precision preservation
- relationship/context preservation
- provenance preservation
- reversibility/round-trip recovery
- irreversible loss count and class
- mapping reuse
- authoring/review effort
- reproducibility
- runtime/throughput as a secondary systems metric

A multidimensional preservation vector is preferred over an unexplained scalar score. Any scalar Semantic Conservation Score requires a defensible weighting/calibration method and sensitivity analysis.

## Core experiment sequence

1. Freeze permitted/synthetic source fixtures and clinical-fact assertions.
2. Pin source/target standard packages, mappings, terminology, and all oracle/tool versions.
3. Implement or invoke selected direct pairwise baselines.
4. Implement the candidate typed intermediate representation only to the scope needed by the experiment.
5. Execute forward transformations.
6. Run structural/profile/template/CDM and terminology validation.
7. Compute semantic-preservation/loss evidence against the predefined assertions.
8. Execute round trips such as FHIR→openEHR→FHIR, FHIR→OMOP→FHIR, and openEHR→OMOP→openEHR.
9. Compare fidelity, reviewer effort, reuse, reproducibility, and failure classes.
10. Publish negative/null findings as well as positive ones.

## Reproducibility artifact

`commandF Bench` is the preferred research artifact: a version-pinned harness containing or referencing permitted fixtures, transformation configurations, validators/oracles, terminology versions, measurements, and machine-readable evidence.

## Related retained research tracks

The broader plan preserves:

- Semantic Conservation measurement methodology
- universal/cross-standard round-trip benchmark
- IG/Profile conflict detection and harmonization
- empirical FHIR server compatibility
- differential FHIRPath semantics
- constrained AI mapping vs unconstrained LLM mapping
- terminology gap registry
- provenance-complete transformations
- Transformation Certificates
- cross-model Clinical Query IR
- AI-oriented clinical serialization
- concurrency/transaction safety
- imaging semantic bridge
- data-quality and AI-readiness assessment

## Data governance

- synthetic/public/permitted datasets by default;
- no PHI committed to repository fixtures;
- data license/DUA tracked independently of software license;
- derived-artifact/redistribution restrictions recorded;
- MIMIC-derived material used only when exact access and downstream-use terms permit the intended experiment.

## Evidence governance

- no invented results;
- exact bibliographic metadata and publication status pinned before protocol freeze;
- experiment inputs, tool versions, package closure, mappings, terminology, configuration, and seeds pinned where applicable;
- validators are evidence sources, not proof of semantic equivalence;
- AI is never the sole semantic or terminology authority;
- architecture is allowed to change when evidence contradicts the hypothesis.
