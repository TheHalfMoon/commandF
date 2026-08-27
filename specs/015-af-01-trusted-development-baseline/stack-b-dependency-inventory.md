# AF-01 Stack B Dependency Inventory and Policy Intent

Status: `T020_COMPLETE_POLICY_INPUT`

This document records the exact dependency/license/source evidence inspected before creating `deny.toml`, plus the observed Stack B scanner calibration used to freeze the initial policies. It is not the final T028 exact-head qualification record.

## Canonical input

Stack A canonical base:

```text
main: 48587578e2d9167ac1c96b51c9942edb2aa74d8c
tree: 4f2ccef845321a7abba8ce5388e78281a1514436
Cargo.lock blob: 69ba1936596a1f3acfef2908fe659c6dc6fe474a
```

Inspection command:

```text
cargo metadata --locked --format-version 1 | python3 .github/scripts/summarize_cargo_metadata.py
```

Exact evidence run:

```text
head: 6f0fa6f2a344e7c6f9adbade2851b6aed3c4d5f9
workflow: af01-security
run: 33044282731
job: dependency-inventory
artifact: 9635015658
artifact name: af01-t020-dependency-inventory
artifact digest: sha256:370637ff98963b3804780bec3439275ab7c1ec80c63f123722d493fe2cd54247
```

The artifact was generated from the locked graph and uploaded by a full-SHA-pinned `actions/upload-artifact` action. That initial evidence established package/license/source counts; later reviewer hardening changed the inventory representation from manifest dependency names to exact Cargo resolved edges, so final T028 evidence must come from the hardened schema described below rather than treating this early artifact as final graph-edge proof.

## Exact graph summary

```text
packages total: 133
workspace packages: 2
crates.io packages: 131
other registry/git packages: 0
packages with unknown license metadata: 0
```

Workspace crates:

- `commandf`
- `commandf-pkg`

The current workspace direct dependency surface is declared in the root/member manifests and includes `clap`, `flate2`, `semver`, `serde`, `serde_json`, `sha2`, `tar`, `thiserror`, `ureq`, and `tempfile`, plus the local path dependency from `commandf` to `commandf-pkg`.

### Resolved-graph authority and reviewer remediation

Qodo identified two material correctness defects in the original T020 summarizer:

1. malformed `packages[].dependencies` records could be silently filtered from the output; and
2. dependency relationships were projected from manifest declarations and reduced to names, so two resolved versions of the same crate could not be distinguished on individual edges.

Both findings were remediated before T028 qualification. The inventory schema is now `2` and fails closed unless:

- every package and manifest dependency record has the required structure and string identity fields;
- `resolve.nodes` exists and contains one unique node for every package in the resolved graph;
- every resolved edge has a dependency name plus exact target package ID;
- every edge target resolves to a known package record;
- Cargo's `dependencies` and `deps[].pkg` resolved-node representations agree; and
- every package in the metadata package set is represented by a resolved node.

Each emitted dependency edge now records the dependency edge name plus the exact selected `package_id`, resolved package name, version, and source. This preserves distinctions such as `getrandom@0.2.17` versus `getrandom@0.4.3` instead of collapsing them to `getrandom`.

Regression coverage in `.github/scripts/test_summarize_cargo_metadata.py` includes exact multi-version edge preservation, malformed dependency-array/record/name rejection, unknown resolved target rejection, disagreement between Cargo's two resolved dependency representations, and missing resolved-node rejection. The suite is re-exported through the existing universal AF-01 workflow-trust unittest discovery surface so these regressions cannot be omitted while that gate remains authoritative.

The original Qodo threads became resolved/outdated only after the implementation changed. T029 still requires a fresh exact-head Qodo review; this remediation record is not a substitute for that review.

## Observed license expressions

The exact locked graph contains the following distinct dependency license expressions:

- `(MIT OR Apache-2.0) AND Unicode-3.0`
- `0BSD OR MIT OR Apache-2.0`
- `Apache-2.0 AND ISC`
- `Apache-2.0 OR ISC OR MIT`
- `Apache-2.0 OR MIT`
- `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`
- `BSD-3-Clause`
- `CDLA-Permissive-2.0`
- `ISC`
- `MIT`
- `MIT OR Apache-2.0`
- `MIT OR Apache-2.0 OR LGPL-2.1-or-later`
- `MIT OR Zlib OR Apache-2.0`
- `MIT/Apache-2.0`
- `Unicode-3.0`
- `Unlicense OR MIT`

The SPDX license atoms required by the observed graph are therefore narrowly bounded to:

```text
0BSD
Apache-2.0
Apache-2.0 WITH LLVM-exception
BSD-3-Clause
CDLA-Permissive-2.0
ISC
LGPL-2.1-or-later
MIT
Unicode-3.0
Unlicense
Zlib
```

No broader license family, wildcard, or blanket approval is authorized by this inventory.

### Legacy metadata boundary

Two exact packages report the legacy non-SPDX expression `MIT/Apache-2.0` in Cargo metadata:

- `filetime@0.2.29`
- `version_check@0.9.5`

