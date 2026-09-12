# commandF Master Architecture V3 Candidate

Status: **PLANNING_CANDIDATE / STRATEGIC OVERLAY ONLY**

This document does not supersede `COMMAND_F_MASTER_ARCHITECTURE_V2.md`, does not reorder any currently authorized AF/CF implementation stack, and does not authorize production code by itself. V2 remains execution authority until a later, separately qualified migration gate says otherwise.

Canonical repository base used for this review:

```text
main: c56d9619758ea3e1075e751bec83ddea6706a81e
tree: 8c0cdef02a1169c9e815657397b3dcc9828c53bf
```

## Product north star

**commandF tells you what an interoperability change will break, who it can break, why, how certain that conclusion is, and what reproducible evidence supports the answer — before the change ships.**

The product should evolve from excellent FHIR change intelligence into the evidence layer for healthcare interoperability engineering. It should not become a replacement FHIR server, terminology server, integration engine, or universal semantic language merely to expand scope.

## Why V3 is needed

V2 correctly narrowed the implementation spine and prevented a premature universal IR. The current repository has since built unusually strong deterministic diff, compatibility, graph, impact, gate, oracle, source-map, and assurance foundations. The next competitive risk is no longer lack of ideas; it is failure to turn the preserved discovery envelope into measurable, dependency-ordered product capabilities.

V3 therefore adds four strategic requirements without invalidating V2:

1. **classification completeness** — prove which change space commandF understands and fail closed on the rest;
2. **consumer evidence** — model actual downstream contracts, not only producer artifact deltas;
3. **ecosystem intelligence** — analyze the historical public FHIR ecosystem reproducibly at scale;
4. **developer platform quality** — make the deterministic engine easy to adopt through stable APIs, local workflows, and bounded extensions.

## Non-negotiable invariants

The following remain true across every future slice:

- immutable package/artifact identity, exact versions, provenance, and digests are first-class;
- no silent coercion, discarded information, or unclassified compatibility state;
- AI may propose, summarize, rank, or explain, but deterministic code, pinned oracles, policy, tests, and human authority decide;
- an authoritative external validator is not reimplemented solely to remove a runtime boundary;
- mutable CI/current-build endpoints are telemetry, never immutable release authority;
- patient data is not required by the core review path; repository CI remains synthetic/public and PHI-free;
- terminology-content rights are governed separately from software licenses;
- vector/RAG indexes are optional retrieval aids, never canonical evidence;
- every adopted third-party source has a pinned identity, license/data-rights disposition, trust boundary, and explicit adoption mode;
- every public rule has rationale, positive tests, negative/counterexample tests, deterministic identity, and a lifecycle state;
- offline and air-gapped operation remains possible for all deterministic core analysis once required inputs are acquired;
- future plugins are capability-scoped, resource-bounded, deterministic by contract, and denied network/filesystem authority by default;
- no universal clinical IR becomes a product prerequisite without executed research evidence from multiple real dialects.
- static artifact compatibility never implies cross-system transaction/concurrency safety; any such claim requires measured runtime evidence;
- patient identity matching and general consent/authorization policy remain external/bounded domains: commandF may analyze declared compatibility contracts or adapters but does not claim a universal matcher or policy engine.

## Architecture: seven cooperating planes

### Plane 0 — Artifact truth

Owns immutable inputs and normalization needed by every higher layer:

- FHIR NPM package resolution, cache, lock, and multi-version package closure;
- canonical resources, stable element addressing, snapshots/differentials as evidence-bearing representations;
- exact resource/package/source digests;
- source-map fidelity back to FSH and other supported authoring inputs;
- artifact support/maturity matrix by resource type and FHIR version;
- an **Interoperability BOM** recording interoperability dependencies separately from a software SBOM.

The artifact plane never claims semantic equivalence merely because two resources parse or validate.

### Plane 1 — Context and ecosystem graph

Owns deterministic relationships:

- package/version dependency edges;
- canonical-reference edges independent of package dependency completeness;
- profile, extension, terminology, mapping, query, operation, capability, and consumer edges;
- immutable registry snapshots and historical release relationships;
- unresolved-canonical witnesses rather than silently dropped links;
- provenance links from each graph edge back to exact input bytes.

