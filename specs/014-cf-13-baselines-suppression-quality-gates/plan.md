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
4. `gate` can reuse existing two-state loading/classification/check APIs and existing atomic-output plumbing without duplicating semantic engines.

## Proposed implementation surface

Expected library modules in `commandf-pkg`:

- `gate_model.rs` — public V1 schema, explicit-version fingerprint identity, suppression schema, dispositions, decisions, evidence;
- `gate.rs` — validation, recursive canonicalization, fingerprinting, baseline/suppression matching, gate evaluation;
- `gate_error.rs` — bounded typed failures;
- `lib.rs` exports.

Expected CLI surface:

- `crates/commandf-cli/src/main.rs` — `gate` parser/execution wiring only;
- focused CLI regressions under `crates/commandf-cli/tests/`.

Expected proof surface:

- `.github/workflows/cf13-quality-gate-proof.yml` with complete path filters for the CF-13 implementation/test/spec surface;
- retained deterministic evidence artifact containing report/input/provenance identities and `CF13_GATE_SHA256`.

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

The embedded current `CheckReport` remains the current package-evidence authority and already retains package name, exact before/after versions, archive SHA-256 values, ruleset, findings, policy, and decision. CF-13 must not substitute host-local paths for those identities.

### 2. Validate baseline authority

When a baseline is provided:

- bound the input size before allocation;
- parse as `CheckReport`;
- call existing `validate_check_report`;
- require exact current/baseline package-name equality;
- require exact current/baseline CF-04 ruleset equality;
- compute V1 fingerprints for all baseline findings;
- reject duplicate baseline fingerprints;
- retain baseline before/after `PackageEvidence` and the complete sorted unique baseline fingerprint membership set in CF-13 output.

Baseline direction/fail-on metadata is retained by the baseline artifact but ignored for matching. Only validated compatibility findings constitute baseline evidence.

### 3. Explicit-version canonical fingerprint identity

Add a commandF-owned persisted fingerprint identity:

```text
FindingFingerprint {
  schema: 1,
  digest: "sha256:<64 lowercase hex>"
}
```

The V1 hash preimage uses a fixed field order and includes the fingerprint schema plus exactly the semantic fields frozen in `spec.md`: ruleset, rule/severity/direction/source-kind/resource, optional filenames/view/element/field, and before/after values. The message is omitted.

Before serialization, recursively canonicalize every JSON object in `before`/`after` by lexicographically sorting keys at every depth. Arrays preserve order. Serialize the fixed key structure deterministically, hash with `sha2::Sha256`, and format the digest lowercase.

Every persisted baseline member, suppression selector, current-finding identity, and unused-suppression identity carries `schema: 1` explicitly. Validation rejects unsupported fingerprint schemas before digest comparison. Cross-version fingerprints are never compared as equal merely because their digest strings match.

Tests must prove:

- nested object-key permutations yield the same V1 fingerprint;
- semantically meaningful array-order changes yield a different fingerprint;
- unsupported fingerprint versions fail closed;
- message-only changes preserve identity while all frozen semantic/evidence fields remain identity-bearing.

### 4. Suppression model

Public V1 suppression structures:

```text
FindingFingerprint {
  schema: 1,
  digest: "sha256:<64 lowercase hex>"
}

GateSuppressions {
  schema: 1,
  suppressions: Vec<GateSuppression>
}

GateSuppression {
  finding_fingerprint: FindingFingerprint,
  rationale: String,
  reference: Option<String>
}
```

Validation:

- exact suppression schema 1;
- exact fingerprint schema 1;
- bounded entry count and bounded individual string lengths;
- exact `sha256:` + 64 lowercase hexadecimal digest syntax;
- trimmed rationale non-empty;
- duplicate same-version fingerprint rejected.

The implementation should choose explicit conservative V1 bounds and expose them as named constants so boundary tests can prove acceptance/rejection. No regex crate is required.

Canonical suppression evidence is produced by sorting entries by `(fingerprint.schema, fingerprint.digest)` for digest/output metadata. User input order has no policy meaning.

### 5. Baseline canonical evidence and membership

Do **not** hash `CheckReport::to_json_bytes()` directly as the canonical baseline identity. Nested `serde_json::Value` object insertion order is not an accepted source of evidence identity.

Define one `canonical_json_bytes` helper for CF-13 semantic evidence:

