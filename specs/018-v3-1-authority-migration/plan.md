# V3.1 Authority Migration Plan

Status: SPEC_CANDIDATE

## Scope in

- Record the reconciliation in `spec.md`.
- Point `docs/COMMAND_F_PLAN_INDEX.md` at this Spec Kit as the migration candidate, still not as product implementation authority.
- Leave CF/AF product code, workflows, lockfiles, and schemas unchanged.

## Scope out

- Durable retained-authority bytes for issue #100.
- Verified-byte and bounded-input repairs for issues #35, #36, #37, #38, and #40.
- CF-06 oracle identity work for issue #15.
- CF-17 through CF-28 implementation.
- Reviewer substitution, ruleset edits, or advisory waivers.
- Closing any issue.

## Why this is the next unit

PR #91 made the V3.1 plan canonical text. The plan itself forbids using that text as execution authority until this gate exists. Starting CF-17 from the planning docs would skip the gate. Starting the #100 durable-byte successor before recording the gate would leave the V3 execution map unreconciled with live history.

## Next unit after this kit is canonical

Create the successor Spec Kit for issue #100's full durable offline retained-authority contract. That successor must obtain or honestly mark unavailable the historical artifact bytes. It must not invent bytes whose digest is `9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612`.
