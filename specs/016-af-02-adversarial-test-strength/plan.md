# AF-02 Plan — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Entry condition

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

AF-02 implementation starts only after this planning package is exact-head qualified, independently reviewed, merged to canonical `main`, and post-merge authority is re-read.

The first authorized implementation unit after planning closure is **Stack A0 design freeze**. No fuzz/property result may be used for closure before A0 is canonical.

## Normative contract set

Implementation follows the precedence in `spec.md`:

1. repository governance/constitution/`AGENTS.md`;
2. `verification-protocol.md` and machine-readable schemas;
3. `evidence-contracts.md` for non-superseded requirements;
4. `spec.md`;
5. this plan;
6. `tasks.md`;
7. consistency/donor/provenance records.

The old illustrative authority-baseline v1 shape in `evidence-contracts.md` is not implementable authority. The closed baseline is `commandf.af02-authority-baseline/v2` in `schemas/af02-authority-baseline-v2.schema.json`.

## Architecture summary

AF-02 adds an assurance/evidence plane around the existing commandF Rust product core. It adds no new user-facing semantic engine and does not change CF-06/CF-10 authority.

Evidence classes remain separate:

1. deterministic regression/property evidence;
2. deterministic coverage evidence;
3. deterministic frozen-scope mutation evidence;
4. stochastic fuzz-discovery observations.

The final verifier reconstructs deterministic evidence from raw inputs. Stochastic observations are retained but excluded from `AF02_ADVERSARIAL_SHA256`.

## Existing high-risk boundary inventory

### Archive/package ingestion

`crates/commandf-pkg/src/archive.rs` handles gzip/tar input, manifest limits, archive-entry limits, decompressed scan limits, and normalized package paths. Fuzzing enters through an existing public product seam and does not publish private API merely for fuzzing.

### Lockfile retained evidence

Lockfile V1/V2 parsing/canonicalization/validation is a critical structured-property and mutation surface because malformed persisted evidence must not be silently normalized into authority.

### Source mapping and portable paths

CF-09 source mapping enforces source-root containment, portable relative paths, bounded report/input size, line ranges, duplicate-output rejection, and report consistency. AF-02 reaches the narrowest existing public or internal test seam; no public API is added solely for fuzz tooling.

### Context graph and canonical references

Graph construction consumes verified lock/cache evidence and canonical references, then sorts/deduplicates nodes and edges. Important properties are order independence and explicit resolved/ambiguous/external status.

### Compatibility/check/gate evidence

Canonical fingerprints, baseline/suppression membership, report validators, and quality-gate decisions are false-PASS-critical surfaces. Properties cover JSON key ordering, set ordering, evaluate-then-validate agreement, and tamper rejection.

### Acquisition/cache/subprocess boundaries

Registry responses, local mirrors, cache reads/writes/reuse, archive scans, and external-process boundaries belong to the deterministic source scanner inventory even when a particular fuzz target uses an offline seam.

## Frozen initial tool identities

```text
cargo-fuzz 0.13.2
  upstream 984c861c8dfea28055254c5f1d2659ab2cd63f76

libfuzzer-sys =0.4.13
arbitrary =1.4.2
fuzz compiler nightly-2026-08-25

proptest =1.11.0

cargo-nextest 0.9.143
  upstream 60fa45f638ffc3f35e74afa65737f45fcd32db2a

cargo-llvm-cov 0.9.0
  upstream be59056988acd54c7f984b7c85643daea3711b29

cargo-mutants 27.1.0
  upstream 8ab1dc786a1f61a4e370416cc6c68b81a704e917

surface parser: syn =3.0.3
  source registry+https://github.com/rust-lang/crates.io-index
  features [full, visit]

normal product Rust 1.97.1
```

Every executable tool uses the immutable acquisition models in the tool-lock contract and records installed binary digest plus compiler/target/features. Exact registry checksums are read from canonical locked metadata and retained before use.

## Retained authority source resolution

CF-10 authority is not expected to exist on current `main`. AF-02 explicitly references retained source blobs through:

```text
retained-authority-sources.json
```

That file binds repository, retained commit, path, Git blob SHA, PR/base/run/artifact identities, and API locators. The authority projector fetches those exact retained blobs by commit/blob identity and computes raw SHA-256 itself. Hard-coded case values alone are not sufficient.

CF-06 is reconstructed from canonical-base `oracle_model.rs`, donor metadata, and `cf06-oracle.yml`. AF-01 is reconstructed from the two live ruleset endpoints.

## Stack A0 — design freeze and base-controlled enforcement

A0 is policy/verifier infrastructure only. It MUST NOT use newly generated fuzz/property results as evidence that A0 itself is correct.

### A0.1 Authority baseline v2

Create machine-readable `commandf.af02-authority-baseline/v2` validated by `schemas/af02-authority-baseline-v2.schema.json`.

It records independently derived:

