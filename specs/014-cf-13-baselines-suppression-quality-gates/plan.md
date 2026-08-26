# CF-13 Plan — Deterministic Baselines, Suppressions, and Quality Gates

Status: PLANNING_CANDIDATE

## Entry condition

CF-13 is authorized by Master Architecture V2 as `baselines/suppression/quality gates` and depends on CF-05. Canonical CF-05 behavior is already present on main. CF-13 does not depend on the externally blocked CF-06/CF-10 production-oracle path.

The implementation starts only after this planning package passes consistency/review/CI and is merged to canonical main.

## Design summary

CF-13 adds a new `commandf gate` vertical slice rather than changing CF-05 `commandf check` behavior in place.

Reasons:

1. CF-05 JSON/SARIF and exit behavior are already shipped evidence contracts and should remain byte/behavior stable.
2. New-change-first adoption is a separate policy layer over a valid CF-05 current report, not a reinterpretation of compatibility semantics.
3. A separate report schema can preserve baseline/suppression provenance without version-bumping CF-05.
4. `gate` can reuse existing two-state loading/classification/check APIs and existing atomic output plumbing without duplicating semantic engines.

## Proposed implementation surface

Expected library modules in `commandf-pkg`:

- `gate_model.rs` — public V1 schema, suppression schema, dispositions, decisions, evidence;
- `gate.rs` — validation, fingerprinting, canonicalization, baseline/suppression matching, gate evaluation;
- `gate_error.rs` — bounded typed failures;
- `lib.rs` exports.

Expected CLI surface:

- `crates/commandf-cli/src/main.rs` — `gate` parser/execution wiring only;
- focused CLI regressions under `crates/commandf-cli/tests/`.

Expected proof surface:

- `.github/workflows/cf13-quality-gate-proof.yml` with complete path filters for the CF-13 implementation/test/spec surface;
- retained deterministic evidence artifact containing a report digest and fixture/input identity.

No new dependency is expected. `sha2`, `serde`, `serde_json`, and existing atomic-output infrastructure are already direct dependencies/available plumbing.

## Library architecture

### 1. Build the current CF-05 report through existing authority

The CLI follows the current `check` path:

1. load exact before/after lock/cache state using the existing bounded verified loader;
2. build the existing structural diff;
3. classify through CF-04;
4. call `evaluate_compatibility_policy` with the requested CF-05 policy;
5. pass the valid current `CheckReport` into CF-13 evaluation.

No CF-13 function classifies compatibility itself.

### 2. Validate baseline authority

When a baseline is provided:

- bound the input size before allocation;
- parse as `CheckReport`;
- call existing `validate_check_report`;
- require exact current/baseline package-name equality;
- require exact current/baseline CF-04 ruleset equality;
- compute V1 fingerprints for all baseline findings;
- reject duplicate baseline fingerprints.

Baseline direction/fail-on metadata is retained by the baseline artifact but ignored for matching. Only validated compatibility findings constitute baseline evidence.

### 3. Canonical fingerprint key

Add a private/publicly testable commandF-owned key struct with a fixed field order and explicit fingerprint schema version.

Fields are exactly those frozen in `spec.md`: ruleset, rule/severity/direction/source-kind/resource, optional filenames/view/element/field, and before/after values. The message is omitted.

Serialize the key deterministically with `serde_json::to_vec`, hash with `sha2::Sha256`, and format lowercase as `sha256:<hex>`.

`serde_json::Value` object-key determinism must not be assumed accidentally. Before/after JSON values MUST be recursively canonicalized into deterministic object-key order before fingerprint serialization. Arrays preserve their semantic order.

Tests must prove object-key permutations yield the same fingerprint while semantically meaningful array order changes remain distinguishable.

### 4. Suppression model

Public V1 suppression structures:

```text
GateSuppressions {
  schema: 1,
  suppressions: Vec<GateSuppression>
}

GateSuppression {
  finding_fingerprint: String,
  rationale: String,
  reference: Option<String>
}
```

Validation:

- exact schema 1;
- bounded entry count and bounded individual string lengths;
- exact `sha256:` + 64 lowercase hexadecimal syntax;
- trimmed rationale non-empty;
- duplicate fingerprint rejected.