1. serialize the already validated typed value to `serde_json::Value`;
2. recursively rebuild every JSON object with lexicographically sorted keys at every depth;
3. preserve array order exactly;
4. serialize with one fixed compact or otherwise fixed commandF-owned JSON encoding;
5. append no environment-dependent data.

Compute the baseline SHA-256 over those canonical bytes.

`QualityGateBaselineEvidence` V1 contains at least:

- canonical SHA-256;
- fingerprint schema 1;
- package name;
- ruleset;
- exact before `PackageEvidence` (`version`, `archive_sha256`);
- exact after `PackageEvidence` (`version`, `archive_sha256`);
- finding count;
- complete lexicographically sorted unique `Vec<FindingFingerprint>` membership.

This membership is authoritative for validating a persisted `baseline` disposition. The digest is evidence binding, not a substitute for unseen membership.

Do not retain the local baseline input path in machine-readable output.

### 6. Suppression canonical evidence and membership

After validation and deterministic sorting, recursively canonicalize and serialize the normalized suppression object using the same fixed canonical JSON rules, then hash it.

`QualityGateSuppressionEvidence` V1 contains at least:

- canonical SHA-256;
- suppression schema 1;
- fingerprint schema 1;
- entry count;
- normalized complete suppression entries, or an equivalent complete membership structure that retains each exact fingerprint plus rationale/reference and is sufficient to validate every `suppressed` disposition and `unused` fingerprint.

Do not retain local suppression paths.

### 7. Current finding uniqueness

Compute each current finding V1 fingerprint in CF-04 order. Reject duplicate same-version fingerprints as ambiguous.

This is intentionally stricter than silently deduplicating compatibility evidence. If CF-04 ever produces semantically identical findings twice, the quality gate refuses to guess whether one accepted baseline/suppression identity should cover one or both occurrences.

### 8. Disposition algorithm

Build maps keyed by the complete supported-version fingerprint identity only after uniqueness validation.

For each current finding in original deterministic CF-04 order:

