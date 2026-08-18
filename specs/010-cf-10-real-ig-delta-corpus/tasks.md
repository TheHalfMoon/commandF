# CF-10 Tasks — Public Real-IG Delta Corpus

Status: implementation in progress / foundation, eligibility, manifest, and attestation gates complete

## Foundation reconciliation gate

- [x] CF-11 multi-version package graph is canonical on `main` at merge commit `5cb1a4c3445c0ebd86654cfb467a5e008e801c3e`.
- [x] Canonical CF-11 was merged into this frozen CF-10 branch by reconciliation merge `5ec463f0ae53b76f9c2c151335d98598b53e5abc`.
- [x] The five pre-existing CF-10 paths remained byte-identical across reconciliation before authorization updates.
- [x] Frozen selection remains exactly: US Core 8.0.1→9.0.0, IPS 1.1.0→2.0.1, mCODE 3.0.0→4.0.0.
- [x] Rerun the exact same six package states through the existing digest-discovery workflow on the reconciled foundation.
- [x] Review the resulting artifact before authorizing semantic corpus execution.

Eligibility evidence: `cf10-digest-discovery` run `31890014888`, reviewed artifact `9248341586`, with all six states independently A/B-attested. Exact-head rerun `31890859039` also passed.

## Selection and provenance

- [x] T001 Freeze anti-cherry-picking eligibility rules before result discovery.
- [x] T002 Freeze corpus v1 families: US Core, IPS, mCODE.
- [x] T003 Freeze exact candidate version pairs: US Core 8.0.1→9.0.0, IPS 1.1.0→2.0.1, mCODE 3.0.0→4.0.0.
- [x] T004 Record publication/change/rights evidence for every case in donor/provenance metadata.
- [ ] T005 Confirm repository mode is metadata-only and no package/terminology payload is committed.

## Digest discovery

- [x] T006 Add guarded exact-version digest-discovery workflow/tool.
- [x] T007 Resolve every selected package/version into independent cache A and cache B.
- [x] T008 Verify both caches with CF-01.
- [x] T009 Require byte/digest equality between independent resolutions.
- [x] T010 Record package archive SHA-256 and byte size in discovery artifact.
- [x] T011 Review discovery artifact and freeze digests/sizes in canonical corpus manifest.

## Manifest implementation

- [x] T012 Add typed corpus schema-v1 model.
- [x] T013 Add bounded pre-decode manifest input validation.
- [x] T014 Validate unique lexicographically ordered case ids.
- [x] T015 Validate exact package/version, R4 4.0.1, SHA-256, positive archive size, rights and publication evidence.
- [x] T016 Reject unsupported schema/rights/oracle modes and malformed evidence.
- [x] T017 Add deterministic JSON round-trip tests.

## Package attestation

- [x] T018 Add reusable before/after archive attestation against manifest digest and size.
- [x] T019 Add digest mismatch and size mismatch fail-closed tests.
- [x] T020 Preserve CF-01 cache verification as mandatory authority.

T020 is enforced in code: attestation verifies the complete supplied lock graph before selecting/reading the target state; a corrupt or missing unrelated lock entry blocks attestation.

## Corpus evaluator

- [ ] T021 Reuse CF-03 structural diff for every case.
- [ ] T022 Reuse CF-04 compatibility classification for every case.
- [ ] T023 Reuse CF-07 terminology evidence for every case.
- [ ] T024 Reuse CF-06 oracle only for changed matched StructureDefinitions.
- [ ] T025 Add deterministic per-case aggregate summary without changing underlying semantic rules.
- [ ] T026 Represent operational/unsupported/divergence states explicitly; never silently skip a selected case.

## Execution surface

- [x] T027 Decide and document one execution surface: narrow `commandf corpus run` CLI only.
- [ ] T028 Implement only the selected surface.
- [ ] T029 Keep acquisition and semantic evaluation internally separated by digest verification.

## Real corpus evidence

- [ ] T030 Run all three frozen cases from a clean environment.
- [ ] T031 Upload raw structural/compatibility/terminology/oracle reports as short-retention CI artifacts.
- [ ] T032 Run the full corpus a second time from a clean cache.
- [ ] T033 Require byte-identical deterministic summary between the two runs.
- [ ] T034 Freeze exact result digest/count regression evidence only after the two-run proof.

## CI and security

- [ ] T035 Preserve current `ci` workflow gates through the exact final head.
- [ ] T036 Preserve current `cf06-oracle` workflow gates through the exact final head.
- [ ] T037 Add bounded dedicated real-corpus workflow/job.
- [ ] T038 Keep quoted argv/no `eval`; corpus metadata must never become shell code.
- [ ] T039 Bound manifest bytes/case count/result bytes and fail closed on malformed input.
- [ ] T040 Prove no upstream IG NPM tarball or terminology payload enters git history.

## Review and convergence

- [ ] T041 Request CodeRabbit review when available and disposition substantive findings.
- [ ] T042 Request Qodo review when available and disposition substantive findings.
- [ ] T043 Request configured independent/Codex review when available; no unavailable PASS may be invented.
- [ ] T044 Reconcile `spec.md`, `plan.md`, `tasks.md`, donor record, manifest, and implementation truth.
- [ ] T045 Add `convergence.md` with exact final-head CI/reviewer evidence rule.
- [ ] T046 Keep PR open/unmerged until exact final head passes ordinary CI, oracle CI, multi-version proof, and dedicated corpus CI.
