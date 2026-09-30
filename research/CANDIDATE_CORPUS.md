# Candidate Corpus

Status: RESEARCH_PLANNING. Empty. Not frozen.

This document is the admission gate for CommandFBench items. It contains no items.

## Admission rule

The order is `research/BENCHMARK_ADMISSION_LIFECYCLE.md`. A pre-admission digest is not membership.

An item may be added only after a later record supplies all of the following:

- a case class already named in `research/BENCHMARK_PROTOCOL.md`;
- exact bytes, or an exact fixture generator and its inputs;
- SHA-256 of those bytes;
- source identity and rights;
- a label whose authority is not commandF, under `research/LABEL_AUTHORITY.md`;
- either an external published label that already satisfies that rule, or a completed human adjudication of the pre-admission digest, including a retained `DISAGREEMENT`;
- a statement that the bytes are not private clinical data;
- a replay procedure.

A split assignment is not required to enter this corpus. It is computed later from a frozen candidate digest, and the held-out manifest is bound at the protocol freeze. A missing field above keeps the item out. CommandF output cannot fill the label. A registry response is not a corpus item until its bytes are the bytes that were hashed.

## Current manifest

```text
ITEM_COUNT = 0
CORPUS_DIGEST = NOT_COMPUTED
LABELS = NOT_ASSIGNED
HELD_OUT_SPLIT = NOT_FROZEN
RIGHTS = NOT_RECORDED
```

No package name, version, or fixture is a member. This gate does not authorize a download, a product change, or an experiment.

Label authority is specified in `research/LABEL_AUTHORITY.md`. That specification assigns no labels.
