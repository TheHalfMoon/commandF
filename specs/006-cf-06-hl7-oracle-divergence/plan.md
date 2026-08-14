# CF-06 Implementation Plan

Status: Approved for implementation

## Architecture

CF-06 is a parallel slice above CF-03. It introduces no new Rust workspace crate and does not import CF-04/05.

The implementation has four boundaries:

1. CF-03 remains the package matching and structural-fact authority.
2. `tools/hl7-oracle/` is an isolated Java adapter pinned to HL7 core `6.10.2`.
3. `commandf-pkg` owns typed oracle evidence/reconciliation models and validation.
4. `commandf oracle` owns process invocation and explicit two-state CLI wiring.

## Java adapter

Use Maven with an exact dependency version, not a floating range.

Preferred dependencies:

```text
ca.uhn.hapi.fhir:org.hl7.fhir.r5:6.10.2
ca.uhn.hapi.fhir:org.hl7.fhir.validation:6.10.2
```

Add only dependencies actually required by the final adapter. Keep the adapter outside the Rust dependency graph.

The adapter should:

- parse left/right StructureDefinition JSON;
- construct/load deterministic R4 worker context from pinned local package/context inputs;
- create `ComparisonSession` with `annotate=false`;
- call the official `StructureDefinitionComparer` path through `ComparisonSession.compare(left,right)`;
- require a `ProfileComparison` result rather than accepting placeholder failure as success;
- collect public canonical change states;
- collect `ProfileComparison.getMessages()`;
- traverse `getCombined()` as `StructuralMatch<?>`, recursively collecting only public `ValidationMessage` evidence;
- collect metadata `StructuralMatch<String>` messages;
- de-duplicate normalized messages;
- sort deterministic output;
- emit commandF-owned JSON only.

Do not serialize union/intersection resources because HL7 creates generated ids and dates during comparison. Do not serialize comparison ids because HL7 uses process-global counters/UUID-derived identifiers.

## Adapter model

Use simple Java records/classes whose JSON field names exactly match the CF-06 schema.

```text
OracleIdentity
  project
  release
  source_commit

OracleStates
  metadata
  definitions
  content
  content_interpretation

OracleMessage
  level
  location
  message

Hl7OracleReport
  schema
  oracle
  left_identity
  right_identity
  states
  messages
```

Normalize enum values to stable lowercase commandF vocabulary. Reject unexpected enum values in Rust validation.

## Adapter process contract

Successful adapter invocation:

- stdout: exactly one JSON document plus trailing newline;
- stderr: diagnostics only;
- exit 0: valid comparison report;
- non-zero: operational failure; no partial JSON is authoritative.

No HTML files are created.

## Rust model

Add modules in `commandf-pkg`:

```text
oracle_model.rs
oracle_error.rs
oracle_reconcile.rs
```

The Rust oracle-input model mirrors the Java output and derives `Deserialize` with unknown-field rejection where practical.

Validate:

- schema == 1;
- exact project/release/source commit;
- non-empty left/right identities;
- allowed states;
- allowed message levels;
- bounded message counts and string lengths.

## Reconciliation

Start from one complete CF-03 `StructuralDiffReport`.

Build the set of matched StructureDefinition resource identities from the same deterministic inventory/matching path used by CF-03. Do not reverse-engineer matches from emitted changes alone if an internal helper can safely expose the matched pairs.

For each matched StructureDefinition pair:

- determine whether CF-03 emitted any structural fact for that resource;
- invoke the oracle;
- determine whether HL7 emitted any normalized message or a notable definitions/metadata state;
- classify only the relationship:
  - neither -> `agreement`;
  - CF-03 only -> `commandf_only`;
  - HL7 only -> `authority_only`;
  - both -> `both_changed`.

For unmatched or unsupported resources -> `uncomparable`.

If the adapter itself fails, return an operational error for the command rather than embedding `oracle_error` unless the final design explicitly supports a best-effort batch mode. v1 should prefer fail-closed command behavior.

