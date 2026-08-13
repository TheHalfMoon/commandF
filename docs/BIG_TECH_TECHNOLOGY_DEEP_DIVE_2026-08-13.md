# commandF Big Tech & Healthcare Technology Deep Dive

Date: 2026-08-13
Status: Architecture input

> Academic papers and research hypotheses remain separate under `research/`.

## Executive conclusion

The strongest lesson from Google/DeepMind, InterSystems, Microsoft, AWS, Oracle Health, IBM, NVIDIA, Apple, Databricks, OpenAI and Anthropic is that commandF should not become another FHIR server, integration engine, lakehouse, or AI chatbot.

The strategic opportunity is a cross-platform **quality, semantic change intelligence, compilation, verification, evidence, and certification control plane**.

```text
Change Intelligence
  Context Graph · Semantic Diff · Blast Radius · Review
        ↓
Healthcare Compiler
  Dialects · CSIR · Mapping IR · Terminology · Identity · Policy
        ↓
Verification
  Validators · Oracles · Consumer Contracts · Round Trip · Fuzz
        ↓
Evidence
  Loss Ledger · Reconciliation · Provenance · Evidence Chain · Certificate
        ↓
Registry / Certification
  Signed packages · Compatibility · Certification Queue
```

## Google Cloud

Google's health-data architecture keeps FHIR, HL7v2 and DICOM as distinct stores and publishes FHIR→OMOP transformation patterns. Healthcare Data Harmonization / Whistle is a high-value donor for mapping IR, transpilation, runtime, plugins, reconciliation, testing, linting and language-server patterns.

**Decision:** PORT/IMPORT selected Whistle components only after immutable source/commit pinning. Whistle must be one Mapping IR importer, not commandF's only mapping language.

**New commandF contract:** every transform can emit a `ReconciliationArtifact` linking observed source facts to produced target facts, generated facts, unmatched facts, terminology resolutions and mapping rules.

## Google DeepMind

DeepMind's Verifiable Data Audit concept is a direct inspiration for commandF's Evidence Plane: append-only usage records, purpose-of-use, cryptographic hashes/Merkle structures, continuous audit and verifiable integrity without requiring a public blockchain.

**Decision:** evolve isolated Transformation Certificates toward an optional cryptographically linked **Evidence Chain**:

```text
EvidenceEvent
├── previous hash
├── event hash
├── workload/actor
├── purpose of use
├── input digests
├── mapping/ruleset digests
├── terminology digests
├── output digests
├── findings/certificate digest
└── signature/attestation
```

Google health research also reinforces multi-dimensional expert evaluation. For AI mapping/review, commandF should use granular evidence-backed rubrics and deterministic validators wherever machine proof is possible.

## InterSystems

InterSystems is the strongest enterprise architecture analogue discovered.

### SDA validates an intermediate representation

SDA is explicitly used as an intermediary format when converting among FHIR, HL7v2 and CDA/C-CDA. This validates commandF's CSIR architecture. InterSystems also warns that SDA is an intermediary and should not become a persistent clinical data model.

**Decision:** CSIR remains an intermediary representation by default. Persistence requires a separate future ADR. CSIR differentiates through typed contracts, source provenance, LossEvents, reversibility metadata, deterministic serialization and verification.

### DTL vs process orchestration

InterSystems separates data transformation from business-process orchestration.

**Decision:** commandF must keep `Mapping/Recipe IR` separate from `Workflow/Stack IR`; do not create one giant DSL.

### FHIR SQL Builder

FHIR SQL Builder projects FHIR into custom relational views without necessarily copying the data and can analyze repository structure/value ranges before creating transformation specifications.

**Decision:** add `commandf profile-source` and support both virtual query projections and materialized Arrow/Parquet/OMOP lowerings.

InterSystems built-in SDA↔FHIR transformations currently document support for R4 and earlier, not R5; commandF's multi-version compiler should treat this as a competitive opening.

## Microsoft

Azure Health Data Services separates FHIR, DICOM, MedTech, events and de-identification concerns. Microsoft's open-source FHIR Converter remains a donor for HL7v2/C-CDA/JSON/FHIR-version transformations; `microsoft/fhir-server` is an independent implementation oracle.

**Decision:** modular planes, not one service. Use FHIR Converter as IMPORT/PORT after pinning; use FHIR Server primarily as ORACLE/TEST FIXTURE.

## AWS

AWS 2026 architecture material treats FHIR and openEHR as complementary: FHIR for interoperable exchange and openEHR for richer clinical modeling. This independently supports commandF's decision that FHIR is not the sole canonical semantic model.

`awslabs/mcp` contains an Apache-2.0 HealthLake MCP server exposing FHIR workflows.

**Decision:** build provider-neutral commandF tool/MCP contracts with explicit read/write scope, tenant, purpose-of-use, data classification and evidence output. AWS is an adapter/reference, not a core dependency.

