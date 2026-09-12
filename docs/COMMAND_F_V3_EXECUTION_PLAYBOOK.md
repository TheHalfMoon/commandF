# commandF V3 Execution Playbook

Status: **PLANNING_CANDIDATE / BUILD PLAYBOOK**

This document turns the V3 strategic overlay into a dependency-ordered build system. It does not supersede `COMMAND_F_MASTER_ARCHITECTURE_V2.md`, does not authorize a CF/AF implementation by itself, and does not borrow evidence from future slices. Every implementation slice still requires its own canonical Spec Kit and exact-head qualification.

## 1. How to use this playbook

An agent or maintainer should be able to answer four questions from this file without guessing:

1. **What is the next eligible unit?** Follow the dependency graph and current canonical repository truth.
2. **What exactly must that unit ship?** Use the slice card and its closure contracts.
3. **How is it proven?** Use the global acceptance contract plus slice-specific evidence.
4. **When may work move forward?** Only after the unit's Spec Kit, implementation, evidence, review, and canonical merge converge.

If this playbook conflicts with live canonical Spec Kit authority, the canonical Spec Kit wins. If it conflicts with V2 before a V3 migration gate is canonical, V2 wins.

## 2. Non-negotiable execution rules

- Re-read canonical `main`, rulesets, active PRs, and the governing Spec Kit before every new implementation unit.
- Never treat a planning document, donor repository, mutable upstream branch, AI output, or live service response as implementation authority.
- Never reuse stale CI/review evidence after candidate head or relevant base truth changes.
- No silent downgrade from unsupported/unclassified/error into PASS.
- No new network, filesystem, process, plugin, schema, storage, or privacy authority enters the product without an explicit contract and tests.
- Every new persisted or public machine schema has a version, compatibility policy, validator, and migration/rejection behavior.
- Every external source has identity, provenance, rights/license disposition, trust class, and an adoption mode before use.
- Every deterministic core path remains reproducible offline once its declared inputs have been acquired.
- Quantitative scale/performance claims require AF-04 evidence; release/distribution claims require AF-03 evidence.
- Patient data is never required to qualify core behavior. Any later observed-interaction workflow must have a privacy-safe contract and synthetic/public test path.

## 3. Dependency graph

The existing V2/AF frontier remains authoritative until canonical migration. The candidate post-CF-16 graph is:

```text
CURRENT AUTHORITY
  AF-02 ---> AF-03 ---> AF-04
     |          |          |
     |          |          +---------------------------+
     |          +----------------------+               |
     v                                 v               v
  existing CF-14 / CF-15 / CF-16 under their own V2 Spec Kits

POST-CF-16 CANDIDATE
  CF-17 Ecosystem Observatory -----------+--------------------+
       |                                 |                    |
       v                                 v                    v
  CF-18 Compatibility Coverage -----> CF-19 Consumer Contracts
       |                                 |                    |
       +--------------+------------------+                    |
                      v                                       |
                 CF-20 Compatibility Lab <--------------------+
                      |
  CF-17 --------------+-----> CF-21 Interop BOM + Repro Build
                      |                    |
                      +--------------------+
                                           v
                                      CF-22 TestGen

  CF-18 + CF-19 -----------------------> CF-23 Developer Platform
  CF-23 + AF-03 + AF-04 --------------> CF-24 Plugin/Policy SDK
  CF-07 + CF-17 + CF-20 --------------> CF-25 Terminology Intelligence

  CF-16 + CF-21 + CF-25 --------------> CF-26 Transformation Evidence
  CF-17..CF-22 + privacy review -------> CF-27 Public Observatory
  relevant shipped slices + research --> CF-28 Bench/Research Convergence
```

Planning may happen ahead of implementation, but closure evidence may not be borrowed across an unmet dependency edge.

## 4. Build waves

### Wave 0 — Finish governed foundation

Do not use V3 to bypass current work. Complete AF-02/03/04 and CF-14/15/16 only according to their canonical Spec Kits and live dependency state.

### Wave 1 — Make FHIR change intelligence provably complete

Build CF-17 through CF-22 according to the dependency graph. The outcome is not merely more rules: it is declared change-space coverage, real consumer evidence, empirical compatibility evidence, reproducible IG builds, and deterministic test evidence.

### Wave 2 — Make the engine easy and safe to adopt

