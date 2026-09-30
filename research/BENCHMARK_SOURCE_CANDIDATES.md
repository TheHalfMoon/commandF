# Benchmark Source Candidates

Status: RESEARCH_PLANNING. No source is admitted.

These rows name public sources already listed as candidates in `docs/COMMAND_F_OPEN_SOURCE_QUALIFICATION_2026-09-12.md`, plus constructed-fixture classes that have no generator yet. A row is not a corpus item. No revision is pinned. No license text was re-read for this inventory. Rights stay `NOT_VERIFIED` until a later record quotes the license that applies to the exact revision.

| SOURCE_ID | SOURCE_TYPE | AUTHORITATIVE_CLASS | EXACT_VERSION_OR_REVISION | RIGHTS_LICENSE | ACQUISITION_METHOD | EXPECTED_ARTIFACT_TYPE | LABEL_AUTHORITY | STATUS |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SRC-IG-REGISTRY | public git candidate | OFFICIAL_REFERENCE, not adopted | NOT_PINNED | NOT_VERIFIED | not acquired | package-feed metadata | not a compatibility label | NOT_ADMITTED |
| SRC-FHIR-TEST-CASES | public git candidate | OFFICIAL_REFERENCE, not adopted | NOT_PINNED | NOT_VERIFIED | not acquired | published test artifacts | the published artifact, if pinned | NOT_ADMITTED |
| SRC-FHIRPATH-SPEC | public git candidate | NORMATIVE candidate | NOT_PINNED | NOT_VERIFIED | not acquired | specification text | the pinned specification text | NOT_ADMITTED |
| SRC-SQL-ON-FHIR | public git candidate | NORMATIVE candidate | NOT_PINNED | NOT_VERIFIED | not acquired | view-definition specification | the pinned specification text | NOT_ADMITTED |
| SRC-FSH-SPEC | public git candidate | NORMATIVE candidate | NOT_PINNED | NOT_VERIFIED | not acquired | shorthand specification | the pinned specification text | NOT_ADMITTED |
| SRC-SMART | public git candidate | NORMATIVE candidate | NOT_PINNED | NOT_VERIFIED | not acquired | declared protocol text | the pinned specification text | NOT_ADMITTED |
| SRC-BULK-DATA | public git candidate | NORMATIVE candidate | NOT_PINNED | NOT_VERIFIED | not acquired | declared protocol text | the pinned specification text | NOT_ADMITTED |
| SRC-VALIDATOR-CORE | public git candidate | process-oracle candidate | NOT_PINNED | NOT_VERIFIED | not acquired | pinned validator behavior | the pinned tool, not commandF | NOT_ADMITTED |
| SRC-FIRELY-SDK | public git candidate | independent implementation candidate | NOT_PINNED | NOT_VERIFIED | not acquired | pinned independent behavior | the pinned tool, not commandF | NOT_ADMITTED |
| SRC-PACKAGE-HOST | already authorized product host | packages.fhir.org or packages2.fhir.org | per package, not a corpus member | NOT_VERIFIED for benchmark reuse | one exact package only, already specified for product acquisition | package archive bytes | not commandF | NOT_ADMITTED |
| SRC-CONSTRUCTED-ADVERSARIAL | fixture not written | none | none | not a clinical record | not acquired | constructed bytes | a recorded adjudication | NOT_ADMITTED |
| SRC-CONSTRUCTED-IRRELEVANT | fixture not written | none | none | not a clinical record | not acquired | constructed bytes | a recorded adjudication | NOT_ADMITTED |
| SRC-CONSTRUCTED-MALFORMED | fixture not written | none | none | not a clinical record | not acquired | constructed bytes | a recorded adjudication | NOT_ADMITTED |

`ITEM_COUNT` stays 0. Admission still requires the gate in `research/CANDIDATE_CORPUS.md`. This inventory does not download bytes and does not authorize a registry crawl.

## One observed revision, not a pin

On 2026-09-30 the GitHub content API returned `FHIR/ig-registry` `package-feeds.json` at commit `5f9e60bf5c15b091d90f8c914c7f0c0a47768685`. The git blob was `8521f455517e91e2782183d5c6143d272dbadae1`. The decoded body was 20016 bytes. The SHA-256 of those bytes was `787b8440f72bc71787cf798906991d0361683024f90b84bf4dc0404da99626ee`. The repository license field was null. No license text was stored.

Those facts are an observation of one response. They are not a pin, not an admitted source, and not a CF-17 catalog authorization. The default branch can move. The bytes are not in this repository. `SRC-IG-REGISTRY` stays `NOT_ADMITTED`.
