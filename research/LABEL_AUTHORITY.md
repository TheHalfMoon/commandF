# Independent Label Authority

Status: RESEARCH_PLANNING. No labels assigned.

A CommandFBench label is not a commandF result. commandF may be the system under test. It cannot mint the label it is later scored against.

## Who may label

A label is admissible only when its authority is one of these, named in the same record as the pre-admission digest:

- an external published artifact whose bytes match the recorded SHA-256, and whose relevant statement is quoted by location inside those bytes;
- a human adjudication record that names each adjudicator, the item digest, the label, and the disagreement if the adjudicators do not agree.

These are alternatives. An external published label does not also require a human adjudication. Human adjudication, when it is the authority used, is performed on that pre-admission digest and does not wait for corpus membership. CommandF output, a model probability, and an unlabeled package listing are not label authorities.

## What this record does not contain

```text
ASSIGNED_LABELS = 0
ADJUDICATION_RUNS = 0
AGREEMENT = NOT_COMPUTED
```

Disagreement is retained when an adjudication later happens. It is not averaged into a pass. This document does not adjudicate any item, freeze a split, or authorize an experiment.

The procedure for a later adjudication is `research/ADJUDICATION_PROTOCOL.md`. It has not been run.
