# CF-09 Implementation Plan — FSH Source Mapping

Status: converged / final review

## Stack

```text
canonical base: main
canonical main used for reconciliation: f2c331b3f832407b6834aaaa3b5b03ef73b770c9
reconciled implementation tree: b633e2a1d46419614801d0bb3f9671a422df30bd
Qodo/reviewer-fix tree before final CodeRabbit correction: d36b0042289c3a8c8cad270892a49a251802b89e
current merge-candidate head and exact ci/cf06-oracle run ids: PR #10 metadata
```

## Implemented design

CF-09 is an enrichment layer over persisted CF-05 evidence and CF-08 presentation. It adds:

1. a typed, bounded SUSHI `fsh-index.json` reader/validator;
2. deterministic exact current-tree source mapping;
3. schema-v1 mapped report and `commandf source-map` CLI;
4. optional source-map consumption in `github-annotations` and the root composite Action.

No compatibility rule or policy decision changes were introduced.

## Upstream authority / donor posture

Source-format behavior is studied from:

```text
FHIR/sushi
ref: 31daab4b486915c2650bcde6649c34b019937777
paths:
  src/utils/Processing.ts
  src/import/FSHImporter.ts
  src/export/StructureDefinitionExporter.ts
```

Adoption mode is STUDY / ORACLE_BEHAVIOR only. No SUSHI TypeScript source is copied and SUSHI is not a commandF runtime dependency.

The V1 contract consumes SUSHI's emitted `fsh-generated/data/fsh-index.json`; it does not parse FSH syntax.

## Rust implementation

`commandf-pkg` owns:

```text
source_map.rs
source_map_error.rs
source_map_model.rs
```

The mapped report embeds the complete validated `CheckReport`, one deterministic mapping entry per finding, source-index SHA/count/root evidence, and optional repository-relative definition range.

Persisted CF-05 validation was factored into one shared `validate_check_report` path reused by CF-08 and CF-09.

## Exact mapping algorithm

The implementation builds a deterministic map keyed by exact SUSHI `outputFile`.

For each `report.compatibility.findings[i]`:

```text
if after_filename is None:
    unmapped_no_after_filename
else if exact after_filename is absent:
    unmapped_no_index_entry
else:
    canonicalize the referenced FSH source under explicit roots
    require a regular current source file
    require endLine <= current source line count
    emit mapped repo-relative definition range
```

Duplicate `outputFile` is fatal ambiguity. No fuzzy, canonical, resource-id, FSH-name, element-id/path, rule-id, or `before_filename` fallback exists.

## Filesystem / untrusted-input boundary

Validation performs:

1. repository-root canonicalization to a directory;
2. repository-relative FSH-root parsing and canonical containment;
3. portable relative `fshFile` validation;
4. canonical source containment under FSH root and repository root;
5. regular-file validation;
6. streaming current-file line counting and stale `endLine` rejection;
7. repository-relative UTF-8 `/`-separator serialization.

Absolute paths, `..`, drive-style prefixes, malformed components, and symlink escapes fail closed.

The core library enforces SUSHI-index byte and entry limits before/after parsing as appropriate:

```text
SUSHI index <= 16 MiB
entries <= 100,000
```

CLI inputs additionally retain the CF-05/08 bounded-read contracts. Persisted source-map validation rechecks declared FSH-root containment and entry-count bounds.

## CLI / publication

Implemented command:

```text
commandf source-map \
  --input <check-report.json> \
  --fsh-index <fsh-index.json> \
  --repo-root <repo> \
  --fsh-root <relative-root> \
  [--output <mapped-report.json>]
```

Output defaults to stdout. `--output` reuses the existing same-directory atomic publication path. Operational failures exit 1.

`github-annotations` accepts:

```text
--source-map <mapped-report.json>
```

The source map must embed the exact same validated CheckReport. Mapped findings add only `file`, `line`, and `endLine`; unmapped findings remain locationless. Messages explicitly distinguish definition-range evidence from exact rule-line attribution.

## Action integration

The root Action accepts optional:

```text
fsh-index = ""
fsh-root = input/fsh
```

With mapping enabled the Action runs CF-05 check, creates the CF-09 mapped report in the same checked-out workspace/run, renders annotations with that report, and preserves the original CF-05 exit 0/2. Source-map or renderer failure becomes operational 1. With mapping disabled, CF-08 behavior is unchanged.

All wrapper arguments remain quoted argv; no input is evaluated as shell code. Operational failure continues to expose an empty report-path rather than stale evidence.

## SARIF boundary

CF-09 V1 does not modify CF-05 SARIF physical locations. This slice adds reusable mapped JSON plus GitHub workflow-command physical locations only.

## Regression evidence

Coverage includes:

- exact outputFile mapping and definition ranges;
- no-after-filename and missing-index unmapped states;
- duplicate generated identity;
- malformed JSON/field types/ranges;
- absolute/traversal/drive-like paths;
- symlink escape;
- missing/non-file source;
- persisted mapped-path escape outside declared FSH root;
- `endLine` beyond current source EOF;
- 16 MiB library byte bound before parse;
- 100,000-entry parser and persisted-map bounds;
- deterministic bytes;
- CF-05 decision consistency and embedded-report equality;
- mapped annotation `file/line/endLine`;
- unmapped locationless behavior;
- workflow-command property escaping;
- Action quoted argv, source-map failure, renderer failure, and exit 0/1/2 preservation.

Committed synthetic fixture:

```text
crates/commandf-cli/tests/fixtures/cf09/input/fsh/example.fsh
crates/commandf-cli/tests/fixtures/cf09/fsh-index.json
```

`source_map_fixture.rs` executes the real CLI map→render path against that SUSHI-shaped fixture. CI also exercises the real root `uses: ./` Action with source mapping enabled using a valid empty SUSHI-shaped index for a real R4 self-check state.

No PHI or controlled terminology content is present.

## Security-diff audit disposition

A manual security-diff review using the Codex Security diff-scan threat-model/source-to-sink methodology covered the CF-09 source/script/workflow delta. Three valid issues were found before reviewer freeze and fixed with regressions:

1. a tampered persisted map could move a mapped path outside the declared FSH root while remaining repository-relative — fixed with component-wise FSH-root containment;
2. the 16 MiB index limit existed only at CLI level — fixed in the public core builder before JSON parse;
3. a stale/malicious index could claim an `endLine` beyond the current FSH source — fixed with streaming current-file line validation.

No fourth security finding was identified in the manual audit. This is not represented as a Codex Security product scan: the actual Codex Security scan executor is not exposed in this ChatGPT host.

## Reviewer truth at convergence candidate

- CodeRabbit: substantive review requested twice on exact implementation candidate; request was rate-limited and no substantive review result or inline finding returned.
- Qodo: `/review` requested; no substantive result returned.
- Codex Code Review: `@codex review for security vulnerabilities` requested; no result returned.
- Codex Security: installed workflow/skill applied as manual methodology, but actual product scan NOT RUN in this host because the executor is not exposed; no PASS claimed.
- Ponytail/independent plugin lane: availability checked; no Ponytail plugin/connector was available in this host; no PASS claimed.

Reviewer unavailability does not waive deterministic gates.

## Convergence procedure

Reconcile `spec.md`, `plan.md`, `tasks.md`, add `convergence.md`, run exact docs-head CI, verify PR remains Draft/open/unmerged with auto-merge disabled, verify unresolved threads, and verify CF-10 has not started.
