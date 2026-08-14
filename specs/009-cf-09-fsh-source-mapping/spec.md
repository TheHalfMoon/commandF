# CF-09 — FSH Source Mapping

Status: implemented / convergence candidate

## Purpose

CF-09 adds deterministic, evidence-backed mapping from CF-05 compatibility findings to FSH-authored repository source ranges for GitHub review delivery.

CF-09 depends on CF-08. CF-04 remains compatibility authority, CF-05 remains policy/exit authority, and CF-08 remains GitHub presentation authority.

## Exact stack

```text
base branch: feat/cf-08-github-action-annotations
base SHA: 03dbdc956847b9edd2eedd58058894118e338beb
```

CF-10+ are not part of this slice.

## Upstream source-mapping authority

CF-09 V1 consumes the machine-readable SUSHI index:

```text
fsh-generated/data/fsh-index.json
```

Pinned study source:

```text
repository: FHIR/sushi
ref: 31daab4b486915c2650bcde6649c34b019937777
```

At that ref SUSHI emits entries containing:

```text
outputFile
fshFile
fshName
fshType
startLine
endLine
```

SUSHI writes this JSON index for machine usage. The exported index maps a generated FHIR artifact to the containing FSH definition range. It does not export per-rule source locations.

Therefore CF-09 MUST NOT claim exact rule-line attribution from this index.

## Public capability

```text
commandf source-map \
  --input <check-report.json> \
  --fsh-index <fsh-generated/data/fsh-index.json> \
  --repo-root <repository-root> \
  --fsh-root <repo-relative-fsh-root> \
  [--output <mapped-report.json>]
```

The command performs no package acquisition and no SUSHI execution. It consumes an already generated SUSHI machine index plus repository source files.

CF-08 presentation accepts optional source-map evidence:

```text
commandf github-annotations \
  --input <check-report.json> \
  [--source-map <mapped-report.json>]
```

The root Action accepts optional inputs:

```text
fsh-index
fsh-root
```

When `fsh-index` is empty, CF-08 artifact-level behavior remains unchanged. When supplied, the Action uses `$GITHUB_WORKSPACE` as repository root and requires valid CF-09 mapping before rendering physical locations.

## Mapping rule

For each complete CF-04 finding embedded in the CF-05 report:

1. use `after_filename` only;
2. require exact equality to one SUSHI `outputFile`;
3. map to that entry's `fshFile`, `startLine`, and `endLine`;
4. resolve the FSH path beneath the explicit repository/FSH root;
5. require the mapped source to canonicalize to a regular file inside both roots;
6. require the exported `endLine` to exist in the current source file at mapping time.

No fallback is permitted from `before_filename`, resource canonical/id, FSH name, element path/id, rule id, or fuzzy filename similarity.

A finding without a proven exact current-source mapping remains unmapped.

## Current-tree and range rule

GitHub annotations describe the checked-out after/current source tree. V1 therefore maps only `after_filename`.

A mapped location is the SUSHI-exported FSH definition range:

```text
file=<repo-relative FSH path>
line=<startLine>
endLine=<endLine>
```

Columns are not emitted. The annotation message states that the range is definition-level SUSHI evidence, not an exact rule-line proof.

A removed resource normally has no current FSH location and remains locationless rather than receiving a fabricated path.

## Source-map report

CF-09 emits deterministic schema-v1 JSON containing:

- the complete unmodified CF-05 `CheckReport`;
- SUSHI-index provenance metadata;
- one mapping entry per complete compatibility finding in original order;
- mapping status;
- optional repository-relative source range.

V1 mapping statuses:

```text
mapped
unmapped_no_after_filename
unmapped_no_index_entry
```

An invalid, stale, ambiguous, escaping, or oversized index is an operational failure rather than an unmapped success state.

The serialized source map is deterministic derived evidence, not a cryptographic freshness attestation. At creation time commandF validates the current repository/FSH roots, source existence, canonical containment, and exported line range. A persisted source-map file can later become stale if the repository changes; callers must not treat its index SHA as a signature of current workspace state. The GitHub Action creates and renders mapping evidence within the same checked-out run/workspace.

## SUSHI-index validation and bounds

The index fails closed when:

- JSON is malformed or not an array;
- required fields are missing or malformed;
- required identity strings are empty;
- line numbers are zero or `startLine > endLine`;
- `endLine` exceeds the current source-file line count at mapping time;
- duplicate `outputFile` identities exist;
- `fshFile` or `fsh-root` is absolute or traverses outside its allowed root;
- canonicalization reveals a symlink/path escape;
- a mapped path is absent or not a regular file;
- input bounds are exceeded.

V1 bounds are enforced in the core library as well as CLI entry points:

```text
CheckReport input <= 64 MiB
SUSHI index <= 16 MiB
SUSHI entries <= 100,000
```

Persisted source-map validation also rejects an entry count above 100,000 and requires every serialized mapped path to remain component-wise beneath its declared `source_index.fsh_root` unless that root is `.`.

Unknown future SUSHI index fields MAY be ignored; the six fields owned by CF-09 remain shape-validated.

## Path and publication safety

All emitted GitHub `file` properties are normalized repository-relative paths using `/` separators.

CF-09 rejects absolute roots/files, parent traversal, symlink escape, repository/FSH-root escape, and non-UTF-8 V1 output paths. No FSH source content is inserted into annotations.

Workflow-command properties reuse CF-08 escaping for `%`, CR/LF, `:`, and `,`. Action inputs are passed as quoted argv and are never evaluated as shell fragments.

## Authority preservation

Source mapping cannot change severity, direction, rule id, compatibility message/evidence, blocking counts, CF-05 policy decision, or CF-05 exit code.

The persisted CF-05 report is revalidated before mapping and again before GitHub projection. A supplied mapped report must embed the exact same validated `CheckReport` being rendered.

## GitHub presentation

Mapped findings add only proven location properties to the existing bounded CF-08 workflow command:

```text
file
line
endLine
```

CF-08 escaping, 10/10/10 presentation caps, title/message bounds, overflow disclosure, and decision validation remain unchanged. Unmapped findings remain artifact-level annotations and explicitly state that no proven current FSH source mapping exists.

## Determinism

For identical check report, SUSHI index, repository tree, and roots, CF-09 source-map and annotation bytes are deterministic. No timestamps, host-absolute paths, random ids, or environment-dependent ordering are serialized.

## Review fleet truth rule

CF-09 requests CodeRabbit, Qodo, configured independent/Ponytail review when available, Codex Code Review, and a separate Codex Security application-security lane. Unavailability or rate limiting is recorded exactly and MUST NOT be converted into a PASS.

Codex Security findings are security evidence, not compatibility authority, and never replace format, Clippy, tests, deterministic acceptance, or founder review.

## Explicit deferrals

CF-09 does not add:

- a custom FSH parser or SUSHI replacement;
- exact FSH rule-line attribution absent upstream evidence;
- automatic SUSHI execution/download;
- mapping for non-FSH authoring formats;
- SARIF physical-location enrichment in V1;
- public real-IG delta corpus — CF-10;
- ecosystem graph/blast radius — CF-11/12;
- baselines/suppressions — CF-13;
- AutoFix — CF-15;
- mapping execution or AI semantic authority.
