# commandF Master Product & Execution Plan

Date: 2026-08-13
Status: Draft foundation plan
Repository: `TheHalfMoon/commandF`
Branch: `bootstrap/commandf-foundation`

> Research hypotheses, candidate papers, and master's-specific experimental protocols remain separated in `research/RESEARCH_AGENDA.md`. This document defines the product and engineering plan.

---

## 1. North Star

commandF is not a FHIR converter.

**commandF is the quality, intelligence, compilation, verification, and deployment layer for healthcare interoperability.**

It should make heterogeneous health-data changes reviewable with the same rigor that strong engineering organizations apply to source-code changes.

Core promise:

> Give commandF a healthcare interoperability change. It will understand the affected ecosystem, compile and validate the change, explain its blast radius, detect breaking and semantic regressions, generate evidence-backed remediation, prove what was preserved or lost, and only certify deployment when policy is satisfied.

### Product analogy

- Greptile-style context and blast radius, but for health-data assets.
- Cubic/Qodo-style review and remediation, but grounded in deterministic healthcare validators.
- Graphite-style stacks and merge queue, but for interoperability migrations and certifications.
- SonarQube-style quality gates, but for new mappings, profiles, terminology and connector changes.
- Buf-style lint/breaking/module/registry workflow, but across FHIR/openEHR/OMOP/HL7/DICOM/etc.
- OpenRewrite-style safe recipes, but for semantics-preserving health-data migrations.
- SLSA/Sigstore-style provenance and attestation, but for healthcare transformations.

---

## 2. Product invariants

1. **FHIR is a dialect, not the universal internal truth.**
2. **Target validity is necessary but not sufficient.**
3. **No silent loss.** Every lossy transformation emits typed evidence.
4. **AI proposes; deterministic systems decide what can be proven.**
5. **Every finding must have evidence.**
6. **Every automated fix must be recompiled, revalidated, and retested.**
7. **If commandF cannot prove a fix is safe, it does not silently apply it.**
8. **Compatibility is multidimensional.** Structural compatibility is not semantic compatibility.
9. **Every artifact is content-addressable and versioned.**
10. **Every adopted donor is provenance-pinned.**
11. **Clinical facts are never invented to satisfy a target schema.** Generated/default values are explicitly marked.
12. **Human policy can be stricter than standards.** Hospital, national, payer, research and organizational policy are first-class.
13. **Private/local deployment is a first-class architecture, not an afterthought.**
14. **A vector index is never the authoritative knowledge source.**
15. **Production certification must be reproducible from pinned inputs.**

---

## 3. Core platform architecture

```text
                   commandF Studio / CLI / GitHub App / API
                                  |
                                  v
+----------------------------------------------------------------+
|                    CHANGE INTELLIGENCE PLANE                    |
| Context Graph | Semantic Diff | Blast Radius | Review | Inbox   |
| Rules | Risk | Deep Review | Verified Fix | Migration Stacks    |
+----------------------------------------------------------------+
                                  |
                                  v
+----------------------------------------------------------------+
|                         COMPILER PLANE                           |
| Source Dialect -> Typed Source IR -> CSIR -> Mapping IR         |
| -> Terminology/Identity/Policy Passes -> Target Dialect         |
+----------------------------------------------------------------+
                                  |
                                  v
+----------------------------------------------------------------+
|                       VERIFICATION PLANE                         |
| Official Validators | Differential Oracles | Compatibility      |
| Semantic Verifier | Round Trip | Property/Fuzz | Quality Gates   |
+----------------------------------------------------------------+
                                  |
                                  v
+----------------------------------------------------------------+
|                         EVIDENCE PLANE                           |
| Loss Ledger | Provenance Graph | Findings | Certificates         |
| Signatures | Attestations | Lineage export | Audit trail          |
+----------------------------------------------------------------+
                                  |
                                  v
+----------------------------------------------------------------+
|                  REGISTRY / DELIVERY / RUNTIME                  |
| Packages | OCI artifacts | Lockfiles | Certification Queue      |
| Gateway | Batch | Streaming | CDC | Edge adapters | WASM plugins |
+----------------------------------------------------------------+
```

---

## 4. First-class healthcare dialects

### Tier 1

