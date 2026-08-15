# CF-11 Convergence — Multi-Version Package Graph

Status: foundation behavior proven; reviewer findings reconciled; final convergence-head gates pending

## Decision

```text
CF-11_FOUNDATION_BEHAVIOR_PROVEN_PENDING_FINAL_EXACT_HEAD_GATES
```

CF-11 corrects only the package-closure identity model. It does not reinterpret compatibility, policy, terminology, oracle, source attribution, or CF-10 corpus semantics.

## Canonical base and implementation identity

```text
repository: TheHalfMoon/commandF
PR: #13
base: main
canonical base: 4c72f4a21aca757fbdadd2fe34384b8d0c746b85
branch: fix/cf-11-multi-version-package-graph
resolver implementation evidence head: 7411cebaa3052ccd71e83a916eb8d02e8269912c
proof/reviewer-hardening evidence head: 744a64c7fcd84961aed9ce0417d443129f230541
```

The resolver changes the selected-closure key from package name alone to exact `(package name, concrete version)` identity. Same exact identities deduplicate; different concrete versions of the same name remain distinct closure nodes. Lock schema v1 is unchanged and retains manifest-declared dependency constraints rather than claiming explicit resolved edges.

## Synthetic regression evidence

The workspace regression suite proves:

- two branches may retain `acme.dep@1.0.0` and `acme.dep@2.0.0` simultaneously;
- repeated requests resolving to the same exact identity produce one locked package;
- exact and patch-wildcard requests for the same package name can resolve to distinct versions;
- equivalent root-order permutations produce byte-identical synthetic lockfiles;
- exact-identity cycles terminate through deduplication;
- existing stable-patch selection and cache digest verification remain intact.

The byte-identical statement above is intentionally limited to deterministic synthetic sources. It is not imposed on independent real-registry acquisitions because real lock provenance records the actual validated transport URL.

## Real frozen-state foundation proof

CF-10 remains frozen. CF-11 reused one of its previously ineligible states without replacing or cherry-picking the case:

```text
state: C002-ips-after
package: hl7.fhir.uv.ips@2.0.1
prior CF-10 failure: hl7.terminology.r4 selected 7.2.0, requested 7.1.0
```

Final proof/reviewer-hardening evidence on head `744a64c7fcd84961aed9ce0417d443129f230541`:

```text
workflow: cf11-multi-version-proof
run: 31858847516
result: SUCCESS
artifact id: 9239883413
artifact digest: sha256:7770c8f6e1f78b37279efaf69a1e938596516c9310df6583860108d2c85be21c
```

The workflow performed two independent clean `pkg resolve` + `pkg verify` runs and required identical deterministic semantic lock identity across the two resolutions:

```text
roots
(name, version, sha256, declared dependencies)
```

Transport provenance is kept explicit for every locked package in both resolutions but is not used as a cross-run equality requirement. `LockedPackage.source` records the actual validated acquisition URL, which may legitimately differ if registry fallback or an accepted redirect is used. The final evidence artifact records both resolution package sets with exact `name`, `version`, `source`, `sha256`, and declared dependencies. In the observed final evidence run, `transport_provenance_identical` was also `true`, but that observation is not elevated into a convergence requirement.

The same-name multi-version closure contained:

```text
hl7.fhir.uv.extensions.r4: 5.2.0, 5.3.0
hl7.terminology.r4:        6.2.0, 7.1.0, 7.2.0
```

This proves the old name-level flattening was insufficient for this frozen real graph. It does not claim every FHIR package graph is supported.

## Reproducible proof environment

The real proof executes inside an immutable digest-pinned Rust container rather than relying on the mutable GitHub runner image as the execution environment:

```text
container: docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f
machine: x86_64
rustc: rustc 1.97.1 (8bab26f4f 2026-07-14)
cargo: cargo 1.97.1 (c980f4866 2026-06-30)
```

