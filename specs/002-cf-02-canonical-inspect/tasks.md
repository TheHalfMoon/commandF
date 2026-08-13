# CF-02 Tasks

Status: Draft

## T001 — Verified archive read API
Expose the minimum `commandf-pkg` API to locate an exact locked package, verify its cache digest, and return its archive bytes. No network access.

## T002 — Artifact crate and inspection model
Add `commandf-artifact` as an immediately exercised crate with deterministic `PackageInspection`, `ResourceArtifact`, and `ElementAddress` models.

## T003 — Bounded package resource scanner
Scan verified archive resources under the CF-01 safety policy. Exclude package metadata, `.index.json`, examples, and auxiliary files from the conformance inventory.

## T004 — Resource identity and hashing
Parse resource type/id/canonical URL/version and hash exact archived JSON bytes. Reject malformed inspected identity fields.

## T005 — Canonical index
Build deterministic canonical lookup state. Allow one URL across distinct explicit versions; reject ambiguous duplicate qualified/unversioned identities.

## T006 — StructureDefinition element addressing
Index existing snapshot/differential element arrays using exact `ElementDefinition.id`; preserve path/slice metadata and reject missing/duplicate ids per view.

## T007 — Deterministic JSON
Define stable ordering/serialization and prove byte-identical output for identical inputs.

## T008 — CLI inspect
Implement `commandf inspect <package@exact-version> --format json` using lock/cache defaults and explicit non-zero failure behavior.

## T009 — Tests
Cover filtering, hashes, canonical duplicates/versions, element slices/reslices, malformed fields, offline behavior, deterministic bytes, and cache verification.

## T010 — Real-package smoke
Resolve a pinned published FHIR package, inspect it, and prove inspection succeeds against locked cached bytes.

## T011 — Review and convergence
Run locked CI, CodeRabbit when available, Qodo when available, and final Spec Kit convergence. Do not introduce CF-03 diff behavior.