1. if a same-version suppression exists -> `suppressed` and attach its rationale/reference;
2. else if retained baseline membership contains the same-version fingerprint -> `baseline`;
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
FindingFingerprint
QualityGateReport
QualityGateDecision
QualityGateFinding
QualityGateDisposition
QualityGateBaselineEvidence
QualityGateSuppressionEvidence
GateSuppressions
GateSuppression
```

`QualityGateReport` embeds the complete current `CheckReport` unchanged and includes the current CF-05 policy explicitly or via that embedded report. Per-finding gate evidence carries explicit-version fingerprint/disposition and matched suppression metadata.

Output order:

- current findings: original CF-04 order;
- baseline membership: lexicographic `(schema, digest)` order;
- unused suppressions: lexicographic `(schema, digest)` order;
- normalized suppression evidence: lexicographic `(schema, digest)` order.

### 11. Validation API

Provide `validate_quality_gate_report` as a true persisted-evidence validator rather than a partial invariant checker.

V1 report evidence is deliberately sufficient to revalidate all disposition authority without an external baseline/suppression file. Validation MUST:

- validate the embedded current `CheckReport`;
- validate supported report, suppression, and fingerprint schemas;
- recompute every current V1 fingerprint from the embedded current finding;
- verify unique current identities;
- verify baseline evidence package/ruleset/before/after identity syntax, canonical digest syntax, count, uniqueness, ordering, and membership set;
- verify suppression evidence canonical digest syntax, count, uniqueness, ordering, rationale/reference bounds, and complete membership;
- require every `baseline` disposition to have an exact member in retained baseline evidence;
- require every `suppressed` disposition and attached suppression metadata to match retained suppression evidence;
- recompute `unused_suppressions` from retained suppression membership and current identities;
- recompute all disposition and decision counts from embedded current evidence plus retained memberships;
- reject altered fingerprints, forged dispositions, count mismatches, decision mismatches, unknown values, and insufficient membership evidence.

The validator does not claim that a digest proves unseen content; it validates the content that the report actually retains and its self-binding invariants.

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
- no host-local path values serialized into reports;
- existing sanitized runtime diagnostics retained;
- no PHI/instance fixtures;
- no external tracker lookup for suppression references;
- no current-time evaluation.

A suppression is explicit local policy evidence, not proof that a finding is safe.

## Immutable authority and provenance binding

Runtime product evidence and repository proof evidence have different scopes and must both be explicit.

### Runtime product evidence

The CF-13 report retains:

- current package name and exact before/after package versions/archive SHA-256 values through the embedded CF-05 report;
- baseline package name, ruleset, exact before/after versions/archive SHA-256 values, canonical baseline digest, and full baseline membership;
- canonical suppression digest and complete normalized suppression membership;
- no host-local lock/cache/baseline/suppression paths.

### Repository proof evidence

The dedicated proof artifact must bind its result to immutable development/runtime authorities by recording:

- commandF exact head SHA and tree SHA;
- repository-relative path plus blob/content SHA for `spec.md`, `plan.md`, `tasks.md`, `.specify/memory/constitution.md`, `AGENTS.md`, and the relevant CF-05/CF-04 source files consumed as implementation authority;
- pinned Rust toolchain version and immutable GitHub Action refs used by the proof workflow;
- `Cargo.lock` repository path plus blob/content SHA and a SHA-256 digest of its exact proof-head bytes;
- exact synthetic source fixture paths relative to the repository/workflow staging root plus SHA-256 content digests;
- exact before/after package name/version/archive SHA-256 identities produced from those fixtures;
- baseline/suppression canonical evidence digests;
- final report digest `CF13_GATE_SHA256`.

Repository-relative paths identify which governed inputs were inspected; immutable SHAs/digests establish identity. Mutable branch names, local absolute paths, timestamps, and floating dependency/action references are insufficient proof identity.

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
- baseline with different package versions but same package name;
- legitimate serialized gate report validates successfully.

### Library negative/counterexample cases

- severity change invalidates baseline match;
- direction/rule/source-kind/resource/evidence changes invalidate match;
- message-only change preserves fingerprint;
- malformed/uppercase/short digest rejected;
- unsupported fingerprint schema rejected before matching;
- empty rationale rejected;
- duplicate suppressions rejected;
- duplicate current/baseline fingerprints rejected;
- invalid baseline check schema/decision/ruleset rejected;
- baseline package mismatch rejected;
- unsupported suppression schema rejected;
- input/string/count bounds rejected;
- forged `baseline` disposition absent from retained membership rejected;
- forged `suppressed` disposition or altered rationale/reference rejected;
- altered persisted current fingerprint rejected;
- baseline/suppression membership count mismatch rejected;
- gate decision/count mismatch rejected;
- unknown disposition/report/fingerprint schema values rejected.

### Determinism

- repeated report bytes equal;
- baseline whitespace/top-level/nested-object-key variants canonicalize to the same baseline digest when parsed semantics are identical;
- array order remains identity-bearing in baseline/fingerprint evidence;
- suppression entry order/key-order variants canonicalize to the same suppression digest;
- fingerprint nested JSON object-key permutations equal;
- validation returns the same accepted/rejected result and bounded error classification for repeated identical persisted inputs;
- output remains stable under deterministic fixture repetition.

### CLI

- `gate --help` contract;
- no-baseline BREAKING exit 2 with complete JSON;
- matching baseline exit 0;
- matching suppression exit 0;
- stale suppression still exit 2 when current blocker is new;
- malformed baseline/suppression exit 1;
- unsupported fingerprint schema exit 1;
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
2. record exact fixture/source-input SHA-256 values and resulting package name/version/archive SHA-256 identities;
3. produce a baseline/current scenario with one accepted old finding and one genuinely new finding;
4. prove the baseline finding does not block and its disposition validates against retained membership;
5. prove the new finding blocks under the selected policy;
6. add an exact suppression and prove only that explicit-version fingerprint becomes suppressed;
7. validate the persisted final report, including retained baseline/suppression membership;
8. execute the same final gate twice and compare bytes;
9. record exact head/tree, governed repository paths with immutable blob/content identities, toolchain/action refs, and dependency lock identity;
10. emit a retained deterministic evidence artifact containing those identities plus `CF13_GATE_SHA256`;
11. assert the repository remains clean.

Proof output must not contain timestamps/random ids/host absolute paths.

## Delivery stacks

### Planning stack

This Spec Kit package only:

- `spec.md`;
- `plan.md`;
- `tasks.md`;
- `consistency.md`.

No production code in planning PR.

### Stack A — library

Implement explicit-version fingerprint identity, canonicalization, baseline/suppression models/validation, membership-bearing evidence, gate evaluator, deterministic/revalidatable report, and library contract/tamper tests. No CLI yet.

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