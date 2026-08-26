# CF-12 Plan — Deterministic Impact Analysis

Status: planning candidate

## Goal

Ship one independently useful vertical slice:

```text
commandf impact <package> \
  --before-lock before.lock \
  --before-cache before-cache \
  --after-lock after.lock \
  --after-cache after-cache \
  --format json
```

The command reports deterministic dependency exposure for a selected package change without promoting graph reachability into compatibility, safety, or clinical authority.

## Canonical prerequisites

CF-12 starts only from canonical main at or after CF-11G closeout merge:

```text
CF-11G closeout main: 8f2ce65de3565a81968bb127c96b451f617593c4
```

Required existing capabilities:

- CF-01 / CF-11 exact package resolution and schema-v2 dependency edges;
- CF-02 bounded verified artifact inspection;
- CF-03 deterministic structural diff evidence;
- CF-04/CF-05 compatibility evidence as an optional linked authority, never recreated here;
- CF-11G deterministic before/after Context Graph construction with explicit `resolved`, `external`, and `ambiguous` reference states.

CF-06/CF-10 upstream HL7 governance is not a dependency of this independent graph-plane slice and must remain unchanged.

## Architecture

### 1. Library-owned report model

Add a CF-12 report model in `commandf-pkg` rather than encoding policy directly in CLI code.

The model should include stable typed records for:

- subject package identity;
- side-specific evidence identities;
- change seeds;
- artifact impact relations;
- package impact relations;
- unresolved boundaries;
- inherited graph extraction coverage.

All public machine-readable structures use deterministic normalized ordering and canonical JSON serialization.

### 2. Reuse existing side inputs

The CLI uses the same explicit before/after lock/cache shape as `diff`, `classify`, `check`, `terminology`, and `oracle`.

No implicit cache, registry, branch, tag, or network state becomes evidence.

Both locks are parsed through the existing lock boundary. Both caches are verified through `PackageCache`. Context Graph construction uses the existing CF-11G library entry point.

### 3. Seed construction

Reuse the existing package archive diff pipeline for the selected package to derive artifact changes.

Do not create a second structural diff engine.

Normalize changed canonical artifacts into side-aware seed records:

- added;
- removed;
- modified with non-empty structural delta.

If existing diff evidence cannot establish the required canonical artifact identity, fail closed or retain an explicitly unsupported seed state rather than guessing from filenames. The implementation task must choose one stable behavior and test it before CLI shipping.

### 4. Reverse artifact traversal

Build deterministic reverse indexes over CF-11G `resolved` canonical-reference edges for each side.

For each seed:

1. enqueue the exact seed identity on its available side(s);
2. find source artifacts whose resolved edge targets the current node;
3. record exposure relation and predecessor evidence;
4. continue until no unseen exact artifact identity remains;
5. terminate cycles by visited exact `(side, artifact identity, seed identity)` state.

Only resolved edges participate in traversal.

Path reporting uses shortest path first. Equal-length ties are resolved lexicographically using stable exact identities.

### 5. Reverse package traversal

Build a reverse index over schema-v2 exact package dependency edges for each side.

Traverse from the selected changed package identity to exact dependent package identities, preserving declared constraints and side evidence.

Do not collapse versions by package name.

Package exposure is reported separately from artifact exposure.

### 6. Unresolved boundary collection

For artifacts that are seeds or become impacted, inspect their outgoing CF-11G reference edges.

Retain `external` and `ambiguous` states in a deterministic unresolved-boundary collection. Never insert them into resolved traversal.

This collection is evidence about analysis limits; it is not a generated compatibility finding.

### 7. Side normalization

Compute before and after evidence independently first.

A relation may be normalized to `both` only if its stable path and evidence identity are identical after normalization. Otherwise retain separate before/after records.

This avoids erasing removed or newly added dependency evidence.

### 8. CLI boundary

Add `Impact` to the existing `Command` enum using current CLI conventions:

