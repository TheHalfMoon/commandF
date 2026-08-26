# CF-11G Plan — Ecosystem Context Graph

Status: planning candidate

## Base and sequencing

```text
repository: TheHalfMoon/commandF
planning base: 5bafce4f63537e0507e9b0708e1ebd8e22e3c463
planning branch: docs/cf11g-context-graph-planning
historical CF-11: multi-version package graph / PR #13
new product slice: CF-11G ecosystem context graph
next downstream slice: CF-12 commandf impact
```

CF-11G is independent of the blocked CF-10/HL7 upstream-maintainer path. It consumes canonical package/canonical inspection foundations already present in commandF and does not require a production oracle-pin decision.

## Architectural decision

### Keep historical numbering stable

Do not rename completed CF-11 commits, specs, PRs, or evidence. Do not shift CF-12..CF-16 identifiers.

Insert `CF-11G` as the graph-gap restoration slice. Spec Kit sequence `012` is a documentation/package sequence only and is not a product renumbering.

### Ship a graph report, not a database scaffold

The first Context Graph vertical slice is a deterministic normalized JSON relation set exposed by `commandf context`.

No graph database is introduced. The report itself is the canonical evidence artifact. If CF-12 later needs indexed persistence, evaluate embedded relational storage first. This follows the Master Architecture's relational-first constraint without creating a storage dependency before a shipped query requires one.

## Current-state constraints that drive the design

### Lock schema v1 cannot prove exact multi-version dependency edges

Current `Lockfile` schema v1 stores:

- roots;
- exact locked packages;
- package digest/source;
- each manifest's declared dependency constraints.

The current resolver selects exact `(name, version)` identities but the v1 lock does not retain which exact child identity was selected for each parent request. With multiple versions of one package name in the same closure, reconstructing that edge after the fact can be ambiguous.

CF-11G therefore earns the explicit resolved-edge schema that CF-11 intentionally deferred until a shipped consumer required it.

### CF-02 already owns artifact inspection

Reuse the existing bounded package archive reader and `PackageInspection`/`ResourceArtifact` concepts. Do not build a second unbounded tar/JSON ingestion path.

## Implementation shape

### 1. Lock schema v2

Add a deterministic top-level relation such as:

```text
resolved_dependencies: [
  {
    from_name,
    from_version,
    to_name,
    to_version,
    declared_constraint
  }
]
```

Exact field names may be refined during implementation, but the semantic contract is fixed:

- parent and child exact package identities are explicit;
- declared dependency constraint is retained;
- collection ordering is canonical;
- duplicate identical edge records are deduplicated deterministically;
- source/digest identity remains on `LockedPackage` and is not copied inconsistently into each edge.

#### Backward compatibility

Implement a version-aware lock decoder.

- v1 remains accepted by existing commands;
- v2 is emitted by new `pkg resolve` runs;
- `pkg verify` accepts both;
- graph construction requires v2 and produces a stable fail-closed error on v1.

Avoid `#[serde(default)]` behavior that would make a v1 lock indistinguishable from a malformed v2 lock. Schema-specific validation must be explicit.

### 2. Resolver edge capture

Change the resolver queue from a bare `PackageRequest` to an internal pending request carrying optional exact parent identity plus the declared constraint that created the request.

Processing order:

1. deterministically select concrete version for the request;
2. form exact child identity;
3. if a parent exists, record the exact parent→child edge immediately;
4. if child identity was already expanded, stop there for this queue item;
5. otherwise acquire/verify/cache child package, record package node, and enqueue its dependencies with this child as parent.

This preserves shared-child and cycle edges without repeated expansion.

### 3. Context Graph model

Add a library-owned model in `commandf-pkg`, not CLI-only ad hoc JSON.

Recommended model families:

```text
ContextGraphReport
ContextPackageNode
ContextArtifactNode
PackageDependencyEdge
CanonicalReferenceEdge
CanonicalTargetResolution
ContextCoverage
```

All public collections sort by stable explicit keys before serialization.

### 4. Artifact extraction

For each locked package:

1. verify cache digest;
2. load the archive through existing bounded inspection helpers;
3. create artifact nodes from CF-02 metadata;
4. inspect the same bounded resource JSON for only the V1 reference extractors defined in `spec.md`;
5. retain exact source canonical strings and deterministic relation labels.

Do not normalize or dereference canonical URLs through the network.

### 5. Canonical target matching

Build a deterministic in-closure canonical index keyed by canonical URL and optional canonical version.

For each extracted reference:

- exact `url|version` + one candidate → `resolved`;
- unversioned URL + one candidate → `resolved`;
- no candidate → `external`;
- more than one candidate → `ambiguous`, with candidates sorted by exact package/artifact identity.

Do not use package order, root distance, publication recency, or semantic guessing to choose among ambiguous candidates.

### 6. Coverage metadata

The report must distinguish:

- supported source resource types with active extractors;
- resource types present in the closure but not covered by a canonical-reference extractor;
- extractor schema/version.

This prevents a partial V1 graph from being mistaken for exhaustive FHIR dependency coverage.

### 7. CLI

Add:

