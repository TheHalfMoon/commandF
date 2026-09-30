# CF-17 Exit Frontier Plan

Status: NOT_EXECUTION_AUTHORITY

## Why this unit

`specs/042-cf17-exit-gap-record/` was written before official acquisition, advertised versions, and the telemetry partition. Its table is historical. This record recomputes that table against canonical main `93a182428533ae8c7b2ab5569fcb07d445f880cd`.

## Scope in

- One requirement table with state, evidence, blocker, and next action.
- An explicit statement that CF-17 is not closed and CF-18 is not started.

## Scope out

- Product code, a registry catalog, AF-03, AF-04, scale numbers, issue #100, issue #15, and CF-18.

## Exit

Normal merge after exact-head required checks. The merge does not authorize the next implementation. The next implementation stays blocked on the rows marked `NOT_AUTHORIZED`, `DEFERRED_BY_PLAYBOOK`, and `NOT_STARTED`.
