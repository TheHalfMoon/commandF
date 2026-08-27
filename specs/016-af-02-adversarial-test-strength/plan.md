# AF-02 Plan — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Entry condition

AF-02 planning begins from canonical:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

Implementation is not authorized until T006 is canonical. CF-14 planning may proceed independently, but this plan does not authorize CF-14/15/16 product implementation.

## Normative design authority

`evidence-contracts.md` is normative for executable AF-02 evidence. It freezes before implementation:

- semantic authority snapshots for AF-01 live rulesets, CF-06 production oracle, and CF-10 frozen corpus;
- `commandf.af02-authority-baseline/v1`;
- deterministic critical-boundary discovery and `commandf.af02-surface-policy/v1`;
- normalized adversarial outcome classes;
- `commandf.af02-resource-policy/v1` with offline enforcement;
- immutable executable and registry package provenance through `commandf.af02-tool-lock/v1`;
- property/generator independent models;
- `commandf.af02-corpus/v1` plus assertion binding;
- no-PHI and fuzz-artifact safety rules;
- nextest CLI override-resistant flaky-as-failure behavior;
- fixed pre-measurement coverage descriptor and rebaseline rules;
- frozen mutation inventory/config/result/waiver rules;
- CI partial-run/cancellation/retention semantics;
- base-policy anti-forgery;
- `commandf.af02-adversarial-proof/v1` canonical JSON and independent verifier algorithm;
- separate design-freeze gates before every implementation stack.

No implementation PR may silently choose a weaker alternative to those contracts.

## Evidence model

AF-02 keeps four evidence classes distinct:

1. **deterministic regression/property evidence** — exact inputs/configuration and reproducible assertions;
2. **coverage evidence** — exercised source under a frozen descriptor;
3. **mutation evidence** — test sensitivity to a frozen exact mutant inventory;
4. **stochastic discovery evidence** — bounded fuzz exploration that may discover failures but whose no-crash observation is not correctness proof.

Only independently validated deterministic evidence contributes to `AF02_ADVERSARIAL_SHA256`. Stochastic observations remain separately retained metadata.

## Initial product boundary inventory

Planning has already identified these high-value existing boundaries:

- package registry/acquisition/cache plus gzip/tar archive, manifest and resource ingestion;
- Lockfile V1/V2 parse, validation, canonical serialization, exact package/dependency identity;
- SUSHI source-map index, portable paths, canonicalized filesystem containment, line/report bounds;
- context graph and canonical reference resolution including zero/one/many candidates;
- compatibility/check/quality-gate reports, fingerprints, baseline/suppression evidence, gate decisions;
- deterministic machine-readable serializers and persisted evidence validators.

Stack A design freeze will turn this into a machine-readable discovered inventory. It is not allowed to reduce the minimum set based on which targets happen to be easy to fuzz.

## Frozen initial tool identities

```text
cargo-fuzz 0.13.2
  source commit 984c861c8dfea28055254c5f1d2659ab2cd63f76

cargo-mutants 27.1.0
  source commit 8ab1dc786a1f61a4e370416cc6c68b81a704e917

cargo-llvm-cov 0.9.0
  source commit be59056988acd54c7f984b7c85643daea3711b29

cargo-nextest 0.9.143
  source commit 60fa45f638ffc3f35e74afa65737f45fcd32db2a

proptest =1.11.0
libfuzzer-sys =0.4.13
arbitrary =1.4.2
fuzz compiler nightly-2026-08-25
normal product Rust 1.97.1
```

Executable tools are acquired only through the immutable modes in the donor manifest/normative contract and produce an exact tool-lock record including executable digest. Crates.io packages retain exact registry checksums.

## Authority non-regression baseline

AF-02 continuously verifies, rather than merely asserting, these independent authorities:

### AF-01

