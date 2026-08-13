# CF-02 Implementation Plan

Status: Draft

## Components

Add one exercised Rust crate, `commandf-artifact`.

- `commandf-pkg`: locked package/cache/archive authority.
- `commandf-artifact`: deterministic FHIR resource inspection.
- `commandf`: `inspect` CLI command and JSON output.

## Data model

`PackageInspection` records locked package identity, archive digest, and ordered resources.

`ResourceArtifact` records filename, resource type, optional id, optional canonical URL/version, exact resource-byte SHA-256, and StructureDefinition element addresses.

`ElementAddress` records canonical URL/version, `snapshot|differential`, exact `ElementDefinition.id`, and optional path/slice name.

## Parsing

- consume verified cached archive bytes from CF-01;
- use bounded archive traversal;
- ignore package metadata, derived `.index.json`, examples, and non-resource auxiliary files;
- hash exact resource bytes before JSON reserialization;
- require expected string types for inspected identity fields;
- detect canonical duplicates deterministically;
- inspect existing StructureDefinition snapshot/differential arrays only;
- never synthesize element ids or snapshots.

## CLI

```text
commandf inspect <package@exact-version> --format json
```

Defaults reuse `commandf.lock` and `.commandf/cache`. JSON ordering and bytes are deterministic.

## Tests

Cover deterministic output, offline behavior, filtering, exact hashing, canonical identities/duplicates, multiple explicit canonical versions, snapshot/differential addresses, slices/reslices, missing/duplicate element ids, malformed identity fields, and one real-package smoke.

## Gates

Existing locked Rust gates remain mandatory. Add an integration smoke that resolves a published package and then inspects it.

## Non-goals

FHIR validation, snapshot generation, diff, breaking rules, terminology execution, dependency graph, mapping execution, and AI runtime.
