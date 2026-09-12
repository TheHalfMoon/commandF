# commandF Whole-Plan Gap Review — 2026-09-12

Status: **PLANNING_CANDIDATE / GAP ANALYSIS**

This review evaluates the canonical V2 plan, product family, gap ledger, discovery annex, research charter, Assurance Program, existing CF-01..CF-13 implementation history, and currently active AF-02 work. It does not change current implementation authority.

Canonical base reviewed:

```text
main: c56d9619758ea3e1075e751bec83ddea6706a81e
tree: 8c0cdef02a1169c9e815657397b3dcc9828c53bf
```

## Executive assessment

commandF is already unusually strong in deterministic identity, fail-closed behavior, oracle separation, source fidelity, graph/impact thinking, and repository assurance. The main planning weakness is not missing concepts; the discovery annex contains many of them. The weakness is that several potential moats remain aspirational instead of becoming independently measurable execution units.

The highest-value correction is to keep the current narrow spine while adding explicit post-CF-16 units for ecosystem observation, change-space coverage, consumer contracts, compatibility labs, reproducible interoperability bills of materials, deterministic TestGen, developer APIs, bounded plugins, and terminology federation.

Severity meaning:

- **P0** — blocks a defensible "best-in-class FHIR change intelligence" claim.
- **P1** — materially limits adoption, scale, or trust but does not invalidate current functionality.
- **P2** — important later differentiation or research maturity.

## Gap ledger