- assurance ruleset `21652953` remains active on `refs/heads/main`, no bypass actor, deletion/non-fast-forward protected, strict required checks `rust`, `assurance-proof`, `scorecard`, each integration ID 15368;
- review-governance ruleset `21652974` remains active on main with one review, code-owner/latest-push/thread resolution protections, merge-only, PR-only repository-role bypass.

The exact semantic projections and SHA-256s are frozen in `evidence-contracts.md`. Every stack performs live API read-back and recomputation.

### CF-06

Production oracle remains:

```text
hapifhir/org.hl7.fhir.core 6.10.2
d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
validator_cli.jar sha256 a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6
hl7.fhir.r4.core@4.0.1
```

### CF-10

Frozen cases remain:

```text
C001 hl7.fhir.us.core  8.0.1 -> 9.0.0
C002 hl7.fhir.uv.ips   1.1.0 -> 2.0.1
C003 hl7.fhir.us.mcode 3.0.0 -> 4.0.0
```

Retained six-state evidence remains bound to PR #11 head `5fe10d9859407272acf6649fc3e868d3eb2fbd12`, run `31916124080`, artifact `9255732702`, digest `9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612`.

AF-02 does not merge PR #11 or reinterpret its blocked production-oracle result.

# Phase 0 — planning correction and authorization

The initial PR #54 head `3224098403f6bfb64525bfab002e94d5c3d82e69` passed all five existing workflows but received substantive Qodo and CodeRabbit findings. Those findings are accepted.

The planning correction must:

1. make `evidence-contracts.md` normative;
2. reconcile spec/plan/tasks/consistency/donor provenance to it;
3. receive fresh exact-head CI;
4. receive fresh exact-head Qodo and CodeRabbit review;
5. close every substantive inline thread;
6. merge only from the exact qualified head with expected-head protection;
7. re-read canonical main/tree plus live AF-01 rulesets after merge.

Only that post-merge state completes T006 and authorizes Stack A design freeze.

# Stack A — design freeze: adversarial surfaces, resources, properties, corpus

Stack A is split deliberately into **design freeze** then implementation. The design-freeze candidate MUST merge before any dependent harness code.

## A0 design-freeze artifacts

Create and independently review:

- authority baseline file with independently derived AF-01/CF-06/CF-10 values;
- `commandf.af02-surface-policy/v1` and deterministic boundary discovery validator;
- `commandf.af02-resource-policy/v1` and proof of effective offline/resource controls;
- initial `commandf.af02-tool-lock/v1` acquisition procedure for cargo-fuzz/property tooling;
- property configuration schema and initial independent models;
- `commandf.af02-corpus/v1`, assertion registry and replay registry design;
- no-PHI/provenance and opaque artifact-retention policy.

### Surface discovery implementation rule

A conservative deterministic scanner/AST-aware audit discovers production input/acquisition/cache/filesystem/subprocess boundaries under frozen source roots. New unclassified boundaries fail. Missing source/seam/corpus references fail. Reviewed exclusions are narrow and complete.

### Resource policy initial bounds

Routine target defaults from the normative contract include:

```text
PR deterministic job <= 30 min
single fuzz input <= 256 KiB
per-input timeout <= 5 s
process memory <= 768 MiB
CPU <= 2
PIDs <= 256
tmpfs <= 512 MiB
scheduled discovery per target <= 900 s
single retained crash artifact <= 1 MiB
retained discovery bundle <= 32 MiB
single promoted corpus fixture <= 256 KiB by default
aggregate committed AF-02 corpus <= 8 MiB
```

These are upper bounds, not performance targets. AF-04 owns large-stress qualification.

### Offline control

Tool/dependency acquisition occurs separately. Deterministic AF-02 execution then uses `CARGO_NET_OFFLINE=true` plus OS/container network denial. The intended Linux mechanism is a digest-pinned container with `--network none` and explicit CPU/memory/PID/tmpfs controls. Any equivalent replacement requires design-freeze proof before harness implementation.

## A1 implementation after canonical A0

