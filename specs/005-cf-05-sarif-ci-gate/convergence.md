# CF-05 Convergence Record

## Decision

```text
CF-05_IMPLEMENTATION_CONVERGED
FINAL_DOCS_HEAD_REQUIRES_FRESH_EXACT_HEAD_CI
READY_FOR_FOUNDER_REVIEW_ONLY_AFTER_THAT_RUN_IS_GREEN
```

This record reconciles the implemented CF-05 behavior, tests, reviewer findings, and deferrals. It does not authorize merge and does not authorize CF-06.

## Stack identity

```text
Repository:
TheHalfMoon/commandF

CF-05 branch:
feat/cf-05-sarif-ci-gate

CF-04 base branch:
feat/cf-04-compatibility-rules

Exact CF-04 base SHA:
ae33586a925023d92b4d58db01663bf26f3bd9a3

Exact CF-04 base tree:
754bfe468c1cb231cfcc12185bd3423e0e387917

Converged implementation head before this docs-only record:
2f959acd08a1878c620498be23d6e4b0adef47e3

Converged implementation tree before this docs-only record:
2bfefd9ccc635b7e25c9d3f50bca754ef5296113

Exact implementation validation run:
31804887047

Implementation run conclusion:
SUCCESS
```

The final docs-only convergence commit must receive its own fresh pull-request CI run. That exact final run id belongs in the PR body after completion; embedding it here would require another repository commit and would invalidate the head it certified.

## User-visible contract

```text
commandf check <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  [--direction both|producer|consumer] \
  [--fail-on breaking|risky|none] \
  [--format json|sarif] \
  [--output <path>]
```

Defaults:

- direction: `both`;
- fail-on: `breaking`;
- format: `json`;
- destination: stdout.

`check` performs no acquisition. It reuses CF-03 two-state locked package loading and CF-04 deterministic classification.

## Exit-code convergence

The final CF-05 process contract is:

- `0` — check evaluation completed and policy passed;
- `1` — check usage/input, operational, output, CF-03, or CF-04 failure;
- `2` — check evaluation completed successfully, output was emitted, and policy failed.

A self-review after the first green implementation candidate found that default Clap parse failures also use exit `2`. That violated the CF-05 invariant that `2` uniquely means completed policy failure.

Correction:

- `Cli::try_parse()` is used at the process boundary;
- `commandf check` parse failures normalize to exit `1`;
- `commandf check --help` remains `0`;
- existing non-`check` Clap parse behavior is preserved;
- regression `check_exit_contract.rs` proves an invalid `--fail-on` value returns `1`.

Correction head:

```text
f0750dbc0a02f0c870f8d7cc1b0987d71689c0d1
```

Exact correction validation run:

```text
31803887511 — SUCCESS
```

## Policy convergence

CF-05 does not reinterpret compatibility semantics.

The pure evaluator validates:

- CF-04 schema `1`;
- CF-04 ruleset `cf04-rules-v1`.

It then applies direction selection before threshold evaluation:

- `producer` selects producer findings;
- `consumer` selects consumer findings;
- `both` selects both.

Thresholds:

- `breaking` blocks selected `BREAKING` findings;
- `risky` blocks selected `BREAKING` and `RISKY` findings;
- `none` blocks nothing;
- `ADDITIVE` never blocks under `breaking` or `risky`.

The JSON `CheckReport` contains the complete unfiltered CF-04 report. Direction filtering is decision-only and does not remove evidence.

## SARIF convergence

CF-05 emits a deterministic SARIF 2.1.0 interchange artifact with the OASIS SARIF 2.1.0 Errata 01 schema URI.

The serializer:

