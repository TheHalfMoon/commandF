# CF-01 Convergence Review

Status: Draft convergence checkpoint
Date: 2026-08-13

## Consistency result

The implementation remains inside the approved CF-01 scope: two exercised Rust crates, transport-neutral resolution semantics, bounded archive parsing, content-addressed cache, deterministic commandF lockfile, and no CF-02 canonical indexing or later feature work.

The implementation advanced beyond the original plan in controlled ways:

- public FHIR registry acquisition is now the CLI default;
- local mirror acquisition remains available through `--source-dir`;
- registry fallback behavior is grounded in the pinned FHIR package loader used by SUSHI;
- source provenance is recorded per acquired archive;
- a real registry smoke test resolves `hl7.fhir.r4.core@4.0.1` and verifies its cache offline;
- cache verification validates SHA-256 syntax before cache lookup;
- local-mirror provenance is logical rather than machine-path-specific.

No product requirement expanded into FHIR validation, snapshot generation, canonical indexing, structural diff, mapping execution, or AI runtime.

## Evidence checkpoint

GitHub Actions run `31717420374` passed format, Clippy with warnings denied, tests, real public FHIR package resolution, and offline cache verification.

GitHub Actions run `31717736017` additionally exported the exact generated `Cargo.lock` artifact successfully.

The downloaded lockfile contains 1184 lines. Its local SHA-256 is `0c58bb1b2a78935dfa494334f4d6bddf9fb3f82c888f031c0e4fc803e7d810d0` and its Git blob SHA is `0a82d71a67daf342c573b49718d03a4bbb1c053b`.

## Remaining readiness blockers

1. Commit that exact Cargo.lock and switch Cargo gates to `--locked`.
2. Add dedicated CLI integration/exit-behavior coverage, or explicitly narrow T008 before readiness.
3. Remove the temporary Cargo.lock export step after the lockfile is committed.
4. Retry CodeRabbit after its rate limit resets.
5. Run Qodo when Draft feedback is available or when the PR is otherwise ready for review.
6. Re-run this convergence check against the exact readiness candidate.

## Explicitly deferred quality items

- atomic cache temp-write plus rename hardening;
- moving the external registry smoke to a different cadence if per-PR registry availability proves flaky;
- private/authenticated registries;
- broader semver syntax;
- CRMI release-manifest interoperability.

These deferrals do not authorize silent behavior changes. Any later adoption requires a new slice or explicit CF-01 amendment.
