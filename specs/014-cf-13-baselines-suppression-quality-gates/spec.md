# CF-13 — Baselines, Suppressions, and Quality Gates

Status: PLANNING_CANDIDATE

## Purpose

CF-13 makes commandF adoptable in repositories that already contain known compatibility debt without weakening CF-04 compatibility semantics or CF-05 policy semantics.

The slice adds a deterministic **new-change-first** quality gate. It composes the complete CF-05 `CheckReport`, an optional previously accepted CF-05 baseline, and optional exact finding suppressions. Existing baseline findings and explicitly suppressed findings remain visible evidence but do not block the CF-13 gate. New unsuppressed findings are evaluated with the existing CF-05 direction and severity threshold.

CF-13 does not redefine `BREAKING`, `RISKY`, or `ADDITIVE`, and it does not remove findings from evidence.

## User-visible command

```text
commandf gate <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  [--direction both|producer|consumer] \
  [--fail-on breaking|risky|none] \
  [--baseline <cf05-check-report.json>] \
  [--suppressions <suppressions.json>] \
  [--format json] \
  [--output <path>]
```

Defaults:

- `--direction both`;
- `--fail-on breaking`;
- no baseline;
- no suppressions;
- JSON output;
- stdout when `--output` is omitted.

The command performs no package acquisition or network lookup. It reuses the exact existing two-state package loader, CF-04 classifier, and CF-05 policy evaluator for the current candidate.

## Exit contract

`commandf gate` follows the CI-stable check/gate contract:

- `0` — evaluation completed and the CF-13 quality gate passed;
- `1` — usage, input, baseline, suppression, classification, serialization, or output failure;
- `2` — evaluation completed successfully, output was emitted, and the quality gate failed.

Exit `2` is reserved for a completed quality-gate policy failure. Parse failures for `gate` MUST normalize to exit `1` just as `check` parse failures do.

## Current-report authority

The current candidate is evaluated through the existing CF-05 library API. The resulting valid `CheckReport` is embedded unchanged in the CF-13 report.

CF-13 MUST NOT:

- reimplement structural diff;
- reimplement CF-04 compatibility classification;
- reinterpret CF-05 direction or `fail_on` semantics;
- mutate or filter the embedded current CF-05 evidence.

## Baseline contract

`--baseline` accepts a bounded UTF-8 JSON file containing a valid CF-05 `CheckReport` schema v1.

A baseline is admissible only when:

- the report passes existing CF-05 validation;
- its `package_name` exactly equals the current package name;
- its CF-04 ruleset exactly equals the current report ruleset;
- every finding produces a unique CF-13 fingerprint.

The baseline policy (`direction` / `fail_on`) is not CF-13 authority. The baseline contributes only its validated compatibility findings as previously accepted evidence.

A baseline may represent different before/after package versions from the current candidate. That is the expected adoption use case.

If no baseline is supplied, no current finding is treated as pre-existing.

## Exact finding fingerprint

CF-13 owns a versioned deterministic finding fingerprint used only for baseline/suppression identity. It is not a replacement for CF-04 `rule_id`.

V1 fingerprint input includes:

- fingerprint schema/version;
- CF-04 ruleset;
- `rule_id`;
- compatibility severity;
- compatibility direction;
- structural source change kind;
- exact `ResourceKey` kind/value;
- before filename;
- after filename;
- element view;
- element id;
- field;
- before value;
- after value.

The human-readable message is deliberately excluded: a wording-only message edit MUST NOT invalidate an otherwise identical accepted finding.

The input is serialized through a fixed commandF-owned canonical structure and hashed with SHA-256. The public representation is:

```text
sha256:<64 lowercase hex characters>
```

A severity, direction, rule, resource, source-kind, or evidence-value change therefore creates a different fingerprint and is treated as new unless separately accepted/suppressed.

Duplicate fingerprints in either the current or baseline report are rejected as ambiguous rather than silently collapsed.

## Suppression contract

`--suppressions` accepts bounded UTF-8 JSON with this V1 logical shape:

```json
{
  "schema": 1,
  "suppressions": [
    {
      "finding_fingerprint": "sha256:<64 lowercase hex characters>",
      "rationale": "Approved interoperability exception",
      "reference": "optional external tracking reference"
    }
  ]
}
```

Rules:

- suppression fingerprints MUST use the exact V1 syntax;
- `rationale` MUST be non-empty after trimming and is retained in output;
- `reference` is optional evidence text only and carries no authority;
- duplicate suppression fingerprints fail closed;
- there are no glob, rule-wide, severity-wide, resource-wide, or wildcard suppressions in V1;
- there is no clock-based expiry evaluation in V1, avoiding hidden wall-clock nondeterminism;
- an unmatched/stale suppression is retained as `unused` evidence but does not itself fail the gate;
- a misspelled or stale suppression cannot hide a finding because only exact fingerprint equality suppresses it.

Suppressions do not alter the embedded CF-05 report.

## Finding disposition

Every current finding receives exactly one CF-13 disposition:

1. `suppressed` — an exact suppression entry matches the fingerprint;
2. `baseline` — otherwise, an exact baseline finding matches the fingerprint;
3. `new` — otherwise.

Suppression precedence over baseline is intentional so explicit waiver evidence remains visible when both inputs contain the finding.

Disposition is independent of CF-05 direction selection. All current findings remain classified for auditability.

## Quality-gate decision

