# CF-08 Implementation Plan

Status: Implemented — exact final documentation-head CI pending

## Architecture

CF-08 is a parallel delivery slice directly above converged CF-05:

```text
base branch: feat/cf-05-sarif-ci-gate
base SHA: 9f158f1dcb7d04bc5ee582eabd1ee8dd93bd5019
```

No CF-06 oracle or CF-07 terminology behavior is a dependency. CF-09 owns source mapping.

The delivered layers are:

1. `commandf-pkg` — validated deterministic GitHub annotation projection from CF-05 `CheckReport`;
2. `commandf-cli` — bounded `github-annotations --input` adapter;
3. repository root `action.yml` — composite Action public surface;
4. `scripts/github-action.sh` — source/toolchain/build wrapper;
5. `scripts/github-action-run.sh` — pure quoted-argv gate/annotation/exit wrapper;
6. `.github/scripts/test-cf08-action-runner.sh` — network-independent security/exit regression harness;
7. CI — exact Rust, real R4, and local `uses: ./` integration gates.

## Shared CF-05 authority

The existing CF-05 direction predicate is crate-private shared code used by both policy evaluation and annotation projection. No second producer/consumer selection algorithm exists.

Before rendering a persisted report, CF-08:

1. validates CF-05 report schema;
2. validates embedded CF-04 schema/ruleset;
3. recomputes CF-05 policy evaluation;
4. requires the persisted `decision` to equal the recomputed decision exactly.

This prevents a structurally valid but tampered/stale report from producing misleading GitHub UI evidence.

## Projection module

Module:

```text
crates/commandf-pkg/src/check_github.rs
```

Public API:

```text
check_report_to_github_annotations_bytes(&CheckReport) -> Result<Vec<u8>, CheckError>
```

Mapping:

```text
BREAKING -> error
RISKY    -> warning
ADDITIVE -> notice
```

Projection is deterministic and follows the finding ordering already established by CF-04.

### Bounds

```text
semantic error annotations   <= 10
semantic warning annotations <= 10
all notice commands          <= 10
annotation title             <= 256 chars
annotation message           <= 4000 chars
```

When any semantic level overflows, one notice slot is reserved for the deterministic incompleteness summary. Omitted counts are recorded in error/warning/notice order. The complete CF-05 JSON report remains authority.

### Escaping

Data:

```text
% -> %25
CR -> %0D
LF -> %0A
```

Properties additionally:

```text
: -> %3A
, -> %2C
```

Percent is encoded first.

No `file`, `line`, `col`, `endLine`, or `endColumn` property is emitted.

## CLI

Command:

```text
commandf github-annotations --input <path>
```

The CLI uses a 64 MiB hard input limit, deserializes `CheckReport`, calls the package projection API, and writes bytes to stdout.

A valid policy-failed report renders successfully with renderer exit `0`; this command does not create a second policy gate.

Malformed, oversized, unsupported, or inconsistent reports return operational `1` through normal commandF runtime error handling. Missing required CLI arguments remain normal Clap usage behavior.

## Composite Action

Public metadata:

```text
action.yml
runs.using: composite
```

Inputs:

```text
package
before-lock
before-cache
after-lock
after-cache
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

No token input or repository-write permission is requested.

## Build wrapper

`scripts/github-action.sh`:

- Linux V1 only;
- validates required inputs and runner channels;
- rejects CR/LF in user report path;
- uses caller report path verbatim when supplied;
- otherwise creates `$RUNNER_TEMP/commandf/check-report.json` parent;
- never creates a caller-specified missing parent;
- requires rustup/cargo;
- installs pinned Rust 1.97.1 only when unavailable;
- builds exact checked-out source using `cargo +1.97.1 build --locked -p commandf`;
- uses `$RUNNER_TEMP/commandf-target` as target dir;
- transfers control to the pure run wrapper with the exact executable path.

Build-wrapper operational failure writes:

```text
report-path=
exit-code=1
passed=false
```

and emits a static trusted operational annotation.

## Pure Action runner

`scripts/github-action-run.sh` receives the executable path as argv and uses only quoted argv to run:

```text
commandf check ... --format json --output <report>
```

The check exit is captured without shell-aborting.

Behavior:

```text
check 0 -> require report -> render annotations -> outputs -> exit 0
check 2 -> require report -> render annotations -> outputs -> exit 2
check 1 -> static operational error -> empty report output -> exit 1
other   -> static operational error -> empty report output -> exit 1
renderer failure after 0/2 -> operational exit 1
```

Thus policy failure retains the complete report and renders before the failing Action exit.

## Security regression harness

`.github/scripts/test-cf08-action-runner.sh` injects a fake executable and proves the run wrapper independently of Rust compilation or network state.

Cases:

- pass `0`;
- policy failure `2` with renderer invoked first;
- operational `1`;
- renderer failure -> `1`;
- package input containing shell metacharacters remains literal argv;
- paths containing spaces remain literal argv;
- caller report parent is not silently created.

The harness is intentionally testing a real production wrapper, not a hidden production test hook.

## Rust regression matrix

Package tests cover:

- severity mapping;
- exact shared direction selection;
- fail-on independence;
- command/property escaping;
- workflow-command injection attempts;
- no explicit source location;
- 10/10/10 presentation bounds and overflow counts;
- title/message bounds;
- deterministic bytes;
- valid policy-failed report rendering;
- inconsistent persisted decision rejection;
- unsupported CF-05/CF-04 authority rejection.

CLI tests cover:

- help;
- missing input usage;
- valid empty report;
- valid policy-failed report;
- malformed JSON operational failure;
- oversized file operational failure.

## Real integration gate

CI preserves the CF-01..05 real R4 chain, using two independently resolved/verified `hl7.fhir.r4.core@4.0.1` states.

After the ordinary CF-05 real self-check succeeds, CI invokes:

```yaml
- uses: ./
```

with those explicit states.

It then asserts:

```text
exit-code = 0
passed = true
report-path exists
schema = 1
policy = both/breaking
decision.passed = true
blocking_findings = 0
compatibility.findings = []
```

The synthetic runner regression proves exit `2` behavior without intentionally failing the required real Action smoke.

## Implementation evidence

Exact green implementation candidate:

```text
head: a5c24bc5fa9ee0360a3f6822eb7d8a97f8fe06a7
run: 31830175341
Format: PASS
Clippy: PASS
Full tests: PASS
CF-08 runner security regression: PASS
Real CF-01..05 R4 smoke: PASS
Local composite Action self-check: PASS
Action output verification: PASS
```

## Reviewer disposition

CodeRabbit substantive review completed on this exact implementation head and reported no blocking issues in the requested CF-08 boundaries. It specifically verified escaping, bounds, persisted decision validation, no fabricated locations, quoted argv, exit preservation, report-path safety, exact source build, stack identity, Draft state, and green check evidence.

No actionable inline review thread was produced.

Qodo `/review` was requested; no substantive result was observed and no Qodo PASS is claimed.

## Final convergence procedure

1. reconcile spec, plan, tasks, and convergence against implementation candidate;
2. commit docs only;
3. run exact-final-documentation-head Format/Clippy/tests/runner-security/real-R4/local-Action gates;
4. record final run in PR metadata rather than creating a self-referential documentation commit;
5. verify PR open/Draft/unmerged, auto-merge null, clean mergeability, zero unresolved threads;
6. verify no CF-09 branch/PR;
7. update PR body with exact head/tree/run and reviewer truth.
