# CF-05 Implementation Plan

Status: Implemented — convergence candidate

## Architecture

CF-05 remains inside the existing `commandf-pkg` crate and `commandf` CLI. No new workspace crate is introduced.

The slice has three layers:

1. a pure policy evaluator over `CompatibilityReport`;
2. deterministic JSON/SARIF serializers;
3. the `commandf check` CLI adapter, which reuses the existing two-state diff/classify path.

CF-05 does not reopen archives or reimplement CF-03/CF-04 semantics.

## Public model

The versioned gate model is:

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

The pure evaluator is:

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

CF-05 implements the minimum typed SARIF 2.1.0 surface needed by commandF rather than adding a third-party SARIF dependency.

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
- `level` from CF-04 severity (`error`, `warning`, `note`);
- message text copied from CF-04;
- the original CF-04 severity and all available commandF evidence in result properties.

The serializer emits no timestamps, GUIDs, random values, host paths, repository assumptions, or physical locations.

The active rule descriptor set is sorted by rule id and de-duplicated. Rule metadata remains minimal and factual; result-specific direction/evidence stays on results.

## SARIF / GitHub boundary

OASIS SARIF 2.1.0 is the interchange contract. GitHub Code Scanning supports a subset of SARIF 2.1.0 and requires a physical location for an alert to display.

CF-04 findings currently refer to FHIR package artifacts, not checked-in repository source paths. CF-05 therefore emits standards-oriented SARIF without fabricated physical locations. GitHub source annotations remain blocked on CF-09 source mapping.

This is an explicit product boundary, not a missing implementation detail. Generated third-party PR summaries that describe current SARIF as "upload-ready" are not treated as commandF authority.

## CLI

`Check` is added beside `Diff` and `Classify`.

`Check` accepts:

- package name;
- before/after lock and cache paths;
- direction;
- fail-on threshold;
- `json` or `sarif` format;
- optional output path.

`Inspect`, `Diff`, and `Classify` keep their existing output contracts and do not silently gain SARIF.

The top-level executable uses `Cli::try_parse()` so `commandf check` parse/usage failures return `1`, `check --help` remains `0`, and exit `2` is reserved for completed CF-05 policy failures. Existing non-`check` Clap parse behavior is preserved.

The check execution path remains:

```text
build_diff_report
  -> classify_structural_diff
  -> evaluate_compatibility_policy
  -> serialize JSON or SARIF
  -> write output
  -> return 0 or 2
```

Operational errors from any stage return `1`.

## Output writer

The CLI helper writes bytes to stdout or atomically replaces the requested output path.

File publication:

1. verify the parent directory exists;
2. allocate a unique temporary file in that same directory with create-new semantics;
3. write and sync all bytes;
4. close the temporary file;
5. rename the temporary file over the destination;
6. remove a leftover temporary file on publication failure.

This permits repeated CI runs to replace stale prior reports while preventing a partially written result from becoming the requested output. The complete output is published before returning policy exit `2`.

## Test strategy

Package regressions prove:

- breaking/risky/none thresholds;
- producer/consumer/both direction filtering;
- count accuracy;
- unsupported CF-04 schema/ruleset failure;
- repeated independent policy evaluations produce byte-identical JSON and SARIF;
- SARIF schema/version/tool metadata;
- stable rule ids and rule sorting/de-duplication;
- severity-to-level mapping while preserving original CF-04 severity evidence;
- message and full available evidence-property preservation;
- no physical locations.

CLI regressions prove:

- `check --help` contract;
- policy-failure exit `2` with JSON output present;
- `--fail-on none` pass behavior without evidence removal;
- SARIF file output exists before exit `2`;
- existing output replacement on both pass and fail paths;
- corrupted cache remains exit `1`;
- invalid check policy syntax remains exit `1`;
- missing output parent remains exit `1`;
- no network acquisition through a dead-proxy test environment.

## Real smoke

The existing independent R4 smoke runs:

```text
commandf check hl7.fhir.r4.core ... --format json
commandf check hl7.fhir.r4.core ... --format sarif
```

It requires JSON `passed == true`, `blocking_findings == 0`, and embedded CF-04 findings empty. It requires SARIF version `2.1.0`, one run, tool name `commandF`, empty results, the passing decision property, and `commandf.sourceMapping = deferred_cf09`.

## Review priorities and disposition

1. exit `2` must only mean a completed policy failure — self-review found and fixed the original Clap ambiguity;
2. output must exist before exit `2` — proven by CLI regression;
3. existing output must be replaced atomically — CodeRabbit finding fixed with same-directory rename publication and pass/fail regressions;
4. no CF-04 evidence loss or severity reinterpretation — strengthened SARIF regression proves original `BREAKING` evidence is preserved while level maps to `error`;
5. no fake source locations in SARIF — enforced by serializer shape and regression;
6. deterministic byte output — repeated independent evaluation regression;
7. no network in `check` — CLI tests and real locked-cache path;
8. no CF-06 oracle leakage — no validator execution or CF-06 branch/work in this slice.

## Deferred work

Do not add GitHub upload actions, security-event permissions, validator execution, terminology expansion, FSH source mapping, graph analysis, or AI judgment in this slice.
