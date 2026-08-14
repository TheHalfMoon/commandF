# CF-08 — GitHub Action + Bounded Annotations

Status: Implemented — final documentation-head certification pending

## Purpose

CF-08 turns converged CF-05 into a directly usable GitHub Action and projects deterministic CF-05 findings into bounded GitHub Actions workflow-command annotations.

CF-08 is a delivery layer only. CF-04 remains compatibility-classification authority and CF-05 remains policy/exit-code authority.

## Exact stack

```text
base branch: feat/cf-05-sarif-ci-gate
base SHA: 9f158f1dcb7d04bc5ee582eabd1ee8dd93bd5019
```

CF-06 and CF-07 are not dependencies. CF-09 source mapping is deferred and MUST NOT leak into this slice.

## Public capabilities

### Annotation renderer

```text
commandf github-annotations --input <check-report.json>
```

The renderer:

1. reads at most 64 MiB;
2. parses `CheckReport` schema v1;
3. validates the embedded CF-04 schema/ruleset;
4. recomputes the expected CF-05 decision from the persisted policy + compatibility evidence and rejects inconsistent persisted decision/count data;
5. reuses the exact CF-05 direction-selection helper;
6. emits deterministic escaped GitHub workflow-command annotations;
7. returns `0` for every valid report, including a policy-failed report;
8. returns operational `1` for malformed, unsupported, oversized, or inconsistent report evidence.

Normal Clap usage errors for this presentation command remain normal non-check usage errors.

### Root composite Action

Repository-root `action.yml` exposes:

Required inputs:

```text
package
before-lock
before-cache
after-lock
after-cache
```

Optional inputs:

```text
direction = both
fail-on = breaking
report-path = ""
```

Outputs:

```text
report-path
exit-code
passed
```

The V1 Action supports Linux runners and compiles the exact checked-out Action source with pinned Rust `1.97.1`, `cargo --locked`, and a target directory under `RUNNER_TEMP`.

## Authority preservation

CF-08 MUST NOT reinterpret:

- compatibility severity;
- producer/consumer direction;
- `fail_on` threshold;
- blocking-finding counts;
- pass/fail decision;
- CF-04 finding evidence.

The persisted `CheckReport` is revalidated before presentation. A report whose stored `decision` differs from a fresh CF-05 evaluation is rejected as operationally invalid rather than rendered.

## Annotation mapping

```text
BREAKING -> error
RISKY    -> warning
ADDITIVE -> notice
```

Direction filtering follows `CheckReport.policy.direction` exactly.

`fail_on` affects the gate decision only. It does not hide otherwise-selected non-blocking annotations.

## No fabricated source locations

CF-09 owns repository/FSH source mapping. CF-08 emits no explicit:

```text
file
line
col
endLine
endColumn
```

Annotation messages explicitly say:

```text
artifact-level finding; source mapping deferred to CF-09
```

Package artifact filenames, resource canonicals, element ids, views, fields, rule ids, directions, severities, and CF-04 messages may appear as artifact evidence, but they are not represented as repository source locations.

CF-08 does not upload SARIF to code scanning using fabricated physical locations.

## Workflow-command safety

Finding-controlled content is untrusted workflow-command input.

Command data escaping:

```text
%  -> %25
CR -> %0D
LF -> %0A
```

Property escaping additionally encodes:

```text
:  -> %3A
,  -> %2C
```

Percent is escaped before later replacements. Regression coverage proves an injected `::error` sequence remains annotation data and cannot start a second command.

Action inputs are passed through environment variables and then through fully quoted argv positions. No input is interpolated into an evaluated shell command string.

Caller-provided `report-path` rejects carriage return and line feed before it can be written to `GITHUB_OUTPUT`.

## Annotation bounds

V1 semantic-finding presentation caps:

```text
error   <= 10
warning <= 10
notice  <= 10
```

If any level overflows, one notice slot is reserved for a deterministic incompleteness summary. Therefore the total emitted notice workflow commands remain at most 10.

The summary records omitted error/warning/notice counts and states that the complete CF-05 JSON report remains authoritative.

Presentation caps never change:

- `decision.passed`;
- `blocking_findings`;
- the complete JSON report;
- the Action exit code.

Additional output bounds:

```text
annotation title   <= 256 characters
annotation message <= 4000 characters
report input       <= 64 MiB
```

Truncation markers are deterministic.

