# commandF Provenance and Donor Policy

commandF is intended to reuse substantial prior art while remaining auditable, maintainable, and safe to distribute.

## Rule 1 — every adopted donor is pinned

Before source code, mappings, fixtures, schemas, or generated assets from another project enter commandF, record:

- donor repository/project
- exact upstream commit/tag/content version
- exact source path(s)
- upstream license for those paths
- separate content/data/terminology license where relevant
- adoption mode (`DEPEND`, `EMBED`, `PORT`, `COPY`, `IMPORT`, `ORACLE`, `STUDY`)
- commandF destination
- modifications
- upstream notices/attribution requirements
- checksum when practical
- permission evidence if rights are not established by a public license

## Rule 2 — repository license is not enough

A repository may include:
- code under one license
- third-party libraries under other licenses
- terminology/data with separate rights
- sample/vendor assets with separate rights

Therefore donor review is file/artifact scoped.

In particular, an Apache-2.0 terminology server does **not** imply unrestricted rights to SNOMED CT, ICD, LOINC distributions, vendor code sets, or other terminology content.

## Rule 3 — prefer dependency/import over unnecessary copies

Copying increases long-term patch/security burden. Use:

1. `DEPEND` when an upstream library/service has a stable interface.
2. `IMPORT` for declarative mappings/schemas/tests that commandF must compile.
3. `ORACLE` for independent validators/implementations.
4. `PORT/COPY` only when commandF needs a materially different runtime boundary or must integrate the algorithm directly.

## Rule 4 — copied code retains provenance

Copied/ported files should include an SPDX-compatible header or adjacent metadata where appropriate, for example:

```text
commandF donor-origin:
  repository: https://github.com/example/project
  commit: <sha>
  path: src/example.rs
  upstream-license: Apache-2.0
  adoption: PORT
  modifications: <summary>
```

Do not erase upstream copyright/notice text required by the applicable license.

## Rule 5 — permission-only material requires evidence

If the founder possesses rights beyond the public license, record a permission artifact/reference before the material is committed.

Minimum record:
- rights holder/authority
- scope of permission
- material covered
- date
- redistribution/publication constraints
- evidence location

Do not rely on an undocumented verbal assumption when commandF may later be open-sourced or commercialized.

## Rule 6 — no unverifiable leaked/stolen source

commandF will not search for, retrieve, or ingest leaked/proprietary source whose lawful provenance cannot be verified.

If the founder owns or is explicitly authorized to use private source, provide it through an authorized source/repository and record its permission evidence. It can then be audited like any other donor.

Public API behavior, published specifications, and independently observable behavior may be used for clean-room compatible implementations where appropriate.

## Rule 7 — benchmark/data provenance is separate

Research datasets and benchmark artifacts must record:
- data license/DUA
- permitted purpose
- whether redistribution is allowed
- PHI/de-identification status
- derived-artifact restrictions

Production test fixtures should default to synthetic data.

## Rule 8 — generated artifacts preserve source closure

Generated FHIR/openEHR/OMOP artifacts must retain enough metadata to reproduce:
- source package/version
- generator/compiler version
- input definitions
- mapping package
- terminology versions

## Required repository records

- `donors/manifest.yaml` — source-level adoption registry
- `THIRD_PARTY_NOTICES.md` — generated/curated notices before public release
- SBOM for released binaries/containers
- lockfiles for dependencies
- transformation certificates for benchmark/reference transformations

## Merge gate for donor code

No donor-code PR is merge-ready unless:

- [ ] exact donor ref is pinned
- [ ] source paths are enumerated
- [ ] license/permission is recorded
- [ ] third-party subtree/dependency inheritance is understood
- [ ] destination paths are enumerated
- [ ] tests demonstrate intended behavior
- [ ] provenance metadata is retained
- [ ] security scan/SBOM impact is known
- [ ] terminology/data content rights are handled separately

This policy is a technical provenance control, not legal advice.