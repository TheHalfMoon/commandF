# AF-01 Stack B Dependency Inventory and Policy Intent

Status: `T020_COMPLETE_POLICY_INPUT`

This document records the exact dependency/license/source evidence inspected before creating `deny.toml`. It is the policy input for AF-01 T021; it is not itself a scanner PASS.

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

The artifact was generated from the locked graph and uploaded by a full-SHA-pinned `actions/upload-artifact` action.

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

### Legacy metadata exception boundary

Two exact packages report the legacy non-SPDX expression `MIT/Apache-2.0` in Cargo metadata:

- `filetime@0.2.29`
- `version_check@0.9.5`

T021 must not convert this into a global acceptance rule. If cargo-deny 0.20.2 does not normalize the legacy expression itself, any remediation must be package-scoped and evidence-backed (for example, an exact package exception or license clarification tied to upstream license material). The policy must remain narrower than accepting arbitrary malformed license expressions.

## Duplicate/version inventory

The locked graph contains exactly three package names at multiple versions:

```text
getrandom: 0.2.17, 0.4.3
syn: 2.0.119, 3.0.3
windows-sys: 0.52.0, 0.61.2
```

These are current transitive graph facts, not silently approved permanent exceptions.

T021 policy intent:

- keep duplicate versions visible and machine-diagnosable;
- use `multiple-versions = "warn"` initially rather than creating opaque skip lists for the current graph;
- set wildcard dependency requirements to fail closed;
- do not add `skip` or `skip-tree` entries unless a later exact finding proves a narrow necessity and records a reason/revisit condition.

## Source policy intent

Observed source authority is only:

- local workspace/path packages; and
- crates.io via `https://github.com/rust-lang/crates.io-index`.

T021 must therefore enforce:

```text
unknown registry: deny
unknown git source: deny
allowed registry: crates.io only
allowed git sources: none
```

A future git dependency, alternate registry, or additional source authority must require a repository diff and explicit policy review; it must not be admitted by a wildcard source rule.

## Advisory policy intent

At the policy layer:

- RustSec advisories are not blanket-ignored;
- `advisories.ignore` starts empty;
- current/future advisory waivers are governed by T024 rather than handwritten anonymous ignore entries;
- yanked or vulnerable dependency evidence must remain visible to the scanner gates;
- scanner transport/tooling failures must be CI failures, not PASS-equivalent outcomes.

T023 remains responsible for an independent cargo-audit view so cargo-deny is not the only advisory signal.

## License policy intent

T021 should allow only the observed SPDX atoms listed above and use cargo-deny 0.20.2's exact configuration semantics. The policy must:

- avoid wildcard license approval;
- avoid ignoring private/workspace packages as a shortcut unless repository semantics require it;
- preserve `confidence-threshold = 0.8` unless exact scanner evidence justifies a narrower change;
- scope any legacy-license remediation to the exact affected package/version;
- fail on any new dependency whose license cannot be matched to the checked-in policy.

## T020 decision

`T020 = COMPLETE`

The current graph, source authority, license surface, and duplicate families are now documented from exact locked evidence. T021 may create `deny.toml` from this inventory. No cargo-deny, cargo-audit, advisory-waiver, or zizmor PASS is claimed by this document.
