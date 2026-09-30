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
| SRC-SMART | public git candidate | NORMATIVE candidate | observed, not pinned | NO_LICENSE_FILE_AT_ROOT | not stored | declared protocol text | the pinned specification text | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-HL7-FHIR | public git candidate | official repository, not adopted | observed, not pinned | MULTI_LICENSE | not stored | mixed source and specification | none assigned | NOT_ADMITTED_LABEL_UNRESOLVED |
| SRC-HAPI-FHIR | public git candidate | independent implementation, not adopted | observed, not pinned | Apache-2.0 | fixture hashed, bytes not stored as a corpus member | validation fixture plus upstream assertion | upstream test assertion, not commandF | NOT_ADMITTED |
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

## SMART App Launch observation, not a pin

On 2026-09-30 the GitHub API returned `HL7/smart-app-launch` commit `3aea9cc057629a437e1fefda433660aed60abc80` on `master`. The repository license field was null. The root tree has no `LICENSE`, `LICENSE.md`, `LICENSE.txt`, `COPYING`, or `NOTICE`.

`README.md` blob `f9296c89712a00ab163694a524def8af82f50cbb` is 1082 bytes. It says: `The SMART App Launch Framework connects third-party applications to Electronic Health Record data, allowing apps to launch from inside or outside the user interface of an EHR system.` It names no license and no copyright holder.

`SRC-SMART` stays `NOT_ADMITTED_RIGHTS_UNRESOLVED`. The commit is not a pin. No specification bytes were stored.

## HL7 FHIR repository, not a corpus item

On 2026-10-01 the GitHub API returned `HL7/fhir` commit `68b299928d75bc6521624c03213069fd9206bdba` on `master`. The repository license field was `NOASSERTION`. The commit is not a corpus pin. No candidate bytes were stored.

`LICENSE` blob `a9a83cd7e26ade2456383bac3d02128b081db355` is 1971 bytes. SHA-256 `ba2245729ea077c4816d4440279fb014f936aaa3426728476fff27934b29027b`. It says: `This source is covered by multiple licenses and has multiple contributors.` It says some content is covered under other licenses, including Apache, EPL, and Creative Commons, as described in the files themselves. It says: `Note the the FHIR specification itself is covered under a different license, as software licenses are not appropriate.` The general license then permits redistribution of source and binary forms under three conditions: retain the copyright notice and conditions, reproduce them in binary distributions, and do not use the HL7 name to endorse derived products without prior written permission. That general license is not a grant for every file in the tree.

`source/license.html` blob `3334ec9800081af0c7edd3916f34d04010a4b048` is 9106 bytes. SHA-256 `62ee743fb60ee0c345f3c02a6f3c61d49e9f028ee4fdb967256f4265bb233912`. It says the specification, specifically the materials in the `fhir-spec.zip` file, is licensed under Creative Commons CC0. It also says acceptance of those terms grants no rights in third-party intellectual property, and it names SNOMED CT, DICOM, LOINC, ICD, and CPT as examples. Membership of any one git path in `fhir-spec.zip` was not proven. Specification content and terminology content stay out.

`tools/merge-audit.py` blob `0a28c34e81642850ff7ba4e7522dd843345e14cb` is 8271 bytes. SHA-256 `60a5beaa95925acf1f4efca0ff22e7c746cae49dc40ea090ff5cfbc122875b8f`. The file header describes a merge-audit script and does not state a different license. It is a repository tool, not the FHIR specification. The general redistribution license is the applicable text for this file. It is not a compatibility case, and no external label exists for it. The held-out split is not frozen. The file stays out of the corpus.

```text
SOURCE_ID = SRC-HL7-FHIR
REPOSITORY = HL7/fhir
EXACT_COMMIT = 68b299928d75bc6521624c03213069fd9206bdba
ARTIFACT_PATH = tools/merge-audit.py
ARTIFACT_BLOB = 0a28c34e81642850ff7ba4e7522dd843345e14cb
ARTIFACT_SHA256 = 60a5beaa95925acf1f4efca0ff22e7c746cae49dc40ea090ff5cfbc122875b8f
LICENSE_PATH = LICENSE
LICENSE_BLOB = a9a83cd7e26ade2456383bac3d02128b081db355
LICENSE_SHA256 = ba2245729ea077c4816d4440279fb014f936aaa3426728476fff27934b29027b
LICENSE_IDENTIFIER = multi-license repository; general text is a BSD-style redistribution grant; specification license is separate
LICENSE_QUOTE = This source is covered by multiple licenses
LICENSE_SCOPE_ANALYSIS = general grant covers this tool file; CC0 is stated for fhir-spec.zip materials; third-party terminologies are excluded; zip membership of other git paths was not proven
PROVENANCE = GitHub content API at the named commit
CASE_CLASS = none
LABEL_AUTHORITY = none
ADMISSION_STATE = NOT_ADMITTED_LABEL_UNRESOLVED
EXACT_BYTES_RETAINED = no
ITEM_COUNT = 0
```

## HAPI FHIR, rights proven, not admitted

On 2026-10-01 the GitHub API returned `hapifhir/hapi-fhir` commit `e307df6b64ff87c55af1607160f57141dbeb0360` on `master`. The repository license field was `Apache-2.0`. The commit is not a corpus pin. No library bytes were stored.

`LICENSE.txt` blob `261eeb9e9f8b2b4b0d119366dda99c6fd7d35c64` is 11357 bytes. SHA-256 `c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4`. It is the Apache License, Version 2.0, January 2004, and it begins: `TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION`.

