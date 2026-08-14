# CF-06 — HL7 Comparison Oracle Divergence

Status: Approved for implementation

## Purpose

CF-06 adds an **independent, advisory HL7 comparison oracle** beside CF-03 structural facts. It measures where commandF and the official HL7 Java comparison implementation agree, diverge, cannot be compared, or fail operationally.

CF-06 does **not** replace CF-03 authority and does **not** classify compatibility severity. CF-03 remains commandF's deterministic structural-fact contract. CF-06 records external evidence about those facts.

## Stack boundary

CF-06 depends on CF-03 only.

Exact base:

```text
branch: feat/cf-03-structural-diff
sha: aa212b108e05fa0e22312f244f393c59602192b9
```

CF-04/CF-05 rules, severities, policy decisions, SARIF, and CI exit semantics are out of scope for this slice.

## Pinned oracle identity

Official project:

```text
https://github.com/hapifhir/org.hl7.fhir.core
```

Pinned release:

```text
6.10.2
```

Pinned source commit:

```text
d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
```

Official validator release artifact evidence:

```text
validator_cli.jar
sha256: a3addadf23fac3ab7acf74b63730ceb02c9073564235d29fca98d899c9e4ccd6
```

The CF-06 adapter should prefer the published Java libraries at exactly `6.10.2`; commandF MUST NOT commit the 200+ MiB validator jar. The jar digest is recorded as release provenance, not as a requirement to execute the fat jar when the library API is sufficient.

Changing the oracle version is a contract change and requires an explicit later slice/reconciliation.

## Official structured comparison surface

CF-06 uses the official structured comparison model **before rendering**, not generated HTML.

At 6.10.2:

- `ComparisonSession.compare(...)` dispatches `StructureDefinition` pairs to `StructureDefinitionComparer`;
- `StructureDefinitionComparer.compare(...)` returns `ProfileComparison`;
- `ProfileComparison` exposes canonical comparison state and a `StructuralMatch` tree;
- `StructuralMatch` exposes public children and `ValidationMessage` lists;
- canonical comparison metadata is exposed through `getMetadata()` and change-state accessors;
- `ValidationMessage` carries issue severity, location/path, and message text;
- `ComparisonRenderer` is a presentation layer and MUST NOT be parsed as oracle data.

CF-06 MUST use only public API surfaces. No reflection into private `ElementDefinitionNode` internals is permitted.

## Oracle adapter contract

Add a small isolated Java adapter under `tools/hl7-oracle/`.

The adapter accepts one invocation containing two normalized `StructureDefinition` JSON files and produces exactly one UTF-8 JSON document on stdout.

Recommended CLI:

```text
java -jar commandf-hl7-oracle.jar \
  --left <StructureDefinition.json> \
  --right <StructureDefinition.json>
```

The adapter output schema is commandF-owned and versioned:

```text
Hl7OracleReport {
  schema,
  oracle,
  left,
  right,
  states,
  messages
}
```

`oracle` contains the pinned project/release/source identity.

`states` contains the public HL7 comparison states that are stable enough to expose, including metadata and definitions state. Unknown future state values fail closed rather than being coerced.

Each normalized message contains only deterministic public evidence:

```text
OracleMessage {
  level,
  location,
  message
}
```

Message output is sorted deterministically by `(level, location, message)` after collection and exact duplicates are removed.

The adapter MUST NOT emit HL7-generated comparison ids, UUIDs, dates from generated union/intersection resources, absolute host paths, timestamps, renderer HTML, or environment-dependent values.

## Context / snapshot boundary

The official comparer requires populated `StructureDefinition.snapshot` content and worker contexts for referenced FHIR definitions.

CF-06 must therefore use a deterministic context setup pinned to FHIR R4 core for R4 profile comparisons. Context acquisition/building belongs to the adapter/CI setup, not to CF-03.

For fixture-level tests, the adapter may operate on self-contained StructureDefinitions whose required core definitions are supplied from a pinned local package/context.

A missing required context, empty snapshot, unsupported derivation, or comparison exception is an **oracle operational failure**, not a commandF structural divergence.

## commandF oracle report

Add a Rust CF-06 model that combines:

1. the complete unmodified CF-03 `StructuralDiffReport`;
2. normalized HL7 oracle observations for comparable StructureDefinition pairs;
3. a deterministic reconciliation layer.

Suggested public model:

```text
OracleDivergenceReport {
  schema,
  oracle,
  package_name,
  before,
  after,
  structural_diff,
  resources
}

OracleResourceResult {
  resource,
  status,
  oracle_states,
  oracle_messages,
  commandf_change_kinds
}
```