The implementation should choose explicit conservative V1 bounds and expose them as named constants so boundary tests can prove acceptance/rejection. No regex crate is required.

Canonical suppression evidence is produced by sorting entries by fingerprint for digest/output metadata. User input order has no policy meaning.

### 5. Baseline canonical evidence digest

Canonicalize the parsed baseline report using its existing `to_json_bytes()` after validation. Because the embedded compatibility findings already have deterministic ordering, compute SHA-256 over those canonical bytes.

Record a baseline evidence object containing at least:

- canonical SHA-256;
- package name;
- ruleset;
- finding count.

Do not retain the local input path in machine-readable output.

### 6. Suppression canonical evidence digest

After validation and deterministic sorting, serialize the normalized suppression object through a fixed canonical structure and hash it. Record:

- canonical SHA-256;
- entry count.

Do not retain local paths.

### 7. Current finding uniqueness

Compute each current finding fingerprint in CF-04 order. Reject duplicate fingerprints as ambiguous.

This is intentionally stricter than silently deduplicating compatibility evidence. If CF-04 ever produces byte-identical semantic findings twice, the quality gate refuses to guess whether one accepted baseline/suppression identity should cover one or both occurrences.

### 8. Disposition algorithm

Build maps keyed by fingerprint only after uniqueness validation.

For each current finding in original deterministic CF-04 order:

1. if suppression exists -> `suppressed` and attach its rationale/reference;
2. else if baseline contains fingerprint -> `baseline`;
3. else -> `new`.

Track suppression fingerprints that never matched a current finding as deterministic sorted `unused_suppressions`.

### 9. Decision algorithm

Reuse CF-05 policy semantics rather than duplicating independent rule meaning. The CF-13 evaluator may share/expose a crate-private helper from `check.rs` if needed, but any refactor must preserve CF-05 behavior and tests exactly.

For counts/decision:

- all current findings remain in evidence/disposition output;
- `selected` is based on current CF-05 direction;
- only selected findings with disposition `new` can block;
- the threshold is current CF-05 `fail_on`;
- baseline/suppressed findings contribute to their disposition counts but not blockers;
- `fail_on=none` yields zero blockers.

Prefer reusing the current `CheckReport.decision` for total/selected severity counts where semantically identical, but compute new/baseline/suppressed/blocking counts explicitly and test against CF-05 policy behavior.

### 10. Report model

Expected V1 structures:

```text
QualityGateReport
QualityGateDecision
QualityGateFinding
QualityGateDisposition
QualityGateBaselineEvidence
QualityGateSuppressionEvidence
GateSuppressions
GateSuppression
```

`QualityGateReport` embeds the complete current `CheckReport` unchanged and includes the current CF-05 policy explicitly or via that embedded report. Per-finding gate evidence carries fingerprint/disposition and matched suppression metadata.

Output order:

- current findings: original CF-04 order;
- unused suppressions: lexicographic fingerprint order;
- suppression normalization/digest: lexicographic fingerprint order.

### 11. Validation API

Provide a `validate_quality_gate_report` function that recomputes the gate from embedded current evidence and normalized baseline/suppression evidence where enough source content is present, or otherwise validates all internal invariants deterministically.

If the report does not embed the full baseline report/suppression source, validation MUST still verify:

- current CheckReport validity;
- unique current fingerprints;
- per-finding fingerprints match recomputation;
- disposition/count consistency;
- suppression evidence syntax and unique matched identities;
- decision consistency with current policy/dispositions.

Do not claim the digest alone proves unseen external content.

## CLI architecture

Add `Command::Gate` with the exact spec arguments.

CLI steps:

1. parse package/policy arguments;
2. build current CF-05 report using existing functions;
3. bounded-read/parse optional baseline;
4. bounded-read/parse optional suppression file;
5. evaluate CF-13 gate;
6. serialize JSON;
7. publish via existing atomic `write_check_output` helper;
8. return 0/2 from the CF-13 decision.

The process-boundary parse normalization currently special-cases `check`. Extend it narrowly so both `check` and `gate` usage failures return 1 while help remains 0. Do not change other commands' Clap behavior.

V1 JSON only. SARIF remains the complete CF-05 artifact and is not filtered by baseline/suppression state.