- FHIR R4 / R4B / R5, with R6 readiness
- openEHR RM / archetypes / templates
- OMOP CDM 5.4
- HL7 v2
- CDA / C-CDA

### Tier 2

- DICOM / DICOMweb
- SQL schemas
- CSV / JSON / XML / NDJSON / Parquet
- Bulk FHIR
- SQL-on-FHIR

### Tier 3 / ecosystem adapters

- vendor EHR APIs and feeds
- X12
- NCPDP
- national health exchange adapters
- organization-specific databases and flat-file protocols

---

## 5. commandF Context Engine

### Goal

Build a deterministic, queryable representation of the user's **interoperability ecosystem**, not just a document RAG index.

### Graph assets

```text
Standard
Version
FHIR Resource
StructureDefinition
ElementDefinition
ImplementationGuide
Profile
Extension
ValueSet
CodeSystem
ConceptMap
StructureMap
FHIR Package
openEHR Archetype
openEHR Template
AQL Query
OMOP Table
OMOP Field
OMOP Concept
HL7v2 Message
Segment
CDA Template
DICOM Object
Database Table
Column
Mapping
Mapping Rule
Recipe
Connector
Query
Policy
Rule
Test
Fixture
Transformation Run
Finding
Loss Event
Certificate
Organization
System
Environment
Owner
Incident
Decision
```

### Relationship examples

```text
PROFILE_DERIVES_FROM
BINDS_TO_VALUESET
USES_EXTENSION
MAPS_TO
LOWERS_TO
DEPENDS_ON
CONSUMED_BY
PRODUCED_BY
VALIDATED_BY
TESTED_BY
AFFECTED_BY
SUPERSEDES
CONFLICTS_WITH
EQUIVALENT_TO
BROADER_THAN
NARROWER_THAN
APPROVED_BY
DERIVED_FROM
```

### Storage strategy

Prototype a three-index architecture:

1. **Exact graph / provenance:** Oxigraph or a compatible embedded graph abstraction.
2. **Lexical full-text:** Tantivy.
3. **Semantic retrieval side-index:** Qdrant, optional and replaceable.

All three indexes point to immutable content-addressed commandF objects.

The LLM receives retrieved evidence. It does not become the graph.

---

## 6. Semantic Diff

`commandf diff` must explain changes in healthcare terms, not merely text terms.

Example:

```text
BREAKING SEMANTIC CHANGE

Patient.identifier
  min: 0 -> 1

Direct impact:
  4 mappings
  3 profiles
  2 connectors
  19 fixtures

Observed source coverage:
  Hospital-B lacks the required identifier in 7.3% of protected fixtures.

Round-trip impact:
  14 regressions

Consumers:
  OMOP patient ETL: affected
  openEHR demographics mapping: affected
  analytics projection: unaffected

Recommendation:
  stage compatibility rule before enforcing the new cardinality.
```

### Diff dimensions

- syntax
- cardinality
- datatype
- binding strength
- terminology
- value-set membership
- profile/IG constraints
- source/target coverage
- relationship semantics
- temporal semantics
- precision
- identity behavior
- privacy/security labels
- query behavior
- consumer compatibility
- reversibility
- known-loss delta

---

## 7. Breaking-change system

Inspired by Buf, commandF should make breaking-change detection mechanical and available in three locations:

1. developer workstation
2. pull request / CI
3. registry push / production certification

### Proposed compatibility categories

These are commandF categories, not HL7-defined categories.

```text
STRUCTURAL
FHIR
PROFILE
TERMINOLOGY
SEMANTIC
ROUND_TRIP
CONSUMER
FULL
```

Policies may additionally be `TRANSITIVE`, meaning comparison against all protected prior versions rather than only the latest.

Examples:

```bash
commandf breaking --against git:main --category PROFILE
commandf breaking --against registry:acme/lab-mapping@stable --category SEMANTIC
commandf breaking --against certificate:sha256:... --category FULL --transitive
```

### Behavior

A breaking result must distinguish:

- mechanically incompatible
- potentially incompatible
- empirically incompatible against known consumers/fixtures
- semantically lossy
- policy-prohibited
- unknown / requires human judgment

---

## 8. Quality profiles and Quality Gates

