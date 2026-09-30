# AF-02 Precanonical Path Gap

Status: NOT_EXECUTION_AUTHORITY

Observed canonical main: `c6de83595ee54d601a8bc7af5317cae5dd118a90`

## Question

Can one absent, already-known authority path carry operational precanonical strengthening that the current canonical-base gate accepts, without editing an existing authority file?

## Answer

No. The current gate accepts an absent inventory path only as unexecuted candidate data. Nothing in the canonical runner compiles or calls that data. Making it executable requires an edit to an existing authority file, and that edit is rejected by the same gate.

This record is not a strengthening and is not a bypass.

## How the gate classifies a path

`known_authority_paths` in `tools/af02-verifier/src/base_gate.rs` is the exact paths, the tracked files under `.github/scripts`, `.github/workflows`, `donors`, `specs/016-af-02-adversarial-test-strength`, and `tools/af02-verifier`, plus every `planned_path` in `enforcement-inventory.json` that does not end with `/`.

`.github/workflows/af02-base-verifier.yml` checks out the base and builds `tools/af02-verifier` from that base. The candidate tree is data. `candidate_code_executed` stays false.

`tools/af02-verifier/src/lib.rs` declares `authority`, `canonical`, `corpus`, `github_provenance`, `resource`, `retained`, `surface`, `surface_proof`, and `waiver`. It does not declare `tool_lock`, `policy`, `replay`, `nextest`, `coverage`, `mutation`, or `proof`. `Cargo.toml` does not add a binary for those files. An unreferenced `.rs` file is not part of the base build.

## Frozen future paths

| PATH | EXISTS_ON_BASE | KNOWN_TO_BASE_GATE | IN_ENFORCEMENT_INVENTORY | REQUIRED_FROM_STACK | CURRENT_GATE_RESULT_IF_ADDED | CAN_BE_OPERATIONAL_WITHOUT_EDITING_EXISTING_AUTHORITY | SAFE_FOR_PRECANONICAL_STRENGTHENING |
| --- | --- | --- | --- | --- | --- | --- | --- |
| tools/af02-verifier/src/tool_lock.rs | no | yes | yes | A0 | FUTURE_AUTHORITY_ADDITION_VERIFIED, not executed | no; needs an existing module or manifest edit | no |
| tools/af02-verifier/src/policy.rs | no | yes | yes | A0 | same | no | no |
| tools/af02-verifier/src/replay.rs | no | yes | yes | A1 | same; unit test names this path | no | no |
| tools/af02-verifier/src/nextest.rs | no | yes | yes | B0 | same | no | no |
| tools/af02-verifier/src/coverage.rs | no | yes | yes | B0 | same | no | no |
| tools/af02-verifier/src/mutation.rs | no | yes | yes | C0 | same | no | no |
| tools/af02-verifier/src/proof.rs | no | yes | yes | C0 | same | no | no |

Paths that already exist, including `enforcement-inventory.json`, the verifier sources, the workflow, the runner script, and the AF-02 planning files, are immutable. Adding `retained-authority-sources.json` is not an admission question: the file is already tracked.

`specs/049-af02-review-gate-block/spec.md` says those retained-source files are not in the tree. Re-reading `c6de83595ee54d601a8bc7af5317cae5dd118a90` shows both files are tracked. This record corrects that observation and does not rewrite spec 049.

A local structural comparison of `retained-authority-sources.json` with `schemas/af02-retained-authority-sources-v1.schema.json` found every required field, no extra field, and no `const` mismatch. T004 still requires reconstruction of AF-01 and CF-06 expected authority. Those identities are not fields of this closed schema. T004 stays `NOT_PROVEN`. Issue #100 stays open. The historical artifact bytes are not fabricated.

## Why no higher authority was invoked

The phrase "dedicated precanonical strengthening is required" is the gate's own rejection text. No tracked ruleset, workflow, or Spec Kit names a migration artifact, a one-time exception, or a second verifier that can admit an edit of existing AF-02 authority. Creating that mechanism would edit an existing authority file or would be unused data.

Do not disable `af02-base-verifier`. Do not weaken a ruleset. Do not reopen pull request #146. Do not edit `specs/016-af-02-adversarial-test-strength/`, `tools/af02-verifier/`, or `.github/workflows/` in this record.

AF-02 planning stays `PLANNING_CANDIDATE`. Stack A0 stays unauthorized. AF-03 and AF-04 stay ineligible. T005 and T006 stay open.