Embedded relational storage remains the default. A graph database is justified only by measured workloads, not architecture fashion.

### Plane 2 — Compatibility semantics

Owns typed changes, compatibility dimensions, and rule coverage.

A finding is not simply `BREAKING` or `SAFE`. It is evaluated across a versioned compatibility lattice:

| Dimension | Core question |
| --- | --- |
| `STRUCTURAL` | Did the machine-readable shape or constraint surface change? |
| `PRODUCER` | Can a producer valid under the old contract become invalid under the new contract? |
| `CONSUMER` | Can a consumer written to the old contract stop accepting/understanding the new output? |
| `TERMINOLOGY` | Did allowed coded meaning, binding strength, system/version, expansion, or translation behavior change? |
| `QUERY` | Can Search/FHIRPath/CQL/SQL-on-FHIR or other declared query behavior change? |
| `RUNTIME` | Does observed validator/server/client behavior diverge despite apparently compatible metadata? |
| `PROTOCOL` | Can a REST operation, SMART launch, Bulk Data, subscription/event, or other declared interaction contract stop working? |
| `AUTHORIZATION` | Did declared SMART/backend-service scopes, capabilities, or access expectations change in a way that breaks a protected client? |
| `AUTHORING` | Can source-authored FSH/mapping intent no longer reproduce the built artifact? |
| `MIGRATION` | Is an explicit migration/recipe required for protected consumers? |

The exact enum and cross-dimension algebra require their own Spec Kit freeze before implementation. V3 intentionally does not make this table executable authority.

### Plane 3 — Consumer contract intelligence

Owns evidence about who depends on what:

- FHIR Search usage and SearchParameter dependencies;
- FHIRPath expressions and context;
- CQL/ELM and Library dependencies;
- SQL-on-FHIR `ViewDefinition` dependencies;
- CapabilityStatement expectations;
- TestScript/Inferno contract evidence;
- selected observed HTTP interaction contracts where privacy policy permits;
- protected consumer/version sets used by `can-i-certify` decisions.

Consumer evidence must preserve deployment/version context. A consumer contract is never inferred solely from package popularity.

### Plane 4 — Review and migration experience

Owns point-of-change developer workflow:

- CLI, GitHub/GitLab review, SARIF, annotations, and stable machine JSON;
- semantic diff and blast radius;
- deterministic explanation record: what changed, dimension, why, affected consumers, evidence, source locations, and remediation options;
- baselines, suppressions, rule lifecycle, and quality gates;
- migration notes and verified dry-run recipes;
- IDE/LSP support and local preflight;
- optional Studio visualization without duplicating the semantic engine.

AI assistance can draft explanations, tests, or recipes, but the stored finding and acceptance result remain deterministic.

### Plane 5 — Compatibility Lab and evidence

Owns differential and empirical evidence:

- authoritative validator/IG Publisher comparisons;
- independent FHIR SDK/server comparisons;
- FHIRPath differential execution across independent engines;
- TestScript/Inferno execution where applicable;
- declared CapabilityStatement behavior versus observed behavior;
- reproducible IG source-to-package build evidence;
- versioned divergence classification instead of oracle laundering;
- signed evidence bundles and, later, transformation/conformance certificates.

Live external availability is reported separately from deterministic qualification.

### Plane 6 — Semantic and transformation research

Owns later cross-standard work only after evidence justifies it:

- mapping-analysis IR for FML/StructureMap, FHIRconnect, Whistle and other proven dialects;
- openEHR/OMOP correspondence analysis;
- explicit Loss Ledger and reversibility evidence;
- round-trip experiments;
- semantic-conservation benchmark methodology;
- possible multi-dialect typed IR only as an evidence-derived architecture.

This plane may fail scientifically without invalidating the FHIR change-intelligence product.

## Evidence truth classes

commandF must stop treating every corpus or external source as the same kind of truth. Every retained input belongs to exactly one class:

1. **NORMATIVE_OR_OFFICIAL** — standards, published packages, official registries and test specifications.
2. **AUTHORITATIVE_PROCESS_ORACLE** — official validator/publisher executions whose exact tool/config/input identities are pinned.
3. **INDEPENDENT_DIFFERENTIAL_ORACLE** — HAPI/Firely/Blaze/Candle/FHIRPath implementations and similar independent behavior references.
4. **REFERENCE_CORPUS** — official/shared test cases with known version and rights.
5. **HISTORICAL_ECOSYSTEM_CORPUS** — immutable public IG/package releases and deltas used to measure real-world behavior.
6. **ADVERSARIAL_REGRESSION_CORPUS** — synthetic/minimized fuzz/property/mutation regressions.
7. **LIVE_TELEMETRY_ONLY** — mutable CI/current builds, registry health, latest development branches, and external availability.
8. **PATTERN_OR_RESEARCH_INPUT** — prior art that may influence architecture but is not semantic authority.

A report must never upgrade one class into another merely because it is convenient.

## Artifact coverage and maturity matrix

The plan needs an explicit matrix rather than an implicit assumption that "FHIR resources" are uniformly understood. Each artifact type advances through `PARSE -> INDEX -> DIFF -> CLASSIFY -> IMPACT -> CONSUMER -> ORACLE` maturity, independently by FHIR version.

Initial priority groups:

| Priority | Artifact families |
| --- | --- |
| P0 | StructureDefinition profiles/extensions, ValueSet, CodeSystem, ConceptMap, SearchParameter |
| P1 | OperationDefinition, CapabilityStatement, ImplementationGuide, StructureMap, NamingSystem |
| P1 | Questionnaire, Library, Measure, PlanDefinition |
| P2 | GraphDefinition, CompartmentDefinition, MessageDefinition, SubscriptionTopic/Subscription |
| P2 | additional canonical/conformance resources proven relevant by the ecosystem corpus |

Unsupported artifact/field transitions must be explicitly reported as unsupported/unclassified, never silently ignored.


## Protocol coverage and maturity matrix

FHIR interoperability risk also exists above individual resource schemas. Protocol support therefore advances through an independent maturity matrix rather than being implied by artifact support.

Initial candidate order:

| Priority | Protocol/interaction surfaces |
| --- | --- |
| P0 | FHIR REST interactions/operations, Search behavior, CapabilityStatement-declared behavior |
| P1 | SMART App Launch/backend-service declarations and scopes; FHIR Bulk Data export |
| P1 | R4/R4B/R5 subscriptions/eventing where standards/IGs define computable contracts |
| P2 | bounded FHIRcast/CDS Hooks contracts when an owned consumer/IG declares them |
| P2 | GraphQL-on-FHIR and other evolving interfaces only where ecosystem evidence justifies support |

A protocol can be `DISCOVER -> PARSE -> DIFF -> CLASSIFY -> CONSUMER -> ORACLE` mature independently of the underlying resource artifact maturity. Authorization analysis is compatibility evidence about declared client/server expectations; commandF does not become a universal access-control engine.

## Change-space coverage

CF-04 rules are valuable but rule count is not proof of semantic coverage. A future coverage unit must define the finite, machine-readable change space for each supported artifact model and prove that every transition is one of:

- classified by at least one compatibility rule;
- explicitly non-semantic/ignored with reviewed rationale;
- version-inapplicable;
- intentionally unsupported and fail-closed.

Coverage must be queryable as a machine artifact. A new field/model version cannot silently create an uncovered transition.

## Rule lifecycle

Rules require a lifecycle distinct from code release status. Candidate lifecycle:

```text
PROPOSED -> EXPERIMENTAL -> VALIDATED -> DEFAULT -> DEPRECATED
```

Promotion requires retained evidence: positive/negative cases, real-corpus observations where applicable, false-positive/false-negative analysis, affected FHIR versions/artifacts, and rule-identity migration behavior. The exact lifecycle is frozen only in the future compatibility-coverage Spec Kit.

## Finding contract

Every stable machine finding should eventually bind:

- finding/fingerprint schema version;
- rule id and rule version;
- old/new package and artifact identities;
- normalized changed location plus source-authored location when available;
- compatibility dimensions and direction;
- evidence class and evidence references;
- affected graph/consumer nodes;
- classification and uncertainty/unsupported state;
- rationale and machine-readable remediation category;
- baseline/suppression disposition;
- commandF engine/version identity.

