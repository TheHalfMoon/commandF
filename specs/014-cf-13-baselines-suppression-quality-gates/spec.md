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
- `1` — usage, input, baseline, suppression, classification, serialization, validation, or output failure;
- `2` — evaluation completed successfully, output was emitted, and the quality gate failed.

Exit `2` is reserved for a completed quality-gate policy failure. Parse failures for `gate` MUST normalize to exit `1` just as `check` parse failures do.

## Current-report authority

The current candidate is evaluated through the existing CF-05 library API. The resulting valid `CheckReport` is embedded unchanged in the CF-13 report.

CF-13 MUST NOT:

- reimplement structural diff;
- reimplement CF-04 compatibility classification;
- reinterpret CF-05 direction or `fail_on` semantics;
- mutate or filter the embedded current CF-05 evidence.

The embedded current `CheckReport` retains exact current package name, before/after package versions, archive SHA-256 identities, ruleset, findings, and policy decision. CF-13 must not replace those identities with local file paths.

## Baseline contract

`--baseline` accepts a bounded UTF-8 JSON file containing a valid CF-05 `CheckReport` schema v1.

A baseline is admissible only when:

- the report passes existing CF-05 validation;
- its `package_name` exactly equals the current package name;
- its CF-04 ruleset exactly equals the current report ruleset;
- every finding produces a unique CF-13 V1 fingerprint.

The baseline policy (`direction` / `fail_on`) is not CF-13 authority. The baseline contributes only its validated compatibility findings as previously accepted evidence.

A baseline may represent different before/after package versions from the current candidate. That is the expected adoption use case.

If no baseline is supplied, no current finding is treated as pre-existing.

A persisted CF-13 report that claims any finding disposition `baseline` MUST retain sufficient baseline membership evidence to revalidate that claim without re-reading an unseen external file. V1 therefore retains the validated baseline before/after package identities and the complete sorted set of baseline finding fingerprints in `QualityGateBaselineEvidence`, in addition to the canonical baseline digest. A digest plus count alone is not authoritative membership evidence.

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

The input is serialized through a fixed commandF-owned canonical structure. Any nested JSON object in `before` or `after` is recursively canonicalized by lexicographically sorting object keys while preserving array order. Those canonical bytes are hashed with SHA-256.

Every persisted fingerprint identity uses the explicit versioned object:

```json
{
  "schema": 1,
  "digest": "sha256:<64 lowercase hex characters>"
}
```

The schema is part of the persisted identity, not merely an implicit hash-preimage detail. Consumers MUST reject unsupported fingerprint schema values; they MUST NOT compare a V1 digest against an identity from another fingerprint schema.

A severity, direction, rule, resource, source-kind, or evidence-value change therefore creates a different fingerprint and is treated as new unless separately accepted/suppressed.

Duplicate V1 fingerprints in either the current or baseline report are rejected as ambiguous rather than silently collapsed.

## Suppression contract

`--suppressions` accepts bounded UTF-8 JSON with this V1 logical shape:

```json
{
  "schema": 1,
  "suppressions": [
    {
      "finding_fingerprint": {
        "schema": 1,
        "digest": "sha256:<64 lowercase hex characters>"
      },
      "rationale": "Approved interoperability exception",
      "reference": "optional external tracking reference"
    }
  ]
}
```

Rules:

- every persisted suppression fingerprint MUST carry the explicit supported fingerprint schema;
- fingerprint digests MUST use exact `sha256:` plus 64 lowercase hexadecimal characters;
- unsupported fingerprint schemas fail closed even if the digest text is otherwise valid;
- `rationale` MUST be non-empty after trimming and is retained in output;
- `reference` is optional evidence text only and carries no authority;
- duplicate suppression fingerprints fail closed;
- there are no glob, rule-wide, severity-wide, resource-wide, or wildcard suppressions in V1;
- there is no clock-based expiry evaluation in V1, avoiding hidden wall-clock nondeterminism;
- an unmatched/stale suppression is retained as `unused` evidence but does not itself fail the gate;
- a misspelled or stale suppression cannot hide a finding because only exact supported-version fingerprint equality suppresses it.

Suppressions do not alter the embedded CF-05 report.

## Finding disposition

Every current finding receives exactly one CF-13 disposition:

1. `suppressed` — an exact same-version suppression fingerprint matches;
2. `baseline` — otherwise, an exact same-version baseline fingerprint matches;
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
- per-current-finding explicit-version fingerprint and disposition;
- matched suppression rationale/reference when applicable;
- unused explicit-version suppression fingerprints;
- enough evidence to distinguish and revalidate new, baseline, and suppressed findings without repository paths or external lookups.

`QualityGateBaselineEvidence` V1 contains at least:

- baseline canonical SHA-256;
- fingerprint schema `1`;
- package name;
- CF-04 ruleset;
- exact baseline before package version and archive SHA-256;
- exact baseline after package version and archive SHA-256;
- finding count;
- the complete lexicographically sorted unique set of baseline V1 fingerprint identities.

`QualityGateSuppressionEvidence` V1 contains at least:

- suppression canonical SHA-256;
- suppression schema;
- fingerprint schema `1`;
- entry count;
- normalized suppression entries or equivalent complete membership evidence sufficient to validate every `suppressed` disposition and every `unused` identity.

Local baseline/suppression/lock/cache paths MUST NOT be serialized as authority.

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

