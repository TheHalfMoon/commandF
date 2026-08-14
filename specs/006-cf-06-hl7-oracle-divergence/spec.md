# CF-06 — HL7 Comparison Oracle Divergence

Status: Approved for implementation

## Purpose

CF-06 adds an **independent, advisory HL7 comparison oracle** beside CF-03 structural facts. It measures where commandF and the official HL7 Java comparison implementation agree, diverge, cannot be compared, or fail operationally.

CF-06 does **not** replace CF-03 authority and does **not** classify compatibility severity. CF-03 remains commandF's deterministic structural-fact contract. CF-06 records external evidence about those facts.

## Stack boundary

CF-06 depends on CF-03 only.

```text
base branch: feat/cf-03-structural-diff
base SHA: aa212b108e05fa0e22312f244f393c59602192b9
```

CF-04/CF-05 rules, severities, policy decisions, SARIF, and CI exit semantics are out of scope.

## Pinned oracle identity

Official project:

```text
https://github.com/hapifhir/org.hl7.fhir.core
```

Pinned release/source:

```text
release: 6.10.2
source commit: d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
```

Official release artifact provenance:

```text
validator_cli.jar
sha256: a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6
```

The adapter uses the published Java libraries at exact version `6.10.2`; commandF MUST NOT commit the large validator jar. The jar digest is provenance evidence, not a requirement to execute the fat jar when the library API is sufficient.

Changing oracle release/source is a later explicit contract change.

## Official structured comparison surface

CF-06 consumes the official structured comparison model **before rendering**:

- `ComparisonSession.compare(...)` dispatches StructureDefinition pairs to `StructureDefinitionComparer`;
- the comparison returns `StructureDefinitionComparer.ProfileComparison`;
- canonical comparison states are exposed through public state accessors;
- metadata is exposed as public `StructuralMatch<String>` values;
- `StructuralMatch` exposes public children and `ValidationMessage` lists;
- `ValidationMessage` carries severity, location/path, and message text;
- `ComparisonRenderer` is presentation only and MUST NOT be parsed as oracle evidence.

CF-06 MUST use public API surfaces only. Reflection into private comparer node types is forbidden.

## Deterministic local context boundary

The official comparer requires populated snapshots and worker contexts for referenced FHIR definitions. CF-06 therefore makes all context packages explicit and local.

For R4 v1, the core context is exactly:

```text
hl7.fhir.r4.core@4.0.1
```

The Java adapter invocation receives:

```text
java -jar commandf-hl7-oracle.jar \
  --core-package <verified-hl7.fhir.r4.core-4.0.1.tgz> \
  --left-package <verified-before-package.tgz> \
  --right-package <verified-after-package.tgz> \
  --left-url <canonical-url> [--left-version <canonical-version>] \
  --right-url <canonical-url> [--right-version <canonical-version>]
```

The adapter loads NPM `.tgz` files locally with the official HL7 package/context APIs. It MUST NOT rely on an implicit user-level HL7 package cache or package download during comparison.

Missing context, wrong core identity/version, empty snapshots, unsupported derivation, unresolved matched canonical resource, or comparison exception is an **oracle operational failure**.

## Java adapter output

The adapter emits exactly one UTF-8 JSON document plus trailing newline on stdout:

```text
Hl7OracleReport {
  schema,
  oracle,
  left,
  right,
  states,
  messages
}

OracleResourceIdentity {
  url,
  version,
  id,
  type
}

OracleStates {
  metadata,
  definitions,
  content,
  content_interpretation
}

OracleMessage {
  level,
  location,
  message
}
```

`oracle` carries exact project/release/source provenance.

Allowed normalized states:

```text
unknown
not_changed
changed
cannot_evaluate
```

Allowed message levels:

```text
fatal
error
warning
information
```

Messages are sorted deterministically by `(level, location, message)` and exact duplicates removed. Rust repeats canonical sorting/de-duplication before embedding oracle evidence.

The adapter MUST NOT emit HL7-generated comparison ids, session UUIDs, generated union/intersection dates, absolute host paths, timestamps, renderer HTML, or environment-dependent values.

## Matching authority

CF-06 MUST reuse CF-03 deterministic resource matching. It MUST NOT invent a second package-resource matcher.

The HL7 two-sided comparer is invoked only for **matched canonical StructureDefinition pairs**.

- unique canonical matching follows CF-03;
- canonical multiplicity/version identity follows CF-03;
- unmatched add/remove resources are `uncomparable`;
- matched non-canonical StructureDefinitions are `uncomparable` in v1;
- non-StructureDefinition resources are `uncomparable` in v1.

## commandF oracle report

The Rust report contains:

1. the complete unmodified CF-03 `StructuralDiffReport`;
2. normalized HL7 observations for comparable matched canonical StructureDefinitions;
3. deterministic evidence-relationship statuses.