`cargo-deny 0.20.2` accepted the locked graph under the narrow atom allowlist without a package-specific legacy-expression waiver. No global malformed-license exception or package exception was added.

The two workspace crates are unpublished (`publish = false`). `deny.toml` therefore ignores private workspace-crate license declarations while continuing to enforce every third-party dependency license. This is a repository-owned-boundary decision, not permission to ignore a private third-party registry.

## Duplicate/version inventory

The locked graph contains exactly three package names at multiple versions:

```text
getrandom: 0.2.17, 0.4.3
syn: 2.0.119, 3.0.3
windows-sys: 0.52.0, 0.61.2
```

These are current transitive graph facts, not silently approved permanent exceptions.

T021 policy intent and implemented boundary:

- keep duplicate versions visible and machine-diagnosable;
- use `multiple-versions = "warn"` initially rather than creating opaque skip lists for the current graph;
- keep external wildcard dependency requirements fail-closed;
- permit the repository-owned unpublished workspace path edge with `allow-wildcard-paths = true` because the local path requirement has no registry version requirement to pin;
- do not add `skip` or `skip-tree` entries unless a later exact finding proves a narrow necessity and records a reason/revisit condition.

`allow-wildcard-paths = true` does not authorize wildcard version requirements for crates.io, git, or alternate registries. Source policy remains independently fail-closed.

## Source policy intent

Observed source authority is only:

- local workspace/path packages; and
- crates.io via `https://github.com/rust-lang/crates.io-index`.

T021 therefore enforces:

```text
unknown registry: deny
unknown git source: deny
allowed registry: crates.io only
allowed git sources: none
```

A future git dependency, alternate registry, or additional source authority requires a repository diff and explicit policy review; it cannot be admitted by a wildcard source rule.

## Advisory policy intent

At the policy layer:

- RustSec advisories are not blanket-ignored;
- `advisories.ignore` starts empty;
- current/future advisory waivers are governed by T024 rather than handwritten anonymous ignore entries;
- yanked or vulnerable dependency evidence remains visible to the scanner gates;
- scanner transport/tooling failures are CI failures, not PASS-equivalent outcomes.

T023 supplies an independent `cargo-audit` view so cargo-deny is not the only advisory signal.

## License policy intent

T021 allows only the observed SPDX atoms listed above and uses cargo-deny 0.20.2's exact configuration semantics. The policy:

- has no wildcard license approval;
- ignores only the unpublished workspace crates as first-party license subjects while retaining third-party dependency checks;
- preserves `confidence-threshold = 0.8`;
- has no package/version license exception or skip list;
- fails a new dependency whose license cannot be matched to the checked-in policy.

## Stack B scanner calibration

The following run is calibration evidence, not the final T028 head qualification:

```text
PR head: 66ba48fa7aaa895c8b3cd3d7fefbb82ec55abac7
PR head tree: 53196a991a7ee579da0d72f4f0b8c5373ba7698a
GitHub PR merge-ref tree: 53196a991a7ee579da0d72f4f0b8c5373ba7698a
workflow: af01-security
run: 33050044070
```

The PR head and GitHub's temporary merge ref had the same tree, so the scanner inputs were byte-identical. The workflow was subsequently hardened to checkout and attest `AF01_SOURCE_SHA` explicitly because the original proof JSON labeled GitHub's temporary `GITHUB_SHA` as `head_sha`.

Observed calibration results:

- dependency inventory: `SUCCESS` under the earlier representation, superseded for final resolved-edge proof by schema 2;
- cargo-deny action `v2.1.1` at commit `3c6349835b2b7b196a839186cb8b78e02f7b5f25`, cargo-deny `0.20.2`: `SUCCESS` for advisories, bans, licenses, and sources;
- RustSec cargo-audit `0.22.2`: exit `0` against exact `Cargo.lock`;
- RustSec advisory database origin: `https://github.com/RustSec/advisory-db.git`;
- observed advisory database commit: `a7bfe16948bf6f3ee25bdee4822209f87da21b80`;
- observed `Cargo.lock` SHA-256: `0c58bb1b2a78ad5ed7e196ef20622fb5673536146ab6e0f787eb2d5f6517cf66`;
- zizmor action `v0.6.2` at commit `3dc1ecc9bcb9e94e9b2c709687979e1298497054`, zizmor `1.29.0`, `min-severity=medium`, online audits disabled: `SUCCESS` with no blocking medium/high finding;
- security waivers: zero.

This observed baseline freezes the initial zizmor gate at `medium`; it does not authorize lowering the threshold around a future finding. T026 therefore has no current zizmor high/medium finding to disposition. Any later finding must be fixed or explicitly dispositioned under the AF-01 waiver policy without silently weakening the gate.

## T020 decision

`T020 = COMPLETE`

The current graph, source authority, license surface, duplicate families, resolved-edge representation, fail-closed inventory validation, and initial scanner calibration are documented. T021–T027 implement the checked-in dependency/workflow security gates and waiver/coverage policy. Final Stack B PASS remains governed by T028 exact-head workflow evidence and T029 exact-head independent reviews plus canonical merge truth.