## Canonical evidence digests

Baseline and suppression evidence digests are semantic-content digests, not original-file-byte digests.

For the baseline:

1. bound, parse, and validate the CF-05 `CheckReport`;
2. serialize the validated typed report to a JSON value;
3. recursively sort **every JSON object key at every depth**, including nested `before`/`after` evidence values, while preserving array order;
4. serialize that normalized value with one fixed commandF-owned JSON encoding;
5. hash those bytes with SHA-256.

Therefore semantically identical baseline reports differing only in whitespace or JSON object-key insertion order produce the same canonical baseline digest. Array reordering remains identity-bearing.

Suppression evidence is normalized by validating every entry, sorting entries by explicit-version fingerprint identity, recursively canonicalizing JSON objects, and hashing the fixed canonical serialization.

The canonical digest never substitutes for membership evidence needed to validate a persisted disposition.

## Determinism

For identical pinned package inputs, policy, canonical baseline content, and suppression content:

- fingerprints are byte-stable;
- dispositions are stable;
- JSON output is byte-identical;
- no timestamps, host paths, random ids, run ids, environment fields, or wall-clock decisions are emitted.

Baseline and suppression evidence digests are computed from commandF canonical parsed content rather than original whitespace or object insertion order.

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
- unsupported fingerprint schema;
- malformed fingerprint digest syntax;
- empty suppression rationale;
- duplicate suppression fingerprints;
- persisted `baseline` disposition without retained matching baseline membership evidence;
- persisted `suppressed` disposition without retained matching suppression membership evidence;
- persisted fingerprint/count/decision mismatch;
- serialization/output publication failure;
- any underlying CF-03/CF-04/CF-05 operational failure.

Unknown future disposition/schema/fingerprint-version values are not coerced.

## Security and trust boundary

- no network access is added;
- no repository source discovery is added;
- no PHI or instance data is required;
- no arbitrary code/predicate execution is allowed in suppressions;
- suppression text is evidence, not executable policy;
- diagnostic output remains bounded/sanitized by existing CLI behavior;
- input size limits are explicit and tested.

## Provenance and retained proof evidence

Runtime CF-13 evidence retains exact package identity through the embedded current `CheckReport` and baseline evidence: package name, exact versions, ruleset, and archive SHA-256 values. The product report does not serialize host-local lock/cache paths.

The dedicated repository proof artifact additionally MUST bind the execution to immutable repository and input provenance without granting those records runtime policy authority. It records at least:

- exact commandF head SHA and tree SHA;
- repository-relative paths and blob/content SHA identities for the governing CF-13 `spec.md`, `plan.md`, `tasks.md`, constitution, AGENTS.md, and relevant CF-05 implementation authority inspected for the proof;
- pinned Rust/toolchain and GitHub Action identities used by the workflow;
- dependency lockfile identity/digest;
- exact synthetic before/after package names, versions, archive SHA-256 identities, and fixture/source-input SHA-256 values;
- canonical baseline and suppression evidence digests;
- final `CF13_GATE_SHA256`.

No mutable branch name, local host path, timestamp, or floating dependency reference is sufficient as retained proof identity.

## Acceptance

CF-13 is complete only when all of the following are proven on the exact final implementation head:

1. `commandf gate --help` exposes two-state inputs, direction, threshold, optional baseline/suppressions, JSON format, and optional output.
2. With no baseline/suppressions, a current selected BREAKING finding is `new`, blocks under default policy, emits JSON, and exits `2`.
3. An exact matching valid baseline finding is `baseline`, remains in evidence, does not block, and allows exit `0` when no other blockers exist.
4. A changed severity/direction/rule/evidence field does not match the baseline fingerprint and remains new.
5. An exact suppression changes only disposition/gate decision, retains the full CF-05 finding plus rationale/reference evidence, and does not remove it.
6. A stale or misspelled suppression is reported unused and cannot hide a current finding.
7. Duplicate finding fingerprints and duplicate suppression fingerprints fail closed.
8. Baseline package mismatch, unsupported schema/ruleset, malformed suppression schema, unsupported fingerprint schema, invalid fingerprint digest, and empty rationale exit `1`.
9. Direction and `fail_on` semantics match CF-05 exactly, including `none`.
10. Reordered/whitespace-different equivalent suppression and baseline JSON canonicalize deterministically, including nested object-key permutations; semantically meaningful array-order changes remain distinguishable.
11. Repeated identical evaluation produces byte-identical report bytes.
12. `--output` atomically replaces an existing file and emits complete output before policy-failure exit `2`.
13. Existing `commandf check` JSON/SARIF bytes and exit semantics remain unchanged.
14. Persisted report validation accepts a legitimate report and rejects forged baseline/suppressed dispositions, altered fingerprints, count mismatches, decision mismatches, unsupported fingerprint/disposition/schema values, and insufficient membership evidence deterministically.
15. Full workspace format, Clippy, tests, security regressions, and configured real-FHIR smoke remain green.
16. A dedicated CF-13 deterministic proof demonstrates baseline-match, suppression-match, new-finding block, repeated-byte equality, immutable repository/input provenance, and a clean repository.
17. Independent review findings are dispositioned; reviewer unavailability/rate limits are recorded without invented PASS.
18. Convergence records exact final head/tree, workflow run/job/artifact identities, deterministic proof digest, coverage limits, and explicit deferrals.

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