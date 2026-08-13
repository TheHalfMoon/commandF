# CF-01 Convergence Review

Status: Draft convergence checkpoint
Date: 2026-08-13

## Consistency result

The implementation remains inside the approved CF-01 scope: two exercised Rust crates, transport-neutral resolution semantics, bounded archive parsing, content-addressed cache, deterministic commandF lockfile, public/local package acquisition, and no CF-02 canonical indexing or later feature work.

The implementation advanced beyond the original plan in controlled ways:

- public FHIR registry acquisition is now the CLI default;
- local mirror acquisition remains available through `--source-dir`;
- registry fallback behavior is grounded in the pinned FHIR package loader used by SUSHI;
- source provenance is recorded per acquired archive;
- a real registry smoke test resolves `hl7.fhir.r4.core@4.0.1` and verifies its cache offline;
- cache verification validates SHA-256 syntax before cache lookup;
- local-mirror provenance is logical rather than machine-path-specific;
- root resolution is order-independent for equivalent exact/wildcard constraint sets;
- CLI integration tests now prove exit behavior: success `0`, runtime/verification failure `1`, and Clap usage error `2`.

No product requirement expanded into FHIR validation, snapshot generation, canonical indexing, structural diff, mapping execution, or AI runtime.

## Plan coverage reconciliation

The commandF plan set now preserves prior discovery rather than relying on conversation memory:

- `docs/COMMAND_F_PLAN_INDEX.md` defines the plan layers;
- `docs/COMMAND_F_DISCOVERY_COVERAGE_2026-08-13.md` retains standards, open-source candidates, product/tool inspirations, benchmarks, runtime candidates, and sixteen research tracks;
- `docs/COMMAND_F_GAP_LEDGER_2026-08-13.md` preserves the 35 gap hypotheses and commandF response map;
- `docs/PROVENANCE_AND_DONOR_POLICY.md` defines adoption/pinning/permission gates;
- MLIR, GoFSH, and Open Concept Lab are explicitly retained as coverage corrections in the Plan Index.

This documentation expansion does not enlarge CF-01 runtime scope.

## Evidence checkpoint

GitHub Actions run `31717420374` passed format, Clippy with warnings denied, tests, real public FHIR package resolution, and offline cache verification.

GitHub Actions run `31717736017` additionally exported the exact generated `Cargo.lock` artifact successfully.

That exact lockfile was committed in `147f7f61e26e638c4a94a6b169447275d16fd2f8`; its Git blob SHA is `0a82d71a67daf342c573b49718d03a4bbb1c053b`.

GitHub Actions run `31725045080` passed Format, Clippy, all tests including dedicated CLI exit-behavior integration tests, the real FHIR registry smoke, offline cache verification, and the temporary lockfile export step on head `fec2215b2fbb9112220698963c70b73413fe3c7c`.

## Closed readiness items

1. Generated `Cargo.lock` is committed byte-for-byte from the CI-exported artifact.
2. Bounded public FHIR registry acquisition is implemented.
3. A pinned real FHIR package resolves end-to-end and verifies offline from cache.
4. Dedicated CLI exit behavior is covered by integration tests.
5. Spec/plan/tasks implementation consistency has been reviewed and this convergence record updated.
6. CF-01 donor provenance is recorded for the package-resolution implementation sources/patterns used by this slice.

## Remaining readiness blockers

1. Switch Cargo CI gates and smoke commands to `--locked` and remove the temporary `Cargo.lock` export step. Multiple connector writes to `.github/workflows/ci.yml` were blocked before execution; this remains an explicit unresolved gate, not an assumed success.
2. Retry CodeRabbit after its rate limit permits a fresh review.
3. Run Qodo when Draft feedback is available or when the PR is otherwise ready for review.
4. Re-run final convergence against the exact readiness candidate after the locked-CI change and reviewer findings are resolved.

## Explicitly deferred quality items

- atomic cache temp-write plus rename hardening;
- moving the external registry smoke to a different cadence if per-PR registry availability proves flaky;
- private/authenticated registries;
- broader semver syntax;
- CRMI release-manifest interoperability.

These deferrals do not authorize silent behavior changes. Any later adoption requires a new slice or explicit CF-01 amendment.
