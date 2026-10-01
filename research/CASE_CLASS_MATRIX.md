# Case Class Matrix

Status: RESEARCH_PLANNING. Not a corpus item. Not a protocol freeze.

This matrix covers the classes in `research/BENCHMARK_PROTOCOL.md` after the three candidate members. It does not add a class. `ITEM_COUNT` stays 3. `CORPUS_DIGEST` stays `e92d734ab2981691322074f9d963bcf6ffc9058dace3875fdea64d2558b417aa`.

The three members already bound are `Malformed evidence`, `Unsupported evidence`, and `Partial evidence`, all from `hapifhir/hapi-fhir` at `e307df6b64ff87c55af1607160f57141dbeb0360`. They are not repeated here as new candidates.

## Remaining classes

| CASE_CLASS | REQUIRED_EVIDENCE_SHAPE | CANDIDATE_SOURCE | RIGHTS_STATUS | EXACT_REVISION | EXACT_BYTES_AVAILABLE | EXTERNAL_LABEL_AVAILABLE | HUMAN_ADJUDICATION_REQUIRED | REPLAY_FEASIBLE | ADMISSION_PROBABILITY | BLOCKER |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Conflicting evidence | Two supplied evidence classes do not agree | `SRC-MICROSOFT-FHIR` method read, not used | MIT | `e164d39f1eff719bf0fd1f8ece559f52c93c1c9b` | no | the test does not state that disagreement | no, because there is no item | the method exists; this record did not run it | low | the method stubs two integer comparison results |
| Correct abstention | The safe output is to refuse a proven verdict | `SRC-VALIDATOR-CORE` join-on-empty read, not used | Apache-2.0 | `643617f2e4a17283350aa24c5999edfc0a51f124` | the expression literal exists | the assertion is an empty list, not a refusal | no, because there is no item | the method exists; this record did not run it | low | an empty collection is a result |
| Irrelevant mutations | A constructed change leaves the declared contract unchanged | `SRC-CONSTRUCTED-IRRELEVANT` | not a clinical record; generator absent | none | no | no | yes, when no external label exists | no generator | blocked | no fixture and no independent label |
| Adversarial mutations | A constructed change resembles a safe edit and violates the contract | `SRC-CONSTRUCTED-ADVERSARIAL` | not a clinical record; generator absent | none | no | no | yes, when no external label exists | no generator | blocked | no fixture and no independent label |
| Oracle disagreement | Two named oracles classify the same inputs differently | none pinned as a pair | not verified | none | no | no | yes, if the two publications do not exist | no | blocked | a second independent published result is not bound |
| FHIRPath | A path expression changes its accepted or rejected inputs | `SRC-VALIDATOR-CORE` join commit, not used | Apache-2.0 | `503ec1c025f6535ab7e3800d653f51bd513eb2d4` on parent `9c59ed2acd5f08aa2ff37a107abb9b28fcff06b4` | the expression literal exists | the new assertion is an empty list | no, because there is no item | the method exists; this record did not run it | low | the change is the result of an already accepted input |
| Protocol behavior | A request or response expectation changes | `SRC-BULK-DATA` | pointer plus a CC0 name | `1939654c9c11cfe9ec83f649ba457be4cd4510c6` | no | no | not reached | no | blocked | rights unresolved; SMART stays unresolved |
| Authorization behavior | A permission or consent expectation changes | `SRC-SMART` | no license file at the previously recorded root | the earlier observation, not a new pin | no | no | not reached | no | blocked | rights unresolved |
| FHIR package and IG evolution | A published package identity changes between pinned versions | `SRC-PACKAGE-HOST` | not verified for benchmark reuse | per package, not pinned here | no | no | not reached | one official package path exists in product code | blocked | two exact package identities are not bound |
| Profiles | A profile constraint changes between those versions | same package pair | same | none | no | no | not reached | no | blocked | the version pair is not bound |
| Extensions | An extension definition or binding changes | same package pair | same | none | no | no | not reached | no | blocked | the version pair is not bound |
| SearchParameters | A search parameter identity or semantics changes | same package pair | same | none | no | no | not reached | no | blocked | the version pair is not bound |
| Terminology | A code system, value set, or binding changes | a terminology artifact with its own grant | not verified | none | no | no | not reached | no | blocked | terminology bytes stay excluded |
| CQL and ELM | A clinical logic artifact changes its accepted or rejected inputs | none pinned | not verified | none | no | no | not reached | no | blocked | no public grant is pinned here |
| SQL-on-FHIR | A view definition changes its result membership | `SRC-SQL-ON-FHIR` | HL7 contribution grant, not a public license | the earlier observation | no | no | not reached | no | blocked | rights unresolved |
| Consumer dependencies | A downstream contract depends on an upstream element | none pinned | not verified | none | no | no | not reached | no | blocked | no pair is bound |
| Transformations | A declared transform changes an instance or artifact | none pinned | not verified | none | no | no | not reached | no | blocked | a FHIRPath evaluation is not recorded as this class |

`FHIR/fhir-test-cases` stays `NOT_ADMITTED_RIGHTS_UNRESOLVED`. This matrix does not reopen it.

No row assigns a split. No row freezes the protocol. No row authorizes an experiment or a product change.