Build CF-23 through CF-25. The outcome is one stable deterministic engine exposed through supported developer surfaces, bounded institutional extension, and terminology federation/gap intelligence.

### Wave 3 — Make semantic trust externally inspectable

Build CF-26 through CF-28. The outcome is transformation evidence, public reproducibility, and research/benchmark claims that can be independently reproduced.

## 5. Universal Spec Kit skeleton

Every new CF unit should begin with this minimum structure:

```text
specs/<next>-cf-xx-<slug>/
  spec.md
  plan.md
  tasks.md
  consistency.md
  convergence.md
```

Recommended task stack for new candidate units:

```text
T001-T009  Canonical truth capture, authority, dependencies, non-goals
T010-T019  Public/internal contract freeze and schema/version decisions
T020-T029  Core deterministic implementation
T030-T039  Unit/property/adversarial regression coverage
T040-T049  Corpora/oracles/differential evidence and retained counterexamples
T050-T059  CLI/API/integration surface and bounded I/O
T060-T069  Provenance, evidence bundle, offline/replay, rights/privacy checks
T070-T079  Performance/resource/portability qualification where applicable
T080-T089  Exact-head CI, review, ruleset qualification, convergence/merge
```

A slice may refine task numbering, but should preserve the dependency meaning.

## 6. Global acceptance contract

A future V3 slice is not closed unless all applicable properties are proven:

| Property | Required closure evidence |
| --- | --- |
| Determinism | Same declared inputs/config/tool identities produce byte- or schema-equivalent semantic output. |
| Fail-closed | Unknown version, unsupported transition, malformed evidence, ambiguous authority, and verifier failure cannot become PASS. |
| Provenance | Every conclusion binds exact package/artifact/rule/tool/config identities and source evidence. |
| Offline replay | Declared deterministic path can replay without network after acquisition bundle creation. |
| Resource bounds | Hostile or oversized inputs have explicit limits and classified resource failures. |
| Stable machine contract | Public JSON/API/exit-code/schema behavior is versioned and validated. |
| Version honesty | Support is capability-by-capability; parse support is never presented as semantic support. |
| Rights | Software license and data/terminology redistribution rights are separately dispositioned. |
| Privacy | Core CI stays PHI-free; interaction-derived evidence has explicit capture/redaction/retention rules. |
| Performance honesty | Measured budgets only; no invented threshold or unqualified scale claim. |
| Review provenance | Exact candidate head/base and required status/reviewer provenance are retained. |
| Upgrade behavior | Old/new commandF machine artifacts are accepted, migrated, or rejected according to a documented policy. |

## 7. Gap closure contracts

A gap is **PLANNED** when mapped to a future unit. It becomes **CLOSED** only when the closure contract below is canonical and backed by executable evidence.

