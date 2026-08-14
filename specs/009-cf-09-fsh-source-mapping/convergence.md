# CF-09 Convergence — FSH Source Mapping

Status: converged; final task-state exact-head revalidation required before founder-review verdict

## Stack identity

```text
repository: TheHalfMoon/commandF
PR: #10
base branch: feat/cf-08-github-action-annotations
exact base SHA: 03dbdc956847b9edd2eedd58058894118e338beb
implementation branch: feat/cf-09-fsh-source-mapping
exact green implementation head: 3819f35116a5bf18070cc00453f34176b549688a
exact green implementation tree: 3d80d4a76d29611a6cde771fc4d343d58e44435b
exact green implementation CI: 31840014291
implementation CI result: SUCCESS
first converged docs head: 79847a2bcb97566dff10f0de1186953998dcb68d
first converged docs CI: 31840724719
first converged docs CI result: SUCCESS
```

The PR remains intentionally Draft/open/unmerged. No merge or auto-merge is authorized by this convergence record.

## Upstream source-mapping evidence

CF-09 uses SUSHI's machine-readable source index as source-format authority:

```text
repository: FHIR/sushi
ref: 31daab4b486915c2650bcde6649c34b019937777
machine index: fsh-generated/data/fsh-index.json
fields consumed: outputFile, fshFile, fshName, fshType, startLine, endLine
```

The studied SUSHI implementation writes this JSON for machine usage. It maps generated FHIR artifacts to FSH definition ranges. It does not export exact per-rule source locations. CF-09 therefore reports definition-level source ranges and explicitly refuses to claim exact rule-line attribution.

No SUSHI source code was copied and SUSHI is not a runtime dependency.

## Shipped capability

```text
commandf source-map \
  --input <check-report.json> \
  --fsh-index <fsh-index.json> \
  --repo-root <repository-root> \
  --fsh-root <repo-relative-fsh-root> \
  [--output <mapped-report.json>]
```

and:

```text
commandf github-annotations \
  --input <check-report.json> \
  [--source-map <mapped-report.json>]
```

The root composite Action accepts optional `fsh-index` and `fsh-root` inputs. With mapping disabled, CF-08 behavior is unchanged. With mapping enabled, source-map generation and annotation rendering occur in the same checked-out workspace/run.

## Authority boundary

CF-09 adds source attribution only.

- CF-04 remains compatibility-classification authority.
- CF-05 remains policy/decision/exit-code authority.
- CF-08 remains bounded GitHub presentation authority.
- CF-09 cannot modify severity, direction, rule id, compatibility evidence, policy counts, pass/fail decision, or the original CF-05 exit 0/2.
- Source-map/render failure becomes operational exit 1.
- The complete validated CF-05 report remains embedded and unchanged.

## Mapping contract

V1 mapping is intentionally narrow:

```text
current/after tree only
finding key: after_filename only
match: exact equality to one SUSHI outputFile
range: fshFile + startLine + endLine
```

There is no fallback from `before_filename`, canonical URL, resource id, FSH definition name, element id/path, rule id, or fuzzy similarity.

Unmapped states are first-class and locationless:

```text
unmapped_no_after_filename
unmapped_no_index_entry
```

Mapped findings add only:

```text
file
line
endLine
```

No columns are emitted.

## Fail-closed and security contract

Untrusted inputs include the CF-05 report, SUSHI index, explicit roots/paths, persisted mapped report, and Action inputs.

Implemented safeguards include:

- CF-05 schema/ruleset/decision revalidation;
- exact embedded CheckReport equality before mapped rendering;
- 64 MiB CheckReport CLI bound;
- 16 MiB SUSHI-index bound enforced in the public core builder before JSON parse as well as CLI;
- 100,000-entry bound during parse and persisted-map validation;
- required SUSHI field and line-range shape validation;
- duplicate outputFile rejection;
- absolute/drive-style/traversal path rejection;
- repository and FSH-root canonical containment;
- symlink escape rejection;
- regular-file requirement;
- streaming current-file line counting and rejection when exported `endLine` exceeds current EOF;
- persisted mapped-path component containment beneath serialized `source_index.fsh_root`;
- repository-relative UTF-8 `/`-separator output paths;
- workflow-command escaping for finding-controlled properties/data;
- fully quoted Action argv with no `eval`;
- operational failure exposes no stale report-path;
- no FSH source content is copied into annotation messages.

