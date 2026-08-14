# CF-08 Convergence Record

Status: Candidate — exact final documentation-head CI required before founder-review certification

## Exact stack identity

```text
repository: TheHalfMoon/commandF
PR: #9
branch: feat/cf-08-github-action-annotations
base branch: feat/cf-05-sarif-ci-gate
base SHA: 9f158f1dcb7d04bc5ee582eabd1ee8dd93bd5019
```

CF-06 and CF-07 are not dependencies. CF-09 has not started and is not authorized by this record.

## Delivered product boundary

CF-08 adds a GitHub delivery layer above CF-05 without adding compatibility authority:

- deterministic bounded GitHub workflow-command annotation projection;
- `BREAKING -> error`, `RISKY -> warning`, `ADDITIVE -> notice`;
- exact reuse of CF-05 direction selection;
- fail-on independence from annotation visibility;
- persisted CF-05 decision consistency revalidation;
- workflow-command escaping for finding-controlled data/properties;
- explicit no-source-location boundary before CF-09;
- bounded title/message/report inputs;
- repository-root composite `action.yml`;
- pinned source-backed Linux build using Rust 1.97.1 and Cargo `--locked`;
- pure quoted-argv Action run wrapper;
- preservation of CF-05 exit `0/1/2`;
- complete JSON report retention on policy failure;
- Action outputs `report-path`, `exit-code`, and `passed`.

No FSH mapping, repository file/line annotations, fabricated SARIF locations, code-scanning upload, Check Runs mutation, repository write, graph behavior, terminology behavior, or AI authority is introduced.

## Annotation safety and bounds

Workflow-command data escapes `%`, CR, and LF. Property values additionally escape `:` and `,`. Regression coverage includes finding text containing an injected `::error` sequence and proves it remains data.

CF-08 emits no explicit `file`, `line`, `col`, `endLine`, or `endColumn` properties. Messages state that findings are artifact-level and source mapping is deferred to CF-09.

V1 bounds:

```text
errors   <= 10
warnings <= 10
all notice commands <= 10
title <= 256 chars
message <= 4000 chars
report input <= 64 MiB
```

When any finding level overflows, the renderer reserves one notice slot for a deterministic incompleteness summary. Omitted counts are reported while the complete CF-05 JSON report remains authoritative. Presentation truncation never changes policy truth.

## Persisted-report truth

`commandf github-annotations --input` validates:

1. CF-05 schema v1;
2. embedded CF-04 schema/ruleset;
3. a freshly recomputed CF-05 decision from persisted policy + compatibility evidence;
4. exact equality between persisted and recomputed `decision`.

A stale or tampered decision is rejected operationally rather than rendered.

A valid policy-failed report is still renderable and the renderer exits `0`; it is a presentation command, not another policy gate.

## Action execution truth

Public Action metadata lives at repository root `action.yml`.

The build wrapper `scripts/github-action.sh`:

- supports Linux V1;
- validates required inputs/channels;
- rejects CR/LF in caller report-path;
- creates only the default internal report directory under `RUNNER_TEMP`;
- does not create a caller-specified missing report parent;
- installs pinned Rust 1.97.1 only when unavailable;
- builds exact checked-out source with Cargo `--locked` into a runner-temp target directory.

The pure run wrapper `scripts/github-action-run.sh` executes all user-derived values as quoted argv.

Final status behavior:

```text
check 0 -> report -> annotations -> outputs -> exit 0
check 2 -> report -> annotations -> outputs -> exit 2
check 1 -> operational annotation -> empty report-path output -> exit 1
renderer failure after 0/2 -> operational exit 1
unsupported check exit -> operational exit 1
```

Operational output intentionally exposes an empty `report-path`, preventing a stale path from being mistaken for canonical evidence.

## Security regression truth

`.github/scripts/test-cf08-action-runner.sh` uses a fake executable against the real production run wrapper and proves:

