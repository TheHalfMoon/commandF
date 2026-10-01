# Candidate Corpus

Status: RESEARCH_PLANNING. Three candidate members. Not frozen.

This document is the admission gate for CommandFBench items. Membership here is not a protocol freeze and not an experiment.

## Admission rule

The order is `research/BENCHMARK_ADMISSION_LIFECYCLE.md`. A pre-admission digest is not membership.

An item may be added only after a later record supplies all of the following:

- a case class already named in `research/BENCHMARK_PROTOCOL.md`;
- exact bytes, or an exact fixture generator and its inputs;
- SHA-256 of those bytes;
- source identity and rights;
- a label whose authority is not commandF, under `research/LABEL_AUTHORITY.md`;
- either an external published label that already satisfies that rule, or a completed human adjudication of the pre-admission digest, including a retained `DISAGREEMENT`;
- a statement that the bytes are not private clinical data;
- a replay procedure.

A split assignment is not required to enter this corpus. It is computed later from a candidate digest under `research/SPLIT_POLICY.md`, and the held-out manifest is bound at the protocol freeze. The current three members share one source repository, so version `sp-1` does not assign them. A missing field above keeps the item out. CommandF output cannot fill the label. A registry response is not a corpus item until its bytes are the bytes that were hashed.

## Current manifest

```text
ITEM_COUNT = 3
CORPUS_DIGEST = e92d734ab2981691322074f9d963bcf6ffc9058dace3875fdea64d2558b417aa
LABELS = 3 external published labels
LABELS_MINTED_BY_COMMANDF = 0
HELD_OUT_SPLIT = NOT_FROZEN
SPLIT_POLICY = SPECIFIED
SPLIT_POLICY_VERSION = sp-1
SPLIT_ASSIGNMENT = NOT_ASSIGNED
RIGHTS = Apache-2.0 for all three members
```

The digest of one item is the SHA-256 of the UTF-8 canonical JSON in that item record, with keys sorted and no insignificant whitespace. The corpus digest is the SHA-256 of those item lines in item order, joined by a single newline and with no trailing newline.

## Item 1

Case class: Malformed evidence. The input is not a FHIR JSON document because a token follows the object. The upstream test expects a parse failure.

```text
{"case_class":"Malformed evidence","input_sha256":"c1d536a36fdd1c3c75fb4aa13d51e1ca52039987d32c249e54cbd34bcc75fc6b","input_text":"{\"resourceType\":\"Patient\"} trailing","label_method":"rejectsMalformedJsonContent","label_statement":"DataFormatException containing Failed to parse JSON encoded FHIR content","source_blob":"ec074d00c7a756e201f89006f81d42e7d7954636","source_commit":"e307df6b64ff87c55af1607160f57141dbeb0360","source_path":"hapi-fhir-base/src/test/java/ca/uhn/fhir/parser/json/jackson/JacksonStructureTest.java","source_repository":"hapifhir/hapi-fhir","source_sha256":"ec3d8c0947d473c326a926f28c9acb853e9675bc33976ac4b124e2430ffba201"}
```

The input is 35 UTF-8 bytes. SHA-256 `c1d536a36fdd1c3c75fb4aa13d51e1ca52039987d32c249e54cbd34bcc75fc6b`. It names a resource type and no person, identifier, or clinical note.

`JacksonStructureTest.java` at `hapifhir/hapi-fhir` commit `e307df6b64ff87c55af1607160f57141dbeb0360` is blob `ec074d00c7a756e201f89006f81d42e7d7954636`, 4362 bytes, SHA-256 `ec3d8c0947d473c326a926f28c9acb853e9675bc33976ac4b124e2430ffba201`. The file starts with `package ca.uhn.fhir.parser.json.jackson;` and states no license of its own. The repository grant is Apache-2.0 in `LICENSE.txt`, as recorded for `SRC-HAPI-FHIR`. The Java file is not copied here.

Method `rejectsMalformedJsonContent` loads that exact text and asserts `DataFormatException` with a message containing `Failed to parse JSON encoded FHIR content`. That statement is the external label. Human adjudication is not required. The test was not executed in this repository. Replay is to run that method at the named commit.

This item does not assign a split. It does not freeze the protocol. It does not authorize an experiment or a product change.

## Item 2

Case class: Unsupported evidence. The input is a token search with the `:in` qualifier. The upstream matcher test expects that qualifier to be outside the in-memory matcher contract.

```text
{"case_class":"Unsupported evidence","input_sha256":"912692bee010a2710b77fa07a8bfa0a0d04ce9c4f1cfef4d7de346e34a7f04f0","input_text":"code:in=http://hl7.org/some-vs","label_method":"testUnsupportedIn","label_statement":"unsupported reason Parameter: <code:in> Reason: Qualified parameter not supported","source_blob":"e2f2e5fb97af0d12c96fb77fd4844beadef6de6e","source_commit":"e307df6b64ff87c55af1607160f57141dbeb0360","source_path":"hapi-fhir-jpaserver-searchparam/src/test/java/ca/uhn/fhir/jpa/searchparam/matcher/InMemoryResourceMatcherConfigurationR5Test.java","source_repository":"hapifhir/hapi-fhir","source_sha256":"b866cca5c7f1bdbc10aba915b59a2f947774394590e6882c7affa3effc1b984d"}
```