Implement:

- isolated `fuzz/` workspace on `nightly-2026-08-25`;
- raw archive/package fuzzing through an existing product entrypoint;
- Lockfile raw and structured adversarial tests;
- source-map/path adversarial properties through narrow existing/internal test seams;
- context graph/canonical-reference structured properties;
- compatibility/check/gate/fingerprint/suppression adversarial properties;
- deterministic corpus promotion and assertion-bound replay;
- safe crash-artifact handling and no-PHI provenance gate;
- PR target build/corpus replay/property CI and bounded scheduled discovery.

Expected normalized outcomes come only from the surface policy enum. `UNEXPECTED_ACCEPTANCE`, invariant violation, oracle divergence, or panic requires minimization and deterministic regression before closure.

No public API is added solely for fuzzing convenience.

## A2 qualification

Exact Stack A implementation head must pass:

- canonical `cargo test --workspace --all-features --locked`;
- all new policy validators and adversarial deterministic replay/property gates;
- fuzz target compilation;
- every path-applicable existing product/oracle/AF-01 workflow;
- live AF-01 and canonical CF-06/CF-10 identity assertions;
- fresh Qodo and CodeRabbit with zero unresolved substantive threads.

Merge only from the exact qualified head and re-read post-merge authority before Stack B.

# Stack B — design freeze: flaky-as-failure and coverage

Again, design semantics merge before enforcement code.

## B0 nextest design freeze

Freeze:

```toml
[profile.ci]
retries = 2
flaky-result = "fail"
slow-timeout = { period = "60s", terminate-after = 2 }
```

and CI invocation:

```text
cargo nextest run ... --retries 2 --flaky-result fail
```

The command-line options intentionally disable weaker per-test override authority.

Freeze the isolated non-workspace retry fixture: first attempt atomically creates an AF-02-owned state file and fails; retry sees it and passes. Expected normalized outcome is `FLAKY_RETRY_PASS` with non-zero process exit. No clock/RNG/network/scheduler dependency.

## B0 coverage design freeze

Before measuring any percentage, freeze:

```text
platform Linux/x86_64
Rust 1.97.1 + llvm-tools-preview
cargo llvm-cov --workspace --all-features --locked --json
production source crates/*/src/**
```

plus explicit non-product exclusions, normalized baseline descriptor schema, raw-report retention, critical-surface mapping, and dedicated rebaseline/floor-reduction governance.

## B1 measured baseline

Only after B0 is canonical:

- measure workspace production line coverage;
- measure every `COVERAGE_CRITICAL` surface line coverage;
- freeze each floor as integer floor of its own exact `covered/total*100` result;
- retain function/region diagnostics but do not invent hidden floors;
- record numerator/denominator, not floating-point proof values.

No averaging across surface percentages.

## B2 implementation/qualification

Add nextest and coverage lanes, repository-owned policy validation and anti-gaming checks. A floor/descrip­tor/exclusion weakening cannot make its own PR green. Canonical cargo test remains independently mandatory.

Exact Stack B head must pass all Stack A, canonical, AF-01 and relevant product/oracle gates plus fresh independent review before merge.

# Stack C — design freeze: mutation and exact-head AF-02 proof

## C0 mutation design freeze

Before running required mutation qualification, freeze exact cargo-mutants 27.1.0:

- tool-lock executable digest;
- source SHA/tree;
- exact command/config and build profile;
- test command;
- parallelism (default bound 2 jobs);
- baseline required;
- minimum test timeout 20 s;
- explicit maximum test timeout <=120 s unless a smaller measured bound is frozen;
- explicit maximum build timeout <=180 s unless smaller;
- source scope and reviewed exclusions;
- `--list --json` inventory digest;
- stable mutant-ID derivation;
- timeout/unviable bounded retry and diagnosis policy;
- exact waiver schema and dedicated waiver-policy governance.

