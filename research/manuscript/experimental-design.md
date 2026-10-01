# Experimental Design

Status: RESEARCH_PLANNING. Methods only. Not a run. Not a freeze.

The admission facts, the current corpus, the split record, the baseline identities, and the analysis rules are already written in `research/manuscript/commandfbench-methods.md`. This section states what an experiment is allowed to be. It does not repeat that methods text as a new measurement.

## What would have to be bound

`research/PROTOCOL_FREEZE.md` is `NOT_FROZEN`. A later freeze would bind a protocol version, the candidate digest, the label set, any adjudication records that were required, a held-out manifest computed after that digest, pinned baselines, and the statistics plan. Those binds are absent. `research/EXPERIMENT_REGISTRY.md` has one row, E0, status `NOT_RUN`, result `RESULT_PENDING`. No later experiment id exists.

The corpus used by any future headline run is the candidate set only after the lifecycle says it is frozen. The current set has three members and one source family. That set is not declared final. `research/CASE_CLASS_MATRIX.md` leaves the remaining classes blocked or low. This design does not add a member to clear a class.

## Factors the design already names

Labels come from outside commandF, or from the adjudication procedure in `research/ADJUDICATION_PROTOCOL.md` when no published label exists. Adjudication runs are zero. A `DISAGREEMENT` is retained. commandF does not mint the label and does not choose a surviving label.

Split assignment is `sp-1`. It stays unassigned while one leakage group is the whole corpus. The role rule is not applied. The held-out manifest is not bound. sap-1 therefore has no held-out items, and the primary rate stays `RESULT_PENDING`. The mapping from a command document to `ALLOW`, `DENY`, or `ABSTAIN` is not bound either.

Baseline families stay as recorded: B0 `commandf diff`, B2 `commandf check` at the pinned command identity, B1 and B5 `NOT_PINNED`, B3, B4, B6, and B7 `NOT_IMPLEMENTED`. Execution is `NOT_EXECUTED`. This design does not substitute a command for a missing family.

`research/ABLATIONS.md` plans removals of consumer contracts, the context graph, oracle escalation, terminology evidence, abstention, evidence completeness, and advisory calibration. Their measured contribution is `RESULT_PENDING`. Several of those factors are not implemented, so a removal study cannot be run by turning off a switch that does not exist.

## What this section does not claim

It does not claim the three-item corpus is the benchmark. It does not claim a protocol version. It does not claim an experiment ran. Results, ablation numbers, calibration, and performance stay `RESULT_PENDING` in `research/PAPER_OUTLINE.md`.