| Gap | Owning slice(s) | Closure contract |
| --- | --- | --- |
| G01 | CF-17 | Reproducible immutable registry/feed snapshots and historical package graph exist with provenance and replay. |
| G02 | CF-18 | Every declared change transition is classified, explicitly ignored, version-inapplicable, or fail-closed unsupported. |
| G03 | CF-18 | Compatibility dimensions/algebra are versioned, validated, and regression tested. |
| G04 | CF-19 | Consumer dependencies are parsed/indexed from declared contract inputs rather than inferred from popularity. |
| G05 | CF-19 | `can-i-certify`-style decisions bind explicit protected consumer/version sets and refuse insufficient evidence. |
| G06 | CF-20 | Reproducible differential matrices retain agreement, disagreement, tool identity, and availability state. |
| G07 | CF-17/20/22 | Evidence truth class is mandatory and cannot be upgraded by serialization or report wording. |
| G08 | CF-18 | Artifact × FHIR-version maturity is machine-readable through PARSE/INDEX/DIFF/CLASSIFY/IMPACT/CONSUMER/ORACLE. |
| G09 | CF-18/23 | Capability-by-version support matrix is published through machine and human surfaces. |
| G10 | CF-20/21 | Snapshot/differential conclusions can be compared against pinned Publisher/validator evidence without laundering disagreement. |
| G11 | CF-21 | Source tree + SUSHI + Publisher + package closure can reproduce or explicitly classify output divergence. |
| G12 | CF-21 | Interoperability BOM captures healthcare contract dependencies independently of software SBOM. |
| G13 | CF-18 | Rule lifecycle promotion requires retained positive/negative/real-corpus evidence and migration behavior. |
| G14 | CF-18/23 | Finding schema has explicit versioning, validator, compatibility policy, and deterministic migration/rejection tests. |
| G15 | CF-17 | Incremental cache/graph identity and invalidation are content-addressed and measured before scale claims. |
| G16 | CF-23 | CLI/API/daemon/LSP surfaces consume the same semantic engine and contract tests prove no privileged duplicate logic. |
| G17 | CF-24 | Plugin capability manifest denies ambient authority, pins identity, enforces resource limits, and classifies plugin failure. |
| G18 | CF-15 + CF-19 + CF-22 | Dry-run recipe core is bound to affected consumers and deterministic verification so migration artifacts are executable evidence, not prose. |
| G19 | CF-21 | Portable bundle carries complete allowed inputs/provenance and replays deterministic analysis offline. |
| G20 | CF-23 + AF-03 | Install/update/verify workflows consume AF-03 release evidence and are tested on supported platforms. |
| G21 | CF-27 | Public evidence view serves immutable analyzed identities with reproducibility links and no package-registry authority claim. |
| G22 | CF-17 | Published, CI/draft, extension, terminology, and availability telemetry are partitioned by mutability/truth class. |
| G23 | CF-20/21 | GoFSH/SUSHI/source-map round-trip observations are retained and divergence is explicit. |
| G24 | CF-22 | Every generated assertion traces to exact deterministic evidence; AI output cannot activate authority by itself. |
| G25 | CF-18/28 research lane | Compatibility algebra receives randomized/property/differential evidence; formal work stays research until executed. |
| G26 | all future slices + V3 reconciliation gate | Any adopted source has qualification record, exact identity/pin, rights, trust boundary, owner slice, adoption mode, and immutable lifecycle evidence for any archived/deprecated/superseded claim. |
| G27 | CF-17 | Package dependency closure and canonical-reference closure are reported independently with unresolved witnesses. |
| G28 | CF-17 + donor policy | Source lifecycle/staleness is recorded and refreshed before adoption. |
| G29 | CF-21/25 | Software license and terminology/content rights are separately encoded and redistribution is fail-closed. |
| G30 | CF-18/23 | commandF output/rule/API compatibility policy includes deprecation, migration, and cross-version rejection tests. |
| G31 | CF-18/19/20 | Protocol maturity covers REST operations, SMART App Launch, Bulk Data, subscriptions/eventing, and selected adjacent contracts without pretending they are resource-only changes. |
| G32 | CF-18/23 | Configuration/policy precedence is deterministic, provenance-bearing, schema-versioned, and visible in output evidence. |
| G33 | CF-18/23 | Stable error/partial-result taxonomy prevents parser/oracle/network/resource failure from collapsing into semantic PASS. |
| G34 | CF-19/20 | Observed interaction evidence has explicit synthetic/public qualification path plus capture, redaction, retention, and opt-in rules. |
| G35 | CF-21/23 | Evidence bundle has versioned manifest/schema, validator, compatibility policy, and import/replay migration or rejection behavior. |
| G36 | CF-17/19/23 | Multi-package/workspace/monorepo and branch-range comparison semantics are explicit and deterministic. |
| G37 | CF-27 | Public observatory has correction/retraction/freshness policy while immutable historical evidence remains addressable. |
| G38 | CF-24 | Plugin API/ABI compatibility, revocation, upgrade, signature/key rollover, and unsupported-version behavior are explicit. |
| G39 | CF-20/28 | Runtime concurrency/transaction behavior has reproducible bounded experiments/benchmarks where commandF claims support; static compatibility never implies transaction safety. |
| G40 | CF-19/24 | Identity/consent/authorization dependencies are represented only as declared contracts or bounded policy adapters; commandF does not claim a universal patient-matching or authorization semantics engine. |


## 8. Legacy interoperability-gap reconciliation

The original `COMMAND_F_GAP_LEDGER_2026-08-13.md` contains 35 product/research hypotheses. None may disappear silently. This crosswalk records the V3 disposition; `BOUNDARY/RESEARCH` means the problem is deliberately not claimed solved by the core product.