- AF-01 assurance/review projection digests;
- CF-06 exact projection and authoritative source digests;
- CF-10 exactly three deltas and six states;
- retained PR/head/base/run/conclusion/artifact id/name/digest;
- retained manifest/donor Git blob identities and derived raw SHA-256.

Candidate-edited baseline values never establish authority.

### A0.2 Deterministic surface policy

Implement one `syn=3.0.3` AST scanner over Git-tracked `.rs` source under:

```text
crates/**/src/**
tools/**/src/**
```

The protocol freezes alias/macro/comment/string/cfg/dead-code and stale-entry semantics. Every finding gets exactly one disposition. The scanner dependency checksum, scanner source, matcher policy, source universe, and raw/classified inventories are evidence.

### A0.3 Resource/offline policy

Freeze checked-in campaign/per-input memory/CPU/PID/tmpfs/generated/decompressed/temp-file/subprocess/artifact/corpus/retention limits.

Canonical proof execution uses the digest-pinned Linux Rust container with network none, read-only root/source, dedicated writable output, cgroup limits, tmpfs, runtime inspection, and negative network/write probes. “Equivalent mechanism” is not canonical proof authority.

### A0.4 Tool lock

For each executable, freeze one acquisition mode:

- locked exact-revision source build; or
- immutable release asset with verified SHA-256.

For crates, retain exact registry package/version/checksum. Record commands, compiler, cargo, target, features, executable SHA-256, and version-output digest.

### A0.5 Property/model design

Freeze independently expressed model registries for archive manifest behavior, Lockfile graph/canonicalization, portable paths, canonical-reference resolution, graph ordering, and gate/fingerprint/suppression truth tables.

### A0.6 Corpus/assertion design

Freeze corpus manifest and `commandf.af02-assertion-registry/v1` before discovery results. Each scenario/registry entry is bijective and binds runner kind, target, exact argv, expected normalized outcome, parser, source/config digests, provenance, and fixture digest.

### A0.7 Proof schemas

Adopt the machine-readable closed schemas:

```text
schemas/af02-authority-baseline-v2.schema.json
schemas/af02-adversarial-proof-v1.schema.json
```

Schema files and SHA-256 are mandatory proof contract files. Repository-owned semantic validation adds cross-field/arithmetic/path relations that JSON Schema alone cannot express.

### A0.8 Base-verifier execution anchor

A0 introduces a base-controlled GitHub Actions gate using `pull_request_target` semantics so candidate workflow code cannot decide whether its own verifier weakening is accepted.

The base-controlled job MUST:

- execute workflow/script/verifier blobs from the PR canonical base, not candidate copies;
- use read-only GitHub permissions;
- check out base and candidate into separate directories with credentials disabled;
- never execute candidate code in the base-controlled job;
- treat candidate policy/evidence as data only;
- record exact base workflow, verifier, schema, and enforcement-inventory blob SHAs;
- trigger for every AF-02 policy/schema/verifier/scanner/parser/result/workflow/inventory path;
- fail if the base verifier cannot run or parse candidate evidence.

A0 first proves this check has universal terminal topology. Only then may a separate live-policy reconciliation add the new check to the AF-01 assurance ruleset. Read-back is mandatory before A0 merge if promoted.

## Stack A1 — fuzz/property/regression implementation

A1 starts only after A0 is canonical.

### Fuzz workspace

Expected layout:

```text
fuzz/
  Cargo.toml
  rust-toolchain.toml
  fuzz_targets/
  corpus/
  artifacts/   # generated, ignored
```

The nightly/libFuzzer workspace remains isolated from normal product dependency authority.

### Required reachability

Initial target families cover package archive bytes, Lockfile JSON/structured forms, retained report evidence, context graph/reference shapes, and source-map/path semantics.

One raw fuzzer plus properties may jointly cover a surface when that exercises the real public/internal seam more strongly than forcing every path through libFuzzer.

### Harness bounds

Routine raw input cap is `1 MiB` unless a narrower surface limit applies. Structured collection/depth sizes are small and policy-bound. Production maximums implying routine multi-hundred-MiB allocations are not normal fuzz iteration sizes.

Every run uses A0's OCI/offline/resource runner.

### Regression promotion

Every crash/invariant failure/unexpected acceptance/property counterexample is minimized and committed only when provenance/no-PHI rules permit. Default fixture <=256 KiB and aggregate committed corpus <=8 MiB.

A1 cannot close while a discovered defect lacks a deterministic assertion/replay entry.

## Stack B0 — flaky/coverage design freeze

B0 contains policy/fixture/parser design only.

### Nextest fixture

Freeze the isolated fixture specified by `verification-protocol.md`:

```text
tests/assurance/af02-nextest-flake-fixture/
selected test: af02_retry_pass_is_failure
```

Invocation forces `--retries 2 --flaky-result fail` so per-test overrides cannot weaken the result.