Human explanation may evolve without changing the stable semantic identity unless the underlying classification changes.

## Ecosystem Observatory architecture

A future observatory should ingest immutable public ecosystem evidence without becoming a package registry:

- official FHIR IG registry snapshots;
- official package-feed discovery metadata;
- immutable package releases and historical publication pages;
- core/extensions/package releases;
- selected public national/vendor-independent IG releases subject to rights;
- mutable CI build metadata only in the telemetry partition.

Each ingest snapshot records retrieval time, source URL, upstream immutable identity when available, raw digest, parser version, and rights/provenance disposition. Historical analysis is reproducible from frozen snapshots.

The observatory enables standards/vendor drift, empirical rule validation, compatibility trend analysis, and a future public evidence site without becoming semantic authority itself.

## Scale and incremental computation

Before ecosystem-scale analysis, commandF needs explicit scaling semantics:

- content-addressed raw inputs and normalized artifacts;
- deterministic cache keys over input digests + engine/rule/config identities;
- incremental graph updates keyed by immutable artifact identities;
- invalidation based on dependency edges rather than global rebuilds;
- bounded parallelism and resource envelopes measured by AF-04;
- local/offline mirrors with the same logical identity as network-fetched artifacts;
- no cache hit may bypass a changed rule/model/tool identity.

A Merkle-style identity graph is a useful implementation pattern, not a mandated storage technology.

## Interoperability BOM

Software SBOM answers which software components built or shipped commandF. The Interoperability BOM answers which interoperability contracts a result depended on.

Candidate fields include:

- exact FHIR package closure and digests;
- canonical resource identities and source package/version;
- terminology system/version endpoints or package identities;
- mappings, rules, query contracts, and consumer contracts;
- validator/publisher/tool identities;
- source-to-built IG evidence;
- policy/baseline identities;
- evidence bundle digest.

Where a generic standard can represent a relationship precisely, export to it. commandF must not distort CycloneDX, SPDX, purl, or other standards to encode healthcare semantics they do not define.

## Reproducible IG build evidence

For source-authored IG review, commandF should be able to distinguish source changes from build-tool drift. Future reproducible-build evidence should pin:

- SUSHI version/configuration;
- IG Publisher version/digest/configuration;
- FHIR core/package closure;
- terminology dependencies or explicitly offline terminology mode;
- source tree identity;
- produced package/output digests;
- source-map/GoFSH round-trip observations where applicable.

Network-disabled Publisher execution should be preferred where the required input closure permits it. Failure to reproduce becomes a classified finding, not an excuse to trust mutable CI output.

## Developer platform

"Best" requires adoption quality as well as semantic depth. The deterministic engine should eventually expose one stable contract through multiple surfaces:

- CLI;
- GitHub/GitLab CI;
- stable JSON schemas and exit codes;
- library API with compatibility policy;
- local daemon/cache for large workspaces;
- pre-commit/pre-push integration;
- LSP/VS Code diagnostics and source navigation;
- optional read-only MCP adapter over typed deterministic APIs;
- Studio UI consuming the same API rather than reimplementing rules.

No surface receives privileged semantic logic.

## Plugin and institutional policy seam

Core compatibility semantics remain built in and reviewable. Institutional extensions may later run as pinned, signed WebAssembly components with explicit capability manifests.

Required defaults:

- no network;
- no ambient filesystem;
- explicit host functions only;
- input/output JSON or WIT contracts with schema validation;
- memory/time/fuel/output limits;
- deterministic clock/randomness policy;
- plugin digest/signature/provenance retained in evidence;
- plugin failure cannot become silent PASS;
- organization policy expressions are separated from healthcare semantic rules.

Extism/Wasmtime patterns may be studied; no runtime is adopted by this document.

## Offline and air-gapped product mode

Offline capability is a product invariant, not merely an assurance-runner feature. A future bundle/import workflow should carry:

- packages and exact dependency closure;
- configured validator/oracle artifacts when redistribution permits;
- terminology metadata/content only when licensing permits;
- rules, policies, baselines, consumer contracts, and evidence schemas;
- provenance manifest and verification material.

Online convenience must not change deterministic analysis semantics.

## Version support policy

