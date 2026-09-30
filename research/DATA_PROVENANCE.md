# Data Provenance

Status: RESEARCH_PLANNING.

Each future benchmark item should record source, revision, version, acquisition date, digest, rights, transformations, split, label provenance, adjudication, disagreement, and uncertainty where those facts exist.

Public interoperability artifacts and synthetic adversarial changes are preferred. Private clinical data is not authorized for a public benchmark. Current corpus assignment is `RESULT_PENDING`.

## Provenance inventory

This inventory names the provenance fields for each planned case class. It does not assign an item, a count, a digest, a label, a split, or a rights disposition. A later candidate corpus has to fill those fields from retrieved bytes or from an explicit adjudication record. Nothing in this table was downloaded for the inventory.

| Class | Candidate source class | Bytes | Rights | Label authority | Split |
| --- | --- | --- | --- | --- | --- |
| FHIR package and IG evolution | published package bytes from an already authorized official host | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Profiles | the same pinned package bytes | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Extensions | the same pinned package bytes | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| SearchParameters | the same pinned package bytes | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Terminology | the same pinned package bytes, or a separately pinned terminology artifact | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| FHIRPath | a published expression artifact, or a synthetic expression with a recorded generator | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| CQL and ELM | a published clinical logic artifact, or a synthetic artifact with a recorded generator | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| SQL-on-FHIR | a published view definition, or a synthetic view with a recorded generator | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Protocol behavior | a published declared contract, not an observed private server | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Authorization behavior | a published declared contract, not an observed private server | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Consumer dependencies | a published downstream contract that names an upstream element | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Oracle disagreement | two named pinned tools on the same public or synthetic inputs | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Transformations | a published transform, or a synthetic transform with a recorded generator | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |
| Adversarial mutations | a constructed fixture; not yet written | `RESULT_PENDING` | not a clinical record | `RESULT_PENDING` | `RESULT_PENDING` |
| Irrelevant mutations | a constructed fixture; not yet written | `RESULT_PENDING` | not a clinical record | `RESULT_PENDING` | `RESULT_PENDING` |
| Malformed evidence | a constructed fixture; not yet written | `RESULT_PENDING` | not a clinical record | `RESULT_PENDING` | `RESULT_PENDING` |
| Partial evidence | a constructed fixture; not yet written | `RESULT_PENDING` | not a clinical record | `RESULT_PENDING` | `RESULT_PENDING` |
| Unsupported evidence | a constructed fixture; not yet written | `RESULT_PENDING` | not a clinical record | `RESULT_PENDING` | `RESULT_PENDING` |
| Conflicting evidence | a constructed fixture; not yet written | `RESULT_PENDING` | not a clinical record | `RESULT_PENDING` | `RESULT_PENDING` |
| Correct abstention | a constructed fixture, or a public case whose safe output is refusal | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` | `RESULT_PENDING` |

No row is a corpus member. No row authorizes a registry download, a product change, or an experiment.
