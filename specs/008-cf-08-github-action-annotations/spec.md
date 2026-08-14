# CF-08 — GitHub Action + Bounded Annotations

Status: Approved for implementation

## Purpose

CF-08 turns the converged CF-05 compatibility gate into a directly usable GitHub Action and projects deterministic CF-05 findings into GitHub Actions annotations.

CF-08 is a delivery layer. It does not create new compatibility authority.

## Stack boundary

CF-08 depends on converged CF-05 only.

```text
base branch: feat/cf-05-sarif-ci-gate
base SHA: 9f158f1dcb7d04bc5ee582eabd1ee8dd93bd5019
```

CF-06 oracle behavior and CF-07 terminology semantics are not dependencies. CF-09 source mapping is explicitly deferred.

## Authority

CF-04 remains the compatibility-classification authority. CF-05 remains the policy/exit-code authority.

CF-08 may only:

1. package the existing CF-05 gate as a GitHub Action;
2. render a bounded GitHub-specific projection of an existing valid `CheckReport`;
3. preserve CF-05 exit semantics exactly.

CF-08 MUST NOT reinterpret severity, direction, blocking policy, or compatibility evidence.

## GitHub annotation boundary

GitHub Actions workflow commands support `error`, `warning`, and `notice` annotations. CF-08 maps:

```text
BREAKING -> error
RISKY    -> warning
ADDITIVE -> notice
```

Direction filtering follows the exact `CheckReport.policy.direction` value. `fail_on` controls the final gate decision but does not hide selected non-blocking findings from annotation rendering.

### No fabricated source locations

CF-09 owns FSH/repository source mapping. CF-08 therefore MUST NOT emit a `file`, `line`, `column`, or SARIF physical location for a FHIR artifact finding.

The annotation message may retain package artifact identity such as resource canonical, package filename, element id, view, field, rule id, severity, and direction, but it MUST clearly remain artifact-level evidence.

GitHub may apply its own UI defaults to workflow commands without explicit locations; commandF MUST NOT describe those defaults as source mapping.

CF-08 does not upload CF-05 SARIF to GitHub code scanning because GitHub requires a location to display a code-scanning result, and CF-09 has not yet established repository source locations.

## Annotation bounds

GitHub Actions limits warning/error annotations per step. CF-08 MUST bound its projection independently of the full report.

V1 limits:

```text
max error annotations:   10
max warning annotations: 10
max notice annotations:  10
```

The full CF-05 `CheckReport` remains the complete evidence and policy input. Reaching an annotation limit MUST NOT change:

- `decision.passed`;
- `blocking_findings`;
- the process exit code;
- the complete JSON report.

When selected findings exceed a presentation limit, the renderer emits one deterministic summary notice describing how many findings were not projected for each affected level. It MUST NOT silently imply that the UI projection is complete.

## Workflow-command escaping

Finding-controlled text is untrusted workflow-command input.

CF-08 MUST escape workflow-command data and property values before writing them to stdout. At minimum:

- command data escapes `%`, carriage return, and line feed;
- property values additionally escape `:`, `,`, carriage return, and line feed;
- raw finding text can never create a second workflow command or inject annotation properties.

No shell interpolation of untrusted action inputs is allowed. Composite-action inputs are passed through environment variables and quoted argv positions.

## GitHub projection command

Add a deterministic renderer command:

```text
commandf github-annotations --input <check-report.json>
```

The command:

1. parses `CheckReport` schema v1;
2. validates the embedded CF-04 schema/ruleset through the CF-05 contract;
3. applies only the report's already-recorded direction selection;
4. emits bounded escaped workflow-command lines to stdout;
5. exits `0` for every valid report regardless of whether the policy passed or failed;
6. exits `1` for malformed, unsupported, or inconsistent report evidence.

It does not evaluate a new policy and does not acquire packages.

Repeated rendering of the same report is byte-identical.

## GitHub Action

