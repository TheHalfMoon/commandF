# CF-09 Implementation Plan — FSH Source Mapping

Status: implementation plan

## Stack

```text
base: feat/cf-08-github-action-annotations
base SHA: 03dbdc956847b9edd2eedd58058894118e338beb
```

## Design

CF-09 is an enrichment layer over persisted CF-05 evidence and CF-08 presentation.

The implementation will add four bounded pieces:

1. typed SUSHI `fsh-index.json` reader + validator;
2. deterministic exact current-tree source mapper;
3. schema-v1 mapped report + `commandf source-map` CLI;
4. optional CF-09 location consumption in `github-annotations` and the root composite Action.

No compatibility rule or policy decision changes are permitted.

## Upstream donor/reference

Use SUSHI machine-index behavior as the source-format authority:

```text
FHIR/sushi
ref: 31daab4b486915c2650bcde6649c34b019937777
paths:
  src/utils/Processing.ts
  src/import/FSHImporter.ts
  src/export/StructureDefinitionExporter.ts
```

Adoption mode is STUDY / ORACLE_BEHAVIOR only. No SUSHI TypeScript source is copied and no runtime dependency on SUSHI is introduced.

The V1 contract consumes SUSHI's emitted `fsh-generated/data/fsh-index.json`; it does not parse FSH syntax itself.

## Rust module shape

Add commandF-owned source mapping types under `commandf-pkg`, expected as:

```text
source_map.rs
source_map_error.rs
source_map_model.rs
```

The exact file split may collapse if a smaller implementation is clearer, but no new workspace crate is authorized.

Core types:

```text
SushiFshIndexEntry
SourceLocation
SourceMappingStatus
SourceMappingEntry
SourceMappedCheckReport
```

The mapped report embeds the complete validated `CheckReport` so locations cannot drift independently from the evidence they annotate.

## Validation reuse

Refactor persisted-check validation into one shared internal/public helper used by:

- CF-08 GitHub renderer;
- CF-09 source mapper.

Do not duplicate the CF-05 decision-consistency logic.

## Mapping algorithm

Build a deterministic map keyed by exact SUSHI `outputFile`.

For each `report.compatibility.findings[i]`:

```text
if after_filename is None:
    status = unmapped_no_after_filename
else if exact after_filename not in index:
    status = unmapped_no_index_entry
else:
    validate/canonicalize mapped FSH path
    status = mapped
    location = repo-relative path + start/end line
```

Duplicate `outputFile` is fatal ambiguity.

The algorithm never selects "closest" matches and never uses `before_filename` for current-tree GitHub locations.

## Filesystem boundary

`source-map` accepts explicit repository root and FSH root.

Validation sequence:

1. repository root must canonicalize to a directory;
2. FSH root must be relative and must canonicalize beneath repository root;
3. each index `fshFile` must be relative;
4. joined source path must canonicalize to a regular file beneath the FSH root and repository root;
5. serialized path is repository-relative UTF-8 with `/` separators.

This rejects symlink escapes and stale source indices.

Unmapped findings do not require any path lookup.

## Input bounds

Read using bounded streaming/file reads before JSON parsing:

```text
CheckReport: 64 MiB
SUSHI index: 16 MiB
entries: 100,000
```

## CLI

Add:

```text
commandf source-map \
  --input <check-report.json> \
  --fsh-index <fsh-index.json> \
  --repo-root <repo> \
  --fsh-root <relative-root> \
  [--output <mapped-report.json>]
```

Output defaults to stdout. When `--output` is used, reuse CF-05 same-directory atomic replace semantics rather than inventing a second publication mechanism.

`source-map` operational failures exit 1; normal Clap usage behavior remains the normal non-check usage contract.

## GitHub renderer integration

Extend:

```text
commandf github-annotations --input <check-report.json>
```

with optional:

```text
--source-map <mapped-report.json>
```

When a source map is supplied:

- its embedded CheckReport must exactly equal the separately supplied report;
- mapped locations add `file`, `line`, and `endLine` properties;
- unmapped findings remain locationless;
- the definition-range limitation is included in mapped messages;
- existing escaping and bounds remain authoritative.

Reject mismatched source-map/check-report pairs.

## Action integration

Add optional Action inputs:

```text
fsh-index = ""
fsh-root = input/fsh
```

If `fsh-index` is non-empty, the Action:

1. runs CF-05 check exactly as before;
2. on completed report exit 0 or 2, runs `commandf source-map` using `$GITHUB_WORKSPACE`;
3. renders annotations with the generated mapped report;
4. preserves original CF-05 exit 0/2 unless source mapping or rendering fails, in which case the Action exits operational 1.

If `fsh-index` is empty, exact CF-08 behavior remains unchanged.

Action wrapper argv remains fully quoted; no input is eval'd.

## SARIF boundary

CF-09 may later enrich SARIF with proven physical locations, but V1 implementation scope is GitHub workflow-command annotations plus a reusable mapped JSON report.

Do not silently change CF-05 SARIF output in this slice unless a separate acceptance task explicitly proves the location model and source artifact URI semantics.

## Tests

Synthetic tests must cover:

- exact outputFile mapping;
- definition start/end lines;
- no after filename;
- missing index entry;
- duplicate outputFile failure;
- malformed field types and line ranges;
- absolute/traversal paths;
- symlink escape;
- missing/non-regular mapped source;
- nested relative FSH paths;
- deterministic bytes;
- full CF-05 evidence preservation;
- decision inconsistency rejection;
- check/source-map mismatch rejection;
- mapped annotation file/line/endLine;
- unmapped annotation remains locationless;
- workflow-command path escaping;
- presentation caps remain unchanged;
- Action source mapping on/off behavior;
- Action exit 0/1/2 preservation.

## Real integration fixture

Add a minimal public/synthetic FSH fixture in repository CI that includes:

- `input/fsh/*.fsh`;
- a SUSHI-shaped `fsh-index.json` fixture with documented provenance;
- matching generated FHIR package artifact filenames;
- one deterministic CF-04/05 finding whose `after_filename` maps to the FSH declaration range.

No PHI and no controlled terminology content.

A live SUSHI invocation is not required in CF-09 CI; the machine-index parser is tested against the pinned upstream shape and synthetic fixtures. This keeps CF-09 offline and avoids unpinned Node/package acquisition in the trusted gate.

## Review fleet

After first exact green implementation candidate:

1. CodeRabbit substantive review request;
2. Qodo `/review` request;
3. configured Ponytail/independent reviewer when available;
4. `@codex review for security vulnerabilities` on the PR;
5. Codex Security scan when the repository is separately enabled/available.

Codex Security is tracked independently from Codex Code Review. Unavailability is recorded, not converted into a fake PASS and not used to waive deterministic gates.

## Convergence

After reviewer dispositions:

- reconcile `spec.md`, `plan.md`, `tasks.md`;
- add `convergence.md`;
- run exact-final-docs-head CI;
- verify PR remains Draft/open/unmerged/auto-merge disabled;
- verify no CF-10 branch before final verdict.
