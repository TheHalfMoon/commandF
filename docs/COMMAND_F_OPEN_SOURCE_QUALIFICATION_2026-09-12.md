# commandF Open-Source Qualification Review — 2026-09-12

Status: **PLANNING_CANDIDATE / SOURCE QUALIFICATION INVENTORY**

This document is a research and adoption filter. It does **not** authorize copying, vendoring, dependency installation, executable invocation, redistribution, or data ingestion by itself.

Before actual adoption, commandF's existing donor/provenance policy still requires the exact commit/release/digest, relevant paths, license text, notices, source identity, data/content rights, and the concrete CF/AF unit that needs the source.

## Adoption vocabulary

### Authority class

- `NORMATIVE` — standard/specification or official publication authority.
- `OFFICIAL_REFERENCE` — official/community-maintained reference implementation, registry, test corpus, or build tool.
- `AUTHORITATIVE_PROCESS_ORACLE` — process whose behavior commandF intentionally treats as authoritative for a bounded judgment.
- `INDEPENDENT_ORACLE` — independent implementation useful for differential evidence, never hidden semantic authority.
- `PATTERN` — architecture/workflow prior art only.
- `RESEARCH` — experimental/low-maturity input that may inform a study.
- `DEPRECATED_OR_ARCHIVED` — retain for historical context, do not start a new dependency without exceptional justification.

### Adoption mode

- `PINNED_DATA` — consume exact immutable data/spec/corpus bytes.
- `PINNED_PROCESS_ORACLE` — execute exact pinned process in an explicit process boundary.
- `DIFFERENTIAL_ONLY` — compare behavior without delegating commandF semantics to it.
- `OPTIONAL_TOOLCHAIN` — optional build/release/developer tool with pinned identity.
- `PATTERN_ONLY` — study design and reimplement commandF-native semantics independently.
- `RESEARCH_ONLY` — experiment only; no production dependency.
- `REJECT` — not suitable for the intended role.

### Trust boundary tags

- `DATA_ONLY` — parsed as untrusted data; never executed.
- `PROCESS` — spawns or invokes external executable/runtime.
- `NETWORK` — normally requires external network interaction.
- `UNTRUSTED_CODE` — may execute user/plugin/project code and needs stronger sandbox policy.
- `CONTENT_RIGHTS` — software may be open source while content has separate licensing restrictions.

## Qualification rule

A candidate is accepted into a future Spec Kit only if all applicable questions have evidence:

1. Is the source normative, official, independent, or merely inspirational?
2. Is the exact version immutable and reproducible?
3. Is the source still maintained, deprecated, or archived?
4. What software license applies, and what separate data/content rights apply?
5. Does consuming it execute untrusted code or cross a network boundary?
6. Can it run offline with pinned inputs where deterministic evidence requires that?
7. Which FHIR/standard versions and artifact types does it actually cover?
8. Is it sufficiently independent to be useful as a differential oracle?
9. Does it add unique value relative to existing commandF authority?
10. Which CF/AF slice owns the dependency and its removal/upgrade path?

## Tier A — official FHIR discovery, specification, tests, and build evidence

