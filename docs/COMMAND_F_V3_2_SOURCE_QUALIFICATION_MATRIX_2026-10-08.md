# commandF V3.2 — Source Qualification Matrix

Status: **PLANNING_CANDIDATE / NOT AN ADOPTION RECORD**

Observed canonical main: `f82565cca917d119e1c774b2c470e2ac20e0d6dd`. Research date: 2026-10-08.

This matrix ranks candidate sources. It does not adopt any of them. Under `docs/PROVENANCE_AND_DONOR_POLICY.md`, adoption still requires a slice-specific donor record with:

- exact identity;
- file paths;
- license and rights disposition;
- adoption mode;
- tests.

Donor records live under `donors/`, which is an AF-02 authority prefix. They therefore cannot be added until the Phase 0 governance migration exists (see the roadmap). This document deliberately places no file under `donors/`.

## 1. Adoption-mode crosswalk

The brief's modes map onto the existing V3 modes as follows. Both vocabularies are kept, and neither replaces the other.

| Brief mode | Meaning here | Nearest V3 mode |
| --- | --- | --- |
| DEPEND | Compiled or linked dependency, version-locked. | OPTIONAL_TOOLCHAIN or a Cargo dependency |
| EMBED | Bytes shipped inside a commandF artifact. | PINNED_DATA |
| PORT | Re-implement a donor's design in commandF Rust, with attribution. No code copied. | PATTERN_ONLY with a port record |
| COPY | Copy donor source files. Requires a file-level notice and license compatibility. | (no V3 equivalent; the strictest gate) |
| IMPORT | Read external data at runtime from a user-supplied, pinned bundle. | PINNED_DATA, not redistributed |
| ORACLE | Execute as an independent or authoritative comparator. | PINNED_PROCESS_ORACLE / DIFFERENTIAL_ONLY |
| STUDY | Read only, to inform design. | PATTERN_ONLY / RESEARCH_ONLY |

## 2. Rights dimensions

Every row is evaluated separately on these six rights dimensions:

| Code | Dimension |
| --- | --- |
| S | software license |
| C | IG/spec content rights |
| T | terminology rights |
| D | data rights |
| M | model-weight rights |
| TM | trademark |

A software license never clears C, T, D, M, or TM.

## 3. External sources — prioritized

Identities were read on 2026-10-08 with `gh api`. "Head" is the default-branch commit at read time. It is a research identity, not an adoption pin. Adoption must re-pin to a release tag plus a commit, or a commit plus a blob.

### P0 — needed by the first product journey or by an active blocker

| # | Source | Identity observed | S | Other rights | Proposed mode | Owner | Purpose | Why reuse beats building | Acceptance evidence before adoption |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `hapifhir/org.hl7.fhir.core` validator | Production pin 6.10.2 (`d06577db…`, jar `a3addadf…`). Latest is 7.0.0 (2026-10-06). | Apache-2.0 | C: HL7 core spec content is CC0 per HL7 FHIR license. Verify per package. | ORACLE (existing) | CF-06/CF-20 | Authoritative validation and snapshot comparator. | V2 oracle rule: do not reimplement. | Keep 6.10.2. Any repin needs a full CF-06 regression, a frozen CF-10 A/B byte-identical summary, independent review, and FD-4. 7.0.0's R6-internal shift and snapshot rework make it a **new oracle identity**, not a patch bump. |
| 2 | `FHIR/ig-registry` `package-feeds.json` | Last-touch commit `0b680b7486eeaf7aa73b8ef816e944f7c4528584` (2026-10-07). Blob `2ba3a30538295e6e95d5e1bb6f6f754b6c20ce13`, 20,534 bytes. 78 feeds, 17 of them plain `http://`. 49 `package-restrictions` masks. | **No LICENSE file in the repository** (GitHub license: null) | C: unknown | IMPORT, as candidate data only | CF-17 catalog + G54 | Bounded official feed list, and namespace-restriction data for package-confusion defense. | It is the official crawl configuration for packages.fhir.org. Inventing a list would be weaker. | (a) Founder or HL7 rights confirmation (FD-5). (b) Pin by commit + blob. (c) Never fetch a plain-`http://` feed as authority; record it as `UNAUTHENTICATED_TRANSPORT`. (d) Coverage is reported as `OFFICIAL_FEED_LIST_AT_COMMIT`, never `COMPLETE_REGISTRY_HISTORY`. |
| 3 | `packages.fhir.org/catalog?…` query endpoint | Observed to answer `catalog?name=…` with JSON. Semantics, pagination, and completeness are undocumented. | n/a (service) | C: per package | STUDY now; IMPORT later only if documented | CF-17 | Possible bounded catalog on an **already-authorized host** (spec 043). | Spec 048 said no authorized host names a catalog document. This endpoint is on an authorized host, but it is query-shaped. | Write a spec amendment that names exact query strings, records response bytes and digests, and defines the coverage label `QUERY_RESULT_AT_TIME`. Never call it a complete listing. |
| 4 | `FHIR/fhir-test-cases` | Release 1.8.0 (2026-10-06), head `d49929d4…` | Apache-2.0 | C: test content under the same license. Verify per file. | IMPORT, as benchmark data | CommandFBench | Externally labeled validation cases. | The labels come from HL7 test authors, not commandF (`research/LABEL_AUTHORITY.md`). | Pin the release. Map each case to a CommandFBench case class. Make sure none is used for rule tuning (split policy). |
| 5 | `HL7/fhir-ig-publisher` | 2.3.5 (2026-10-06), head `abc1dfaa…` | Apache-2.0 | — | ORACLE (process, network-disabled where possible) | CF-21 (G10/G11) | Source-to-package reproducible build and snapshot authority. | It is the only authoritative snapshot generator. | Pinned jar digest, offline package closure, and recorded divergence classes. |
| 6 | `FHIR/sushi` | v3.20.1 (2026-08-18) | Apache-2.0 | — | ORACLE / OPTIONAL_TOOLCHAIN | CF-09/CF-21 | FSH compile for the source-map round trip. | It is the reference FSH compiler. | Pinned npm tarball integrity, offline run. |