- positional package identity string;
- explicit `--before-lock`, `--before-cache`, `--after-lock`, `--after-cache`;
- `--format json` only in V1;
- canonical JSON to stdout;
- sanitized diagnostic to stderr on failure;
- no output file option in the first slice unless implementation evidence shows the existing command pattern requires it.

No new crate is expected for this slice. Existing data structures and standard collections are sufficient unless a concrete implementation task proves otherwise.

## Trust and security boundary

- No PHI or instance data.
- No package acquisition during impact analysis.
- No network canonical resolution.
- Existing archive bounds and verified-cache reads remain authoritative.
- No new unbounded archive reader.
- No model/AI decision path.
- No mutable registry state.
- No automatic compatibility severity derived from reachability.
- Runtime diagnostics remain bounded/sanitized.

## Determinism strategy

Use ordered collections or explicit canonical sorts for all report fields.

Traversal queues may use implementation-efficient structures, but serialized output order MUST be independently normalized.

Canonical shortest-path selection:

1. minimum edge count;
2. lexicographically smallest stable normalized path for equal lengths.

Repeated execution against identical pinned bytes must produce byte-identical report bytes and the same SHA-256.

## Testing strategy

### Library tests

Cover:

- direct reverse artifact exposure;
- transitive exposure;
- cycle termination;
- added and removed seeds;
- exact multi-version package reverse reachability;
- shared package/artifact dependents;
- ambiguous and external boundaries retained but never traversed;
- equal-length path tie-breaking;
- before/after side separation and safe `both` normalization;
- reachability without invented compatibility severity;
- input-order permutations producing identical bytes.

### CLI tests

Cover:

- help/argument contract;
- schema-v1 refusal inherited from Context Graph construction;
- missing/corrupt cache refusal;
- valid deterministic JSON output;
- no registry acquisition in the command path;
- stable sanitized errors;
- repeat-run byte equality.

### Workflow proof

Add a dedicated `cf12-impact-proof` workflow only when the implementation vertical slice exists. It should:

- use an immutable digest-pinned Rust 1.97.1 container;
- use immutable action SHAs with `persist-credentials: false`;
- run a deterministic impact fixture twice;
- compare output bytes exactly;
- emit `CF12_IMPACT_SHA256=<sha256>`;
- assert repository cleanliness;
- upload retained checksum evidence with an immutable GitHub artifact digest.

The workflow path filter must include every code/spec/fixture/workflow path capable of changing the proof result.

## Delivery stack

Keep changes independently reviewable.

### Stack A — library model and traversal

Implement tasks T010–T018:

- report/data model;
- change-seed adapter using existing diff evidence;
- reverse artifact traversal;
- reverse package traversal;
- unresolved boundaries;
- side normalization;
- deterministic serialization and focused library fixtures.

No CLI shipping before the library contract is reviewable.

### Stack B — user-visible CLI and deterministic proof

Implement tasks T020–T025:

- `commandf impact` CLI;
- boundary/failure tests;
- end-to-end fixtures;
- dedicated deterministic workflow proof;
- evidence documentation.

### Stack C — convergence only if needed

Use a docs-only closeout PR after implementation stacks merge if final run/review identities cannot be recorded without moving an already qualified implementation head.

## Migration impact

CF-12 introduces no new lock schema. It consumes schema v2 from CF-11G/CF-11.

Existing commands remain unchanged. No compatibility policy/rule-pack migration is introduced.

## Review plan

Prioritize:

1. any path that silently traverses ambiguous/external edges;
2. same-name multi-version collapse;
3. reachability presented as compatibility severity;
4. removal of before-only evidence or addition of after-only evidence;
5. nondeterministic path selection;
6. unbounded traversal/archive behavior;
7. provenance/digest loss.

Use CodeRabbit when available and Qodo when connected/available. Record reviewer unavailability rather than inventing PASS.

## Acceptance gate before merge

Every implementation PR must pass on its exact candidate head:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

and all applicable repository workflows, including the CF-12 deterministic proof once introduced. Every substantive returned review finding must be fixed or explicitly rejected against the frozen specification with evidence.
