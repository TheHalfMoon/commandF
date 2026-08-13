# commandF Discovery Coverage Annex — 2026-08-13

Status: **plan coverage authority / candidate inventory**

This annex exists to ensure that prior commandF discovery is not lost as execution becomes narrower and more disciplined.

It is part of the commandF plan, but it is **not** an adoption manifest and it does not authorize code copying by itself. A named project, product, paper, standard, or tool is a candidate/reference until its exact ref, relevant paths, license/permission, notices, and adoption mode are recorded under commandF provenance policy.

The Master Architecture remains the build-order authority. This annex answers a different question: **have we preserved all useful prior art, product ideas, tools, standards, open-source candidates, research directions, and gap hypotheses that we discussed?**

---

## 1. Product north star retained

commandF is not merely a FHIR converter.

The long-term product direction remains a healthcare interoperability quality, intelligence, compilation, verification, review, compatibility, provenance, and deployment platform.

Core future capabilities retained in the plan:

- interoperability Context Graph
- semantic and structural diff
- blast-radius analysis
- breaking-change detection
- quality profiles and quality gates
- interoperability review / deep review
- evidence-backed risk analysis
- deterministic rules and scoped institutional memory
- safe recipes / Verified AutoFix
- generated and adversarial tests
- differential Compatibility Lab
- Transformation Stacks
- Certification Queue
- Interoperability Inbox
- Living Interoperability Wiki
- standards/vendor drift monitoring
- Consumer Compatibility Matrix
- `can-i-certify`
- terminology gap registry
- mapping analysis and later Mapping IR
- semantic loss vocabulary / Loss Ledger
- round-trip verification
- transformation evidence and certificates
- optional, replaceable AI/agent plane where AI proposes and deterministic systems prove

---

## 2. Standards, official specifications, registries, and primary ecosystem sources

Retain as first-class standards/specification inputs and authoritative-or-near-authoritative references where applicable:

- HL7 FHIR R4 / R4B / R5, with R6 readiness
- HL7 FHIR NPM package format and package registries
- HL7 Validator / `hapifhir/org.hl7.fhir.core`
- HL7 IG Publisher
- FHIR Shorthand / SUSHI
- HL7 FHIR Mapping Language / StructureMap
- HL7 CRMI
- SMART on FHIR
- Bulk Data / Bulk FHIR
- SQL-on-FHIR
- FHIRPath
- GraphQL-on-FHIR as an evolving/secondary interface, not a foundation dependency
- CQL
- openEHR Reference Model, archetypes, templates, AOM/ADL/BMM, AQL, REST specifications
- OMOP CDM 5.4 and OHDSI vocabulary conventions
- HL7 v2
- CDA / C-CDA
- DICOM / DICOMweb
- public FHIR ecosystem portals and validators discussed during discovery, including `fhir.org`, `validator.fhir.org`, `open-fhir.com`, and openEHR official resources

---

## 3. FHIR core, validators, SDKs, servers, and runtime oracles

Candidates/reference implementations retained:

- `hapifhir/org.hl7.fhir.core`
  - official validator/core behavior; Tier-1 oracle
- `hapifhir/hapi-fhir`
  - parser, validation, FHIRPath, terminology, REST/server semantics; independent oracle/runtime candidate
- `FirelyTeam/firely-net-sdk`
  - independent .NET parsing/profile/FHIRPath/snapshot oracle
- `FHIR/fhir-candle`
  - multi-version FHIR development/test server and compatibility target
- `samply/blaze`
  - FHIR server, CQL, terminology behavior; compatibility target
- `LinuxForHealth/FHIR`
  - modular FHIR server/validator; compatibility target
- `microsoft/fhir-server`
  - server-internals and behavior reference; exact license/ref pin required before any reuse
- `medplum/medplum`
  - FHIR-native developer/runtime reference and real integration target
- OpenMRS + FHIR2 module
  - realistic modular EHR + FHIR integration laboratory
- OpenEMR
  - real EHR integration/conformance target; use license-aware process boundary where appropriate
- Bahmni
  - real hospital workflow integration reference
- Beda FHIR EMR ecosystem
  - FHIR-native workflow/reference target