The input is 30 UTF-8 bytes. SHA-256 `912692bee010a2710b77fa07a8bfa0a0d04ce9c4f1cfef4d7de346e34a7f04f0`. The string is the first argument of `testUnsupportedIn`. At the same commit, `TokenParamModifier.IN` is `":in"` in `TokenParamModifier.java`, blob `8604be7d2df136df66abab2e94066fcde81d128b`, 2033 bytes, SHA-256 `012a5d7bbe92f24952d6db94b5a4145a3bc02dd6912857f14a4324ee2192880d`. The value-set constant in the test file is `http://hl7.org/some-vs`. The method also passes a constructed Observation whose code is `MATCH`, system `http://hl7.org/some-cs`, display `Match`, plus token params extracted from that object. Those values are not a person and not an imported code system. They are not part of the hashed input line. The hashed line is the match URL the unsupported reason names.

The test file is blob `e2f2e5fb97af0d12c96fb77fd4844beadef6de6e`, 6345 bytes, SHA-256 `b866cca5c7f1bdbc10aba915b59a2f947774394590e6882c7affa3effc1b984d`. It starts with `package ca.uhn.fhir.jpa.searchparam.matcher;` and states no license of its own. The repository grant is the same Apache-2.0 `LICENSE.txt` already recorded for `SRC-HAPI-FHIR`. The Java file is not copied here.

`testUnsupportedIn` asserts `supported()` is false and the unsupported reason is exactly `Parameter: <code:in> Reason: Qualified parameter not supported`. That assertion is the external label. Human adjudication is not required. The test was not executed in this repository. Replay is to run that method at the named commit. The method is ordered after another test in the same class. This record does not claim that order was reproduced.

`testUnsupportedNotIn` is the same shape with `:not-in`. It is not a second item.

This item does not assign a split. It does not freeze the protocol. It does not authorize an experiment or a product change.

## Item 3

Case class: Partial evidence. The input is well-formed JSON. The key is `resourceTypeeeee`, so the required FHIR element `resourceType` is absent. The upstream client test expects that absence.

```text
{"case_class":"Partial evidence","input_sha256":"b68aeeeccfe2b45dd5dde91f331e0dfa53fc7ca112e1de2209d8a4bcfc4e3255","input_text":"{\"resourceTypeeeee\":\"Patient\"}","label_method":"testReadWithUnparseableResponse","label_statement":"FhirClientConnectionException containing missing required element: 'resourceType'","source_blob":"1bd9b98c060c43b5afa07ad557016f662eafc1ad","source_commit":"e307df6b64ff87c55af1607160f57141dbeb0360","source_path":"hapi-fhir-structures-r4/src/test/java/ca/uhn/fhir/rest/client/GenericClientR4Test.java","source_repository":"hapifhir/hapi-fhir","source_sha256":"94b987a0a8dd9dfb3948cf40260080c8bbe590ebc02413dd7ed9122b95e491fd"}
```

The input is 30 UTF-8 bytes. SHA-256 `b68aeeeccfe2b45dd5dde91f331e0dfa53fc7ca112e1de2209d8a4bcfc4e3255`. It names a resource type only as a misspelled key and a string value. It names no person, identifier, or clinical note.

`GenericClientR4Test.java` at the same HAPI commit is blob `1bd9b98c060c43b5afa07ad557016f662eafc1ad`, 185964 bytes, SHA-256 `94b987a0a8dd9dfb3948cf40260080c8bbe590ebc02413dd7ed9122b95e491fd`. The file starts with `package ca.uhn.fhir.rest.client;` and states no license of its own. The repository grant is the same Apache-2.0 `LICENSE.txt` already recorded for `SRC-HAPI-FHIR`. The Java file is not copied here.

`testReadWithUnparseableResponse` supplies that exact string as the HTTP response body with content type `Constants.CT_FHIR_JSON`. It asserts `FhirClientConnectionException` and `assertEquals` against a message built with `Msg.code(1359)` and `Msg.code(1838)`. The string literals in that assertion include `Failed to parse response from server when performing GET to URL http://example.com/fhir/Patient/123?_elements=identifier%2Cname` and `Invalid JSON content detected, missing required element: 'resourceType'`. The hashed label keeps the required-element fragment. The test was not executed here, so this record does not store an expanded runtime message. Replay is to run that method at the named commit.

The method name says unparseable. The bytes are still one JSON object. The published failure is a missing required element, not a trailing token and not an array root. `Malformed evidence` stays the trailing-token item. This item does not assign a split. It does not freeze the protocol. It does not authorize an experiment or a product change.

