# CF-06 Tasks

Status: Approved for implementation

- [ ] **T001 — Oracle provenance.** Pin official HL7 core release `6.10.2`, source commit `d06577dbc5c62c74a2a8823fbc4830a3024d5b0b`, and validator artifact SHA-256 evidence; document upgrade as a later explicit contract change.
- [ ] **T002 — Java adapter scaffold.** Add isolated `tools/hl7-oracle/` Maven project with exact HL7 dependencies and no Rust workspace coupling.
- [ ] **T003 — Structured HL7 extraction.** Use `ComparisonSession` / `StructureDefinitionComparer` / public `StructuralMatch` and `ValidationMessage` objects; no renderer HTML and no reflection/private-node dependency.
- [ ] **T004 — Deterministic adapter JSON.** Add schema-v1 commandF-owned oracle identity, states, left/right identity, sorted/de-duplicated public messages, trailing newline, and no generated ids/dates/host paths.
- [ ] **T005 — Adapter regressions.** Cover self-equivalence, cardinality/type/binding/mustSupport observations, invalid snapshots, exact provenance, and byte determinism.
- [ ] **T006 — Rust oracle model.** Add typed adapter-input and `OracleDivergenceReport` models with exact schema/version/source validation and evidence bounds.
- [ ] **T007 — CF-03 pair reuse.** Expose/reuse CF-03 deterministic matched StructureDefinition pairs without inventing a second matching algorithm.
- [ ] **T008 — Reconciliation.** Implement `agreement`, `commandf_only`, `authority_only`, `both_changed`, and `uncomparable` evidence relationships while embedding the complete unmodified CF-03 report.
- [ ] **T009 — Hardened process boundary.** Explicit adapter path, no shell, per-pair timeout, bounded stdout/stderr, private temporary inputs, cleanup, and fail-closed nonzero/malformed behavior.
- [ ] **T010 — `commandf oracle` CLI.** Add exact two-state lock/cache inputs, explicit oracle adapter path, JSON output, and no package acquisition.
- [ ] **T011 — Rust/CLI regressions.** Cover valid/invalid adapter reports, wrong provenance, timeout/nonzero/malformed/oversized output, corrupted caches, deterministic report bytes, and offline execution.
- [ ] **T012 — Official-oracle CI.** Build/test adapter at exact `6.10.2` and run a real R4 self-equivalence oracle smoke while preserving all existing CF-01..03 Rust gates.
- [ ] **T013 — Review reconciliation.** Request CodeRabbit/Qodo on the first green implementation candidate, verify each finding, fix only valid issues, and record exact reviewer truth.
- [ ] **T014 — Convergence.** Reconcile spec/plan/tasks, add `convergence.md`, prove exact-final-head Rust + Java + real oracle gates, keep PR Draft, and do not start CF-07.
