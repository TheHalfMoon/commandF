# CF-17 Advertised Version Acquisition

Status: AUTHORIZATION_SPEC

`specs/044-cf17-historical-package-graph/` records `SUPPLIED_VERSIONS_ONLY` for versions the caller already holds. It does not read an official version listing. This spec authorizes that listing for exactly one package name. This pull request does not implement it.

The hosts remain the two named by `specs/043-cf17-registry-acquisition/`: `packages.fhir.org` and `packages2.fhir.org`. No other host, mirror, or branch is a source. The only archive redirect remains the one pinned secondary tarball. The same URL is not requested twice.

## Listing

The listing is the exact metadata body returned for one package name. The sequence is a bounded read, a SHA-256 of those bytes, then a parse of those same bytes. A later reopen is not a substitute.

Every key in `versions` must be an exact semantic version. `latest` and any other non-exact token fail the acquisition. A repeated version fails. More than 256 versions fails. A listing larger than 4 MiB fails before the version set is accepted. A truncated listing fails. A timeout is a timeout. A redirect of the listing fails.

The listing document is `commandf.ecosystem-advertised-versions/v1`. Retrieval time, ETag, Last-Modified, content type, and HTTP status are provenance. They are not part of the listing digest.

## Archives

Each advertised version is acquired from the same host that supplied the listing. Metadata from one host and an archive from the other are not one observation. Each archive uses the existing 128 MiB compressed bound and the 30 second request budget. This unit does not decompress archives.

The aggregate compressed budget is 256 times 128 MiB. The request count is at most one listing plus one archive per advertised version plus one pinned redirect per archive. The total duration is that request count times 30 seconds.

An advertised version whose archive cannot be verified fails the acquisition. The failure names that version. The implementation must not omit it and still return `ADVERTISED_BY_OFFICIAL_SOURCE`.

## Coverage

`ADVERTISED_BY_OFFICIAL_SOURCE` means every exact version in the verified listing has a verified archive from that same listing. `SUPPLIED_VERSIONS_ONLY` remains the caller-supplied graph and is a different document. The two markers must not be interchangeable.

This unit does not define `COMPLETE_REGISTRY_HISTORY`.

The output contains no compatibility or policy label and does not grant one.

## Replay

Stored listing bytes and stored archive bytes replay without a transport. A listing or archive whose bytes no longer match the recorded digest fails. A stored archive whose name or version differs from the listing entry fails.

## Out of this unit

- Enumeration of every package in the registry.
- Extension, terminology, and availability telemetry.
- Scale measurement, which remains deferred until AF-04.
- Issue #100, issue #15, and CF-18.

## Implementation tests required after this spec is on main

1. a valid listing is accepted
2. the same listing bytes have one digest
3. advertised versions sort by semantic version
4. a duplicate version fails
5. a malformed version fails
6. an unauthorized host fails before a request
7. an oversized listing fails before the version set is accepted
8. 257 advertised versions fail
9. a redirect outside the pinned tarball fails
10. a timeout is an explicit timeout
11. an archive digest mismatch fails
12. a package-name mismatch fails
13. a package-version mismatch fails
14. replay reads the verified stored bytes
15. replay performs no network read
16. a tampered stored listing fails
17. a tampered stored archive fails
18. `ADVERTISED_BY_OFFICIAL_SOURCE` and `SUPPLIED_VERSIONS_ONLY` are different documents
19. the output contains no compatibility label
20. the output contains no field that grants a compatibility verdict