Support must be explicit by capability, not implied by parsing.

- R4 may carry the strongest production evidence first.
- R4B and R5 advance capability-by-capability through the artifact maturity matrix.
- R6 remains preview/research while it is a draft publication sequence; mutable CI builds can inform readiness but cannot support a production compatibility guarantee.
- cross-version conversion evidence is always labeled separately from same-version compatibility.

## Configuration, error, workspace, and privacy contracts

Future V3 surfaces must share one deterministic configuration model with explicit precedence and retained effective semantic identity. Environment variables may carry secrets/locations where necessary but must not silently change compatibility policy.

Machine interfaces must classify invalid input, unsupported version/protocol, unclassified transition, configuration/policy error, oracle unavailable/unsupported/disagreement, resource-limit failure, rights/privacy denial, plugin failure, and internal invariant failure separately. Partial reports state which stages completed; missing evidence is never equivalent to no impact.

Workspace analysis must define multi-package roots, exact commit/range identity, stacked-change behavior, generated/cache exclusions, and stable source identifiers without leaking host-local paths into portable evidence.

Observed HTTP interaction evidence is optional and requires an explicit opt-in, minimization/redaction/retention contract plus synthetic/public fixtures that qualify the same semantic path. Core repository CI remains PHI-free.

## Execution playbook

`COMMAND_F_V3_EXECUTION_PLAYBOOK.md` is the implementation-facing companion to this architecture candidate. It provides the candidate dependency DAG, universal Spec Kit skeleton, G01-G40 executable closure contracts, per-slice work packages, exact acceptance expectations, protocol scope, configuration/error/privacy/workspace rules, and agent operating loop. It remains planning-only until a later canonical migration gate activates V3 execution authority.

## New post-CF-16 candidate roadmap

These identities are planning candidates only and do not renumber, bypass, or authorize the current CF-01..CF-16 or AF program.

### CF-17 — Ecosystem Observatory

Ships immutable registry/feed snapshots, historical package/artifact graph, unresolved-canonical evidence, and strictly separated mutable-CI telemetry.

Depends on: CF-11G, AF-03 for stable distribution claims, AF-04 before quantitative scale claims.

### CF-18 — Compatibility Coverage Model

Ships artifact maturity matrix, versioned compatibility dimensions, machine-readable change-space coverage, rule lifecycle, and fail-closed uncovered-transition evidence.

Depends on: CF-03/04/07/13 plus real-corpus evidence.

### CF-19 — Consumer Contract Scanner

Ships parsers/indexers for selected Search, SearchParameter, FHIRPath, CQL, SQL-on-FHIR ViewDefinition, CapabilityStatement, TestScript, REST-operation, SMART/Bulk/subscription declarations and other bounded protocol consumer evidence plus protected consumer/version edges and privacy-safe optional interaction-contract imports.

Depends on: CF-11G, CF-12, CF-18.

### CF-20 — Compatibility Lab

Ships reproducible differential matrices for selected validators/FHIRPath engines/test frameworks and supported protocol conformance suites, plus declared-versus-observed behavior with external availability separated from deterministic conclusions.

Depends on: CF-06, CF-10 evidence model, CF-18/19.

### CF-21 — Interoperability BOM and Reproducible Build Evidence

Ships exact interoperability dependency closure, IG source-to-built evidence, a versioned Interoperability BOM, and versioned validated machine export/bundle formats distinct from software SBOM.

Depends on: CF-01/02/09/17 and AF-03.

### CF-22 — Deterministic TestGen

Ships requirement/change -> test-plan/TestScript/regression generation where every generated assertion is traceable to deterministic source evidence. AI-assisted requirement extraction is optional and cannot activate tests without deterministic/human acceptance.

Depends on: CF-18/20/21.

### CF-23 — Developer Platform

Ships stable machine API, deterministic configuration/error contracts, workspace/monorepo comparison semantics, local daemon/cache, pre-commit workflow, LSP/VS Code integration, verified install/update flows, and optional read-only MCP adapter over the same deterministic engine.

Depends on: stabilized finding/compatibility schemas from CF-18/19.

### CF-24 — Capability-Scoped Plugin and Policy SDK

