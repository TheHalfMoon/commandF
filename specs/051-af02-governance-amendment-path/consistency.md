# G51 — Consistency and Authority Audit

Status: PLANNING_CANDIDATE / NOT_EXECUTION_AUTHORITY

## Authority mapping

| Question | Existing authority | This proposal |
| --- | --- | --- |
| What is the product execution plan? | `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md` plus canonical Spec Kits | No change. PR #187 V3.2 remains planning only. |
| Who builds the verifier? | Canonical-base AF-02 workflow | Keep base-built verifier and candidate-as-data-only. |
| Why not add an unused Rust verifier file? | `specs/050-af02-precanonical-path-gap/` | Explicitly states it would be inert. |
| Why is a one-time exception contemplated? | Currently protected verifier cannot admit edits to itself; live review-layer admin capability exists | Only a **separate** exact-head founder FD-1 could authorize that one time. |
| Who can permit future protected edits? | A future base-controlled admission algorithm with pre-merged records | Two PRs: amendment then exact blob-matched protected change. Not active yet. |
| When to make check required? | After positive and negative tests and separate FD-2 | Never before that qualification. |
| Is issue #100 fixed? | No: historical Actions bytes remain unavailable | No change; forward durability is separately designed and tested. |
| Can D1 govern compatibility? | No; deterministic evidence and policy remain authority | No change. |
| Are AF-03/04 parallel after AF-01? | Assurance Program depicts parallel units; later candidate playbook serializes | Do not quietly resolve conflicts here; reverify precedence in migration and shape AF-03 separately. |

## Self-consistency checks

1. The only changed paths are this new `specs/051-af02-governance-amendment-path/` planning package.
2. No claim of approval, authority migration, protected-file edit, issue closure, or verifier code execution exists in this package.
3. Every implemented-state checkbox beyond P001–P004 remains unmarked; the planning checklist does not count as CI/review evidence.
4. `spec.md`, `plan.md`, `tasks.md` and `threat-model.md` demand exact blob identity and non-self-approval, but recognize that a future candidate commit SHA cannot be precomputed as part of the first record PR.
5. Merge only after live exact-head required checks, signed commits/DCO and a normal guarded PR merge. Under G53 the current ruleset requires **zero human approvals**; optional independent reviews count as evidence only if actually performed. A later protected-file exception requires its own separate explicit exact-SHA founder FD-1 decision.
6. The previous V3.2 plan proposal remains a candidate even if this package is merged; two planning records cannot authorize execution by assertion alone.

## Before any next step

Re-read live main SHA/tree, PR #187 head/status, issue #100, AF-02 path policy and both rulesets. If live evidence disagrees with this observation, record a new delta without rewriting the historical evidence.

## Live-policy reconciliation (2026-10-10)

- Historical review=1 AF-02 v2 authority remains immutable; G53 tests
  the new live review=0 policy independently on current GitHub.
- Branch rules now require signed+DCO, exact-head assurance CI and
  guarded normal PR merges; no mandatory human review or Code Owner
  approval. Alibaba OCR/Jev are actual optional evidence when run.
- Founder **FD-1 protected-path authorization** remains an independent,
  explicit exact-identity decision and is **not** substituted by the
  removal of general human-review approvals. FD-2 remains separate.
- This planning-only Spec Kit cannot authorize protected-file edits,
  G51 bootstrap, changes to rulesets, or issue #100/#215 closure.
