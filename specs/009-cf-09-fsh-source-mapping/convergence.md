# CF-09 Convergence — FSH Source Mapping

Status: reconciled with canonical main; implementation exact-head gates green; final documentation-head revalidation required before merge.

## Decision

```text
CF-09_RECONCILED_READY_FOR_FINAL_REVIEW_AFTER_DOCS_HEAD_CI
```

CF-09 adds source attribution only. It does not reinterpret structural evidence, compatibility severity, policy, pass/fail decisions, terminology semantics, HL7-oracle evidence, or GitHub Action trust boundaries.

## Current stack identity

```text
repository: TheHalfMoon/commandF
PR: #10
base branch: main
canonical main used for reconciliation: f2c331b3f832407b6834aaaa3b5b03ef73b770c9
implementation branch: feat/cf-09-fsh-source-mapping
pre-reconciliation CF-09 head: 4966f645becfa38cdbd3a94a9f5c201952222ce1
reconciled implementation head: 9907531264227be0c63293d1d1c478ad51b107e2
reconciled implementation tree: b633e2a1d46419614801d0bb3f9671a422df30bd
```

The reconciled commit has the pre-reconciliation CF-09 head as first parent and canonical main as second parent. The final tree was reconstructed from tested Git blob identities and matched the independently tested reconciliation tree byte-for-byte before the CF-09 branch moved.

## Reconciliation proof

CF-09 had exactly three merge-conflict paths against canonical main:

```text
.github/workflows/ci.yml
crates/commandf-cli/src/main.rs
crates/commandf-pkg/src/lib.rs
```

All other CF-09 paths were preserved from the existing CF-09 head. The three conflicts were resolved with these invariants:

- preserve CF-09 source-map CLI, mapped GitHub annotations, diagnostic sanitization, Action wiring, and security checks;
- preserve current-main `terminology` and `oracle` commands and exports;
- preserve the single structural matching authority rather than introducing a competing matcher;
- preserve current mainline CI and add CF-09 source-map security / Action gates without removing terminology coverage.

A guarded GitHub Actions reconciliation independently performed the real three-way merge, asserted the exact conflict set, applied the deterministic resolver, and passed formatting, Clippy, the full workspace tests, CF-08 Action security regression, and CF-09 source-map security regression. That tested tree was exported as an artifact. GitHub's reconstructed tree SHA matched it exactly:

```text
tested tree: b633e2a1d46419614801d0bb3f9671a422df30bd
reconstructed tree: b633e2a1d46419614801d0bb3f9671a422df30bd
```

The temporary reconciliation branch/workflows were not merged into CF-09 and were reset to canonical main after the branch update.

## Upstream source-mapping evidence

CF-09 uses SUSHI's machine-readable source index as source-format authority:

```text
repository: FHIR/sushi
ref: 31daab4b486915c2650bcde6649c34b019937777
machine index: fsh-generated/data/fsh-index.json
fields consumed: outputFile, fshFile, fshName, fshType, startLine, endLine
```

The studied SUSHI implementation maps generated artifacts to FSH definition ranges. It does not provide exact per-rule source locations. CF-09 therefore reports definition-level source ranges only. No SUSHI source code is copied and SUSHI is not a runtime dependency.

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
  [--source-map <mapped-report.json> \
   --fsh-index <fsh-index.json> \
   --repo-root <repository-root> \
   --fsh-root <repo-relative-fsh-root>]