- Metriport
  - universal healthcare API/reference patterns; exact reuse posture to be pinned before adoption

Rule: commandF does not replace independent validator/oracle implementations merely to remove Java/.NET dependencies. Differential behavior is an asset.

---

## 4. Cross-version FHIR conversion

Retained donor:

- `FHIR/fhir-codegen`
  - cross-version converters and mapping loader
  - R2/R3/R4/R4B/R5 differential/conversion evidence
  - source areas previously identified include `Converter_20_50`, `Converter_30_50`, `Converter_40_50`, `Converter_43_50`, `MappingLoader`, and ConceptMap exporters

Future commandF role:

- version-aware dialect analysis
- explicit reversibility/loss classification
- differential tests across versions
- never assume cross-version syntactic conversion is semantically lossless

---

## 5. Mapping engines, DSLs, mapping corpora, and transformation donors

Retained candidates:

- `GoogleCloudPlatform/healthcare-data-harmonization` / Whistle 2
  - typed/intermediate mapping concepts, transpiler, runtime, plugins, reconciliation, linter, language server, mapping tests
- `microsoft/FHIR-Converter`
  - HL7v2/C-CDA/JSON/STU3→FHIR mapping corpus and Liquid transformation patterns
- `beda-software/FHIRPathMappingLanguage`
  - pragmatic FHIRPath-centered mapping DSL/UX reference
- FHIR Mapping Language / StructureMap
  - standard mapping input; importer/analyzer target
- `SevKohler/FHIRconnect-spec`
  - reusable bidirectional openEHR↔FHIR mapping specification
- `openFHIR/openfhir`
  - FHIRconnect execution/reference engine
- `SevKohler/Eos`
  - openEHR→OMOP transformation/reference system
- `SevKohler/OMOCL`
  - openEHR/OMOP mapping language/corpus
- HL7/open implementations of FHIR↔OMOP mapping where executable

Long-term rule retained: commandF should **import/analyze multiple established mapping forms** rather than force all users into a new proprietary mapping language.

---

## 6. openEHR foundation

Retain:

- `openEHR/archie`
- `openEHR/specifications-RM`
- `openEHR/specifications-AM`
- `openEHR/specifications-QUERY`
- `openEHR/specifications-ITS-REST`
- `openEHR/CKM-mirror`
- `openEHR/openEHR-antlr4`
- CKM/archetype/template fixtures subject to artifact-level rights

Use for:

- RM/AOM/ADL/BMM semantics
- archetype/template validation and normalization
- AQL/query analysis
- real-world archetype corpus
- cross-model semantic-conservation experiments

---

## 7. OMOP / OHDSI foundation

Retain:

- `OHDSI/CommonDataModel`
- `OHDSI/DataQualityDashboard`
- `OHDSI/WhiteRabbit`
- `OHDSI/Usagi`
- `OHDSI/Achilles`
- OHDSI Atlas
- OHDSI vocabulary ecosystem

Use for:

- OMOP 5.4 structural ground truth
- source profiling
- data-quality/conformance checks
- terminology mapping workflow
- analytics/ETL reference behavior
- research baselines and semantic-loss evidence

---

## 8. Terminology infrastructure and gap analysis

Retain:

- Snowstorm
- Snowstorm Lite
- `termx-health/termx-server`
- `wardle/hades`
- OHDSI vocabularies
- standards terminology operations such as `$lookup`, `$validate-code`, `$expand`, `$translate`, `$subsumes`

Future commandF capabilities retained:

- terminology federation/adapters
- terminology version pinning
- terminology evidence in findings
- terminology gap registry
- exact/equivalent/broader/narrower/compositional/no-map classifications
- explicit refusal to silently coerce unsupported concepts

Important boundary: software license and terminology-content license are separate. SNOMED CT, ICD, LOINC, vendor code sets, and other controlled content require independent rights checks.

---

## 9. CDA, HL7v2, CSV, and legacy ingestion

Retain:

- `hl7ch/cda-fhir-maps`
- `HL7/CDA-Examples`
- Microsoft FHIR Converter HL7v2/C-CDA mappings
- `LinuxForHealth/CsvToFHIR`
- Apache Camel
- Apache NiFi
- Debezium
- OpenHIM
- NATS / Kafka patterns for event transport

