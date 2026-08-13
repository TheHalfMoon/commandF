# commandF Gap → Solution Architecture

This document converts known interoperability gaps into build decisions. Research hypotheses are intentionally kept out of this file and live in `research/RESEARCH_AGENDA.md`.

## Architecture rule

FHIR is an essential dialect and exchange standard, but it is **not** commandF's sole internal canonical model.

The core pipeline is:

```text
source bytes/events
  -> source dialect parser
  -> typed source IR
  -> CSIR normalization
  -> mapping/terminology passes
  -> target dialect lowering
  -> target validator(s)
  -> semantic verifier
  -> loss ledger
  -> transformation certificate
```

## G01 — syntactic validity does not prove clinical equivalence

### Existing solution to reuse
- HL7 official Validator
- HAPI FHIR validator integration
- Firely validation as independent oracle
- Inferno test kits

### commandF addition
`SemanticVerifier` runs domain-neutral preservation checks:
- source facts represented in target
- values and units preserve magnitude/precision
- clinical code equivalence is evidenced
- temporal scope preserved
- subject/encounter/specimen/performer relationships preserved
- target defaults are marked generated rather than mistaken for source facts

No transformation receives `verified` status merely because the target validator passes.

## G02 — silent semantic loss

### commandF addition: Loss Ledger

Every lowering pass may emit typed `LossEvent`s:
- `Dropped`
- `Generalized`
- `Narrowed`
- `Split`
- `Merged`
- `Defaulted`
- `CodeApproximation`
- `UnitConversion`
- `PrecisionReduction`
- `CardinalityReduction`
- `TemporalApproximation`
- `ContextLoss`
- `ReferenceLoss`
- `UnsupportedTargetFeature`

Each event records source path, target path where applicable, rule id, evidence, recoverability, severity, and provenance.

## G03 — no neutral clinical IR

### commandF addition: CSIR

CSIR is a typed semantic intermediate representation, inspired by compiler multi-dialect architectures rather than a new EHR wire format.

Minimum semantic primitives:
- entity identity and aliases
- clinical concept/coding
- scalar/quantity/range/ratio/value-set values
- explicit time and temporal uncertainty
- actor/subject/specimen/location context
- relationships and provenance
- source assertions vs generated assertions
- security/policy labels
- unsupported/source-native payload preservation

CSIR must preserve source-native extensions as evidence until lowering is complete.

## G04 — N×N mapping explosion

### Existing solutions to ingest
- FHIR Mapping Language / StructureMap
- FHIRconnect
- Whistle
- OMOCL
- Microsoft Liquid templates
- ConceptMap
- FHIRPathMappingLanguage
- future SQL/CSV mapping imports

### commandF addition: Mapping IR

All imported mappings compile into a typed `MappingPlan` containing:
- source/target contracts
- selection predicates
- structural transforms
- terminology transforms
- unit transforms
- cardinality transforms
- defaults
- reference construction
- reversibility metadata
- asserted invariants
- declared losses
- tests
- provenance

Source mapping languages remain round-trippable where feasible; commandF does not erase authoring provenance.

## G05 — weak round-trip guarantees

### commandF addition
Each mapping rule receives one reversibility class:
- `Bijective`
- `LeftInvertible`
- `RightInvertible`
- `ConditionallyReversible`
- `Lossy`
- `Unknown`

Property tests are generated for supported classes.

Core checks:

```text
A -> B -> A'
semantic_equivalent(A, A')
```

and, when appropriate:

```text
B -> A -> B'
semantic_equivalent(B, B')
```

## G06 — FHIR version fragmentation

### Reuse
- `FHIR/fhir-codegen` cross-version converters/maps
- official FHIR packages
- HAPI and Firely multi-version implementations

### commandF addition
Represent R2/STU3/R4/R4B/R5/R6 as explicit dialect versions. Version conversion is normal compiler lowering, not JSON mutation.

Every version conversion reports:
- exact source/target spec version
- changed resource/type/element
- map/rule source
- introduced extensions
- unsupported fields
- irreversible loss

## G07 — profile / IG conflict

### Reuse
- SUSHI/FSH parser/compiler
- IG Publisher
- official package dependency metadata
- Firely/HAPI snapshot generation

### commandF addition: Profile Graph

Normalize StructureDefinitions into constraint graphs, then compare:
- base type compatibility
- cardinality intersection
- type/profile intersection
- slicing compatibility
- fixed/pattern conflicts
- binding strength/value-set intersection
- invariants
- mustSupport semantics
- extension identity/equivalence

Outputs:
- compatible
- equivalent
- strict subset
- strict superset
- conditionally compatible
- conflicting

The solver must produce a machine-readable conflict witness, not only prose.

## G08 — terminology fragmentation and local-code gaps

### Reuse
- Snowstorm / Snowstorm Lite
- TermX
- Hades as a conformance/performance oracle
- FHIR terminology ecosystem registry

### commandF addition: Terminology Federation

