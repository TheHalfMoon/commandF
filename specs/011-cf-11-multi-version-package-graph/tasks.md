# CF-11 Tasks — Multi-Version Package Graph

Status: implementation, reviewer reconciliation, and implementation-head evidence complete; final documentation-head gates pending

- [x] T001 — Record CF-10 comprehensive eligibility evidence and freeze CF-10 as `BLOCKED_BY_FOUNDATION` without changing cases.
- [x] T002 — Audit current resolver, lock schema, CLI locked-package selection, and terminology ambiguity boundaries.
- [x] T003 — Specify exact package identity `(name, concrete version)` and explicit non-goals.
- [x] T004 — Change resolver selected closure to exact identities while preserving request-local version selection.
- [x] T005 — Replace the old same-name/different-version failure regression with positive multi-version graph coverage.
- [x] T006 — Add same-identity dedup, wildcard/exact coexistence, root-order determinism, and cycle regressions.
- [x] T007 — Preserve schema-v1 lock ordering and cache verification; do not add implicit edge semantics.
- [x] T008 — Add bounded real-network proof for a previously failing CF-10 state.
- [x] T009 — Run Format, locked Clippy, full workspace tests, CF-08/CF-09 security gates, real FHIR smoke, and CF-06 oracle gates.
- [x] T010 — Reconcile Codex, Qodo, CodeRabbit, and Greptile review truth on the implementation candidate; no substantive returned finding remains unresolved and unavailable/rate-limited reviewers are recorded without invented PASS.
- [x] T011 — Write convergence truth with exact implementation head/run identities and real multi-version lock evidence; final documentation head must rerun all configured CF-11 gates.
- [ ] T012 — Merge only after final exact-head gates; then reconcile CF-10 and rerun the same frozen six-state eligibility sweep before semantic execution.
