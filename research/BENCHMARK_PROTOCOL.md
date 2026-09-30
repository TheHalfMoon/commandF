# Benchmark Protocol

Status: RESEARCH_PLANNING. Not a completed benchmark.

CommandFBench is intended to include historical package changes, profiles, extensions, search, terminology, query artifacts, protocol expectations, consumer-contract breakage, oracle disagreement, transformations, adversarial and irrelevant edits, malformed and partial evidence, and cases where abstention is correct.

No item is labeled by CommandF as its own ground truth. Protocol version, corpus digest, and split freeze are `RESULT_PENDING`.

## Case classes

Each class below is planned. None has a frozen item, a count, or a label produced by CommandF. Ground truth must come from an external artifact or an explicit adjudication record. A missing artifact stays `RESULT_PENDING`.

| Class | Planned content | Corpus state |
| --- | --- | --- |
| FHIR package and IG evolution | A published package identity changes between pinned versions | `RESULT_PENDING` |
| Profiles | A profile constraint changes between those versions | `RESULT_PENDING` |
| Extensions | An extension definition or binding changes | `RESULT_PENDING` |
| SearchParameters | A search parameter identity or semantics changes | `RESULT_PENDING` |
| Terminology | A code system, value set, or binding changes | `RESULT_PENDING` |
| FHIRPath | A path expression changes its accepted or rejected inputs | `RESULT_PENDING` |
| CQL and ELM | A clinical logic artifact changes its accepted or rejected inputs | `RESULT_PENDING` |
| SQL-on-FHIR | A view definition changes its result membership | `RESULT_PENDING` |
| Protocol behavior | A request or response expectation changes | `RESULT_PENDING` |
| Authorization behavior | A permission or consent expectation changes | `RESULT_PENDING` |
| Consumer dependencies | A downstream contract depends on an upstream element | `RESULT_PENDING` |
| Oracle disagreement | Two named oracles classify the same inputs differently | `RESULT_PENDING` |
| Transformations | A declared transform changes an instance or artifact | `RESULT_PENDING` |
| Adversarial mutations | A change is constructed to resemble a safe edit and is not | `RESULT_PENDING` |
| Irrelevant mutations | A change is constructed to leave the claimed contract unchanged | `RESULT_PENDING` |
| Malformed evidence | The input bytes do not parse as the declared artifact | `RESULT_PENDING` |
| Partial evidence | A required evidence class is absent | `RESULT_PENDING` |
| Unsupported evidence | The declared evidence class is outside the evaluator contract | `RESULT_PENDING` |
| Conflicting evidence | Two supplied evidence classes do not agree | `RESULT_PENDING` |
| Correct abstention | The safe output is to refuse a proven verdict | `RESULT_PENDING` |

This table is not a freeze. It does not authorize a product change, a registry download, or an experiment run.
