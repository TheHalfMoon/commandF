# CF-01 Convergence Review

Status: Draft readiness candidate
Date: 2026-08-13

## Consistency result

The implementation remains inside CF-01 scope: two exercised Rust crates, deterministic FHIR package resolution, bounded public/local acquisition, bounded archive parsing, content-addressed caching, deterministic commandF locking, and offline verification. No CF-02 canonical indexing, structural diff, FHIR resource validation, mapping execution, or AI runtime has been introduced.

## Plan coverage reconciliation

Prior commandF discovery is now an explicit plan set rather than conversation-only memory:

- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md` — execution authority;
- `docs/COMMAND_F_PLAN_INDEX.md` — plan-set index and no-silent-drop rule;
- `docs/COMMAND_F_DISCOVERY_COVERAGE_2026-08-13.md` — retained standards, open-source candidates, tools, product inspirations, benchmarks, runtime candidates, and sixteen research tracks;
- `docs/COMMAND_F_GAP_LEDGER_2026-08-13.md` — 35 interoperability gap hypotheses and response map;
- `docs/PROVENANCE_AND_DONOR_POLICY.md` — pin/license/permission/adoption controls;
- slice-specific donor manifests under `donors/`.

The coverage set retains the discussed FHIR/openEHR/OMOP/HL7v2/CDA/DICOM ecosystems; independent validators and servers; Whistle, FHIRconnect, openFHIR, Eos, OMOCL, Microsoft FHIR Converter, FML/StructureMap and FHIRPath mapping prior art; terminology systems; query/analytics tooling; privacy/identity/policy tooling; edge/gateway/runtime candidates; provenance/supply-chain tooling; context/search systems; fuzz/differential testing; software-quality/review donors; and product inspirations including Greptile, Cubic, Graphite, Augment, Qodo, and SonarQube.

MLIR, GoFSH, and Open Concept Lab are explicitly retained in the Plan Index as audit corrections. A retained candidate is not automatically adopted; exact provenance and rights gates still apply.

## Verified implementation state

CF-01 includes:

- public FHIR registry acquisition by default with secondary-registry fallback;
- local mirror acquisition through `--source-dir`;
- exact and FHIR `major.minor.x` latest-patch selection semantics;
- transitive dependency resolution and fail-closed concrete-version conflicts;
- deterministic `commandf.lock`;
- exact archive SHA-256 cache identity and offline verification;
- exact generated `Cargo.lock` committed from CI-exported bytes;
- CI-enforced Cargo dependency locking;
- CLI exit coverage for success `0`, runtime/verification failure `1`, and usage error `2`;
- real public-registry resolution of `hl7.fhir.r4.core@4.0.1` followed by offline cache verification;
- immutable CF-01 donor provenance;
- bounded compressed HTTP bodies and bounded decompressed TAR traversal;
- fail-closed malformed registry-version handling with secondary endpoint fallback;
- RAII-managed temporary cache files, explicit file sync, and atomic persistence to the final digest path.

## Current exact CI evidence

Commit `aa58fd7ffae95aa0b18239a85e17cbf09658e6af` changed CI to require `Cargo.lock` and removed the temporary lock export step.

The later cache-finalization head `9c6f2db93c3c4f6e44e476ea64b19e1d2b88c8b0` passed GitHub Actions run `31728835097`:

- Format — PASS;
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` — PASS;
- `cargo test --locked --workspace --all-features` — PASS;
- real FHIR registry resolve using `cargo run --locked` — PASS;
- offline cache verify using `cargo run --locked` — PASS.

The workspace lockfile originated from a successful CI artifact and was committed in `147f7f61e26e638c4a94a6b169447275d16fd2f8`; its Git blob SHA is `0a82d71a67daf342c573b49718d03a4bbb1c053b`.

## CodeRabbit review disposition

CodeRabbit produced six Major findings and one Minor finding on the pre-hardening state.

### Fixed and resolved

1. **CI did not enforce `Cargo.lock`.**
   - Fixed in `aa58fd7ffae95aa0b18239a85e17cbf09658e6af`.
   - Thread resolved after locked CI passed.

2. **Archive traversal was not bounded after decompression.**
   - Fixed in `8625a54aa2175bf525c63d7c3eed896e67a7f3e4` plus formatting follow-up `0994b7c1a9ec85c08a219b449977fc57a6c26824`.
   - Total decompressed TAR bytes and entry count are bounded and tested.
   - Thread resolved.

3. **Cache object final path could be exposed during a direct write.**
   - Initial atomic-publication fix: `1afe6dcbcc7a8a1e2cb243fbb020cad89ace12c6` plus formatting follow-up `24bf4443b8dc2d1f47b9b78f918da8ab42558019`.
   - Final RAII cleanup/durability refinement: `4e135541aa4e49e1e4d2688b15b130ec7f47f62d` and `9c6f2db93c3c4f6e44e476ea64b19e1d2b88c8b0` using `NamedTempFile`, `sync_all`, and atomic `persist`.
   - Thread resolved after locked CI passed.

4. **Malformed registry version keys were silently discarded.**
   - Fixed/refined through `bcc6a2a1049fab8f0e45422e26e24b7372be63ec` and `726e7943a3a10e20b3e4a6022b5ece56544d4100`.
   - Invalid metadata invalidates that endpoint response and preserves secondary-registry fallback.
   - Thread resolved.

5. **Spec Kit donor used a mutable tag.**
   - Fixed in `d99d027fe224ca88784394d54866a27010641882`.
   - `v0.16.2` is supplementary metadata; immutable donor ref is `4871b485f97c7fa452ec58eba325d87536c55c34`.
   - Thread resolved.

6. **README/spec showed unavailable `pkg add`.**
   - Fixed in `33dba294bc710980d37ed50b04b16c2b49076834` and `8b4636bf722433a91b8f4313d2d119364fcb75f5`.
   - User flow now uses `commandf pkg resolve`.
   - Thread resolved.

### Review disposition: resolver range-intersection suggestion not adopted

CodeRabbit suggested treating exact `1.2.3` plus `1.2.x` as compatible even when `1.2.x` would otherwise select a higher patch.

CF-01 intentionally does not model `major.minor.x` as a generic semver range. The FHIR package specification defines `x` as selecting the highest found patch number. The CF-01 specification now records that latest-patch-selector semantic explicitly. A lower exact request that differs from the concrete wildcard-selected version is therefore an intentional fail-closed conflict in this slice.

A generalized constraint-intersection solver remains a possible future feature, but adopting it requires an explicit specification amendment rather than an implicit behavior change during review.

## Remaining readiness items

1. Record the resolver thread as reviewed/not-adopted according to the source-backed CF-01 specification.
2. Trigger one final CodeRabbit review against the post-fix head and inspect any new findings.
3. Qodo was manually requested, but no Qodo result/comment has been returned and therefore no Qodo PASS is claimed.
4. Keep the PR Draft; no merge is authorized by this convergence record.

## Explicit deferrals

- deterministic injected-filesystem multi-process stress testing beyond the current publication invariant;
- private/authenticated registries;
- broader semver/range syntax or a generalized constraint solver;
- CRMI release-manifest interoperability;
- any CF-02 or later feature.

These deferrals do not authorize silent behavior changes. Later adoption requires a new slice or explicit CF-01 amendment.
