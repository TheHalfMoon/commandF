# CF-11G Tasks — Ecosystem Context Graph

Status: CLOSED_CANONICAL — effective only after this closeout PR passes exact-head gates/review and merges without content changes.

Tasks are dependency ordered. A task is complete only with executable evidence on the exact candidate state.

## Planning and governance

- [x] T001 — Reconcile slice identity without rewriting canonical CF-11 history.
  - Product identity: `CF-11G`.
  - Spec Kit sequence: `012`.
  - `CF-12` remains `commandf impact` and depends on CF-11G.
  - No CF-10/CF-06 production-oracle dependency is introduced.

- [x] T002 — Define the user-visible vertical slice.
  - Command: `commandf context --lock ... --cache ... --format json`.
  - Output: deterministic package/artifact/reference graph report.
  - Graph build is offline and evidence-only.

- [x] T003 — Close spec/plan/tasks consistency before implementation.
  - Recorded in `consistency.md` and made canonical through PR #20.

## Stack A — explicit resolved package-edge evidence

- [x] T010 — Introduce explicit lock schema v2 model and version-aware decoding.
- [x] T011 — Capture exact parent→child dependency edges during resolver traversal.
- [x] T012 — Prove lock v2 determinism and v1 compatibility.

Canonical Stack A evidence:

```text
PR #21 head:  40983be3bbd6aed098c18cae8f381cab0ed1826e
PR #21 merge: 4bc5df2cfcb9437e1d4d84b19bc7ddca556d6996
ci:                    32843594591 SUCCESS
cf06-oracle:           32843594634 SUCCESS
cf11-multi-version:    32843594609 SUCCESS
```

## Stack B — deterministic Context Graph library

- [x] T020 — Add library-owned Context Graph schema v1.
- [x] T021 — Build artifact nodes through the existing bounded CF-02 inspection boundary.
- [x] T022 — Implement StructureDefinition V1 reference extraction.
- [x] T023 — Implement ValueSet and CodeSystem V1 reference extraction.
- [x] T024 — Implement deterministic in-closure canonical target resolution.
- [x] T025 — Expose explicit extraction coverage.
- [x] T026 — Prove Context Graph byte determinism and graph invariants.

Canonical Stack B evidence:

```text
PR #22 head:  1dce479f7e2b548109e8d2b99a928353639be4b6
PR #22 merge: 8b93a04d573f182e902d21d42ad54180d36d5223
ci:                    32843690946 SUCCESS
cf06-oracle:           32843690704 SUCCESS
cf11-multi-version:    32843690836 SUCCESS
```

## Stack C — shipped `commandf context`

- [x] T030 — Add `commandf context` CLI command.
- [x] T031 — Enforce lock/cache fail-closed behavior at CLI boundary.
- [x] T032 — Add end-to-end graph fixtures.
- [x] T033 — Add exact-head deterministic CLI proof.

Canonical Stack C evidence:

```text
PR #23 final reviewed head: 1475a9d117f11dd5af3de6b118cd72fc8ccdfebd
PR #23 merge:               7579309d6298998a0bad47bac080be156b4d80df
canonical tree:             160df0b89642e2588fddc17b5daed517c3689857
ci:                         32844511038 SUCCESS
cf06-oracle:                32844511085 SUCCESS
cf11-multi-version-proof:   32844511014 SUCCESS
cf11g-context-proof:        32844511055 SUCCESS
```

Deterministic output evidence:

```text
CF11G_CONTEXT_SHA256=cbc08088a858ca12af0a2a773be5f4b02a03bc099442e59f7300f5edaca069c0
artifact id: 9561791650
artifact digest: sha256:f4134aef0acfd3dff671d6219ee3a2d80dd73a5dbb33312c2ff97a9135f5b421
```

## Regression, review, and convergence

- [x] T040 — Run mandatory workspace gates on the exact final implementation head.
  - `cargo fmt --all -- --check` covered by final `ci`.
  - Clippy with warnings denied covered by final `ci`.
  - workspace tests covered by final `ci`.

- [x] T041 — Preserve existing workflow gates.
  - `ci` — SUCCESS on `1475a9d117f11dd5af3de6b118cd72fc8ccdfebd`.
  - `cf06-oracle` — SUCCESS on the same head.
  - `cf11-multi-version-proof` — SUCCESS on the same head.
  - `cf11g-context-proof` — SUCCESS on the same head.
  - real FHIR smoke and CF-08/CF-09 security regressions remain inside the successful `ci` job.

- [x] T042 — Independent review.
  - Every substantive finding actually returned on PRs #21–#23 is fixed or explicitly rejected against the frozen Spec 012 contract with CodeRabbit confirmation and resolved threads.
  - PR #20 review availability/rate-limit truth is recorded; no PASS is invented.
  - Qodo was not observed connected/available; no Qodo PASS is invented.

- [x] T043 — Run convergence pass.
  - final planning/Stack A/B/C heads and merge commits recorded in `convergence.md`;
  - lock schema migration evidence recorded;
  - Context Graph repeat-equality SHA-256 and artifact digest recorded;
  - reviewer findings and availability limitations dispositioned explicitly;
  - no untracked CF-11G implementation blocker remains;
  - `CF-12` becomes eligible only after this closeout PR itself passes exact-head configured gates/review and merges without content changes.

## Hard sequencing rules

1. T010–T012 preceded graph consumption because schema-v1 does not contain sufficient exact edge evidence.
2. T020 preceded extractor tasks.
3. T021 preceded graph-wide canonical resolution.
4. T022/T023 preceded T024.
5. T020–T026 preceded CLI shipping.
6. T030–T033 preceded final regression/review/convergence.
7. `CF-12 commandf impact` MUST NOT begin implementation until this T043 closeout record passes exact-head PR qualification and becomes canonical on `main`.
8. The blocked external HL7 maintainer path does not alter CF-11G closure and must not be used to change CF-06 production semantics implicitly.