Adopt the strongest SonarQube idea: **protect new change first** rather than blocking adoption because legacy interoperability is already imperfect.

### Quality Profile

A versioned collection of rules, validators, thresholds, policies and required test suites.

Examples:

```text
commandf/standard
commandf/strict
saudi/production
hospital-x/laboratory
research/deidentified
vendor-y/r4-ingestion
```

### Quality Gate

A deployable change fails if any configured blocker is violated.

Example configurable gate:

```yaml
quality_gate:
  require:
    target_validation: pass
    undeclared_loss_events: 0
    blocker_findings: 0
    required_terminology_unresolved: 0
    protected_round_trip_regressions: 0
    protected_consumers_verified: true
    certificate_generated: true
  review:
    high_clinical_impact: required
    identity_ambiguity: required
    privacy_policy_change: required
```

No universal numeric semantic threshold is hard-coded. Organizations define their own risk tolerance.

---

## 9. Finding model

Every analyzer, agent, validator and external oracle emits one common commandF Finding contract.

Required concepts:

```text
id
rule_id
category
severity
confidence
status
summary
explanation
source_location
target_location
affected_assets
evidence
clinical_impact
compatibility_impact
loss_impact
policy_impact
fixability
suggested_actions
validator/oracle provenance
fingerprint
```

### Severity

```text
BLOCKER
HIGH
MEDIUM
LOW
INFO
```

### GitHub compatibility

Implement a SARIF exporter for repository-located findings so commandF can annotate GitHub code scanning / PR workflows without inventing a proprietary GitHub surface.

The commandF-native finding schema remains richer for clinical and cross-system locations that do not map cleanly to a source file.

---

## 10. Review system

### `commandf review`

Fast, high-signal review for normal changes.

Pipeline:

```text
Diff classifier
 -> deterministic lint
 -> graph impact traversal
 -> compatibility checks
 -> validator/oracle checks
 -> selected semantic checks
 -> AI explanation/triage
 -> findings deduplication
 -> quality gate
```

### `commandf review --deep`

Risk-triggered or user-requested deep review.

Specialized reviewers:

```text
Conformance Reviewer
Semantic Preservation Reviewer
Terminology Reviewer
FHIR Version Reviewer
Profile/IG Reviewer
openEHR Reviewer
OMOP Reviewer
HL7v2/CDA Reviewer
Identity Reviewer
Privacy Reviewer
Security/Policy Reviewer
Provenance Reviewer
Query Impact Reviewer
Performance Reviewer
Round-Trip Reviewer
Consumer Compatibility Reviewer
```

A final Judge/Triage stage removes duplicates, attaches evidence, and routes uncertain clinical decisions to humans.

### Critical rule

No LLM reviewer can convert an unverified result into `PASS` on its own.

---

## 11. Review effort modes

Automatically choose depth based on risk.

```text
PREFLIGHT
STANDARD
DEEP
CERTIFICATION
```

Risk signals include:

- affected patient identity logic
- required terminology bindings
- privacy/consent changes
- migration between FHIR major versions
- high fan-out blast radius
- changes to protected mappings
- new undeclared loss
- new national/organizational profile
- modifications to target cardinality
- production connector behavior
- previous incidents around the same asset

---

## 12. Risk analysis

`commandf risk` should return an evidence-based vector, not an unexplained AI score.

Example:

```text
Overall deployment risk: HIGH

Semantic impact:        HIGH
Terminology uncertainty:MEDIUM
Profile breakage:       HIGH
Privacy impact:         NONE
Identity impact:        LOW
Consumer blast radius:  37 assets
Round-trip regressions:  4
Protected tests passed:  96 / 100
Human review:           REQUIRED
```

Every component links to exact findings/evidence.

---

## 13. Rules and institutional memory

### Rule sources

- standard rules
- national rules
- organization rules
- system/vendor rules
- environment rules
- mapping-package rules
- approved human decisions

### Scoped inheritance

Borrow Greptile/Cubic's hierarchical rule concept:

```text
commandf.rules/
  global/
  jurisdiction/sa/
  fhir/r5/
  hospital-x/
  hospital-x/lab/
  production/
```

More-specific rules can strengthen or explicitly override allowed parent behavior, with provenance.

### Rule language strategy

Use two levels:

1. **CEL-like safe expressions** for fast deterministic predicates.
2. **CUE/Rego-style policy/constraint adapters** for richer structured policy and attestation validation.

Do not expose arbitrary Python/JavaScript evaluation in the trusted core.

### Learning

AI may propose a new rule from repeated human decisions, but a proposed rule is not active until approved and provenance-recorded.

---

## 14. Safe Recipe Engine / Verified AutoFix

This subsystem borrows the philosophy of OpenRewrite, not its source-code-specific model.

### Recipe types

```text
MappingRecipe
ProfileRecipe
TerminologyRecipe
VersionMigrationRecipe
DataRepairRecipe
PolicyMigrationRecipe
QueryMigrationRecipe
```

### Recipe invariants

- typed inputs and outputs
- explicit preconditions
- deterministic transformations where possible
- idempotence tested
- known-loss declaration
- reversible operation metadata where possible
- no-op when safety cannot be established
- dry-run plan before application
- generated patch + evidence

### AI role

AI can author a candidate recipe or suggest parameters.

Before a fix is called `VERIFIED`, commandF must run:

```text
compile
 -> deterministic validation
 -> terminology checks
 -> semantic checks
 -> round-trip/property tests
 -> protected regression suite
 -> consumer compatibility checks
 -> new certificate
```

---

## 15. Test system

### `commandf test`

Run deterministic fixture and integration suites.

### `commandf test --generate`

Generate candidate tests from the change and blast radius, then verify that generated tests meaningfully discriminate broken from correct behavior.

Test classes:

- nominal
- missing/optional source values
- invalid code
- terminology ambiguity
- unknown unit
- precision edge
- cardinality overflow
- reference failure
- temporal edge
- duplicate identity
- profile mismatch
- version mismatch
- privacy redaction
- loss declaration
- round trip
- consumer regression

### `commandf fuzz`

- Rust parser/compiler fuzzing via `cargo-fuzz`
- FHIR API property/stateful fuzzing via Schemathesis and commandF-specific FHIR state models
- corpus minimization and replay

### Differential testing

Protected behavior can be run against multiple independent implementations:

- official HL7 Validator
- HAPI
- Firely
- FHIR Candle
- Blaze
- LinuxForHealth FHIR
- Medplum
- other configured vendor servers

Disagreements become explicit Compatibility Lab findings.

---

## 16. Compatibility Lab

Goal: answer **what the implementation actually does**, not only what its CapabilityStatement claims.

Capabilities:

- endpoint discovery
- advertised capability capture
- empirical test execution
- stateful transaction sequences
- search parameter behavior
- include/revinclude behavior
- conditional create/update behavior
- version semantics
- terminology operations
- subscription behavior
- bulk-data behavior
- error/OperationOutcome behavior
- concurrency probes where safe

Outputs:

```text
claimed
observed
verified
partial
unsupported
inconsistent
unknown
```

---

## 17. Transformation Stacks

Inspired by stacked PRs, large interoperability migrations are decomposed into reviewable dependent changes.

Example:

```text
01 terminology package update
02 profile migration
03 mapping migration
04 FHIR R4 -> R5 lowering update
05 openEHR mapping update
06 OMOP ETL update
07 connector rollout
08 benchmark/regression certification
09 production release
```

Each stack node has:

- dependency edges
- independent diff
- independent findings
- independent certificate
- rollback information
- cumulative blast radius

---

## 18. Certification Queue

Graphite-like merge queue semantics for healthcare transformations.

The queue:

1. rebases/refreshes candidate against current protected registry state
2. re-runs required gates
3. re-tests affected consumers
4. creates signed evidence
5. publishes only if all protected conditions pass

The queue must detect when two independently safe mappings become unsafe when released together.

---

## 19. Review Inbox

One workspace for changes needing human attention.

```text
BLOCKERS
HIGH clinical impact
Terminology review
Identity review
Privacy review
IG migration
Vendor drift
Round-trip regression
Certification awaiting approval
```

Filters:

- owner
- system
- standard
- jurisdiction
- environment
- severity
- age
- mapping package
- affected patient-domain type

---

## 20. Living Interoperability Wiki

Generated documentation must always link back to authoritative assets/evidence.

Example question:

> How do allergies move from Hospital-X HL7v2 to OMOP?

Answer graph:

```text
HL7v2 AL1
 -> source assertion
 -> terminology mapping
 -> CSIR allergy concept
 -> FHIR AllergyIntolerance
 -> OMOP target tables
```

Show:

- mappings
- terminology
- owners
- known loss
- tests
- last certificate
- incidents
- change history

Generated prose is disposable; the underlying graph is authoritative.

---

## 21. Package and Registry model

### First-class package kinds

```text
profile-package
ig-package
mapping-package
terminology-package
rule-package
recipe-package
connector-package
query-package
benchmark-package
certificate-bundle
```

### Proposed repository files

```text
commandf.yaml
commandf.lock
commandf.rules/
commandf.recipes/
```

`commandf.lock` pins:

- standard packages
- terminology versions
- mappings
- rules
- validators/oracles where applicable
- external dependencies
- content hashes

### Distribution

Use OCI artifacts/ORAS as the initial transport rather than inventing a proprietary blob protocol.

Potential references:

```text
registry.example.com/commandf/mappings/lab:v3
registry.example.com/commandf/profiles/saudi-core:2026.1
registry.example.com/commandf/rules/hospital-x:7.2
```

All production artifacts are referenced by digest internally.

---

## 22. Signing and attestations

Use Sigstore/Cosign and in-toto-compatible attestations for:

- mapping packages
- rule packages
- recipes
- connector bundles
- benchmark evidence
- Transformation Certificates

A signature proves origin/integrity. It does **not** replace semantic verification.

### Proposed commandF certificate assurance levels

These are commandF-defined levels, inspired by provenance assurance systems.

```text
CF-C0  result exists, no verification guarantee
CF-C1  pinned inputs + deterministic evidence recorded
CF-C2  CI-produced signed certificate + protected validators/tests
CF-C3  hardened/reproducible certification + independent oracle policy
```

Exact requirements must be versioned and documented before public use.

---

## 23. Provenance and lineage

Internal provenance must support field/fact-level traceability:

```text
output fact
 -> mapping rule
 -> source fact
 -> source pointer
 -> source artifact digest
 -> terminology decision
 -> conversion operation
 -> validator evidence
```

Export compatible run/job/dataset lineage to OpenLineage where useful.

Do not force commandF's richer semantic lineage into a lower-fidelity external model.

---

## 24. Observability

Instrument the runtime with OpenTelemetry:

- traces for transformation/compiler stages
- metrics for throughput, latency, validation failures, cache behavior
- logs for operational events
- correlation IDs linking runtime traces to transformation evidence

Do not propagate PHI through generic telemetry baggage.

---

## 25. Developer surfaces

### CLI

Initial target UX:

```bash
commandf init
commandf build
commandf lint
commandf diff --against <ref>
commandf breaking --against <ref>
commandf review
commandf review --deep
commandf risk
commandf test
commandf test --generate
commandf fuzz
commandf verify
commandf explain <finding-or-asset>
commandf fix <finding>
commandf graph <asset>
commandf package build
commandf registry push
commandf registry pull
commandf certify
commandf deploy
```

### Git provider application

Initial GitHub-first experience:

- PR summary
- semantic diff
- blast radius
- inline findings
- action cards
- review gate/check
- fix/test actions
- persistent incremental review
- deep-review trigger

Use PR-Agent as donor/reference for provider integration and patch/context plumbing rather than building all VCS integrations from zero.

### Studio

Later visual surface:

- graph explorer
- mapping editor
- semantic diff viewer
- terminology browser
- test/fixture runner
- loss ledger
- certificate viewer
- stack/queue manager
- review inbox

### API / MCP

Expose deterministic commandF capabilities to AI agents and IDEs without giving agents implicit authority to bypass gates.

---

## 26. Agent architecture

Agents are consumers of deterministic commandF tools.

```text
Planner
Context Retriever
Reviewer Fleet
Test Planner
Fix Author
Judge/Triage
Explainer
```

Agents call tools such as:

```text
resolve_asset
query_graph
get_diff
run_rule
run_validator
translate_code
run_round_trip
run_fixture
run_consumer_test
get_loss_ledger
build_recipe
verify_recipe
```

