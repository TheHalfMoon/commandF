# CF-08 Implementation Plan

Status: Implementation authorized

## Architecture

CF-08 is a parallel delivery slice directly above converged CF-05:

```text
base branch: feat/cf-05-sarif-ci-gate
base SHA: 9f158f1dcb7d04bc5ee582eabd1ee8dd93bd5019
```

No compatibility rule, policy threshold, or source-location authority moves into CF-08.

Implementation layers:

1. `commandf-pkg` — deterministic GitHub annotation projection from a validated CF-05 `CheckReport`;
2. `commandf-cli` — `github-annotations --input` adapter;
3. repository-root `action.yml` + a narrowly scoped shell runner — packaged Action execution;
4. CI — unit/CLI/action integration gates.

## Package module

Add:

```text
crates/commandf-pkg/src/check_github.rs
```

Public entrypoint:

```text
check_report_to_github_annotations_bytes(&CheckReport) -> Result<Vec<u8>, CheckError>
```

The function validates the report using the same CF-05 compatibility validation authority before projecting findings.

## Finding selection

Refactor the existing CF-05 direction predicate so the same crate-private helper is reused by policy evaluation and GitHub projection.

No second producer/consumer selection algorithm is permitted.

`fail_on` is deliberately not part of annotation selection. It remains only a decision threshold.

## Annotation ordering and caps

Use the deterministic finding order already guaranteed by CF-04.

Maintain independent counters:

```text
error   <= 10
warning <= 10
notice  <= 10
```

Findings beyond a cap are counted, not emitted. After finding traversal, emit one deterministic notice if any level overflowed, with stable counts in error/warning/notice order.

The overflow notice does not count as a semantic finding and never changes the gate decision.

## Workflow command encoding

Renderer helpers:

```text
escape_data
escape_property
annotation_level
annotation_title
annotation_message
```

`escape_data`:

```text
%  -> %25
CR -> %0D
LF -> %0A
```

`escape_property` additionally encodes:

```text
:  -> %3A
,  -> %2C
```

Replacement order must encode `%` first so introduced escape sequences are not re-escaped.

V1 output shape:

```text
::error title=commandF CF04-...::artifact-level message
::warning title=commandF CF04-...::artifact-level message
::notice title=commandF CF04-...::artifact-level message
```

No `file`, `line`, `col`, `endLine`, or `endColumn` property is emitted.

## Message evidence

Message construction is deterministic and bounded to CF-04 finding fields. Include when present:

- severity/direction;
- source change kind;
- resource identity;
- package filename(s);
- view;
- element id;
- field;
- CF-04 message.

The phrase `artifact-level finding; source mapping deferred to CF-09` is included so GitHub UI defaults cannot be mistaken for commandF source mapping.

## CLI

Add subcommand:

```text
commandf github-annotations --input <path>
```

Execution:

1. bounded `fs::read` of report path using a hard maximum report byte size;
2. deserialize `CheckReport`;
3. validate schema/ruleset through the projection function;
4. write escaped annotation bytes to stdout;
5. return `0` on valid report, including policy-failed reports;
6. return `1` on invalid/unsupported input.

Use a CF-08 hard report-input limit of 64 MiB. This is a presentation adapter, not a generic unbounded JSON reader.

## Composite Action

Create repository-root:

```text
action.yml
scripts/github-action.sh
```

The composite action invokes one script with inputs supplied as environment variables, never interpolated into a shell command string.

### Toolchain/build

The script:

1. requires Linux V1 (`RUNNER_OS=Linux`);
2. ensures `rustup` and `cargo` exist;
3. installs Rust `1.97.1` with minimal profile if `cargo +1.97.1 --version` fails;
4. builds exact action source with:

```text
cargo +1.97.1 build --locked --manifest-path "$GITHUB_ACTION_PATH/Cargo.toml" -p commandf
```

5. uses a target directory under `RUNNER_TEMP` so action-source directories are not treated as mutable build state.

### Check execution

The script allocates the report path:

