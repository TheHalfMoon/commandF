# CF-06 Implementation Plan

Status: Implemented — convergence evidence is recorded in `convergence.md` and final-head GitHub metadata

## Architecture

CF-06 is a parallel slice directly above converged CF-03. It introduces no new Rust workspace crate and does not import CF-04/05 compatibility policy.

The implementation has four boundaries:

1. CF-03 remains package matching and structural-fact authority.
2. `tools/hl7-oracle/` is an isolated Java adapter pinned to HL7 core `6.10.2`.
3. `commandf-pkg` owns typed oracle evidence, validation, process hardening, and deterministic reconciliation.
4. `commandf oracle` owns explicit two-state CLI wiring and invokes the oracle only where CF-03 has a comparable changed StructureDefinition pair.

Exact CF-03 base:

```text
feat/cf-03-structural-diff
aa212b108e05fa0e22312f244f393c59602192b9
```

## Java adapter

Use Maven with exact dependency versions, never floating ranges:

```text
ca.uhn.hapi.fhir:org.hl7.fhir.r5:6.10.2
ca.uhn.hapi.fhir:org.hl7.fhir.validation:6.10.2
```

The adapter accepts explicit local FHIR package archives plus canonical StructureDefinition identities:

```text
--core-package <hl7.fhir.r4.core@4.0.1.tgz>
--left-package <before-package.tgz>
--right-package <after-package.tgz>
--left-url <canonical>
[--left-version <version>]
--right-url <canonical>
[--right-version <version>]
```

The adapter:

- requires the core context package to be exactly `hl7.fhir.r4.core#4.0.1`;
- loads deterministic R4 worker contexts from the explicit local package archives;
- loads each side package into its side context when it is not the core package itself;
- resolves the requested StructureDefinitions from those contexts;
- creates `ComparisonSession` with `annotate=false`;
- calls the official comparison path through `ComparisonSession.compare(left, right)`;
- requires a `ProfileComparison` result;
- collects public canonical comparison states;
- collects `ProfileComparison.getMessages()`;
- traverses `getCombined()` only through public `StructuralMatch<?>` APIs;
- collects public metadata match messages;
- de-duplicates and deterministically sorts normalized messages;
- emits commandF-owned JSON only.

It must not parse `ComparisonRenderer` HTML, reflect into private node types, serialize generated union/intersection resources, or emit HL7-generated ids/dates/host paths.

The executable adapter artifact is built with Maven Shade. Dependency signature metadata (`META-INF/*.SF`, `*.DSA`, `*.RSA`) is excluded because those signatures are invalid after re-packaging into an uber-JAR; source/library provenance remains pinned separately.

## Adapter model

The Java JSON schema and Rust input model are structurally identical:

```text
OracleIdentity
  project
  release
  source_commit

OracleResourceIdentity
  url
  version
  id
  type

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
  left
  right
  states
  messages
```

State vocabulary is commandF-owned and normalized explicitly:

```text
unknown
not_changed
changed
cannot_evaluate
```

Message levels are normalized to the allowed commandF vocabulary and Rust validation rejects unknown schema/provenance/fields.

## Adapter process contract

Successful adapter invocation:

- stdout: exactly one JSON document plus trailing newline;
- stderr: diagnostics only;
- exit `0`: valid comparison report;
- non-zero: operational failure; no partial JSON is authoritative.

No HTML files are created.

## Rust model and reconciliation

`commandf-pkg` owns:

```text
oracle_model.rs
oracle_error.rs
oracle_reconcile.rs
oracle_process.rs
```

Validation requires:

- schema `1`;
- exact HL7 project/release/source commit;
- bounded structured left/right identities;
- allowed states and message levels;
- at most 100,000 messages;
- at most 64 KiB per normalized string.

Start from one complete CF-03 `StructuralDiffReport`. CF-06 reuses the exact CF-03 matched StructureDefinition helper rather than reconstructing matches from emitted changes.

For matched canonical StructureDefinitions that have one or more CF-03 structural changes:

- invoke the HL7 adapter;
- determine whether HL7 exposes any normalized message or change/cannot-evaluate state;
- classify only the evidence relationship:
  - CF-03 change only -> `commandf_only`;
  - HL7 change only -> `authority_only`;
  - both signal change -> `both_changed`.

For an explicitly compared no-change pair, neither side signaling change is `agreement`.

One-sided resources and unsupported/non-StructureDefinition resources remain `uncomparable`. Oracle operational failure fails the command rather than being converted into `agreement` or `uncomparable`.

For package self-diff with no CF-03 changes, `commandf oracle` validates all inputs and emits an empty CF-06 resource list without launching hundreds of redundant JVM comparisons. A separate real adapter self-equivalence gate proves that the pinned HL7 comparer itself reports no false divergence for an identical R4 StructureDefinition.

