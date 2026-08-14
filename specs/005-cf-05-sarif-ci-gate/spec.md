# CF-05 — SARIF and CI Compatibility Gate

Status: Implemented — convergence candidate

## Purpose

CF-05 turns the deterministic CF-04 compatibility report into a CI-facing decision and a SARIF 2.1.0 interchange artifact.

CF-05 does not change CF-03 structural facts or CF-04 compatibility semantics. It consumes `CompatibilityReport` as authority.

## User-visible command

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

- `--direction both`
- `--fail-on breaking`
- `--format json`
- output to stdout when `--output` is omitted

The command performs no acquisition and reuses the exact CF-03 two-state loader plus the CF-04 classifier.

## Exit contract

`commandf check` has a CI-stable exit contract:

- `0` — evaluation completed and the selected policy passed;
- `1` — operational/input/classification failure;
- `2` — evaluation completed successfully, output was emitted, and the selected policy failed.

A policy failure is not an operational error. `commandf check` usage/parse failures are normalized to exit `1` so exit `2` remains reserved for completed compatibility-policy failures. Existing non-`check` command parsing behavior is preserved.

## Policy semantics

Direction filtering happens before threshold evaluation.

`--direction`:

- `producer` — evaluate only producer findings;
- `consumer` — evaluate only consumer findings;
- `both` — evaluate both directions.

`--fail-on`:

- `breaking` — fail on selected `BREAKING` findings only;
- `risky` — fail on selected `BREAKING` or `RISKY` findings;
- `none` — never fail because of findings.

`ADDITIVE` findings never fail under `breaking` or `risky`.

The policy result includes counts for total, selected, breaking, risky, additive, and blocking findings.

## JSON gate report

JSON output is a versioned CF-05 report containing:

- schema version;
- policy (`direction`, `fail_on`);
- decision (`passed`, counts);
- the complete unmodified CF-04 compatibility report.

Filtering is used only for the decision. Evidence is never removed from the embedded CF-04 report.

## SARIF contract

SARIF output:

- uses SARIF version `2.1.0`;
- includes the OASIS SARIF 2.1.0 Errata 01 schema URI;
- identifies the tool as `commandF`;
- preserves stable CF-04 `rule_id` values as SARIF `ruleId` values;
- maps `BREAKING` -> `error`, `RISKY` -> `warning`, `ADDITIVE` -> `note`;
- preserves the original CF-04 severity value separately in evidence properties;
- emits deterministic result ordering inherited from CF-04;
- carries commandF evidence in SARIF `properties`, including compatibility severity, direction, source change kind, FHIR resource identity, filenames, view, element id, field, and before/after values when present;
- carries CF-05 policy and decision metadata in run properties.

CF-05 MUST NOT invent repository source paths or line numbers. Current compatibility findings identify FHIR package artifacts, not repository source files. Physical GitHub annotations therefore remain deferred to CF-09 source mapping.

SARIF generation is independent of the policy threshold: the artifact contains all CF-04 findings, not only blocking findings.

## Output semantics

When `--output <path>` is supplied, output is written through a same-directory temporary file, synced, then atomically published with a rename over the destination. stdout remains quiet. The parent directory must already exist. Existing output files are replaced only after successful serialization and temporary-file write/sync.

When `--output` is omitted, output is written to stdout.

Output is emitted before returning exit code `2` for a policy failure.

## Determinism

For the same CF-04 input and CF-05 policy:

- JSON output is byte-identical across repeated evaluations;
- SARIF output is byte-identical across repeated evaluations;
- no timestamps, host paths, random ids, run ids, or environment-dependent fields are emitted.

## Fail-closed behavior

CF-05 fails closed on:

- unsupported CF-04 report schema or ruleset;
- malformed policy values;
- serialization failure;
- output write/publication failure;
- any CF-03/CF-04 operational or classification error.

Unknown future CF-04 severities or directions are not silently coerced.

## Acceptance

CF-05 is complete only when all of the following are proven on the exact final head:

1. `commandf check --help` exposes the two-state inputs, direction, threshold, format, and optional output path.
2. Default self-equivalent R4 evaluation exits `0` and reports zero blocking findings.
3. Synthetic breaking findings produce exit `2` under the default policy while still emitting valid JSON/SARIF.
4. `--fail-on risky` fails for RISKY findings; `--fail-on breaking` does not.
5. Direction filtering is proven independently for producer and consumer findings.
6. `--fail-on none` exits `0` regardless of findings.
7. JSON and SARIF serialization are deterministic across repeated evaluations.
8. SARIF contains SARIF 2.1.0 metadata, stable rule ids, levels, messages, and commandF evidence properties while preserving CF-04 severity evidence.
9. SARIF does not invent repository source locations.
10. `--output` writes or replaces complete output before a policy-failure exit and is tested for pass/fail paths.
11. Corrupted cache, invalid policy input, and output-path failures remain exit `1`, not exit `2`.
12. Existing CF-01 through CF-04 commands and tests remain green.
13. Real independent `hl7.fhir.r4.core@4.0.1` resolve/verify/inspect/self-diff/self-classify/check smoke passes with no findings and exit `0`.
14. Review findings are dispositioned and convergence is recorded with exact reviewer truth.
15. PR remains Draft and CF-06 does not start.

## Explicit deferrals

CF-05 does not add:

- FHIR Validator oracle judgments — CF-06;
- terminology set inclusion — CF-07;
- GitHub-native annotations/upload automation — CF-08/CF-09 boundary;
- FSH source mapping or invented physical SARIF locations — CF-09;
- dependency graph or blast radius — CF-11/12;
- AI/agent authority.
