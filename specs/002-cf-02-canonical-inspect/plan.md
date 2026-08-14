# CF-02 Implementation Plan

Status: Implemented — convergence candidate

## Components

CF-02 does **not** add a new workspace crate. The early `commandf-artifact` split was exercised during implementation and then folded into `commandf-pkg` before convergence because the inspection path uses the same bounded archive, hashing, and package trust boundary and required no independent dependency graph.

- `commandf-pkg`: CF-01 lock/cache authority plus CF-02 deterministic package inspection modules.
- `commandf`: `inspect` CLI command and JSON output.

This keeps the committed CF-01 `Cargo.lock` valid and avoids a crate that would add no independently shipped boundary.

## Data model

`PackageInspection` records locked package identity, archive digest, and ordered resources.

`ResourceArtifact` records filename, resource type, optional id, optional canonical URL/version, exact resource-byte SHA-256, and StructureDefinition element addresses.

`ElementAddress` records `snapshot|differential`, exact `ElementDefinition.id`, and optional path/slice name. Canonical URL/version remain on the containing resource rather than being duplicated on every element.

## Parsing

- consume the exact package selected by `commandf.lock`;
- verify the CF-01 cache digest before reading, then independently recheck the archive SHA-256 inside the inspector;
- use bounded archive traversal;
- ignore package metadata, derived `.index.json`, examples, and non-resource auxiliary files;
- hash exact resource bytes before JSON reserialization;
- require expected string types for inspected identity fields;
- detect canonical duplicates deterministically;
- allow one canonical URL across distinct explicit versions;
- inspect existing StructureDefinition snapshot/differential arrays only;
- never synthesize element ids or snapshots.

## CLI

```text
commandf inspect <package@exact-version> --format json
```

Defaults reuse `commandf.lock` and `.commandf/cache`. Inspection performs no acquisition. Wildcard package requests are rejected because inspection is against an exact locked artifact.

## Determinism and safety

- resources sort by archive-relative filename before serialization;
- exact resource bytes and the package archive receive SHA-256 evidence;
- identical verified inputs serialize byte-identically;
- archive traversal is bounded to 512 MiB decompressed and 50,000 entries;
- an inspected resource is bounded to 64 MiB;
- canonical ambiguity, malformed identity fields, archive digest mismatch, and missing/duplicate StructureDefinition element ids fail explicitly.

## Tests

Regression coverage proves:

- `.index.json` is ignored as derived metadata and nested examples are excluded;
- resource ordering and hashes are deterministic;
- duplicate qualified canonical identities fail;
- distinct explicit versions may share a canonical URL;
- malformed identity-field types fail;
- archive digest mismatch fails before parsing;
- identical inputs produce byte-identical JSON;
- duplicate element ids fail per StructureDefinition view;
- slice-aware element ids and snapshot/differential view identity are preserved.

CI additionally resolves, verifies, and inspects published `hl7.fhir.r4.core@4.0.1` using locked dependencies.

## Gates

Existing locked Rust gates remain mandatory. The real-package smoke must resolve the published package, verify its cache object, inspect it, parse the emitted JSON, and prove at least one StructureDefinition has indexed elements.

## Non-goals

FHIR validation, snapshot generation, structural diff, breaking rules, terminology execution, dependency graph construction, mapping execution, and AI runtime.
