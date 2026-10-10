# G53: Approval-free live governance with retained AF-02 v2 provenance

Status: CANDIDATE. On 2026-10-10 the CommandF founder withdrew
mandatory human review. GitHub rule 21652974 now requires zero
approvals and none of the five human-review-specific flags; Pull
Requests and merge-only commits remain mandatory. The separate
assurance rule 21652953 still enforces rust, assurance-proof and
scorecard, and blocks deletion/non-fast-forward.

## Dual fail-closed authority

- HISTORICAL: AF-02 authority-baseline v2 records *past*
  approval=1/Code Owner authority at pinned main 54b9772a...
  Its original source SHA, tree, retained evidence, semantic digest
  and authority verifier fixtures remain immutable and must
  reproduce independently. The v2 reconstruction now uses
  the historical frozen review fixture, never the newer live rule.
- CURRENT: A separate exact live GitHub read-back compares rule
  21652974 against review-policy.json. The strict projection
  includes zero approvals and all five disabled human review flags,
  Pull Request requirement, merge-only method, main branch scope,
  enforcement and unchanged PR-only admin bypass. Explicit
  mutation probes reject restoring approvals, deleting the bypass,
  changing target, or allowing squash merges.

The founder's governance decision is recorded at issue #195,
comment 6094536144. This migration is issue #222. PR #216
was integrated into main as 9074b48c after rule changes.

The retained .github/main-review-ruleset.json, AF-01 closure
and AF-02 v2 baseline remain historical snapshots, not current
deployment instructions; guarded authority file changes require
separate G51/FD-1 authorization and predecessor verification.
Do not retroactively edit or erase them.

The legacy live assurance projection still checks the separately
protected rule 21652953. The new live review comparison is
*additional*, not a waiver or skipped verification. Alibaba OCR
and Jev outcomes must be actual executions, not inferred from tool
installation or provider unavailability.

Admit only with exact-head green CI and no substantive red
checks. Keep issue #214 registry bytes conflict independent;
never change required status checks to evade it. Merge via
normal expected-head merge commit and verify post-merge main.
