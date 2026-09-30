# CF-17 Historical Package Graph

Status: AUTHORIZATION_SPEC

This is the next G01 slice after `specs/043-cf17-registry-acquisition/`. Merging it authorizes a following implementation. This pull request does not implement the graph.

Official acquisition of one exact package is canonical. G01 still requires a historical package graph with provenance and replay. This slice builds that graph from exact version identities that are already in hand. It does not contact the network and it does not claim the registry contains no other versions.

## Contract

- Input is one package name and one or more exact versions. Each version carries an archive SHA-256 and a source host of `packages.fhir.org` or `packages2.fhir.org`.
- `latest`, a missing patch version, and any other non-exact version are rejected.
- The graph orders versions by semantic version, not by input order.
- Each node's snapshot digest is recomputed with `project_snapshot` for that single package. A caller-supplied digest that differs is not accepted, because the graph does not store a snapshot digest it did not compute.
- A repeated version fails closed.
- More than 256 versions fails closed.
- Source mutability on every node is `IMMUTABLE_RELEASE`. That word is source mutability, not a compatibility verdict.
- The graph document is `commandf.ecosystem-package-history/v1`. Its identity excludes retrieval time, ETag, Last-Modified, content type, and HTTP status.
- The document records `SUPPLIED_VERSIONS_ONLY`. That value means the graph covers the supplied set. It is not a claim that the official registry has no other versions.
- Offline replay of the same inputs produces the same graph digest and performs no network read.
- The document must not contain a compatibility or policy label, and it must not grant one.

## Out of this unit

- A live catalog crawl or a multi-version download.
- Extension, terminology, and availability telemetry.
- Scale measurement, which remains deferred until AF-04.
- Issue #100, issue #15, and CF-18.

## Implementation tests required after this spec is on main

1. the same versions in a different order share one graph digest
2. semantic order places `1.10.0` after `1.9.0`
3. a repeated version fails
4. `latest` fails before any graph is produced
5. an unauthorized source host fails
6. 257 versions fail
7. recomputed snapshot digests change when the archive digest changes
8. replay is stable and has no transport argument
9. retrieval time is not part of the graph document
10. the document contains `SUPPLIED_VERSIONS_ONLY` and no compatibility label
