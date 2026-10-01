# Benchmark Source Candidates

Status: RESEARCH_PLANNING. Membership is recorded in `research/CANDIDATE_CORPUS.md`.

These rows name public sources already listed as candidates in `docs/COMMAND_F_OPEN_SOURCE_QUALIFICATION_2026-09-12.md`, plus constructed-fixture classes that have no generator yet. A row is not a corpus item by itself. A row stays `NOT_VERIFIED` until a later section records the exact revision that was read. Revisions already recorded below stay pinned to those sections.

| SOURCE_ID | SOURCE_TYPE | AUTHORITATIVE_CLASS | EXACT_VERSION_OR_REVISION | RIGHTS_LICENSE | ACQUISITION_METHOD | EXPECTED_ARTIFACT_TYPE | LABEL_AUTHORITY | STATUS |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SRC-IG-REGISTRY | public git candidate | OFFICIAL_REFERENCE, not adopted | observed, not pinned | CONFLICTING_STATEMENTS | not stored | package-feed metadata | not a compatibility label | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-FHIR-TEST-CASES | public git candidate | OFFICIAL_REFERENCE, not adopted | 0dd8336f3c584f7b9491b74be23721d1edf58eaa | CONFLICTING_STATEMENTS | not stored | published test artifacts | not assigned | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-FHIRPATH-SPEC | public git candidate | NORMATIVE candidate | observed, not pinned | NO_LICENSE_FILE_AT_ROOT | not stored | specification text | the pinned specification text | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-SQL-ON-FHIR | public git candidate | NORMATIVE candidate | observed, not pinned | HL7_CONTRIBUTION_GRANT_NOT_A_PUBLIC_LICENSE | not stored | view-definition specification | the pinned specification text | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-FSH-SPEC | public git candidate | NORMATIVE candidate | NOT_PINNED | NOT_VERIFIED | not acquired | shorthand specification | the pinned specification text | NOT_ADMITTED |
| SRC-SMART | public git candidate | NORMATIVE candidate | observed, not pinned | NO_LICENSE_FILE_AT_ROOT | not stored | declared protocol text | the pinned specification text | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-HL7-FHIR | public git candidate | official repository, not adopted | observed, not pinned | MULTI_LICENSE | not stored | mixed source and specification | none assigned | NOT_ADMITTED_LABEL_UNRESOLVED |
| SRC-HAPI-FHIR | public git candidate | independent implementation, not adopted | observed, not pinned | Apache-2.0 | fixture hashed, bytes not stored as a corpus member | validation fixture plus upstream assertion | upstream test assertion, not commandF | NOT_ADMITTED |
| SRC-HAPI-JSON-PARSE | public git candidate | independent implementation, not adopted | e307df6b64ff87c55af1607160f57141dbeb0360 | Apache-2.0 | input quoted; test file not copied | malformed JSON literal | upstream test assertion, not commandF | CANDIDATE_CORPUS |
| SRC-HAPI-UNSUPPORTED-IN | public git candidate | independent implementation, not adopted | e307df6b64ff87c55af1607160f57141dbeb0360 | Apache-2.0 | input quoted; test file not copied | in-memory search qualifier | upstream test assertion, not commandF | CANDIDATE_CORPUS |
| SRC-HAPI-PARTIAL-RESOURCE-TYPE | public git candidate | independent implementation, not adopted | e307df6b64ff87c55af1607160f57141dbeb0360 | Apache-2.0 | input quoted; test file not copied | FHIR JSON object missing resourceType | upstream test assertion, not commandF | CANDIDATE_CORPUS |
| SRC-BULK-DATA | public git candidate | NORMATIVE candidate | 1939654c9c11cfe9ec83f649ba457be4cd4510c6 | FHIR_LICENSE_POINTER_AND_CC0_NAME | not stored | declared protocol text | not assigned | NOT_ADMITTED_RIGHTS_UNRESOLVED |
| SRC-VALIDATOR-CORE | public git candidate | process-oracle candidate | 643617f2e4a17283350aa24c5999edfc0a51f124 | Apache-2.0 | not stored | pinned validator behavior | not assigned | NOT_ADMITTED |
| SRC-MICROSOFT-FHIR | public git candidate | independent implementation candidate | e164d39f1eff719bf0fd1f8ece559f52c93c1c9b | MIT | not stored | search-parameter validator test | upstream test, class not matched | NOT_ADMITTED |
| SRC-FIRELY-SDK | public git candidate | independent implementation candidate | NOT_PINNED | NOT_VERIFIED | not acquired | pinned independent behavior | the pinned tool, not commandF | NOT_ADMITTED |
| SRC-PACKAGE-HOST | already authorized product host | packages.fhir.org or packages2.fhir.org | per package, not a corpus member | NOT_VERIFIED for benchmark reuse | one exact package only, already specified for product acquisition | package archive bytes | not commandF | NOT_ADMITTED |
| SRC-CONSTRUCTED-ADVERSARIAL | fixture not written | none | none | not a clinical record | not acquired | constructed bytes | a recorded adjudication | NOT_ADMITTED |
| SRC-CONSTRUCTED-IRRELEVANT | fixture not written | none | none | not a clinical record | not acquired | constructed bytes | a recorded adjudication | NOT_ADMITTED |
| SRC-CONSTRUCTED-MALFORMED | fixture not written | none | none | not a clinical record | not acquired | constructed bytes | a recorded adjudication | NOT_ADMITTED |