No agent receives a raw unrestricted production data plane by default.

---

## 27. Donor strategy: build vs borrow

### Build and own

- CSIR
- Semantic Verifier
- Loss Ledger
- commandF Finding model
- Semantic Diff
- healthcare Blast Radius engine
- healthcare compatibility categories
- Mapping IR integration layer
- Transformation Certificate contract
- Context Graph healthcare ontology
- Certification Queue semantics
- commandF Bench integration layer
- clinical Query IR

### Borrow / depend

- authoritative FHIR/openEHR/OMOP tooling
- terminology engines
- DataFusion/Arrow/Parquet
- cargo-fuzz
- Schemathesis
- ORAS
- Cosign
- OpenTelemetry
- policy engines where appropriate

### Port / selectively copy after provenance pin

- Google Healthcare Data Harmonization / Whistle pieces
- FHIR cross-version mappings/codegen knowledge
- FHIRconnect/openFHIR mapping execution ideas/components
- Microsoft FHIR Converter mappings/templates
- Eos/OMOCL mapping assets
- PR-Agent provider/review plumbing
- selected OpenRewrite-inspired recipe framework patterns where direct code reuse is beneficial

### Oracle / test against

- official HL7 Validator
- HAPI
- Firely
- FHIR servers
- openEHR implementations
- OHDSI DQD
- terminology services
- Inferno suites

---

## 28. What commandF must not become

- a wrapper around an LLM
- another generic ETL UI
- a FHIR-only canonical data lake
- a proprietary mapping DSL that cannot import existing mappings
- a single-validator monoculture
- a vector database presented as a knowledge graph
- an opaque risk score
- a system that mutates mappings without evidence
- a runtime that requires cloud connectivity
- a giant fork of every donor repository

---

## 29. Execution roadmap

### P0 — Foundation contracts (current Draft PR)

Deliver:

- CSIR primitives
- LossEvent contract
- Transformation Certificate contract
- provenance policy
- donor manifest
- initial CI
- Master Plan

Exit gate:

- workspace green
- no adopted donor code with unpinned provenance
- interfaces explicitly unstable/pre-1.0

### P1 — FHIR Review MVP

Deliver:

- FHIR R4/R4B/R5 package loader/index
- official validator integration
- `commandf.yaml` and `commandf.lock` v0
- Finding schema v0
- rules v0
- Context Graph v0
- `lint`
- `diff`
- `breaking`
- `review`
- SARIF export
- GitHub PR integration
- Quality Gate v0

Primary product demo:

> Change a FHIR profile/mapping and commandF explains the semantic diff, blast radius, breaking consumers, findings, and gate result inside the PR.

### P2 — Mapping Compiler + Verified Fix

Deliver:

- Mapping IR v0
- FML/StructureMap importer
- FHIRconnect importer
- Whistle importer/adapter
- Microsoft Liquid importer where feasible
- Recipe Engine v0
- Verified AutoFix
- generated regression tests
- round-trip harness
- Loss Ledger integrated into lowering

Primary demo:

> commandF detects a mapping defect, proposes a safe recipe, applies it in a branch, and proves the fix against validators and regression fixtures.

### P3 — Cross-standard semantic triangle

Deliver:

- FHIR <-> CSIR <-> openEHR
- CSIR <-> OMOP 5.4
- terminology federation
- Synthea-based protected fixtures
- cross-model semantic diff
- commandF Bench v0
- signed certificates

Primary demo:

> the same synthetic patient truth crosses FHIR/openEHR/OMOP with explicit preservation and loss evidence.

### P4 — Enterprise interoperability

Deliver:

- HL7v2
- CDA/C-CDA
- DICOM metadata/SR core
- SQL/CSV ingestion
- CDC
- identity matching integration
- de-identification passes
- policy engine integration
- gateway
- WASM connector SDK
- registry service
- Certification Queue
- Transformation Stacks

Primary demo:

> hospital legacy feed to multiple target systems with review, policy, deployment and rollback evidence.

### P5 — Intelligence platform

Deliver:

- Deep Review fleet
- continuous background audit
- institutional rule mining with human approval
- Review Inbox
- Living Interoperability Wiki
- risk-weighted test planning
- impact history and incidents
- cross-repository/system review
- consumer drift monitoring

