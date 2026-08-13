# CF-02 Convergence Review

Status: Implementation converged — final exact-head gate pending
Date: 2026-08-14

## Scope result

CF-02 remains inside its authorized boundary: deterministic offline inspection of an exact FHIR package already locked and cached by CF-01.

It does not validate FHIR resources, generate snapshots, compute structural diffs, classify breaking changes, execute terminology, build the ecosystem graph, execute mappings, or introduce AI runtime authority. Those remain CF-03 or later work.

## Architecture reconciliation

The early implementation introduced a standalone `commandf-artifact` crate. Before convergence it was folded into `commandf-pkg` because inspection shares the CF-01 archive/hash/package trust boundary and required no independent dependency graph.

The final design therefore uses:

- `commandf-pkg` for lock/cache authority and deterministic inspection;
- `commandf` for the `inspect` CLI surface.

No new workspace crate remains and the committed CF-01 `Cargo.lock` remains valid under `--locked`.

## Contract achieved

`commandf inspect <package@exact-version> --format json` now:

- requires an exact package present in `commandf.lock`;
- performs no package acquisition;
- verifies the CF-01 cache object before reading it;
- independently rechecks the archive SHA-256 before parsing;
- rebuilds inventory from package-root FHIR JSON resources rather than trusting `.index.json`;
- excludes package metadata, nested examples, and auxiliary files;
- records exact resource-byte SHA-256 plus resourceType/id/url/version;
- treats canonical identity as URL with optional explicit version qualification;
- rejects duplicate qualified or duplicate unversioned canonical identities while allowing distinct explicit versions for one URL;
- inspects existing StructureDefinition snapshot/differential arrays only;
- preserves exact `ElementDefinition.id`, view, path, and sliceName;
- rejects missing/duplicate element ids per view;
- serializes deterministically.

## Safety bounds

- decompressed TAR traversal: 512 MiB maximum;
- archive entries: 50,000 maximum;
- one inspected resource: 64 MiB maximum;
- malformed inspected identity field types fail explicitly;
- digest mismatch fails before resource parsing.

## Deterministic test evidence

The CF-02 contract tests cover:

1. rebuilding inventory while ignoring derived `.index.json` and nested examples;
2. stable resource ordering and exact resource hashes;
3. duplicate qualified canonical rejection;
4. distinct explicit versions sharing one canonical URL;
5. malformed identity-field rejection;
6. archive digest mismatch before parsing;
7. byte-identical JSON for identical verified inputs;
8. duplicate `ElementDefinition.id` rejection per view;
9. slice-aware id and snapshot/differential preservation.

## Implementation candidate evidence

Implementation/test candidate:

`6cfd653f56d69aeebdcc6345a4e16ec755ae7f4b`

GitHub Actions run:

`31755422949`

Result:

- Format — PASS;
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` — PASS;
- `cargo test --locked --workspace --all-features` — PASS;
- real registry resolve/verify — PASS;
- real `commandf inspect hl7.fhir.r4.core@4.0.1 --format json` smoke — PASS;
- emitted JSON parsed successfully and contained at least one StructureDefinition with indexed elements.

## Reviewer evidence

### CodeRabbit

A manual review was requested on PR #3. CodeRabbit did not return a review result because the repository/account review limit was reached. No CodeRabbit review PASS is claimed. At the latest inspection there were no inline review threads.

Disposition: **UNAVAILABLE / RATE LIMITED**.

### Qodo

`/review` was requested on PR #3. No Qodo review result or finding was returned and no Qodo PASS is claimed.

Disposition: **NO EVIDENCE / NOT RETURNED**.

Reviewer unavailability does not replace deterministic CI evidence and is recorded explicitly rather than silently treated as success.

## Remaining gate

After this convergence documentation and task reconciliation are committed, run the full locked CI and real-package inspect smoke on the exact final head. Only an exact-head green result may close T011.

PR #3 must remain Draft and unmerged. No merge or auto-merge is authorized.

## Explicit deferrals

- FHIR resource validation and external validator/oracle comparison;
- snapshot generation;
- structural diff and breaking/risk classification;
- terminology expansion or binding execution;
- ecosystem dependency graph and blast radius;
- mapping analysis/execution and semantic loss analysis;
- AI/agent runtime features.
