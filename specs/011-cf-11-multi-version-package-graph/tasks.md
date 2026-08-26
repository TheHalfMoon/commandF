# CF-11 Tasks — Multi-Version Package Graph

Status: CLOSED_CANONICAL — PR #13 merged with exact-head gates green; the post-merge CF-10 six-state eligibility rerun completed on the canonical CF-11 foundation with immutable run/head/artifact-digest identity recorded below.

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
- [x] T012 — Merge only after final exact-head gates; then reconcile CF-10 and rerun the same frozen six-state eligibility sweep before semantic execution. Completed by the retained CF-10 full-corpus reproof identified below; the later CF-06 production-oracle failure is a separate gate.

## Canonical closeout evidence

```text
PR: #13
final candidate head: 0c2519202372e6d9d4f7da08fc23e6b012caff9d
merge commit: 5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
final candidate tree: c81fa47a31a08a7d3bf6af849a76f166de9f73c7
ci: 31889322720 SUCCESS
cf06-oracle: 31889322723 SUCCESS
cf11-multi-version-proof: 31889322717 SUCCESS

post-merge six-state evidence:
CF-10 PR: #11
CF-10 evidence head: 5fe10d9859407272acf6649fc3e868d3eb2fbd12
CF-10 base / CF-11 merge commit: 5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
workflow: cf10-real-corpus
run: 31916124080
unchanged package states: C001/C002/C003 before + after = 6 / 6 attested
artifact: cf10-real-corpus-evidence
artifact id: 9255732702
artifact digest: sha256:9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612
```

Run `31916124080` has an overall `failure` conclusion because final enforcement preserves the separately governed pinned CF-06 oracle failures for C001/C002. Before that final enforcement failure, the run completed the six package-state acquisition/closure evidence, deterministic A/B work, retained closure binding, repository-boundary verification, and evidence upload. The GitHub-recorded artifact is now expired, but its run/head/digest identity remains immutable evidence of the retained result.

The later CF-10 production gate is separate from CF-11 foundation completion. CF-10 remains governed by its own CF-06 oracle contract and frozen-corpus evidence requirements.
