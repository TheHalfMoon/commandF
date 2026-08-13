# commandF Donor Deep Audit — 2026-08-13

Status: foundation candidate inventory. No donor code is considered adopted until its exact commit, path, license, notices, and modification plan are pinned in `donors/manifest.yaml`.

## Decision

commandF will **reuse commodity interoperability code and invent the semantic-verification layer**.

Adoption modes:

- `DEPEND`: consume upstream as a package/service with a narrow adapter.
- `EMBED`: vendor upstream component with its history/license intact.
- `PORT`: translate a well-understood implementation into the commandF Rust architecture while preserving provenance.
- `COPY`: copy selected source files where this gives a material maintenance benefit and the license/permission record is complete.
- `IMPORT`: ingest mappings, schemas, tests, fixtures, or definitions into a commandF registry format.
- `ORACLE`: run an independent implementation as a differential/conformance oracle.
- `STUDY`: architectural donor only; no source copied.

The default is `DEPEND` or `IMPORT`, not wholesale copying.

---

## Tier S — foundation donors

### 1. HL7 FHIR core / official validator

Repository: `hapifhir/org.hl7.fhir.core`

Use:
- official Java FHIR core model and utilities
- validator CLI/core
- profile/StructureDefinition validation behavior
- FHIRPath implementation/tests
- terminology hooks
- core package loading
- official behavior used by IG Publisher

Adoption: `ORACLE + DEPEND`, selected test/definition `IMPORT`.

Why: commandF must not invent an incompatible definition of FHIR validity.

### 2. FHIR codegen cross-version pipeline

Repository: `FHIR/fhir-codegen`
License: MIT

High-value source areas observed in the current repository:
- `src/Fhir.CodeGen.CrossVersionLoader/Converter_20_50.cs`
- `src/Fhir.CodeGen.CrossVersionLoader/Converter_30_50.cs`
- `src/Fhir.CodeGen.CrossVersionLoader/Converter_40_50.cs`
- `src/Fhir.CodeGen.CrossVersionLoader/Converter_43_50.cs`
- `src/Fhir.CodeGen.Comparison/CrossVersionSource/MappingLoader.cs`
- `src/Fhir.CodeGen.CrossVersionExporter/ConceptMapToR4.cs`
- generated per-datatype conversion implementations under `Convert_*`

Use:
- seed commandF FHIR version-dialect diff model
- import cross-version maps into Mapping IR
- build R2/R3/R4/R4B/R5 conversion differential tests
- retain R6 as a separate evolving dialect until published

Adoption: `PORT + IMPORT + ORACLE`.

### 3. HAPI FHIR

Repository: `hapifhir/hapi-fhir`
License: Apache-2.0

Use:
- parser/serializer behavior
- package/profile tooling
- FHIRPath
- FHIR REST semantics
- terminology client/server behavior
- validation integration
- batch and server test fixtures

Adoption: `DEPEND + ORACLE`; copy only sharply bounded reusable utilities when dependency isolation is impossible.

### 4. Firely .NET SDK

Repository: `FirelyTeam/firely-net-sdk`

Use as an independent implementation oracle for:
- JSON/XML parsing
- canonical/profile handling
- FHIRPath
- snapshot/differential behavior
- cross-version resource behavior

Adoption: `ORACLE`. License must be pinned at the exact selected upstream ref before source copying.

### 5. Google Healthcare Data Harmonization / Whistle 2

Repository: `GoogleCloudPlatform/healthcare-data-harmonization`
License: Apache-2.0

This is one of the highest-value architectural/code donors.

Directly relevant modules:
- `proto/` — intermediate representation
- `transpiler/` — source syntax to IR
- `runtime/` — execution engine
- `plugins/` — plugin contract
- `plugins/harmonization/` — FHIR code translation
- `plugins/reconciliation/` — resource identity/reconciliation
- `plugins/test/` — transformation tests in the language
- `tools/languageserver/` — LSP
- `tools/linter/` — language tooling
- `mappings/` — reusable mapping assets

Use:
- donor for commandF Mapping IR
- donor for typed transformation bytecode/plan
- donor for plugin ABI concepts
- donor for mapping unit tests and language tooling

Adoption: `PORT + COPY` selected components after file-level provenance review. Do not make commandF internally Whistle-only; add a Whistle importer.

