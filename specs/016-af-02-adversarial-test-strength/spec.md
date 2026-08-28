# AF-02 Specification — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Identity

AF-02 is the second commandF Assurance Foundation unit. It measures whether commandF's existing deterministic product core fails safely under generated, malformed, mutated, adversarial, reordered, and retained-evidence inputs.

Spec Kit directory `016` is a planning sequence only. It does not rename or consume product identity `CF-16`.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
CF-13: CLOSED_CANONICAL
```

AF-02 does not authorize a CF-06 production-oracle change, a CF-10 corpus reinterpretation, or CF-14/15/16 implementation.

## Normative authority and precedence

AF-02 implementation and review MUST use this exact authority order:

1. canonical repository constitution/governance and `AGENTS.md`;
2. `verification-protocol.md` plus the machine-readable schemas under `schemas/`;
3. `evidence-contracts.md` for requirements not replaced by the closed protocol/schemas;
4. this `spec.md`;
5. `plan.md`;
6. `tasks.md`;
7. `consistency.md` and donor/provenance records.

`verification-protocol.md` is therefore part of the normative AF-02 authority set, not an optional addendum.

The earlier illustrative `commandf.af02-authority-baseline/v1` shape in `evidence-contracts.md` is **deprecated planning history** and MUST NOT be implemented. The single closed authority-baseline schema is:

```text
schemas/af02-authority-baseline-v2.schema.json
schema id: commandf.af02-authority-baseline/v2
```

The single closed proof schema is:

```text
schemas/af02-adversarial-proof-v1.schema.json
schema id: commandf.af02-adversarial-proof/v1
```

Where prose and a machine-readable schema disagree on type, required field, enum, pattern, cardinality, or unknown-field handling, the machine-readable schema is authoritative and the inconsistency fails planning/implementation qualification until reconciled.

## User problem

commandF already has deterministic example-based tests, fail-closed validators, real-FHIR smoke paths, oracle reconciliation, exact-head CI, and the canonical AF-01 trusted-development baseline. Those controls prove known scenarios and development-path integrity. They do not yet prove test adequacy against plausible defects and hostile input families.

A dangerous gap can produce:

- false compatible/non-breaking results;
- non-deterministic reports or fingerprints;
- acceptance of malformed or non-canonical retained evidence;
- path traversal or source escape;
- graph/reference ambiguity;
- retry-pass flakiness presented as green;
- logic mutations not detected by tests;
- producer-authored evidence that verifies itself;
- an assurance-policy change that weakens the verifier which judges the same change.

## Required outcome

AF-02 may close only after commandF has an independently executable adversarial evidence plane that:

1. deterministically discovers and classifies the required input/trust boundaries;
2. fuzzes raw and structured inputs with bounded, offline execution;
3. proves explicit properties with shrinking and retained reproducer identity;
4. promotes every discovered defect into deterministic regression evidence;
5. measures every mutation listed inside the frozen critical mutation scope, except exact previously canonical exclusions;
6. records coverage against a Git-derived production source universe and pre-frozen measurement descriptor;
7. treats retry-pass as failure;
8. verifies evidence with a base-controlled verifier rather than trusting candidate summaries;
9. preserves AF-01, CF-06, CF-10, and product semantics.

## Functional requirements

### FR-001 — deterministic adversarial surface inventory

AF-02 MUST maintain a machine-checkable surface policy over the exact Git-tracked Rust source universe defined by `verification-protocol.md`.

The scanner MUST cover parser/deserializer, archive/compression, filesystem/path, network/acquisition, cache/persistence, and subprocess boundaries under both:

```text
crates/**/src/**/*.rs
tools/**/src/**/*.rs
```

Every scanner finding has exactly one disposition: critical surface or reviewed exclusion. Unclassified discoveries and stale policy entries fail closed. Newly introduced matching boundaries fail until classified.

### FR-002 — isolated fuzz workspace and exact tool identity

Use a dedicated fuzz workspace isolated from normal product runtime dependencies.

Frozen initial identities:

```text
cargo-fuzz: 0.13.2
upstream: 984c861c8dfea28055254c5f1d2659ab2cd63f76
libfuzzer-sys: =0.4.13
arbitrary: =1.4.2
fuzz compiler: nightly-2026-08-25
normal product Rust: 1.97.1
```

Tool acquisition, source/release digest, installed executable SHA-256, compiler/target/features, registry checksums, and version output MUST be retained according to the tool-lock contract. Nightly is fuzz-only and cannot change product compatibility claims.

### FR-003 — raw and structure-aware fuzzing

Raw-byte fuzzing is required where malformed bytes are the security boundary, including archive/compression and JSON parsing.

Structured generation is required where raw bytes would mostly exercise trivial rejection, including Lockfile V2 graphs, canonical references, source-map paths/ranges, report/suppression/fingerprint combinations, and graph permutations.

Each target MUST execute under the exact resource/offline protocol. Missing or ambiguous runtime isolation is incomplete evidence, not PASS.

### FR-004 — independent cross-path/property models

Where independent models are practical, AF-02 MUST compare product behavior with a separately expressed model rather than calling the same implementation twice.

Initial model families include:

- Lockfile V2 graph validity/canonicalization;
- archive manifest inventory/count/order;
- portable-path containment and rejection;
- canonical-reference parsing/resolution outcomes;
- graph set/order invariants;
- quality-gate/fingerprint/suppression set and truth-table behavior;
- constructor/serializer/retained-validator agreement where paths are independently meaningful.

Unknown divergence fails until minimized and understood.

### FR-005 — property testing

Adopt:

```text
proptest = "=1.11.0"
```

Property configuration, case count, shrinking behavior, and model registry MUST be checked in before qualification. A counterexample MUST be retained as a deterministic reproducer before its defect can close.

### FR-006 — regression corpus and assertion binding

Every fuzz crash, invariant failure, unexpected acceptance, hostile-input timeout attributable to the input, or property counterexample MUST be minimized and promoted to the deterministic corpus before the fix closes.

Corpus entries MUST use synthetic or publicly redistributable non-PHI data. Default individual payload maximum is `256 KiB`; aggregate committed AF-02 corpus maximum is `8 MiB` unless a dedicated policy PR changes it first.

Every scenario MUST have exactly one assertion-registry entry and every assertion entry MUST map to exactly one scenario. Replay target, argv, parser identity, source/config digests, expected normalized outcome, fixture SHA-256, and raw replay result are machine-checkable.

### FR-007 — no-PHI/provenance gate

No PHI or patient-derived data may enter AF-02 source, corpora, artifacts, logs, or crash reports. Public artifacts with unclear redistribution terms are metadata-only until rights are established. Opaque crash artifacts are not uploaded unless provenance scanning classifies them safe.

### FR-008 — flaky-as-failure

Adopt:

```text
cargo-nextest: 0.9.143
upstream: 60fa45f638ffc3f35e74afa65737f45fcd32db2a
```

The canonical AF-02 invocation MUST include both:

```text
--retries 2
--flaky-result fail
```

Repository profile `ci` MUST also specify retries 2 and flaky-result fail. The deterministic isolated retry-pass fixture and JUnit/process-exit provenance protocol are frozen in `verification-protocol.md`. A test that fails then passes on retry remains a failed AF-02 run.

Canonical `cargo test --workspace --all-features --locked` remains mandatory and nextest is additive.

### FR-009 — coverage measurement

Adopt:

```text
cargo-llvm-cov: 0.9.0
upstream: be59056988acd54c7f984b7c85643daea3711b29
```

Before observing percentages, freeze the measurement descriptor: source/tree, compiler/tool binary, exact command, Cargo inputs, replay/property/test inputs, source-universe definition, and exact exclusions.

Coverage source authority is the Git tree, not the report producer. The authoritative line universe includes tracked Rust under both `crates/**/src/**` and `tools/**/src/**`, minus only exact previously canonical non-product exclusions. Missing, unknown, duplicate-normalized, or zero-total critical paths fail closed.

Floors are derived from measured integer covered/total values. A floor, source-scope, command, test selection, or exclusion change cannot self-green in the same candidate.

### FR-010 — targeted mutation adequacy with complete in-scope selection

Adopt:

```text
cargo-mutants: 27.1.0
upstream: 8ab1dc786a1f61a4e370416cc6c68b81a704e917
```

Stack C0 freezes exact target source paths and exact reviewed exclusions **before mutation execution**. The pinned cargo-mutants JSON listing is then authoritative:

> Every listed mutant whose source path is inside the frozen target scope is REQUIRED unless it matches exactly one previously canonical exact exclusion.

There is no top-N, percentage, operator preference, manual “important mutant” choice, or post-result selection.

Every required result MUST close as `KILLED` or a previously canonical narrow waiver. `TIMEOUT`, `UNVIABLE_OR_BUILD_FAILURE`, `SURVIVED`, and unclassified results are non-green until the required bounded retry/diagnosis and closure rules are satisfied.

### FR-011 — deterministic versus stochastic evidence

Deterministic qualification includes fixed corpus replay, property configuration/outcomes, nextest policy evidence, coverage, the frozen mutation inventory/results, canonical cargo test, authority projections, and required checks.

Stochastic fuzz discovery records bounded observations only. “No crash for N minutes” is never a correctness proof and never enters the deterministic proof digest.

### FR-012 — bounded CI topology

AF-02 separates:

1. deterministic PR qualification;
2. scheduled/manual bounded fuzz discovery;
3. targeted mutation execution;
4. final proof verification;
5. the base-controlled acceptance-authority verifier gate.

Every lane has explicit timeout/resource/artifact limits and least GitHub token permissions.

Heavy checks do not automatically become main required checks. Any live ruleset change follows the AF-01 topology/read-back process.

### FR-013 — independent base-verifier anchor

After Stack A0 becomes canonical, candidate-controlled workflow code MUST NOT be the only authority deciding whether a policy/verifier/enforcement change is acceptable.

The canonical design is a repository-owned `pull_request_target` base-verifier gate executing from canonical base workflow code with read-only permissions. It MUST:

- run the verifier from immutable canonical-base blob identities;
- obtain base and candidate trees into separate directories;
- never execute candidate code in the privileged/base-controlled job;
- treat candidate material only as data/evidence;
- trigger on every policy/schema/verifier/scanner/parser/result/workflow/enforcement-inventory path;
- record base workflow/verifier/enforcement-inventory blob SHAs and exact base/head identities;
- fail closed if base verifier cannot run or parse evidence.

Before A0 merge, the new gate must demonstrate universal terminal topology. If it is promoted into live required checks, the ruleset update/read-back occurs only after the check exists successfully on the exact A0 head.

### FR-014 — exact proof artifact

The final proof MUST validate against:

```text
schemas/af02-adversarial-proof-v1.schema.json
```

and the semantic invariants in `verification-protocol.md`.

The independent verifier reconstructs the deterministic proof object from raw evidence and computes:

```text
AF02_ADVERSARIAL_SHA256=<64 lowercase hex>
```

Producer-authored green summaries are not authority. Schema digest, raw evidence digests, source/tool identities, authority projections, corpus/assertions, nextest, coverage, mutation, cargo-test, and required-check provenance are bound into proof evidence.

### FR-015 — product and assurance non-regression

AF-02 MUST NOT weaken or reinterpret CF-03/04/05/06/07/09/10/11/12/13 semantics or AF-01 workflow/dependency/source-control policy. Minimal internal test seams are allowed only when public behavior/API remains unchanged and semantic regressions pass.

### FR-016 — reviewer truth

Every AF-02 planning/design/implementation stack requests CodeRabbit and Qodo when available. Reviewer timeout, quota, summary-only output, or unavailable service is recorded as such and never called PASS. Findings are dispositioned against the exact current head.

## Non-functional requirements

### NFR-001 — determinism

Canonical product output, corpus replay, property assertions, coverage accounting, mutation policy parsing, authority projections, and deterministic proof construction are reproducible from retained exact inputs.

### NFR-002 — fail closed

Missing required evidence, stale/unclassified boundary, invalid schema, unknown mutation outcome, missing replay, missing critical coverage, authority drift, verifier ambiguity, or incomplete resource/offline proof fails qualification.

### NFR-003 — bounded resources

Routine adversarial jobs stay within the checked-in `commandf.af02-resource-policy/v1` limits. A stress test outside those limits is a separate bounded scenario and cannot silently redefine normal qualification.

### NFR-004 — no hidden retries

Retries diagnose; they do not manufacture green.

### NFR-005 — no PHI

Synthetic/public conformance metadata only.

### NFR-006 — exact provenance

Every external tool, registry package, retained authority source, corpus fixture, workflow result, and proof artifact has an exact immutable identity appropriate to its channel.

## Planning closure rule

This package becomes canonical only when T006 completes on one exact final planning head: existing CI green, exact required-context provenance proven, Qodo and CodeRabbit findings dispositioned, merge guarded by expected head, and canonical post-merge main/tree plus live AF-01 rulesets re-read.

Only then is Stack A0 design freeze authorized. No fuzz/property/mutation implementation is authorized directly by this planning PR.