Allowed resource statuses:

```text
agreement
commandf_only
authority_only
both_changed
uncomparable
oracle_error
```

These statuses are evidence relationships, **not compatibility judgments**.

`both_changed` means both systems report some comparable change signal; it does not assert field-level semantic equivalence unless a later explicit mapping proves that equivalence.

`agreement` is reserved for self-equivalent/no-change cases where both CF-03 and the HL7 oracle report no relevant change evidence.

## Matching boundary

CF-06 only invokes the HL7 StructureDefinition comparer for resource pairs that CF-03 has already matched deterministically.

- unique canonical matching follows CF-03 authority;
- canonical multiplicity behavior follows CF-03 authority;
- unmatched additions/removals remain CF-03 facts and are recorded as `uncomparable` for the two-sided HL7 comparer unless an explicit one-sided oracle contract is later added;
- non-StructureDefinition resources are `uncomparable` in CF-06 v1.

CF-06 MUST NOT invent a second package-resource matching algorithm.

## User-visible command

Add an explicit command:

```text
commandf oracle <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --oracle-adapter <path> \
  --format json
```

The command performs no package acquisition. Both package states must already be present in verified CF-01 caches.

The Rust process invokes the explicitly supplied adapter executable/jar through a bounded child-process boundary and validates its output before accepting it.

No implicit PATH lookup is permitted for the oracle adapter in v1.

## Process / security boundary

The oracle is an external process and is treated as untrusted evidence input.

CF-06 MUST:

- pass explicit temporary input paths only;
- use a bounded per-resource execution timeout;
- bound stdout/stderr capture sizes;
- reject non-zero adapter exit status;
- reject malformed/non-UTF-8/oversized JSON output;
- validate adapter report schema and exact oracle identity;
- clean temporary files;
- never execute shell command strings;
- never pass package-controlled values as shell syntax;
- never grant oracle output authority to modify repository state.

## Determinism

For the same verified CF-03 inputs and byte-identical normalized adapter outputs, the CF-06 JSON report must be byte-identical.

No clock, random id, host path, temporary path, process id, network address, or environment field may enter the commandF report.

## Fail-closed behavior

CF-06 fails operationally on:

- unsupported CF-03 schema;
- wrong/missing oracle schema;
- wrong oracle release/source identity;
- adapter spawn failure;
- adapter timeout;
- non-zero adapter exit;
- malformed/oversized adapter stdout;
- missing matched StructureDefinition input;
- corrupted before/after cache objects;
- any existing CF-03 diff failure.

Operational failure MUST NOT be silently converted into `agreement` or `uncomparable`.

## Acceptance

CF-06 is complete only when the exact final head proves:

1. CF-06 is based exactly on converged CF-03 and contains no CF-04/05 behavior.
2. Oracle provenance is pinned to official HL7 core `6.10.2` / source commit `d06577dbc5c62c74a2a8823fbc4830a3024d5b0b`.
3. The adapter uses `ComparisonSession` / `StructureDefinitionComparer` structured objects and never parses `ComparisonRenderer` HTML.
4. No reflection/private-node dependency is used.
5. Adapter self-equivalent StructureDefinitions emit no change messages and stable no-change states.
6. Synthetic cardinality/type/binding/mustSupport changes produce deterministic public HL7 messages/states where the official comparer exposes them.
7. Rust reconciliation preserves the complete CF-03 report unchanged.
8. Self-equivalent CF-03 + HL7 comparison reports `agreement`.
9. One-sided resource changes are explicit `uncomparable` rather than guessed.
10. Malformed adapter JSON fails closed.
11. Wrong oracle release/source identity fails closed.
12. Non-zero adapter exit and timeout fail closed.
13. Oracle stdout/stderr and input/output sizes are bounded.
14. Repeated reconciliation is byte-deterministic.
15. Existing CF-01 through CF-03 Rust commands/tests remain green.
16. Java adapter build/tests are locked to exact dependency version `6.10.2`.
17. A real R4 smoke compares identical `hl7.fhir.r4.core@4.0.1` StructureDefinition input through the pinned oracle and proves no false divergence.
18. Reviewer findings are dispositioned and convergence is recorded.
19. PR remains Draft and CF-07 does not start before convergence.

## Explicit deferrals

CF-06 does not add:

- CF-04 compatibility severity or producer/consumer rules;
- CF-05 SARIF/policy exit behavior;
- terminology set inclusion — CF-07;
- GitHub annotations/upload — CF-08/09;
- FSH source mapping — CF-09;
- dependency graph / blast radius — CF-11/12;
- mapping execution;
- AI/agent semantic authority.