Use for:

- CDA/C-CDA fixtures and mapping analysis
- HL7v2 ingestion
- CSV/flat-file ingestion
- batch, streaming, CDC, replay, and backpressure architecture
- future Gateway/connector runtime

---

## 10. Analytics, query, and data-plane tooling

Retain:

- `ohs-foundation/fhir-data-pipes`
- SQL-on-FHIR
- Pathling
- Apache Arrow
- Apache DataFusion
- Parquet
- Substrait
- CQFramework CQL
- CQF Ruler
- CQL-on-OMOP references
- openEHR AQL
- FHIR Search
- FHIRPath

Long-term research/product candidate retained:

- a cross-model Clinical Query IR only if empirical need justifies it
- explicit unsupported semantic differences rather than pretending all query systems are equivalent

---

## 11. Identity, de-identification, privacy, and policy

Retain identity/privacy candidates:

- `tuva-health/tuva_empi`
- MIRACUM FHIR Pseudonymizer / `$de-identify`
- Microsoft Presidio

Retain policy/security candidates:

- Cedar
- Open Policy Agent / Rego
- OpenFGA
- SPIFFE / SPIRE
- Keycloak
- SMART on FHIR patterns
- OHS FHIR Gateway
- Wasmtime for capability-scoped plugins

Rules retained:

- commandF does not invent a universal patient matching algorithm
- privacy/identity logic must be explicit evidence-bearing policy
- future instance-data profilers are on-premises by default and emit aggregate/statistical evidence where possible
- repository CI uses synthetic/public fixtures; no PHI

---

## 12. Edge and mobile interoperability

Retain:

- Open Health Stack Android FHIR SDK
- OpenSRP FHIR Core
- OHS FHIR Gateway

Use as edge/mobile interoperability and constrained-runtime references, not as commandF core dependencies by default.

---

## 13. DICOM and imaging

Retain:

- pydicom
- dcm4che
- Orthanc
- OHIF Viewer
- DCMTK

Future use:

- DICOM/DICOMweb compatibility
- SR/SEG/study metadata analysis
- imaging↔FHIR/openEHR/analytics semantic-conservation experiments

---

## 14. Provenance, metadata, registries, signing, and supply chain

Retain:

- OpenLineage
- DataHub
- Apicurio Registry
- ORAS
- Sigstore / Cosign
- in-toto
- SLSA
- SBOM tooling as release requirement

Future commandF use:

- content-addressed packages
- external lineage export
- field-level transformation provenance above generic lineage
- signed packages and evidence bundles
- reproducible Transformation Certificates
- immutable/pinned execution inputs

Signature alone never means semantic correctness; commandF evidence must state exactly what was checked.

---

## 15. Context Graph and local/private retrieval

Retain candidates:

- Oxigraph
  - embedded RDF/SPARQL graph prototype
- Tantivy
  - local lexical index
- Qdrant
  - optional semantic/hybrid retrieval side-index
- DataHub
  - lineage/catalog UX/reference patterns

Rule retained: vector search is never authoritative truth. Immutable artifacts + deterministic graph/provenance relationships remain primary evidence.

---

## 16. Policy/rule languages

Retain layered rule candidates:

- CEL for safe, bounded predicates
- CUE for structured constraints/unification
- OPA/Rego for richer organizational policy
- Cedar/OpenFGA where authorization semantics require them

Do not embed arbitrary Python/JavaScript execution in the trusted rule core.

AI may propose rules; human/policy authority activates them.

---

## 17. Testing, fuzzing, differential validation, and compatibility

Retain:

- Inferno Framework / HL7 test kits
- Schemathesis
- `cargo-fuzz`
- RESTler as an API-stateful-fuzzing research/reference candidate
- FHIR Candle
- Blaze
- HAPI
- Firely
- LinuxForHealth FHIR
- Medplum
- official HL7 Validator
- FHIRPath Lab / `fhirpath-lab`

Future commandF Compatibility Lab tests:

- advertised vs observed capabilities
- search semantics
- include/revinclude
- conditional operations
- transaction behavior
- terminology operations
- subscriptions
- bulk-data behavior
- OperationOutcome/error behavior
- safe concurrency probes
- FHIRPath implementation divergence

