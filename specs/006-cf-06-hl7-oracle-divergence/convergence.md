# CF-06 Convergence

Status: Reconciled with canonical main; implementation behavior green; exact final documentation-head CI is recorded in GitHub PR metadata

## Decision

```text
CF-06_COMPLETE_READY_FOR_FOUNDER_MERGE_DECISION
```

CF-06 remains an advisory evidence slice. CF-03 owns deterministic structural facts; CF-06 measures agreement/divergence with a pinned official HL7 comparison oracle. The repository now also contains already-canonical CF-04/CF-05 compatibility policy and SARIF/check behavior, CF-07 terminology evidence, and CF-08 GitHub Action integration. Their presence in the reconciled branch does not change CF-06 authority: oracle states remain evidence relationships only and never become compatibility severity, policy, terminology proof, or source-mapping authority.

## Reconciled stack identity

```text
repository: TheHalfMoon/commandF
PR: #7
base branch: main
canonical base SHA: 45a78a8cc8dd0e2f575a56fa79d3a275c0e0fc36
head branch: feat/cf-06-hl7-oracle-divergence
prior CF-06 head: 53598f14505387e7c80c7415212820e314c43c54
reconciliation merge commit: 28612ae46b78c6a7be483dd140e0aa7d16cdb2ce
style-only follow-up: 8f8afdbd515d2f0be60f663226889cf2d0d10ee2
```

The reconciliation used no force-push or history rewrite. It preserves canonical main and adds the CF-06 oracle implementation, tests, donor provenance, Spec Kit authority files, and Java adapter.

## Pinned oracle provenance

```text
project: hapifhir/org.hl7.fhir.core
release: 6.10.2
source commit: d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
validator_cli.jar sha256: a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6
R4 core context: hl7.fhir.r4.core@4.0.1
```

The validator fat jar is not vendored. The Java adapter builds against exact `6.10.2` libraries and consumes public structured comparison objects before rendering. `ComparisonRenderer` HTML is not parsed and private comparer nodes are not accessed through reflection.

## Reconciliation architecture

The current canonical main already exposes the CF-07 matched-resource path used by terminology. CF-06 requires matched StructureDefinition pairs for the HL7 oracle. Reconciliation therefore keeps one matching authority:

- `matched_resource_pairs(...)` remains the internal shared matcher;
- `matched_structure_definition_pairs(...)` is a narrow public projection over those matched pairs;
- no second resource matcher was introduced;
- corrected CF-03 fail-closed structural validation remains intact;
- current `check`, SARIF, GitHub annotations, composite Action, and terminology commands remain intact;
- `commandf oracle` is added alongside them without changing their semantics.

The general mainline workflow remains `.github/workflows/ci.yml`. Oracle-specific Java/HL7 integration gates live separately in `.github/workflows/cf06-oracle.yml`; the final form intentionally contains the `oracle-adapter` job only, avoiding a duplicate Rust/mainline CI job.

## Implemented CF-06 contract

CF-06 provides:

- isolated Java 17 adapter under `tools/hl7-oracle/`;
- explicit local R4 core/before/after package context with no oracle-time package acquisition;
- commandF-owned schema-v1 oracle evidence with exact provenance validation;
- deterministic message sorting/de-duplication and bounded evidence strings/counts;
- reuse of CF-03 canonical matching rather than a second matcher;
- evidence relationships `agreement`, `commandf_only`, `authority_only`, `both_changed`, and `uncomparable`;
- complete unmodified structural diff evidence embedded in the CF-06 report;
- `commandf oracle` with explicit lock/cache/adapter/Java inputs;
- no implicit PATH lookup for the adapter/Java boundary;
- 60-second per-pair timeout, 8 MiB stdout cap, and 1 MiB stderr cap;
- fail-closed malformed JSON, wrong provenance, non-zero exit, timeout, corrupted cache, missing context, and invalid snapshot behavior;
- Unix process-group termination and Windows process-tree termination fallback so descendants cannot retain inherited pipes past timeout.

`both_changed` does not claim field-level semantic equivalence, and no CF-06 status is a compatibility severity judgment.