| Legacy | V3 disposition |
| --- | --- |
| 01 FHIR-valid != semantic equivalence | CF-18 compatibility dimensions + CF-20 empirical evidence + CF-26/28 semantic research. |
| 02 no adopted Semantic Loss metric | CF-26 Loss Ledger + CF-28 research/benchmark; no premature universal metric claim. |
| 03 bidirectional mapping != round-trip safety | CF-26 reversibility evidence + CF-28 round-trip benchmark. |
| 04 profile proliferation burden | CF-17 ecosystem history + CF-18 coverage + CF-19 consumer impact. |
| 05 overlapping/incompatible IG constraints | CF-17 graph/history + CF-18 typed conflict/change coverage. |
| 06 no universal safe profile harmonizer | BOUNDARY/RESEARCH: conflict evidence first; no automatic universal harmonizer claim. |
| 07 extension semantic fragmentation | CF-17 extension drift + CF-18 extension change-space coverage. |
| 08 terminology remains incomplete | CF-25 terminology evidence/federation/gaps. |
| 09 domain/local terminology gaps | CF-25 gap classification + CF-28 measured research where needed. |
| 10 FHIR search varies by implementation | CF-19 query contracts + CF-20 empirical server/engine evidence. |
| 11 CapabilityStatement may differ from behavior | CF-19 declared contract + CF-20 declared-vs-observed evidence. |
| 12 no universal server compatibility matrix | CF-20 differential lab + CF-27 public evidence view. |
| 13 cross-version conversion semantic risk | CF-18 version-aware dimensions + CF-20/28 oracle/benchmark evidence. |
| 14 FHIR conversion not universal cross-model solution | CF-26 evidence; BOUNDARY against universal conversion claims. |
| 15 no universal mapping registry | CF-16 mapping analysis + CF-26 evidence; commandF does not become a universal mapping registry. |
| 16 FHIR-openEHR mappings incomplete | CF-16/26/28 research/evidence path. |
| 17 FHIR-OMOP mappings incomplete | CF-16/26/28 research/evidence path. |
| 18 no accepted neutral clinical IR | BOUNDARY/RESEARCH: only evidence-derived IR after CF-16/26/28 experiments. |
| 19 semantic ambiguity within valid FHIR | CF-18 typed uncertainty/unsupported states + CF-20/26 evidence. |
| 20 inconsistent clinical validation practice | CF-20 conformance evidence + CF-22 deterministic test generation. |
| 21 GraphQL-on-FHIR is secondary/evolving | Retained P2 protocol surface in CF-18/19; no foundation dependency. |
| 22 SQL-on-FHIR evolves | CF-19 ViewDefinition contract scanner + CF-20 differential evidence. |
| 23 no universal cross-model query language | BOUNDARY/RESEARCH under CF-28; no universal query IR prerequisite. |
| 24 eventing/subscription sensitivity | CF-18 protocol maturity + CF-19 consumer contracts + CF-20 conformance evidence. |
| 25 bulk/analytics differs from ordinary REST | CF-18/19/20 Bulk Data protocol lane. |
| 26 FHIR REST is not a general analytics engine | BOUNDARY: commandF analyzes compatibility and does not build an analytics FHIR server. |
| 27 Consent/Permission portability is difficult | CF-19 declared contract evidence + CF-24 bounded policy adapters; no universal policy semantics claim. |
| 28 SMART is not universal policy | CF-18/19 SMART compatibility only; CF-24 organization policy remains separate. |
| 29 no universally correct patient matching | BOUNDARY/RESEARCH: external identity systems/declared dependencies only; commandF does not choose a universal matcher. |
| 30 generic provenance != field-level reasoning | CF-26 field-level transformation evidence graph. |
| 31 no machine-verifiable Transformation Certificate | CF-26 certificate envelope tied to evidence, with integrity != correctness explicitly stated. |
| 32 concurrency/race can break workflows | CF-20 bounded runtime/transaction experiments + CF-28 reproducible concurrency benchmark where claims require it. |
| 33 AI-generated FHIR/mappings are unsafe as authority | CF-22 untrusted-proposal boundary + global deterministic/human authority invariant. |
| 34 AI over large clinical graphs can be inefficient/representation-sensitive | BOUNDARY/RESEARCH: optional retrieval/MCP aids only; CF-28 may benchmark, never semantic authority. |
| 35 no comprehensive semantic-conservation benchmark | CF-28 benchmark/research convergence. |

This crosswalk is preservation evidence, not proof that any legacy gap is already solved.

## 9. Slice build cards

### CF-17 — Ecosystem Observatory

**Goal:** turn public ecosystem history into reproducible evidence without becoming a registry.