### P1 — consumer contracts, differential evidence, and benchmark breadth

| # | Source | Identity observed | S | Mode | Owner | Purpose | Notes and acceptance |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 7 | `HL7/sql-on-fhir` (redirect from `FHIR/sql-on-fhir-v2`) | 3.0.0-ballot (2026-08-13), head `e3e1d3c7…` | NOASSERTION on GitHub | IMPORT (ViewDefinition schema and tests) + STUDY | CF-19 | ViewDefinition consumer contracts, including the analytics and AI feature-extraction consumers. | Distinguish 2.0.0 (published) from 3.0.0-ballot (draft). Treat the license as unknown until the repository's actual terms are read. |
| 8 | `inferno-framework/inferno-core` and `us-core-test-kit` | v1.4.5 (2026-10-05) and v1.1.6 (2026-09-03) | Apache-2.0 | ORACLE (optional, local Docker) + STUDY | CF-20 | Executable consumer and protocol contract evidence. | Running it needs a server under test, so it is never on the deterministic core path. Record `REQUIRED_NOT_RUN` when absent. |
| 9 | `oasdiff/oasdiff` | v1.33.0 (2026-10-01) | Apache-2.0 | STUDY | CF-18 | Mature breaking-change taxonomy design: check IDs, levels, and an ignore mechanism. | Port concepts only. OpenAPI semantics are not FHIR semantics. |
| 10 | `bufbuild/buf` (breaking rules) | head at read | Apache-2.0 | STUDY | CF-18 | Rule categories (FILE, PACKAGE, WIRE_JSON, WIRE) are a precedent for a compatibility-level lattice. | Concept only. |
| 11 | `pact-foundation/pact-specification` | last push 2024-04-11 | MIT | STUDY | CF-19 | Consumer-driven contract identity, provider verification, and "can-i-deploy" matrix semantics. | Older and less active. Use the matrix concept, not the format. |
| 12 | `samply/blaze` | v1.11.0 (2026-08-23) | Apache-2.0 | ORACLE (independent, FHIRPath and search) | CF-20 | Independent server behavior. | Differential only. |
| 13 | `FHIR/fhir-candle` | no release, head `f04a1d4b…` | MIT | ORACLE (independent, R4/R4B/R5) | CF-20 | Multi-version reference server. | No release tag, so pin by commit and record that. |
| 14 | `hapifhir/hapi-fhir` | head at read | Apache-2.0 | ORACLE (independent) | CF-20 | Server and parser behavior. | Shares the core library with #1. **Not independent of the HL7 validator for structure-definition logic**, so disagreement analysis must account for the shared code. |
| 15 | `FirelyTeam/firely-net-sdk` | head at read | NOASSERTION on GitHub | ORACLE (independent) | CF-20 | Truly independent .NET implementation. | Read the license file before any use. |
| 16 | `HL7/fhirpath.js` | head at read | NOASSERTION on GitHub | ORACLE (FHIRPath differential) | CF-19/CF-20 | Independent FHIRPath engine. | Read the license file. |
| 17 | `FHIR/GoFSH`, `FHIR/fhir-package-loader` | v2.6.1 and v2.2.4 (both 2026-03-08) | Apache-2.0 | ORACLE / STUDY | CF-21 / CF-01 | FSH round trip; package-loading semantics reference. | Lower activity (last release in March). Record the lifecycle state. |

