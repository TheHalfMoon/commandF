# CF-12 Specification — Deterministic Impact Analysis

Status: planning candidate

## Identity and roadmap role

`CF-12` ships `commandf impact` and depends on canonical CF-11G Context Graph evidence.

Canonical CF-11G closed through PR #24 on main commit `8f2ce65de3565a81968bb127c96b451f617593c4`. CF-12 may therefore be planned, but implementation is not complete merely because this specification exists.

## Problem

commandF can already resolve packages, inspect canonical artifacts, compute structural/terminology differences, classify compatibility policy, and build an offline deterministic Context Graph. It still cannot answer the product question:

> Given a change in one package, which in-closure packages and canonical artifacts are exposed to that change, and through what evidence path?

A useful answer must not collapse dependency reachability into a compatibility claim. A dependent artifact can be exposed to a change without being proven broken. Conversely, an unresolved or ambiguous canonical reference must not be silently guessed into a dependency path.

## User-visible outcome

A new command:

```text
commandf impact <package> \
  --before-lock before.lock \
  --before-cache before-cache \
  --after-lock after.lock \
  --after-cache after-cache \
  --format json
```

produces a deterministic impact report for the selected package change across the verified before/after closures.

The report identifies changed canonical artifacts, package-level exposure, artifact-level reverse dependency paths, and unresolved graph boundaries. Identical pinned inputs MUST produce byte-identical JSON output.

## Normative behavior

### 1. Input identity and trust boundary

CF-12 MUST reuse the existing explicit before/after lock and cache inputs used by `diff`, `classify`, and `check`.

Both sides MUST:

- use supported lock schema v2 for Context Graph construction;
- verify required cached package bytes through the existing package-cache digest boundary;
- remain offline with respect to package acquisition and canonical resolution;
- retain exact package name, concrete version, source provenance, archive digest, artifact digest, and graph schema/extractor identity.

Missing, corrupt, unsupported, or malformed required evidence MUST fail closed.

### 2. Change seeds

Impact traversal MUST begin from deterministic change evidence for the selected package, not from filename guesses or mutable registry state.

A seed represents a canonical artifact in the selected package whose before/after evidence shows a material artifact change relevant to existing commandF diff evidence, including at minimum:

- added canonical artifact;
- removed canonical artifact;
- modified canonical artifact with a non-empty structural delta.

The seed MUST retain the available before and after artifact identities and resource digests.

CF-12 MUST NOT invent a BREAKING/RISKY/ADDITIVE classification. Existing CF-03/CF-04/CF-05 evidence may be attached or linked when available, but impact reachability is a separate relation.

### 3. Side-aware graph analysis

CF-12 MUST build or consume deterministic CF-11G Context Graph evidence for both before and after closures.

Impact paths are side-aware:

- references to an artifact that existed only before may be proven from the before graph;
- references to an artifact that exists after may be proven from the after graph;
- the report MUST retain which side proves each edge/path;
- equivalent evidence from both sides MAY be represented as `both` only when the exact normalized relation is identical.

The implementation MUST NOT erase evidence merely because a node or edge exists on only one side.

### 4. Artifact blast radius

For each change seed, CF-12 MUST traverse canonical-reference relations in reverse: if artifact A has a resolved reference to artifact B, a change seed at B can expose A.

Traversal MUST:

- use only exact `resolved` CF-11G reference targets;
- retain deterministic shortest evidence paths from each impacted artifact to each seed;
- support transitive reverse traversal until no new exact artifact identity is discovered;
- deduplicate exact nodes/paths deterministically;
- terminate on cycles through exact-identity visited-state handling;
- never traverse through `external` or `ambiguous` target states as if resolved.

An impacted artifact is evidence of dependency exposure, not proof that behavior breaks.

### 5. Package blast radius

CF-12 MUST also report package exposure derived from exact schema-v2 package dependency edges.

A package is package-exposed when its exact dependency graph reaches the changed package identity on the relevant side. Package paths MUST preserve exact package identities and declared dependency constraints.

Package exposure and artifact exposure are distinct:

- package exposure proves dependency-closure reachability;
- artifact exposure proves a supported canonical-reference path;
- neither alone is a compatibility verdict.

### 6. Unresolved boundaries

Any relevant CF-11G canonical edge with state `external` or `ambiguous` MUST remain explicit evidence.

The report MUST include deterministic unresolved-boundary entries when such an edge originates from an otherwise impacted artifact or blocks a possible path that commandF cannot resolve from the pinned closure.