Primary demo:

> a new IG/terminology/vendor release arrives and commandF identifies exactly which systems and mappings need action before production breaks.

---

## 30. First implementation slices after foundation

Order matters.

```text
S1  Finding contract + SARIF exporter
S2  commandf.yaml / commandf.lock
S3  package/content-address model
S4  FHIR package indexer
S5  validator oracle abstraction
S6  graph asset/edge model
S7  lint + rules
S8  semantic/profile diff
S9  breaking checker
S10 GitHub review adapter
S11 quality gate
S12 protected fixtures + differential tests
S13 Mapping IR
S14 Recipe Engine
S15 round-trip verifier
```

Do not start with the full Studio UI.

The first compelling product should work from CLI + CI + GitHub.

---

## 31. Success metrics

### Review quality

- precision of surfaced blocker/high findings
- recall against curated defect corpus
- false-positive dismissal rate
- time to verified finding
- proportion of findings with deterministic evidence

### Change safety

- unintended breaking changes caught before registry/deployment
- protected consumer regressions prevented
- undeclared loss events prevented
- successful round-trip regressions detected

### Developer efficiency

- median review time
- mapping migration effort
- time from standards update to certified compatibility
- proportion of remediation completed by verified recipes

### Platform quality

- deterministic reproduction rate
- certificate verification success
- oracle disagreement rate
- fuzz defects discovered
- registry package reproducibility

Do not optimize for number of AI comments.

---

## 32. Sources and design inspirations

### Code intelligence / review

- Greptile graph context: https://www.greptile.com/docs/how-greptile-works/graph-based-codebase-context
- Greptile agent/review concepts: https://www.greptile.com/agent
- cubic AI review: https://docs.cubic.dev/ai-review/key-features
- Graphite workflow: https://graphite.com/docs/get-started
- Augment/Cosmos: https://www.augmentcode.com/
- Qodo review: https://docs.qodo.ai/code-review
- PR-Agent OSS: https://github.com/The-PR-Agent/pr-agent

### Quality / migration / compatibility

- OpenRewrite: https://docs.openrewrite.org/concepts-and-explanations/lossless-semantic-trees
- OpenRewrite recipes: https://docs.openrewrite.org/concepts-and-explanations/recipes
- Buf breaking: https://buf.build/docs/breaking/
- SonarQube Quality Gates: https://docs.sonarsource.com/sonarqube-server/2026.1/quality-standards-administration/managing-quality-gates/introduction-to-quality-gates
- CodeQL custom queries: https://docs.github.com/en/code-security/concepts/code-scanning/codeql/custom-queries
- SARIF upload: https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/integrate-with-existing-tools/upload-sarif-file

### Testing

- cargo-fuzz: https://github.com/rust-fuzz/cargo-fuzz
- Schemathesis: https://schemathesis.readthedocs.io/

### Packaging / evidence

- ORAS: https://oras.land/docs/
- Sigstore/Cosign: https://docs.sigstore.dev/
- SLSA: https://slsa.dev/spec/v1.2/
- OpenLineage: https://openlineage.io/
- OpenTelemetry: https://opentelemetry.io/docs/

### Rules / constraints

- CEL: https://cel.dev/
- CUE: https://cuelang.org/docs/
- OPA: https://www.openpolicyagent.org/docs/

### Context/search candidates

- Oxigraph: https://github.com/oxigraph/oxigraph
- Tantivy: https://github.com/quickwit-oss/tantivy
- Qdrant: https://github.com/qdrant/qdrant

### Healthcare donors

See:

- `docs/DONOR_AUDIT_2026-08-13.md`
- `docs/GAP_SOLUTION_MAP.md`
- `donors/manifest.yaml`
- `donors/final-pass-2026-08-13.yaml`
- `donors/review-quality-2026-08-13.yaml`

---

## 33. Immediate founder-level product definition

The first public story for commandF should be simple:

> **Review healthcare interoperability like code.**
>
> commandF understands your FHIR profiles, mappings, terminology and connected systems; tells you what a change will break; validates what can be proven; shows what clinical meaning may be lost; generates safe fixes and tests; and produces a signed certificate before deployment.

The conversion engine is essential infrastructure underneath this promise, not the whole product.