- user-provided `report-path` when non-empty;
- otherwise `$RUNNER_TEMP/commandf/check-report.json`.

It creates the report parent directory only for the internally selected default path. A caller-provided path retains CF-05 fail-closed parent semantics rather than being silently created.

Then it runs `commandf check --format json --output` with fully quoted argv.

Capture exit code without aborting the script.

### Annotation execution

- exit `0` or `2`: complete report must exist; run `github-annotations` against it;
- exit `1`: do not pretend a report exists; emit one static operational error workflow command and preserve exit `1`;
- annotation renderer failure overrides a prior `0` or `2` to operational exit `1` because UI projection cannot be trusted.

### Action outputs

Write to `GITHUB_OUTPUT`:

```text
report-path=<resolved path>
exit-code=<0|1|2>
passed=<true|false>
```

`passed=true` only for exit `0`.

The final script exit is the resolved commandF status.

## Action metadata

`action.yml` defines:

```text
name: commandF compatibility gate
runs.using: composite
```

Required inputs:

```text
package
before-lock
before-cache
after-lock
after-cache
```

Defaults:

```text
direction: both
fail-on: breaking
report-path: ""
```

The composite action does not request a GitHub token, write repository contents, create checks through the REST API, upload SARIF, or upload artifacts.

## Tests

### Projection unit matrix

- BREAKING -> error;
- RISKY -> warning;
- ADDITIVE -> notice;
- producer/consumer/both filtering exactly matches CF-05;
- fail-on does not alter selected annotation set;
- title/message stable ordering;
- percent/CR/LF/colon/comma escaping;
- injected `::error` text remains data;
- no location properties;
- 10/10/10 caps;
- overflow summary counts;
- valid policy-failed report still renders;
- unsupported schema/ruleset fails closed;
- repeated bytes identical.

### CLI matrix

- help;
- required `--input`;
- valid empty report -> empty output;
- synthetic findings -> exact workflow-command bytes;
- policy-failed report -> exit 0 from renderer;
- malformed JSON -> exit 1;
- oversized input -> exit 1.

### Action script matrix

Use a shell-level test harness with a fake commandF binary to prove without network/toolchain installation:

- policy pass -> renderer runs, outputs written, exit 0;
- policy fail -> renderer runs before final exit 2;
- operational check failure -> renderer not run, final exit 1;
- renderer failure -> final exit 1;
- arguments containing spaces/shell metacharacters are passed as literal argv;
- caller report parent is not silently created;
- default report parent is created under runner temp.

The real Action integration gate separately validates the actual compiled commandF binary.

## Real GitHub Action integration gate

Preserve all existing CF-01..05 gates.

After the existing independent real R4 before/after states are produced, invoke the local root action:

```yaml
- uses: ./
  with:
    package: hl7.fhir.r4.core
    before-lock: /tmp/commandf-smoke/before.lock
    before-cache: /tmp/commandf-smoke/before-cache
    after-lock: /tmp/commandf-smoke/after.lock
    after-cache: /tmp/commandf-smoke/after-cache
```

Acceptance:

- Action step succeeds;
- outputs report path, `exit-code=0`, `passed=true`;
- report exists and contains schema 1, passing decision, and empty findings;
- annotation renderer emits no false finding annotation on self-equivalence.

A synthetic CLI/script regression supplies the policy-failure annotation case so CI does not intentionally fail its own required Action smoke.

## Security review priorities

1. workflow-command injection escaping;
2. shell/argv injection through Action inputs;
3. source-location fabrication before CF-09;
4. annotation truncation changing policy truth;
5. policy exit `2` accidentally converted to operational `1` or success;
6. complete report loss on policy failure;
7. unbounded report parsing;
8. hidden network/package acquisition in check path;
9. action source/toolchain mismatch;
10. third-party Action dependency creep.

## Convergence

CF-08 converges only after exact-final-head format/clippy/tests, real CF-01..05 smoke, local composite Action smoke, reviewer truth disposition, and Spec Kit reconciliation. PR remains Draft/open/unmerged with auto-merge disabled. CF-09 remains unstarted.
