# commandF

**Universal health interoperability compiler and verification platform.**

commandF is being designed to compile healthcare data across heterogeneous standards and systems while preserving clinical meaning, provenance, and verifiable transformation evidence.

> Status: foundation/bootstrap. The repository is intentionally establishing semantic contracts, donor provenance, and verification boundaries before production adapters are imported.

## Foundation architecture

The intended pipeline is:

```text
source bytes/events
  -> source dialect parser
  -> typed source IR
  -> CSIR normalization
  -> mapping + terminology passes
  -> target dialect lowering
  -> authoritative validator/oracle
  -> semantic verifier
  -> loss ledger
  -> transformation certificate
```

FHIR is a first-class dialect and validation target, not commandF's sole internal canonical model.

## Repository map

- `docs/DONOR_AUDIT_2026-08-13.md` — deep audit of open/authorized projects and code worth reusing.
- `docs/GAP_SOLUTION_MAP.md` — each interoperability gap mapped to existing donors plus commandF-owned work.
- `docs/PROVENANCE_AND_DONOR_POLICY.md` — mandatory provenance and permission boundary for reused code/data.
- `donors/manifest.yaml` — initial donor candidates.
- `donors/final-pass-2026-08-13.yaml` — candidates discovered in the final gap-filling pass.
- `research/RESEARCH_AGENDA.md` — **research ideas only**, deliberately separated from product requirements.
- `crates/commandf-csir` — initial typed Clinical Semantic Intermediate Representation and loss-event contracts.
- `crates/commandf-evidence` — initial transformation evidence/certificate contracts.

## Foundational invariants

1. No known information loss is silent.
2. A target validator PASS does not by itself establish semantic equivalence.
3. Source facts remain traceable to exact source locations and content hashes.
4. FHIR/openEHR/OMOP/HL7/CDA/DICOM and vendor formats remain explicit dialects; none silently becomes “the truth” for every use case.
5. Mapping languages are imported into a typed Mapping IR rather than forcing users to discard existing institutional mappings.
6. Terminology resolution records the terminology edition/version and resolver evidence; missing mappings remain explicit gaps.
7. Donor code/assets are not adopted until repository, immutable ref, path, license/permission, notices, and modifications are pinned.
8. AI may propose mappings; deterministic compilers, terminology services, validators, and semantic verification remain the authority boundary.

## Current validation

The bootstrap PR configures GitHub Actions to run Rust formatting, Clippy, and workspace tests. Until those checks run successfully, the Rust contracts are foundation candidates rather than a certified release.