## Security / trust boundary

- no network access;
- no arbitrary expressions/scripts;
- bounded baseline and suppression input bytes;
- bounded suppression entry count and string lengths;
- no path values serialized into reports;
- existing sanitized runtime diagnostics retained;
- no PHI/instance fixtures;
- no external tracker lookup for suppression references;
- no current-time evaluation.

A suppression is explicit local policy evidence, not proof that a finding is safe.

## Compatibility / migration impact

- CF-04 model/ruleset unchanged;
- CF-05 `CheckReport` schema unchanged;
- existing `check` CLI arguments/output/exit behavior unchanged;
- existing SARIF/GitHub annotation flows unchanged;
- CF-06 identity and CF-10 corpus unchanged;
- no lockfile schema change;
- no new crate/dependency expected.

## Test plan

### Library positive cases

- current BREAKING finding -> new/blocking;
- exact baseline -> baseline/non-blocking;
- exact suppression -> suppressed/non-blocking with rationale/reference;
- baseline + suppression same fingerprint -> suppression precedence;
- `breaking`, `risky`, `none` parity with CF-05;
- producer/consumer/both parity;
- unused suppression retained;
- empty baseline/no suppressions;
- baseline with different package versions but same package name.

### Library negative/counterexample cases

- severity change invalidates baseline match;
- direction/rule/source-kind/resource/evidence changes invalidate match;
- message-only change preserves fingerprint;
- malformed/uppercase/short fingerprint rejected;
- empty rationale rejected;
- duplicate suppressions rejected;
- duplicate current/baseline fingerprints rejected;
- invalid baseline check schema/decision/ruleset rejected;
- baseline package mismatch rejected;
- unsupported suppression schema rejected;
- input/string/count bounds rejected.

### Determinism

- repeated report bytes equal;
- baseline whitespace/key-order variants canonicalize to same baseline digest where parsed semantics are identical;
- suppression entry order/key-order variants canonicalize to same suppression digest;
- fingerprint JSON object-key permutations equal;
- output remains stable under deterministic fixture repetition.

### CLI

- `gate --help` contract;
- no-baseline BREAKING exit 2 with complete JSON;
- matching baseline exit 0;
- matching suppression exit 0;
- stale suppression still exit 2 when current blocker is new;
- malformed baseline/suppression exit 1;
- gate parse failure exit 1;
- output atomic replace on pass/fail;
- existing `check` behavior regressions unchanged.

### Repository regression

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-features`;
- configured security regressions;
- configured real-FHIR smoke;
- all path-applicable existing proof workflows.

## Dedicated proof workflow

Add `cf13-quality-gate-proof` only in the implementation stack after the library/CLI is functional.

The proof must use pinned runner/toolchain/action identities consistent with repository policy and must:

1. create deterministic local synthetic package states without network acquisition;
2. produce a baseline/current scenario with one accepted old finding and one genuinely new finding;
3. prove the baseline finding does not block;
4. prove the new finding blocks under the selected policy;
5. add an exact suppression and prove only that fingerprint becomes suppressed;
6. execute the same final gate twice and compare bytes;
7. emit a SHA-256 identity artifact;
8. assert the repository remains clean.

Proof output must not contain timestamps/random ids/host paths.

## Delivery stacks

### Planning stack

This Spec Kit package only:

- `spec.md`;
- `plan.md`;
- `tasks.md`;
- `consistency.md`.

No production code in planning PR.

### Stack A — library

Implement fingerprint, baseline/suppression models/validation, gate evaluator, deterministic report, and library contract tests. No CLI yet.

### Stack B — CLI + proof

Add `commandf gate`, bounded input handling, atomic output, exit semantics, integration tests, and dedicated proof workflow.

### Stack C — convergence

Documentation-only final convergence record if exact implementation/run/artifact/reviewer identities need canonical capture.

## Review plan

- CodeRabbit when available;
- Qodo when connected/available;
- other repository-installed reviewers are informational unless governance makes them authoritative;
- every substantive finding is fixed or rejected with contract-grounded reasoning;
- rate limits/unavailability are recorded, never converted into PASS.

Every head mutation invalidates prior exact-head qualification and requires re-reading all path-applicable gates.