- identifies `commandF` as the tool;
- uses CF-04 rule ids as SARIF `ruleId` values;
- maps `BREAKING` to SARIF `error`;
- maps `RISKY` to SARIF `warning`;
- maps `ADDITIVE` to SARIF `note`;
- separately preserves the original CF-04 severity evidence (`BREAKING`, `RISKY`, `ADDITIVE`) in result properties;
- preserves direction, structural change kind, FHIR resource identity, before/after filenames, view, element id, field, and before/after values when present;
- includes policy, decision counts, package versions/digests, and CF-04 ruleset in run properties;
- sorts/de-duplicates active rule descriptors by rule id;
- emits no clock, random, host-path, run-id, or environment-dependent values;
- emits no fabricated physical source locations.

A strengthened SARIF test initially expected lowercase compatibility severity in the evidence property and failed on head `56bff55fb1e002bda95b879d7b7c2b4f2c76c15a`, run `31804733313`. Inspection confirmed CF-04 intentionally serializes severity as SCREAMING_SNAKE_CASE. The test was corrected rather than changing runtime evidence. Final implementation run `31804887047` is green.

## GitHub Code Scanning boundary

CF-04 findings identify FHIR package artifacts, not checked-in repository source locations. CF-05 therefore refuses to invent repository paths or line numbers.

The SARIF run records:

```text
commandf.sourceMapping = deferred_cf09
```

Physical GitHub annotations remain a CF-09 source-mapping concern. A generated third-party PR summary described the artifact as "upload-ready SARIF"; that wording is not commandF authority and is not adopted as a product guarantee for GitHub alert display.

## Atomic output convergence

The first reviewed implementation used a hard link from a temporary file to the requested destination. That was safe for new output paths but could not replace an existing report.

CodeRabbit actionable finding:

```text
Existing --output files must be atomically replaced rather than causing a failure that leaves stale CI evidence.
```

Disposition:

- VALID;
- fixed;
- thread resolved.

Final publication path:

1. validate the parent directory;
2. create a unique same-directory temporary file with create-new semantics;
3. write all bytes;
4. `sync_all` the temporary file;
5. close it;
6. `rename` it over the destination;
7. clean up a leftover temporary file if publication fails.

Regressions prove:

- existing output is replaced on policy failure and exit `2` occurs only after the complete replacement exists;
- existing output is replaced on policy pass and exit `0` follows;
- missing parent directory is an operational exit `1` and no result file is created.

Fix head:

```text
42875888075e177ba66d8522d1ce8ac2d429af7f
```

Exact fix validation run:

```text
31804376560 — SUCCESS
```

The review reply cited that exact head/run and CodeRabbit resolved the thread.

## Determinism evidence

Package regression `repeated_evaluation_and_serialization_are_byte_deterministic` evaluates the same CF-04 input twice with the same CF-05 policy and proves:

- the two `CheckReport` values are equal;
- JSON bytes are equal;
- SARIF bytes are equal.

The serializer uses ordered collections for SARIF rule/property ordering and contains no timestamps or random identifiers.

## CLI / failure-path evidence

CLI regressions prove:

- help exposes all required two-state, direction, threshold, format, and output arguments;
- default synthetic breaking change emits JSON and exits `2`;
- `--fail-on none` exits `0` without removing findings;
- SARIF file output is complete before policy exit `2`;
- an existing result is replaceable on pass and fail;
- corrupted cache exits `1` with digest-mismatch evidence;
- invalid `--fail-on` syntax exits `1` rather than `2`;
- missing output parent exits `1`;
- check execution does not acquire from the network in the synthetic CLI tests.

Package regressions separately prove producer and consumer direction selection, `breaking` vs `risky` thresholds, count accuracy, and fail-closed CF-04 authority validation.

## Real FHIR smoke evidence

Exact implementation run `31804887047` passed the complete locked chain:

1. first independent resolve of `hl7.fhir.r4.core@4.0.1`;
2. first cache verification;
3. CF-02 inspect;
4. second independent resolve into a distinct lock/cache state;
5. second cache verification;
6. CF-03 self-diff with `changes == []`;
7. CF-04 self-classification with `findings == []`;
8. CF-05 JSON self-check with `passed == true` and `blocking_findings == 0`;
9. CF-05 SARIF self-check with version `2.1.0`, one commandF run, empty results, passing decision metadata, and `deferred_cf09` source-mapping marker.

Format, locked Clippy with `-D warnings`, and the full locked workspace test suite also passed on that exact implementation head.

## Reviewer truth

### CodeRabbit

A real review on the first green implementation candidate reported one actionable finding: stale existing output could not be replaced atomically. The finding was valid, fixed, regression-tested, replied to with exact evidence, and the thread was resolved.

A later incremental review was requested after the exit-code correction, atomic replacement fix, and strengthened tests. CodeRabbit reported that the review limit was reached and did not perform that incremental review. The attempted range was `4a258503d58940f7407d554fbdeff3d7df7663ba` through `2f959acd08a1878c620498be23d6e4b0adef47e3`.

Therefore:

- original actionable review: performed;
- actionable findings: 1;
- valid findings fixed: 1;
- unresolved review threads: 0 at convergence time;
- incremental final implementation re-review: RATE-LIMITED;
- no unsupported final CodeRabbit PASS is claimed.

CodeRabbit also reported a non-functional docstring coverage warning. It is not a behavioral or acceptance finding and is not treated as certification failure for CF-05.

### Qodo

`/review` was requested on both the initial green implementation and the converged implementation candidate. No Qodo review result or actionable finding was observed at convergence time.

No Qodo PASS is claimed.

### Cubic

Cubic generated PR summaries. Those summaries are informational only and are not treated as compatibility, SARIF, or merge certification.

## Acceptance matrix

| Acceptance | Evidence | State |
| --- | --- | --- |
| help contract | `check_behavior.rs` | PASS |
| real R4 default pass | workflow real smoke | PASS |
| synthetic breaking exit 2 with output | `check_behavior.rs` | PASS |
| risky vs breaking threshold | `check_gate.rs` | PASS |
| producer/consumer filtering | `check_gate.rs` | PASS |
| fail-on none | package + CLI regressions | PASS |
| deterministic JSON/SARIF | repeated independent evaluation regression | PASS |
| SARIF metadata/rules/levels/messages/evidence | strengthened `check_gate.rs` | PASS |
| no fake locations | SARIF regression + serializer model | PASS |
| output before exit 2 | SARIF/output regressions | PASS |
| input/cache/output failures stay exit 1 | parse/cache/missing-parent regressions | PASS |
| CF-01 through CF-04 regressions | full workspace tests | PASS |
| real independent R4 chain | run `31804887047` | PASS |
| reviewer disposition | this record + resolved thread | PASS WITH FINAL RE-REVIEW RATE-LIMITED |
| Draft/no CF-06 | PR/branch governance check required at final gate | PENDING FINAL GATE |

## Scope audit

CF-05 contains no:

- FHIR Validator oracle execution;
- terminology expansion/set-inclusion engine;
- GitHub source-location fabrication;
- FSH source mapping;
- dependency graph/blast-radius engine;
- mapping execution;
- AI or agent semantic authority.

CF-06 remains unauthorized and must not begin as part of this convergence step.

## Final gate

After this docs-only convergence commit:

1. verify PR #6 remains OPEN / DRAFT / UNMERGED with auto-merge disabled;
2. verify base is still exact CF-04 head `ae33586a925023d92b4d58db01663bf26f3bd9a3`;
3. run exact-final-head Format, locked Clippy, full locked workspace tests, and the full independent R4 smoke;
4. verify there are zero unresolved review threads;
5. verify no CF-06 branch/work exists;
6. publish the exact final head, tree, run id, and reviewer truth in the PR body without changing repository contents.

Only after those checks may the state be reported as:

```text
CF-05_COMPLETE_READY_FOR_FOUNDER_REVIEW
```