## CLI

Add:

```text
commandf oracle <package-name>
  --before-lock <path>
  --before-cache <path>
  --after-lock <path>
  --after-cache <path>
  --oracle-adapter <path>
  --format json
```

`--oracle-adapter` must resolve to an explicit existing regular file. Do not search PATH.

The CLI loads and verifies the two package states exactly as CF-03 does, computes CF-03 diff, prepares only matched StructureDefinition inputs, invokes the adapter per pair, validates every response, reconciles, then prints deterministic JSON.

## Child-process hardening

Implement a small process runner with explicit limits.

Initial bounded contract:

- timeout: 60 seconds per StructureDefinition pair;
- stdout maximum: 8 MiB per invocation;
- stderr maximum: 1 MiB per invocation;
- adapter message count maximum: 100,000;
- each normalized string maximum: 64 KiB.

If standard-library process APIs cannot enforce capture bounds without risk of deadlock, use a minimal well-maintained process-timeout implementation or implement concurrent bounded readers explicitly. Do not spawn `sh`, `cmd`, or PowerShell.

Temporary files must be created in a private temporary directory and removed after use. Inputs are exact bytes extracted from already verified package archives.

## Test strategy

### Java adapter tests

- self-equivalent profile;
- cardinality change;
- type change;
- binding change;
- mustSupport change;
- stable repeated JSON;
- no generated ids/dates/paths in output;
- invalid/empty snapshot failure;
- oracle identity exact.

### Rust package tests

- valid adapter report parsing;
- unknown schema/version/source rejection;
- invalid state/level rejection;
- oversized evidence rejection;
- agreement;
- commandf_only;
- authority_only;
- both_changed;
- uncomparable addition/removal/non-StructureDefinition;
- complete CF-03 report preservation;
- deterministic output bytes.

### CLI tests

Use a tiny executable fixture adapter for process-boundary tests so ordinary Rust unit tests do not require Maven/network:

- `oracle --help`;
- missing adapter;
- nonzero adapter;
- malformed stdout;
- wrong oracle identity;
- timeout fixture;
- successful synthetic reconciliation;
- corrupted before/after cache;
- no package acquisition.

### Official-oracle integration gate

CI separately builds/tests `tools/hl7-oracle` against exact HL7 `6.10.2`, then runs one real R4 self-equivalence oracle smoke.

## CI

Preserve existing Rust gates unchanged:

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

Add an oracle job/steps that:

1. install/setup the repository's chosen supported Java version (minimum compatible with official HL7 core; prefer 17 for CI unless Maven proves otherwise);
2. build adapter with Maven dependency version locked to `6.10.2`;
3. run adapter tests;
4. resolve/verify two independent `hl7.fhir.r4.core@4.0.1` states using commandF;
5. choose an identical matched StructureDefinition from the two states;
6. invoke `commandf oracle` using the built adapter;
7. assert no false divergence for that self-equivalent resource/package path.

Do not download/commit the fat validator jar unless the library API proves insufficient. If any release artifact is downloaded, verify its pinned SHA-256 before execution.

## Review priorities

1. no CF-04/05 semantic leakage;
2. no renderer/HTML parsing;
3. no reflection/private HL7 API dependency;
4. exact oracle provenance;
5. process boundary cannot hang or emit unbounded data;
6. external failure cannot become false agreement;
7. CF-03 report remains unmodified;
8. deterministic JSON;
9. no network during `commandf oracle` itself;
10. no generated HL7 ids/dates in normalized evidence.

## Convergence

After implementation:

- exact-head Rust + Java + real R4 oracle CI must pass;
- CodeRabbit/Qodo are requested only on a green candidate;
- valid findings are fixed and exact reviewer truth is recorded;
- `spec.md`, `plan.md`, `tasks.md`, and `convergence.md` are reconciled;
- PR stays Draft;
- CF-07 does not start until this slice converges.