- pass exit `0`;
- policy exit `2` after renderer execution;
- operational exit `1`;
- renderer failure -> `1`;
- literal handling of a package value containing shell metacharacters;
- literal paths containing spaces;
- no command injection side effect;
- caller missing report parent is not silently created.

## Green implementation candidate

Exact implementation candidate:

```text
head: a5c24bc5fa9ee0360a3f6822eb7d8a97f8fe06a7
Actions run: 31830175341
```

Exact result:

```text
Format: PASS
Clippy --locked --workspace --all-targets --all-features -- -D warnings: PASS
Full workspace tests: PASS
CF-08 Action runner security regression: PASS
Real independently resolved/verified CF-01..05 R4 self-check smoke: PASS
Local `uses: ./` composite Action self-check: PASS
Action output verification: PASS
```

The local Action self-check uses the already independently resolved real `hl7.fhir.r4.core@4.0.1` before/after states and verifies `exit-code=0`, `passed=true`, a real report path, schema v1, both/breaking policy, a passing decision, zero blockers, and empty compatibility findings.

Synthetic policy-failure Action behavior is proved compositionally: CF-05 tests establish complete report publication before check exit `2`, while the CF-08 production-wrapper regression proves renderer execution and output publication before the wrapper returns `2`.

## Reviewer truth

CodeRabbit first reported a rate-limit response, then completed a substantive manual review on exact implementation head `a5c24bc5fa9ee0360a3f6822eb7d8a97f8fe06a7`.

Its completed review reported **no blocking issues** in the requested CF-08 areas and explicitly checked:

- workflow-command injection escaping;
- bounded 10/10/10 projection and deterministic overflow notice;
- CF-05/CF-04 authority validation and decision recomputation;
- no explicit source locations before CF-09;
- composite Action environment/quoted argv safety;
- exit `0/2` preservation and operational/renderer exit `1`;
- CR/LF report-path protection and empty operational report output;
- pinned exact-source build with Rust 1.97.1 / Cargo `--locked`;
- exact head/base ancestry and Draft state;
- successful GitHub rust check.

No actionable inline review thread was created.

Qodo `/review` was requested. No separate substantive Qodo result was observed.

```text
CodeRabbit substantive review: COMPLETED — NO BLOCKING ISSUES
CodeRabbit actionable threads: 0
Qodo result: NOT RETURNED
Qodo PASS claimed: NO
```

Cubic-generated summaries are informational only.

## Canonical authority

- `specs/008-cf-08-github-action-annotations/spec.md`
- `specs/008-cf-08-github-action-annotations/plan.md`
- `specs/008-cf-08-github-action-annotations/tasks.md`
- `specs/008-cf-08-github-action-annotations/convergence.md`

These documents are reconciled to the implemented persisted-decision validation, annotation bounds, title/message limits, overflow-notice reservation, split build/run wrappers, operational report-path behavior, security regression, Action integration, and reviewer truth.

## Final certification gate

CF-08 may be reported as:

```text
CF-08_COMPLETE_READY_FOR_FOUNDER_REVIEW
```

only when the exact PR head containing this documentation proves all of the following:

1. Format PASS;
2. locked full-workspace Clippy with `-D warnings` PASS;
3. locked full-workspace tests PASS;
4. CF-08 Action runner security regression PASS;
5. real independent R4 CF-01..05 self-check smoke PASS;
6. local `uses: ./` Action self-check PASS;
7. Action output verification PASS;
8. PR #9 remains open, Draft, unmerged;
9. auto-merge remains disabled;
10. base remains exact CF-05 head `9f158f1dcb7d04bc5ee582eabd1ee8dd93bd5019`;
11. no unresolved actionable review threads exist;
12. reviewer truth is not overstated;
13. no `feat/cf-09` branch/PR exists.

The exact final documentation-head run is recorded in PR metadata after this documentation commit to avoid self-referential documentation churn.

Until that final gate passes, the correct state is:

```text
CF-08_IMPLEMENTATION_COMPLETE_FINAL_CERTIFICATION_PENDING
```
