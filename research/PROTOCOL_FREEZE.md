# Benchmark Protocol Freeze

Status: NOT_FROZEN.

A freeze is the last bind in `research/BENCHMARK_ADMISSION_LIFECYCLE.md`. It would bind a protocol version, a candidate corpus digest, a label set, human adjudication records where those were required, a held-out manifest computed after that digest, pinned baselines, and a statistics plan. None of those binds exist. Corpus membership does not wait for this freeze.

```text
PROTOCOL_VERSION = NOT_FROZEN
CORPUS_DIGEST = NOT_COMPUTED
ITEM_COUNT = 0
LABELS = NOT_ASSIGNED
ADJUDICATION_RUNS = 0
HELD_OUT_SPLIT = NOT_FROZEN
BASELINES = NOT_EXECUTED
EXPERIMENTS = NOT_RUN
```

The case classes, provenance inventory, empty candidate gate, label rule, and adjudication procedure stay planning records. They do not become the freeze by being merged. An experiment stays blocked until a later record supplies the missing binds from real bytes.
