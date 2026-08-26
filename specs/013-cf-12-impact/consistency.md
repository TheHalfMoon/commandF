# CF-12 Consistency Analysis — Deterministic Impact Analysis

Status: planning consistency candidate; independent review still required before T004 closes.

## Inputs checked

This analysis reconciles:

- `AGENTS.md`;
- `.specify/memory/constitution.md`;
- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`;
- canonical CF-11G `spec.md` and convergence evidence;
- CF-12 `spec.md`;
- CF-12 `plan.md`;
- CF-12 `tasks.md`;
- the current CLI conventions for `diff`, `classify`, `check`, and `context`.

## Result

No blocking internal contradiction is known in the planning candidate.

The implementation MUST NOT begin until T004 is closed through exact-head planning review/CI and this package becomes canonical.

## Consistency checks

### 1. Roadmap dependency

Master Architecture requires:

```text
CF-12 = commandf impact
CF-12 depends on CF-11G
```

CF-11G is now canonically closed. The planning package therefore satisfies the entry dependency without making CF-06/CF-10 upstream governance a new prerequisite.

Result: CONSISTENT.

### 2. Vertical-slice rule

The constitution prohibits a scaffold-only slice. The plan ends in a shipped user-visible `commandf impact` CLI plus deterministic proof rather than a library-only endpoint.

Result: CONSISTENT.

### 3. Existing CLI shape

Current `diff`, `classify`, and `check` accept a selected package plus explicit before/after lock/cache paths. CF-12 adopts the same shape instead of inventing repository/branch/network inputs.

Result: CONSISTENT.

### 4. Determinism

The specification requires byte-identical JSON for identical pinned inputs. The plan defines canonical sorting, shortest-path normalization, lexicographic tie-breaking, exact-identity visited state, and a dedicated repeat-run proof workflow.

Result: CONSISTENT.

### 5. Fail-closed ambiguity

CF-11G freezes canonical target states as `resolved`, `external`, and `ambiguous`. CF-12 traverses only `resolved` edges and retains the other two states as explicit unresolved boundaries.

No task adds a preferred-candidate heuristic or network lookup.

Result: CONSISTENT.

### 6. Compatibility authority

AGENTS/constitution require precision and prohibit invented compatibility states. CF-12 defines impact as reachability/exposure evidence and explicitly separates it from CF-04/CF-05 BREAKING/RISKY/ADDITIVE authority.

An impacted node without an existing compatibility finding remains impact evidence only.

Result: CONSISTENT.

### 7. Multi-version package identity

Canonical CF-11 and CF-11G require exact multi-version identities. CF-12 package traversal consumes schema-v2 exact package edges and forbids name-only collapse.

Result: CONSISTENT.

### 8. Added/removed evidence

A single after-only graph would erase dependents of removed artifacts; a single before-only graph would erase newly introduced dependency evidence. The specification therefore requires side-aware before and after graph analysis, with `both` normalization only for exact identical evidence.

Result: CONSISTENT.

### 9. Path semantics

Transitive blast radius can have multiple valid paths. Returning all paths would increase output/noise and complicate determinism; returning arbitrary first traversal would violate determinism. V1 therefore returns one canonical shortest path per exact `(impacted, seed, side)` relation, with lexicographic stable-identity tie-breaking.

This is a reporting normalization and does not discard unresolved-boundary entries.

Result: CONSISTENT.

### 10. Coverage truth

CF-11G V1 does not extract every possible FHIR relation. CF-12 carries graph extraction coverage forward and makes no exhaustive artifact-impact claim outside supported relation kinds. Package-level exposure remains separately representable.

Result: CONSISTENT.

### 11. Archive/cache trust boundary

The plan reuses existing lock parsing, verified cache reads, bounded package inspection, and Context Graph construction. It does not introduce a second archive reader or mutable registry dependency.

Result: CONSISTENT.

### 12. Oracle boundary

CF-12 neither changes CF-06 oracle identity nor requires resolution of the separate CF-10 production-oracle governance blocker. Existing oracle workflow regression remains a repository gate where applicable.

Result: CONSISTENT.

### 13. Dependency additions

No new Rust crate is planned. If implementation later proves a dependency necessary, the relevant task and plan must be amended before adding it, with an immediate shipped/tested consumer.

Result: CONSISTENT.

## Risks requiring reviewer attention

Independent review should challenge these specific planning choices:

1. whether the existing structural diff provides a sufficiently stable canonical artifact seed identity for added/removed/modified resources;
2. whether package-level exposure should include only dependency reachability to the selected changed package or require additional artifact evidence before presentation;
3. whether canonical shortest-path reporting preserves enough evidence for actionable review UX;
4. whether an `external`/`ambiguous` edge from an impacted artifact is the correct V1 boundary for unresolved-impact reporting;
5. whether side normalization can accidentally erase changed edge provenance;
6. whether any report field would be interpreted as compatibility severity despite the explicit authority separation.

A substantive reviewer finding on any of these points reopens the corresponding planning contract and prevents T004 closure until dispositioned.

## Explicit deferrals

The following remain outside CF-12 V1 and are not planning gaps:

- SQL-on-FHIR ViewDefinition impact;
- CQL impact;
- SearchParameter-expression impact;
- FHIRPath invariant impact;
- persistent relational/graph storage;
- graph database adoption;
- network completion of external canonical references;
- model/AI-generated impact claims;
- clinical/runtime breakage claims;
- changes to CF-06 production oracle identity;
- changes to the frozen CF-10 corpus.

These require later separately specified slices or measured evidence.

## Planning close rule

T004 may be marked complete only when:

- exact-head planning CI is green for applicable repository gates;
- independent review is inspected;
- every substantive returned planning finding is fixed or explicitly rejected against canonical governance with evidence;
- the planning PR merges without unresolved contradiction.

Only then may Stack A implementation begin.
