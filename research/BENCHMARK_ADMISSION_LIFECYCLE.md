# Benchmark Admission Lifecycle

Status: RESEARCH_PLANNING. Not execution authority. Not a corpus freeze. Not an experiment.

This record removes a circular sequence among the research planning files. It does not admit an item. Current membership is the manifest in `research/CANDIDATE_CORPUS.md`. Results stay `RESULT_PENDING`.

## Dependency graph before this record

| From | Requires | To |
| --- | --- | --- |
| `research/CANDIDATE_CORPUS.md` admission | an adjudication state | `research/ADJUDICATION_PROTOCOL.md` |
| `research/ADJUDICATION_PROTOCOL.md` | an item that already satisfied the corpus gate | `research/CANDIDATE_CORPUS.md` |
| `research/CANDIDATE_CORPUS.md` admission | a split assignment after the held-out split is frozen | `research/PROTOCOL_FREEZE.md` |
| `research/PROTOCOL_FREEZE.md` | a corpus digest, a label set, an adjudication record, and a held-out split in one manifest | a non-empty corpus |

The first pair cannot start, because admission waits for adjudication and adjudication waits for admission. The second pair cannot start, because admission waits for a freeze and the freeze waits for a corpus digest. The empty corpus is the result of that order, not a missing fixture.

## Order after this record

The sequence is one way.

1. Source candidate. A public source is named. It is not a corpus member.
2. Pre-admission evidence. A record binds the exact revision, the exact bytes or a generator and its inputs, the SHA-256, rights, provenance, a case class already named in `research/BENCHMARK_PROTOCOL.md`, the label authority from `research/LABEL_AUTHORITY.md`, the no-PHI state, and a replay procedure. This digest is not corpus membership.
3. Label. When the label is an external published statement quoted inside those pinned bytes, human adjudication is not required. When no such statement exists, `research/ADJUDICATION_PROTOCOL.md` runs on that pre-admission digest. commandF does not mint the label.
4. Candidate corpus. The item may be added only after step 3 is complete. A split assignment is not an entry condition.
5. Split. The policy, the assignment, and the final held-out manifest are separate. Assignment is computed only after a candidate corpus digest exists. The final manifest is bound at the protocol freeze.
6. Protocol freeze. One later record binds the corpus digest, item identities, labels, adjudication records where human adjudication was required, split assignments, benchmark schema version, baseline identities, and the statistics plan. Headline experiments stay blocked until that freeze.

## What stays required

Exact bytes, SHA-256, rights, provenance, a named case class, an independent label, disagreement retention, no commandF self-label, no private clinical data, leakage prevention, a frozen held-out manifest before headline scoring, and a replay path all remain. This record does not drop them and does not run them.

## Split invariants

The split policy is not frozen by this record. The assignment function is specified in `research/SPLIT_POLICY.md` as version `sp-1`. Assignment is a function of the item SHA-256, the leakage group defined in that policy, and that policy version. It is not a function of a commandF score. That specification does not assign the current corpus and does not bind a held-out manifest.

After the candidate corpus digest is fixed:

- every member receives exactly one assignment;
- an item is not removed because a score is bad;
- an item is not moved out of the held-out assignment because a score is bad;
- the assignment is not edited after the protocol freeze;
- commandF output is not an input to the assignment.

```text
SPLIT_POLICY = specified in research/SPLIT_POLICY.md as sp-1
SPLIT_ASSIGNMENT = NOT_ASSIGNED
HELD_OUT_MANIFEST = NOT_BOUND
ITEM_COUNT = recorded in research/CANDIDATE_CORPUS.md
```

## What this record does not decide

It does not place `SRC-HAPI-FHIR` in a case class. It does not execute the upstream HAPI test. It does not authorize CF-17, AF-02, AF-03, AF-04, or CF-18.