### P2 — supply chain and trust (AF-03)

| # | Source | Identity | S | Mode | Purpose |
| --- | --- | --- | --- | --- | --- |
| 18 | `sigstore/cosign` | v3.1.3 | Apache-2.0 | OPTIONAL_TOOLCHAIN | Keyless signing and bundle verification for releases. Offline verification uses the bundle. |
| 19 | GitHub artifact attestations / SLSA v1.2 | spec | — | STUDY → DEPEND (CI) | Build provenance. |
| 20 | `in-toto/in-toto-golang` | head | NOASSERTION | STUDY | Attestation statement format for receipts. Reuse the predicate shape; do not run their tool in the core. |
| 21 | `anchore/syft` | v1.54.1 | Apache-2.0 | OPTIONAL_TOOLCHAIN | Software SBOM (CycloneDX) for release artifacts only. It is **not** the Interoperability BOM. |
| 22 | `CycloneDX/specification` | head | Apache-2.0 | STUDY | Export target for the software SBOM. Do not encode FHIR semantics in it (V3 rule). |

### P3 — synthetic and research data

| # | Source | Identity | S | Rights caution | Mode | Purpose |
| --- | --- | --- | --- | --- | --- | --- |
| 23 | `synthetichealth/synthea` | head `d9d07a6e…` (rolling "master-branch-latest") | Apache-2.0 | T: generated output embeds SNOMED CT, LOINC, and RxNorm codes. **Code use is subject to the terminology licenses.** Do not commit generated bundles until T is cleared. | IMPORT (user-generated, local) | Synthetic consumer instance fixtures for runtime probes and AI-readiness research. |
| 24 | `OHDSI/DataQualityDashboard` | head | license null on GitHub | — | STUDY | Data-quality check taxonomy (conformance, completeness, plausibility) for AI-readiness research. |

### P4 — embedded model runtime (see the D1 plan)

| # | Source | Identity | Rights | Mode | Notes |
| --- | --- | --- | --- | --- | --- |
| 25 | `ggml-org/llama.cpp` | Latest release v0.6.0 (2026-10-05, `8345f333…`). d1-3B model support merged to master in PR #30110 (`88dcc460d628`, 2026-10-07), **after** v0.6.0. `/v1/systemone` API merged in PR #29818 (2026-10-02). Hardening PR #30027 was **closed unmerged**. | S: MIT | DEPEND (separate process), pinned by commit | No tagged release contains d1-3B support yet. |
| 26 | `LiquidAI/d1-3B-GGUF` | Repository revision `bb1e436ea78eb96a3f1acb6da865f70c2fbeb563`. `d1-3B-Q4_K_M.gguf` is 1,674,456,672 bytes, LFS sha256 `16aff27e…22402`. | M: **LFM Open License v1.0**. Commercial use is licensed only to entities below $10M annual revenue. TM: no trademark grant. | IMPORT (user-acquired model pack), never default-embedded | See the D1 plan, §4. |

### Regional

| # | Source | Finding | Mode |
| --- | --- | --- | --- |
| 27 | Saudi NPHIES IGs | `packages.fhir.org/catalog?name=nphies` returned `[]` on 2026-10-08. A general web search found no authoritative public package identity or license statement. | FUTURE_RESEARCH. Requires a rights inquiry to the publisher (CHI/NPHIES) before any byte is fetched or redistributed. Never treat portal documents as redistributable or as final normative authority. |
| 28 | Mature regional IGs (US Core, AU Base, CH Core, and others) | Present in the official feeds (row 2). | IMPORT via the official registry once the catalog source is qualified. They carry their own C/T rights per package `license` field. |

## 4. Founder-owned sources

All eight repositories were accessed with the founder's authenticated GitHub CLI on 2026-10-08 (shallow clones). Visibility and HEAD:

| Repo | Visibility | HEAD | License |
| --- | --- | --- | --- |
| Ascout | public | `ca6b6f514e5a8881e2cfa2789b5e5ea43e3aaaad` | Apache-2.0 |
| Sentrdel | private | `f5747319a50831ef7cee983d253c0ca5503c9a64` | Apache-2.0 |
| Kodac | private | `406b335277f2df1e3dedf24cdb45847dff919d44` | Apache-2.0 |
| kernux | private | `2085b6ed1121b1a94c66c076bdd6b578da4dad0d` | Apache-2.0 |
| DAL | public | `81da6c58bdc2d67e888c78bedac9e25f02070eb3` | Apache-2.0 |
| SafeEvidence | public | `ad9f7b972d4d0cb3dc388ce4caa6c18e369e3b22` | Apache-2.0 |
| MESC | public | `f9b7579189b77b06379d1a71d104d4b0267cf500` | Apache-2.0 |
| MedScale | public | `37f5ae8a965c1e49d961010c3135a5b5dde12d67` | Apache-2.0 |

