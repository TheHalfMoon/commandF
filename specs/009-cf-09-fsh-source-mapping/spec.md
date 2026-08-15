# CF-09 — FSH Source Mapping

Status: converged / final review

## Purpose

CF-09 adds deterministic, evidence-backed mapping from CF-05 compatibility findings to FSH-authored repository source ranges for GitHub review delivery.

CF-09 depends on CF-08. CF-04 remains compatibility authority, CF-05 remains policy/exit authority, and CF-08 remains GitHub presentation authority.

## Exact stack

```text
repository: TheHalfMoon/commandF
base branch: main
canonical main used for reconciliation: f2c331b3f832407b6834aaaa3b5b03ef73b770c9
reconciled implementation tree: b633e2a1d46419614801d0bb3f9671a422df30bd
current merge-candidate head and exact ci/cf06-oracle run ids: PR #10 metadata
```

The current merge candidate preserves the canonical CF-03/04/05/06/07/08 authority boundaries and adds only CF-09 source attribution. Exact current-head workflow evidence is recorded in PR #10 metadata to avoid a self-referential documentation SHA chain.

## User stories

### US1 — Map compatibility findings to FSH source ranges

Given a CF-05 `CheckReport` and a pinned SUSHI `fsh-generated/data/fsh-index.json`, `commandf source-map` emits deterministic source mapping evidence for each compatibility finding.

### US2 — Render verified GitHub physical locations

Given a CF-05 `CheckReport`, a persisted source map, the current SUSHI index, and the current checked-out repository/FSH root, `commandf github-annotations` revalidates the mapping evidence before emitting GitHub annotation locations.

### US3 — Use source mapping through the composite Action

The root composite Action accepts optional source-mapping inputs and, when all are present, generates and verifies source mapping in the same checked-out workspace before rendering annotations.

## Public CLI contract

```text
commandf source-map \
  --input <check-report.json> \
  --fsh-index <fsh-index.json> \
  --repo-root <repository-root> \
  --fsh-root <repo-relative-fsh-root> \
  [--output <mapped-report.json>]
```

Mapped GitHub rendering:

```text
commandf github-annotations \
  --input <check-report.json> \
  --source-map <mapped-report.json> \
  --fsh-index <fsh-index.json> \
  --repo-root <repository-root> \
  --fsh-root <repo-relative-fsh-root>
```

All four mapped-rendering inputs are required together. Without them, CF-08 unmapped annotation behavior remains unchanged.

## SUSHI evidence contract

Pinned study ref:

```text
repository: FHIR/sushi
commit: 31daab4b486915c2650bcde6649c34b019937777
index: fsh-generated/data/fsh-index.json
```

Consumed fields:

```text
outputFile
fshFile
fshName
fshType
startLine
endLine
```

The index is used as machine-readable source-format authority only. SUSHI is not a runtime dependency and no SUSHI source code is copied into commandF.

## Mapping semantics

V1 is intentionally exact and narrow:

```text
current/after tree only
finding identity input: compatibility finding after_filename
index match: exact outputFile equality
location: fshFile + startLine + endLine
```

There is no fallback from `before_filename`, canonical URL, resource id, FSH name/type, element id/path, rule id, or fuzzy similarity.

Each compatibility finding receives exactly one `SourceMappingEntry`:

```text
mapped
unmapped_no_after_filename
unmapped_no_index_entry
```

A mapped entry carries:

```text
file
line
end_line
```

No columns are fabricated. Mapping adds provenance only; it cannot change CF-04 severity, CF-05 policy/decision/exit semantics, CF-07 terminology evidence, or CF-06 oracle evidence.

## Persisted source-map evidence

A `SourceMappedCheckReport` embeds:

- schema/versioned source-index evidence;
- exact validated CF-05 `CheckReport`;
- one mapping entry per compatibility finding;
- SUSHI-index SHA-256 and entry count;
- repository-relative FSH-root identity.

