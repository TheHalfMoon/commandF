# CF-17 Official Registry Acquisition

Status: AUTHORIZATION_SPEC

This Spec Kit is the later spec named by `specs/041-cf17-offline-replay/plan.md`. After it is a normal merge on canonical `main`, the next package may implement this contract. This pull request does not contain that implementation.

`specs/018-v3-1-authority-migration/spec.md` point 3 allows a CF-17 slice after the Wave -1 unit it depends on is closed or explicitly deferred. Issues #35, #36, #37, #38, and #40 are closed. Issue #100 is explicitly deferred for this slice: its historical Actions artifact is not an input to registry acquisition. Issue #15 is not a dependency. CF-01 registry download remains package resolution. It does not by itself satisfy G01.

## Purpose

Acquire one exact official FHIR package registry archive and bind its verified bytes to one published CF-17 snapshot observation. The acquisition creates retrieval evidence. It does not classify compatibility.

## Source authority

The only authorized hosts are the two already qualified by `crates/commandf-pkg/src/registry.rs`:

- `https://packages.fhir.org`
- `https://packages2.fhir.org`

The only allowed redirect is one HTTP 302 from `https://packages2.fhir.org/packages/{name}/{version}` to exactly `https://packages2.fhir.org/web/{name}-{version}.tgz`. Any other host, any other redirect, and any recursive redirect are unauthorized.

If the primary host fails before returning an archive, the secondary host may be used for both the version metadata and the archive. Metadata from one host and archive bytes from the other are not a valid observation. There is no third mirror.

## Identity

The caller supplies an exact package name and an exact semantic version. A floating range, a missing version, and the token `latest` are rejected before any request.

The observation source identity is the host that produced the archive bytes: `packages.fhir.org` or `packages2.fhir.org`. The source mutability recorded on the snapshot is `IMMUTABLE_RELEASE`. That word describes the package source. It is not a compatibility verdict.

Retrieval fields are provenance and are excluded from `snapshot_sha256`:

- final request URL
- HTTP status
- content type, empty when absent
- ETag, empty when absent
- Last-Modified, empty when absent
- retrieval time

Changing only the retrieval time must not change the snapshot digest.

## Verified bytes

The archive sequence is: request, bounded read of the response body, SHA-256 of those exact bytes, verify the caller-supplied expected digest when one was supplied, then store and consume those same bytes. A later reopen of a path is not a substitute for the bytes just read.

`bytes_consumed` equals `bytes_digest_verified`.

## Bounds

These match the existing registry client:

- global request timeout, including connect time and one allowed redirect: 30 seconds
- metadata body: 4 MiB
- compressed archive body: 128 MiB
- packages in one acquisition: 1
- feed entries in this unit: 0
- redirects followed: 1, and only the pinned secondary tarball
- retries of the same URL: 0
- total acquisition duration: the same 30 second budget

A body that reaches the limit is a failure. A declared length above the limit is a failure. This unit does not decompress the archive.

## Fail closed

The implementation must reject, with a typed error rather than a partial snapshot:

- oversized or truncated body
- timeout
- malformed metadata or a body that is not a gzip archive
- redirect outside the pinned secondary tarball
- any unauthorized host
- digest mismatch against a supplied expected SHA-256
- duplicate `name@version` when the observation is bound into a snapshot that already contains that identity
- mixed-host metadata and archive
- a second attempt of the same URL
- an unverifiable acquisition, including a missing body after a success status

## Offline replay

Stored exact bytes plus the recorded name, version, digest, and source host replay through the existing snapshot projector with no network call. A transport that is still configured must not be invoked by replay.

## Separation

The acquisition document and the snapshot it feeds must not contain `PROVEN_COMPATIBLE`, `PROVEN_BREAKING`, `ALLOW`, `BLOCK`, or any other compatibility or policy label. Acquisition cannot grant a downstream compatibility verdict.

## Out of this unit

- A multi-package registry catalog or historical version graph.
- Extension, terminology, and availability telemetry.
- Scale measurement. The playbook still defers that until AF-04.
- Issue #100 byte recovery, the issue #15 pin, and CF-18.

## Implementation tests required after authorization

The implementation package must include deterministic tests, using an injected transport rather than a required live network, for:

1. an exact authorized response is accepted
2. the same bytes produce the same digest
3. the same normalized acquisition produces the same snapshot digest
4. an oversized body fails
5. a timeout fails as a timeout
6. malformed metadata or a non-gzip body fails
7. a redirect outside the pinned tarball fails
8. an unauthorized host fails before a request
9. a truncated body fails
10. a digest mismatch fails
11. a duplicate `name@version` fails
12. cache storage and reuse use the verified bytes
13. offline replay succeeds from stored bytes
14. offline replay performs no network read
15. the source host is preserved
16. retrieval time and headers do not change the snapshot digest
17. a second try of the same URL cannot change the snapshot digest
18. stored bytes that no longer match the recorded digest fail verification
19. the output contains no compatibility label
20. the output contains no field that grants a compatibility verdict