```text
OracleDivergenceReport {
  schema,
  oracle,
  package_name,
  structural_diff,
  resources
}

OracleResourceResult {
  resource,
  status,
  oracle,
  commandf_change_kinds
}
```

Allowed statuses:

```text
agreement
commandf_only
authority_only
both_changed
uncomparable
```

These are evidence relationships, **not compatibility judgments**.

- `agreement`: neither CF-03 nor HL7 reports a relevant change signal;
- `commandf_only`: CF-03 reports structural facts and HL7 reports none;
- `authority_only`: HL7 reports a change signal and CF-03 reports none;
- `both_changed`: both report some change signal; field-level equivalence is NOT implied;
- `uncomparable`: resource is outside the v1 two-sided canonical StructureDefinition comparison boundary.

Oracle operational failure fails the command; it is not silently turned into `uncomparable` or `agreement`.

## User-visible command

```text
commandf oracle <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --oracle-core-lock <path> --oracle-core-cache <path> \
  --oracle-java <absolute-or-explicit-path> \
  --oracle-adapter <commandf-hl7-oracle.jar> \
  --format json
```

The command performs no package acquisition. Before/after package states and the exact R4 core package must already exist in verified CF-01 lock/cache state.

No implicit PATH lookup is permitted for Java or the adapter in v1.

## Process / security boundary

The external oracle process is untrusted evidence input.

CF-06 MUST:

- use explicit Java executable and adapter jar paths;
- invoke `java` directly, never through `sh`, `cmd`, or PowerShell;
- pass package-controlled data only as argv values;
- use private temporary state only where necessary;
- enforce a bounded per-resource timeout;
- drain stdout/stderr concurrently while retaining bounded bytes;
- bound accepted stdout to 8 MiB and retained stderr to 1 MiB per invocation;
- reject non-zero adapter exit;
- reject malformed/non-UTF-8/oversized JSON;
- validate schema and exact oracle identity;
- bound message count to 100,000 and each normalized string to 64 KiB;
- ensure external failure cannot become false agreement;
- never grant oracle output authority to mutate repository or package state.

## Determinism

For the same verified CF-03 inputs and equivalent normalized oracle evidence, CF-06 JSON must be byte-identical.

No clock, random id, host/temp path, process id, network address, or environment field may enter the public report.

## Fail-closed behavior

CF-06 fails operationally on:

- unsupported CF-03 schema;
- wrong/missing oracle schema;
- wrong oracle project/release/source identity;
- invalid/missing explicit core package identity;
- adapter or Java executable spawn failure;
- timeout;
- non-zero adapter exit;
- malformed/non-UTF-8/oversized adapter output;
- oracle observation identity inconsistent with the CF-03 canonical match;
- corrupted before/after/core cache objects;
- any existing CF-03 diff failure.

## Acceptance

CF-06 is complete only when the exact final head proves:

1. CF-06 is based exactly on converged CF-03 with no CF-04/05 behavior.
2. Oracle provenance is exact `6.10.2` / `d06577dbc5c62c74a2a8823fbc4830a3024d5b0b` and the recorded release-jar digest matches official release metadata.
3. Adapter uses structured `ComparisonSession` / `StructureDefinitionComparer` objects; no renderer HTML.
4. No reflection/private-node dependency.
5. Core context is explicit local `hl7.fhir.r4.core@4.0.1`; no hidden package acquisition during `oracle`.
6. Adapter self-equivalent StructureDefinitions produce stable no-change evidence.
7. Synthetic cardinality/type/binding/mustSupport changes expose deterministic public HL7 evidence where supported.
8. Rust parsing and Java output schemas are byte/field compatible.
9. Rust reconciliation preserves the complete CF-03 report unchanged.
10. Self-equivalent CF-03 + HL7 evidence reports `agreement`.
11. One-sided/non-supported resources are explicit `uncomparable`.
12. Malformed JSON and wrong oracle provenance fail closed.
13. Non-zero adapter exit and timeout fail closed.
14. stdout/stderr/evidence sizes are bounded.
15. Repeated reconciliation is byte-deterministic.
16. Existing CF-01 through CF-03 Rust gates remain green.
17. Java adapter builds/tests at exact dependency version `6.10.2` on Java 17.
18. Real R4 self-equivalence smoke proves no false divergence.
19. Review findings are dispositioned and convergence recorded.
20. PR remains Draft and CF-07 does not start before convergence.

## Explicit deferrals

CF-06 does not add CF-04 compatibility severity, CF-05 SARIF/policy behavior, CF-07 terminology set inclusion, GitHub annotations/upload, FSH source mapping, ecosystem graph/blast radius, mapping execution, or AI/agent semantic authority.