The CF-13 decision uses the current CF-05 policy and only **new, unsuppressed** findings.

Direction selection happens exactly as in CF-05. Threshold semantics remain exactly:

- `breaking` blocks selected `BREAKING` findings;
- `risky` blocks selected `BREAKING` or `RISKY` findings;
- `none` blocks no finding;
- `ADDITIVE` does not block under `breaking` or `risky`.

A selected finding with disposition `baseline` or `suppressed` never contributes to `blocking_findings` in CF-13 V1, but remains present in the report.

## CF-13 JSON report

The versioned report contains at least:

- schema version;
- current CF-05 policy;
- gate decision counts;
- the complete unmodified current CF-05 `CheckReport`;
- deterministic baseline evidence when supplied;
- deterministic suppression-file evidence when supplied;
- per-current-finding fingerprint and disposition;
- matched suppression rationale/reference when applicable;
- unused suppression fingerprints;
- enough evidence to distinguish new, baseline, and suppressed findings without repository paths or external lookups.

Decision counts include:

- total current findings;
- selected findings;
- new findings;
- baseline findings;
- suppressed findings;
- new selected breaking/risky/additive findings;
- blocking findings;
- unused suppressions.

The report MUST NOT delete findings merely because they are baseline or suppressed.

## Determinism

For identical pinned package inputs, policy, canonical baseline content, and suppression content:

- fingerprints are byte-stable;
- dispositions are stable;
- JSON output is byte-identical;
- no timestamps, host paths, random ids, run ids, environment fields, or wall-clock decisions are emitted.

Baseline and suppression evidence digests are computed from commandF canonical parsed content rather than original whitespace, so semantically identical JSON formatting does not change the gate result.

## Output semantics

`--output` reuses the existing CF-05 same-directory atomic publication contract. Output is fully written before exit `2` is returned. stdout remains quiet when an output path is supplied.

## Fail-closed behavior

CF-13 fails closed on:

- invalid current CF-05 authority;
- unsupported baseline CF-05 schema/ruleset;
- baseline package-name mismatch;
- duplicate current or baseline fingerprints;
- malformed or oversized baseline/suppression files;
- unsupported suppression schema;
- malformed fingerprint syntax;
- empty suppression rationale;
- duplicate suppression fingerprints;
- serialization/output publication failure;
- any underlying CF-03/CF-04/CF-05 operational failure.

Unknown future disposition/schema values are not coerced.

## Security and trust boundary

- no network access is added;
- no repository source discovery is added;
- no PHI or instance data is required;
- no arbitrary code/predicate execution is allowed in suppressions;
- suppression text is evidence, not executable policy;
- diagnostic output remains bounded/sanitized by existing CLI behavior;
- input size limits are explicit and tested.

## Acceptance

CF-13 is complete only when all of the following are proven on the exact final implementation head:

1. `commandf gate --help` exposes two-state inputs, direction, threshold, optional baseline/suppressions, JSON format, and optional output.
2. With no baseline/suppressions, a current selected BREAKING finding is `new`, blocks under default policy, emits JSON, and exits `2`.
3. An exact matching valid baseline finding is `baseline`, remains in evidence, does not block, and allows exit `0` when no other blockers exist.
4. A changed severity/direction/rule/evidence field does not match the baseline fingerprint and remains new.
5. An exact suppression changes only disposition/gate decision, retains the full CF-05 finding plus rationale/reference evidence, and does not remove it.
6. A stale or misspelled suppression is reported unused and cannot hide a current finding.
7. Duplicate finding fingerprints and duplicate suppression fingerprints fail closed.
8. Baseline package mismatch, unsupported schema/ruleset, malformed suppression schema, invalid fingerprint, and empty rationale exit `1`.
9. Direction and `fail_on` semantics match CF-05 exactly, including `none`.
10. Reordered/whitespace-different equivalent suppression and baseline JSON canonicalize deterministically where semantic ordering is irrelevant.
11. Repeated identical evaluation produces byte-identical report bytes.
12. `--output` atomically replaces an existing file and emits complete output before policy-failure exit `2`.
13. Existing `commandf check` JSON/SARIF bytes and exit semantics remain unchanged.
14. Full workspace format, Clippy, tests, security regressions, and configured real-FHIR smoke remain green.
15. A dedicated CF-13 deterministic proof demonstrates baseline-match, suppression-match, new-finding block, repeated-byte equality, and a clean repository.
16. Independent review findings are dispositioned; reviewer unavailability/rate limits are recorded without invented PASS.
17. Convergence records exact final head/tree, workflow run/job/artifact identities, deterministic proof digest, coverage limits, and explicit deferrals.

## Explicit deferrals / non-goals

CF-13 V1 does not add:

- wildcard, regex, rule-wide, severity-wide, or resource-wide suppressions;
- time-based suppression expiry or current-time policy;
- remote/shared baseline registries;
- GitHub API lookups or issue validation;
- organizational CEL/Rego/CUE policy languages;
- repository-level multi-package quality profiles;
- impact/CF-12 reachability as compatibility severity;
- SARIF filtering that removes accepted findings;
- automatic suppression generation;
- AI/model/agent suppression authority;
- changes to CF-04 classification semantics or CF-05 report schema;
- changes to CF-06 production oracle identity or the frozen CF-10 corpus.

Broader institutional policy/profile systems may build on this exact deterministic gate in a later slice.