| ID | Sev | Gap | Current plan signal | Risk if unchanged | V3 response |
| --- | --- | --- | --- | --- | --- |
| G01 | P0 | No ecosystem observatory execution unit | Standards/vendor drift and public IG corpus are retained concepts | commandF cannot learn systematically from historical ecosystem change | CF-17 immutable registry/feed snapshots + historical package/artifact graph |
| G02 | P0 | No machine proof of compatibility-rule change-space coverage | CF-04 has rules and tests | new FHIR fields/change forms can be silently outside the rule catalog | CF-18 enumerated change-space coverage with fail-closed uncovered states |
| G03 | P0 | Compatibility classification is not yet a versioned multidimensional lattice | producer/consumer direction exists; broader dimensions are scattered | one severity can hide producer/consumer/terminology/query/runtime differences | CF-18 compatibility dimensions and algebra |
| G04 | P0 | Consumer Compatibility Matrix is aspirational, not backed by a concrete contract scanner | Context Graph/impact exist | blast radius can show upstream graph reach without knowing actual downstream usage | CF-19 consumer contract scanner |
| G05 | P0 | `can-i-certify` lacks protected-consumer/version semantics | retained product idea | a green answer could become ambiguous or policy-driven without evidence | CF-19 explicit protected deployment/version matrix |
| G06 | P0 | Compatibility Lab lacks an executable slice | many validators/servers are retained donors | declared compatibility may diverge from actual validator/server/client behavior | CF-20 differential and empirical compatibility lab |
| G07 | P0 | Corpora have different epistemic purposes but no plan-level truth-class taxonomy | CF-10 real corpus and AF-02 adversarial corpus exist separately | evidence can be overinterpreted or mixed | V3 evidence truth classes; CF-17/20/22 preserve class |
| G08 | P0 | No artifact-type maturity matrix | profile/terminology features were built slice-by-slice | "FHIR support" can be interpreted more broadly than actually proven | CF-18 artifact × version maturity matrix |
| G09 | P0 | Version support is not capability-by-capability | R4 strongest; R4B/R5/R6 readiness noted | parsing a version can be mistaken for compatibility support | explicit support matrix; R6 preview only while draft |
| G10 | P0 | Snapshot/differential semantics need explicit oracle governance | Publisher/validator are oracles | custom normalization can drift from Publisher semantics | CF-20/21 differential Publisher evidence and reproducible build evidence |
| G11 | P0 | Source IG reproducibility is not a first-class product proof | FSH source mapping exists | a package change can be caused by SUSHI/Publisher/tool drift rather than source intent | CF-21 reproducible source-to-package build evidence |
| G12 | P0 | No Interoperability BOM distinct from software SBOM | AF-03 plans software SBOM/provenance | analysis cannot package exact healthcare contract dependencies as one portable proof input | CF-21 Interoperability BOM |
| G13 | P0 | Rule governance has no explicit evidence-driven lifecycle | versioned rules + baselines exist | experimental rules and mature default gates can look equivalent | CF-18 lifecycle and promotion evidence |
| G14 | P0 | Finding/explanation schema needs a long-lived machine compatibility contract | SARIF/CLI/fingerprints exist | downstream automation may break or explanations may become non-reproducible | CF-18 stable finding schema + migration policy |
| G15 | P1 | No scaling design for incremental ecosystem analysis | AF-04 measures performance, graph exists | full rebuilds and cache ambiguity can block observatory scale | content-addressed incremental cache/graph identity before CF-17 scale claims |
| G16 | P1 | CLI + Action are insufficient as the only durable integration surfaces | Product family includes Studio/Copilot | IDEs, internal platforms, and CI systems may reimplement wrappers inconsistently | CF-23 stable API/daemon/LSP/pre-commit/MCP wrapper |
| G17 | P1 | No explicit plugin/rule SDK trust model | Wasmtime/CEL/Rego are retained candidates | arbitrary scripts could undermine deterministic core or create unsafe extension pressure | CF-24 signed/pinned capability-scoped Wasm SDK |
| G18 | P1 | Migration artifact is secondary to findings | Verified AutoFix retained | teams still need manually assembled rollout plans even when a breaking change is understood | make migration notes/recipes first-class outputs in review plane / CF-15 evolution |
| G19 | P1 | Offline/air-gapped operation is assurance-focused, not product-bundled | resource runner is offline; local cache exists | enterprise users can reproduce CI but not necessarily move complete analysis inputs across boundaries | CF-21 portable input/evidence bundle |
| G20 | P1 | Release/adoption UX is outside the product roadmap | AF-03 verifies releases | trustworthy artifacts may exist without easy install/update/verify workflows | CF-23 distribution UX consuming AF-03 evidence |
| G21 | P1 | No public evidence database/site execution unit | public Compatibility Matrix/observatory ideas retained | commandF's strongest data moat stays private/local and hard to inspect | CF-27 public compatibility observatory |
| G22 | P1 | Drift monitoring does not yet distinguish published releases, mutable CI, extensions, terminology, and draft standards | drift retained as concept | mutable development signals can be mistaken for stable standards changes | CF-17 source-class and mutability partition |
| G23 | P1 | FSH source fidelity does not yet exploit independent round-trip evidence | CF-09 maps FSH source | source location can be accurate while regenerated semantics drift | GoFSH/SUSHI round-trip observations under CF-20/21 |
| G24 | P1 | TestGen has no concrete deterministic acceptance pipeline | TestGen is retained product idea | AI-generated tests could become opaque or ungrounded | CF-22 source-evidence -> generated test -> deterministic acceptance chain |
| G25 | P2 | Compatibility algebra has no formal-methods research lane | property/fuzz assurance exists | subtle direction/set/order algebra bugs may remain difficult to reason about | research-only formal semantics + differential randomized testing inspired by verified policy engines |
| G26 | P0 | Open-source candidates lack one uniform adoption qualification schema | donor policy exists; discovery list is broad | stale, archived, high-risk, or semantically dependent projects can enter by enthusiasm | new open-source qualification matrix and adoption modes |
| G27 | P0 | Package dependency closure and canonical-reference closure are not explicitly separate product guarantees | CF-11/11G graph exists | a package graph can be complete while canonical targets are unresolved | CF-17 unresolved canonical inventory and witness paths |
| G28 | P1 | Donor lifecycle/staleness is not automatically visible | discovery preserves sources | archived/deprecated upstreams can remain listed as if current | qualification records maintenance/deprecation state and requires refresh before adoption |
| G29 | P0 | Software license and terminology/data-content rights need a stronger cross-plan gate | existing docs state separation | open-source terminology server can be legal while the terminology content is not redistributable | CF-21/25 rights-aware BOM/evidence and explicit content-license disposition |
| G30 | P0 | commandF's own output/rule compatibility needs a public migration policy | fingerprints and schemas exist piecemeal | downstream automation can lose trust across commandF upgrades | CF-18/23 schema compatibility and deprecation policy |

## Additional gaps discovered during open-source comparison

### A. Change-space completeness versus rule count

`oasdiff` exposes a `checks changelog coverage` capability that maps possible schema edits to checks. The important lesson is not OpenAPI semantics; it is the discipline of proving coverage of a defined change universe. commandF should own an equivalent healthcare-specific mechanism rather than treating an expanding rule list as proof of completeness.

### B. Compatibility needs context-sensitive dimensions

