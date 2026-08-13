# CF-01 Tasks

Status: Draft

## T001 — Workspace bootstrap

- Add `commandf-pkg` library crate.
- Add `commandf` CLI crate that consumes it.
- Keep the workspace to shipped or directly exercised code only.

Done when the workspace builds with no placeholder crate.

## T002 — Package identity and constraints

- Define package name and exact version types.
- Parse `name@version` roots.
- Support exact versions and `major.minor.x` only.
- Reject unsupported constraints explicitly.

Done when positive and negative parser tests pass.

## T003 — PackageSource abstraction

- Define transport-neutral metadata and archive acquisition contract.
- Add in-memory test source.
- Ensure the resolver owns selection semantics.

Done when resolver tests require no external service.

## T004 — Archive manifest reader

- Read `package/package.json` from `.tgz` bytes without extracting to disk.
- Bound archive entry reads.
- Validate package identity against the selected package.

Done when missing manifest and identity-mismatch fixtures fail closed.

## T005 — Content-addressed cache

- Compute SHA-256 over exact archive bytes.
- Store objects by digest.
- Verify cache objects by recomputing the digest.

Done when corruption is detected deterministically.

## T006 — Dependency resolver

- Resolve roots and transitive dependencies.
- Choose highest stable patch for `major.minor.x`.
- Exclude prereleases from wildcard selection.
- Detect incompatible concrete-version requests.

Done when deterministic graph and conflict tests pass.

## T007 — Deterministic lockfile

- Define lock schema v1.
- Sort roots and packages deterministically.
- Serialize byte-stably.
- Record dependency constraints and archive digests.

Done when repeated serialization is byte-identical.

## T008 — CLI vertical slice

- Implement `commandf pkg resolve`.
- Implement `commandf pkg verify`.
- Use explicit exit codes for usage and verification failure.

Done when CLI integration tests exercise the library.

## T009 — CI

- Add fmt, clippy, and test workflow.
- Keep unit tests offline.
- Add a separate pinned-package smoke path before readiness.

Done when the exact PR head is green.

## T010 — Review and convergence

- Run Spec Kit consistency analysis over spec, plan, and tasks.
- Run CodeRabbit when available.
- Run Qodo when connected.
- Record any remaining gap as a new task or explicit deferral.

Done when there is no hidden work required for the CF-01 definition of done.