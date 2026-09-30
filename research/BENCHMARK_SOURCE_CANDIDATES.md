# Benchmark Source Candidates

Status: RESEARCH_PLANNING. No source is admitted.

These rows name public sources already listed as candidates in `docs/COMMAND_F_OPEN_SOURCE_QUALIFICATION_2026-09-12.md`, plus constructed-fixture classes that have no generator yet. A row is not a corpus item. No revision is pinned. A row stays `NOT_VERIFIED` until a later section records the exact revision that was read.

| SOURCE_ID | SOURCE_TYPE | AUTHORITATIVE_CLASS | EXACT_VERSION_OR_REVISION | RIGHTS_LICENSE | ACQUISITION_METHOD | EXPECTED_ARTIFACT_TYPE | LABEL_AUTHORITY | STATUS |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SRC-IG-REGISTRY | public git candidate | OFFICIAL_REFERENCE, not adopted | observed, not pinned | CONFLICTING_STATEMENTS | not stored | package-feed metadata | not a compatibility label | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-FHIR-TEST-CASES | public git candidate | OFFICIAL_REFERENCE, not adopted | NOT_PINNED | NOT_VERIFIED | not acquired | published test artifacts | the published artifact, if pinned | NOT_ADMITTED |
| SRC-FHIRPATH-SPEC | public git candidate | NORMATIVE candidate | observed, not pinned | NO_LICENSE_FILE_AT_ROOT | not stored | specification text | the pinned specification text | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-SQL-ON-FHIR | public git candidate | NORMATIVE candidate | observed, not pinned | HL7_CONTRIBUTION_GRANT_NOT_A_PUBLIC_LICENSE | not stored | view-definition specification | the pinned specification text | NOT_ADMITTED_RIGHTS_UNRESOLVED |
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

On 2026-09-30 the GitHub content API returned `FHIR/ig-registry` `package-feeds.json` at commit `5f9e60bf5c15b091d90f8c914c7f0c0a47768685`. The git blob was `8521f455517e91e2782183d5c6143d272dbadae1`. The decoded body was 20016 bytes. The SHA-256 of those bytes was `787b8440f72bc71787cf798906991d0361683024f90b84bf4dc0404da99626ee`. The repository license field was null. The bytes are not in this repository.

The same commit's root tree has no `LICENSE`, `LICENSE.md`, `LICENSE.txt`, `COPYING`, or `NOTICE`. `README.md` line 27 says: `* License: Content is licensed under Creative Commons Public Domain`. `package.json` says `"license": "MIT"`. Those two statements are not the same grant. Neither file is a standalone license text, and the bytes of `package-feeds.json` contain no rights statement that was recorded here.

`SRC-IG-REGISTRY` is `NOT_ADMITTED_RIGHTS_UNRESOLVED`. The observation is not a pin and not a CF-17 catalog authorization. The default branch can move.

## FHIRPath observation, not a pin

On 2026-09-30 the GitHub API returned `HL7/fhirpath` commit `c95ad83b35babc67a383369c96535c39e9487fd3` on `master`. The repository license field was null. The root tree has no `LICENSE`, `LICENSE.md`, `LICENSE.txt`, `COPYING`, or `NOTICE`.

`README.md` blob `a1e9c0be26422203a6ccd1682d2fa8aa86af0080` is 728 bytes. Its opening paragraph is: `FHIRPath Specification - this is the source for the HL7 FHIRPath specification, as published at http://hl7.org/fhirpath`. The file names no license and no copyright holder. `input/includes` contains only `menu.xml`.

`SRC-FHIRPATH-SPEC` stays `NOT_ADMITTED_RIGHTS_UNRESOLVED`. The commit is not a pin. No specification bytes were stored.

## SQL on FHIR observation, not a pin

On 2026-09-30 the GitHub API returned `HL7/sql-on-fhir` commit `e3e1d3c7efa6541af4172a01d373739dd5a366bc` on `main`. The repository license field was `NOASSERTION`. There is no `package.json` license field.

`LICENSE.md` blob `3c3c3eb104801e4ef6f671525450702161d09a26` is 8543 bytes. It says the specification was contributed to HL7 under section 09.01.02 of the HL7 Governance and Operations Manual, and that each contributor grants HL7 a free, irrevocable licence to "permit others, at HL7's sole discretion, to reproduce the resulting Protocol Specifications in whole or in part." That sentence gives the reproduction choice to HL7. It is not a public license for CommandF to copy the specification into a benchmark.

`SRC-SQL-ON-FHIR` stays `NOT_ADMITTED_RIGHTS_UNRESOLVED`. The commit is not a pin. No specification bytes were stored. `ITEM_COUNT` stays 0.