A repository-root `action.yml` exposes commandF as a composite GitHub Action.

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
fail-on   = breaking
report-path = empty
```

If `report-path` is empty, the action creates a deterministic per-run report location under `RUNNER_TEMP`; the resolved path is returned as an action output.

Outputs:

```text
report-path
exit-code
passed
```

### Action execution sequence

1. obtain/use the pinned commandF Rust toolchain required by this source revision;
2. build the exact commandF source at the referenced Action SHA with `--locked`;
3. run `commandf check ... --format json --output <report>` while capturing, not discarding, exit `0`, `1`, or `2`;
4. when a valid report exists, run `commandf github-annotations --input <report>`;
5. expose outputs;
6. exit with the original CF-05 check code unless annotation rendering itself fails, in which case exit operationally as `1`.

CF-05 semantics therefore remain:

```text
0 = completed policy pass
1 = usage / operational / classification / output failure
2 = completed policy failure after complete output
```

An exit `2` MUST still render available annotations before the Action fails.

## Tool acquisition boundary

`commandf check` itself remains fully offline against the explicit lock/cache inputs.

The source-backed V1 Action may install the repository-pinned Rust toolchain when the runner does not already provide it. That is Action build infrastructure, not FHIR package or terminology acquisition. No registry/source object is created by `check` or `github-annotations`.

A later release-packaging slice may replace source compilation with signed prebuilt binaries without changing this CF-08 report/annotation contract.

## Full-report evidence

The GitHub Action always writes a complete CF-05 JSON report for successful evaluations, including policy failures. The path is exposed even when the policy fails.

Annotations are UI hints only. They are never the canonical evidence bundle and never replace the JSON report.

## Deterministic annotation content

Each projected finding title contains stable commandF identity and the rule id. The message includes deterministic artifact-level context sufficient to understand the finding without claiming source-line location.

No timestamp, random id, runner path, temporary path, GitHub actor, repository secret, or environment-dependent value may enter annotation bytes.

## Fail-closed behavior

CF-08 fails rather than guessing on:

- unsupported `CheckReport` schema;
- unsupported embedded CF-04 schema/ruleset;
- malformed enum/data values;
- invalid action input direction/fail-on values;
- missing required lock/cache/report paths;
- inability to build/run the exact commandF source;
- inability to render a valid report safely.

An operational failure MUST NOT be rewritten to policy exit `2`.

## Acceptance

CF-08 is complete only when the exact final head proves:

1. exact CF-05 base and no CF-06/07 behavior leakage;
2. `commandf github-annotations` deterministic rendering;
3. BREAKING/RISKY/ADDITIVE map to error/warning/notice;
4. direction filtering matches CF-05 exactly;
5. fail-on changes blocking policy but does not hide selected non-blocking annotations;
6. workflow-command injection characters are escaped;
7. no explicit file/line/column is fabricated before CF-09;
8. annotation caps do not alter full-report decision truth;
9. overflow produces deterministic incompleteness notice;
10. malformed/unsupported report fails closed;
11. repository-root composite `action.yml` works on a GitHub-hosted Linux runner;
12. action policy pass returns 0 and emits a complete report;
13. synthetic policy fail renders annotations, preserves a complete report, and returns 2;
14. operational failure returns 1, never 2;
15. action outputs `report-path`, `exit-code`, and `passed` are correct;
16. action inputs are argv-safe and not shell-injected;
17. existing CF-01..05 gates remain green;
18. real public R4 self-check through the packaged Action passes without false annotations;
19. CodeRabbit/Qodo truth is reconciled without invented PASSes;
20. spec/plan/tasks/convergence match exact implementation truth;
21. PR remains Draft/open/unmerged with auto-merge disabled;
22. CF-09 does not start before CF-08 convergence.

## Explicit deferrals

CF-08 does not add:

- FSH/repository source mapping or file/line annotations — CF-09;
- GitHub code-scanning upload based on fabricated SARIF locations;
- public real-IG delta corpus — CF-10;
- ecosystem graph/blast radius — CF-11/12;
- baselines/suppressions — CF-13;
- terminology-server execution;
- mapping execution;
- AI/agent compatibility authority.