## CLI

Public command:

```text
commandf oracle <package-name>
  --before-lock <path>
  --before-cache <path>
  --after-lock <path>
  --after-cache <path>
  --oracle-adapter <path>
  [--oracle-java <path>]
  --format json
```

Rules:

- `--oracle-adapter` must be an explicit existing regular file; do not search `PATH`;
- when the adapter path is a JAR, `--oracle-java` is required and must itself be an explicit existing regular file;
- the adapter/Java paths are validated before lock processing so a no-change run cannot silently accept an unusable adapter;
- both package states must already exist in verified CF-01 caches;
- `hl7.fhir.r4.core@4.0.1` must be present in both states with the same verified digest;
- `commandf oracle` performs no package acquisition.

## Child-process hardening

The process runner has explicit limits:

- timeout: 60 seconds per StructureDefinition pair;
- stdout maximum: 8 MiB per invocation;
- stderr maximum: 1 MiB per invocation;
- adapter message count maximum: 100,000;
- each normalized string maximum: 64 KiB.

The runner:

- never spawns `sh`, `cmd`, or PowerShell to launch the oracle;
- passes arguments directly to an explicit executable;
- captures stdout/stderr concurrently with hard byte caps;
- rejects malformed or oversized output;
- rejects non-zero exit status;
- terminates the adapter process tree on timeout before joining capture readers;
- uses a dedicated Unix process group for the adapter and kills that group on timeout;
- uses explicit `%SystemRoot%\\System32\\taskkill.exe /T /F` on Windows with direct-child kill as fallback;
- never allows descendant processes to retain inherited pipes and extend the promised timeout indefinitely.

## Test strategy

### Rust oracle evidence tests

Cover:

- exact schema/provenance validation;
- malformed adapter JSON;
- wrong oracle identity;
- deterministic message canonicalization;
- evidence count/string bounds;
- `agreement`, `commandf_only`, `authority_only`, `both_changed`, `uncomparable`;
- complete CF-03 report preservation;
- repeated byte-deterministic reconciliation.

### Process-boundary tests

Use tiny executable fixture adapters so ordinary Rust tests require no Maven/network:

- valid pinned JSON;
- JAR without explicit Java fails;
- malformed JSON fails;
- non-zero exit fails with bounded stderr;
- direct infinite adapter times out;
- Unix descendant process retaining inherited pipes (`sleep 60 & wait`) is terminated with the process group and returns promptly.

### CLI tests

Cover:

- `oracle --help` exposes all explicit inputs including `--oracle-java`;
- missing required paths are usage errors;
- missing adapter fails before lock processing;
- missing pinned R4 core context fails closed;
- corrupted before/after cache fails closed;
- no package acquisition during oracle execution.

### Official-oracle integration gate

CI separately:

1. builds `tools/hl7-oracle` against exact HL7 `6.10.2` on Java 17;
2. resolves and verifies `hl7.fhir.r4.core@4.0.1` through commandF;
3. runs the actual shaded adapter on `Patient` vs the identical `Patient` from that verified archive and asserts no messages and no change/cannot-evaluate state;
4. runs `commandf oracle` on a real core self-diff with explicit adapter and Java paths and asserts the embedded CF-03 change list and CF-06 resource list are empty.

The ordinary Rust job continues to preserve the stronger CF-03 reproducibility gate using two independently resolved real R4 states.

## CI

Preserve existing Rust gates:

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
```

The oracle job uses:

- `actions/setup-java@v5.7.0` / Temurin 17;
- Rust `1.97.1` for commandF package resolution;
- Maven adapter dependencies locked to `6.10.2`;
- real R4 package evidence resolved and verified through commandF.

No official 200+ MiB validator fat JAR is committed or required for execution; its official SHA-256 remains provenance evidence for the pinned HL7 release.

## Review priorities

1. no CF-04/05 semantic leakage;
2. no renderer/HTML parsing;
3. no reflection/private HL7 API dependency;
4. exact oracle provenance;
5. process boundary cannot hang or emit unbounded data;
6. descendants cannot outlive the timeout while retaining pipes;
7. external failure cannot become false agreement;
8. CF-03 report remains unmodified;
9. deterministic JSON;
10. no network during `commandf oracle` itself;
11. no generated HL7 ids/dates in normalized evidence.

## Convergence

CF-06 converges only after:

- exact-head Rust + Java + real R4 oracle CI pass;
- CodeRabbit findings are verified, fixed or explicitly rejected with evidence, and all actionable threads are resolved;
- Qodo truth is recorded without inventing a PASS if no review result arrives;
- `spec.md`, `plan.md`, `tasks.md`, and `convergence.md` match the implementation and evidence;
- PR #7 remains Draft/open/unmerged with no auto-merge;
- CF-07 remains unstarted.
