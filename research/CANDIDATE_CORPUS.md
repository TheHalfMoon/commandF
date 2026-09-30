# Candidate Corpus

Status: RESEARCH_PLANNING. Empty. Not frozen.

This document is the admission gate for CommandFBench items. It contains no items.

## Admission rule

An item may be added only in a later record that supplies all of the following at once:

- a case class already named in `research/BENCHMARK_PROTOCOL.md`;
- exact bytes, or an exact fixture generator and its inputs;
- SHA-256 of those bytes;
- source identity and rights;
- a label whose authority is not commandF;
- an adjudication state;
- a split assignment that exists only after the split itself is frozen.

A missing field keeps the item out. CommandF output cannot fill the label. A registry response is not a corpus item until its bytes are the bytes that were hashed.

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
