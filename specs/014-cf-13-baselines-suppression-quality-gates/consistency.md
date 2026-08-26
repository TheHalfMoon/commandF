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
- **determinism** — explicit-version canonical fingerprints, recursively canonical baseline/suppression digests, deterministic report bytes, no time/random/env fields;
- **fail closed** — malformed, version-incompatible, ambiguous, tampered, or insufficient baseline/suppression evidence is rejected;
- **evidence explicit** — current CF-05 evidence remains complete; baseline/suppression membership, package identities, provenance, and rationale remain visible;
- **precision over noise** — adoption debt can be baseline/suppressed without changing CF-04 severity truth;
- **product/research separation** — no AI or speculative semantic policy becomes authority.

### AGENTS.md

The plan does not silently discard/coerce source information, does not invent compatibility meaning, keeps ruleset/package/version/content-digest evidence explicit, adds no unconsumed crate, and includes rationale plus positive/counterexample/determinism/tamper/failure tests for public rules.

## CF-05 composition check

No conflict was found with canonical CF-05:

- CF-05 owns current compatibility policy (`direction`, `fail_on`) and its 0/1/2 check exit contract.
- CF-13 calls the existing CF-05 evaluator for current evidence and preserves the complete `CheckReport`.
- The embedded current report already retains current package name, exact before/after versions and archive SHA-256 identities, ruleset, findings, and decision.
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
- exact same-fingerprint-schema semantic identity is required;
- duplicate semantic fingerprints fail closed rather than being treated as a set implicitly;
- baseline evidence retains exact before/after package versions/archive SHA-256 identities;
- baseline evidence retains the complete sorted unique fingerprint membership set, so a persisted `baseline` disposition can be revalidated without trusting an unseen external file;
- the canonical baseline digest binds normalized baseline semantics but is not treated as proof of membership by itself.

No contradiction with CF-05 evidence completeness was found.

## Canonical evidence digest check

The initial planning draft incorrectly proposed hashing `CheckReport::to_json_bytes()` directly for baseline identity. Review correctly identified that nested `serde_json::Value` objects can preserve insertion order, so semantically equivalent nested object-key order could change bytes.

The corrected contract defines one CF-13 semantic canonicalization rule:

1. serialize the validated typed value to JSON value form;
2. recursively sort object keys at every depth;
3. preserve array order;
4. serialize with a fixed commandF-owned JSON encoding;
5. hash those bytes.

The same recursive canonicalization discipline applies to normalized suppression evidence and the JSON-valued components of finding fingerprints. Therefore whitespace and object insertion order are non-authoritative while meaningful array order remains identity-bearing.

## Fingerprint semantics check

The fingerprint fields include all CF-04 finding semantics/evidence except human-readable message wording.

Consequences are intentional:

- severity escalation becomes new;
- direction change becomes new;
- rule change becomes new;
- resource/source-kind/field/value change becomes new;
- message-only rewording remains the same finding identity;
- filename evidence remains identity-bearing because source-artifact movement can be semantically material for an exact finding record.

The persisted identity is explicitly versioned as:

```text
FindingFingerprint { schema: 1, digest: sha256:<64 lowercase hex> }
```

Versioning only an internal preimage would be insufficient because persisted baseline/suppression/current identities could otherwise be compared across incompatible future algorithms. The corrected contract requires schema validation before matching and an explicit cross-version rejection test.

Recursive JSON object-key canonicalization is required so map-key insertion order cannot create different fingerprints. Array order remains preserved.

No cryptographic collision recovery mechanism is needed in V1 beyond exact SHA-256 identity; duplicate output fingerprints are treated as ambiguous and rejected.

## Suppression semantics check

Suppressions are exact, explicit waivers keyed to one explicit-version fingerprint and carrying mandatory rationale.

The plan deliberately excludes:

- wildcards;
- rule-wide/severity-wide/resource-wide selectors;
- executable predicates;
- remote issue/tracker authority;
- clock-based expiry.

This avoids broad accidental evidence hiding and nondeterministic current-time decisions.

Unmatched suppressions are surfaced as unused rather than causing failure. This is consistent with fail-closed safety because an unmatched suppression cannot affect any current finding; a typo leaves the real finding new/blocking.

Suppression precedence over baseline is consistent: explicit waiver provenance remains visible even if the same finding was also historically baselined.

The retained suppression evidence is complete enough to validate every persisted `suppressed` disposition, rationale/reference, and unused identity. A suppression digest/count alone is not treated as membership authority.

## Gate decision and persisted-report validation check

No new compatibility threshold semantics are introduced.

