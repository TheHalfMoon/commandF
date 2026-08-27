# AF-02 Normative Evidence Contracts

Status: PLANNING_CANDIDATE

This file is normative for AF-02 only where it is not superseded by the higher-precedence closed verification protocol and machine-readable schemas.

Authoritative precedence:

1. repository constitution/governance and `AGENTS.md`;
2. `verification-protocol.md`;
3. `schemas/af02-authority-baseline-v2.schema.json` and `schemas/af02-adversarial-proof-v1.schema.json`;
4. this file;
5. `spec.md`, `plan.md`, `tasks.md`, `consistency.md`, donor/provenance records.

If a lower-precedence document is looser or contradictory, the higher-precedence fail-closed rule controls. A future weakening requires a dedicated reviewed policy/verifier change under the previously canonical contract.

## 1. Canonical planning authority

Planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

External authorities are inputs, not AF-02-owned semantics.

### AF-01

Live rulesets:

```text
21652953 commandF main assurance
21652974 commandF main review governance
```

AF-02 reconstructs the closed semantic projections defined in `verification-protocol.md` from live GitHub ruleset read-back. The older illustrative semantic digests previously recorded in this file are superseded by the closed projection definitions in `verification-protocol.md`.

### CF-06

Frozen production identity unless separately changed canonically:

```text
project: hapifhir/org.hl7.fhir.core
release: 6.10.2
source_commit: d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
validator_cli_jar_sha256: a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6
R4_core_context: hl7.fhir.r4.core@4.0.1
```

The authority projector derives this from canonical-base repository source files named in the protocol; candidate AF-02 prose is never source authority.

### CF-10

Frozen deltas:

```text
C001 hl7.fhir.us.core   8.0.1 -> 9.0.0
C002 hl7.fhir.uv.ips    1.1.0 -> 2.0.1
C003 hl7.fhir.us.mcode  3.0.0 -> 4.0.0
```

Retained evidence:

```text
PR: 11
head: 5fe10d9859407272acf6649fc3e868d3eb2fbd12
base: 5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
run: 31916124080
run conclusion: failure
artifact_id: 9255732702
artifact_name: cf10-real-corpus-evidence
artifact_sha256: 9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612
```

The exact retained manifest/donor commit, path, Git blob SHA, and API locators are machine-readable in `retained-authority-sources.json`. Current `main` presence is not assumed. The retained failed run is never relabeled as CF-10 production success.

## 2. Authority baseline schema transition

The earlier planning-only `commandf.af02-authority-baseline/v1` shape is **deprecated and non-implementable**. It MUST NOT be accepted by AF-02 code.

The only implementation schema is:

```text
commandf.af02-authority-baseline/v2
schemas/af02-authority-baseline-v2.schema.json
```

The v2 schema structurally requires:

- canonical captured main SHA/tree;
- both AF-01 live ruleset projection identities;
- exact CF-06 projection plus authoritative source digests;
- exactly three CF-10 deltas;
- exactly six expanded CF-10 states;
- retained PR/head/base/run/conclusion/artifact id/name/digest;
- retained manifest/donor Git blob identities and derived raw SHA-256;
- CF-10 semantic projection digest.

Candidate editing of a baseline can never establish the authority it claims to describe.

## 3. Surface discovery contract

AF-02 uses one deterministic AST discovery model, not an implementation-time choice between scanners.

The Git-derived Rust source universe is:

```text
crates/**/src/**/*.rs
tools/**/src/**/*.rs
```

minus only exact previously canonical reviewed non-product exclusions.

The scanner uses `syn=3.0.3` with locked registry checksum and frozen features `[full, visit]`. It parses every source-universe file and visits production-tracked syntax regardless of cfg/dead-code reachability. Comments and string literals do not create executable matches. Alias/import handling, conservative macro handling, uncertain method-call handling, and stale-entry behavior are specified in the protocol.

Frozen boundary categories:

```text
SERDE_OR_TEXT_PARSE
ARCHIVE_OR_COMPRESSION
FILESYSTEM
NETWORK_OR_ACQUISITION
CACHE_OR_PERSISTENCE
SUBPROCESS
```