The review was at README and module-inventory depth. It was not a line-level code review.

The founder's general permission does not replace file-level provenance. Any COPY or PORT still needs a donor record naming exact files and blob SHAs, plus third-party notices inherited from the donor. For example, Ascout and SafeEvidence carry `THIRD_PARTY_NOTICES.md`, and copied files may themselves be third-party.

| Repo | Reusable element (path) | commandF owner | Mode | Duplicate check | Acceptance evidence |
| --- | --- | --- | --- | --- | --- |
| DAL | Negative findings in `registry/study1_d1_pilot_diagnosis.json` (F1 majority-class collapse; F2 paper equals control; F3 trivial sufficiency separation; F4 coverage target above the answerable fraction; F5 constant-score tie forcing full commit). Frozen-claims ledger pattern in the README `DAL:FROZEN-RESULTS` block. | D1 plan, CommandFBench | STUDY | Not duplicated | The D1 and bench plans must carry explicit tests for F1–F5 (see those plans). Note: "D1" in DAL names a study pilot. **It is not Liquid AI's d1 model.** DAL contains no evaluation of LiquidAI/d1-3B. |
| Ascout | Receipt separating proved, failed, refused, unresolved, and not-run (`src/receipt/`, `schemas/`) | CF-21 Decision Receipt | PORT (concepts) | commandF has stage states in V3.1 §9.2. Port only the "refused versus not-run" distinction if it is missing. | Receipt invalid-state tests. |
| Sentrdel | `crates/sentrdel-schema/src/coverage.rs` (`CoverageState`, `CoverageRecord::is_gap`); `schemas/v1/coverage.schema.json`; canonical `content_id` in `canonical.rs` | CF-18 coverage matrix | PORT (coverage-state design). STUDY only for canonical JSON. | commandF already has canonical byte emission (CF-17). **Do not import a second canonicalizer**; see G50. | Coverage-state invariant tests. |
| MedScale | `crates/medscale-pack/src/format.rs` admission errors (`MissingRights`, `DigestMismatch`, `MissingSignature`, `UnknownTrustRoot`, `AntiRollback`) | D1 model-pack admission, offline bundle (G19/G35) | PORT | Not present in commandF | Admission-refusal tests for each error. |
| MedScale | `crates/medscale-desktop` (Slint, native, non-WebView) | Studio shell option | STUDY | — | Studio shell decision record (UX plan §6). |
| SafeEvidence | Invariants: no silent cloud fallback, abstention as a first-class outcome, "permission makes a source eligible, provenance makes it adoptable" | D1 plan, donor policy | STUDY | Already reflected in V3.1 | — |
| Kodac | Done-Gate `PROVEN_READY` and policy-gated tool execution | CF-24 adapter boundary | STUDY | V3.1 §12 covers it | — |
| kernux | Capability kernel and "no silent external fallback" | CF-24 | STUDY | Product implementation has not started (README) | Not mature enough to port. |
| MESC | FHIR validation boundary (`medscale.fhirkit`, Python) | — | REJECT for commandF code | It would add a Python runtime to the core. commandF's oracle boundary is the HL7 validator. | — |

## 5. Rejected alternatives

| Alternative | Reason |
| --- | --- |
| Pin HL7 core 7.0.0 or 6.10.4 because it is newer. | Explicitly prohibited without differential qualification. 7.0.0 changes snapshot generation and the internal model basis. |
| Use mutable `build.fhir.org` CI output as package authority. | V3 invariant: telemetry only. |
| Use Jev (TypeSafe API) as a product decision advisor. | It is a hosted, paid API (`jev login` stores a TypeSafe key). That violates the offline and no-cloud requirement. It is allowed only as a research comparator with a cost authorization (D1 plan §8). |
| Bundle d1-3B weights in the default installer. | The LFM v1.0 threshold binds each downstream licensee (D1 plan §4). |
| Add a graph database for the ecosystem graph. | No measured workload. Embedded relational storage stays the default (V3). |
| Treat GitHub repository metadata license fields as rights clearance. | `ig-registry` shows none, and several repositories show NOASSERTION. Read the actual file. |
| Use `smart-on-fhir/bulk-data-tools` as a contract source. | Last push 2022-12-03 and no license on GitHub. Use the Bulk Data IG text instead. |