**Must ship:** immutable IG-registry/package-feed snapshots; exact raw digests; package/artifact history; separate package vs canonical-reference closure; unresolved canonical witnesses; source lifecycle/mutability partition; content-addressed incremental cache; multi-package/workspace identity primitives needed downstream.

**Non-goals:** hosting packages as a competing registry; using mutable CI as release authority; popularity as compatibility evidence.

**Minimum work packages:**

- freeze snapshot/provenance/right-state schemas;
- ingest official registry/feed sources with exact digest and retrieval evidence;
- resolve historical immutable package releases under existing CF-01 identity rules;
- build package and canonical-reference graphs separately;
- retain unresolved/reference-conflict witnesses;
- implement incremental invalidation keyed by input + engine/schema identity;
- expose deterministic snapshot/list/query machine output;
- qualify replay offline and measure scale only after AF-04.

**Exit evidence:** two independent replays of the same frozen snapshot produce equivalent graph/output identities; mutation tests prove mutable CI cannot become published authority; unresolved canonicals survive serialization.

### CF-18 — Compatibility Coverage Model

**Goal:** prove what commandF knows, what it does not know, and why.

**Must ship:** versioned compatibility dimensions; artifact and protocol maturity matrices; finite declared change space; machine-readable coverage map; fail-closed uncovered transitions; rule lifecycle; finding/config/error schema compatibility policy.

**Protocol surfaces to model explicitly:** ordinary FHIR REST interactions/operations, Search behavior, SMART App Launch declarations/scopes where applicable, Bulk Data contracts, subscriptions/eventing, and selected adjacent FHIR-bound contracts. CDS Hooks/FHIRcast may be retained as bounded adjacent surfaces, not silently promoted to universal core semantics.

**Non-goals:** claiming every possible healthcare protocol is supported; reusing OpenAPI/Protobuf compatibility semantics as FHIR authority.

**Minimum work packages:**

- freeze compatibility and error dimensions/algebra;
- enumerate supported artifact/version and protocol/version state spaces;
- define configuration schema/precedence/provenance;
- map every current rule to covered transition identities;
- create explicit reviewed ignore/non-semantic states;
- fail closed on a newly introduced uncovered model field/transition;
- add rule lifecycle promotion evidence;
- define public finding/schema migration/deprecation policy.

**Exit evidence:** machine coverage report has no silent transition state inside the declared support envelope; injecting a synthetic new field/change form produces uncovered/fail-closed evidence until explicitly classified.

### CF-19 — Consumer Contract Scanner

**Goal:** replace graph-reach/popularity guesses with declared downstream usage evidence.

**Must ship:** parsers/indexers for selected Search/SearchParameter, FHIRPath, CQL/ELM, SQL-on-FHIR ViewDefinition, CapabilityStatement, TestScript/Inferno metadata, REST operation expectations, SMART/Bulk/subscription declarations where applicable; protected consumer/version matrix; workspace/multi-package scope; privacy-safe interaction-contract import format.

**Non-goals:** scraping production PHI; treating downloads or centrality as consumer authority; becoming a general API gateway.

**Minimum work packages:**

- freeze consumer identity/version/deployment schema;
- implement one deterministic contract adapter at a time behind a common evidence interface;
- retain source byte/location/provenance for every edge;
- distinguish static declared contract from optional observed interaction evidence;
- define protected consumer set and insufficient-evidence behavior;
- add deterministic consumer impact output and machine query surface;
- bind CF-15 migration recipes to affected consumer evidence when available.

**Exit evidence:** synthetic consumers with known dependencies produce exact expected impact edges; absence of consumer evidence is reported as unknown/insufficient, never safe.

### CF-20 — Compatibility Lab

**Goal:** measure actual implementation behavior without promoting any one implementation into universal semantics.

**Must ship:** pinned validator/Publisher/FHIRPath/server/test-framework adapters; reproducible differential matrix; declared-vs-observed behavior; agreement/disagreement/unsupported classification; external availability separated from semantic result; privacy-safe interaction test policy.

**Non-goals:** embedding a FHIR server into commandF; rerun-to-green qualification; hiding oracle divergence.

**Minimum work packages:**

- freeze oracle adapter/evidence schema;
- qualify exact tool acquisition and offline replay where licensing allows;
- run official and independent implementations over shared frozen cases;
- retain raw bounded outputs/digests plus normalized comparison evidence;
- classify version/tool unsupported separately from semantic disagreement;
- add FSH GoFSH/SUSHI round-trip observations;
- add selected SMART/Bulk/subscription conformance suites where they are relevant to declared support.