The workflow also uses immutable action SHAs, `persist-credentials: false`, and path coverage for resolver, lock, cache, registry/source, CLI, Cargo inputs, CF-11 specs, donor metadata, and the proof workflow itself.

## Exact proof/reviewer-hardening regression gates

Head `744a64c7fcd84961aed9ce0417d443129f230541` passed:

```text
ci                         31858847463  SUCCESS
cf06-oracle                31858846042  SUCCESS
cf11-multi-version-proof   31858847516  SUCCESS
```

The mainline workflow passed Format, locked Clippy with `-D warnings`, full workspace tests, CF-08 and CF-09 security regressions, real FHIR resolve/verify + inspect/diff/classify/check, terminology smoke, CF-09 fixture preparation, local composite Action source-map self-check, and output verification.

The dedicated oracle workflow passed the pinned HL7 adapter build, real R4 context, self-equivalence, `commandf oracle` self-diff, invalid-snapshot and corrupted-cache fail-closed gates, changed-profile determinism, and end-to-end reconciliation.

## Downstream ambiguity boundary

CF-11 does not add a name-only version choice rule. Existing commands that request one locked package by name and encounter multiple locked versions MUST fail closed. Exact `inspect name@version` selection remains exact.

Terminology canonical ambiguity and duplicate protections are unchanged.

## CF-10 boundary

CF-10 / PR #11 remains frozen and `BLOCKED_BY_FOUNDATION` until CF-11 is canonical. No CF-10 case may be replaced merely because the previous resolver rejected it.

After CF-11 becomes canonical, the first CF-10 action must be to reconcile onto the new main and rerun the exact same six frozen package states. Semantic diff/classify/check/terminology/oracle execution remains blocked until that eligibility rerun establishes the new foundation state.

## Reviewer truth and dispositions

- **Codex Code Review:** reviewed resolver implementation head `7411cebaa3052ccd71e83a916eb8d02e8269912c` and reported: `Didn't find any major issues.` This is positive implementation-head review evidence; it is not represented as a separate approval state.
- **Qodo:** returned one substantive Medium finding: the original real proof compared complete lockfile bytes even though `LockedPackage.source` can legitimately vary with validated registry fallback/redirect transport. This finding was accepted. The proof was changed to compare deterministic semantic lock identity while preserving complete per-resolution source and digest provenance. Qodo marked the original thread resolved/outdated after the correction.
- **CodeRabbit:** returned actionable findings on proof path coverage, immutable execution environment, explicit source/digest evidence, mandatory downstream ambiguity wording, actual proof-state documentation, and full-SHA documentation hygiene. The valid findings were implemented. A later suggestion to fail whenever transport source URLs differ was intentionally not adopted because it would recreate the Qodo-identified flakiness and incorrectly turn transport-route equality into package-identity semantics. That thread was answered with exact run/artifact evidence and resolved explicitly.
- **Greptile:** exact-head review was requested; no returned substantive result was observed. No PASS is claimed.
- **Cubic:** generated PR summaries only; not treated as correctness certification.

Reviewer absence is recorded rather than replaced with invented approval. A new substantive reviewer finding after this convergence update reopens the merge gate.

## Research inputs explicitly outside CF-11

Recent research/donor inputs — CPGPrompt, PathWISE, and `reason-healthcare/rh-skills` — are not dependencies of this foundation correction and do not modify its acceptance criteria. They belong to later research/benchmark/clinical-knowledge roadmap work after the package/corpus foundation is stable.

## Final convergence-head rule

This final convergence reconciliation changes documentation only. Its resulting repository head MUST pass all three configured CF-11 gates again:

```text
ci
cf06-oracle
cf11-multi-version-proof
```

The final convergence-head SHA and final run ids are recorded in PR #13 metadata/body after those workflows settle. A failure or new substantive unresolved review thread reopens convergence. This document intentionally does not self-reference its own future commit SHA/run ids.