```

The root composite Action accepts optional `fsh-index` and `fsh-root` inputs. With mapping disabled, CF-08 behavior remains unchanged. With mapping enabled, source-map generation and annotation rendering occur in the same checked-out workspace/run.

## Authority boundary

- CF-03 remains deterministic structural-fact authority.
- CF-04 remains compatibility-classification authority.
- CF-05 remains policy/decision/exit-code authority.
- CF-07 terminology remains independent terminology evidence.
- CF-08 remains bounded GitHub presentation authority.
- CF-06 remains advisory pinned-HL7-oracle evidence.
- CF-09 can add source attribution only.

CF-09 cannot modify severity, direction, rule id, compatibility evidence, policy counts, pass/fail decision, original CF-05 exit 0/2, terminology evidence, or oracle evidence. Source-map/render failure is operational failure, not a compatibility judgment.

## Mapping contract

V1 is intentionally narrow:

```text
current/after tree only
finding key: after_filename only
match: exact equality to one SUSHI outputFile
range: fshFile + startLine + endLine
```

There is no fallback from `before_filename`, canonical URL, resource id, FSH definition name, element id/path, rule id, or fuzzy similarity. Unmapped findings remain first-class and locationless. Mapped findings add only file/line/endLine; no columns are fabricated. Current EOF validation detects numerically impossible exported ranges, not same-length/range-preserving source edits; CF-09 does not claim cryptographic source freshness.

Render-time rebuilding from the current index/repository is intentionally retained as a trust boundary for persisted maps. It repeats one full mapping pass across the process boundary by design; the builder now caches validation per repeated `after_filename` so multiple findings for the same generated artifact do not repeatedly canonicalize/stat/scan the same FSH source within a pass.

## Fail-closed and security contract

Implemented safeguards include:

- CF-05 schema/ruleset/decision revalidation;
- exact embedded CheckReport equality before mapped rendering;
- bounded CheckReport, SUSHI index, and persisted mapped-report inputs;
- entry-count and required-field validation;
- duplicate outputFile rejection;
- absolute/drive-style/traversal path rejection;
- repository and FSH-root canonical containment;
- symlink escape rejection;
- regular-file requirement;
- current-file line counting and rejection when exported `endLine` exceeds current EOF;
- persisted mapped-path component containment beneath serialized FSH root;
- repository-relative UTF-8 `/`-separator output paths;
- workflow-command escaping for finding-controlled properties/data;
- fully quoted Action argv with no `eval`;
- operational failure exposes no stale report-path;
- no FSH source content is copied into annotation messages.

Three valid issues found by the manual security-diff methodology were fixed and regression-tested before convergence: persisted mapped-path escape from the declared FSH root, public-library SUSHI-index bound bypass, and stale index `endLine` beyond current EOF.

This remains a manual security-diff audit, not a completed Codex Security product scan.

## Exact reconciled implementation CI

Exact head:

```text
9907531264227be0c63293d1d1c478ad51b107e2
```

Mainline workflow:

```text
workflow: ci
run: 31851096849
result: SUCCESS
```

Passed on the exact reconciled head:

- `cargo fmt --all -- --check`;
- locked workspace Clippy with `-D warnings`;
- full workspace tests;
- CF-08 Action runner security regression;
- CF-09 Action source-map security regression;
- real `hl7.fhir.r4.core@4.0.1` resolve/verify + inspect/diff/classify/check smoke;
- real terminology self-diff smoke;
- real CF-09 source-map fixture preparation;
- local repository-root composite Action `uses: ./` with source mapping enabled;
- Action output verification.

Dedicated HL7 oracle workflow:

```text
workflow: cf06-oracle
run: 31851096862
result: SUCCESS
```

Passed on the same exact CF-09 head:

- pinned HL7 6.10.2 adapter build;
- real R4 context resolve/verify;
- HL7 StructureDefinition self-equivalence;
- `commandf oracle` self-diff;
- deterministic changed-profile fixture construction;
- invalid snapshot fail-closed behavior;
- corrupted before/after cache fail-closed behavior;
- changed-profile evidence determinism;
- end-to-end changed-profile reconciliation.

## Reviewer truth

### CodeRabbit

Historical CF-09 review requests encountered the reviewer PR/rate limit and did not return a substantive final-head review. No CodeRabbit PASS is claimed. A fresh final-head review may be requested after the PR is marked ready.

### Qodo

No substantive Qodo result has been observed. No Qodo PASS is claimed.

### Codex Code Review / Codex Security

No Codex Code Review result has been observed for the final reconciled head. The installed Codex Security methodology informed the manual security audit, but the Codex Security product scan executor was not run in this host. No Codex review or Codex Security PASS is claimed.

### Ponytail / independent reviewer

No Ponytail reviewer result is available in this host. No PASS is claimed.

Reviewer unavailability is recorded rather than substituted with invented certification.

## Explicit deferrals

CF-09 does not implement a custom FSH parser, unsupported exact rule-line mapping, live SUSHI execution/download, non-FSH source mapping, SARIF physical-location rewriting in V1, CF-10 corpus work, graph/blast radius, baselines/suppression, AutoFix, mapping execution, or AI semantic authority.

## Final documentation-head rule

This convergence update changes documentation only. Its resulting repository head must pass both configured workflows again:

```text
ci
cf06-oracle
```

The final documentation head and both final run ids are recorded in PR #10 metadata/body after those workflows settle. A failure reopens convergence. The convergence document intentionally does not self-reference its own future commit SHA/run ids, avoiding an endless documentation-commit chain.
