# CF-09 — FSH Source Mapping

Status: specified / implementation authorized

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

SUSHI documents this JSON index in code as machine usage. The exported index maps a generated FHIR artifact to the containing FSH definition range. It does not export per-rule source locations.

Therefore CF-09 MUST NOT claim exact rule-line attribution from this index.

## Public capability

CF-09 adds:

```text
commandf source-map \
  --input <check-report.json> \
  --fsh-index <fsh-generated/data/fsh-index.json> \
  --repo-root <repository-root> \
  --fsh-root <repo-relative-fsh-root> \
  [--output <mapped-report.json>]
```

The command performs no package acquisition and no SUSHI execution. It consumes an already generated SUSHI machine index plus repository source files.

CF-08 presentation gains optional source-map consumption:

```text
commandf github-annotations \
  --input <check-report.json> \
  [--source-map <mapped-report.json>]
```

The root Action gains optional inputs:

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
4. resolve the FSH path beneath the explicit repository/FHS root;
5. require the mapped source file to exist as a regular file inside the allowed root.

No fallback is permitted from:

- `before_filename`;
- resource canonical URL;
- resource id;
- FSH definition name;
- element id/path;
- rule id;
- fuzzy filename similarity.

A finding without a proven exact current-source mapping remains unmapped.

## Current-tree rule

GitHub annotations describe the checked-out after/current source tree. Therefore V1 maps only `after_filename`.

A resource removed from the after state normally has no current FSH source location. CF-09 MUST preserve that finding without a fabricated physical location.

## Range precision

A mapped location is the SUSHI-exported FSH definition range:

```text
file=<repo-relative FSH path>
line=<startLine>
endLine=<endLine>
```

Columns are not emitted.

The annotation message states that the range is a definition-level SUSHI mapping, not an exact rule-line proof.

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

An invalid/stale/ambiguous index is an operational failure, not an unmapped success state.

## SUSHI-index validation

The index MUST fail closed when:

- JSON is malformed or not an array;
- an entry lacks a required field;
- required string fields are empty or malformed;
- `startLine` or `endLine` is zero;
- `startLine > endLine`;
- duplicate `outputFile` identities exist;
- `fshFile` is absolute;
- `fshFile` or `fsh-root` contains traversal outside the allowed root;
- a mapped file cannot be canonicalized to a regular file inside the repository/FHS root;
- input bounds are exceeded.

V1 bounds:

```text
check report <= 64 MiB
SUSHI index <= 16 MiB
SUSHI entries <= 100,000
```

Unknown future index fields MAY be ignored because SUSHI's machine index can add metadata without changing the six fields CF-09 owns. The six required fields remain shape-validated.

## Path safety

All emitted GitHub `file` properties are normalized repository-relative paths using `/` separators.

CF-09 rejects:

- absolute FSH roots;
- absolute index `fshFile` values;
- parent traversal escaping the configured FSH root;
- symlink/canonical-path escape outside repository root;
- non-UTF-8 repository-relative paths for V1 output.

No source file content is inserted into annotations.

## Authority preservation

Source mapping cannot change:

- severity;
- direction;
- rule id;
- message;
- blocking counts;
- policy decision;
- CF-05 exit code.

The persisted CF-05 report is revalidated before mapping and again before GitHub projection.

## GitHub presentation

Mapped findings add only proven location properties to the existing bounded CF-08 workflow command:

```text
file
line
endLine
```

CF-08 escaping, 10/10/10 presentation caps, title/message bounds, overflow disclosure, and decision validation remain unchanged.

Unmapped findings remain ordinary artifact-level annotations and explicitly say that no proven current FSH source mapping exists.

## Determinism

For identical check report, SUSHI index, repository tree, and roots, CF-09 output bytes and GitHub annotation bytes MUST be identical.

No timestamps, host-absolute paths, random ids, or environment-dependent ordering may be serialized.

## Review fleet

Required when available:

- CodeRabbit;
- Qodo;
- Ponytail / configured independent code-review lane;
- Codex Code Review with explicit security guidance;
- Codex Security as a separate application-security lane when the repository is enabled for that product.

Codex Security findings are security evidence, not compatibility authority. Codex Security availability MUST NOT replace or waive format, Clippy, tests, deterministic acceptance, or human/founder review. The product currently requires separate repository enablement and is not assumed available merely because Codex Code Review is available.

## Explicit deferrals

CF-09 does not add:

- custom FSH parser or SUSHI replacement;
- exact FSH rule-line attribution absent upstream evidence;
- automatic SUSHI execution/download;
- mapping for non-FSH authoring formats;
- public real-IG delta corpus — CF-10;
- ecosystem graph/blast radius — CF-11/12;
- baselines/suppressions — CF-13;
- AutoFix — CF-15;
- mapping execution or AI semantic authority.
