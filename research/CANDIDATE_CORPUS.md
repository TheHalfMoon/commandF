# Candidate Corpus

Status: RESEARCH_PLANNING. One candidate member. Not frozen.

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

A split assignment is not required to enter this corpus. It is computed later from a frozen candidate digest, and the held-out manifest is bound at the protocol freeze. A missing field above keeps the item out. CommandF output cannot fill the label. A registry response is not a corpus item until its bytes are the bytes that were hashed.

## Current manifest

```text
ITEM_COUNT = 1
CORPUS_DIGEST = 955211e7e09cf3749d184513976c235f37499967e965fc0f614b7a5278deef0f
LABELS = 1 external published label
LABELS_MINTED_BY_COMMANDF = 0
HELD_OUT_SPLIT = NOT_FROZEN
RIGHTS = Apache-2.0 for the one member
```

The digest is the SHA-256 of the UTF-8 canonical JSON in the item record below, with keys sorted and no insignificant whitespace.

## Item 1

Case class: Malformed evidence. The input is not a FHIR JSON document because a token follows the object. The upstream test expects a parse failure.

```text
{"case_class":"Malformed evidence","input_sha256":"c1d536a36fdd1c3c75fb4aa13d51e1ca52039987d32c249e54cbd34bcc75fc6b","input_text":"{\"resourceType\":\"Patient\"} trailing","label_method":"rejectsMalformedJsonContent","label_statement":"DataFormatException containing Failed to parse JSON encoded FHIR content","source_blob":"ec074d00c7a756e201f89006f81d42e7d7954636","source_commit":"e307df6b64ff87c55af1607160f57141dbeb0360","source_path":"hapi-fhir-base/src/test/java/ca/uhn/fhir/parser/json/jackson/JacksonStructureTest.java","source_repository":"hapifhir/hapi-fhir","source_sha256":"ec3d8c0947d473c326a926f28c9acb853e9675bc33976ac4b124e2430ffba201"}
```

The input is 35 UTF-8 bytes. SHA-256 `c1d536a36fdd1c3c75fb4aa13d51e1ca52039987d32c249e54cbd34bcc75fc6b`. It names a resource type and no person, identifier, or clinical note.

`JacksonStructureTest.java` at `hapifhir/hapi-fhir` commit `e307df6b64ff87c55af1607160f57141dbeb0360` is blob `ec074d00c7a756e201f89006f81d42e7d7954636`, 4362 bytes, SHA-256 `ec3d8c0947d473c326a926f28c9acb853e9675bc33976ac4b124e2430ffba201`. The file starts with `package ca.uhn.fhir.parser.json.jackson;` and states no license of its own. The repository grant is Apache-2.0 in `LICENSE.txt`, as recorded for `SRC-HAPI-FHIR`. The Java file is not copied here.

Method `rejectsMalformedJsonContent` loads that exact text and asserts `DataFormatException` with a message containing `Failed to parse JSON encoded FHIR content`. That statement is the external label. Human adjudication is not required. The test was not executed in this repository. Replay is to run that method at the named commit.

This item does not assign a split. It does not freeze the protocol. It does not authorize an experiment or a product change.

