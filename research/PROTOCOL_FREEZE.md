# Benchmark Protocol Freeze

Status: NOT_FROZEN.

A freeze is the last bind in `research/BENCHMARK_ADMISSION_LIFECYCLE.md`. It would bind a protocol version, a candidate corpus digest, a label set, human adjudication records where those were required, a held-out manifest computed after that digest, pinned baselines, and a statistics plan. None of those binds exist. Corpus membership does not wait for this freeze.

```text
PROTOCOL_VERSION = NOT_FROZEN
FREEZE_MANIFEST = NOT_BOUND
CANDIDATE_MEMBERSHIP = see research/CANDIDATE_CORPUS.md
LABELS = not bound by this freeze
ADJUDICATION_RUNS = 0
HELD_OUT_SPLIT = NOT_FROZEN
BASELINES = NOT_EXECUTED
EXPERIMENTS = NOT_RUN
```

The case classes, provenance inventory, candidate corpus, label rule, and adjudication procedure stay planning records. They do not become this freeze by being merged. The candidate count and digest, if any, live in `research/CANDIDATE_CORPUS.md`. `research/SPLIT_POLICY.md` version `sp-1` specifies the assignment function and does not bind the held-out manifest this freeze still requires. An experiment stays blocked until a later record supplies the missing binds from real bytes.
