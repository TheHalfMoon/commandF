# CF-11G Specification — Ecosystem Context Graph

Status: planning candidate

## Identity and roadmap role

`CF-11G` is the gap-restoration slice for the ecosystem Context Graph originally planned as CF-11 in `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`.

Canonical repository history already used `CF-11` for the multi-version package-graph foundation correction merged in PR #13. That history remains unchanged. `CF-11G` does not renumber or reinterpret completed CF-11 work, and it does not shift downstream product identities: `CF-12` remains `commandf impact` and depends on a working CF-11G Context Graph.

Spec Kit sequence directory `012-*` is only the next available planning-package sequence; the product slice identity is `CF-11G`.

## Problem

commandF can resolve exact package identities, inspect canonical FHIR artifacts, and diff/check individual packages, but it cannot yet answer deterministic cross-artifact dependency questions.

Two foundation facts make a Context Graph require explicit evidence rather than reconstruction by guesswork:

1. CF-11 permits multiple concrete versions of the same package name in one closure.
2. `commandf.lock` schema v1 stores each package manifest's declared dependency constraints but does not store the exact parent-to-child resolved dependency edges selected during traversal.

For a patch wildcard or other branch-local request, a v1 lock containing multiple matching concrete package versions is therefore insufficient to reconstruct which exact child identity was selected for a specific parent. A graph built by choosing the first or highest matching locked version would violate commandF's provenance and fail-closed rules.

Likewise, canonical references such as an unversioned `StructureDefinition.baseDefinition` may resolve to zero, one, or multiple in-closure canonical artifacts. The graph must retain that state explicitly rather than silently selecting a target.

## User-visible outcome

A new command:

```text
commandf context --lock commandf.lock --cache .commandf/cache --format json
```

builds a deterministic Context Graph report from one verified package closure.

The report is independently useful before CF-12. It provides exact package nodes, canonical artifact nodes, resolved package-dependency edges, supported canonical-reference edges, explicit unresolved/ambiguous target state, extraction coverage, and source provenance.

Identical pinned lock/cache inputs MUST produce byte-identical JSON output.

## Normative behavior

### 1. Exact package identities

Every package node is identified by:

```text
(name, concrete version, archive sha256)
```

The graph MUST preserve the package source/provenance already recorded by the lockfile.

### 2. Explicit resolved dependency evidence

New package resolutions MUST record the exact parent-to-child package identity selected for every dependency request, together with the declared dependency constraint that caused that edge.

The resolver MUST record an edge even when the selected child package was already present and therefore does not need to be downloaded or expanded again.

Cycles MUST terminate by exact-identity expansion deduplication while retaining the cycle edge itself.

### 3. Lockfile migration boundary

The explicit resolved-edge evidence requires a lock schema revision.

- New `pkg resolve` output MUST use lock schema v2.
- Schema v2 MUST retain all v1 package identity, digest, source, root, and declared dependency information.
- Existing commands that do not require explicit graph edges MUST continue to read valid schema-v1 locks.
- `pkg verify` MUST continue to verify all package digests for valid v1 and v2 locks.
- `commandf context` MUST fail closed on schema v1 because exact package dependency edges cannot always be reconstructed from it.
- No command may silently synthesize v2 edge evidence from an ambiguous v1 lock.

### 4. Canonical artifact nodes

For every verified locked package archive, CF-11G MUST reuse the CF-02 artifact inspection boundary to identify FHIR resources and retain at least:

- owning package exact identity;
- archive digest;
- resource filename;
- resource type;
- resource id when present;
- canonical URL when present;
- canonical version when present;
- resource content SHA-256.

Multiple versions of the same canonical URL are distinct artifact nodes.

### 5. Canonical reference edges — V1 coverage

CF-11G V1 MUST deterministically extract the following dependency relations from supported FHIR conformance resources:

#### StructureDefinition (including profiles and extensions)

- `baseDefinition`;
- differential `element[].type[].profile[]`;
- differential `element[].type[].targetProfile[]`;
- differential `element[].binding.valueSet`.

Differential elements are used for element-level profile/binding references to avoid silently treating every inherited snapshot constraint as a newly declared local dependency. The top-level `baseDefinition` remains an explicit edge.

#### ValueSet

- `compose.include[].system`;
- `compose.include[].valueSet[]`;
- `compose.exclude[].system`;
- `compose.exclude[].valueSet[]`.

#### CodeSystem

- `supplements` when present.

Resources outside this extractor set MAY still appear as artifact nodes, but the report MUST expose their resource types in deterministic extraction-coverage metadata. Unsupported resource types MUST NOT be presented as fully analyzed canonical-reference sources.

### 6. Canonical reference target resolution

