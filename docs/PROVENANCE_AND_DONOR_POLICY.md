# commandF Provenance and Donor Policy

commandF is designed to reuse substantial prior art while remaining auditable, maintainable, and safe to distribute.

## 1. Every adopted donor is pinned

Before source code, mappings, fixtures, schemas, generated assets, research datasets, or terminology content from another project enter commandF, record:

- donor repository/project
- exact upstream commit/tag/content version
- exact source path(s)
- upstream license for those paths
- separate content/data/terminology license where relevant
- adoption mode (`DEPEND`, `EMBED`, `PORT`, `COPY`, `IMPORT`, `ORACLE`, `STUDY`)
- commandF destination or execution boundary
- modifications
- required notices/attribution
- checksum where practical
- permission evidence if rights are not established by a public license

## 2. Repository license is not enough

A repository can contain code, third-party libraries, terminology, data, samples, generated assets, and vendor content under different rights. Review is file/artifact scoped.

In particular, an open-source terminology server does not grant unrestricted rights to terminology content such as SNOMED CT, ICD, LOINC distributions, or vendor code sets.

## 3. Prefer dependency/import/oracle over unnecessary copies

Default order:

1. `DEPEND` for stable upstream libraries/services.
2. `IMPORT` for declarative mappings/schemas/tests commandF must analyze or compile.
3. `ORACLE` for independent validators and implementations.
4. `PORT`/`COPY` only where a materially different trusted runtime boundary or direct algorithm integration is justified.
5. `STUDY` for architecture/product patterns without source adoption.

## 4. Copied/ported code retains provenance

Copied or ported files preserve upstream notices and should carry adjacent donor metadata where appropriate:

```text
commandF donor-origin:
  repository: <repo>
  commit: <immutable-sha>
  path: <upstream-path>
  upstream-license: <license>
  adoption: <mode>
  modifications: <summary>
```

## 5. Permission-only material requires evidence

Where the founder has rights beyond a public license, record a permission artifact/reference before material is committed. Minimum record: rights holder, scope, covered material, date, redistribution/publication constraints, and evidence location.

## 6. No unverifiable leaked/proprietary source

Do not search for, retrieve, or ingest leaked/proprietary source whose lawful provenance cannot be verified. Authorized private source may be audited when supplied through an authorized source and accompanied by permission evidence.

Published specifications, public APIs, and independently observable behavior may be used for clean-room compatible implementations where appropriate.

## 7. Benchmark/data provenance is separate

Research datasets and benchmark artifacts record:

- data license/DUA
- permitted purpose
- redistribution rights
- PHI/de-identification status
- derived-artifact restrictions

Repository and CI fixtures default to synthetic/public/permitted data.

## 8. Generated artifacts preserve source closure

Generated healthcare artifacts retain enough metadata to reproduce source package/version, compiler/generator version, input definitions, mapping package, terminology versions, rules/policies, and validator/oracle identities.

## 9. Required records

As commandF matures, maintain:

- donor manifests for adopted/candidate sources
- third-party notices before public distribution
- dependency lockfiles
- SBOMs for released binaries/containers
- evidence/certificate records for protected benchmark/reference transformations

## 10. Donor merge gate

A donor-code PR is not merge-ready unless:

- exact donor ref is pinned
- source paths are enumerated
- license/permission is recorded
- third-party inheritance is understood
- destination/execution boundary is explicit
- tests demonstrate intended behavior
- provenance metadata is retained
- security/SBOM impact is understood
- terminology/data rights are handled separately

This is a technical provenance control, not legal advice.
