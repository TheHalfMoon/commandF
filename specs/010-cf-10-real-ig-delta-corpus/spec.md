# CF-10 — Public Real-IG Delta Corpus

Status: implementation authorized / six frozen package states eligible and digest-frozen

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

The frozen corpus remains exactly:

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

No case or version was replaced after discovery.

## Foundation reconciliation and eligibility truth

CF-11 multi-version package-graph support is canonical on `main` at merge commit:

```text
5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
```

It was reconciled into the frozen CF-10 branch by merge commit:

```text
5ec463f0ae53b76f9c2c151335d98598b53e5abc
```

The original frozen CF-10 files remained byte-identical across that reconciliation.

The same six package states were then rerun without changing the corpus. `cf10-digest-discovery` run `31890014888` succeeded and produced reviewed artifact `9248341586`; a later exact-head rerun `31890859039` also succeeded. Every frozen state resolved twice into independent clean caches, passed CF-01 verification, and produced identical archive bytes/digests across the two resolutions.

The canonical manifest freezes only those observed digest/size facts. No semantic result counts were used to select or replace cases.

## Rights and redistribution boundary

CF-10 is **metadata-only in the repository**.

The repository MUST NOT vendor, commit, mirror, repackage, or redistribute the selected IG NPM archives, examples, terminology expansions, SNOMED CT content, RxNorm content, ISO content, or other upstream copyrighted/licensed payloads.

The corpus manifest stores only:

- package/version identity;
- commandF-observed content digest after public resolution;
- publication/provenance URLs;
- upstream rights/IP evidence URLs and conservative source-specific metadata;
- deterministic commandF result metadata/hashes that contain no redistributed source package bytes.

Package bytes are ephemeral runtime inputs acquired through the existing public package resolver and verified against the locked digest. Mixed upstream terminology/IP statements remain upstream obligations; commandF's repository license MUST NOT be interpreted as relicensing them.

## Digest discovery and freeze rule

The package SHA-256 values are not copied from web pages or manually invented.

For each frozen state, discovery MUST:

1. resolve the exact package version through commandF CF-01;
2. record the resolver-produced content SHA-256 and byte size;
3. independently resolve the same exact version into a second clean cache;
4. require identical digest and archive bytes between the two resolutions;
5. verify both package graphs through CF-01 cache verification;
6. persist only reviewed digest/size metadata into the corpus manifest.

After a digest is frozen, ordinary corpus runs MUST fail closed if resolution produces different bytes for the same package/version.

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

- schema is exactly `1`;
- manifest bytes and case count are bounded before/after decode as applicable;
- case ids are unique, stable, and lexicographically ordered;
- package names/versions are exact, never ranges/wildcards or path-like identities;
- archive digests are lowercase SHA-256 hex;
- archive sizes are positive and bounded;
- `fhir_version` is exactly `4.0.1` in CF-10 v1;
- publication/change/rights evidence is explicit per case;
- `rights_mode` for v1 is `metadata_only_no_redistribution`;
- `oracle_mode` is `changed_structure_definitions_only`;
- unknown schema, fields, or enum values fail closed;
- malformed/duplicate/unsorted cases fail closed;
- canonical JSON round-trip bytes are deterministic.

The manifest parser does not fetch the network.

## Package attestation contract

Before any semantic analysis of a case side, commandF MUST:

1. verify the complete supplied lockfile cache through the canonical CF-01 `Lockfile::verify_cache` authority;
2. require exactly one locked package matching the manifest `(package, version)` identity;
3. require the locked digest to equal the manifest digest;
4. read the target archive only through verified cache access;
5. require the observed archive byte length and SHA-256 to equal the manifest state.

A corrupt or missing dependency anywhere in the supplied lock graph blocks attestation even if the root archive itself is present.

## Authorized execution surface

CF-10 v1 authorizes exactly one user-visible execution surface:

```text
commandf corpus run \
  --manifest corpus/real-ig/v1/corpus.json \
  --work-root <path> \
  --oracle-adapter <path> \
  --oracle-java <path> \
  --format json
```

There is no second repository-owned public corpus harness in v1.

The command is an integration/benchmark operation and MAY acquire exact packages from the public FHIR package ecosystem, but it MUST preserve a hard internal boundary:

```text
acquire -> CF-01 verify -> manifest digest/size attest -> semantic evaluation
```

Semantic evaluation MUST NOT begin for a case until both before/after states attest successfully.

## Runner contract

A corpus run processes every frozen case in canonical case-id order. For one case it performs:

1. exact before/after package resolution into isolated state roots;
2. full CF-01 cache verification for each state;
3. manifest digest and archive-size attestation for both roots;
4. CF-03 structural diff using the attested root bytes;
5. CF-04 compatibility classification of that exact structural report;
6. CF-07 terminology evidence using those same verified package graphs;
7. CF-06 oracle comparison only for changed matched StructureDefinitions and only through the pinned adapter boundary;
8. deterministic case-summary emission plus raw sub-report evidence.

The runner MUST reuse those canonical authorities rather than reimplementing their semantics.

A case failure is represented explicitly. The runner MUST NOT silently skip a selected case because of unsupported content, a commandF/oracle disagreement, network/package failure, attestation failure, or unexpected finding count.

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

V1 summary output MUST be deterministic for the same canonical manifest, pinned packages, commandF build, and pinned oracle. It MUST NOT serialize timestamps, host-absolute paths, random ids, network timing, temporary cache paths, or unordered-map iteration.

## No golden-answer fabrication

Before the first real corpus execution, CF-10 MUST NOT hard-code expected structural counts, compatibility severities, terminology counts, oracle agreement counts, or a required "good" outcome.

After the first independently verified run, exact result hashes/counts MAY be frozen as regression evidence only if:

- raw commandF sub-reports are preserved in CI artifacts for review;
- a second clean run is byte-identical at the deterministic summary layer;
- expected values are derived from the observed deterministic run, not edited to make tests pass;
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

## Security boundary

Corpus metadata is untrusted input. The implementation must bound manifest/result sizes and counts, reject path-like package/version tricks where a package identity is expected, use explicit subprocess paths for the oracle boundary, and inherit CF-01/CF-06 process/cache hardening rather than bypass it.

No corpus field becomes shell code. CI uses quoted argv and no `eval`.

## Acceptance gates

A converged CF-10 candidate requires:

- frozen selection methodology before result discovery;
- exact package digest discovery using two independent clean resolutions;
- provenance/rights evidence for every case;
- typed manifest and failure-path tests;
- mandatory package attestation before semantic execution;
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
