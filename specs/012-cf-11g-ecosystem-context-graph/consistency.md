# CF-11G Consistency Analysis — Ecosystem Context Graph

Status: PASS / planning consistency closed

## Inputs reviewed

- `AGENTS.md`;
- `.specify/memory/constitution.md`;
- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`;
- `docs/COMMAND_F_DISCOVERY_COVERAGE_2026-08-13.md`;
- canonical CF-11 multi-version package-graph `spec.md` and `plan.md`;
- current `Lockfile` schema-v1 implementation;
- current resolver implementation;
- current CF-02 `PackageInspection` / `ResourceArtifact` model;
- current CLI command surface;
- issue #19 roadmap reconciliation boundary.

## Roadmap contradiction

### Observed

The Master Architecture originally assigned:

```text
CF-11 = ecosystem context graph
CF-12 = commandf impact
```

Canonical execution later used CF-11 for the multi-version package-graph foundation correction, and that work is already merged/history-bearing.

### Resolution

Use `CF-11G` as the graph-gap restoration identity.

Properties of this decision:

- completed CF-11 history is not renamed or rewritten;
- CF-12 remains `commandf impact`;
- CF-13..CF-16 do not shift;
- the next Spec Kit sequence is `012`, but sequence number and product slice identity are explicitly separate;
- mapping IR work that requires the Context Graph depends on CF-11G rather than the historical numeric CF-11 alone.

Result: PASS.

## Vertical-slice check

### Constitution requirement

Every feature must produce a user-visible command, report, annotation, or independently executable verification result.

### Resolution

CF-11G ships `commandf context` with deterministic JSON graph output. The graph is independently inspectable before `commandf impact` exists.

Result: PASS.

## Determinism check

### Risk

Resolver traversal and graph extraction can inherit queue/hash/input ordering.

### Resolution

- exact resolved dependency edges are stored canonically;
- public graph collections use stable explicit sort keys;
- repeat-run byte equality is an acceptance criterion;
- root-order permutation is tested for both lock-v2 and graph output.

Result: PASS.

## Multi-version provenance check

### Risk

Schema-v1 locks cannot always reconstruct exact parent→child edges when multiple concrete versions satisfy similar constraints.

### Resolution

- new resolver runs write schema v2 with exact resolved edge evidence;
- existing commands keep valid v1 compatibility;
- `commandf context` fails closed on v1 instead of guessing.

Result: PASS.

## Backward-compatibility check

### Risk

A lock schema bump could unnecessarily invalidate existing user workflows.

### Resolution

Schema-specific decoding retains v1 support for commands that do not require exact resolved-edge evidence. Only the new Context Graph command requires v2.

Result: PASS.

## Canonical-resolution authority check

### Risk

An unversioned canonical URL can match multiple in-closure versions. Choosing one would create false graph authority.

### Resolution

The graph explicitly serializes `resolved`, `external`, and `ambiguous` target states. No first/newest/root-nearest heuristic is allowed.

Result: PASS.

## Extraction-completeness check

### Risk

FHIR contains more canonical/reference-bearing fields than the first graph slice can responsibly implement. Silently ignoring them while claiming a complete Context Graph would violate evidence rules.

### Resolution

CF-11G V1 has an explicit extractor contract for StructureDefinition, ValueSet, and CodeSystem dependency fields, while all present unsupported source resource types are listed in deterministic coverage metadata. Artifact nodes may exist without a claim of complete outgoing-edge extraction.

Result: PASS.

## Snapshot/differential check

### Risk

Extracting all element references from generated StructureDefinition snapshots can duplicate inherited dependencies and present them as local declarations.

### Resolution

Use top-level `baseDefinition` plus differential element profile/targetProfile/binding references in V1. This gives author-local dependency evidence without snapshot inheritance inflation.

Result: PASS.

## Storage-architecture check

### Master Architecture constraint

Graph-plane persistence should be relational-first unless measurement justifies a graph database.

### Resolution

CF-11G does not introduce a database at all. Its canonical output is a normalized deterministic relation set in JSON. If a persisted index becomes necessary for CF-12, embedded relational storage is the first candidate. Graph databases/vector stores require later evidence.

Result: PASS.

## Dependency check

### Risk

Adding a graph crate/database before a shipped consumer would violate repository dependency discipline.

### Resolution

The plan prefers no new Rust dependency. Existing serde, archive, digest, and deterministic collection facilities are sufficient for CF-11G V1.

Result: PASS.

## Trust-boundary check

### Risk

Graph build could accidentally become a second package acquisition path or bypass archive bounds.

### Resolution

`commandf context` is cache-only/offline, verifies lock digests, and reuses the CF-02 bounded archive inspection boundary. No registry lookup participates in graph construction.

Result: PASS.

## CF-06 / CF-10 independence check

### Risk

The blocked upstream HL7-maintainer path could become an accidental prerequisite or motivate a semantic workaround.

### Resolution

CF-11G depends only on canonical package/canonical inspection foundations. It does not change CF-06 oracle identity, reinterpret oracle failures, or modify the frozen CF-10 corpus.

Result: PASS.

## CF-12 sequencing check

`commandf impact` requires a working graph. Tasks explicitly prohibit CF-12 implementation until CF-11G convergence closes.

Result: PASS.

## Open design questions

No blocking design ambiguity remains for implementation start.

Implementation may refine Rust type names and private module boundaries provided it does not change these normative contracts:

- explicit lock-v2 resolved-edge evidence;
- v1 backward compatibility for existing commands;
- v1 refusal for Context Graph build;
- deterministic graph JSON;
- explicit canonical target state;
- V1 extractor coverage;
- offline verified-cache boundary;
- no compatibility authority in CF-11G.

## Final classification

```text
SPEC_PLAN_TASKS_CONSISTENCY = PASS
ROADMAP_IDENTITY            = CF-11G
IMPLEMENTATION_ORDER        = T010 -> T011 -> T012 -> T020...
CF-12_ELIGIBLE              = NO / requires CF-11G convergence
CF-06_PIN_CHANGE            = NOT AUTHORIZED BY THIS SLICE
```