## Oracle Health

Oracle Health's interoperability stack highlights enterprise patient matching, record location, HIE/QHIN, FHIR/Bulk/SMART APIs and versioned vendor endpoints.

**Decisions:** identity resolution is a first-class subsystem; add an `EndpointCapabilityRegistry` combining declared and empirically observed capabilities; vendor version retirement becomes a first-class impact-analysis event.

## IBM

IBM App Connect for Healthcare demonstrates composable HL7/FHIR/DICOM message flows, MLLP handling, duplicate detection, FHIR validation and transformation patterns.

**Decision:** transports, parsing, transformation and validation are separate nodes/passes. Operational correctness such as ACK behavior and duplicate detection must be testable independently from semantic mapping.

## NVIDIA / MONAI

MONAI Deploy separates application packages, an informatics gateway, workflow manager and pluggable execution engines.

**Decision:** commandF Registry package lifecycle:

```text
EXPERIMENTAL → CANDIDATE → VERIFIED → CERTIFIED → PRODUCTION_CERTIFIED
                                      ↓
                               DEPRECATED / REVOKED
```

Package signature and package certification are different facts.

## Apple HealthKit

`HKFHIRResource` preserves source URL, resource id/type, exact FHIR version and raw JSON. Apple also documents that resource id uniqueness depends on source/type context.

**Decision:** every CSIR assertion/source artifact must retain source system, exact dialect/version, original payload digest and precise source pointer. Raw PHI may remain outside commandF while the Evidence Graph stores a digest/pointer.

## Databricks

`dbignite` provides FHIR R4/R5/NDJSON analytics and OMOP-related patterns; `smolder` is a high-volume HL7v2 Spark datasource; the Databricks X12/Ember project handles 837i/837p/834/835 and Arrow-based large-file processing.

**Decision:** keep the core Rust/streaming/Arrow-friendly. Spark/Databricks is an optional scale adapter. Databricks code requires exact license/path audit before reuse; X12 project remains STUDY-only until provenance is explicitly cleared.

## OpenAI and Anthropic

HealthBench/HealthBench Professional demonstrate expert-authored, realistic health evaluations, granular rubrics, adversarial examples and worst-case reliability analysis. Claude Science demonstrates a scientific workbench that integrates tools and produces auditable artifacts.

**Decision:** create `commandF ReviewBench` for AI-generated mappings/reviews/autofixes; commandF Studio should be an interoperability workbench, not a chat page.

## Architecture changes from this review

1. Add `ReconciliationArtifact`.
2. Add optional cryptographically linked `EvidenceChain`.
3. Keep CSIR intermediary by default.
4. Split Mapping/Recipe IR from Workflow/Stack IR.
5. Add Source Profiler.
6. Add virtual projection mode alongside materialized analytics lowerings.
7. Add Registry certification lifecycle.
8. Add Endpoint Capability Registry.
9. Make Identity a first-class subsystem.
10. Add provider-neutral typed agent/MCP capabilities.
11. Preserve source-bound identity/version/raw digest.
12. Add `ReviewBench` before certifying AI-assisted automation.

## Big-tech synthesis

```text
Google       → Mapping IR + reconciliation + lineage
DeepMind     → Evidence Chain + auditability + evaluation discipline
InterSystems → CSIR analogue + transform/workflow separation + SQL projection
Microsoft    → modular health services + converter/server donors
AWS          → FHIR/openEHR hybrid + agent tool contracts
Oracle       → identity + endpoint/capability registry
IBM          → message-flow discipline + operational validation
NVIDIA       → package/workflow/gateway + certification lifecycle
Apple        → source identity/version/raw payload provenance
Databricks   → scalable HL7/FHIR/X12 analytics adapters
OpenAI       → expert rubric evaluation + worst-case reliability
Anthropic    → auditable workbench UX
```

No reviewed platform currently provides the complete combination commandF targets: cross-standard semantic compilation, semantic change review, blast radius, explicit information-loss accounting, differential implementation testing, consumer compatibility, verified autofix, signed transformation evidence and certification queue.

## Implementation impact

Extend the Master Plan sequence with:

1. Finding Contract + SARIF
2. Source Identity contract
3. Reconciliation Artifact
4. `commandf.yaml` + lockfile
5. content-addressed package model
6. FHIR indexer + Source Profiler
7. validator/oracle abstraction
8. Context Graph
9. lint/rules
10. Semantic Diff + blast radius
11. Breaking Checker
12. GitHub Review adapter
13. Quality Gate
14. protected fixtures + differential tests
15. Mapping IR
16. Recipe Engine
17. Round-trip Verifier
18. Consumer Compatibility Matrix
19. Registry certification lifecycle
20. signed Evidence Chain prototype

The first product wedge remains review-before-production; Evidence Chain becomes essential before production-grade certification claims.