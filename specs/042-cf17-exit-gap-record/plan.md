# CF-17 Exit Gap Record Plan

Status: NOT_EXECUTION_AUTHORITY

## Why this unit

`specs/041-cf17-offline-replay/` is merged as `eb83dce4a7e4430c8adc2a9ae37a9ecf8ba84648`. Its plan says official registry or feed acquisition stays unauthorized until a later spec says otherwise. No such spec is on `main`. This record traces that sentence to the playbook and to `specs/018-v3-1-authority-migration/spec.md`. It does not supply the missing authorization.

## Scope in

- A checklist of CF-17 requirements with state, evidence, blocker, and next action.

## Scope out

- Registry client, crawler, or snapshot publication code.
- CF-18 schemas.
- Issue #100 byte recovery and issue #15 pin changes.
- Benchmark numbers, experiment results, and paper claims.

## Exit

Normal merge of this record after exact-head required checks. The next product implementation unit remains unauthorized. Official registry acquisition still needs a later Spec Kit that scopes it in. CF-18 stays closed. Issue #100 and issue #15 stay open.