Every finding has exactly one disposition:

```text
CRITICAL_SURFACE:<id>
REVIEWED_EXCLUSION:<id>
```

Unclassified, multiply classified, stale, or unresolved source entries fail.

## 4. Resource/offline contract

`commandf.af02-resource-policy/v1` is executable policy. It includes at minimum:

```text
campaign_wall_seconds
max_executions_or_zero_if_time_bounded
per_input_timeout_seconds
max_input_bytes
process_memory_mib
cpu_count
pids_limit
tmpfs_mib
max_decompressed_or_generated_bytes
max_temporary_files
subprocess_timeout_seconds
max_single_artifact_bytes
max_total_artifact_bytes
max_committed_corpus_bytes
artifact_retention_days
offline_required
```

Canonical deterministic qualification uses the digest-pinned Linux OCI image and exact runtime enforcement from the protocol. Network-enabled acquisition and network-denied execution are separate phases.

A required run with missing/ambiguous network, cgroup, tmpfs, mount, timeout, or negative-probe evidence is incomplete, never green.

## 5. Tool-lock contract

Every executable AF-02 tool uses one of:

```text
LOCKED_GIT_REV_SOURCE_BUILD
IMMUTABLE_RELEASE_ASSET_WITH_SHA256
```

Executable evidence records:

```text
id
version
upstream_repository
upstream_commit
acquisition_mode
source_lock_sha256_or_release_asset_sha256
installed_executable
installed_executable_sha256
version_output_sha256
build_rustc
build_cargo
build_target
features[]
```

Registry/test packages record exact package, version, registry checksum and features. Upstream commit alone is never treated as an installed binary identity.

Initial tools remain:

```text
cargo-fuzz 0.13.2 @ 984c861c8dfea28055254c5f1d2659ab2cd63f76
cargo-nextest 0.9.143 @ 60fa45f638ffc3f35e74afa65737f45fcd32db2a
cargo-llvm-cov 0.9.0 @ be59056988acd54c7f984b7c85643daea3711b29
cargo-mutants 27.1.0 @ 8ab1dc786a1f61a4e370416cc6c68b81a704e917
proptest =1.11.0
libfuzzer-sys =0.4.13
arbitrary =1.4.2
nightly-2026-08-25 fuzz-only
product Rust 1.97.1
```

## 6. Property/model contract

Each property family records stable property id, surface id, generator model, validity domain, invalidity mutations, collection/depth limits, case count, seed/shrink policy, expected model, and independent oracle/model identity.

Initial independent models cover archive manifest contents, Lockfile graph/canonical order, portable paths, canonical-reference resolution, graph ordering, and gate/fingerprint/suppression set/truth-table behavior.

Calling the same product implementation twice is not independent evidence.

## 7. Corpus and replay contract

`commandf.af02-corpus/v1` contains stable scenario IDs, exact fixture path/SHA-256/bytes/provenance/expected outcome, assertion id, replay identity, discovery origin, and minimization lineage.

Default promoted fixture maximum: `256 KiB`.
Aggregate committed AF-02 corpus maximum: `8 MiB`.

No PHI or patient-derived data may enter source, corpus, artifacts, logs, or crash retention. Public-source/license ambiguity is metadata-only until resolved.

`commandf.af02-assertion-registry/v1` is bijective with the corpus. Exact runner kind, package/binary, target/test, argv, cwd, environment allowlist, parser, source/config digests and expected normalized outcome are bound. Shell command strings are not authority.

## 8. Deterministic outcome contract

Allowed normalized deterministic surface outcomes:

```text
ACCEPT_CANONICAL
REJECT_INVALID
FAIL_CLOSED_LIMIT
UNEXPECTED_ACCEPTANCE
INVARIANT_VIOLATION
ORACLE_DIVERGENCE
PANIC_OR_ABORT
HARNESS_TIMEOUT
HARNESS_MEMORY_LIMIT
HARNESS_FILESYSTEM_LIMIT
HARNESS_PROCESS_LIMIT
HARNESS_INTERNAL_ERROR
```