The candidate count is recorded in `research/CANDIDATE_CORPUS.md`. This inventory does not copy the upstream test file and does not authorize a registry crawl.

## FHIR test cases, rights unresolved

On 2026-10-01 the GitHub API returned `FHIR/fhir-test-cases` commit `0dd8336f3c584f7b9491b74be23721d1edf58eaa`. The repository license field was `Apache-2.0`. `LICENSE.txt` at that commit is blob `261eeb9e9f8b2b4b0d119366dda99c6fd7d35c64`, 11357 bytes, the same standard Apache-2.0 text already recorded for HAPI. `README.md` at that commit is blob `6cbfce9119d72bb0504b0856f0646f9f21174136`, 6555 bytes, SHA-256 `0472e87131218991b1c4c29d813eae0bb75a360187c4128d23f3b338979ee37d`. It says: `* License: The contents in here are covered by Creative Commons Public Domain`.

Those two grants are not the same. The root tree also contains `snomed` and `ucum`. No example from this repository is stored. No case class is assigned. `ITEM_COUNT` is unchanged.

```text
SOURCE_ID = SRC-FHIR-TEST-CASES
REPOSITORY = FHIR/fhir-test-cases
SOURCE_COMMIT = 0dd8336f3c584f7b9491b74be23721d1edf58eaa
RIGHTS_STATE = NOT_ADMITTED_RIGHTS_UNRESOLVED
RIGHTS_AUTHORITY = LICENSE.txt Apache-2.0 AND README Creative Commons Public Domain
LICENSE_PATH = LICENSE.txt
LICENSE_BLOB = 261eeb9e9f8b2b4b0d119366dda99c6fd7d35c64
STATE = NOT_ADMITTED_RIGHTS_UNRESOLVED
```

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

## Malformed JSON parse, candidate member

`Malformed evidence` asks for input bytes that do not parse as the declared artifact. In `JacksonStructureTest.rejectsMalformedJsonContent` at the same HAPI commit, the input is `{"resourceType":"Patient"} trailing`. The method asserts a `DataFormatException` whose message contains `Failed to parse JSON encoded FHIR content`.

That is a parse rejection, not a successful snapshot. Rights are the Apache-2.0 project grant already recorded. The file states no different license. The input is not a clinical record. The item is member 1 in `research/CANDIDATE_CORPUS.md`. It is not a protocol freeze.

```text
SOURCE_ID = SRC-HAPI-JSON-PARSE
CASE_CLASS = Malformed evidence
STATE = CANDIDATE_CORPUS
ITEM_COUNT = 1
```

## Array root, not malformed evidence

