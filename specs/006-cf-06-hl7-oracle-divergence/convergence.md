# CF-06 Convergence

Status: Converged implementation; exact final documentation-head CI is recorded in GitHub PR metadata

## Decision

```text
CF-06_COMPLETE_READY_FOR_FOUNDER_REVIEW
```

CF-06 is implemented as a parallel slice directly above converged CF-03. It remains advisory evidence only: CF-03 owns deterministic structural facts, while CF-06 measures agreement/divergence with a pinned official HL7 comparison oracle. No CF-04/CF-05 compatibility severity, policy, SARIF, or exit semantics are imported into this slice.

## Stack identity

```text
repository: TheHalfMoon/commandF
PR: #7
base branch: feat/cf-03-structural-diff
base SHA: aa212b108e05fa0e22312f244f393c59602192b9
head branch: feat/cf-06-hl7-oracle-divergence
```

The PR must remain Draft, open, and unmerged. CF-07 is not part of this convergence.

## Pinned oracle provenance

```text
project: hapifhir/org.hl7.fhir.core
release: 6.10.2
source commit: d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
validator_cli.jar sha256: a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6
R4 core context: hl7.fhir.r4.core@4.0.1
```

The validator fat jar is not vendored. The Java adapter builds against exact `6.10.2` libraries and consumes public structured comparison objects before rendering. `ComparisonRenderer` HTML is not parsed and private comparer nodes are not accessed through reflection.

## Implemented contract

CF-06 now provides:

- isolated Java 17 adapter under `tools/hl7-oracle/`;
- explicit local R4 core/before/after package context with no oracle-time package acquisition;
- commandF-owned schema-v1 oracle evidence with exact provenance validation;
- deterministic message sorting/de-duplication and bounded evidence strings/counts;
- reuse of CF-03 matched canonical StructureDefinition pairs rather than a second matcher;
- evidence relationships `agreement`, `commandf_only`, `authority_only`, `both_changed`, and `uncomparable`;
- complete unmodified CF-03 structural report embedded in the CF-06 report;
- `commandf oracle` with explicit lock/cache/adapter/Java inputs;
- no implicit PATH lookup for the adapter/Java boundary;
- 60-second per-pair timeout, 8 MiB stdout cap, 1 MiB stderr cap;
- fail-closed malformed JSON, wrong provenance, non-zero exit, timeout, corrupted cache, missing context, and invalid snapshot behavior;
- Unix process-group termination and Windows process-tree termination fallback so descendants cannot retain inherited pipes past timeout.

CF-06 remains evidence-only. `both_changed` does not claim field-level semantic equivalence, and no status is a compatibility severity judgment.

## Implementation evidence

Exact implementation head:

```text
fae23c8b555ae2ecaa5feb9fd30f2c095575738a
```

GitHub Actions run:

```text
31815040530
```

That run completed successfully with both jobs green.

### Rust job

- Format — PASS
- locked workspace Clippy with `-D warnings` — PASS
- full workspace tests — PASS
- existing real FHIR registry / inspect / independent CF-03 self-diff smoke — PASS

### Oracle-adapter job

- build pinned HL7 oracle adapter — PASS
- resolve pinned real R4 oracle context — PASS
- real HL7 R4 profile self-equivalence oracle smoke — PASS
- real `commandf oracle` self-diff smoke — PASS
- deterministic changed-profile fixture construction — PASS
- invalid empty snapshot fails closed — PASS
- corrupted oracle caches fail closed on both sides — PASS
- real HL7 changed-profile evidence is deterministic — PASS
- real `commandf oracle` changed-profile reconciliation — PASS

The real changed-profile reconciliation proves a positive oracle/CF-03 change relationship while an unchanged control proves `agreement`; the self-equivalence gate proves the pinned HL7 comparer does not create a false divergence for identical R4 input.

## Reviewer reconciliation

### CodeRabbit

A substantive review produced three actionable inline findings:

1. unused `Path` import causing configured Rust checks to fail — **valid / fixed / thread resolved**;
2. timeout killed only the direct child, allowing descendants to retain output pipes — **valid Major / fixed with process-tree termination / regression-tested / thread resolved**;
3. implementation-plan CLI omitted the required `--oracle-java` path for a JAR adapter — **valid / fixed / thread resolved**.

All three CodeRabbit inline review threads are resolved. Exact implementation run `31815040530` validates the fixes. CodeRabbit commit status at the implementation head is `success`.

CodeRabbit's high-level walkthrough still contains historical risk wording generated against earlier heads, including earlier CI/process concerns. That text is not treated as a fresh final manual PASS or as current exact-head evidence. The resolved inline threads plus exact implementation CI are the authoritative disposition evidence.

The generic CodeRabbit docstring-coverage warning is non-functional reviewer metadata and is not represented as a CF-06 behavioral PASS requirement.

### Qodo

No substantive Qodo review result was observed during convergence. **No Qodo PASS is claimed.**

### Cubic

Cubic generated PR summaries describing the implementation. They are informational and are not treated as oracle correctness or merge certification.

## Spec Kit reconciliation

The canonical CF-06 authority set is:

- `specs/006-cf-06-hl7-oracle-divergence/spec.md`
- `specs/006-cf-06-hl7-oracle-divergence/plan.md`
- `specs/006-cf-06-hl7-oracle-divergence/tasks.md`
- `specs/006-cf-06-hl7-oracle-divergence/convergence.md`

The specification and plan are reconciled to the implemented structured HL7 API, explicit local context, structured Java/Rust schema, hardened process tree, CLI inputs, real R4 gates, and CF-07+ deferrals. The task checklist records T001-T014 complete subject to the exact final documentation-head GitHub Actions gate described below.

## Final documentation-head validation rule

This file intentionally does not embed the SHA or Actions run created by its own documentation commit. Doing so would create an endless self-referential commit chain.

Therefore convergence uses two evidence layers:

1. the implementation behavior is certified by exact implementation head `fae23c8b555ae2ecaa5feb9fd30f2c095575738a` and run `31815040530`;
2. the final documentation head must independently pass the same configured Rust and oracle-adapter workflow, and that exact head/run is recorded in PR metadata after the workflow settles.

A final documentation-head CI failure reopens convergence and must be corrected before founder review.

## Explicit deferrals

CF-06 does not introduce:

- CF-04 compatibility severity or producer/consumer rules;
- CF-05 SARIF or policy-failure exits;
- CF-07 terminology expansion/set inclusion;
- GitHub source annotations/upload;
- FSH/repository source mapping;
- ecosystem dependency graph or blast radius;
- mapping execution;
- AI/agent semantic authority.

## Stop condition

```text
CF-06: COMPLETE, SUBJECT TO EXACT FINAL DOCUMENTATION-HEAD CI PASS
PR #7: MUST REMAIN DRAFT / OPEN / UNMERGED
AUTO-MERGE: MUST REMAIN DISABLED
CF-07 STARTED: NO
```
