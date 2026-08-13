# commandF Master Architecture V2

Status: execution authority for the post-bootstrap build order.

## Product sentence

**commandF tells you what your interoperability change will break — before you ship it.**

The long-term vision remains broader: healthcare interoperability change intelligence, verification, mapping review, loss analysis, and evidence. The build order is deliberately narrower so every layer ships independently.

## Architectural correction

The superseded bootstrap treated a future semantic compiler architecture as the immediate implementation plan. V2 separates the product critical path from research/future semantics.

The first product does **not** require a universal clinical semantic IR, mapping execution engine, terminology server, EMPI, custom registry, or certification authority.

## Four planes

### 0. Artifact plane — build first

- FHIR NPM package resolution and deterministic lockfile
- content-addressed package cache
- canonical resource index
- stable element addressing, including slicing
- typed structural diff
- versioned breaking-change rule corpus
- SARIF and CLI exit codes

This plane must be useful by itself.

### 1. Graph plane — earned by artifact data

- published package dependency graph
- profile/extension/value-set/code-system dependency edges
- blast radius over public artifacts
- terminology gap registry
- query impact over SQL-on-FHIR ViewDefinitions, CQL, SearchParameters, and FHIRPath invariants

Start with embedded relational storage unless measurements justify a graph database.

### 2. Review plane — point-of-change delivery

- GitHub/GitLab annotations
- FSH/SUSHI source mapping
- baselines and suppressions for legacy debt
- quality gates
- deterministic AutoFix recipes, dry-run first
- AI proposals only under deterministic acceptance

CodeRabbit and Qodo are build-time independent reviewers for commandF itself. Graphite-style stacks are the development workflow for commandF slices.

### 3. Semantic plane — future, evidence-derived

- mapping analysis IR for FML/StructureMap, FHIRconnect, and Whistle
- openEHR and OMOP correspondence analysis
- explicit transformation loss vocabulary and Loss Ledger
- round-trip experiments
- signed reproducible evidence bundles

A future CSIR is **not deleted from the vision**, but it is not a V1 dependency. If justified, it must emerge from at least two implemented dialects and use dialect/conversion concepts closer to MLIR than a single universal LLVM-style IR.

## What commandF owns

The strongest defensible assets are:

1. a versioned interoperability breaking-change taxonomy and rule corpus;
2. a cross-artifact dependency graph over the public healthcare conformance ecosystem;
3. an open, citable vocabulary for transformation loss;
4. FSH-authored-source fidelity for review findings;
5. empirically validated differential behavior against authoritative and independent oracles.

## What commandF adopts instead of inventing

- FHIR NPM package format and public package registries
- CRMI lifecycle/dependency semantics where applicable
- HL7 Validator and IG Publisher as Tier-1 oracles
- HAPI/Firely/FHIR Candle/Blaze/etc. as independent comparison oracles
- FSH/SUSHI rather than a new FHIR authoring language
- FML/StructureMap, FHIRconnect, and Whistle rather than a new mapping language
- SQL-on-FHIR v2, CQL, FHIRPath rather than a new universal query language
- Snowstorm/TermX/OCL rather than a terminology server
- existing MPI/EMPI systems rather than patient matching
- SARIF for findings interchange
- OpenLineage/in-toto/SLSA/Sigstore/ORAS for mature lineage/attestation/signing plumbing where later required

## Oracle rule

commandF does not reimplement a validation judgment already supplied by an authoritative implementation merely to remove a JVM dependency. Where commandF computes overlapping behavior, differential testing is mandatory and every divergence must be classified.

The JVM oracle boundary is permanent and intentional.

## Product trust boundary

The primary review product analyzes conformance metadata, not patient data.

Any future source profiler that touches instances must be a separate on-premises trust boundary and emit aggregate/statistical evidence only by default. Synthetic/public fixtures are mandatory in repository CI.

## First execution stack

| Slice | User-visible result | Depends on |
|---|---|---|
| CF-01 | `commandf pkg` | — |
| CF-02 | `commandf inspect` | CF-01 |
| CF-03 | `commandf diff` | CF-02 |
| CF-04 | BREAKING/RISKY/ADDITIVE rules, producer + consumer direction | CF-03 |
| CF-05 | `commandf check --format sarif` | CF-04 |
| CF-06 | published HL7-oracle divergence report | CF-03 |
| CF-07 | ValueSet/CodeSystem/binding diff | CF-04 |
| CF-08 | GitHub Action + annotations | CF-05 |
| CF-09 | FSH source mapping | CF-08 |
| CF-10 | public real-IG delta corpus | CF-06, CF-07 |
| CF-11 | ecosystem context graph | CF-02 |
| CF-12 | `commandf impact` | CF-11 |
| CF-13 | baselines/suppression/quality gates | CF-05 |
| CF-14 | on-prem aggregate-only source profiler | CF-02 |
| CF-15 | verified dry-run recipes | CF-04, CF-09 |
| CF-16 | mapping analysis IR, parse-only | CF-11 |

## Mandatory acceptance gates

Every PR must satisfy:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- deterministic output for identical inputs where applicable
- no silent unclassified delta or compatibility state
- no new crate unless a shipped command or immediately exercised test uses it
- no PHI fixtures
- no redistributed terminology content whose license has not been explicitly cleared
- CodeRabbit review when available
- Qodo review when connected/available
- no merge of a stack whose exact candidate state is not green

## CF-01 authority

CF-01 owns only FHIR package acquisition, dependency resolution, content-addressed caching, and deterministic locking.

It does not validate FHIR resources, build snapshots, index canonicals, diff artifacts, or execute mappings.

## References that constrain V2

- HL7 FHIR NPM Packages: https://hl7.org/fhir/packages.html
- HL7 CRMI STU1: https://hl7.org/fhir/uv/crmi/STU1/
- FHIR Shorthand / SUSHI: https://build.fhir.org/ig/HL7/fhir-shorthand/
- SQL-on-FHIR v2: https://sql-on-fhir.org/ig/2.0.0/

Research hypotheses remain separate from product guarantees.
