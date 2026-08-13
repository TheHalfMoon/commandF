# CF-01 Tasks

Status: In implementation / convergence

## T001 — Workspace bootstrap — COMPLETE

- Add `commandf-pkg` library crate.
- Add `commandf` CLI crate that consumes it.
- Keep the workspace to shipped or directly exercised code only.

## T002 — Package identity and constraints — COMPLETE

- Define package name and exact version types.
- Parse `name@version` roots.
- Support exact versions and `major.minor.x` only.
- Reject unsupported constraints explicitly.

## T003 — PackageSource abstraction — COMPLETE

- Define transport-neutral metadata and archive acquisition contract.
- Add in-memory test source.
- Ensure the resolver owns selection semantics.
- Record actual source provenance per acquired archive.

## T004 — Archive manifest reader — COMPLETE

- Read `package/package.json` from `.tgz` bytes without extracting to disk.
- Bound manifest reads.
- Validate package identity against the selected package.

## T005 — Content-addressed cache — COMPLETE FOR CF-01 CONTRACT

- Compute SHA-256 over exact archive bytes.
- Store objects by digest.
- Verify cache objects by recomputing the digest.
- Reject malformed lockfile digests before cache lookup.

Atomic temp-write/rename hardening is explicitly deferred as a quality follow-up.

## T006 — Dependency resolver — COMPLETE

- Resolve roots and transitive dependencies.
- Choose highest stable patch for `major.minor.x`.
- Exclude prereleases from wildcard selection.
- Detect incompatible concrete-version requests.
- Resolve constraints to concrete versions before compatibility comparison.

## T007 — Deterministic lockfile — COMPLETE

- Define lock schema v1.
- Sort roots and packages deterministically.
- Serialize byte-stably.
- Record dependency constraints, archive digests, and source provenance.
- Reject unsupported lock schemas.

## T008 — CLI vertical slice — PARTIAL

- [x] Implement `commandf pkg resolve`.
- [x] Implement `commandf pkg verify`.
- [x] Use public FHIR registries by default with optional local mirror.
- [x] Support multiple root package requests.
- [ ] Add dedicated integration coverage for CLI failure/exit behavior.

## T009 — CI — PARTIAL

- [x] Add fmt, clippy, and test workflow.
- [x] Keep unit tests independent of external services.
- [x] Resolve a pinned real FHIR package and verify its cache offline.
- [ ] Commit exact CI-generated Cargo.lock.
- [ ] Run Cargo CI gates with `--locked`.
- [ ] Remove temporary Cargo.lock export step.

## T010 — Review and convergence — PARTIAL

- [x] Run spec/plan/tasks consistency review.
- [ ] Retry CodeRabbit after rate limit reset.
- [ ] Run Qodo when available for this Draft/readiness state.
- [ ] Re-run convergence against exact readiness candidate.

CF-01 is not ready to merge while any readiness blocker remains.
