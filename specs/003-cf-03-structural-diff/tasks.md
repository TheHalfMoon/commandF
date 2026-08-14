# CF-03 Tasks

Status: Implementation and convergence complete — founder review candidate

- [x] **T001 — Shared bounded resource scanner.** CF-02 package-root JSON scanning is factored into one internal helper used by inspect and diff with unchanged filtering and archive/resource bounds.
- [x] **T002 — Diff models.** `StructuralDiffReport`, package evidence, resource keys, structural change kinds, and change records are deterministic and contain no severity/compatibility fields.
- [x] **T003 — Two-state package loading.** `commandf diff` selects exactly one package version from each explicit lock, verifies each CF-01 cache object, reads the content-addressed archives, and independently rechecks archive digests during structural parsing.
- [x] **T004 — Deterministic resource matcher.** Unique canonical URLs match logically; multi-version URL groups use exact `url|version`; non-canonical resources fall back through unique `resourceType/id` then filename; ambiguity and duplicate archive filenames fail closed.
- [x] **T005 — StructureDefinition shape parser.** Selected structural metadata plus snapshot/differential element objects are compared using exact `ElementDefinition.id` addresses.
- [x] **T006 — Structural normalization.** Objects are canonicalized; representation/condition, type profile/targetProfile/aggregation, and constraint ordering are normalized while potentially meaningful ordering remains preserved.
- [x] **T007 — Typed diff engine.** Resource/view/element additions/removals and selected resource/StructureDefinition field changes are emitted without CF-04 compatibility classification.
- [x] **T008 — Deterministic serialization.** Changes are stable-sorted and identical inputs serialize byte-identically; self-diff is empty.
- [x] **T009 — CLI.** `commandf diff <package-name>` uses explicit before/after lock/cache paths and `--format json`; it performs no acquisition.
- [x] **T010 — Synthetic contract tests.** Coverage includes no-op/determinism, unique and multi-version canonical matching, ambiguity failure, duplicate filename failure, view/element additions and removals, cardinality/type/binding/slicing/fixed changes, editorial exclusion, and set-like ordering normalization.
- [x] **T011 — Real self-diff smoke.** CI resolves/verifies `hl7.fhir.r4.core@4.0.1` into two independent states, runs the real CLI, and proves `changes` is empty.
- [x] **T012 — Review/convergence.** Implementation head `2a4ece32313ba7b92b8dde038bf5e231a12b2dff` passed run `31759991866` (Format, Clippy `--locked`, Test `--locked`, real registry + inspect + two-state diff smoke). CodeRabbit status was success but its actual review was skipped because PR #4 is Draft; no submitted review or inline thread existed, and no Qodo result was present. Convergence is recorded in `convergence.md`. PR remains Draft and CF-04 severity logic is not introduced.
