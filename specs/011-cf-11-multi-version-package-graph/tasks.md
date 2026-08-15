# CF-11 Tasks — Multi-Version Package Graph

Status: in progress

- [x] T001 — Record CF-10 comprehensive eligibility evidence and freeze CF-10 as `BLOCKED_BY_FOUNDATION` without changing cases.
- [x] T002 — Audit current resolver, lock schema, CLI locked-package selection, and terminology ambiguity boundaries.
- [x] T003 — Specify exact package identity `(name, concrete version)` and explicit non-goals.
- [ ] T004 — Change resolver selected closure to exact identities while preserving request-local version selection.
- [ ] T005 — Replace the old same-name/different-version failure regression with positive multi-version graph coverage.
- [ ] T006 — Add same-identity dedup, wildcard/exact coexistence, root-order determinism, and cycle regressions.
- [ ] T007 — Preserve schema-v1 lock ordering and cache verification; do not add implicit edge semantics.
- [ ] T008 — Add bounded real-network proof for a previously failing CF-10 state.
- [ ] T009 — Run Format, locked Clippy, full workspace tests, CF-08/CF-09 security gates, real FHIR smoke, and CF-06 oracle gates.
- [ ] T010 — Request Codex, Qodo, CodeRabbit, and Greptile review on the exact implementation candidate; disposition every substantive returned finding.
- [ ] T011 — Write convergence truth with exact head/tree/run identities and real multi-version lock evidence.
- [ ] T012 — Merge only after exact-head gates/reviews; then reconcile CF-10 and rerun the same frozen six-state eligibility sweep before semantic execution.