The same pinned file, blob `ec074d00c7a756e201f89006f81d42e7d7954636`, SHA-256 `ec3d8c0947d473c326a926f28c9acb853e9675bc33976ac4b124e2430ffba201`, contains `rejectsArrayRootWhenNotAllowed`. The input text is `[1, 2, 3]`. That text is 9 UTF-8 bytes. SHA-256 `a36b1f2c3f84522dd1005145646617d7054c0851e97c72a039c0bdfac9fa07f3`. The method asserts `DataFormatException` with a message containing `must be '{'`. The test was not executed here.

`[1, 2, 3]` is well-formed JSON. The rejection is that this parser, when an array root is not allowed, requires an object. That is a root-shape constraint. `Malformed evidence` is for bytes that do not parse as the declared artifact. The already admitted item fails because a token follows the object and the message is `Failed to parse JSON encoded FHIR content`. This input does not fail that way. It is not added to that class. It is not assigned to another class. No further method in this file is being taken as a near-duplicate.

The Apache-2.0 project grant still covers the file. The input is not a clinical record. Human adjudication is not required, because there is no membership decision that needs a new label. `ITEM_COUNT` stays 1.

```text
SOURCE_ID = SRC-HAPI-ARRAY-ROOT
CASE_CLASS = none matched
INPUT_TEXT = [1, 2, 3]
INPUT_BYTE_COUNT = 9
INPUT_SHA256 = a36b1f2c3f84522dd1005145646617d7054c0851e97c72a039c0bdfac9fa07f3
LABEL_AUTHORITY = upstream method rejectsArrayRootWhenNotAllowed
LABEL_STATEMENT = DataFormatException containing must be '{'
UPSTREAM_TEST_REPRODUCED = no
STATE = NOT_ADMITTED
```

## Unsupported search qualifier, candidate member

`Unsupported evidence` asks for a declared evidence class outside the evaluator contract. `testUnsupportedIn` in `InMemoryResourceMatcherConfigurationR5Test.java` at the same HAPI commit calls the in-memory matcher with `code:in=http://hl7.org/some-vs`. The method asserts that the result is unsupported and that the reason is `Parameter: <code:in> Reason: Qualified parameter not supported`.

That is a contract refusal, not a JSON parse failure, and not the array-root shape rejection already kept out of `Malformed evidence`. `:in` is `TokenParamModifier.IN` at this commit. The value-set URI is a constant in the test. The same method passes a synthetic Observation. Those fields are not imported terminology. The item is member 2 in `research/CANDIDATE_CORPUS.md`. It is not a protocol freeze.

`testUnsupportedNotIn` uses `:not-in` and the same reason shape. It is not a second member.

```text
SOURCE_ID = SRC-HAPI-UNSUPPORTED-IN
CASE_CLASS = Unsupported evidence
STATE = CANDIDATE_CORPUS
ITEM_COUNT = 2
UPSTREAM_TEST_REPRODUCED = no
```

## Missing resourceType, candidate member

`Partial evidence` asks for a required evidence class that is absent. `testReadWithUnparseableResponse` in `GenericClientR4Test.java` at the same HAPI commit sets the response body to `{"resourceTypeeeee":"Patient"}`. That text is 30 UTF-8 bytes. SHA-256 `b68aeeeccfe2b45dd5dde91f331e0dfa53fc7ca112e1de2209d8a4bcfc4e3255`. The method asserts `FhirClientConnectionException` and a message that contains `missing required element: 'resourceType'`.

The bytes are one JSON object. The required FHIR element is absent because the key is misspelled. That is not the trailing-token parse failure and not the array-root rejection. The method name says unparseable. The class follows the assertion, which names a missing required element. The item is member 3 in `research/CANDIDATE_CORPUS.md`. The test was not executed here.

`testEncodeWithInvalidExtensionMissingUrl` in `JsonParserR4Test.java` at the same commit also reports a missing required `url`. Its input is a Java object graph, and the test does not bind the encoded bytes. It is not a corpus member.

```text
SOURCE_ID = SRC-HAPI-PARTIAL-RESOURCE-TYPE
CASE_CLASS = Partial evidence
INPUT_TEXT = {"resourceTypeeeee":"Patient"}
INPUT_BYTE_COUNT = 30
INPUT_SHA256 = b68aeeeccfe2b45dd5dde91f331e0dfa53fc7ca112e1de2209d8a4bcfc4e3255
STATE = CANDIDATE_CORPUS
ITEM_COUNT = 3
UPSTREAM_TEST_REPRODUCED = no
```

