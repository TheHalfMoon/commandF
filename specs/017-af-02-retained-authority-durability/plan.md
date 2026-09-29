# AF-02 Retained Authority Durability Repair Plan

Status: BOOTSTRAP_REPAIR_CANDIDATE
Issue: #100
Entry main: `18819c7fdaee618f4aa86c6c3279b1acb70ec9f2`

## Scope

This repair is intentionally narrow.

In scope:

- distinguish historical live-artifact unavailability from retained-authority mismatch in the canonical reconstruction test;
- preserve the exact canonical retained contract and CF-10 projection;
- prove the shared candidate-input artifact verifier remains strict;
- restore fresh CI determinism after normal GitHub Actions retention removes the historical artifact.

Out of scope:

- changing `retained-authority-sources.json` historical identities;
- changing `verify_artifacts` shared candidate-input semantics;
- changing the AF-02 authority-baseline schema or snapshot;
- changing CF-06 pins or semantics;
- changing the frozen CF-10 corpus;
- replacing the expired artifact with a new artifact and calling it historical authority;
- V3/V3.1 implementation;
- issue #99 dependency repair.

## Implementation

1. Load and validate the retained contract from the already pinned canonical authority-base commit/blob.
2. Reconstruct and verify the exact live historical workflow run.
3. Fetch the run-artifacts collection.
4. If `total_count > 0`, execute the unchanged strict `verify_artifacts` path.
5. If and only if the API reports `total_count == 0` and an empty artifact array, classify the live hosting state as `HISTORICAL_UNAVAILABLE` and preserve the artifact identity/digest solely from the already pinned canonical retained contract.
6. Continue reconstructing the semantic CF-10 projection from exact manifest/donor Git objects.
7. Compare generated authority baseline bytes against the unchanged canonical baseline.

## Required regression

A deterministic regression must prove both facts simultaneously:

- the live-only historical-unavailability helper accepts a structurally exact empty live collection after canonical retained-contract binding;
- the unchanged shared `verify_artifacts` function rejects that same empty collection.

This is the critical anti-self-green property.

## Qualification sequence

1. exact branch/head identity;
2. diff review confirms only the bounded test boundary plus Spec Kit changed;
3. `cargo fmt --check`;
4. relevant AF-02 reconstruction tests;
5. full locked workspace CI;
6. AF-01 security/assurance reconciliation;
7. CF-06 oracle workflow;
8. Jev review where executable;
9. Alibaba Open Code Review where executable;
10. all substantive findings dispositioned;
11. exact-head checks re-read;
12. guarded normal merge only if current repository governance permits it;
13. post-merge fresh-main verification.

## Interaction with #99

The branch is expected to remain red on AF-01 dependency security until issue #99 is repaired. That independent failure must not be interpreted as failure of this AF-02 repair and must not be waived inside this branch.

Conversely, issue #99 must not absorb this AF-02 change.