A resolver routes canonical terminology operations to authoritative configured servers and records:
- server identity
- terminology edition/version
- operation
- parameters
- result hash
- mapping equivalence/confidence

`TerminologyGap` is first-class when no acceptable map exists. Never hallucinate a code to make a transform pass.

## G09 — CapabilityStatement vs actual behavior

### Reuse
- Inferno
- official examples/TestScripts
- fhirpath-lab differential-testing pattern

### commandF addition: Compatibility Lab

For each server build/version:
- capture declared CapabilityStatement
- execute behavior probes
- record verified capability matrix
- detect drift across upgrades

Keep `claimed` and `observed` capabilities separate.

## G10 — fragmented clinical query languages

### Reuse
- FHIR Search/FHIRPath
- SQL-on-FHIR / Pathling
- openEHR AQL
- OMOP SQL
- Substrait architecture
- Apache DataFusion + Arrow

### commandF addition: Clinical Query IR

Initial scope should be read-only cohort/fact retrieval. Normalize concepts, predicates, temporal windows, traversals, aggregations, and output projections. Lower only when backend semantics are proven equivalent; otherwise report unsupported semantics.

## G11 — weak field-level lineage

### Reuse
- OpenLineage run/job/dataset concepts
- NiFi provenance/replay ideas
- DataHub lineage graph concepts

### commandF addition: Transformation Evidence Graph

Every output assertion can point to:
- one or more source assertions
- mapping rule
- terminology decision
- unit conversion
- generated/default reason
- validator result
- loss events

This graph is content-addressed and certificate-ready.

## G12 — no transformation proof artifact

### commandF addition: Transformation Certificate

Certificate includes:
- input/output content hashes
- source and target dialect versions
- profile/IG/package closure
- mapping package hashes
- terminology versions/resolution evidence
- validator identities/versions/results
- semantic verifier results
- loss ledger summary
- reproducibility/environment identifiers
- optional signature/attestation

Certificates should be machine-readable first, rendered for humans second.

## G13 — legacy / proprietary EHR ingestion

### Reuse
- Microsoft FHIR Converter templates
- Whistle mapping engine patterns
- Debezium CDC
- OpenHIM integration patterns
- OpenMRS/OpenEMR/Bahmni/Medplum as test systems

### commandF addition: Adapter SDK

Adapter contracts should distinguish:
- batch snapshots
- append-only feeds
- CDC/update streams
- request/response APIs
- document payloads
- binary/imaging references

Adapters produce source dialect IR and cannot bypass provenance or policy.

## G14 — unsafe third-party extensions

### Reuse
- Wasmtime/WASI Component Model

### commandF addition
A capability-scoped plugin ABI for parsers, emitters, connectors, policy hooks, and custom functions. Network/filesystem/secret access is explicit and denied by default.

## G15 — policy fragmentation

### Reuse
- SMART/OAuth/OIDC for identity/authorization protocols
- SPIFFE/SPIRE for workload identity
- OpenFGA for relationship authorization candidate
- Cedar or OPA for policy evaluation candidate

### commandF addition
Normalize healthcare policy facts (purpose-of-use, consent, security labels, jurisdiction, dataset sensitivity, actor, action, destination) before delegating to a policy engine.

No transformation engine may silently bypass policy because it is running as an internal job.

## G16 — imaging silo

### Reuse
- DICOM/DCMTK/dcm4che/pydicom/Orthanc family
- DICOMweb
- OHIF for inspection UX

### commandF addition
CSIR imaging dialect connects DICOM Study/Series/Instance/SR/SEG facts to FHIR/openEHR/analytics representations while retaining imaging identifiers and source references.

Do not copy pixel data into FHIR unless the target contract requires it.

## G17 — insufficient benchmark coverage

### Reuse
- Synthea multi-format exports
- Inferno
- FHIR-AgentBench
- FHIRBench
- OHDSI DQD
- independent FHIR implementations

### commandF addition: commandF Bench

Benchmark dimensions:
- syntax/profile validity
- semantic preservation
- terminology fidelity
- round-trip recovery
- profile compatibility
- server compatibility
- throughput/latency/memory
- reproducibility

Benchmark fixtures and research results are separated from production runtime code.

## Implementation priority

### Foundation 0
- provenance/donor manifest
- CSIR minimal types
- LossEvent + certificate contracts

### Foundation 1
- FHIR R4/R4B/R5 package/model ingestion
- official validator adapter
- cross-version map importer

### Foundation 2
- Mapping IR
- Whistle importer
- FHIRconnect importer
- Microsoft Liquid importer

### Foundation 3
- openEHR adapter/oracle
- OMOP 5.4 adapter/oracle
- terminology federation

### Foundation 4
- Synthea round-trip benchmark
- differential HAPI/Firely/official validation
- initial SemanticVerifier

Only after these are stable should commandF expand aggressively into CDA/HL7v2/DICOM/vendor EHR adapters.