## Validator core, rights recorded, no item

On 2026-10-01 the GitHub API returned `hapifhir/org.hl7.fhir.core` commit `643617f2e4a17283350aa24c5999edfc0a51f124`. `LICENSE.txt` is blob `261eeb9e9f8b2b4b0d119366dda99c6fd7d35c64`, 11357 bytes, the same Apache-2.0 text already hashed for HAPI. `license/README.md` is blob `aa1944857a0a1e2b9cbac1195dcfbc9fc2199b6e`, 4133 bytes, SHA-256 `984a008e03fce6b0412b13f3fe4f49e4adfea338005cb2c282a4f03ac4d698ef`. It says this project uses an Apache 2.0 license. The root README does not name a second grant. No test bytes are stored.

`FHIRPathTests.testEvaluate_JoinOnEmptyInput` evaluates `Patient.name.given.join(',')` on an empty Patient and expects an empty list. That is one successful evaluation. `FHIRPath` in the protocol asks for an expression that changes its accepted or rejected inputs. This method does not show that change. An empty result is also not a refused verdict, so it is not `Correct abstention`.

```text
SOURCE_ID = SRC-VALIDATOR-CORE
REPOSITORY = hapifhir/org.hl7.fhir.core
SOURCE_COMMIT = 643617f2e4a17283350aa24c5999edfc0a51f124
RIGHTS_STATE = Apache-2.0 project grant
STATE = NOT_ADMITTED
```

## Microsoft FHIR server, conflict test does not match

On 2026-10-01 the GitHub API returned `microsoft/fhir-server` commit `e164d39f1eff719bf0fd1f8ece559f52c93c1c9b`. `LICENSE` is blob `21071075c24599ee98254f702bcfc504cdc275a6`, 1162 bytes, SHA-256 `27ebda9d51f0a56b7e281ccd8230a27236dcb51c05f64b07869ecf6e965d68b0`. The file contains `MIT License` and `Copyright (c) Microsoft Corporation`. The README does not name a different grant. The GitHub license field was `MIT`.

`SearchParameterValidatorTests.GivenSearchParameter_WhenValidatingProperties_ThenConflictingPropertiesShouldBeReported` builds one `SearchParameter` and stubs `CompareExpression` and `CompareComponent` with integers. The assertion depends on `int.MinValue`. It does not supply two evidence classes and it does not quote a disagreement between them. It is not `Conflicting evidence`. No bytes from this test are stored.

```text
SOURCE_ID = SRC-MICROSOFT-FHIR
REPOSITORY = microsoft/fhir-server
SOURCE_COMMIT = e164d39f1eff719bf0fd1f8ece559f52c93c1c9b
CASE_CLASS = none matched
RIGHTS_STATE = MIT
STATE = NOT_ADMITTED
```

## Bulk Data license pointer

On 2026-10-01 the GitHub API returned `HL7/bulk-data` commit `1939654c9c11cfe9ec83f649ba457be4cd4510c6`. The repository license field was `NOASSERTION`. `LICENSE` is blob `d8ffe253d1e0d83b3d5531df2c4f02ae5ca378a3`, 167 bytes, SHA-256 `4bbeba0e466d6f2a941fe67bd33dfc551d2434f6c7f7a4d57c9f20d2580a60ea`. The whole file names the HL7 FHIR license at `http://hl7.org/fhir/license.html` and also names Creative Commons CC0. It is not a standalone license text. The FHIR repository license record is already `MULTI_LICENSE`. No specification example is stored.

```text
SOURCE_ID = SRC-BULK-DATA
REPOSITORY = HL7/bulk-data
SOURCE_COMMIT = 1939654c9c11cfe9ec83f649ba457be4cd4510c6
RIGHTS_STATE = NOT_ADMITTED_RIGHTS_UNRESOLVED
STATE = NOT_ADMITTED_RIGHTS_UNRESOLVED
```