### 6. Microsoft FHIR Converter

Repository: `microsoft/FHIR-Converter`
License: MIT

Use:
- HL7v2 → FHIR template corpus
- C-CDA → FHIR template corpus
- JSON → FHIR mappings
- STU3 → R4 conversion patterns
- Liquid mapping runtime semantics
- FHIR → HL7v2 preview patterns

Adoption: `IMPORT` mapping corpus + `PORT` selected generic conversion abstractions. Add a Liquid importer instead of forcing hospitals to rewrite existing mappings.

### 7. FHIRconnect specification + openFHIR

Repositories:
- `SevKohler/FHIRconnect-spec`
- `openFHIR/openfhir`

Licenses verified on public repositories: Apache-2.0.

Use:
- formal YAML mapping DSL
- triple-layer reusable openEHR↔FHIR mapping architecture
- bidirectional mapping patterns
- archetype/template/profile resolution
- openFHIR execution engine tests
- mapping library

Adoption: `IMPORT + PORT + ORACLE`.

commandF should compile FHIRconnect into commandF Mapping IR, preserving FHIRconnect identifiers and provenance.

### 8. Eos + OMOCL

Repositories:
- `SevKohler/Eos`
- `SevKohler/OMOCL`

Eos public license verified: Apache-2.0.

Use:
- openEHR → OMOP ETL architecture
- OMOCL DSL and mapping corpus
- AQL-based visit-generation patterns
- ConceptMap/value-set integration
- structural-loss examples and tests

Adoption: `IMPORT + PORT + ORACLE`.

### 9. openEHR Archie + specifications + CKM mirror

Repositories:
- `openEHR/archie` — Apache-2.0
- `openEHR/specifications-RM`
- `openEHR/specifications-AM`
- `openEHR/specifications-QUERY`
- `openEHR/specifications-ITS-REST`
- `openEHR/CKM-mirror`
- `openEHR/openEHR-antlr4`

Use:
- Reference Model
- ADL/AOM/BMM parsing
- archetype/template validation
- AQL grammar and query semantics
- real archetype corpus

Adoption: `DEPEND/ORACLE` for Archie; `IMPORT` standards and test artifacts subject to artifact-level rights.

### 10. OHDSI / OMOP ecosystem

Repositories/projects:
- `OHDSI/CommonDataModel`
- `OHDSI/DataQualityDashboard`
- `OHDSI/WhiteRabbit`
- `OHDSI/Achilles`
- `OHDSI/Usagi`
- Atlas / HADES ecosystem as external analytics references

Use:
- OMOP 5.4 structural ground truth
- source profiling before transformation
- ~thousands of DQD conformance checks
- terminology mapping workflows
- ETL design patterns

Adoption: `DEPEND + IMPORT + ORACLE`.

### 11. Terminology: Snowstorm / Snowstorm Lite / TermX / Hades

Repositories:
- `IHTSDO/snowstorm`
- `IHTSDO/snowstorm-lite`
- `termx-health/termx-server`
- `wardle/hades`

Use:
- `$lookup`, `$validate-code`, `$expand`, `$translate`, `$subsumes`
- SNOMED CT ECL and hierarchy
- LOINC and FHIR package terminology
- terminology authoring/versioning and ConceptMap workflows
- HL7 terminology conformance tests
- Hades FTRM SQLite terminology-container idea

Adoption:
- Snowstorm/Snowstorm Lite: `DEPEND + ORACLE`
- TermX: `DEPEND + PORT` selected mapping/registry concepts (MIT reported by upstream)
- Hades: `STUDY + ORACLE` by default because EPL-2.0 is less convenient for copied core code

Important: terminology **software licenses do not grant terminology-content licenses**. SNOMED CT and other controlled vocabularies must remain separately licensed/content-addressed.

### 12. Inferno

Project: Inferno Framework / HL7 test kits

Use:
- executable FHIR conformance test DSL
- reusable test kits
- capability verification
- SMART/IG test patterns

Adoption: `ORACLE + PORT` test-harness concepts.

### 13. SUSHI / FSH ecosystem

Repository: `FHIR/sushi`
License verified: Apache-2.0

Also evaluate GoFSH and official IG Publisher.