For ambiguous references, all deterministic candidate identities already retained by CF-11G MUST remain visible. CF-12 MUST NOT select a preferred candidate.

For external references, CF-12 MUST NOT perform network lookup to complete the path.

### 7. Deterministic path semantics

When multiple paths reach the same `(impacted identity, seed identity, side)` relation, V1 MUST retain one canonical shortest path.

Tie-breaking between equal-length paths MUST use lexicographic ordering over stable exact node/edge identities. Traversal order or hash-map iteration MUST NOT affect output.

This rule is a reporting normalization rule only. It MUST NOT hide the existence of unresolved boundaries or convert ambiguous evidence to resolved evidence.

### 8. Output contract

The V1 JSON report MUST contain normalized deterministic collections equivalent to:

```text
schema
subject
before_evidence
after_evidence
seeds
artifact_impacts
package_impacts
unresolved_boundaries
coverage
```

Each impact record MUST identify:

- exact impacted package/artifact identity;
- exact seed identity;
- side (`before`, `after`, or normalized `both` where exact evidence is identical);
- canonical evidence path;
- relationship kind;
- provenance/digest identities needed to reconstruct the result.

Collections MUST be canonically sorted and serialized with stable pretty JSON plus trailing newline.

### 9. Coverage boundary

CF-12 V1 can only prove artifact-level impact across canonical relations extracted by CF-11G V1.

The report MUST carry forward Context Graph extraction coverage and MUST NOT imply exhaustive impact analysis for unsupported resource types or unsupported relation kinds.

Package-level exposure remains available independently of artifact extractor coverage.

### 10. Exit behavior

The command MUST use the repository's stable CLI error discipline and sanitized runtime diagnostics.

V1 `--format` supports JSON only unless a later task explicitly adds another reviewed output contract.

A successful command may contain unresolved boundaries; unresolved evidence is not itself an execution failure when it is explicitly represented. Missing/corrupt required local evidence or an unsupported lock schema is an execution failure.

## Acceptance criteria

### A. Direct artifact impact

A fixture where profile A has a resolved canonical reference to changed profile B MUST report A as artifact-exposed with the exact one-edge path to B.

### B. Transitive artifact impact

A → B → changed C MUST report A and B with deterministic shortest reverse-dependency paths.

### C. Cycle termination

A canonical-reference cycle containing an impacted node MUST terminate and produce deterministic deduplicated impact relations.

### D. Removed target

A canonical artifact removed in the after side but referenced by before-side artifacts MUST retain before-side impact evidence rather than disappearing from the report.

### E. Added target

A newly added canonical artifact with after-side dependents MUST produce after-side exposure evidence.

### F. Ambiguous boundary

An ambiguous CF-11G target MUST never be traversed as resolved and MUST appear as an unresolved boundary with all sorted candidates preserved.

### G. External boundary

An external/unresolved-in-closure target MUST remain explicit and MUST cause no network lookup.

### H. Package exposure

A multi-version package fixture MUST prove exact version-aware reverse package reachability without collapsing same-name package versions.

### I. Reachability is not compatibility

A reachable dependent with no existing breaking classification MUST remain `impacted/exposed` evidence only; the report MUST NOT invent `BREAKING`.

### J. Determinism

Repeated runs over identical pinned before/after inputs MUST produce byte-identical JSON and a retained SHA-256 proof.

### K. Regression

Existing `ci`, `cf06-oracle`, `cf11-multi-version-proof`, `cf11g-context-proof`, real FHIR smoke, and security regressions remain green on the exact candidate head where their path triggers apply.

## Explicit non-goals

CF-12 V1 does not:

- change CF-03 structural diff semantics;
- change CF-04/CF-05 compatibility policy or severity;
- claim that reachability proves runtime or clinical breakage;
- add SQL-on-FHIR, CQL, SearchParameter-expression, or FHIRPath-invariant parsing;
- crawl registries or the internet to resolve missing canonicals;
- introduce a graph database, vector store, embeddings, RAG, model, or agent authority;
- change CF-06 HL7 oracle identity or exception semantics;
- modify the frozen CF-10 corpus;
- require patient/instance data;
- add persistent graph storage without measurement-based justification.

## Evidence and provenance

Every impact report must be reconstructable from the exact before/after lock bytes, verified cache bytes, package/archive digests, Context Graph evidence, selected package identity, graph/report schema versions, and any linked diff/classification evidence.

Mutable aliases or live registry state are not impact evidence.