## Action execution truth

The public composite Action delegates to two implementation scripts:

```text
scripts/github-action.sh
scripts/github-action-run.sh
```

The build wrapper:

- validates required runner/input state;
- supports Linux V1;
- chooses caller report path or `$RUNNER_TEMP/commandf/check-report.json`;
- creates only the default internal report parent;
- never silently creates a caller-specified missing parent;
- requires `cargo`/`rustup`;
- installs Rust 1.97.1 only if the pinned toolchain is unavailable;
- builds exact checked-out commandF source using `--locked`;
- hands the resulting executable to the pure run wrapper.

The run wrapper executes `commandf check` using fully quoted argv, captures CF-05 exit status, renders annotations from the completed JSON report for exits `0` and `2`, writes outputs, then exits with the original CF-05 status unless annotation rendering fails.

## Exit contract

CF-05 semantics are preserved exactly:

```text
0 = completed policy pass
1 = usage / operational / classification / output / renderer failure
2 = completed policy failure after complete report publication
```

A policy exit `2` renders available annotations before the Action fails.

A renderer failure converts a prior `0` or `2` to operational `1` because the GitHub projection cannot be trusted.

On operational exit `1`, the Action exposes:

```text
report-path=
exit-code=1
passed=false
```

The empty report-path deliberately avoids implying that any stale or partial path is canonical evidence.

On successful evaluation (`0` or `2`), `report-path` points to the complete CF-05 JSON report.

## Offline / acquisition boundary

`commandf check` and `commandf github-annotations` perform no FHIR package acquisition and use only explicit lock/cache/report inputs.

The source-backed Action may install the pinned Rust toolchain as build infrastructure. That is not FHIR package or terminology acquisition.

No GitHub token, Check Runs REST mutation, code-scanning upload, or automatic artifact upload is introduced by CF-08.

## Test and CI truth

Regression coverage proves:

- BREAKING/RISKY/ADDITIVE level mapping;
- exact shared direction selection;
- fail-on independence from presentation selection;
- command/property escaping and command-injection resistance;
- no explicit source locations;
- 10/10/10 caps and deterministic overflow disclosure;
- title/message bounds;
- repeated byte-identical rendering;
- persisted-decision consistency validation;
- malformed/unsupported/oversized report failures;
- valid policy-failed report rendering;
- Action pass `0`, policy fail `2`, operational `1`, and renderer-failure `1` behavior;
- renderer executes before final exit `2`;
- shell metacharacters and paths with spaces remain literal argv;
- caller report parent is not silently created;
- default report path is created under runner temp;
- real local `uses: ./` self-check on independently resolved public R4 states;
- output report path / exit code / passed truth.

Exact green implementation evidence before final documentation reconciliation:

```text
head: a5c24bc5fa9ee0360a3f6822eb7d8a97f8fe06a7
Actions run: 31830175341
Format: PASS
Clippy --locked --workspace --all-targets --all-features -- -D warnings: PASS
Full workspace tests: PASS
CF-08 Action runner security regression: PASS
Real CF-01..05 R4 self-check smoke: PASS
Local composite Action self-check: PASS
Action output verification: PASS
```

## Reviewer truth

CodeRabbit substantive review completed on exact implementation head `a5c24bc5fa9ee0360a3f6822eb7d8a97f8fe06a7` after an earlier rate-limit response. It reported **no blocking issues** in the requested CF-08 areas, including workflow-command escaping, annotation bounds, persisted-decision validation, no-source-location boundary, quoted argv, exit preservation, report-path safety, and exact source-backed build.

No actionable inline review thread was created by that review.

Qodo `/review` was requested. No separate substantive Qodo result was observed, so no Qodo PASS is claimed.

Cubic summaries are informational only.

## Acceptance status

The implementation candidate satisfies the behavioral acceptance contract. Final CF-08 certification additionally requires exact-final-documentation-head CI, PR governance checks, reviewer-thread recheck, and confirmation that CF-09 remains unstarted.

## Explicit deferrals

CF-08 does not add:

- FSH/repository source mapping or file/line annotations — CF-09;
- GitHub code-scanning upload based on fabricated locations;
- public real-IG delta corpus — CF-10;
- ecosystem graph/blast radius — CF-11/12;
- baselines/suppressions — CF-13;
- terminology-server execution;
- mapping execution;
- AI/agent compatibility authority.
