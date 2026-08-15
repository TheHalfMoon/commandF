# CF-10 — Public Real-IG Delta Corpus

Status: planned / selection rules frozen before digest discovery

## Purpose

CF-10 establishes a small, reproducible, public corpus of **real published FHIR R4 Implementation Guide version deltas** for commandF evaluation.

The corpus is benchmark evidence, not a new compatibility authority. It reuses canonical commandF capabilities to measure real upstream change:

- CF-01 package resolution / digest verification;
- CF-03 deterministic structural diff;
- CF-04 compatibility classification;
- CF-07 terminology evidence;
- CF-06 pinned HL7 oracle evidence where comparable StructureDefinitions changed.

CF-10 MUST NOT change the semantics of those slices to improve corpus results.

## Anti-cherry-picking rule

Selection criteria are frozen **before** package digests or commandF results are collected.

A case is eligible only when all of the following are true:

1. the IG has at least two stable, published, permanent versions;
2. both selected package versions are FHIR R4 / 4.0.1 compatible with the current commandF surface;
3. the upstream publication provides an explicit package identity and public version/change evidence;
4. the selected versions are ordinary published releases, not CI builds, nightly builds, ballots, snapshots, or local forks;
5. the package can be resolved from the public FHIR package ecosystem by commandF's existing resolver;
6. the case contains no PHI and requires no private dataset access;
7. commandF results are **not** used to decide whether the case remains in the corpus;
8. divergence, unsupported content, oracle disagreement, or zero findings are evidence and MUST NOT cause silent case removal.

Once a case passes this gate, later unfavorable output cannot remove it from corpus v1 except for a documented upstream-rights/access failure or a proven mistaken eligibility fact.

## Corpus v1 families

The initial frozen candidate set deliberately spans three distinct interoperability contexts:

### C001 — US Core annual national-core delta

```text
package: hl7.fhir.us.core
before: 8.0.1
before publication: https://hl7.org/fhir/us/core/STU8.0.1/
after: 9.0.0
after publication: https://hl7.org/fhir/us/core/STU9/
FHIR: 4.0.1
realm/context: US national/base core
```

Rationale: US Core is an annually revised base IG and publishes explicit cross-version/change guidance. The pair is adjacent published annual releases and is not chosen from commandF output.

### C002 — International Patient Summary major-generation delta

```text
package: hl7.fhir.uv.ips
before: 1.1.0
before publication: https://hl7.org/fhir/uv/ips/STU1.1/
after: 2.0.1
after publication: https://hl7.org/fhir/uv/ips/en/
FHIR: 4.0.1
realm/context: international patient summary
```

Rationale: IPS 2 documents explicit non-compatible and compatible substantive changes relative to the previous generation. This gives the corpus an international, terminology-rich document/profile family rather than another US-only base guide.

### C003 — mCODE specialty-oncology major delta

```text
package: hl7.fhir.us.mcode
before: 3.0.0
before publication: https://hl7.org/fhir/us/mcode/STU3/
after: 4.0.0
after publication: https://hl7.org/fhir/us/mcode/STU4/
FHIR: 4.0.1
realm/context: oncology specialty IG
```

Rationale: mCODE publishes stable R4 generations and has explicit version-differential/change material. It exercises specialty profiles and terminology rather than general core/document-only content.

## Rights and redistribution boundary

CF-10 is **metadata-only in the repository**.

The repository MUST NOT vendor, commit, mirror, repackage, or redistribute the selected IG NPM archives, examples, terminology expansions, SNOMED CT content, RxNorm content, ISO content, or other upstream copyrighted/licensed payloads.

The corpus manifest stores only:

- package/version identity;
- commandF-observed content digest after public resolution;
- publication/provenance URLs;
- upstream rights/IP evidence URLs and a conservative rights note;
- deterministic commandF result metadata/hashes that contain no redistributed source package bytes.

Package bytes are ephemeral runtime inputs acquired through the existing public package resolver and verified against the locked digest. Mixed upstream terminology/IP statements remain upstream obligations; commandF's repository license MUST NOT be interpreted as relicensing them.

## Digest discovery and freeze rule

The package SHA-256 values are not copied from web pages or manually invented.

For each frozen case, an isolated discovery gate MUST:

1. resolve the exact package version through commandF CF-01;
2. record the resolver-produced content SHA-256 and byte size;
3. independently resolve the same exact version into a second clean cache;
4. require identical digest and archive bytes between the two resolutions;
5. verify the package through CF-01 cache verification;
6. persist only the digest/size metadata into the corpus manifest.

After the digest is frozen, ordinary corpus runs MUST fail closed if a public resolution produces different bytes for the same package/version.

## Manifest contract

Canonical v1 manifest path:

```text
corpus/real-ig/v1/corpus.json
```

Schema-v1 logical shape:

