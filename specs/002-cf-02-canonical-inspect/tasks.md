# CF-02 Tasks

Status: Implementation complete — final convergence gate

- [x] **T001 — Locked cache consumption.** Reuse CF-01 `Lockfile` and `PackageCache`; require an exact locked package, verify its cache digest, and perform no acquisition.
- [x] **T002 — Inspection model.** Implement `PackageInspection`, `ResourceArtifact`, and `ElementAddress`. The early standalone artifact crate was folded into `commandf-pkg`; no new workspace crate or dependency graph remains.
- [x] **T003 — Bounded scanner.** Scan package-root JSON with bounded decompression/entry/resource limits; ignore package metadata, `.index.json`, nested examples, and auxiliary files.
- [x] **T004 — Identity and hashing.** Record resourceType/id/url/version and SHA-256 over exact archived resource bytes; malformed inspected identity types fail.
- [x] **T005 — Canonical rules.** Reject duplicate qualified/unversioned canonical identities; allow one URL across distinct explicit versions.
- [x] **T006 — StructureDefinition addressing.** Preserve exact `ElementDefinition.id`, view, path, and sliceName; reject missing/duplicate ids per view.
- [x] **T007 — Deterministic JSON.** Stable ordering and byte-identical serialization for identical verified inputs.
- [x] **T008 — CLI.** `commandf inspect <package@exact-version> --format json`, offline and fail-closed.
- [x] **T009 — Contract tests.** Cover filtering, hashes, canonical semantics, malformed identity fields, digest mismatch, deterministic bytes, duplicate element ids, and slice/view preservation.
- [x] **T010 — Real-package smoke.** Resolve, verify, and inspect `hl7.fhir.r4.core@4.0.1`; assert a StructureDefinition with indexed elements exists.
- [ ] **T011 — Final review/convergence.** Record exact-head evidence, record CodeRabbit/Qodo availability truth, keep PR Draft, and do not introduce CF-03 behavior.
