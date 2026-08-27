# AF-01 Stack C Scorecard Posture and Finding Disposition

Status: `T031_DISPOSITIONED_DEVELOPMENT_EVIDENCE_FINAL_T041_PENDING`

This record is posture evidence. OpenSSF Scorecard's aggregate score is not commandF correctness authority and is not used as an AF-01 semantic PASS signal.

## Canonical repository state inspected

Repository-aware posture scan:

```text
canonical main at scan: 301aa5e66089859e938145870dc4a9300a25692a
canonical main tree: d8557e6992ea82c0d2bb36178cf85961243e0691
Stack C development head: 35cfe84118969b9c1fb3ca4d8cd3faef8e1d0918
workflow: af01-scorecard
run: 33054545670
artifact: 9639039033
artifact name: af01-scorecard
artifact digest: sha256:26c415bf0b08e03e0b55a2bbf6336c0c4363638485e8f89194ab625e0032dd6c
```

The artifact contains two independent views:

- `af01-scorecard-local.json`: the exact checked-out Stack C source tree, used for source-file posture such as workflow policy, `SECURITY.md`, and Dependabot configuration.
- `af01-scorecard-repository.json`: a repository-aware scan of live `github.com/TheHalfMoon/commandF`, used for live GitHub posture such as Branch-Protection.

The development run above used Scorecard v5.5.0. Stack C subsequently hardened execution further: instead of relying on `ossf/scorecard-action`'s internally mutable `docker://ghcr.io/ossf/scorecard-action:v2.4.4` image reference, the workflow downloads the official Scorecard v5.5.0 Linux amd64 release archive and verifies GitHub's published SHA-256 before execution:

```text
scorecard version: 5.5.0
scorecard source commit reported by result: c395761df6afe1a69e476bc60a013a94bcbc153f
release asset: scorecard_5.5.0_linux_amd64.tar.gz
release asset sha256: 83b90a05c1540ef1390db1cd5711e5fd04be9c1d8537fb84d39d02092d6a8dff
```

Final T041 evidence must come from the hardened exact final Stack C head; this development artifact is retained as the T031 finding/disposition input.

## Required T031 checks

### Branch-Protection — `0/10` on live canonical main

Observed detail:

```text
Warn: branch protection not enabled for branch 'main'
```

Disposition: `BLOCKING_T038`.

This is not waived. T037 defines the exact checked-in `main` ruleset contract; T038 must apply it through an administrator-authorized GitHub path; T039 must read live GitHub back; and T040 must prove the effective negative-governance properties. AF-01 cannot close while this finding remains true.

### Dangerous-Workflow — `10/10`

Observed result: no dangerous workflow patterns detected.

Disposition: `NO_FINDING`.

This is complementary to, not a replacement for, the repository-owned AF-01 workflow-trust audit and zizmor gate.

### Pinned-Dependencies — `10/10`

Observed result: all dependencies inspected by Scorecard were pinned.

Disposition: `NO_FINDING`.

The AF-01 repository-owned workflow audit remains stricter authority for external Action full-SHA references, credentialless checkout, proof-container identity, and the exact tracked Action-metadata surface.

### Token-Permissions — `10/10`

Observed result: GitHub workflow tokens follow least privilege.

Disposition: `NO_FINDING`.

The checked-in workflow-trust policy remains the machine-checkable permission authority.

### Security-Policy

Live canonical main at the T031 scan scored `0/10` because `SECURITY.md` did not yet exist there. The exact Stack C local tree already detected `SECURITY.md` and scored `4/10`; the initial policy lacked direct linked reporting content. Stack C then added a direct GitHub private security-advisory reporting link while retaining fail-safe public routing instructions that forbid posting exploit material.

Disposition: `FIX_IMPLEMENTED_FINAL_SCORECARD_RECHECK_REQUIRED`.

The final exact-head Scorecard run must inspect the hardened policy. No aggregate-score target is required; the substantive acceptance criterion is a present, usable vulnerability-reporting policy without weakening security handling.

### Dependency-Update-Tool

Live canonical main at the T031 scan scored `0/10`. The exact Stack C local tree scored `10/10` after adding `.github/dependabot.yml` for the Cargo workspace, the Maven HL7 oracle, and GitHub Actions, each on a bounded weekly schedule.

Disposition: `FIX_IMPLEMENTED_FINAL_SCORECARD_RECHECK_REQUIRED`.

Dependabot does not authorize automatic dependency merges or semantic oracle upgrades; each resulting PR remains subject to repository qualification.

### Vulnerabilities — `8/10`

Scorecard reported two repository-level OSV/GHSA findings:

```text
GHSA-rcgg-9c38-7xpx / CVE-2026-45292
GHSA-269g-pwp5-87pp / CVE-2020-15250
```

The first affects OpenTelemetry Java baggage propagation versions before the fixed line; the second affects JUnit4 `TemporaryFolder` before its fixed line. commandF does not declare either package directly in the Rust workspace. RustSec `cargo-audit 0.22.2` on the exact Cargo lockfile is independently enforced and was clean during Stack B qualification.

The repository contains a separate Maven-based HL7/FHIR oracle at `tools/hl7-oracle/pom.xml`, currently pinned to the existing qualified HL7 FHIR toolchain. AF-01 freezes product/oracle semantics and therefore does not authorize blindly changing that oracle dependency graph merely to improve an aggregate posture score.

Disposition: `BOUNDED_EXTERNAL_ORACLE_DEPENDENCY_FINDING_REQUIRES_SEPARATE_REQUALIFICATION`.

Scope and controls:

- no Rust/Cargo advisory waiver is created;
- no Scorecard threshold is lowered or finding hidden;
- the exact advisory identities remain visible in retained Scorecard evidence;
- the Maven oracle is not application runtime authority and remains bounded to the existing proof/oracle workflows;
- weekly Maven Dependabot discovery is enabled, but no update is automatically trusted or merged;
- a future oracle dependency update that proves the affected transitive package is removed/fixed must run the full applicable oracle/quality proof set before becoming canonical.

Revisit/removal condition: remove this disposition only when exact dependency-tree evidence plus qualified oracle workflows prove the affected package versions are no longer present, or when authoritative vulnerability data establishes the finding is inapplicable to the pinned oracle graph.

## Additional local Scorecard checks

The local development view also reported:

- License: `0/10` — no repository license file has been selected. AF-01 has no authority to choose a legal license on the founder's behalf; this is not silently treated as completed assurance.
- SAST: `0/10` — broader static-analysis program remains outside AF-01 and is retained for later assurance work.
- Fuzzing: `0/10` — explicitly retained for AF-02 adversarial test strength.
- Packaging: not applicable in this development view — stable release assurance belongs to AF-03.

These values are not converted into fictitious AF-01 PASS claims.

## T031 decision

`T031 = DISPOSITIONED`

Material Scorecard findings have explicit bounded treatment. Branch protection remains a hard Stack C blocker until T038–T040. Security-policy and dependency-update posture fixes are implemented and require final exact-head evidence. The two Java vulnerability findings remain visible and bounded to separate oracle dependency requalification rather than causing an unauthorized AF-01 semantic dependency mutation.