**Exit evidence:** seeded disagreement remains visible end-to-end and cannot be normalized into consensus; transient oracle unavailability cannot alter deterministic retained conclusion.

### CF-21 — Interoperability BOM and Reproducible Build Evidence

**Goal:** make every conclusion portable and independently reproducible.

**Must ship:** versioned Interoperability BOM; source-to-built IG reproducibility record; exact tool/package/terminology/rule/policy/consumer dependencies; versioned evidence-bundle manifest; offline import/replay; software-license vs data/content-rights distinction.

**Non-goals:** overloading SPDX/CycloneDX with semantics they do not define; redistributing restricted terminology content.

**Minimum work packages:**

- freeze BOM + bundle manifest schemas and compatibility policy;
- bind source tree, SUSHI, Publisher, package closure, terminology mode, produced outputs;
- record source-map/round-trip observations;
- export generic standards only for relationships they accurately represent;
- implement bundle validate/import/replay with tamper and unsupported-version rejection;
- add rights disposition and redistribution filter;
- prove network-disabled replay for an allowed fixture closure.

**Exit evidence:** fresh machine reproduces the retained deterministic result from the bundle; one-byte tampering or unsupported manifest schema fails closed.

### CF-22 — Deterministic TestGen

**Goal:** convert requirements/change evidence into executable regression proof without making AI authoritative.

**Must ship:** deterministic test-plan schema; evidence-linked generated cases/TestScript where appropriate; human/deterministic acceptance boundary; minimized retained regression; consumer-aware migration verification hooks.

**Non-goals:** free-form LLM tests as gate authority; generating assertions with no evidence source.

**Minimum work packages:**

- freeze requirement/assertion/evidence-link schema;
- generate deterministic cases from supported CF-18 transition identities;
- optionally import AI-proposed candidates only as untrusted proposals;
- validate generated tests against authoritative/differential evidence;
- minimize failures and retain source linkage;
- verify applicable CF-15 migration recipes against protected consumers.

**Exit evidence:** deleting or changing the source evidence invalidates/rejects the generated authority; identical evidence/config produces stable test identity.

### CF-23 — Developer Platform

**Goal:** expose one engine through high-quality, stable developer workflows.

**Must ship:** stable machine API; local daemon/cache protocol; pre-commit/pre-push flow; LSP/VS Code diagnostics/navigation; install/update/verify experience using AF-03 evidence; optional read-only MCP adapter; explicit configuration precedence; stable partial-result/error contract; workspace/monorepo behavior.

**Non-goals:** semantic logic duplicated in IDE/UI/MCP; hidden cloud requirement.

**Minimum work packages:**

- freeze API/schema/exit-code/config precedence and deprecation policy;
- create contract tests shared by CLI/library/daemon/LSP surfaces;
- implement content-addressed local daemon cache with authority-safe invalidation;
- add workspace/multi-package and branch-range commands;
- map deterministic finding/source evidence into LSP diagnostics;
- add verified installation/update/rollback/verification paths;
- keep MCP read-only and typed if shipped.

**Exit evidence:** same fixture/config yields equivalent semantic findings through all supported surfaces; unsupported schema/config and daemon cache mismatch fail closed.

### CF-24 — Capability-Scoped Plugin and Policy SDK

**Goal:** support institutional policy without giving extensions ambient authority over core semantics.

**Must ship:** versioned plugin interface; signed/pinned identity; explicit capabilities; no ambient network/filesystem; time/memory/fuel/output limits; deterministic host functions; organization-policy adapter boundary; plugin lifecycle/revocation/key-rollover policy.

**Non-goals:** arbitrary Python/JavaScript execution; plugins overriding core healthcare semantic truth silently.

**Minimum work packages:**

- freeze capability and plugin ABI/API version contract before selecting runtime;
- evaluate runtime only against required sandbox properties;
- implement signature/digest/provenance verification;
- deny undeclared capabilities and nondeterministic sources by default;
- classify timeout/resource/trap/schema/signature failures distinctly;
- test compatibility, revocation, unsupported version, and key rollover;
- prove plugin result cannot erase core findings.

**Exit evidence:** malicious fixture cannot access undeclared network/filesystem/clock/randomness and its failure remains visible in the final evidence.