Use:
- FHIR Shorthand parser/compiler
- profile/extension/ValueSet authoring semantics
- IG build fixtures
- profile normalization inputs for commandF IG Harmonizer

Adoption: `DEPEND + ORACLE`; import FSH AST only if justified.

### 14. FHIRPathMappingLanguage

Repository: `beda-software/FHIRPathMappingLanguage`
License: MIT

Use:
- alternative pragmatic mapping syntax built around FHIRPath
- real-world mapping UX lessons
- importer candidate

Adoption: `IMPORT + STUDY`, not commandF's canonical language.

---

## Tier A — runtime, query, provenance, and platform donors

### 15. SQL-on-FHIR + Pathling

Use:
- ViewDefinition
- SQLQuery operation
- FHIR-to-tabular projection
- analytics interoperability

Adoption: `DEPEND + IMPORT + ORACLE`.

Do not make FHIR JSON the analytics representation. Lower to Arrow/Parquet when appropriate.

### 16. Apache Arrow + DataFusion

Use:
- columnar in-memory representation
- Parquet interoperability
- Rust query planner/execution engine
- vectorized/streaming execution

Adoption: `DEPEND`.

This should fill the execution-engine gap instead of writing a database engine.

### 17. Substrait

Use:
- portable query-plan representation concepts
- extension functions/type system
- cross-engine query plan exchange

Adoption: `DEPEND/STUDY`.

Use its architecture as a donor for commandF Clinical Query IR; do not fork the spec into a healthcare-specific incompatible copy unless proven necessary.

### 18. MLIR

Use:
- multi-dialect typed IR architecture
- canonicalization
- verification passes
- lowering
- dialect conversion

Adoption: `STUDY` by default. commandF's Rust IR should borrow the architecture, not necessarily embed LLVM/MLIR.

### 19. OpenLineage + DataHub

Use:
- run/job/dataset lineage concepts
- extensible facets
- column-level lineage
- lineage graph UX

Adoption: OpenLineage `DEPEND/PORT`; DataHub `STUDY`.

commandF must add health-specific field-level transformation evidence above generic lineage.

### 20. Apache NiFi

Use:
- visual flows
- backpressure
- event provenance
- replay
- processors

Adoption: `STUDY`; use as commandF Studio/Gateway UX and provenance donor rather than embedding NiFi core.

### 21. OpenHIM

License: MPL-2.0

Use:
- healthcare interoperability mediation
- routing/orchestration
- transaction audit
- mediator lifecycle

Adoption: `STUDY + OPTIONAL DEPEND`.

### 22. Debezium

License: Apache-2.0

Use:
- CDC from PostgreSQL/MySQL/SQL Server/Oracle-family environments and others
- ordered change streams
- schema-change handling patterns

Adoption: `DEPEND`.

### 23. Wasmtime

License: Apache-2.0

Use:
- sandboxed adapter/plugin execution
- WASI/Component Model
- resource caps and capability boundaries

Adoption: `DEPEND`.

Target: commandF connector/plugin ABI should be Wasm-friendly from the start.

### 24. Cedar / OPA / OpenFGA / SPIFFE-SPIRE

Use:
- Cedar: typed fine-grained policy and policy validation
- OPA: general policy engine/Rego
- OpenFGA: relationship-based authorization
- SPIFFE/SPIRE: workload identity and SVID rotation

Adoption: `DEPEND` after an explicit security architecture decision; do not copy four policy stacks into commandF.

---

## Tier A — imaging and real-EHR fixtures

### 25. DICOM ecosystem

Evaluate/use:
- `pydicom`
- `dcm4che`
- Orthanc
- OHIF Viewer
- DCMTK

Use:
- DICOM parsing and validation
- DICOMweb
- DICOM SR/SEG metadata
- imaging↔FHIR semantic bridges

Adoption: mostly `DEPEND + ORACLE + TEST FIXTURE`.

### 26. Real EHR / FHIR implementation fixtures

Use as integration laboratories:
- OpenMRS + FHIR2 module
- Medplum
- OpenEMR
- Bahmni
- Beda FHIR EMR

Do not copy entire EHRs. Build commandF adapters and conformance fixtures against them.

Medplum is especially useful for FHIR-native developer experience and TypeScript tooling. OpenMRS is useful for a realistic modular EMR plus a FHIR layer.