Buf demonstrates that compatibility is policy-relative and can be evaluated under different constraints rather than one universal "breaking" flag. commandF already has producer/consumer direction; the next step is to make healthcare-specific dimensions explicit and machine versioned.

### C. Public registries are discovery authority, not semantic authority

The official FHIR IG registry and package-feed metadata provide a strong starting point for reproducible ecosystem discovery. They should answer "what exists and where was it published?" They should not answer "is it compatible?" without commandF analysis.

### D. Shared official test cases deserve their own evidence class

`FHIR/fhir-test-cases` and the HL7 testing work provide reusable test semantics across implementations. They are materially different from commandF's real-world IG delta corpus and from AF-02's adversarial corpus. The plan should preserve all three without mixing their conclusions.

### E. Real consumer contracts are more valuable than popularity proxies

Pact's deployment matrix is useful prior art because it ties compatibility to known consumer/provider versions. commandF should translate the idea into healthcare-specific protected consumer contracts, not infer safety from download counts, registry presence, or graph centrality.

### F. Supply-chain graph patterns generalize to interoperability evidence

GUAC shows how heterogeneous evidence can be normalized into a high-fidelity relationship graph. commandF should borrow the evidence-graph discipline while keeping healthcare identifiers and semantics native rather than reusing software-vulnerability schemas incorrectly.

### G. A plugin framework needs explicit denied capabilities

Extism/Wasmtime are attractive because WebAssembly provides an isolation seam, but the important commandF requirement is the capability manifest: no ambient network/filesystem, explicit host functions, resource limits, pinned module identity, and deterministic error classification. The runtime implementation is replaceable.

## Roadmap response matrix

| Candidate slice | Primary gaps closed |
| --- | --- |
| CF-17 Ecosystem Observatory | G01, G15, G22, G27, G28 |
| CF-18 Compatibility Coverage Model | G02, G03, G08, G09, G13, G14, G30 |
| CF-19 Consumer Contract Scanner | G04, G05 |
| CF-20 Compatibility Lab | G06, G07, G10, G23 |
| CF-21 Interoperability BOM + Reproducible Build | G11, G12, G19, G29 |
| CF-22 Deterministic TestGen | G24 plus evidence-to-test traceability |
| CF-23 Developer Platform | G16, G20, G30 |
| CF-24 Plugin and Policy SDK | G17 |
| CF-25 Terminology Gap/Federation Intelligence | G29 plus terminology gaps already retained |
| CF-26 Transformation Evidence/Certificates | transformation/loss/provenance moat |
| CF-27 Public Compatibility Observatory | G21 plus standards/vendor drift visibility |
| CF-28 Bench and Research Convergence | G25 and objective comparative evidence |

## Things the plan should explicitly *not* add now

Deep research also produced tempting technologies that should **not** become immediate core dependencies:

- graph databases before the existing embedded model is measured insufficient;
- vector databases as truth stores;
- arbitrary Python/JavaScript rule execution;
- a custom FHIR server or terminology server;
- an MCP-first architecture;
- an LLM-controlled semantic gate;
- a universal clinical IR before cross-dialect experiments justify it;
- a new package registry competing with FHIR infrastructure;
- a new software package identifier type pretending FHIR is already standardized in ecosystems where it is not;
- mutable upstream `latest`, CI builds, or current development branches as reproducible authority.

## Priority recommendation

Do not broaden the runtime while AF-02/03/04 and CF-14..CF-16 are still governed work. Canonicalize this planning overlay independently, then execute the new roadmap in three waves:

```text
Wave 1: CF-17 -> CF-18 -> CF-19 -> CF-20 -> CF-21 -> CF-22
Wave 2: CF-23 -> CF-24 -> CF-25
Wave 3: CF-26 -> CF-27 -> CF-28
```

Dependencies can allow selected planning work in parallel, but no slice should borrow future evidence to close itself.

## Definition of plan success

This gap review is successful only if future work can answer these questions mechanically:

1. Which artifact/version/change forms are supported?
2. Which are unsupported, and does the tool fail closed?
3. Which downstream consumer contracts are affected?
4. Which exact evidence and oracle identities support the conclusion?
5. Can another machine reproduce it offline?
6. Can commandF distinguish source intent, build-tool drift, and runtime divergence?
7. Can the rule catalog prove its coverage over the declared change space?
8. Can a downstream automation upgrade commandF without silently changing finding semantics?
9. Can public benchmarks reproduce both successes and failures?
10. Can all of this be done without treating patient data, AI output, or a vector index as semantic authority?
