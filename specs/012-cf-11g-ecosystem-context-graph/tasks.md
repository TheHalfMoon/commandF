# CF-11G Tasks — Ecosystem Context Graph

Status: implementation proven; final convergence gates pending

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
  - Record analysis in `consistency.md`.
  - Any contradiction discovered during review reopens this task.

## Stack A — explicit resolved package-edge evidence

- [x] T010 — Introduce explicit lock schema v2 model and version-aware decoding.
  - Preserve roots, packages, digests, source provenance, and declared manifest dependency constraints.
  - Add deterministic exact resolved dependency edge relation.
  - Existing commands continue accepting valid schema-v1 locks.
  - New resolver output writes schema v2.
  - Malformed or unsupported schema states fail closed.

- [x] T011 — Capture exact parent→child dependency edges during resolver traversal.
  - Record edge after concrete child selection and before expansion dedup short-circuit.
  - Preserve declared constraint on edge evidence.
  - Shared exact child identities remain one node with multiple parent edges.
  - Cycle closing edges are retained while exact-identity expansion remains bounded.

- [x] T012 — Prove lock v2 determinism and v1 compatibility.
  - Multi-version branch-local edge fixture.
  - Shared-child fixture.
  - Cycle fixture.
  - Equivalent root-order byte identity.
  - v1 read + verify regression.
  - Existing inspect/diff/check/terminology/oracle v1 behavior remains supported.

## Stack B — deterministic Context Graph library

- [x] T020 — Add library-owned Context Graph schema v1.
  - Deterministic package nodes.
  - Deterministic artifact nodes.
  - Package dependency edges.
  - Canonical reference edges.
  - Explicit target resolution state.
  - Explicit extraction coverage metadata.
  - Stable pretty-JSON bytes with trailing newline.

- [x] T021 — Build artifact nodes through the existing bounded CF-02 inspection boundary.
  - Verify each lock digest before reading archive bytes.
  - Preserve exact owner package identity, archive digest, filename, resource type, canonical URL/version, resource SHA.
  - No second unbounded archive path.

- [x] T022 — Implement StructureDefinition V1 reference extraction.
  - top-level `baseDefinition`;
  - differential `element[].type[].profile[]`;
  - differential `element[].type[].targetProfile[]`;
  - differential `element[].binding.valueSet`;
  - cover both profile and extension StructureDefinitions.

- [x] T023 — Implement ValueSet and CodeSystem V1 reference extraction.
  - ValueSet include/exclude `system` and imported `valueSet[]`;
  - CodeSystem `supplements`.

- [x] T024 — Implement deterministic in-closure canonical target resolution.
  - exact versioned unique target → `resolved`;
  - unique unversioned target → `resolved`;
  - no target → `external`;
  - multiple eligible targets → `ambiguous` with sorted candidate identities;
  - source canonical string retained exactly;
  - no network lookup or preferred-candidate heuristic.

- [x] T025 — Expose explicit extraction coverage.
  - Supported source resource types/extractor version.
  - Present-but-unsupported resource types sorted deterministically.
  - Unsupported types remain artifact nodes.

- [x] T026 — Prove Context Graph byte determinism and graph invariants.
  - Repeat build on identical lock/cache bytes is byte-identical.
  - Package/artifact/edge input-order permutations do not affect output.
  - Duplicate identical edges deduplicate deterministically.
  - No ambiguous target is serialized as resolved.

## Stack C — shipped `commandf context`

- [x] T030 — Add `commandf context` CLI command.
  - `--lock` path.
  - `--cache` path.
  - JSON-only format in V1.
  - Canonical JSON to stdout.
  - No package acquisition or registry access.

- [x] T031 — Enforce lock/cache fail-closed behavior at CLI boundary.
  - schema-v1 context request rejects with stable migration diagnostic;
  - missing archive rejects;
  - corrupted archive rejects;
  - malformed graph-required resource input rejects according to existing bounded parser policy;
  - runtime diagnostic sanitization remains intact.

- [x] T032 — Add end-to-end graph fixtures.
  - exact multi-version package edges;
  - StructureDefinition profile + extension edges;
  - ValueSet/CodeSystem edges;
  - resolved/external/ambiguous canonical states;
  - unsupported resource type coverage.

- [x] T033 — Add exact-head deterministic CLI proof.
  - run `commandf context` twice from identical pinned fixture inputs;
  - compare output bytes exactly;
  - retain SHA-256 evidence in CI logs or artifact metadata.

## Regression, review, and convergence

- [ ] T040 — Run mandatory workspace gates on the exact final stack head.
  - `cargo fmt --all -- --check`.
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
  - `cargo test --workspace --all-features`.

- [ ] T041 — Preserve existing workflow gates.
  - `ci`.
  - `cf06-oracle`.
  - `cf11-multi-version-proof`.
  - existing real FHIR smoke and CF-08/CF-09 security regressions.

- [ ] T042 — Independent review.
  - CodeRabbit review when available.
  - Qodo review when connected/available.
  - Every substantive finding dispositioned on the exact candidate head.

- [ ] T043 — Run convergence pass.
  - Record final heads/trees/workflow runs.
  - Record lock schema migration evidence.
  - Record Context Graph output SHA-256/repeat equality.
  - Append every remaining gap as a task or explicit deferral.
  - Confirm `CF-12` is either eligible or blocked by an explicit remaining CF-11G gap.

## Hard sequencing rules

1. T010–T012 precede graph consumption because schema-v1 does not contain sufficient exact edge evidence.
2. T020 precedes extractor tasks.
3. T021 precedes graph-wide canonical resolution.
4. T022/T023 precede T024 because target resolution consumes extracted reference edges.
5. T020–T026 precede CLI shipping.
6. T030–T033 precede final regression/review/convergence.
7. `CF-12 commandf impact` MUST NOT begin implementation until T043 closes CF-11G canonical convergence.
8. The blocked external HL7 maintainer path does not block these independent tasks and must not be used to alter CF-06 production semantics implicitly.