`NOTICE.txt` blob `35f38a015ed625fddaa925176011609282cf1d77` is 563 bytes. SHA-256 `705d285dc21a348adf3429e6a76e3126f6953f2b4db203ada028714f1f329fae`. It says: `Copyright 2015, University Health Network` and `Licensed under the Apache License, Version 2.0`.

`README.md` blob `3d233bd612dd5de51a6f1402ec09bf00b33bd956` is 2234 bytes. SHA-256 `6ae6179c29dcb33e300ffb080a54d11efafc41b5099ec29e1c685f5b188b29f3`. Line 26 says: `This project is Open Source, licensed under the Apache Software License 2.0.`

`LOINC_NOTES.txt` is developer notes about LOINC import behavior. It is not a second license for the Java source. It is a reason not to treat LOINC-derived terminology bytes as covered by this observation. No terminology bytes were taken.

## One HAPI validation fixture, not a corpus member

The selected files are at the same commit `e307df6b64ff87c55af1607160f57141dbeb0360`.

`hapi-fhir-validation/src/test/resources/bug872-ext-with-hl7-url.json` blob `0aa5d2e6ebfab82103c650ca1e6a512034476067` is 256 bytes. SHA-256 `f74627045e309cb265bc37b015d4a81f12db70f6df9d7aa92b2003e6be9b5b31`. The file has no license header. It is a Patient resource whose narrative is `HELLO` and whose one extension uses `http://hl7.org/fhir/ValueSet/v3-ActInvoiceGroupCode` with `valueString` `test`. It contains no LOINC code, no SNOMED code, and no personal name, identifier, or clinical note. The JSON is not stored in this repository. The git blob and digest identify the bytes.

`hapi-fhir-validation/src/test/java/org/hl7/fhir/r4/validation/FhirInstanceValidatorR4Test.java` blob `c5c5689553436a9fde6d29fd5c428e44f107e15a` is 80918 bytes. SHA-256 `7fe6a7bae58d25e0f881caf530ecff967cfdcdb4415ccc15eef033a87699641b`. The first lines are the package declaration and imports. The examined file has no license header of its own. The repository `LICENSE.txt` and `NOTICE.txt` remain the applicable project grant. The Java file is not stored here.

The independent expectation is in that test, method `testExtensionUrlWithHl7Url`, lines 548-553. It loads `/bug872-ext-with-hl7-url.json`, validates it, and asserts `assertThat(nonInfo).isEmpty()` after dropping informational messages. The comment above the method says `See #872`. That assertion is the upstream expected result. commandF did not label it. The test was not executed in this record, so the assertion is a recorded expectation, not a reproduced run.

The planned case classes describe a change between versions, a constructed mutation, or a refusal. This artifact is one successful validation snapshot. It does not match those classes. The next section applies the corrected admission order.

```text
SOURCE_ID = SRC-HAPI-FHIR
REPOSITORY = hapifhir/hapi-fhir
EXACT_COMMIT = e307df6b64ff87c55af1607160f57141dbeb0360
ARTIFACT_PATH = hapi-fhir-validation/src/test/resources/bug872-ext-with-hl7-url.json
ARTIFACT_BLOB = 0aa5d2e6ebfab82103c650ca1e6a512034476067
ARTIFACT_SHA256 = f74627045e309cb265bc37b015d4a81f12db70f6df9d7aa92b2003e6be9b5b31
FILE_HEADER_LICENSE = absent
REPOSITORY_LICENSE = Apache-2.0
LICENSE_PATH = LICENSE.txt
LICENSE_BLOB = 261eeb9e9f8b2b4b0d119366dda99c6fd7d35c64
LICENSE_SHA256 = c71d239df91726fc519c6eb72d318ec65820627232b2f796219e87dcf35d0ab4
LICENSE_IDENTIFIER = Apache-2.0
LICENSE_QUOTE = This project is Open Source, licensed under the Apache Software License 2.0.
LICENSE_SCOPE_ANALYSIS = project Apache-2.0 grant; fixture and test file state no different license; LOINC terminology was not selected
PROVENANCE = GitHub content API at the named commit
EXPECTED_RESULT_SOURCE = FhirInstanceValidatorR4Test.java lines 548-553, method testExtensionUrlWithHl7Url
LABEL_AUTHORITY = upstream assertion that non-informational validation messages are empty
CASE_CLASS = none matched
REPLAY_PROCEDURE = execute testExtensionUrlWithHl7Url at the named commit against the named blob
NO_PHI = true for the examined fixture
ADMISSION_STATE = NOT_ADMITTED
EXACT_BYTES_RETAINED = no
ITEM_COUNT = 0
```

## HAPI fixture under the corrected lifecycle

`research/BENCHMARK_ADMISSION_LIFECYCLE.md` is the order used here. The digest above is unchanged. The upstream test was not re-downloaded.

Phase A is already recorded: `SRC-HAPI-FHIR` is a public source and is not a corpus member.

Phase B binds the commit, the fixture blob, the SHA-256, the Apache-2.0 project grant, GitHub provenance, `NO_PHI = true`, and the replay procedure `testExtensionUrlWithHl7Url`. The bytes are identified and are not copied into this repository. The case class is not bound, because no row in `research/BENCHMARK_PROTOCOL.md` describes one successful validation snapshot. Pairing the fixture with a second version was not done. No new class is added.

Phase C would accept the upstream assertion as an external published label, because it is quoted inside the pinned test file and commandF did not write it. Human adjudication is not required for that label. It is not used for corpus membership while the case class is missing.

Phase D does not run. The fixture stays out. `ITEM_COUNT` stays 0.

The fixture remains evidence of an independent validator expectation. It is not a scored CommandFBench item.