Ships pinned/signed Wasm plugin contracts, capability manifests, resource limits, API/ABI compatibility and revocation/key-rollover policy, organization policy adapters, and provenance. Core compatibility rules remain outside the plugin trust boundary.

Depends on: CF-18, AF-03/04, CF-23 stable API.

### CF-25 — Terminology Gap and Federation Intelligence

Ships terminology-endpoint/package/version capability evidence, gap classifications, federated lookup/validation adapters, and reproducible terminology observations without becoming a terminology server.

Depends on: CF-07, CF-17, CF-20.

### CF-26 — Transformation Evidence and Certificates

Ships Loss Ledger, transformation evidence graph, reproducible certificate envelope, and signature/attestation integration. A signature proves provenance/integrity, not semantic correctness.

Depends on: CF-16, CF-21, CF-25 and executed transformation research.

### CF-27 — Public Compatibility Observatory

Ships a public evidence view/database over immutable analyzed releases, rule coverage, standards drift, and compatibility history, with explicit freshness/correction/retraction semantics. It hosts analysis evidence, not FHIR packages as a competing registry.

Depends on: CF-17..22 and production-grade privacy/provenance review.

### CF-28 — Bench and Research Convergence

Ships reproducible benchmark packs for supported compatibility, mapping, round-trip, terminology and semantic-conservation questions. Null/negative results are first-class.

Depends on: commandF Bench research protocol plus relevant product slices.

## Execution horizons

### Horizon 0 — Finish the foundation

Complete currently governed AF-02, AF-03, AF-04 and the already retained CF-14..CF-16 only under their own dependency and Spec Kit authority. Planning for future slices must not create implementation shortcuts around these gates.

### Horizon 1 — World-class FHIR change intelligence

CF-17 through CF-22. The goal is classification completeness, real consumer impact, empirical compatibility, reproducible builds, and executable test evidence.

### Horizon 2 — Platform and adoption

CF-23 through CF-25. The goal is a stable integration surface, safe institutional extensibility, and terminology intelligence.

### Horizon 3 — Semantic trust and cross-standard evidence

CF-26 through CF-28. The goal is transformation evidence, public reproducibility, and research-backed cross-standard capabilities.

## Open-source qualification rule

A project is never adopted because it is popular or convenient. Before use, record:

- authority class and intended role;
- exact commit/release/digest;
- license plus separate data/content rights;
- maintenance/release state and deprecation/archival state;
- whether it executes untrusted input or requires network access;
- deterministic/offline compatibility;
- version/standard coverage;
- independence from existing oracles;
- dependency/runtime cost;
- one adoption mode: `PINNED_DATA`, `PINNED_PROCESS_ORACLE`, `DIFFERENTIAL_ONLY`, `OPTIONAL_TOOLCHAIN`, `PATTERN_ONLY`, `RESEARCH_ONLY`, or `REJECT`.

The deep-research inventory for this candidate is `COMMAND_F_OPEN_SOURCE_QUALIFICATION_2026-09-12.md`.

## What "best ever" must mean in evidence

commandF should never claim superiority from feature count. A world-class claim is justified only by public, reproducible evidence showing that the relevant version can:

- enumerate its supported artifact/change space and expose uncovered states;
- reproduce every finding from immutable inputs and exact rule/tool identities;
- distinguish normative, oracle, differential, corpus, and live-telemetry evidence;
- demonstrate consumer-aware impact rather than only producer-side diff;
- retain every disagreement with an authoritative or independent oracle until classified;
- run deterministic core analysis offline from a complete input bundle;
- survive adversarial, mutation, portability, release, and performance assurance applicable to the claim;
- publish benchmark methodology and negative results;
- avoid patient data for its core review path;
- maintain stable machine contracts for downstream automation.

The benchmark decides whether commandF is better. The roadmap only creates the conditions to measure it.

## Plan migration rule

A future proposal may supersede V2 only after it reconciles:

- this V3 candidate;
- V2 execution authority;
- product family;
- gap ledger;
- discovery coverage annex;
- donor/provenance policy;
- research charter;
- AF assurance program;
- every canonical CF/AF Spec Kit and live repository gate.

No current task, donor, research track, non-goal, or governance requirement may disappear silently.