### CF-25 — Terminology Gap and Federation Intelligence

**Goal:** explain terminology compatibility and gaps without becoming a terminology server.

**Must ship:** terminology endpoint/package/version capability evidence; version-aware lookup/validate/expand/translate observations where supported; federation adapter contract; gap classification; content-rights evidence.

**Non-goals:** redistributing restricted content without rights; assuming a server license grants terminology-content rights.

**Minimum work packages:**

- freeze terminology identity/version/capability/gap schemas;
- implement bounded adapters to qualified independent services/packages;
- distinguish unavailable, unsupported, unauthorized-content, and semantic disagreement;
- retain exact endpoint/tool/package/content-version provenance;
- support offline package-backed paths only when rights permit;
- feed terminology dimensions back to CF-18/19/20 evidence.

**Exit evidence:** same code system with different declared versions cannot be conflated; restricted content cannot enter redistributable bundle without explicit permission.

### CF-26 — Transformation Evidence and Certificates

**Goal:** make cross-model transformations explainable and evidence-bearing.

**Must ship:** Loss Ledger; transformation evidence graph; exact mapping/tool/source/target provenance; reversible/irreversible classification; certificate envelope/signature integration; explicit statement that signature proves integrity/provenance, not semantic correctness.

**Non-goals:** universal clinical IR claim without executed multi-dialect evidence.

**Minimum work packages:**

- consume CF-16 parse-only mapping evidence without retroactively broadening CF-16 authority;
- freeze loss/reversibility/evidence schemas;
- execute selected mapping dialects through qualified adapters;
- retain field-level witness paths and counterexamples;
- create reproducible certificate envelope tied to CF-21 bundle identities;
- integrate CF-25 terminology evidence where mappings depend on coded meaning.

**Exit evidence:** seeded information loss is preserved in the certificate/evidence graph and cannot be hidden by successful target validation.

### CF-27 — Public Compatibility Observatory

**Goal:** make commandF's evidence moat inspectable without publishing false authority.

**Must ship:** public view/API over immutable analyzed release identities; compatibility/rule-coverage/drift history; reproducibility links; freshness/correction/retraction policy; privacy and rights review; mutable telemetry visibly separated.

**Non-goals:** hosting private consumer contracts; package registry replacement; rewriting historical evidence when a conclusion is corrected.

**Minimum work packages:**

- freeze public record and correction/retraction schemas;
- publish only rights-cleared immutable source/result identities;
- expose reproduction bundle/digest where redistribution permits;
- distinguish new analysis version from historical source identity;
- preserve superseded conclusions with reason/evidence rather than deleting history;
- define freshness and source-unavailable states.

**Exit evidence:** corrected analysis creates an auditable superseding record while original immutable evidence remains addressable and clearly non-current.

### CF-28 — Bench and Research Convergence

**Goal:** turn product claims and research hypotheses into reproducible comparative evidence.

**Must ship:** benchmark packs for supported diff/compatibility, consumer impact, terminology, mapping, round-trip, and semantic-conservation questions; fixed methodology; baseline versions; resource measurements; null/negative result retention.

**Non-goals:** leaderboard claims from mutable inputs or incomparable configurations.

**Minimum work packages:**

- freeze benchmark manifest/result schema and comparability requirements;
- select rights-cleared frozen corpora by evidence class;
- publish baseline tool/config/hardware/runtime identities where relevant;
- measure accuracy/coverage/divergence plus resources under AF-04 rules;
- preserve failures and null results;
- connect research hypotheses only to executed benchmark evidence.

**Exit evidence:** an independent runner can reproduce the benchmark pack and determine whether two results are comparable before comparing scores.

## 10. Protocol and adjacent-standard scope rule

commandF should understand protocol contracts only where they materially affect FHIR interoperability change risk. Candidate support order:

1. core FHIR REST interactions and operations;
2. Search/SearchParameter behavior;
3. SMART App Launch and backend-service declarations/scopes;
4. FHIR Bulk Data export contracts;
5. FHIR Subscriptions/eventing;
6. bounded FHIRcast/CDS Hooks contracts when an owned consumer or IG declares them;
7. GraphQL-on-FHIR remains secondary/evolving unless ecosystem evidence raises priority.

This is a maturity queue, not a claim that all items are equally normative or immediately implemented.

## 11. Configuration contract

