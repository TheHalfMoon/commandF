# CF-13 Planning Consistency — Baselines, Suppressions, and Quality Gates

Status: CONSISTENCY_CANDIDATE

This analysis checks `spec.md`, `plan.md`, `tasks.md`, commandF constitution/engineering rules, Master Architecture V2, and the canonical CF-05 contract before any CF-13 production implementation begins.

## Authority alignment

### Roadmap

Master Architecture V2 defines:

```text
CF-13 | baselines/suppression/quality gates | CF-05
```

The planned slice stays within that identity. It does not absorb CF-12 impact, CF-14 source profiling, CF-15 recipes, or CF-16 mapping IR.

### Constitution

The plan satisfies the constitutional requirements:

- **vertical capability** — a shipped `commandf gate` command is the user-visible result;
- **determinism** — exact canonical fingerprints, canonical baseline/suppression digests, deterministic report bytes, no time/random/env fields;
- **fail closed** — malformed/ambiguous baseline or suppression state is rejected;
- **evidence explicit** — current CF-05 evidence remains complete; baseline/suppression provenance and rationale remain visible;
- **precision over noise** — adoption debt can be baseline/suppressed without changing CF-04 severity truth;
- **product/research separation** — no AI or speculative semantic policy becomes authority.

### AGENTS.md

The plan does not silently discard/coerce source information, does not invent compatibility meaning, keeps ruleset/package evidence explicit, adds no unconsumed crate, and includes positive/counterexample/determinism/failure tests.

## CF-05 composition check

No conflict was found with canonical CF-05:

- CF-05 owns current compatibility policy (`direction`, `fail_on`) and its 0/1/2 check exit contract.
- CF-13 calls the existing CF-05 evaluator for current evidence and preserves the complete `CheckReport`.
- CF-13 does not filter or rewrite CF-05 JSON/SARIF.
- CF-13's gate decision is a new adoption-layer decision over finding disposition, not a replacement compatibility classification.
- Existing `commandf check` remains unchanged for users not invoking `gate`.

This separation avoids a conditional CF-05 schema/output change and therefore preserves existing SARIF, source-map, GitHub annotation, and downstream report consumers.

## Baseline semantics check

A baseline is defined as previously accepted **finding evidence**, not as a compatibility oracle and not as a claim that historical findings are safe.

The following ambiguities are explicitly closed:

- baseline policy metadata is not used to decide the current gate;
- only exact package-name and ruleset-compatible valid CF-05 reports are admissible;
- before/after package versions may differ, because otherwise a baseline could not represent historical accepted debt;
- exact semantic fingerprint equality is required;
- duplicate semantic fingerprints fail closed rather than being treated as a set implicitly.

No contradiction with CF-05 evidence completeness was found.

## Fingerprint semantics check

The fingerprint fields include all CF-04 finding semantics/evidence except human-readable message wording.

Consequences are intentional:

- severity escalation becomes new;
- direction change becomes new;
- rule change becomes new;
- resource/source-kind/field/value change becomes new;
- message-only rewording remains the same finding identity;
- filename evidence remains identity-bearing because source-artifact movement can be semantically material for an exact finding record.

Recursive JSON object-key canonicalization is required so map-key insertion order cannot create different fingerprints. Array order remains preserved.

No cryptographic collision recovery mechanism is needed in V1 beyond exact SHA-256 identity; duplicate output fingerprints are treated as ambiguous and rejected.

## Suppression semantics check

Suppressions are exact, explicit waivers keyed to one fingerprint and carrying mandatory rationale.

The plan deliberately excludes:

- wildcards;
- rule-wide/severity-wide/resource-wide selectors;
- executable predicates;
- remote issue/tracker authority;
- clock-based expiry.

This avoids broad accidental evidence hiding and nondeterministic current-time decisions.