Canonical reference strings MUST be preserved exactly as source evidence. commandF MUST NOT perform network lookup or mutable-registry resolution during `context`.

For graph matching only:

- `url|version` targets an exact in-closure canonical URL + canonical version when exactly one exists;
- unversioned `url` resolves when exactly one in-closure artifact has that canonical URL;
- zero matches are retained as `external` / unresolved-in-closure edges;
- multiple eligible matches are retained as `ambiguous` edges with deterministic candidate identities;
- commandF MUST NOT silently pick the first, newest, root-nearest, or otherwise preferred candidate.

Fragments and source strings MUST be retained; any supported parsing rule must be explicit and tested.

### 7. Deterministic relational graph representation

The canonical JSON report MUST use normalized sorted collections rather than relying on traversal or hash-map order. At minimum it contains deterministic relations equivalent to:

```text
packages
artifacts
package_dependency_edges
canonical_reference_edges
coverage
```

CF-11G does not introduce a graph database. If CF-12 demonstrates a need for indexed persistent storage, the Master Architecture's relational-first rule applies: embedded relational storage is the first candidate, and a graph database requires measurement-based justification.

### 8. Cache and trust boundary

`commandf context` MUST be offline with respect to package acquisition.

Before reading an archive, it MUST verify the cached bytes against the lockfile SHA-256 using the existing package-cache trust boundary. Missing or corrupted required archive bytes MUST fail the command.

No PHI or instance data is required or accepted by this slice.

### 9. Bounded input behavior

Existing archive size, entry-count, path, decompression, and JSON safety bounds MUST remain authoritative. CF-11G MUST NOT add an unbounded second archive reader or bypass existing package inspection protections.

### 10. Downstream authority boundary

CF-11G records deterministic dependency evidence. It does not itself classify a change as breaking, risky, additive, clinically safe, or semantically equivalent.

`CF-12 commandf impact` may consume the graph later, but must preserve explicit `external` and `ambiguous` states rather than converting them to compatibility claims.

## Acceptance criteria

### A. Multi-version exact edge proof

A synthetic closure where two parents request different concrete versions of the same dependency name MUST contain two exact package nodes and the correct parent-to-child resolved edge for each request.

### B. Shared identity edge proof

Two parents selecting the same concrete child identity MUST produce two dependency edges to one deduplicated child node.

### C. Cycle proof

A cycle MUST retain the closing dependency edge and terminate without repeated archive expansion.

### D. V1 fail-closed proof

Existing valid schema-v1 locks remain readable by existing commands, while `commandf context` rejects them with a stable diagnostic explaining that resolved-edge evidence is unavailable.

### E. Canonical ownership proof

A package containing multiple canonical resources MUST produce stable artifact nodes tied to the exact owning package identity and archive digest.

### F. StructureDefinition edge proof

Fixtures MUST cover `baseDefinition`, profile, targetProfile, and binding ValueSet edges, including an Extension StructureDefinition.

### G. Terminology graph proof

Fixtures MUST cover ValueSet system/valueSet references and CodeSystem supplements.

### H. Target-state proof

Fixtures MUST prove all three target states: exactly resolved, external/unresolved-in-closure, and ambiguous due to multiple in-closure canonical versions.

### I. Coverage proof

At least one unsupported resource type MUST remain an artifact node while appearing in explicit coverage metadata rather than being silently treated as fully analyzed.

### J. Determinism

Equivalent resolver root-order permutations and repeated `context` builds over identical bytes MUST produce byte-identical lock v2 and Context Graph JSON respectively.

### K. Regression

All existing workspace tests, security regressions, CF-06 oracle workflow, CF-11 multi-version proof, and real FHIR smoke gates remain green.

## Explicit non-goals

CF-11G does not:

- implement `commandf impact` or blast-radius policy;
- change CF-03/04/05 compatibility classification;
- change CF-06 HL7 oracle identity or exception semantics;
- modify the frozen CF-10 corpus;
- crawl the public FHIR registry or internet to complete missing canonical references;
- introduce RDF/SPARQL, Oxigraph, Neo4j, Qdrant, embeddings, vector search, or AI authority;
- parse SQL-on-FHIR, CQL, SearchParameter expressions, or FHIRPath invariants yet;
- claim exhaustive FHIR canonical-reference extraction outside the explicit V1 extractor set;
- infer package dependency edges from schema-v1 constraints when exact resolved identity is not recorded.

## Evidence and provenance

Every graph report must be reconstructable from:

- exact lock bytes/schema;
- exact package name/version identities;
- package archive SHA-256 values;
- recorded package source provenance;
- verified cached archive bytes;
- graph schema/extractor version.

Mutable package aliases or registry state are not graph evidence.
