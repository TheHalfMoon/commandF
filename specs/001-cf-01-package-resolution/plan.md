# CF-01 Implementation Plan

Status: Draft

## Goal

Ship the smallest production-oriented package-resolution vertical slice needed by later commandF artifact analysis.

## Architecture

CF-01 uses two Rust crates only:

- `commandf-pkg`: package identity, dependency resolution, deterministic lockfile, content-addressed cache, and a package-source abstraction.
- `commandf`: CLI consuming `commandf-pkg` through `pkg` subcommands.

No crate may exist without an exercised consumer.

## PackageSource boundary

Resolution logic depends on a narrow `PackageSource` trait rather than transport details. Unit tests use an in-memory source. Package acquisition adapters may provide bytes and metadata but may not override resolution semantics.

## Resolution rules

1. Root requests and transitive dependencies are normalized into package name plus version constraint.
2. Exact versions are selected exactly.
3. `major.minor.x` selects the highest stable matching patch version.
4. Prereleases are excluded from wildcard selection unless explicitly requested by an exact version.
5. Multiple incompatible selected concrete versions for one package fail closed.
6. Unsupported aliases, local paths, git references, and broad semver ranges fail explicitly in CF-01.
7. Every accepted archive must declare the expected package name and version in `package/package.json`.

## Cache

Archives are stored by SHA-256 digest. Verification recomputes the digest and fails on corruption. Package metadata is read from the archive without extracting entries into the caller worktree.

## Lockfile

`commandf.lock` is deterministic and machine-readable. Stable ordering is package name then version. It records schema version, root requests, exact package identity, archive digest, source provenance, and declared dependency constraints.

Identical pinned inputs must serialize to identical bytes.

## CLI

Initial commands:

```text
commandf pkg resolve <package@version> [--lock commandf.lock]
commandf pkg verify [--lock commandf.lock]
```

`resolve` writes the deterministic lockfile. `verify` performs offline cache verification and returns non-zero on missing or corrupt content.

## Trust and security

- package metadata and archives are untrusted input;
- archive parsing is bounded and does not extract entries to disk;
- package identity mismatch is fatal;
- cache identity is digest-based;
- errors never silently downgrade to compatible or verified state.

## Tests

Required before readiness:

- exact-version resolution;
- transitive dependency resolution;
- deterministic byte-identical lockfile serialization;
- wildcard highest-stable-patch selection;
- prerelease exclusion for wildcard resolution;
- incompatible-version conflict;
- unsupported constraint rejection;
- missing manifest;
- manifest identity mismatch;
- corrupted cache detection;
- offline verify;
- one real published FHIR package end-to-end smoke test outside unit-test isolation.

## CI

Every candidate head must pass:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Unit tests must not require external services.

## Non-goals

No resource validation, snapshot generation, canonical indexing, structural diff, SARIF, mapping execution, agent runtime, package publishing, authenticated registry, or custom commandF registry.

## Definition of done

CF-01 is complete only when the exact PR head demonstrates deterministic lock generation, offline cache verification, explicit conflict failure, and successful resolution of at least one real published FHIR package.