Only outcomes explicitly allowed by the surface policy can be green. Any unexpected acceptance, invariant violation, oracle divergence, panic/abort, or unresolved harness/resource failure blocks qualification and requires diagnosis; discovered defects require deterministic minimized replay before closure.

## 9. Nextest contract

Pinned nextest is `0.9.143`.

Both config and command line enforce:

```text
retries = 2
flaky-result = fail
--retries 2
--flaky-result fail
```

The canonical isolated fixture is defined in the protocol. The runner proves its dedicated output mount clean before execution and binds the newly created JUnit file, stdout, stderr and process exit into one waited-for process envelope. A retry-pass result is `FLAKY_RETRY_PASS` and the process must be non-zero.

## 10. Coverage contract

Coverage uses a descriptor frozen before observing percentages. Source authority is the same Git-derived Rust universe used by surface discovery:

```text
crates/**/src/**/*.rs
tools/**/src/**/*.rs
```

minus exact previous exclusions only.

Every authoritative source path appears exactly once in the raw report, including zero-hit files. Missing, unknown, duplicate-normalized, out-of-root or zero-total critical scope fails.

Workspace and critical-surface floors use checked integer arithmetic:

```text
floor_percent = (covered * 100) // total
```

Floor/scope/exclusion/command/test-selection weakening cannot self-green in the same candidate.

## 11. Mutation contract

Before listing/execution, C0 freezes exact target paths, exact exclusions, tool lock, command/config and timeout/test authority.

Then:

```text
required = every cargo-mutants-listed mutant inside frozen target paths
           minus exact pre-frozen reviewed exclusions
```

No top-N, percentage, operator priority, security-interest subset, or post-result manual selection exists.

Result classes remain separate:

```text
KILLED
SURVIVED
TIMEOUT
UNVIABLE_OR_BUILD_FAILURE
WAIVED_EQUIVALENT_OR_OUT_OF_SCOPE
```

Required `SURVIVED`, `TIMEOUT`, `UNVIABLE_OR_BUILD_FAILURE`, or unclassified results are non-green. Retry/diagnosis is mandatory. A waiver used for qualification must already be canonical before the implementation candidate.

## 12. Proof and anti-forgery contract

The only structural proof schema is:

```text
commandf.af02-adversarial-proof/v1
schemas/af02-adversarial-proof-v1.schema.json
```

The schema rejects unknown fields and freezes scalar types, nullability, identity/digest patterns, enum values, cardinalities, conditional tool shapes, fixed green-state fields and bounded counters.

`verification-protocol.md` supplies semantic invariants not safely expressible in JSON Schema: ordering/uniqueness, arithmetic relations, exact required-check membership, cross-artifact digest relationships, path containment and base/candidate authority rules.

The independent verifier reconstructs deterministic evidence from raw artifacts. Producer-created normalized summaries are never source authority.

After A0 canonicalization, all policy/schema/verifier/scanner/parser/result/workflow/enforcement-inventory changes are judged by a canonical-base-controlled verifier execution anchor. Candidate workflow code cannot choose, replace, skip or relabel that base verifier.

## 13. Stochastic discovery contract

Stochastic fuzzing is observational only. Allowed classes:

```text
NO_CRASH_OBSERVED_WITHIN_BOUND
DEFECT_DISCOVERED
INCOMPLETE_RESOURCE_OR_HARNESS_FAILURE
CANCELLED_OR_SUPERSEDED
```

Stochastic fields are outside `AF02_ADVERSARIAL_SHA256`. “No crash within bound” is never a correctness PASS.

## 14. Planning/implementation ordering

Planning T006 must be canonical before A0.
A0 must be canonical before A1.
A1 must be canonical before B0.
B0 must be canonical before B1.
B1 must be canonical before C0.
C0 must be canonical before C1.
C1 must be canonical before final convergence.

Every stack uses exact-head CI, fresh reviewers when available, zero unresolved substantive findings, expected-head guarded merge and post-merge authority read-back. No later head inherits earlier-head qualification.
