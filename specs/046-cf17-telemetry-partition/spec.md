# CF-17 Telemetry Partition

Status: AUTHORIZATION_SPEC

Playbook gap G22 requires published, CI, extension, terminology, and availability observations to stay partitioned. No current package does that. This spec authorizes three separate evidence documents for one already identified package version. This pull request does not implement them.

## Partition

The classes are separate documents, not fields that can be added together:

- `commandf.ecosystem-extension-telemetry/v1`
- `commandf.ecosystem-terminology-telemetry/v1`
- `commandf.ecosystem-availability-telemetry/v1`

Each document names one package, one exact version, one archive SHA-256, and one source host of `packages.fhir.org` or `packages2.fhir.org`. Source mutability is `IMMUTABLE_RELEASE` or `MUTABLE_CI`. `MUTABLE_CI` may be recorded. It remains ineligible as published authority.

Each document has an evidence state of `OBSERVED`, `MISSING`, or `UNSUPPORTED`. `MISSING` means the evidence class was not supplied. `UNSUPPORTED` means the supplied bytes are outside the contract for that class. Neither state is a compatibility verdict.

An extension count must not be stored in the terminology document, and an availability observation must not be stored in the extension document.

## Bounds and bytes

A document holds at most 256 evidence entries. Each entry identifier is at most 2048 bytes. The canonical document is at most 1 MiB. The implementation hashes the exact entry bytes it stores and uses those bytes in the document. No private consumer payload and no patient data are accepted. A payload that is not a bounded identifier fails.

Retrieval time is provenance and is excluded from the document digest. Replay of the same entries produces the same digest and performs no network read.

## Separation

None of the three documents may contain `PROVEN_COMPATIBLE`, `PROVEN_BREAKING`, `ALLOW`, or `BLOCK`. Telemetry does not grant a compatibility verdict.

## Out of this unit

- Registry crawling and scale measurement. Scale remains deferred until AF-04.
- Issue #100, issue #15, and CF-18.

## Implementation tests required after this spec is on main

1. the three documents reject one another's entry kinds
2. the same entries in a different order share one digest inside a class
3. `MISSING` and `UNSUPPORTED` stay distinct
4. `MUTABLE_CI` is retained and does not satisfy published authority
5. more than 256 entries fails
6. an oversized identifier fails
7. replay performs no network read
8. retrieval time does not change the digest
9. none of the documents contains a compatibility label