## Reconciled implementation evidence

Exact reconciled implementation head:

```text
8f8afdbd515d2f0be60f663226889cf2d0d10ee2
```

Mainline GitHub Actions run:

```text
31848197442
```

CF-06 oracle GitHub Actions run:

```text
31848197445
```

Both runs completed successfully.

### Mainline `ci`

- Format — PASS
- locked workspace Clippy with `-D warnings` — PASS
- full workspace tests — PASS
- CF-08 Action runner security regression — PASS
- real FHIR inspect / self-diff / self-classify / self-check — PASS
- real FHIR self-terminology — PASS
- local composite GitHub Action self-check — PASS
- Action output verification — PASS

### `cf06-oracle`

- build pinned HL7 oracle adapter — PASS
- resolve pinned real R4 oracle context — PASS
- real HL7 R4 profile self-equivalence oracle smoke — PASS
- real `commandf oracle` self-diff smoke — PASS
- deterministic changed-profile fixture construction — PASS
- invalid empty snapshot fails closed — PASS
- corrupted oracle caches fail closed on both sides — PASS
- real HL7 changed-profile evidence is deterministic — PASS
- real `commandf oracle` changed-profile reconciliation — PASS

This proves both directions of composition: the oracle works on the current canonical product surface, and the existing check/terminology/Action surfaces remain green with the oracle present.

## Reviewer reconciliation

### CodeRabbit

Historical substantive CF-06 review produced three actionable inline findings:

1. unused `Path` import causing configured Rust checks to fail — **valid / fixed / thread resolved**;
2. timeout killed only the direct child, allowing descendants to retain output pipes — **valid Major / fixed with process-tree termination / regression-tested / thread resolved**;
3. implementation-plan CLI omitted the required `--oracle-java` path for a JAR adapter — **valid / fixed / thread resolved**.

All three historical inline threads remain resolved. The reconciled implementation head reports CodeRabbit commit status `success`, but that status context is not represented as a fresh substantive full re-review PASS.

The generic CodeRabbit docstring-coverage warning remains non-functional reviewer metadata and is not a CF-06 behavioral acceptance gate.

### Qodo

No substantive Qodo review result was observed. **No Qodo PASS is claimed.**

### Cubic

Cubic-generated summaries are informational and are not treated as oracle correctness or merge certification.

## Spec Kit reconciliation

The canonical CF-06 authority set remains:

- `specs/006-cf-06-hl7-oracle-divergence/spec.md`
- `specs/006-cf-06-hl7-oracle-divergence/plan.md`
- `specs/006-cf-06-hl7-oracle-divergence/tasks.md`
- `specs/006-cf-06-hl7-oracle-divergence/convergence.md`

CF-06 semantics remain advisory and independent even though its reconciled branch necessarily contains sibling capabilities already canonical on `main`.

## Final documentation-head validation rule

This file intentionally records the reconciled implementation head and its two successful workflow runs, not the SHA/run generated by this documentation cleanup commit itself. Embedding the latter would create an endless self-referential commit chain.

The exact final documentation head must independently pass both configured workflows. Those exact final head/run identifiers are recorded in PR metadata after the workflows settle. Any final-head failure reopens convergence.

## Explicit non-authority boundaries

CF-06 does not own or alter:

- CF-04 compatibility severity or producer/consumer rules;
- CF-05 SARIF/check policy-failure semantics;
- CF-07 terminology expansion/set-inclusion evidence;
- CF-08 GitHub annotations or Action behavior;
- CF-09 FSH/repository source mapping;
- ecosystem dependency graph or blast radius;
- mapping execution;
- AI/agent semantic authority.

## Stop condition

```text
CF-06: COMPLETE, SUBJECT TO EXACT FINAL DOCUMENTATION-HEAD DUAL-WORKFLOW PASS
PR #7: OPEN / DRAFT / UNMERGED UNTIL THAT PASS
AUTO-MERGE: DISABLED
CF-09: SEPARATE / IN PROGRESS / NOT AUTHORIZED BY CF-06
```