```text
commandf context \
  --lock commandf.lock \
  --cache .commandf/cache \
  --format json
```

Behavior:

- reads only local lock/cache inputs;
- validates lock schema and package cache before graph extraction;
- writes canonical JSON to stdout;
- exits nonzero on invalid lock, schema-v1 graph request, missing/corrupt cache content, malformed required package content, or graph invariant violation;
- never performs package acquisition.

No output file flag is required in this slice; shell redirection is sufficient and keeps the CLI surface small.

## Security and trust boundaries

### Package/archive boundary

Reuse CF-01/CF-02 archive controls. The graph implementation must not call a raw tar reader with looser limits.

### Provenance boundary

A graph edge is evidence only when its origin is reconstructable from pinned lock/archive bytes. No mutable registry lookup participates in graph build.

### Canonical resolution boundary

`resolved` means "uniquely matched inside this exact closure", not "globally authoritative canonical owner".

`external` means not found inside the closure, not invalid.

`ambiguous` is retained as evidence and must remain fail-closed for downstream claims.

### Data boundary

This slice processes public/synthetic FHIR conformance metadata only. No patient instances or PHI.

## Dependency policy

Prefer no new Rust dependency for CF-11G.

Existing serde/JSON, archive, digest, semver, and deterministic collection facilities are sufficient for the first graph report. Do not add SQLite, petgraph, Oxigraph, Qdrant, RDF, or vector-search dependencies in this slice.

A future persisted relational index may add one embedded relational dependency only when a shipped CF-12 query requires it and the relevant PR exercises it immediately.

## Test strategy

### Resolver/lock unit and integration tests

- v2 exact edge recording;
- multiple same-name versions with correct branch-local edges;
- shared child edge retention;
- cycle closing edge retention + bounded expansion;
- root-order byte identity;
- v1 decode compatibility;
- malformed v2 missing/inconsistent edge fields fail closed;
- `pkg verify` v1/v2 regression.

### Graph model tests

- deterministic node/edge ordering;
- duplicate edge deduplication;
- exact artifact provenance;
- stable JSON bytes.

### Canonical extraction fixtures

Create small synthetic/publicly redistributable package archives covering:

- StructureDefinition profile + extension;
- `baseDefinition`;
- differential type `profile` and `targetProfile`;
- differential binding `valueSet`;
- ValueSet include/exclude systems and imported ValueSets;
- CodeSystem `supplements`;
- unsupported resource type coverage reporting.

### Canonical resolution fixtures

- exact versioned target;
- unique unversioned target;
- external target;
- ambiguous unversioned target where two canonical versions are present.

### CLI tests

- v2 success;
- v1 refusal;
- corrupted cache refusal;
- missing cache refusal;
- repeat-run byte equality;
- no network/source invocation during context build.

### Regression gates

At every implementation PR head:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Preserve existing CF-08/CF-09 security tests, real FHIR smoke, `cf06-oracle`, and `cf11-multi-version-proof` workflows.

## PR stack

Keep implementation stack independently reviewable:

### PR A — resolved-edge evidence / lock v2

- version-aware lock decoding;
- v2 writer;
- resolver edge capture;
- backward compatibility tests.

No Context Graph CLI yet, but this PR is immediately executable through `pkg resolve`/`pkg verify` and is required evidence plumbing for the shipped graph consumer stacked above it.

### PR B — Context Graph library

- deterministic graph model;
- bounded canonical extraction;
- target resolution + coverage metadata;
- fixture tests.

### PR C — `commandf context`

- CLI surface;
- end-to-end graph build;
- deterministic output proof;
- negative cache/schema paths;
- workflow/regression qualification.

Do not merge PR A alone to main unless PR B/C exact candidate stack is ready or the maintainer explicitly decides the lock-v2 migration is independently desirable. This prevents an unused schema migration from becoming canonical by accident.

## Review focus

1. any inferred package edge not directly recorded during resolver selection;
2. v1/v2 decoder ambiguity;
3. silent first-match canonical resolution;
4. snapshot-inheritance edge inflation presented as local declaration;
5. unsupported resource types hidden from coverage metadata;
6. traversal/hash-order nondeterminism;
7. archive/cache trust-boundary bypass;
8. accidental network access during graph build;
9. graph state presented as compatibility authority;
10. downstream API choices that make CF-12 require semantic guessing.

## Migration impact

New `pkg resolve` lock bytes change from schema v1 to v2. This is intentional and must be explicitly called out in release notes/convergence evidence.

Existing valid v1 locks remain supported for existing commands, so users are not forced to regenerate locks unless they want `commandf context`.

## Exit criteria

CF-11G can converge only when:

- roadmap reconciliation is canonical;
- spec/plan/tasks consistency is closed;
- all implementation tasks are complete;
- exact-head mandatory CI is green;
- lock v2 deterministic migration evidence is retained;
- Context Graph repeat runs are byte-identical;
- all ambiguity/external/unsupported states are explicit;
- CodeRabbit and Qodo findings are dispositioned when available;
- convergence identifies no untracked blocker for CF-12.

Only then may CF-12 `commandf impact` begin implementation.
