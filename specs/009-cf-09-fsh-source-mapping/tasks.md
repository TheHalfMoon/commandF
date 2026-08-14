# CF-09 Tasks — FSH Source Mapping

Status: implementation in progress

- [ ] T001 — Record pinned SUSHI donor/source-index provenance and no-copy boundary.
- [ ] T002 — Add typed schema-v1 source-map model and deterministic JSON serialization.
- [ ] T003 — Add bounded SUSHI `fsh-index.json` reader and required-field validation.
- [ ] T004 — Reject duplicate `outputFile`, malformed line ranges, malformed paths, and entry-count overflow.
- [ ] T005 — Add canonical repository/FSH-root validation with traversal and symlink-escape rejection.
- [ ] T006 — Reuse one persisted CF-05 CheckReport validation helper across CF-08 and CF-09.
- [ ] T007 — Implement exact `after_filename -> outputFile` current-tree mapping with explicit unmapped states.
- [ ] T008 — Add `commandf source-map` CLI with bounded inputs and optional atomic output publication.
- [ ] T009 — Extend `github-annotations` to consume an optional matching source-map report.
- [ ] T010 — Emit only proven `file`, `line`, and `endLine` properties; keep unmapped findings locationless.
- [ ] T011 — Preserve CF-08 escaping, annotation caps, title/message bounds, overflow disclosure, and decision truth.
- [ ] T012 — Add root Action optional `fsh-index` / `fsh-root` inputs without changing behavior when mapping is disabled.
- [ ] T013 — Preserve Action exit 0/1/2; source-map/render failures become operational 1 after completed report publication.
- [ ] T014 — Add synthetic source-map regressions for exact mapping, unmapped states, ambiguity, path safety, stale source, determinism, and report mismatch.
- [ ] T015 — Add Action wrapper/security regressions for quoted source-map paths, metacharacters, source-map on/off, and exit preservation.
- [ ] T016 — Add deterministic public/synthetic FSH definition-range integration fixture with no PHI or controlled terminology.
- [ ] T017 — Run exact-head Format, locked Clippy, full workspace tests, Action security regression, and real existing R4 smoke.
- [ ] T018 — Request CodeRabbit and Qodo reviews and disposition every substantive finding.
- [ ] T019 — Request Codex Code Review with explicit security guidance and record result separately from Codex Security.
- [ ] T020 — Run/record Codex Security when repository enablement is available; absence is non-blocking and MUST NOT be called PASS.
- [ ] T021 — Run configured Ponytail/independent code-review lane when available and record exact result.
- [ ] T022 — Reconcile Spec Kit and add `convergence.md` with exact implementation evidence and reviewer truth.
- [ ] T023 — Run exact-final-docs-head CI and governance checks; keep PR Draft/open/unmerged with auto-merge disabled.
- [ ] T024 — Confirm CF-10 has not started before final CF-09 founder-review verdict.
