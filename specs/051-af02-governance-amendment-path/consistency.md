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
5. Merge only after live exact-head required checks and independent review. The later code-changing implementation receives a **new** exact-SHA approval decision.
6. The previous V3.2 plan proposal remains a candidate even if this package is merged; two planning records cannot authorize execution by assertion alone.

## Before any next step

Re-read live main SHA/tree, PR #187 head/status, issue #100, AF-02 path policy and both rulesets. If live evidence disagrees with this observation, record a new delta without rewriting the historical evidence.
