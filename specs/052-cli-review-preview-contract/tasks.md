# UX-01 — Bounded Review Preview Tasks

Status: CANDIDATE; marked tasks denote code written or locally checked, never canonical adoption.

- [x] T001 Specify limited, truthful preview scope, supported inputs, outputs, size boundaries and error/exit contract.
- [x] T002 Implement `review-preview` facade with existing typed deterministic check and impact implementations only.
- [x] T003 Add bounded, non-atomic, unsigned JSON envelope and defer output until both calculations finish.
- [x] T004 Add focused regression tests for policy pass/fail, schema rejection, output replacement and invalid CLI usage.
- [x] T005 Execute genuine Windows Rust formatting and focused tests on the candidate code (4/4 PASS before rustfmt-only commit); retain the actual commands and head.
- [x] T010 Implement an alternate SARIF projection with bounded embedded graph-impact provenance using the existing checked SARIF format (candidate only).
- [x] T011 Add SARIF failure-policy, pass-policy output and corrupt-cache negative tests (candidate code; actual execution pending).
- [ ] T006 Prove exact updated head passes Linux CI, including existing CLI and assurance regressions. Do not count pending as success.
- [ ] T007 Obtain actual independent Code Owner approval; neither bot comments nor automated delegation counts.
- [ ] T008 Qualify and merge via a normal merge commit only after required rules permit it.
- [ ] T009 Verify fresh canonical `main` and publish truthful CLI support status; distinguish preview from future full `commandf review`.

Blocked on separate product work, not completed by UX-01: full consumer-contract semantics, atomic snapshot receipt, signed provenance, Studio, benchmark qualification and release artifacts.