The JUnit output root is an **empty dedicated output mount created by the base-controlled runner**. Before nextest:

- parent exists with mode 0700 and expected unprivileged UID/GID;
- target JUnit path does not exist;
- no path component is symlink;
- output mount is empty except runner-created directories.

After nextest:

- JUnit path is a regular non-symlink file owned by the expected UID;
- it resides on the dedicated output mount;
- link count is one;
- the base-controlled runner opens/hashes it directly after process wait;
- no wrapper-supplied alternate path is accepted;
- raw stdout/stderr/exit and JUnit hash are captured together in the same runner result envelope.

The state-file protocol proves deterministic first-fail/retry-pass behavior. JUnit must contain the selected testcase's flaky retry history and process exit must remain non-zero.

### Coverage descriptor

Before measuring percentage, freeze exact source/tree, Linux/x86_64, Rust/tool identity, command, Cargo inputs, replay/property inputs, and exclusions.

Coverage source universe matches surface source universe for tracked Rust production paths:

```text
crates/**/src/**/*.rs
tools/**/src/**/*.rs
```

Only exact previously canonical non-product exclusions can remove a path. Every production source appears exactly once in the report, including zero-hit files. Missing/unknown/duplicate paths fail.

## Stack B1 — flaky/coverage execution

After B0 canonical:

- run canonical `cargo test --workspace --all-features --locked`;
- run nextest ordinary suite and retry-pass self-test;
- collect coverage from the frozen command/input set;
- derive workspace and each critical-surface floor independently from exact integer covered/total pairs;
- fail on any floor decrease, missing path, descriptor drift, or same-candidate scope weakening.

A coverage-policy/floor/exclusion change must be a dedicated policy-only PR evaluated under the prior policy; it cannot modify product source, tests, or the measurement command. A later candidate adopts the new baseline.

## Stack C0 — mutation/proof design freeze

C0 freezes mutation target source paths and exact exclusions **before listing or executing mutants**.

### Complete required mutant set

Run pinned cargo-mutants JSON listing on the exact tree and target paths. Every listed mutant in that scope is REQUIRED unless it matches exactly one pre-frozen exact exclusion.

There is no “choose required mutants” step after listing. There is no percentage, top-N, operator preference, or post-result manual subset.

Stable mutant IDs bind source blob/span/function/diff/tool-lock/policy identity. Duplicate/missing dispositions fail.

### Proof schema and semantic validator

The machine schema defines field types/nullability/patterns/enums/cardinality/conditional shapes. Repository semantic validation freezes:

- counter arithmetic/relations;
- required-check exact membership and success;
- digest/source relationships;
- sorted/unique set identities;
- path normalization/containment;
- tool-lock conditional executable versus registry shapes;
- inventory-object shapes referenced by digest;
- authority/proof/raw-evidence cross-links.

Malformed type/format/range/conditional shape/cross-field relation negative fixtures are required before C0 closes.

## Stack C1 — mutation execution and final proof

After C0 canonical:

- execute every required mutant;
- keep KILLED/SURVIVED/TIMEOUT/UNVIABLE_OR_BUILD_FAILURE separate;
- retry and diagnose TIMEOUT/UNVIABLE required results;
- close every required result as KILLED or a previously canonical exact waiver;
- run stochastic campaigns only as separately labeled observations;
- reconstruct final deterministic proof from raw evidence;
- validate JSON Schema and semantic invariants;
- recompute `AF02_ADVERSARIAL_SHA256` independently;
- prove required check uniqueness/provenance on exact head.

No producer-created normalized outcome or summary is trusted without independent reconstruction.

## CI and path topology

Each AF-02 workflow/action/script/config is included in AF-01 workflow-trust/path auditing.

Expensive lanes use explicit timeouts and resource policy. Path-skipped work must produce explicit terminal neutral/success topology only where the canonical gate contract defines it; absence of a required result is never silently interpreted green.

The base-verifier gate is special: after A0 canonicalization its execution anchor comes from canonical-base `pull_request_target` workflow code, not the candidate workflow copy.

## Closure and convergence

AF-02 reaches `CLOSED_CANONICAL` only after:

1. A0, A1, B0, B1, C0, and C1 each become canonical in dependency order;
2. every required corpus/property/nextest/coverage/mutation/proof gate passes on exact final head;
3. AF-01/CF-06/CF-10 authority is re-derived and unchanged;
4. required checks retain exact GitHub Actions provenance and uniqueness;
5. Qodo and CodeRabbit substantive findings are closed or explicitly recorded unavailable without inventing PASS;
6. final convergence document records exact final head/tree, runs/checks/artifacts, live policy read-back, waivers, and residual risks;
7. final PR merges with expected-head guard;
8. canonical post-merge main/tree and live rulesets are re-read.

AF-02 closure does not authorize a CF-06 production pin change or merge blocked CF-10 work.
