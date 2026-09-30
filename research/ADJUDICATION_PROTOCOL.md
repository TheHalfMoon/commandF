# Adjudication Protocol

Status: RESEARCH_PLANNING. No adjudication has been run.

This protocol applies to a pre-admission item digest from `research/BENCHMARK_ADMISSION_LIFECYCLE.md`. It does not require corpus membership first. It is used when `research/LABEL_AUTHORITY.md` does not already accept an external published label. No human adjudication has been run.

## Procedure

Two adjudicators record labels independently. Each record names the adjudicator, the item SHA-256, the label, and the time as provenance only. The adjudicators do not see commandF output before recording the label.

If the two labels match, the item stores that label and records both adjudicator identities. If they differ, the item stores both labels and the state `DISAGREEMENT`. A later third record may be added. It does not delete the earlier labels. commandF does not choose the surviving label.

A missing adjudicator identity, a missing item digest, or a label with no authority record rejects the adjudication.

## Current execution

```text
ADJUDICATION_RUNS = 0
DISAGREEMENTS = 0
RESOLVED_ITEMS = 0
```

This protocol is not a frozen benchmark, a held-out split, or an experiment.
