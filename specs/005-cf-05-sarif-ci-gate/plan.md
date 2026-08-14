# CF-05 Implementation Plan

Status: Approved for implementation

## Architecture

CF-05 remains inside the existing `commandf-pkg` crate and `commandf` CLI. No new workspace crate is introduced.

The slice has three layers:

1. a pure policy evaluator over `CompatibilityReport`;
2. deterministic JSON/SARIF serializers;
3. the `commandf check` CLI adapter, which reuses the existing two-state diff/classify path.

CF-05 does not reopen archives or reimplement CF-03/CF-04 semantics.

## Public model

Add a versioned gate model:

```text
CheckPolicy {
  direction: both | producer | consumer,
  fail_on: breaking | risky | none,
}

CheckDecision {
  passed,
  total_findings,
  selected_findings,
  breaking_findings,
  risky_findings,
  additive_findings,
  blocking_findings,
}

CheckReport {
  schema,
  policy,
  decision,
  compatibility,
}
```

`compatibility` contains the complete CF-04 report without filtering or mutation.

## Policy engine

Expose a pure function similar to:

```text
evaluate_compatibility_policy(
  &CompatibilityReport,
  CheckPolicy,
) -> Result<CheckReport, CheckError>
```

Validation occurs before evaluation:

- CF-04 schema must be `1`;
- CF-04 ruleset must be `cf04-rules-v1`.

Direction selection is a predicate only. Threshold evaluation counts selected findings whose severity matches the policy.

The function has no filesystem, network, clock, environment, or process dependencies.

## SARIF model

Implement the minimum typed SARIF 2.1.0 surface needed by commandF rather than adding a third-party SARIF dependency.

Top-level shape:

```text
SarifLog {
  $schema,
  version = "2.1.0",
  runs = [SarifRun]
}
```

Each run includes:

- `tool.driver.name = "commandF"`;
- deterministic active rule descriptors keyed by CF-04 `rule_id`;
- all CF-04 findings as results;
- run properties containing package evidence, CF-04 ruleset, CF-05 policy, and decision counts.

Each result includes:

- `ruleId` = CF-04 rule id;
- `level` from CF-04 severity;
- message text copied from CF-04;
- commandF-specific properties for all available evidence.

Do not emit timestamps, GUIDs, random values, host paths, repository assumptions, or physical locations.

The active rule descriptor set is sorted by rule id and de-duplicated. Rule metadata should remain minimal and factual; result-specific direction/evidence stays on results.

## SARIF / GitHub boundary

OASIS SARIF 2.1.0 permits analysis results as an interchange format. GitHub Code Scanning supports a subset of SARIF 2.1.0 and requires a location for an alert to display.

CF-04 findings currently refer to FHIR package artifacts, not checked-in repository source paths. CF-05 therefore emits standards-oriented SARIF without fabricated physical locations. GitHub source annotations remain blocked on CF-09 source mapping.

This is an explicit product boundary, not a missing implementation detail.

## CLI

Add `Check` beside `Diff` and `Classify`.

Extend output format to:

```text
json | sarif
```

`Inspect`, `Diff`, and `Classify` continue to accept only their supported format(s); reject `sarif` for commands that do not support it instead of silently changing their output contract.

`Check` accepts:

- package name;
- before/after lock and cache paths;
- direction;
- fail-on threshold;
- output format;
- optional output path.

To avoid conflating policy failure with operational failure, refactor the top-level runner to return an explicit process outcome. Preserve current behavior for existing commands.

Recommended internal shape:

```text
enum ProcessOutcome {
  Success,
  PolicyFailed,
}
```

`main` maps these to exit `0` and `2`; errors remain exit `1`.

## Output writer

Create a small helper that either writes bytes to stdout or atomically replaces the requested output file.

Atomic file output:

1. verify the parent directory exists;
2. create a temporary file in that same directory;
3. write and sync bytes;
4. persist/rename over the target only after serialization and write succeed.

No partial result file should remain after a write failure.

## Test strategy

Package tests:

- default policy passes empty report;
- breaking threshold behavior;
- risky threshold behavior;
- none threshold behavior;
- producer/consumer/both filtering;
- count accuracy;
- unsupported CF-04 schema/ruleset failure;
- repeated JSON serialization byte identity;
- repeated SARIF serialization byte identity;
- SARIF severity mapping;
- active rule sorting/de-duplication;
- all CF-04 evidence properties preserved where available;
- no timestamps or physical locations.

CLI tests:

- `check --help` contract;
- successful JSON stdout;
- successful SARIF stdout;
- policy-failure exit `2` with output present;
- operational failure exit `1`;
- output-file success;
- output-file policy failure still leaves complete output;
- parent-directory failure remains exit `1`;
- no network acquisition.

Real smoke:

Extend the current independent R4 smoke to run:

```text
commandf check hl7.fhir.r4.core ... --format json
commandf check hl7.fhir.r4.core ... --format sarif
```

Require JSON `passed == true`, `blocking_findings == 0`, and embedded CF-04 findings empty. Require SARIF version `2.1.0`, one run, tool name `commandF`, and empty results.

## Review priorities

1. exit `2` must only mean a completed policy failure;
2. output must exist before exit `2`;
3. no CF-04 evidence loss or severity reinterpretation;
4. no fake source locations in SARIF;
5. deterministic byte output;
6. no network in `check`;
7. no CF-06 oracle leakage.

## Deferred work

Do not add GitHub upload actions, security-event permissions, validator execution, terminology expansion, FSH source mapping, graph analysis, or AI judgment in this slice.
