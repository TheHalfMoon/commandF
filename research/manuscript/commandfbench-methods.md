# CommandFBench Methods

Status: RESEARCH_PLANNING. This is the methods text for the admission and analysis records that already exist. It is not a measured result. It is not a protocol freeze.

## Admission

Membership follows `research/BENCHMARK_ADMISSION_LIFECYCLE.md`. A source name is not membership. A pre-admission digest is not membership. An item enters `research/CANDIDATE_CORPUS.md` only after exact bytes, a SHA-256, rights, a named case class, and a label that commandF did not mint. Human adjudication is required only when no external published label exists. No adjudication has been run.

The current candidate corpus has three members, all from `hapifhir/hapi-fhir` at commit `e307df6b64ff87c55af1607160f57141dbeb0360`, under Apache-2.0. The classes are malformed evidence, unsupported evidence, and partial evidence. CommandF minted no label.

```text
ITEM_COUNT = 3
CORPUS_DIGEST = e92d734ab2981691322074f9d963bcf6ffc9058dace3875fdea64d2558b417aa
LABELS = 3 external published labels
LABELS_MINTED_BY_COMMANDF = 0
```

`research/CASE_CLASS_MATRIX.md` records the remaining classes. None of those rows is a fourth member. `FHIR/fhir-test-cases` stays unresolved on rights.

## Split

`research/SPLIT_POLICY.md` version `sp-1` specifies the assignment function. Assignment reads the item digest and the leakage group. It does not read a commandF score. Items that share a source repository, an input digest, a pair id, a variant family, or a near-duplicate group stay in one group. A group is not split across development and held-out.

The three members share `hapifhir/hapi-fhir`, so the group count is one. Assignment requires two groups. The role rule is not applied.

```text
SPLIT_POLICY = SPECIFIED
SPLIT_POLICY_VERSION = sp-1
SPLIT_ASSIGNMENT = NOT_ASSIGNED
HELD_OUT_MANIFEST = NOT_BOUND
```

## Baselines and calculations

`research/BASELINE_IDENTITIES.md` reads the commands at `616ef5d61762ae9870a3036268e33ea21a9cbcc5`. B0 is `commandf diff`. B2 is `commandf check` with its CLI defaults. B1 and B5 are not pinned. B3, B4, B6, and B7 are not implemented. None of them was executed.

`research/STATISTICAL_ANALYSIS_PLAN.md` version `sap-1` names one confirmatory endpoint: the unsafe auto-allow rate on a held-out manifest, reported with coverage. The manifest does not exist, and the mapping from a commandF document to an allow, deny, or abstain is not bound, so the rate stays `RESULT_PENDING`. A missing run stays in the coverage denominator. It is not called an abstention. A zero denominator stays `UNDEFINED`. B0 does not emit a decision class, so sap-1 names no paired decision test. Brier score and expected calibration error are `NOT_APPLICABLE` while no probability is emitted.

## What this text does not claim

No headline number is reported. No baseline is claimed to lack a capability. The protocol freeze in `research/PROTOCOL_FREEZE.md` remains `NOT_FROZEN`. This text does not authorize CF-17, AF-02, AF-03, AF-04, or CF-18.