A serialized mapped report is deterministic derived evidence, not a cryptographic freshness attestation. It can become stale after creation if the repository changes. The source-index SHA records the consumed index bytes; it is not a signature of later repository state.

## Security-diff findings discovered before reviewer freeze

A manual security-diff audit following the installed Codex Security diff-scan threat-model/source-to-sink method found three valid issues. All were fixed before the reviewer candidate and regression-tested:

1. **Persisted mapped path outside declared FSH root** — VALID / FIXED / REGRESSION-TESTED. Persisted mapped paths must remain component-wise beneath serialized `source_index.fsh_root` unless that root is `.`.
2. **16 MiB index limit bypass through public library API** — VALID / FIXED / REGRESSION-TESTED. The public core builder rejects oversized bytes before JSON parse.
3. **Stale index `endLine` beyond current source EOF** — VALID / FIXED / REGRESSION-TESTED. Current source lines are counted in bounded memory and invalid ranges fail closed.

No fourth security finding was identified in the manual CF-09 source/script/workflow audit.

This manual methodology is not represented as a completed Codex Security product scan:

```text
Codex Security product scan: NOT RUN IN THIS HOST
Codex Security PASS claimed: NO
manual security-diff methodology applied: YES
manual valid findings: 3
manual valid findings fixed: 3
```

## Test and CI evidence

Exact implementation candidate `3819f35116a5bf18070cc00453f34176b549688a`, run `31840014291`, passed:

- `cargo fmt --all -- --check`;
- locked workspace Clippy with `-D warnings`;
- full workspace tests;
- inherited CF-08 Action runner security regression;
- CF-09 source-map Action security regression;
- independent real `hl7.fhir.r4.core@4.0.1` resolve/verify and inspect/diff/classify/check smoke;
- local repository-root composite Action `uses: ./` with source mapping enabled;
- Action output verification.

First converged docs head `79847a2bcb97566dff10f0de1186953998dcb68d`, run `31840724719`, repeated the same gate set and also completed SUCCESS.

Regression coverage includes a committed synthetic FSH fixture and SUSHI-shaped machine index, exact map→render CLI integration, mapped/unmapped states, duplicate identity, malformed shapes, traversal, symlink escape, missing/non-file source, current EOF validation, deterministic bytes, persisted-map tampering, property escaping, and bounds.

## Reviewer truth

### CodeRabbit

A focused substantive review was requested on exact implementation candidate `3819f35116a5bf18070cc00453f34176b549688a`. CodeRabbit selected all 22 changed paths but reported the PR review limit and did not start a substantive review. A retry was posted after the reported window, but no substantive review result or actionable thread returned before convergence closure.

```text
CodeRabbit substantive result: NOT RETURNED / RATE LIMITED
CodeRabbit PASS claimed: NO
```

### Qodo

`/review` was requested. No substantive Qodo result was observed.

```text
Qodo result: NOT RETURNED
Qodo PASS claimed: NO
```

### Codex Code Review

`@codex review for security vulnerabilities` was requested on the exact implementation candidate. No review result was observed.

```text
Codex Code Review result: NOT RETURNED
Codex Code Review PASS claimed: NO
```

### Codex Security

The installed Codex Security diff-scan workflow was used as the methodological framework for the manual security audit. The actual Codex Security scan executor is not exposed in this ChatGPT host and therefore was not run.

```text
Codex Security product result: NOT RUN
Codex Security PASS claimed: NO
```

### Ponytail / independent reviewer

Availability was checked in this host; no Ponytail plugin/connector was available.

```text
Ponytail result: NOT AVAILABLE IN THIS HOST
Ponytail PASS claimed: NO
```

Reviewer unavailability is recorded rather than substituted with invented certification.

## Explicit deferrals

CF-09 does not implement a custom FSH parser, exact rule-line mapping without upstream evidence, live SUSHI execution/download, non-FSH source mapping, CF-05 SARIF physical-location rewriting in V1, CF-10 corpus work, graph/blast radius, baselines/suppression, AutoFix, mapping execution, or AI semantic authority.

## Final evidence rule

The successful converged-docs run above authorizes task-state closure. After T023 is marked complete, the resulting final repository head must receive one final exact-head CI revalidation. That final run and final tree are recorded in PR #10 metadata/body without requiring another repository-content mutation.