| Source | Authority | Recommended mode | Target | Qualification note |
| --- | --- | --- | --- | --- |
| [HL7 FHIR specification/publication directory](https://hl7.org/fhir/) | NORMATIVE | PINNED_DATA | all FHIR slices | Pin published version/package identities. Mutable CI build is telemetry only. |
| [FHIR/ig-registry](https://github.com/FHIR/ig-registry) | OFFICIAL_REFERENCE | PINNED_DATA | CF-17 | Primary ecosystem discovery seed for published IG metadata. Snapshot exact commits/files. |
| `FHIR/ig-registry` `package-feeds.json` | OFFICIAL_REFERENCE | PINNED_DATA | CF-17 | Discover package feed endpoints; feed presence is not compatibility evidence. |
| [FHIR/fhir-test-cases](https://github.com/FHIR/fhir-test-cases) | OFFICIAL_REFERENCE | PINNED_DATA | CF-18/20/22 | First-class reference corpus distinct from real-IG and adversarial corpora. Pin releases and verify per-artifact rights. |
| [HL7/fhir-testing-ig](https://github.com/HL7/fhir-testing-ig) | OFFICIAL_REFERENCE | PINNED_DATA | CF-20/22 | Track emerging official FHIR testing semantics. Do not assume every draft page is stable authority. |
| [FHIR/fhir-package-loader](https://github.com/FHIR/fhir-package-loader) | OFFICIAL_REFERENCE | DIFFERENTIAL_ONLY | CF-01/17 | Compare package/cache/fallback behavior; commandF keeps its own deterministic resolver authority. |
| [HL7/fhir-ig-publisher](https://github.com/HL7/fhir-ig-publisher) | AUTHORITATIVE_PROCESS_ORACLE | PINNED_PROCESS_ORACLE | CF-20/21 | Preserve JVM process boundary. Pin exact publisher + package closure; prefer `-no-network` where viable. |
| [FHIR/auto-ig-builder](https://github.com/FHIR/auto-ig-builder) | OFFICIAL_REFERENCE | PATTERN_ONLY | CF-17/21 | Build logs/current output are mutable telemetry, never immutable semantic authority. |
| [FHIR/packages](https://github.com/FHIR/packages) | OFFICIAL_REFERENCE | PINNED_DATA | CF-17/20 | Useful core/support/cross-version package source. Pin exact artifacts. |
| [HL7/fhir-extensions](https://github.com/HL7/fhir-extensions) | OFFICIAL_REFERENCE | PINNED_DATA | CF-17/18 | Treat separately from core because extension release cadence and drift differ. |
| [HL7/fhir-shorthand](https://github.com/HL7/fhir-shorthand) | NORMATIVE | PINNED_DATA | CF-09/21 | Official FHIR Shorthand specification source; pin a published/spec revision separately from SUSHI implementation behavior. |
| [FHIR/sushi](https://github.com/FHIR/sushi) | OFFICIAL_REFERENCE | PINNED_PROCESS_ORACLE | CF-09/21 | FSH compilation authority for its bounded role, not general FHIR validation authority. |
| [HL7/smart-app-launch](https://github.com/HL7/smart-app-launch) | NORMATIVE | PINNED_DATA | CF-18/19/20 | Official SMART App Launch specification source. Pin published IG/version separately from client implementation behavior. |
| [HL7/bulk-data](https://github.com/HL7/bulk-data) | NORMATIVE | PINNED_DATA | CF-18/19/20 | Official FHIR Bulk Data Access IG source for protocol/change-space and conformance evidence. |
| [HL7/fhir-subscription-backport-ig](https://github.com/HL7/fhir-subscription-backport-ig) | OFFICIAL_REFERENCE | PINNED_DATA | CF-18/19/20 | Official R5-subscription-backport IG for pre-R5 deployments; keep version-specific semantics explicit. |
| [HL7/fhircast-docs](https://github.com/HL7/fhircast-docs) | NORMATIVE | PINNED_DATA | CF-18/19/20 | Official FHIRcast specification source for bounded adjacent event-contract analysis when an owned consumer declares it. |
| [FHIR/GoFSH](https://github.com/FHIR/GoFSH) | OFFICIAL_REFERENCE | DIFFERENTIAL_ONLY | CF-09/20/21 | Strong source-fidelity and round-trip donor; retain round-trip divergence instead of laundering it. |
| [FHIR/vscode-fsh](https://github.com/FHIR/vscode-fsh) | OFFICIAL_REFERENCE | PATTERN_ONLY | CF-23 | IDE/source navigation UX donor. Do not duplicate semantic authority in extension code. |
| [HL7/FHIRPath](https://github.com/HL7/FHIRPath) | NORMATIVE | PINNED_DATA | CF-19/20 | Specification input for expression semantics. |
| [HL7/fhirpath.js](https://github.com/HL7/fhirpath.js) | OFFICIAL_REFERENCE | DIFFERENTIAL_ONLY | CF-20 | Independent implementation comparison; implementation status is not normative semantics. |
| [HL7/sql-on-fhir](https://github.com/HL7/sql-on-fhir) | NORMATIVE | PINNED_DATA | CF-19 | Parse supported ViewDefinition contract semantics from published spec. |
| [FHIR/sql-on-fhir.js](https://github.com/FHIR/sql-on-fhir.js) | OFFICIAL_REFERENCE | DIFFERENTIAL_ONLY | CF-19/20 | Shared implementation/tests useful for ViewDefinition differential evidence. |

## Tier B — validators, servers, SDKs, and differential engines

| Source | Authority | Recommended mode | Target | Qualification note |
| --- | --- | --- | --- | --- |
| [hapifhir/org.hl7.fhir.core](https://github.com/hapifhir/org.hl7.fhir.core) | AUTHORITATIVE_PROCESS_ORACLE | PINNED_PROCESS_ORACLE | CF-06/20 | Existing Tier-1 validator/core boundary remains intentionally external. |
| [hapifhir/org.hl7.fhir.validator-wrapper](https://github.com/hapifhir/org.hl7.fhir.validator-wrapper) | OFFICIAL_REFERENCE | PATTERN_ONLY | CF-20 | Process/API wrapper patterns; commandF should retain its own pin/acquisition/evidence policy. |
| [hapifhir/hapi-fhir](https://github.com/hapifhir/hapi-fhir) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-20/25 | Parser/validation/FHIRPath/terminology/server behavior comparison. |
| [FirelyTeam/firely-net-sdk](https://github.com/FirelyTeam/firely-net-sdk) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-20 | Important cross-language validator/FHIRPath/snapshot comparison. |
| [FHIR/fhir-candle](https://github.com/FHIR/fhir-candle) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-20 | Multi-version server behavior target. |
| [samply/blaze](https://github.com/samply/blaze) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-20 | FHIR server/CQL/terminology behavior target. |
| [LinuxForHealth/FHIR](https://github.com/LinuxForHealth/FHIR) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-20 | Independent server/validator behavior; pin exact release/commit. |
| [medplum/medplum](https://github.com/medplum/medplum) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-20 | Real FHIR-native integration/runtime behavior reference. |
| [brianpos/fhirpath-lab](https://github.com/brianpos/fhirpath-lab) | PATTERN | PATTERN_ONLY | CF-20 | Strong multi-engine comparison UX/methodology donor. |
| [HeliosSoftware/hfs](https://github.com/HeliosSoftware/hfs) | RESEARCH | DIFFERENTIAL_ONLY | CF-20 | Rust FHIR/FHIRPath experimental oracle; record partial-function/version limitations. |
| [fhir-rust/fhir-rust](https://github.com/fhir-rust/fhir-rust) | RESEARCH | RESEARCH_ONLY | CF-20/23 | Architecture/differential research only until maturity and coverage justify more. |
| [lschmierer/fhirbolt](https://github.com/lschmierer/fhirbolt) | RESEARCH | RESEARCH_ONLY | CF-20 | Lightweight Rust parser differential input, not production authority. |

## Tier C — compatibility, contract, and test-generation prior art

| Source | Authority | Recommended mode | Target | Qualification note |
| --- | --- | --- | --- | --- |
| [oasdiff/oasdiff](https://github.com/oasdiff/oasdiff) | PATTERN | PATTERN_ONLY | CF-18 | Adopt the *coverage discipline*: enumerate possible change types and their check coverage; do not transplant OpenAPI semantics. |
| [bufbuild/buf](https://github.com/bufbuild/buf) | PATTERN | PATTERN_ONLY | CF-18 | Compatibility categories/policy maturity donor; commandF needs healthcare-native dimensions. |
| [pact-foundation/pact_broker](https://github.com/pact-foundation/pact_broker) | PATTERN | PATTERN_ONLY | CF-19 | Consumer/provider version matrix and `can-i-deploy` concept donor; independently specify healthcare semantics. |
| [microcks/microcks](https://github.com/microcks/microcks) | PATTERN | PATTERN_ONLY | CF-20/22 | Multi-protocol mock/conformance workflow donor. |
| [AsyncAPI/diff](https://github.com/asyncapi/diff) | PATTERN | PATTERN_ONLY | future eventing | Breaking/non-breaking/unclassified event-contract diff donor for later FHIR subscriptions/eventing. |
| [fhir-crucible/testscript-generator](https://github.com/fhir-crucible/testscript-generator) | PATTERN | PATTERN_ONLY | CF-22 | Deterministic TestScript generation donor. Validate output independently. |
| [inferno-framework/inferno-core](https://github.com/inferno-framework/inferno-core) | PATTERN | PATTERN_ONLY | CF-20/22 | Current open-source conformance-test framework architecture and test-kit execution patterns; Inferno DSL is not FHIR TestScript authority. |
| [inferno-framework/client-fhir-testing](https://github.com/inferno-framework/client-fhir-testing) | PATTERN | PATTERN_ONLY | CF-19/20/22 | Recorded transaction -> validation/replay model; carefully separate recorded data/privacy boundaries. |
| [inferno-framework/smart-app-launch-test-kit](https://github.com/inferno-framework/smart-app-launch-test-kit) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-20 | Executable SMART conformance evidence across supported published versions; pin test-kit and target IG versions. |
| [smart-on-fhir/client-js](https://github.com/smart-on-fhir/client-js) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-19/20 | Real SMART client behavior reference; never substitute implementation behavior for the SMART specification. |
| [onc-healthit/ONCLAIVE](https://github.com/onc-healthit/ONCLAIVE) | RESEARCH | RESEARCH_ONLY | CF-22 | AI-assisted requirements/test generation research only; model output cannot activate commandF authority. |
| [mitre/ig-summary](https://github.com/mitre/ig-summary) | PATTERN | PATTERN_ONLY | CF-18 | IG/profile normalization/comparison inspiration, not semantic oracle. |
| [HealthSamurai/fhir-profile-diff](https://github.com/HealthSamurai/fhir-profile-diff) | PATTERN | PATTERN_ONLY | CF-18/23 | UX/normalization study; any AI explanation remains non-authoritative. |
| [brianpos/UploadFIG](https://github.com/brianpos/UploadFIG) | PATTERN | PATTERN_ONLY | CF-17 | Important lesson: package closure can exist while canonical references remain unresolved. |

## Tier D — terminology infrastructure and evidence

| Source | Authority | Recommended mode | Target | Qualification note |
| --- | --- | --- | --- | --- |
| [IHTSDO/snowstorm](https://github.com/IHTSDO/snowstorm) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-25 | Strong SNOMED/FHIR terminology service behavior target. `CONTENT_RIGHTS` applies to SNOMED content. |
| [termx-health/termx-server](https://github.com/termx-health/termx-server) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-25 | Federation/discovery/terminology workflow donor. |
| [wardle/hades](https://github.com/wardle/hades) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-25 | Lightweight terminology-operation comparison across supported code systems/packages. |
| [IHTSDO/snowstorm-mcp-server](https://github.com/IHTSDO/snowstorm-mcp-server) | PATTERN | PATTERN_ONLY | CF-23/25 | Useful illustration of MCP access plus separate terminology-content licensing; never a reason to make MCP core architecture. |
| [OHDSI/CommonDataModel](https://github.com/OHDSI/CommonDataModel) | OFFICIAL_REFERENCE | PINNED_DATA | CF-16/26/28 | OMOP structural reference for cross-standard research. Vocabulary content rights remain separate. |

## Tier E — mapping and cross-standard analysis

| Source | Authority | Recommended mode | Target | Qualification note |
| --- | --- | --- | --- | --- |
| [GoogleCloudPlatform/healthcare-data-harmonization](https://github.com/GoogleCloudPlatform/healthcare-data-harmonization) | PATTERN | PATTERN_ONLY | CF-16/26 | Whistle 2 typed/intermediate mapping concepts, transpiler/linter/LSP patterns. Pin before any source reuse. |
| [microsoft/FHIR-Converter](https://github.com/microsoft/FHIR-Converter) | PATTERN | PATTERN_ONLY | CF-16/28 | Mapping corpus/Liquid transformation prior art; mapping/data rights reviewed separately. |
| [openFHIR/openfhir](https://github.com/openFHIR/openfhir) | RESEARCH | DIFFERENTIAL_ONLY | CF-16/26/28 | FHIRconnect/openEHR<->FHIR research oracle; upstream itself states OSS edition limitations, so no production dependency assumption. |
| [openehr-fhir/base-spec](https://github.com/openehr-fhir/base-spec) | OFFICIAL_REFERENCE | PINNED_DATA | CF-16/26/28 | Current openEHR-in-FHIR representation/reference input. |
| [openEHR/specifications-ITS-REST](https://github.com/openEHR/specifications-ITS-REST) | NORMATIVE | PINNED_DATA | future cross-standard/API analysis | Computable OpenAPI representations enable future compatibility analysis without inventing a new REST model. |
| [openEHR/archie](https://github.com/openEHR/archie) | INDEPENDENT_ORACLE | DIFFERENTIAL_ONLY | CF-16/28 | openEHR RM/AOM/validation behavior baseline. |
| [OHDSI/DataQualityDashboard](https://github.com/OHDSI/DataQualityDashboard) | PATTERN | PATTERN_ONLY | CF-14/28 | Data-quality measurement patterns; not a semantic conversion authority. |

## Tier F — evidence graph, SBOM, provenance, and signatures

| Source | Authority | Recommended mode | Target | Qualification note |
| --- | --- | --- | --- | --- |
| [guacsec/guac](https://github.com/guacsec/guac) | PATTERN | PATTERN_ONLY | CF-17/21/26 | High-fidelity graph normalization is strong prior art. Keep healthcare relationship semantics commandF-native. |
| [google/deps.dev](https://github.com/google/deps.dev) / deps.dev API | PATTERN | OPTIONAL_TOOLCHAIN | AF-03/CF-21 | Optional software dependency enrichment only; not FHIR package authority. Cache/policy under service terms. |
| [anchore/syft](https://github.com/anchore/syft) | PATTERN | OPTIONAL_TOOLCHAIN | AF-03 | Software SBOM generator; does not replace Interoperability BOM. |
| [CycloneDX/cyclonedx-rust-cargo](https://github.com/CycloneDX/cyclonedx-rust-cargo) | PATTERN | OPTIONAL_TOOLCHAIN | AF-03 | Rust SBOM path; invoking Cargo on untrusted projects requires explicit trust controls. |
| [oss-review-toolkit/ort](https://github.com/oss-review-toolkit/ort) | PATTERN | PATTERN_ONLY | AF-03/CF-21 | License/source/security policy and source-archive provenance donor. |
| [in-toto/in-toto](https://github.com/in-toto/in-toto) | PATTERN | PATTERN_ONLY | CF-21/26 | Materials/products/command link model maps well to reproducible transformation/build evidence. |
| [sigstore/cosign](https://github.com/sigstore/cosign) | PATTERN | OPTIONAL_TOOLCHAIN | AF-03/CF-26 | Signature/attestation plumbing; signatures prove integrity/provenance, not semantic correctness. |
| [sigstore/policy-controller](https://github.com/sigstore/policy-controller) | PATTERN | PATTERN_ONLY | CF-24/26 | Attestation policy evaluation reference, not required commandF runtime. |
| [slsa-framework/slsa](https://github.com/slsa-framework/slsa) | NORMATIVE | PINNED_DATA | AF-03/CF-21/26 | Use released specification version and provenance semantics; drafts are not silently upgraded. |
| [package-url/purl-spec](https://github.com/package-url/purl-spec) | NORMATIVE | PINNED_DATA | CF-21 | Do not invent an unregistered `pkg:fhir` type. Use commandF-native URI/identity until standardization exists. |

## Tier G — safe extension and policy engines

| Source | Authority | Recommended mode | Target | Qualification note |
| --- | --- | --- | --- | --- |
| [extism/extism](https://github.com/extism/extism) | PATTERN | PATTERN_ONLY initially | CF-24 | Strong Wasm plugin host/capability pattern: host-controlled HTTP, timers/limiters, explicit host functions. Evaluate concurrency/runtime behavior before adoption. |
| [bytecodealliance/wasmtime](https://github.com/bytecodealliance/wasmtime) | PATTERN | PATTERN_ONLY initially | CF-24 | Mature Wasm runtime candidate; security/advisory/update burden requires AF-01/03-style pinning if adopted. |
| [cedar-policy/cedar](https://github.com/cedar-policy/cedar) | PATTERN | PATTERN_ONLY | CF-24 | Authorization/policy language candidate for bounded institutional authorization, not compatibility-rule authority. |
| [cedar-policy/cedar-spec](https://github.com/cedar-policy/cedar-spec) | PATTERN | PATTERN_ONLY | research / CF-28 | Lean formal model + differential randomized testing is excellent methodology prior art. |
| [clarkmcc/cel-rust](https://github.com/clarkmcc/cel-rust) | PATTERN | RESEARCH_ONLY | CF-24 | Candidate non-Turing-complete expression layer if future organization predicates require it. Pin/maturity review required. |
| [microsoft/regorus](https://github.com/microsoft/regorus) | PATTERN | RESEARCH_ONLY | CF-24 | Rust Rego interpreter candidate only if OPA compatibility becomes a real user requirement. |

## Explicit lifecycle corrections

Some previously retained sources must not be treated as current simply because they remain useful historically:

- the broad `GoogleCloudPlatform/healthcare` repository was archived in 2026; use the still-relevant healthcare-data-harmonization project only under its own live identity;
- older FHIRconnect specification locations have been superseded/deprecated; the donor ledger must follow the currently maintained specification/repository identity before any new adoption;
- mutable upstream `main`, `latest`, build.fhir.org current content, package CI builds, and "latest release" download URLs are discovery conveniences, never retained evidence identities.

The open-source inventory should gain an automated stale-source check in a future planning/assurance unit: archived repository, renamed/default branch, disappeared release, changed license, or deprecation notice must surface as a review event.

## Strong recommendations by roadmap slice

### CF-17 Ecosystem Observatory

Start from official FHIR registry/feed metadata and immutable package/publication identities. Study GUAC's normalization model and UploadFIG's unresolved-canonical problem. Do not introduce Qdrant, a graph DB, or a registry service into the critical path.

### CF-18 Compatibility Coverage Model

Study `oasdiff checks changelog coverage` and Buf compatibility policies. Implement a FHIR-native change universe, not an adapter over OpenAPI/Protobuf rules.

### CF-19 Consumer Contract Scanner

Use SQL-on-FHIR/FHIRPath/CQL/Search/TestScript specifications as data/contract inputs. Study Pact's version matrix but define commandF's healthcare consumer semantics independently.

### CF-20 Compatibility Lab

Use the official validator/publisher as bounded authoritative process oracles; HAPI, Firely, Candle, Blaze, FHIRPath JS/Lab, and selected Rust engines as independent comparisons. Every divergence remains classified evidence.

### CF-21 Interoperability BOM

Use Syft/CycloneDX for the *software* SBOM and in-toto/SLSA/Sigstore for provenance/signing patterns where appropriate. Keep the healthcare Interoperability BOM schema separate unless a generic standard has exact semantics for a relationship.

### CF-22 Deterministic TestGen

Study FHIR TestScript Generator and Inferno. ONCLAIVE can inform AI-assisted extraction research, but no generated requirement/test becomes authority until deterministic/human acceptance binds it to exact source evidence.

### CF-24 Plugin SDK

Begin with the capability model, not a runtime dependency. Freeze the commandF host contract and negative capabilities first; then benchmark/evaluate Extism versus direct Wasmtime or another Wasm host under AF-04/AF-03 requirements.

### CF-25 Terminology Intelligence

Use Snowstorm, TermX, and Hades as independent endpoints/oracles. Treat SNOMED/LOINC/ICD and other terminology content rights independently of server code licenses. commandF remains a terminology evidence/federation client, not a terminology server.

## Sources retained but intentionally *not* promoted

The discovery annex already contains many valuable technologies that do not need promotion into immediate execution:

- Qdrant/Tantivy/Oxigraph — keep optional until measured retrieval/graph requirements justify them;
- Kafka/NATS/Debezium/Camel/NiFi — future Gateway transport patterns, not current change-intelligence dependencies;
- OpenFGA/Keycloak/SPIFFE/SPIRE — future deployment/authorization infrastructure, not compatibility semantics;
- Presidio/EMPI systems — relevant only to separated instance-data/private workflows;
- DICOM/OHIF/DCMTK/pydicom — later imaging research, not FHIR V1 breadth pressure;
- arbitrary agent frameworks — optional replaceable agent plane only.

## Adoption checklist template

Every future donor record should include at minimum:

```yaml
source:
  repository: owner/name
  upstream_url: https://github.com/owner/name
  exact_ref: <commit-or-release>
  immutable_digest: <when applicable>
  authority_class: <enum>
  adoption_mode: <enum>
  maintenance_state: ACTIVE|DEPRECATED|ARCHIVED|UNKNOWN
rights:
  software_license: <SPDX-or-reviewed-text>
  content_license_or_null: <separate disposition>
  redistribution_allowed: <evidence>
trust:
  data_only: true|false
  executes_external_process: true|false
  executes_untrusted_code: true|false
  network_required: true|false
  offline_mode: <evidence>
scope:
  owning_slice: CF-XX|AF-XX
  relevant_paths: []
  intended_use: <bounded description>
  semantic_authority: NONE|BOUNDED_ORACLE|NORMATIVE_INPUT
exit:
  upgrade_rule: <rule>
  removal_rule: <rule>
```

Conversation permission never replaces upstream licensing or this evidence record.