---

## 18. Software-quality and code-review donors

Retain open-source tooling/pattern donors:

- OpenRewrite
  - lossless/typed trees, deterministic recipes, safe/no-op-on-uncertainty philosophy
- `The-PR-Agent/pr-agent`
  - Git-provider plumbing, review commands, patch compression/context
- Buf
  - lint, breaking-change detection, modules, lockfiles, registry governance
- Semgrep
  - rule-engine and static-analysis workflow patterns
- oasdiff
  - machine-readable API change taxonomy/severity/diff
- Pact / Pact Broker
  - consumer-provider version matrix and `can-i-deploy` pattern
- Schemathesis
  - property/stateful API testing
- cargo-fuzz
  - Rust fuzzing
- ORAS
  - OCI artifact distribution
- Cosign
  - signatures/attestations
- OpenLineage
  - external lineage interchange
- SARIF / GitHub code scanning
  - finding interchange and repository annotations
- CodeQL
  - path-aware finding and SARIF workflow reference
- OpenTelemetry
  - traces/metrics/logs; no PHI in baggage

Product/reference systems retained for inspiration, not source adoption unless a lawful source is separately available:

- Greptile
  - codebase/context graph, blast radius, scoped rules, confidence/explanation
- Cubic
  - incremental/deep review, custom reviewers, local preflight, fix workflows, wiki
- Graphite
  - stacked changes, merge queue, inbox
- Augment Code / Cosmos
  - structured context, Triage→Author→Review→Verify, review fleet, test planning
- Qodo
  - multi-agent review, judge/deduplication, requirement gaps, rule mining, cross-repo conflicts
- SonarQube
  - quality profiles/gates and new-change-first adoption

Translation retained in commandF:

- Context Graph
- Review / Deep Review Fleet
- Semantic Diff
- risk-triggered review depth
- safe Verified AutoFix
- Transformation Stacks
- Certification Queue
- Interoperability Inbox
- Living Interoperability Wiki
- quality gates focused on new/changed interoperability assets
- consumer compatibility / `can-i-certify`

---

## 19. Package/registry and compatibility architecture donors

Retain:

- FHIR package loader behavior used by SUSHI
- Buf package/module/lockfile/registry concepts
- Confluent Schema Registry compatibility-mode concepts
- Pact Broker verification matrix concepts
- ORAS/OCI artifact distribution
- Apicurio Registry

Healthcare-specific compatibility dimensions retained:

- STRUCTURAL
- FHIR
- PROFILE
- TERMINOLOGY
- SEMANTIC
- ROUND_TRIP
- CONSUMER
- FULL

Transitive compatibility must be supported where policy requires comparison against protected historical versions.

---

## 20. Observability and runtime safety

Retain:

- OpenTelemetry
- Wasmtime
- SPIFFE/SPIRE
- NATS/Kafka
- Debezium
- Apache Camel/NiFi

Rules retained:

- traces/metrics/logs identify transformation stages, package hashes, mapping/rule IDs, and evidence IDs
- PHI is not placed into generic telemetry baggage
- plugins/adapters should be capability-scoped and resource-bounded

---

## 21. Research datasets, benchmarks, and test corpora

Retain:

- Synthea
- Inferno test kits
- FHIR-AgentBench
- FHIRBench
- FHIRPath Lab / FHIRPath differential corpora
- FHIRPath-QA
- openEHR archetype/template fixtures
- OMOP synthetic/test datasets
- MIMIC-derived artifacts only where access/license terms permit the exact experiment
- real public FHIR IG/version deltas
- published cross-version FHIR package corpora

Research and production datasets remain separately governed.

---

## 22. Research program retained

The research program is maintained separately in `research/RESEARCH_AGENDA.md` and remains hypothesis-driven, not product truth.

Retained research themes:

1. Semantic Conservation across FHIR, openEHR, and OMOP
2. multidimensional Semantic Conservation Score
3. cross-standard round-trip benchmark
4. commandF Bench universal transformation benchmark
5. IG/Profile conflict detection and harmonization
6. empirical FHIR server compatibility
7. differential FHIRPath semantics
8. constrained AI mapping vs unconstrained LLM mapping
9. terminology gap registry
10. provenance-complete transformations
11. Transformation Certificates and trust
12. cross-model Clinical Query IR
13. AI-oriented clinical serialization
14. concurrency and transaction safety
15. imaging semantic bridge
16. data-quality and AI-readiness assessment across interoperable clinical datasets

---

## 23. Research/literature families retained for bibliography

The permanent bibliography should include and verify exact bibliographic metadata for:

- FHIRconnect openEHR↔FHIR research
- openEHR↔OMOP / Eos / OMOCL research
- recent systematic reviews of health-data-standard adoption and interoperability barriers
- FHIR/openEHR/OMOP digital-twin interoperability comparisons
- modular FHIR transformation pipeline research
- LLM-assisted OMOP/clinical data harmonization studies
- LLM-assisted HL7 FHIR standardization studies
- Infherno / agent-based FHIR synthesis research
- FHIR-AgentBench
- FHIRBench
- FHIRPath-QA and differential FHIRPath work
- recent bidirectional FHIR↔OMOP transformation studies
- terminology coverage/gap studies
- provenance/reproducibility and healthcare transformation auditability research

Exact DOI/URL, publication status, evidence class, data license, and hypothesis linkage must be pinned before a research protocol is frozen.

---

## 24. commandF-owned moat retained

The following are not delegated wholesale to donors and remain commandF-owned differentiators if evidence supports them:

- versioned healthcare interoperability breaking-change taxonomy and rule corpus
- healthcare Context Graph / blast-radius model
- cross-artifact Semantic Diff
- consumer-aware compatibility evidence
- terminology gap evidence model
- explicit semantic-loss vocabulary and Loss Ledger
- Semantic Conservation measurement method
- round-trip/reversibility classification
- Transformation Evidence Graph
- Transformation / Conformance Certificate
- healthcare-specific Compatibility Lab
- `can-i-certify`
- protected-consumer matrix
- safe interoperability Recipe Engine
- FSH-authored-source fidelity for findings
- future Mapping Analysis IR / Mapping IR
- future CSIR only if evidence from implemented dialects justifies it
- commandF Bench

---

## 25. Explicit anti-patterns retained

commandF must not inherit these limitations from donors or incumbent tools:

- FHIR as the only universal internal clinical truth
- N×N pairwise converters as the primary architecture
- silent lossy conversion
- untyped JSON-only mapping as the trusted core
- LLM output as semantic authority
- black-box ETL with no reproducible evidence
- CapabilityStatement claims treated as empirical truth
- cloud-only architecture
- vector/RAG index treated as canonical knowledge
- copied donor source with missing provenance
- terminology software license treated as terminology-content permission
- custom replacements for mature validators solely to eliminate JVM/.NET boundaries

---

## 26. Execution relationship to V2

This annex intentionally does **not** reorder the first execution stack.

The build order remains narrow and evidence-driven:

1. CF-01 deterministic FHIR package resolution/cache/lock
2. CF-02 canonical package inspection/indexing
3. CF-03 structural diff
4. CF-04 breaking/risk rules
5. CF-05 findings/SARIF/quality gate surface
6. CF-06 differential oracle evidence
7. CF-07 terminology-aware diff
8. CF-08 GitHub review delivery
9. CF-09 FSH source fidelity
10. CF-10 real public IG delta corpus
11. CF-11 ecosystem Context Graph
12. CF-12 blast radius / impact
13. CF-13 baselines/suppressions/quality gates
14. CF-14 isolated on-prem aggregate source profiler
15. CF-15 verified recipes
16. CF-16 mapping analysis IR

Later slices may activate the retained candidates in this annex only when a concrete user-visible capability needs them.

---

## 27. Coverage gate

Before a new major architectural plan supersedes V2, the plan review must answer:

- Which entries from this annex are adopted now?
- Which remain candidates?
- Which are explicitly rejected, and why?
- Which research tracks remain scientifically meaningful?
- Which donor refs/licenses/permissions have been pinned?
- Which new discoveries were added?

No future plan may silently drop a candidate/research track merely because it is not on the immediate build path.
