# AF-02 Review Gate Block

Status: NOT_EXECUTION_AUTHORITY

Observed canonical main: `94f0ec6a5daa33be697a84c1e7be369ae9beea51`

## What was attempted

The AF-02 planning candidate still names Qodo and CodeRabbit as qualification evidence in `specs/016-af-02-adversarial-test-strength/`. Current governance does not. A reconciliation edited those planning files on pull request #146, head `7dc731c4e63e9810f8625f93fc280d0188db4b95`.

## What the gate did

`af02-base-verifier` run `36750827129` failed before any product behavior changed:

```text
canonical-base AF-02 authority is immutable under the A0 gate; dedicated precanonical strengthening is required
```

The rejected paths were the existing files:

- `specs/016-af-02-adversarial-test-strength/evidence-contracts.md`
- `specs/016-af-02-adversarial-test-strength/plan.md`
- `specs/016-af-02-adversarial-test-strength/spec.md`
- `specs/016-af-02-adversarial-test-strength/tasks.md`
- `specs/016-af-02-adversarial-test-strength/verification-protocol.md`

Required checks `rust`, `assurance-proof`, and `scorecard` passed on that head. Pull request #146 was closed and was not merged. This record does not weaken the gate.

The gate runs the canonical-base verifier. `tools/af02-verifier/` and `.github/scripts/` are authority prefixes in that same gate. A candidate cannot edit the verifier and then use the edited verifier to admit an edit of the planning files.

## What remains true

AF-01 remains `CLOSED_CANONICAL` in `specs/015-af-01-trusted-development-baseline/closeout.md`.

Live ruleset `21652953` requires `rust`, `assurance-proof`, and `scorecard`. Live ruleset `21652974` is active.

`retained-authority-sources.json` and `schemas/af02-retained-authority-sources-v1.schema.json` are not in the tree. Historical artifact `9255732702` remains identified and its bytes remain unavailable.

T004, T005, and T006 stay open. `AF-02 PLANNING` stays `PLANNING_CANDIDATE`. Stack A0 is not authorized. AF-03 and AF-04 are not eligible. The hosted-review sentences in the planning candidate remain the text on disk. They are not treated as current qualification evidence by this record, and this record does not rewrite them.

## Next action

Do not edit the existing AF-02 authority files again until a canonical-base verifier can accept a named precanonical strengthening without turning every later edit into an allowed rewrite. This record is not that strengthening.
