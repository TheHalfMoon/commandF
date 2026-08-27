# AF-02 Specification — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Identity and authority

`AF-02` is the second commandF Assurance Foundation unit. Spec directory sequence `016` is only repository ordering; it does not rename or consume product identity `CF-16`.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
CF-13: CLOSED_CANONICAL
```

AF-02 does not authorize CF-14/15/16 implementation, a CF-06 production-oracle pin change, a CF-10 corpus change, or weakening AF-01 live source-control policy.

## Normative document set

The AF-02 authority set is:

- `spec.md` — required outcomes and boundaries;
- `evidence-contracts.md` — **normative executable evidence schemas, algorithms, limits, authority snapshots, and anti-forgery rules**;
- `plan.md` — implementation sequence and design-freeze ordering;
- `tasks.md` — repository execution gates;
- `consistency.md` — architecture/review reconciliation;
- `donors/af-02-adversarial-testing.yaml` — exact donor/tool provenance and acquisition modes.

Where wording differs, the stricter fail-closed rule applies. `evidence-contracts.md` intentionally removes implementation discretion for proof canonicalization, resource/offline limits, surface discovery, corpus binding, tool provenance, coverage/mutation semantics, flaky retry behavior, authority preservation, and no-PHI artifact handling.

No implementation task may begin before planning gate T006 is canonical.

## User problem

commandF already has deterministic tests, fail-closed validators, exact-head CI, security gates, and protected `main`. That proves known behavior and trusted development flow; it does not prove that the tests detect hostile inputs, plausible logic mutations, hidden nondeterminism, evidence tampering, path escapes, retry-only passes, or newly introduced unclassified input boundaries.

The dangerous outcome is not only a crash. A test gap can produce a false compatible result, false green quality-gate evidence, non-canonical persisted evidence, source escape, ambiguous graph resolution, flaky retry-pass, or a plausible mutant surviving unnoticed.

## Outcome

AF-02 closes only when commandF has an independently executable adversarial-testing evidence layer that:

1. deterministically discovers and classifies critical external/untrusted input boundaries;
2. uses exact, independently verified tool/package identities;
3. fuzzes raw and structured inputs within executable resource/offline limits;
4. expresses important invariants with property tests and independent models/oracles;
5. promotes discovered failures to minimized deterministic corpus regressions with assertion binding;
6. treats retry-pass as failure while retaining canonical `cargo test` authority;
7. freezes coverage measurement design before seeing percentages and prevents same-PR floor gaming;
8. freezes mutation scope/config/inventory before execution and leaves no unexplained required result;
9. independently recomputes a canonical exact-head `AF02_ADVERSARIAL_SHA256` from validated raw evidence;
10. continuously proves AF-01, CF-06, and CF-10 authority has not drifted;
11. enforces synthetic/public provenance and safe handling of fuzz artifacts;
12. separates deterministic qualification from stochastic discovery observations.

## Functional requirements

### FR-001 — critical-surface policy

Implementation MUST provide `commandf.af02-surface-policy/v1` as defined in `evidence-contracts.md`.

The deterministic discovery rule MUST cover production parser/deserializer, archive/compression, filesystem/path, network/acquisition, cache/persistence, and subprocess boundaries under configured production source roots. Newly discovered unclassified boundaries fail validation. Stale surface entries whose source path, seam, or corpus binding disappears also fail closed.

Initial coverage MUST include:

- package acquisition/cache and archive/manifest/resource ingestion;
- Lockfile V1/V2 parse/validation/canonical serialization;
- source-map/SUSHI index and portable-path/root-containment behavior;
- context graph/canonical reference resolution;
- compatibility/check/quality-gate/fingerprint/suppression retained evidence;
- deterministic serializers whose persisted output can affect policy.

Evidence modes are limited to the normative enum in `evidence-contracts.md`.

### FR-002 — exact tool/package provenance

Every adopted AF-02 executable MUST have a `commandf.af02-tool-lock/v1` record. Allowed executable acquisition modes are only:

```text
LOCKED_GIT_REV_SOURCE_BUILD
IMMUTABLE_RELEASE_ASSET_WITH_SHA256
```

CI MUST retain and verify source/release identity, install/build command, locked source or release digest, installed executable SHA-256, version output, compiler/cargo/target/features.

Initial reviewed source commits:

```text
cargo-fuzz 0.13.2      984c861c8dfea28055254c5f1d2659ab2cd63f76
cargo-mutants 27.1.0   8ab1dc786a1f61a4e370416cc6c68b81a704e917
cargo-llvm-cov 0.9.0   be59056988acd54c7f984b7c85643daea3711b29
cargo-nextest 0.9.143  60fa45f638ffc3f35e74afa65737f45fcd32db2a
```

Test/fuzz library packages use exact crates.io versions plus Cargo registry checksums:

```text
proptest =1.11.0
libfuzzer-sys =0.4.13
arbitrary =1.4.2
```

The fuzz-only compiler is `nightly-2026-08-25`; normal product Rust remains canonical stable `1.97.1` unless separately authorized.

### FR-003 — bounded raw and structure-aware fuzzing

Fuzzing MUST use the normalized outcome classes and `commandf.af02-resource-policy/v1` from `evidence-contracts.md`.

Routine AF-02 fuzzing MUST enforce per-input timeout, max input bytes, memory, CPU, PIDs, temporary filesystem, generated/decompressed work, subprocess time, and artifact limits. Deterministic qualification runs offline after an independently verified acquisition phase, including OS/container network denial.

No-crash duration is never represented as timeless correctness proof. Harness/resource failures are incomplete runs, not clean observations.

### FR-004 — property tests and independent models

`proptest =1.11.0` remains test-only. Every property family records generator domain, bounds, case count, seed/shrink policy, expected outcomes, and an independent model/oracle where a differential claim is made.

Required initial models are frozen in `evidence-contracts.md` for archive/manifest, Lockfile, source-map/path, context graph, and quality-gate/fingerprint semantics.

Calling the same production implementation twice is not an independent oracle.

### FR-005 — deterministic corpus promotion

Every crash, invariant violation, unexpected acceptance, oracle divergence, or property counterexample used to close a defect MUST become a minimized deterministic regression under `commandf.af02-corpus/v1`.

Scenario identity format is `AF02-<SURFACE-SLUG>-NNNN`; digest is SHA-256 of raw stored bytes. Only `SYNTHETIC` and `PUBLIC_REDISTRIBUTABLE` provenance classes are allowed. Public inputs require source/redistribution evidence.

Every manifest entry MUST bind to an executable assertion/replay target and be executed in CI. Orphan metadata or orphan assertions fail closed.

Default limits are <=256 KiB per promoted fixture and <=8 MiB aggregate committed AF-02 corpus unless a separate reviewed policy change is already canonical.

### FR-006 — no-PHI and artifact safety

No PHI, private patient data, credentials, or unknown-provenance fixtures are allowed.

The primary authority is machine-checkable fixture provenance. A scanner provides defense in depth. Fuzz crash inputs are treated as opaque untrusted data: never executed; never used as file names; only bounded regular files under a dedicated artifact root may be retained; symlinks/devices/FIFOs/sockets/executable files/path escapes are rejected; logs record bounded escaped previews plus digest/size rather than dumping arbitrary inputs.

### FR-007 — nextest flaky-as-failure

AF-02 adopts cargo-nextest 0.9.143 only as additive evidence. Canonical:

```text
cargo test --workspace --all-features --locked
```

remains independently mandatory.

Repository CI profile and command-line enforcement are normative:

```toml
[profile.ci]
retries = 2
flaky-result = "fail"
slow-timeout = { period = "60s", terminate-after = 2 }
```

```text
--retries 2 --flaky-result fail
```

The CLI values intentionally prevent weaker per-test retry/flaky-result overrides from making AF-02 green.

An isolated non-workspace fixture MUST deterministically fail first attempt and pass on retry using an AF-02-owned temporary state file; expected process exit remains non-zero.

### FR-008 — measured coverage without same-PR gaming

Coverage uses cargo-llvm-cov 0.9.0 and the fixed pre-measurement descriptor in `evidence-contracts.md`:

```text
Linux/x86_64
Rust 1.97.1 + llvm-tools-preview
cargo llvm-cov --workspace --all-features --locked --json
production source: crates/*/src/**
```

Only explicit non-product exclusions are allowed initially. The raw report and normalized descriptor are retained.

Initial floors are computed after design freeze but mechanically from measurement:

- workspace production line floor = integer floor of measured percentage;
- each `COVERAGE_CRITICAL` surface independently gets integer-floor line coverage;
- no averaging of surface percentages;
- function/region coverage is diagnostic initially.

Floor reduction, source exclusion broadening, or descriptor weakening cannot make the same candidate green. Re-baselining requires a dedicated reviewed policy PR evaluated against the previous canonical policy and merged first.

### FR-009 — targeted mutation adequacy

Cargo-mutants 27.1.0 execution MUST be bound to exact source/tree, tool-lock identity, command/config, build profile, test command, parallelism, timeout policy, source scope, reviewed exclusions, JSON inventory digest, and stable required mutant IDs.

The design-freeze MUST verify exact flags against the pinned version and freeze them before execution. JSON mutant listing (`--list --json` or exact-version equivalent) forms the source inventory.

Required result classes:

```text
KILLED
SURVIVED
TIMEOUT
UNVIABLE_OR_BUILD_FAILURE
WAIVED_EQUIVALENT_OR_OUT_OF_SCOPE
```

Every required survivor must be killed or exactly waived. Every TIMEOUT/UNVIABLE result gets a bounded retry plus diagnosis; unresolved results require the same reviewed exact waiver standard and are never counted as killed. Incomplete/cancelled mutation runs fail.

A newly added waiver or reduced required set cannot make the same PR green; weakening mutation-policy changes require a separate canonical policy PR.

### FR-010 — exact-head proof and anti-forgery

The final proof schema is `commandf.af02-adversarial-proof/v1`.

`AF02_ADVERSARIAL_SHA256` is SHA-256 of the canonical JSON `deterministic` object using the exact normalization algorithm in `evidence-contracts.md`: recursively sorted keys, no floats, validated integer numerator/denominator coverage values, schema-defined array order, compact UTF-8 JSON, no trailing newline.

The verifier MUST independently derive source/tree, policy/spec hashes, corpus digests, raw test/tool classifications, live/canonical authority values, and base-policy comparison. It constructs and hashes the deterministic object itself; producer-supplied green JSON/digest is not trusted.

Stochastic campaign results are retained separately and excluded from deterministic proof identity.

### FR-011 — base-policy anti-forgery

A candidate cannot weaken its policy and use that weaker policy to prove itself green.

Potential weakening includes coverage floor/exclusion changes, critical-surface removal, discovery exclusions, resource/offline relaxation, flaky-pass overrides, mutation-set reduction/new waiver, corpus assertion removal, no-PHI relaxation, or AF-01/CF-06/CF-10 authority-baseline changes.

The candidate must pass the previous canonical policy as well or split the weakening into a dedicated independently reviewed policy PR merged before implementation. Strengthening uses the stricter base/candidate rule immediately.

### FR-012 — authority preservation

Every AF-02 stack and final proof MUST independently verify:

- AF-01 assurance ruleset ID `21652953` semantic policy and required contexts;
- AF-01 review-governance ruleset ID `21652974` semantic policy;
- CF-06 production oracle `hapifhir/org.hl7.fhir.core` release `6.10.2`, source commit `d06577d...`, validator JAR SHA-256 `a3addad...`, R4 core context 4.0.1;
- CF-10 unchanged C001/C002/C003 case membership plus retained head/run/artifact/digest identity.

The canonical semantic digests and complete values are frozen in `evidence-contracts.md`. Live/read-derived values, not a candidate-edited baseline alone, are authority.

### FR-013 — CI topology and partial-run semantics

Required logical jobs are acquisition/tool verification, deterministic adversarial replay/property, nextest-flake, coverage, mutation, stochastic discovery, and exact-head adversarial proof.

Every job uses AF-01 full-SHA Actions, least token authority, credentialless checkout, explicit timeout, bounded artifact retention, and immutable proof-critical execution identity.

Current candidate cancellation/timeout/runner loss is failure. Scheduled discovery cancellation is `INCOMPLETE`, never clean no-crash. Network access is separated from deterministic execution.

AF-02 does not add a live required `main` context by default. Any proposal to do so requires a separate universal-terminal topology proof, ruleset intent change, administrator application, and live read-back.

### FR-014 — separate design-freeze gates

Before dependent implementation code, each stack MUST first merge a separate reviewed design-freeze candidate:

- Stack A: authority/surface/resource/tool acquisition/property models/corpus/no-PHI;
- Stack B: nextest invocation/fixture/timeouts and coverage command/scope/descriptor/rebaseline policy;
- Stack C: exact mutation command/config/inventory/waiver policy and proof canonicalization/verifier/CI retention topology.

A candidate may not introduce new acceptance semantics and implementation depending on them together.

### FR-015 — product semantic non-regression

AF-02 MUST preserve CF-03/04/05/06/07/09/10/11/11G/12/13 semantics and AF-01 trust/security/live-policy authority. Minimal internal test-seam refactors are permitted only if public API remains unchanged and exact semantic regressions prove behavior preservation.

No public product API may be added solely for fuzzing convenience.

### FR-016 — reviewer truth

Every planning/design-freeze/implementation/convergence candidate requests Qodo and CodeRabbit when available. Findings are dispositioned against the exact current head. Summary-only/rate-limit/timeout/bot absence is not PASS. Head mutation supersedes prior exact-head review qualification.

## Non-functional requirements

### NFR-001 — deterministic qualification

Canonical product outputs, corpus replay, property assertions, coverage policy calculations, mutation inventory/classification, authority projections, and deterministic proof construction are reproducible from retained exact inputs.

### NFR-002 — fail closed

Malformed evidence, missing critical surface, unknown result class, stale corpus assertion, missing authority read-back, incomplete run, retry-pass, coverage floor breach, unclassified required mutant, or verifier mismatch fails qualification.

### NFR-003 — bounded resources

All AF-02 lanes use the checked-in resource policy. Routine AF-02 is not AF-04 stress testing.

### NFR-004 — offline deterministic execution

After immutable tool/dependency acquisition, deterministic AF-02 execution must be network-denied at Cargo and OS/container layers as defined by the normative contract.

### NFR-005 — no hidden retries

Retry exists for diagnosis only; retry-pass remains failure.

### NFR-006 — no PHI

Synthetic/public redistributable data only. Unknown provenance fails.

### NFR-007 — exact provenance

Tool binaries and registry packages are bound to exact source/release/package checksums. Tags/versions alone are insufficient proof identity.

## Explicit exclusions

AF-02 does not claim:

- all bugs are found by fuzzing;
- coverage percentage proves semantic correctness;
- mutation score is product correctness authority;
- no-crash runtime proves safety;
- AF-03 portability/release/SBOM/SLSA work is complete;
- AF-04 performance/stress work is complete;
- CF-14/15/16 product functionality is implemented;
- blocked CF-06/CF-10 governance is resolved.

## Planning status

The initial PR #54 head `3224098403f6bfb64525bfab002e94d5c3d82e69` passed its existing workflows but Qodo and CodeRabbit found material planning gaps. Those findings are accepted, not waived. This amended planning package adds the normative executable contracts required to close them.

Fresh exact-head CI and fresh independent review are still required. Until T006 merges and post-merge authority is re-read:

```text
AF-02: PLANNING_CANDIDATE
IMPLEMENTATION_AUTHORITY: NOT_GRANTED
```