All supported surfaces must converge on one configuration model. Precedence must be explicit and deterministic, for example:

```text
compiled defaults < repository config < workspace/package override < explicit CLI/API request
```

The exact precedence is frozen in the owning Spec Kit. Environment variables may supply secrets/locations where necessary but must not silently mutate semantic policy. Effective configuration identity and non-secret semantic fields are retained in evidence.

## 12. Error and partial-result contract

Future APIs must distinguish at least:

- invalid input;
- unsupported artifact/version/protocol;
- unclassified change;
- policy/configuration error;
- oracle unavailable;
- oracle unsupported;
- oracle disagreement;
- resource/limit failure;
- rights/privacy denial;
- plugin failure;
- internal invariant failure.

A partial report must state exactly which stages completed and which did not. Missing evidence is never equivalent to no impact.

## 13. Workspace and comparison semantics

CF-17/19/23 planning must support real repositories without making branch names semantic authority:

- one or many IG/package roots in a workspace;
- explicit package identities derived from built/frozen artifacts;
- branch/range/PR comparison resolved to exact commits plus package evidence;
- stacked changes without double-counting inherited deltas;
- generated/vendor/cache directories excluded only by explicit configuration;
- no host-local path in portable machine evidence unless sanitized to a stable source identifier.

## 14. Privacy-safe interaction evidence

Observed HTTP interaction contracts are optional evidence. Before any production-derived capture is supported, CF-19/20 must freeze:

- opt-in boundary and allowed environments;
- minimization fields and purpose;
- deterministic redaction/tokenization strategy;
- retention and deletion policy;
- secret/header/body handling;
- proof that synthetic/public fixtures cover the semantic code path;
- explicit statement that captured traffic is evidence, not universal contract authority.

Core repository CI remains synthetic/public and PHI-free.

## 15. Source qualification rules for new protocol coverage

Before adoption, protocol-oriented sources should be classified like every other donor/oracle. Current high-value candidates include:

- `HL7/smart-app-launch` — official SMART App Launch specification source;
- `HL7/bulk-data` — official FHIR Bulk Data Access IG source;
- `HL7/fhir-subscription-backport-ig` — official R4/R4B subscription backport reference;
- `HL7/fhircast-docs` — official FHIRcast specification source for bounded adjacent event contracts;
- `smart-on-fhir/client-js` — independent/industry client behavior reference, not normative authority;
- `inferno-framework/smart-app-launch-test-kit` — executable conformance prior art/oracle input.

Pins, exact publication/version identities, rights, and adoption modes are decided only by the owning slice.

## 16. Agent operating loop

For every future implementation unit:

```text
1. REVERIFY
   canonical main + active frontier + rulesets + PRs + Spec Kit
2. FREEZE
   exact scope, non-goals, schemas, dependencies, donors, evidence classes
3. BUILD
   smallest dependency-ordered task stack; no borrowed future behavior
4. BREAK
   negative/property/adversarial/differential tests; retain counterexamples
5. PROVE
   deterministic replay, provenance, offline/resource/privacy/rights checks
6. QUALIFY
   exact-head CI + required reviewers + ruleset/status provenance
7. MERGE
   guarded canonical merge only when head/base/authority still match
8. RE-READ
   canonical main/tree, post-merge evidence, next eligible dependency
```

Do not ask the founder for routine implementation choices already frozen by canonical authority. Escalate only genuinely unresolved product/semantic authority decisions, rights conflicts, privacy-scope expansion, or irreversible governance changes.

## 17. Product-level definition of done

commandF may make a defensible world-class claim only when all applicable statements are backed by public/reproducible evidence:

- the declared artifact/protocol/version change space has machine-verifiable coverage;
- unsupported/unclassified states are explicit and fail closed;
- consumer impact can bind real declared consumer/version contracts;
- differential implementation disagreements are retained, not averaged away;
- source IG builds can distinguish source intent from tool/build drift;
- Interoperability BOM/evidence bundle can reproduce deterministic results offline;
- generated tests and migration recipes trace to exact evidence;
- every supported developer surface shares one semantic contract;
- institutional extensions are capability-scoped and cannot erase core truth;
- terminology rights/version identity are explicit;
- public observatory data is correctable without rewriting history;
- benchmark claims are independently reproducible and preserve negative results;
- no patient data, AI output, popularity metric, mutable upstream branch, or vector index is required as semantic authority.