The exact pinned-version flags are verified during this design freeze and then cannot drift during execution.

## C0 proof/CI design freeze

Freeze `commandf.af02-adversarial-proof/v1`, canonical JSON algorithm and independent verifier behavior from `evidence-contracts.md` plus artifact retention and CI dependency graph.

The verifier takes canonical base policy as authority first. A candidate weakening acceptance criteria cannot prove itself under its own weakening.

## C1 mutation implementation

Generate the frozen JSON inventory, choose required target mutants based on false-PASS/security boundaries, then execute the exact inventory.

Result classes remain separate. Required `SURVIVED`, unresolved `TIMEOUT`, and unresolved `UNVIABLE_OR_BUILD_FAILURE` are not green. Timeouts/unviable receive bounded retry+diagnosis before exact waiver is even eligible.

New waiver or required-set reduction cannot make the same implementation candidate green; it requires a dedicated policy PR merged first.

## C2 stochastic discovery

Run bounded scheduled/manual campaigns. A clean campaign outcome is only:

```text
NO_CRASH_OBSERVED_WITHIN_BOUND
```

An interrupted/resource/harness failure is `INCOMPLETE_RESOURCE_OR_HARNESS_FAILURE`, not clean evidence.

## C3 exact-head proof

The verifier independently recomputes:

- source/tree/base identity;
- policy/spec/contract hashes;
- tool/package identities;
- surface discovery coverage;
- resource/offline enforcement;
- corpus raw digests/assertion replay;
- property outcomes;
- nextest retry self-test and no-flake truth;
- coverage descriptor/floors/numerator-denominator values;
- mutation inventory/results/waivers;
- canonical cargo-test identity;
- live AF-01 + CF-06/CF-10 authority state.

It builds canonical deterministic JSON and recomputes `AF02_ADVERSARIAL_SHA256`; a producer-authored digest is not trusted.

## C4 qualification/merge

Run all AF-02 deterministic evidence plus every path-applicable existing product/oracle/AF-01 workflow on exact head. Obtain fresh Qodo/CodeRabbit. Merge only from exact qualified head and verify post-merge main/tree/proof applicability/live authority.

AF-02 does not add a new required main context by default. A proposal for one is a separate governance/topology mutation with administrator read-back.

# Phase 4 — convergence

Create `convergence.md` only after canonical Stack C. It records:

- planning/A/B/C design-freeze and implementation identities;
- exact tool/package/executable checksums;
- surface policy/discovery results and reviewed exclusions;
- resource/offline enforcement;
- property models/configuration;
- corpus scenario IDs/digests/assertion bindings;
- nextest no-flake truth;
- coverage descriptor/baseline/floors;
- mutation inventory/results/waivers;
- stochastic observations, clearly non-deterministic;
- exact proof artifact/digest;
- reviewer dispositions;
- AF-01/CF-06/CF-10 non-regression;
- limits and AF-03/AF-04/CF deferrals.

Final closeout follows the repository temporal-evidence pattern: qualify the exact convergence head, merge with expected-head guard, re-read post-merge truth, then use a docs-only closeout candidate without circularly embedding future identifiers. `AF-02=CLOSED_CANONICAL` is claimed only after post-merge proof and live authority remain green.

# Acceptance principles

- **No false proof from same-PR policy weakening.** Canonical base policy is independently enforced.
- **No stochastic laundering.** No-crash fuzz time is observation, not correctness authority.
- **No coverage vanity target.** Scope is frozen before measurement; floors are mechanically derived.
- **No mutation score shortcut.** Exact required results are classified individually.
- **No hidden retry green.** CLI flaky-result fail is mandatory.
- **No unsafe fixture provenance.** Synthetic/public redistributable only.
- **No authority drift.** AF-01/CF-06/CF-10 are machine-checked every stack.
- **No hidden design choices.** Each stack's measured/operational semantics are canonicalized in a separate design-freeze PR before dependent implementation.