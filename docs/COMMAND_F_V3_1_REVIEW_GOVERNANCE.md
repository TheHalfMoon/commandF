# commandF V3.1 — Review Governance

Status: **PLANNING_CANDIDATE / FUTURE V3 REVIEW POLICY**

This document records the intended independent-review policy for future V3/V3.1 work. It does not override current canonical V2/AF/CF rulesets or active Spec Kits. If live canonical governance currently requires a different reviewer/check topology, the future V3 migration gate must reconcile that difference explicitly before V3 implementation authority is activated.

## Required future V3/V3.1 review inputs

For source-changing V3/V3.1 implementation work, the planned review stack is:

1. repository-owned deterministic tests, assurance gates, and exact-head CI;
2. **Jev** as a required independent review/qualification input where applicable;
3. **Alibaba Open Code Review** (`alibaba/open-code-review`) as the required external code-review workflow;
4. human disposition of substantive findings and protected merge authority.

## Non-authoritative review services

The following must not be used as qualification evidence for future V3/V3.1 completion:

- CodeRabbit;
- Qodo;
- Cubic;
- similar hosted reviewer scores/checkmarks unless a future canonical governance change explicitly re-authorizes one.

Their presence as an automatic GitHub status or comment does not make them semantic, assurance, or merge authority.

## Evidence rules

- Review evidence must bind the exact candidate head and relevant canonical base.
- Any source-changing head mutation invalidates earlier-head review evidence unless the reviewer/tool contract explicitly and verifiably rebinds to the new exact head.
- Reviewer output is advisory evidence; commandF tests, policy, exact source identity, and human/governance authority decide disposition.
- No reviewer may self-approve its own protected policy/authority mutation.
- A review tool being unavailable is recorded as unavailable, not PASS.
- A reviewer reporting no findings does not replace deterministic tests or prove semantic correctness.

## Jev boundary

Jev may be used to challenge logic, requirements, evidence sufficiency, edge cases, and implementation choices. Jev output must not directly activate rules, suppress findings, mutate protected policy, or become commandF semantic truth.

## Alibaba Open Code Review boundary

Alibaba Open Code Review is used for code-review/accounting and independent findings. Findings must be dispositioned against the exact change. Its output does not replace commandF's deterministic proof or oracle boundaries.

## Migration requirement

The future V3 migration/reconciliation gate must:

- read the live required-check/ruleset topology;
- reconcile any legacy CodeRabbit/Qodo/Cubic references in planning or old acceptance text;
- establish the exact Jev + Alibaba Open Code Review evidence format expected for V3 slices;
- ensure review tooling cannot bypass the independent assurance ruleset;
- retain normal merge commits and prohibit force-push/history rewriting where current governance does so.

Until that migration is canonical, this file is planning intent only.
