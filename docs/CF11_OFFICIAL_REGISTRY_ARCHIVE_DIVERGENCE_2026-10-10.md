# CF11: Proven official FHIR archive divergence

Status: REPRODUCED / STRICT RAW-BYTE IDENTITY PRESERVED / HISTORIC FAILED RUNNER ATTRIBUTION UNKNOWN

Evidence date: 2026-10-10. Machine: authorized macbook (Darwin arm64).
Scope: independently observed archive bytes and lockfiles, not a new source-policy adoption or AF-02 authority decision.

## Proven end-to-end lockfile observation

Canonical [PR #235](https://github.com/TheHalfMoon/commandF/pull/235) added an offline lock comparator; [PR #236](https://github.com/TheHalfMoon/commandF/pull/236) added explicit primary/secondary selection without changing default fallback.

Two independently resolved IPS 2.0.1 graphs, using separate caches and mutually exclusive official source selection, each resolved and verified 9/9 package objects. Source-host checks confirm 9/9 primary objects from packages.fhir.org and 9/9 secondary objects from packages2.fhir.org. Comparison: exit 2, equivalent=false, two exact package SHA-256 differences, identical roots and resolved edges, and the same declared dependency values. The other seven raw digests are equal.

| Package | Primary raw archive SHA-256 | Secondary raw archive SHA-256 |
|---|---|---|
| hl7.fhir.uv.ips@2.0.1 | 7183242b70fb2a9058aa3701fb607517a3c2fd0e3100d1d8c538d744c2adf799 | 5ebcb6d4bfbb32cd0b36b7d2165cb43c1aafc34e2849db3e37ecee07e7d781d2 |
| hl7.fhir.r4.core@4.0.1 | ebd7731df7d36b5b7d39d5fb6c9d77b44bb7fe5742f1a2e87f164738c3289d44 | b090bf929e1f665cf2c91583720849695bc38d2892a7c5037c56cb00817fb091 |

## Compressed and decompressed TAR comparisons

Both pairs were downloaded directly from the approved official registry URLs. The bounded, offline, standard-library Python analyzer at tools/cf11-origin-audit/inspect_archives.py counts original bytes, TAR entries, decompressed regular-file digests, and member metadata without extracting anything onto disk.

| Measurement | IPS primary | IPS secondary | R4 core primary | R4 core secondary |
|---|---:|---:|---:|---:|
| Compressed bytes | 725312 | 749559 | 12815597 | 4531911 |
| TAR members | 155 | 159 | 5046 | 4742 |
| Regular files | 155 | 154 | 5046 | 4739 |
| Non-file TAR entries | 0 | 5 | 0 | 3 |

IPS has one primary-only regular file (package/other/publication-request.json), zero secondary-only regular files, one shared-file content difference (package/package.json), and 154 shared TAR metadata differences. The primary manifest includes notForPublication=true and a local publishing URL, whereas the secondary manifest omits that flag and uses the published IPS URL. Exact package name/version and four dependencies match.

R4 core has 466 primary-only regular paths, 159 secondary-only regular paths, two common-path content differences (package/.index.json and package/package.json), and 4580 shared TAR metadata differences. For example, openapi/Account.schema.json in primary and package/openapi/Account.schema.json in secondary have different paths; counting exact paths does not establish a clinical semantic difference. Both package manifests retain the same name/version and no dependencies, but the primary has fhir-version-list and secondary has fhirVersions.

## Reproduce (authorized network, then offline audit)

The archive inspector uses local file paths only. It makes no network requests and cannot execute archive contents. The downloads below are explicit official URLs, not an arbitrary mirror policy:

    mkdir -p /tmp/commandf-cf11-audit
    curl -fLsS --max-time 65 https://packages.fhir.org/hl7.fhir.uv.ips/2.0.1 -o /tmp/commandf-cf11-audit/ips-primary.tgz
    curl -fLsS --max-time 65 https://packages2.fhir.org/web/hl7.fhir.uv.ips-2.0.1.tgz -o /tmp/commandf-cf11-audit/ips-secondary.tgz
    curl -fLsS --max-time 65 https://packages.fhir.org/hl7.fhir.r4.core/4.0.1 -o /tmp/commandf-cf11-audit/r4-primary.tgz
    curl -fLsS --max-time 65 https://packages2.fhir.org/web/hl7.fhir.r4.core-4.0.1.tgz -o /tmp/commandf-cf11-audit/r4-secondary.tgz
    python3 tools/cf11-origin-audit/inspect_archives.py /tmp/commandf-cf11-audit/ips-primary.tgz /tmp/commandf-cf11-audit/ips-secondary.tgz
    python3 tools/cf11-origin-audit/inspect_archives.py /tmp/commandf-cf11-audit/r4-primary.tgz /tmp/commandf-cf11-audit/r4-secondary.tgz

Return codes: 0 for identical raw byte identity, 2 for genuine byte identity difference, 1 for invalid/oversized/unsafe archives. Explicitly accept expected exit code 2 when running under shell set -e.

Offline regression tests:

    python3 -m unittest discover -v -s tools/cf11-origin-audit -p "test_*.py"

The auditor bounds compressed byte length to 128 MiB, each expanded regular member to 64 MiB, total expanded regular bytes to 512 MiB, and member count to 50000. It records a digest of declared dependency values instead of unbounded original values, and bounds each category to 25 example paths.

## No unsupported closure claims

This is a controlled reproduction of official primary-vs-secondary raw-byte divergence and supports a possible mechanism for intermittent default fallback variation. It does not prove the origins used by a specific historical failing GitHub Actions runner because those lockfiles were not retained. It does not assert that reorganized R4 Core paths are clinically equivalent.

Keep [issue #214](https://github.com/TheHalfMoon/commandF/issues/214) open pending separately governed source-policy disposition and/or historical run attribution. Separate the previously confirmed DNS outage [#221](https://github.com/TheHalfMoon/commandF/issues/221), where no valid lockfile was created. Do not relax strict raw SHA equality, bypass CI, or rewrite the AF-02 protected retained authority [#100](https://github.com/TheHalfMoon/commandF/issues/100).

This document and offline inspector are reproducibility diagnostics only. The original downloaded archives are not checked into the repository.