Unmatched suppressions are surfaced as unused rather than causing failure. This is consistent with fail-closed safety because an unmatched suppression cannot affect any current finding; a typo leaves the real finding new/blocking.

Suppression precedence over baseline is consistent: explicit waiver provenance remains visible even if the same finding was also historically baselined.

## Gate decision check

No new compatibility threshold semantics are introduced.

For a current finding to block CF-13 it must be:

1. selected by the existing CF-05 direction policy;
2. disposed as `new`;
3. blocking under the existing CF-05 `fail_on` severity threshold.

Baseline/suppressed findings remain evidence but are excluded from blockers by the purpose of the slice.

`fail_on=none` continues to mean no policy blockers.

## CLI / exit consistency

`commandf gate` mirrors the CF-05 CI distinction:

- 0 = completed/pass;
- 1 = parse/input/operational failure;
- 2 = completed/fail.

The existing process boundary already normalizes `check` usage errors to 1. The planned narrow extension to `gate` does not alter other commands.

Atomic output is reused rather than rebuilt.

## Security / trust-boundary consistency

The plan adds no:

- network lookup;
- package acquisition;
- PHI/instance processing;
- arbitrary code execution;
- current-time authority;
- model/agent authority;
- external tracker verification;
- source path fabrication.

Baseline/suppression files are bounded local inputs and diagnostics stay under the existing sanitized CLI boundary.

## Dependency / schema consistency

- no new crate is planned;
- existing `sha2`, `serde`, and `serde_json` are sufficient;
- lock schema remains unchanged;
- CF-04 ruleset/schema remains unchanged;
- CF-05 report schema remains unchanged;
- CF-06 production oracle identity remains unchanged;
- frozen CF-10 corpus remains unchanged.

## Task-order consistency

Task dependencies are coherent:

- planning closes before implementation;
- models/fingerprint precede matching;
- baseline/suppression validation precede disposition;
- disposition precedes decision/report validation;
- library precedes shipped CLI/proof;
- exact-head gates/review precede convergence.

No circular task dependency was found.

## Open questions resolved in planning

### Should CF-13 mutate `commandf check`?

No. A separate `commandf gate` preserves CF-05 byte/schema/consumer stability and makes the adoption-layer policy explicit.

### Should baseline matching use CF-05 policy/decision?

No. A valid baseline contributes compatibility findings only. Current policy controls the current gate.

### Should message text be fingerprinted?

No. Rule/evidence identity is authoritative; prose-only wording churn should not force baseline migration.

### Should suppressions support wildcard selectors?

No in V1. Exact fingerprints minimize accidental over-suppression and are auditable.

### Should suppressions expire by date?

No in V1. Current-time evaluation would violate identical-input byte determinism unless time were made an explicit pinned input. Expiry/profile policy is deferred.

### Should stale suppressions fail the gate?

No. They are retained as unused evidence. They have no authority over current findings and therefore cannot create a false pass.

### Should SARIF omit baseline/suppressed findings?

No. CF-05 SARIF remains complete and unchanged. CF-13 V1 is JSON-only.

## Explicit remaining deferrals

Planning intentionally defers:

- repository/org-wide quality profile files;
- wildcard/rule/resource suppressions;
- explicit pinned-time expiry inputs;
- shared/remote baseline stores;
- SARIF disposition overlays;
- GitHub issue/reference verification;
- multi-package aggregate gates;
- CEL/Rego/CUE policy engines;
- impact-informed severity;
- auto-generated waivers;
- AI-based waiver authority.

These are not accepted CF-13 V1 requirements.

## Consistency decision

```text
SPEC_PLAN_TASKS_CONSISTENT=YES
KNOWN_AUTHORITY_CONTRADICTIONS=0
IMPLEMENTATION_AUTHORIZED_BEFORE_PLANNING_MERGE=NO
```

T004 remains open until this planning exact head receives all path-applicable CI and independent review truth and the planning PR is merged. Only canonical planning may authorize Stack A.