Serialization is deterministic. The mapped-report producer and public decoder share an 80 MiB maximum serialized/input size contract. The public decoder rejects over-limit bytes **before** JSON deserialization.

The serialized source map is derived evidence, not standalone physical-location authority. `github-annotations` must rebuild the expected source map from the current check report, current SUSHI index, current repository root, and current FSH root and require full equality before it emits mapped physical locations.

## Current-tree and stale-evidence boundary

The builder canonicalizes the repository root, FSH root, and mapped source paths and requires all mapped files to remain regular files under both declared roots. It counts current source lines and rejects an exported range whose `endLine` exceeds current EOF.

An invalid, ambiguous, escaping, oversized, or **detectably stale** index (for example, an exported `endLine` beyond the current source EOF) is an operational failure rather than an unmapped success state. A same-length or range-preserving FSH edit can remain numerically compatible with an older range; CF-09 does not claim to detect that case without changed current index/map evidence.

The serialized source map is deterministic derived evidence, not a cryptographic freshness attestation and not standalone physical-location authority.

## Fail-closed and security requirements

CF-09 must fail closed on:

- malformed or wrong-schema source-map JSON;
- source-map input larger than the shared 80 MiB bound, rejected before deserialization;
- source index larger than 16 MiB;
- more than 100,000 SUSHI index entries;
- malformed required SUSHI index fields;
- duplicate normalized `outputFile` keys;
- absolute, drive-style, or `..` traversal paths;
- canonical FSH-root or mapped-file escape from the repository;
- mapped-file escape from the declared FSH root;
- symlink escapes;
- mapped paths that are not regular files;
- zero/invalid/reversed line ranges;
- exported `endLine` beyond current source EOF;
- persisted mapped reports that do not exactly revalidate against current evidence;
- inconsistent source-index digest/entry/root evidence;
- mapped GitHub renderer inputs supplied only partially.

Attacker-controlled source-map diagnostics must remain one bounded stderr line and cannot create a second GitHub workflow command. Finding-controlled annotation properties/data continue to use CF-08 workflow-command escaping.

The Action must use fully quoted argv and no `eval`. On any operational source-map/render failure it must expose no stale report path and must return operational exit 1 rather than converting the failure into a compatibility judgment.

## Performance boundary

Within one mapping pass, repeated findings for the same `after_filename` reuse the already validated `SourceLocation` rather than repeatedly canonicalizing/stat'ing/streaming the same FSH source file.

Render-time rebuilding remains intentional: one producer mapping pass plus one verifier mapping pass is the trust-boundary cost for treating persisted source-map JSON as untrusted. CF-09 does not weaken current-evidence revalidation merely to remove that second pass.

## Acceptance gates

A merge-candidate head must pass:

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

plus:

- CF-08 Action runner security regression;
- CF-09 Action source-map security regression;
- real R4 resolve/verify + inspect/diff/classify/check smoke;
- real terminology self-diff smoke;
- local source-mapped composite Action smoke and output verification;
- dedicated pinned HL7 oracle workflow.

Exact current-head `ci` and `cf06-oracle` run ids are recorded in PR #10 metadata. Historical successful runs certify only their recorded heads.

## Reviewer truth

Reviewer outputs are evidence, not authority. Every substantive finding must be verified against current code and either fixed or explicitly dispositioned with evidence. Historical review results do not certify later heads.

No reviewer PASS is inferred when a reviewer is unavailable, rate-limited, pending, or only reviewed an older commit.

## Explicit deferrals

CF-09 does not implement:

- a custom FSH parser;
- exact rule-line attribution beyond SUSHI definition ranges;
- live SUSHI execution/download;
- non-FSH source mapping;
- SARIF physical-location rewriting in V1;
- dependency graph / blast-radius analysis;
- baseline/suppression semantics;
- mapping execution;
- AI/agent semantic authority.
