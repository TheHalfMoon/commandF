# CF-09 Tasks — FSH Source Mapping

Status: converged / final review

- [x] T001 — Record pinned SUSHI donor/source-index provenance and no-copy boundary.
- [x] T002 — Add typed schema-v1 source-map model and deterministic JSON serialization.
- [x] T003 — Add bounded SUSHI `fsh-index.json` reader and required-field validation.
- [x] T004 — Reject duplicate `outputFile`, malformed line ranges, malformed paths, byte/entry overflow, and stale ranges beyond current source EOF.
- [x] T005 — Add canonical repository/FSH-root validation with traversal and symlink-escape rejection.
- [x] T006 — Reuse one persisted CF-05 CheckReport validation helper across CF-08 and CF-09.
- [x] T007 — Implement exact `after_filename -> outputFile` current-tree mapping with explicit unmapped states.
- [x] T008 — Add `commandf source-map` CLI with bounded inputs and optional atomic output publication.
- [x] T009 — Extend `github-annotations` to consume an optional matching source-map report.
- [x] T010 — Emit only proven `file`, `line`, and `endLine` properties; keep unmapped findings locationless.
- [x] T011 — Preserve CF-08 escaping, annotation caps, title/message bounds, overflow disclosure, and decision truth.
- [x] T012 — Add root Action optional `fsh-index` / `fsh-root` inputs without changing behavior when mapping is disabled.
- [x] T013 — Preserve Action exit 0/1/2; source-map/render failures become operational 1 after completed report publication.
- [x] T014 — Add source-map regressions for exact mapping, unmapped states, ambiguity, path safety, stale source, core bounds, persisted-map containment, determinism, and report mismatch.
- [x] T015 — Add Action wrapper/security regressions for quoted source-map paths, metacharacters, source-map on/off, and exit preservation.
- [x] T016 — Add deterministic public/synthetic FSH definition-range integration fixture with no PHI or controlled terminology.
- [x] T017 — Run exact implementation-head Format, locked Clippy, full workspace tests, Action security regressions, real R4 smoke, and real local Action source-map smoke (`3819f35116a5bf18070cc00453f34176b549688a`, run `31840014291`, SUCCESS).
- [x] T018 — Request CodeRabbit and Qodo reviews and record exact availability/result truth. CodeRabbit was rate-limited; Qodo returned no substantive result; no PASS claimed.
- [x] T019 — Request Codex Code Review with explicit security guidance and record result separately from Codex Security. No result returned; no PASS claimed.
- [x] T020 — Apply Codex Security diff-scan methodology and record product-scan availability truth. Actual Codex Security executor is not exposed in this host, so no product scan/PASS is claimed; three manually discovered security findings were fixed and regression-tested.
- [x] T021 — Check configured Ponytail/independent review availability. No Ponytail plugin/connector was available in this host; no PASS claimed.
- [x] T022 — Reconcile Spec Kit and add `convergence.md` with exact implementation evidence, security dispositions, and reviewer truth.
- [x] T023 — Run exact converged-docs CI and governance preparation (`79847a2bcb97566dff10f0de1186953998dcb68d`, run `31840724719`, SUCCESS), then require one final exact task-state-head revalidation before verdict.
- [x] T024 — Confirm CF-10 has not started before final CF-09 founder-review verdict; no CF-10 branch or PR was found at convergence-candidate time.