For a current finding to block CF-13 it must be:

1. selected by the existing CF-05 direction policy;
2. disposed as `new`;
3. blocking under the existing CF-05 `fail_on` severity threshold.

Baseline/suppressed findings remain evidence but are excluded from blockers by the purpose of the slice.

`fail_on=none` continues to mean no policy blockers.

The corrected report-validation contract does not accept disposition/count consistency alone. It requires enough retained baseline/suppression membership to recompute authority and rejects:

- forged baseline dispositions;
- forged suppressed dispositions or waiver metadata;
- altered current fingerprints;
- baseline/suppression membership/count mismatches;
- decision mismatches;
- unsupported fingerprint/report/disposition values;
- insufficient retained membership evidence.

A legitimate serialized report is a required positive test, and each tampering class is a required deterministic negative/counterexample test.

## CLI / exit consistency

`commandf gate` mirrors the CF-05 CI distinction:

- 0 = completed/pass;
- 1 = parse/input/validation/operational failure;
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

## Immutable provenance check

Repository review correctly required stronger retained provenance for reproducible proof. The corrected design distinguishes product/runtime evidence from repository proof evidence rather than serializing host-local paths into the product contract.

Runtime CF-13 evidence retains:

- current exact package identities through the embedded CF-05 report;
- baseline package name/ruleset and exact before/after versions/archive SHA-256 identities;
- baseline/suppression canonical digests and complete membership evidence.

The dedicated deterministic proof artifact additionally records:

- exact commandF head/tree;
- repository-relative paths and immutable blob/content identities for CF-13 spec/plan/tasks, constitution, AGENTS.md, and relevant CF-04/CF-05 implementation authority;
- pinned Rust/toolchain and GitHub Action identities;
- `Cargo.lock` repository path/blob identity and exact-byte SHA-256;
- exact synthetic fixture/source relative paths and content SHA-256 values;
- exact before/after package name/version/archive SHA-256 identities;
- baseline/suppression canonical evidence digests;
- final `CF13_GATE_SHA256`.

Mutable branch names, timestamps, host-local absolute paths, or floating dependency/action refs cannot substitute for these identities.

## Dependency / schema consistency

- no new crate is planned;
- existing `sha2`, `serde`, and `serde_json` are sufficient;
- lock schema remains unchanged;
- CF-04 ruleset/schema remains unchanged;
- CF-05 report schema remains unchanged;
- CF-13 introduces only its own report/suppression/fingerprint schema V1;
- CF-06 production oracle identity remains unchanged;
- frozen CF-10 corpus remains unchanged.

## Task-order consistency

Task dependencies are coherent:

- planning closes before implementation;
- models/fingerprint precede matching;
- baseline/suppression validation precede disposition;
- disposition precedes decision/report validation;
- persisted-report validation precedes proof authority;
- library precedes shipped CLI/proof;
- exact-head gates/review precede convergence.

No circular task dependency was found.

## Review-derived corrections

The first independent review of planning head `bbe57c2a3d684f6418bd299b148e126a80054d54` produced five substantive threads. They are treated as valid planning defects and corrected in this head series:

1. **Baseline canonical digest** — replace direct `CheckReport::to_json_bytes()` hashing with recursive semantic JSON object-key canonicalization.
2. **Immutable provenance** — require explicit package/source/governance/dependency identities in retained proof while keeping host-local paths out of runtime product evidence.
3. **Baseline membership authority** — persist the complete baseline fingerprint membership set and validate every `baseline` disposition against it.
4. **Fingerprint schema visibility** — persist fingerprint schema `1` with every fingerprint identity and reject cross-version identities.
5. **Persisted-report tamper tests** — require positive legitimate-report validation plus negative forged disposition/fingerprint/count/decision/unknown-value cases.

These corrections strengthen determinism and fail-closed evidence without widening CF-13 into remote policy, CF-12 impact, or CF-06/CF-10 authority.

Because the PR head changed, all prior exact-head CI and reviewer status is invalidated and must be re-run before T004 can close.

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

### Is a baseline digest/count sufficient to validate a persisted baseline disposition?

No. V1 retains complete baseline fingerprint membership so validation can recompute the disposition without trusting unseen external content.

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
REVIEW_DEFECTS_INCORPORATED=5
IMPLEMENTATION_AUTHORIZED_BEFORE_PLANNING_MERGE=NO
```

T004 remains open until the **new exact planning head** receives all path-applicable CI and independent review truth with zero unresolved substantive findings and the planning PR is merged. Only canonical planning may authorize Stack A.