---

## Tier S — benchmark and differential-testing donors

### 27. Synthea

License: Apache-2.0

Use:
- synthetic patient generation
- FHIR R4/STU3/DSTU2
- Bulk FHIR NDJSON
- C-CDA
- CSV/CPCDS

Adoption: `DEPEND + IMPORT FIXTURES`.

Use the same generated patient across representations to measure semantic conservation.

### 28. FHIR-AgentBench

Use:
- resource-retrieval evaluation
- agent tools
- ground-truth resource IDs
- 2,931 realistic questions

Adoption: research/benchmark only, respecting dataset/license conditions; do not mix benchmark data with runtime product assets.

### 29. FHIRBench

Use:
- serializer evaluation harness
- AI-oriented FHIR representations
- token/clinical-quality trade-off

Adoption: research-only benchmark initially.

### 30. fhirpath-lab

Use:
- differential FHIRPath execution against multiple implementations

Adoption: `STUDY + ORACLE`.

This pattern should become a generalized commandF differential-conformance laboratory.

---

## Gap → donor solution map

| Gap | Reuse first | commandF must add |
|---|---|---|
| FHIR-valid ≠ clinically equivalent | official Validator, HAPI, Firely, Inferno | Semantic Validator |
| Semantic information loss | FHIRconnect, Eos/OMOCL, mapping corpora | Loss Ledger + Conservation Score |
| No neutral clinical IR | Whistle proto, MLIR architecture | CSIR dialect system |
| N×N mapping explosion | FHIRconnect, Whistle, Microsoft Converter, OMOCL | Mapping IR + importers |
| Weak round-trip guarantees | FHIRconnect bidirectional maps, property-test frameworks | reversibility classes + round-trip verifier |
| FHIR version drift | fhir-codegen cross-version pipeline | version-aware FHIR dialect compiler + loss reports |
| IG/profile conflicts | SUSHI, IG Publisher, Firely snapshots | Profile Graph + IG Conflict Solver |
| terminology gaps | Snowstorm, TermX, Hades | terminology federation + gap registry + evidence |
| server behavior variance | Inferno, HAPI/Firely, fhirpath-lab | Compatibility Lab + empirical capability record |
| query fragmentation | SQL-on-FHIR, Pathling, AQL, Substrait, DataFusion | Clinical Query IR |
| weak field-level provenance | OpenLineage, NiFi | Transformation Evidence Graph |
| no transformation proof | validators + lineage | signed Transformation Certificate |
| difficult legacy ingestion | Microsoft Converter, Whistle, Debezium | adapter SDK + streaming/batch normalizer |
| unsafe third-party adapters | Wasmtime | capability-scoped plugin ABI |
| fragmented policy | SMART, Cedar/OPA/OpenFGA, SPIFFE | healthcare policy compilation layer |
| imaging silo | DICOM ecosystem | CSIR imaging dialect + FHIR/openEHR/OMOP bridges |
| inadequate research benchmarks | Synthea, Inferno, AgentBench, FHIRBench | commandF Bench |

---

## Explicit non-goals for copied code

Do not copy or inherit these architectural limitations:

- FHIR as the only internal canonical clinical model.
- pairwise converters as the main architecture.
- untyped JSON-only mapping execution.
- silent lossy transformation.
- cloud-only runtime.
- LLM output as semantic authority.
- server CapabilityStatement claims without empirical verification.
- terminology content bundled merely because the terminology server is open-source.

---

## Adoption order

1. Pin official FHIR artifacts + validator oracle.
2. Import FHIR cross-version mapping corpus from `FHIR/fhir-codegen`.
3. Prototype Mapping IR using Whistle + FHIRconnect + OMOCL as independent source languages.
4. Implement CSIR and Loss Ledger before building more target adapters.
5. Add openEHR (Archie/openFHIR) and OMOP (OHDSI/Eos) oracles.
6. Add terminology federation (Snowstorm Lite/TermX initially).
7. Build Synthea round-trip benchmark.
8. Add Inferno and differential HAPI/Firely testing.
9. Add DataFusion/Arrow query substrate and Query IR.
10. Add WASM adapter boundary, gateway/CDC, and policy plane.

No donor source should enter commandF without a pinned provenance entry.