```text
schema
selection_policy
cases[]
  id
  package
  before
    version
    archive_sha256
    archive_bytes
    publication_url
  after
    version
    archive_sha256
    archive_bytes
    publication_url
  fhir_version
  publisher
  change_evidence_url
  rights_evidence_url
  rights_mode
  oracle_mode
```

Rules:

- case ids are unique and stable;
- package names/versions are exact, never ranges/wildcards;
- archive digests are lowercase SHA-256 hex;
- archive sizes are positive and bounded;
- `fhir_version` is exactly `4.0.1` in CF-10 v1;
- publication/change/rights evidence is explicit per case;
- `rights_mode` for v1 is `metadata_only_no_redistribution`;
- `oracle_mode` is `changed_structure_definitions_only`;
- unknown schema versions fail closed;
- malformed/duplicate/unsorted cases fail closed.

## Runner contract

CF-10 MAY introduce a thin deterministic corpus orchestrator, but it MUST call existing commandF authorities rather than reimplement their semantics.

A corpus run for one case performs:

1. exact before/after package resolution into isolated caches;
2. digest and archive-size attestation against the frozen manifest;
3. CF-01 cache verification;
4. CF-03 structural diff;
5. CF-04 classification;
6. CF-07 terminology diff;
7. CF-06 oracle comparison for changed comparable StructureDefinition pairs using the pinned local adapter;
8. deterministic case-summary emission.

A case failure is represented explicitly. The runner MUST NOT skip a selected case because of unsupported content, a commandF/oracle disagreement, a network/package failure, or an unexpected finding count.

## Result contract

CF-10 produces commandF-owned deterministic summary evidence only. It does not store upstream IG artifacts.

Per-case result must identify at least:

```text
case_id
package
before_version / before_sha256
after_version / after_sha256
structural_change_count
compatibility_finding_count
compatibility severity/direction aggregate counts
terminology aggregate counts
oracle compared/agreement/divergence aggregate counts
case_status
```

`case_status` is operational/evidence state only and MUST NOT rewrite CF-04/05 compatibility truth.

V1 summary output MUST be deterministic for the same manifest, pinned packages, commandF build, and pinned oracle.

## No golden-answer fabrication

Before the first real corpus execution, CF-10 MUST NOT hard-code expected structural counts, compatibility severities, terminology counts, oracle agreement counts, or a required "good" outcome.

After the first independently verified run, exact result hashes/counts MAY be frozen as regression evidence only if:

- the raw commandF outputs are preserved in CI artifacts for review;
- a second clean run is byte-identical;
- the expected values are derived from the observed deterministic run, not edited to make tests pass;
- future differences fail visibly and require explicit corpus-version reconciliation.

## Failure semantics

CF-10 fails closed on:

- manifest schema/shape violations;
- duplicate or non-canonical case ordering;
- package resolution failure;
- package/version/digest/size mismatch;
- cache verification failure;
- unsupported non-R4 case in v1;
- missing rights/change/publication evidence metadata;
- malformed commandF sub-report;
- oracle operational failure for a case configured to run it;
- non-deterministic repeated result bytes.

Failures are never silently converted to corpus exclusion or pass.

## Determinism and reproducibility

The corpus must not serialize:

- timestamps;
- host-absolute paths;
- random ids;
- network timing;
- temporary cache paths;
- unordered map iteration.

Case order is lexicographic by stable case id. Aggregate maps are canonically ordered.

## Security boundary

Corpus metadata is untrusted input. The implementation must bound manifest/result sizes and counts, reject path-like package/version tricks where a package identity is expected, use explicit subprocess paths for the oracle boundary, and inherit CF-01/CF-06 process/cache hardening rather than bypass it.

No corpus field becomes shell code. CI uses quoted argv and no `eval`.

## Acceptance gates

A converged CF-10 candidate requires:

- frozen selection methodology before result discovery;
- exact package digest discovery using two independent clean resolutions;
- provenance/rights evidence for every case;
- schema and failure-path tests;
- deterministic two-run corpus equality;
- `cargo fmt --all -- --check`;
- locked workspace Clippy with `-D warnings`;
- full workspace tests;
- real public corpus CI on all frozen v1 cases;
- existing CF-08/CF-09 security regressions remain green;
- existing real FHIR / terminology gates remain green;
- `cf06-oracle` remains green;
- CodeRabbit/Qodo/other reviewer findings verified and dispositioned without inventing unavailable PASS states.

## Explicit deferrals

CF-10 does not add:

- new compatibility rules or policy thresholds;
- a new terminology semantic authority;
- a replacement HL7 oracle;
- private/credentialed datasets;
- PHI;
- MIMIC-derived data;
- ballot/nightly IG stratum;
- cross-model FHIR↔openEHR/OMOP benchmark;
- server behavior benchmark;
- ecosystem blast-radius graph;
- baselines/suppressions;
- AutoFix;
- AI/agent semantic authority.
