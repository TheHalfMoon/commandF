# AF-02 Plan — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Entry condition

AF-02 planning begins only after canonical AF-01 closure:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

AF-02 implementation does not start until this planning package, consistency analysis, and donor record are independently reviewed, exact-head green, merged to canonical `main`, and re-read from the merge result.

CF-14 planning may proceed independently under the canonical roadmap, but this AF-02 plan does not grant CF-14 product implementation authority.

## Design summary

AF-02 adds a test-strength evidence plane around commandF's existing Rust product core. It is not a new semantic engine and does not create a user-facing interoperability command.

The independently executable verification result is the AF-02 vertical capability.

The design separates four kinds of evidence that must not be conflated:

1. **deterministic regression/property evidence** — fixed inputs/configuration and byte-stable or invariant assertions;
2. **coverage evidence** — which code regions were exercised by a fixed test/corpus set;
3. **mutation evidence** — whether the current tests detect a frozen set of plausible source mutations;
4. **stochastic discovery evidence** — bounded fuzz exploration that can discover new failures but whose no-crash result is not a correctness proof.

The final AF-02 artifact records all four classes but only hashes normalized deterministic inputs/results into the canonical `AF02_ADVERSARIAL_SHA256` identity. Stochastic campaign metadata is retained and explicitly labeled.

## Current repository boundary inventory

Planning inspected canonical `commandf-pkg` rather than inventing generic fuzz targets.

### Archive/package ingestion

`crates/commandf-pkg/src/archive.rs` currently:

- decompresses gzip/tar input;
- caps manifest size;
- caps archive entry count;
- caps decompressed scan bytes using a floor, ratio, and absolute maximum;
- accepts only `package/package.json` after the existing normalization rule.

The first fuzz boundary should enter through an existing public product path such as `inspect_package` rather than making private implementation internals public solely for the fuzzer. It must exercise malformed compression/tar/JSON, unusual headers/paths, boundary sizes, and deterministic accepted manifests.

### Lockfile retained evidence

`Lockfile::from_slice`, `to_bytes`, and V2 validation enforce:

- schema distinction;
- sorted/deduplicated roots;
- exact package identity uniqueness/order;
- sorted/deduplicated resolved dependency edges;
- source/target existence;
- exact declared-constraint agreement with package manifest evidence;
- target version satisfaction;
- one resolved target per declared dependency;
- complete dependency-edge evidence.

This is a high-value structured property and mutation target because silent canonicalization of hostile persisted evidence would weaken provenance.

### Source mapping / portable paths

The CF-09 source-map path enforces bounded input/report sizes, source-root containment, portable relative paths, line ranges, duplicate output-file rejection, and mapping/report consistency.

The private SUSHI index parser is not a reason to add public API. Stack A should first determine whether the current public builder plus isolated temporary-directory fixtures provides sufficient reachability. If not, use an internal test-only seam or property tests colocated in the crate; do not publish a product API merely to satisfy fuzz tooling.

### Context graph

`build_context_graph` consumes validated Lockfile V2 evidence and verified cached package archives, extracts supported canonical references, and sorts/deduplicates graph nodes/edges. Important adversarial properties include order independence, explicit ambiguous/external/resolved status, malformed canonical target rejection, and inventory agreement.

### Compatibility/check/gate evidence

The check/gate layer computes canonical fingerprints, normalizes baseline/suppression membership, validates retained evidence, and makes quality-gate decisions. This is a critical false-PASS boundary. Properties must cover canonical JSON key ordering, evaluate-then-validate consistency, tamper rejection, and set/order equivalence.

## Frozen initial tool identities

The planning package records exact upstream identities so implementation does not silently switch tools or versions.

### cargo-fuzz

```text
version: 0.13.2
repository: https://github.com/rust-fuzz/cargo-fuzz
commit: 984c861c8dfea28055254c5f1d2659ab2cd63f76
license: MIT OR Apache-2.0
```

The fuzz workspace will also pin:

```text
libfuzzer-sys = =0.4.13
arbitrary = =1.4.2
nightly = nightly-2026-08-25
```

Exact crates.io checksums are recorded when Stack A first creates/locks the fuzz workspace.

### proptest

```text
version: =1.11.0
repository: https://github.com/proptest-rs/proptest
license: MIT OR Apache-2.0
upstream declared rust-version: 1.86
```

The exact registry checksum becomes part of implementation evidence when adopted. `proptest` remains a test/dev dependency only.

### cargo-nextest

```text
version: 0.9.143
repository: https://github.com/nextest-rs/nextest
commit: 60fa45f638ffc3f35e74afa65737f45fcd32db2a
```

The annotated Git tag object is not itself used as product authority; the resolved commit is retained.

### cargo-llvm-cov

```text
version: 0.9.0
repository: https://github.com/taiki-e/cargo-llvm-cov
commit: be59056988acd54c7f984b7c85643daea3711b29
license: Apache-2.0 OR MIT
upstream rust-version: 1.87
```

### cargo-mutants

```text
version: 27.1.0
repository: https://github.com/sourcefrog/cargo-mutants
commit: 8ab1dc786a1f61a4e370416cc6c68b81a704e917
license: MIT
upstream rust-version: 1.88
```

All tool MSRVs are compatible with commandF's current stable `1.97.1`; the fuzz runtime itself remains on an isolated dated nightly.

No tool is adopted as semantic authority. The checked-in AF-02 policy defines what evidence is required from each tool.

## Checked-in AF-02 policy model

Stack A should introduce a small machine-readable policy, expected under `.github/` or `tests/assurance/`, that records at minimum:

```text
schema
critical_surfaces[]
  id
  source_paths
  entrypoint/test_seam
  evidence_modes[]
  corpus_path
  size/case bounds
  mutation_scope
  coverage_critical

tool_identities
corpus_policy
mutation_waiver_schema
coverage_floor_schema
```

The policy must be deterministic and validated by repository-owned tests. A future critical parser path cannot silently evade AF-02 merely because no human remembered to add a fuzz target.

The AF-01 workflow-trust audit remains responsible for CI/action authority. AF-02 adds coverage of its own workflows/configs to AF-01 path/policy scope rather than duplicating that security model.

## Stack A — fuzz/property foundation and regression corpus

### 1. Inventory exact reachable entry points

Before writing harnesses, map every FR-001 surface to existing public or internal test entry points.

Rules:

- prefer existing public product APIs;
- colocated unit/property tests may reach private pure helpers;
- if a refactor is necessary, preserve public API and behavior;
- do not add a public API solely for fuzzing;
- do not create a new product crate just to host assurance code.

### 2. Fuzz workspace

Expected layout:

```text
fuzz/
  Cargo.toml
  rust-toolchain.toml
  fuzz_targets/
  corpus/
  artifacts/        # ignored/generated, never authority
```

The fuzz package is excluded from the normal workspace if necessary to keep nightly-only/libFuzzer dependencies out of standard `cargo test` resolution semantics.

`fuzz/Cargo.toml` pins exact fuzz-only crate versions and depends on local `commandf-pkg` by path. `rust-toolchain.toml` pins `nightly-2026-08-25` and the exact components needed by the selected workflow.

Initial required targets should be selected from this minimum set after reachability inventory:

- `package_archive` — arbitrary raw package archive bytes through a public package-inspection/acquisition parser path;
- `lockfile` — raw JSON plus structure-aware Lockfile V2 forms;
- `retained_report` — malformed/structured check or quality-gate retained evidence through public validators/serde boundaries;
- `context_graph` — structured synthetic lock/cache/archive combinations and canonical reference shapes;
- source-map adversarial parsing/path semantics through the narrowest existing reachable seam.

It is acceptable for one raw fuzzer plus property tests to jointly cover a surface when that gives stronger reachability than forcing every path through libFuzzer.

### 3. Harness bounds

Add explicit policy limits far below large legitimate production package maxima for routine CI fuzzing. Initial planning guidance:

- raw input cap: `1 MiB` unless the target has a smaller domain-specific limit;
- structured collection counts: small bounded vectors chosen to exercise branch combinations without combinatorial explosion;
- temporary filesystem footprint: bounded and deleted per case/campaign;
- no network access from fuzz targets;
- no real registry/oracle invocation;
- no PHI or patient instances.

Large-package/resource stress belongs to AF-04 unless a minimized security/correctness reproducer requires a larger AF-02 fixture.

### 4. Property tests

Add `proptest = =1.11.0` only where the new tests execute immediately.

Prefer deterministic seed replay plus shrinking. Check in property configuration so case counts do not silently drift between developer and CI environments.

Required first properties:

- Lockfile V2 canonical round trip;
- persisted noncanonical V2 rejection;
- canonical fingerprint JSON key-order independence;
- quality-gate construction/validation consistency and tamper rejection;
- deterministic order/set properties for context evidence where fixture construction is bounded;
- path traversal/prefix/line-range rejection for source mapping.

### 5. Corpus promotion

Introduce a corpus manifest with stable scenario IDs and SHA-256 digests. Do not commit an ever-growing libFuzzer discovery corpus.

Promotion workflow:

```text
discovery failure
  -> reproduce on exact source/tool identity
  -> minimize/shrink
  -> classify product bug vs harness issue vs expected rejection
  -> commit smallest stable non-PHI fixture
  -> add deterministic regression assertion
  -> only then fix/close defect
```

The corpus replay gate must fail if a manifest entry is missing, digest-mismatched, unreferenced, or no longer exercised.

## Stack B — flaky-as-failure and coverage evidence

### nextest

Add `.config/nextest.toml` with an explicit AF-02/CI profile.

Initial intended retry semantics are frozen by spec:

```toml
[profile.ci]
retries = 2
flaky-result = "fail"
```

The final slow-timeout configuration is measured from current test timing before it is committed; do not guess a short timeout and then normalize failures as expected.

Prove the policy with a controlled self-test outside the ordinary product test corpus. One acceptable pattern is an ephemeral test fixture created by an AF-02 policy test/workflow that intentionally fails its first attempt and passes the second, then asserts nextest still exits non-zero. The fixture must not make canonical `cargo test` flaky.

`cargo test --workspace --all-features` remains in general CI.

### coverage baseline

Run `cargo-llvm-cov 0.9.0` against an exact canonical source/tree with the frozen stable compiler/tool configuration selected by the implementation.

The first baseline must be evidence, not a target chosen in advance. Stack B then freezes:

- a workspace line floor derived from the measured baseline;
- critical-module/surface floors where stable module-level extraction is available;
- function/region measurements as diagnostics;
- explicit exclusions.

Recommended floor derivation rule: use the integer floor of the exact measured percentage for a given frozen scope unless real measurement demonstrates that tool rounding makes this unstable. Any different rule must be documented before enforcement.

Do not exclude low-coverage product modules merely to raise the number. Exclusions are for generated code, fuzz harnesses, or demonstrably non-product instrumentation and must be path-specific.

Add a repository-owned validator proving that a PR cannot modify both source and coverage policy to lower the floor without an explicit policy-change marker/rationale. If implementation cannot enforce this mechanically without excessive complexity, require a dedicated coverage-policy task/PR and record the limitation; silent same-change floor weakening remains prohibited.

## Stack C — mutation adequacy and AF-02 proof

### mutation inventory

Before running a gate, collect `cargo-mutants` candidate mutations on the exact current implementation source. Freeze the required target paths/functions in checked-in policy.

Initial priority order:

1. compatibility/gate decision predicates that can create false PASS/non-breaking output;
2. Lockfile retained-evidence validation;
3. source-map path/line/source-escape validation;
4. context canonical resolution/order/dedup logic;
5. archive bounds/path/manifest acceptance logic;
6. serializer/report validators that authenticate retained evidence.

Mutation execution should be targeted rather than an unbounded whole-repository every-PR run.

### mutation classifications

Retain counts separately for:

```text
KILLED
SURVIVED
TIMEOUT
UNVIABLE_OR_BUILD_FAILURE
WAIVED_EQUIVALENT_OR_OUT_OF_SCOPE
```

A timeout is not silently counted as killed. A compiler-rejected mutant is not proof that tests caught a behavioral defect.

Waiver entries require:

- exact tool version/commit;
- source path and function/item identity;
- mutation description or stable identifier available from tool output;
- classification reason;
- compensating test/evidence;
- owner/revisit condition;
- removal condition.

Every required `SURVIVED` result must be killed or waived before Stack C can qualify. Aggregate mutation score is retained but is not an acceptance shortcut.

### AF-02 proof workflow

Add a dedicated workflow, expected name `.github/workflows/af02-adversarial-proof.yml`, with AF-01-compliant action pins, checkout credentials, permissions, runners, timeouts, and path coverage.

The proof should retain normalized files similar to:

```text
af02-summary.json
surface-inventory.json
corpus-manifest.json
property-evidence.json
nextest-evidence.json
coverage-evidence.json
mutation-evidence.json
fuzz-tool-identities.txt
fuzz-discovery-observations.json
source-identities.txt
```

`af02-summary.json` has a stable schema. The deterministic digest excludes timestamps and inherently stochastic ordering/coverage-path exploration metadata.

Example terminal identity:

```text
AF02_ADVERSARIAL_SHA256=<64 lowercase hex>
```

The workflow must fail if required deterministic evidence is absent, malformed, source-mismatched, policy-mismatched, or contains an unclassified mutation survivor/corpus failure/flaky result/coverage-floor violation.

A clean stochastic fuzz campaign is retained as `NO_CRASH_OBSERVED_WITHIN_BOUND`, not as a correctness PASS property.

## CI topology

### Required canonical checks

AF-01 currently protects `main` with universal required contexts:

```text
rust
assurance-proof
scorecard
```

AF-02 does not modify that live policy by default.

The AF-02 proof and heavy lanes can be path-applicable/non-required while the existing universal required checks remain enforced. If later AF-02 convergence concludes that an AF-02 aggregation check should be universally required, that is a separate source-control-policy change that must:

1. add an always-triggered terminal job/workflow topology;
2. prove docs-only/nonmatching PR behavior;
3. update checked-in ruleset intent;
4. receive independent review;
5. be applied and live-read back before closure claims depend on it.

No heavy whole-workflow path filter may be selected directly as a required check if it can remain pending.

### Suggested execution cadence

- PR: deterministic corpus replay, properties, target build, nextest, coverage policy where relevant;
- scheduled/manual: bounded fuzz discovery campaigns;
- targeted path-aware PR or dedicated qualification: cargo-mutants;
- exact AF-02 implementation/convergence heads: complete dedicated proof including the frozen mutation set and current coverage floors.

Scheduled discovery failure is actionable when a crash/invariant is found. Scheduled discovery unavailability is operationally distinct from a clean result.

## Security and trust boundary

AF-02 processes repository source and synthetic/public conformance artifacts only.

- no PHI;
- no real patient-instance fixtures;
- no secrets/model credentials;
- no network from fuzz targets;
- no `pull_request_target` introduction;
- no untrusted PR secrets;
- AF-01 least-authority and full-SHA Action rules remain mandatory;
- generated paths/files stay inside isolated temporary directories;
- corpus promotion checks provenance/license before commit;
- `unsafe` harness code is prohibited unless explicitly justified and reviewed;
- fuzz crashes/artifacts may contain arbitrary bytes and must not be executed as scripts.

## Determinism model

### Deterministic evidence

- exact source/tree and policy blobs;
- fuzz target source/build identities;
- committed corpus files/digests and replay outcome;
- property configuration and fixed reproducer seeds;
- nextest configuration and outcome;
- coverage measurement for exact compiler/source/tool/test inputs;
- frozen mutation inventory/result classification for exact source/tool/test inputs;
- AF-02 summary schema/digest.

Mutation wall time can vary while classification over a completed frozen run remains the evidence object. If infrastructure instability prevents complete classification, the run is incomplete, not green.

### Stochastic evidence

- fuzz path exploration;
- generated mutation sequence only if not frozen by exact policy/tool output;
- discovery timing/execution counts.

These observations record source/tool/seed/budget/corpus identity when available. They do not become timeless correctness claims.

## Test plan

### Surface-policy tests

- every initial critical source surface appears in the manifest;
- every surface has at least one required evidence mode;
- no referenced fuzz/property/corpus path is missing;
- future new parser/validator paths matching policy discovery rules cannot silently bypass classification;
- duplicate surface IDs fail.

### Fuzz harness tests

- each required target compiles on the pinned nightly;
- malformed bytes do not panic in target smoke/replay;
- input-size caps are honored before expensive fixture construction;
- targets are network-free;
- temporary filesystem paths cannot escape their root;
- corpus replay gives stable expected acceptance/error classes.

### Property tests

- Lockfile V2 round-trip/normalization/rejection;
- context/order invariance;
- fingerprint JSON-order invariance;
- gate evaluate/validate and tamper rejection;
- source-map path/line-range failures;
- deterministic output repeat tests.

### Corpus tests

- manifest digest verification;
- no orphan/missing fixtures;
- per-file size policy;
- every fixture links to a deterministic regression assertion;
- synthetic/public provenance field required.

### nextest tests

- exact tool identity retained;
- configured retry-pass self-test exits non-zero;
- genuine pass remains pass;
- genuine repeated failure remains failure;
- ordinary `cargo test` still executes independently.

### Coverage tests

- exact tool/compiler/source identity recorded;
- baseline policy generated only from measured evidence;
- floor breach fails;
- unknown/missing product source coverage does not silently disappear;
- exclusions are explicit and bounded.

### Mutation tests

- target inventory stable and source-bound;
- known deliberately seeded mutation in AF-02 self-test fixture survives without a test and causes failure;
- strengthened test kills the mutation;
- unclassified survivor fails;
- timeout/build failure classified distinctly;
- waiver parser rejects missing rationale/revisit/source identity.

### Proof tests

- same normalized deterministic inputs produce same summary bytes/digest;
- source/tree mismatch fails;
- missing corpus/property/nextest/coverage/mutation evidence fails;
- stochastic fields cannot silently enter deterministic digest identity;
- unclassified survivor/flaky result/floor breach/corpus replay failure makes proof non-green.

### Product regression

Every implementation head still runs:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

plus all path-applicable existing proof/oracle workflows and canonical AF-01 assurance/security gates.

## Migration impact

Developer-visible changes after implementation:

- new parser/validator boundaries require AF-02 surface classification;
- fuzz-only dependencies/toolchain exist outside product runtime;
- property tests add reproducible generated cases/shrinking;
- a retry-pass is a visible failure;
- coverage regression below frozen floors blocks the relevant AF-02 gate;
- required mutation survivors require tests or explicit narrow waivers;
- fuzz crashes/counterexamples become committed minimized deterministic regressions;
- scheduled fuzz discovery may open follow-up defects without implying prior green evidence was fraudulent.

No shipped CLI schema or interoperability semantic behavior is intentionally changed.

## Performance/cost impact

AF-02 intentionally adds CI cost, but it must be bounded and tiered.

- property/corpus replay should remain PR-suitable;
- nextest is additive but should not duplicate extremely expensive external network/oracle work where the current test model already separates it;
- coverage may be path-aware but closure heads receive complete frozen coverage evidence;
- mutation testing is targeted, not whole-workspace exhaustive on every edit;
- fuzz discovery is scheduled/manual and bounded;
- AF-04, not AF-02, owns quantitative product performance/resource regression claims.

If measured CI cost is materially higher than planned, amend the plan with measured data rather than silently skipping required evidence.

## Stack ordering

```text
Planning package + donor record
  -> Stack A: surface policy + fuzz workspace + initial fuzz targets + properties + corpus replay
  -> Stack B: nextest flaky-as-failure + measured coverage baseline/floors
  -> Stack C: targeted mutation adequacy + AF-02 exact-head proof
  -> convergence
```

Stack B branches only from canonical Stack A. Stack C branches only from canonical Stack B unless a deliberate stacked-PR topology preserves exact dependencies and all heads are independently qualified.

## Closure criteria

AF-02 is `CLOSED_CANONICAL` only when:

1. planning package/donor record merged from an exact green independently reviewed head;
2. all required critical surfaces are machine-classified and have the evidence modes required by canonical policy;
3. Stack A fuzz/property/corpus foundation is canonical and every discovered failure used for closure has a minimized deterministic regression;
4. Stack B nextest retry-pass self-test proves flaky-as-failure, coverage baseline is measured and floors are frozen/green;
5. Stack C required mutation inventory has no unclassified surviving mutants and all waivers are narrow/reviewed;
6. exact-head AF-02 proof artifact is retained with reproducible deterministic summary digest and stochastic evidence correctly labeled;
7. canonical `cargo test` and all mandatory/path-applicable pre-existing product/oracle/AF-01 gates remain green on the exact candidate heads;
8. CodeRabbit/Qodo exact-head truth has zero unresolved substantive findings;
9. convergence re-reads spec/plan/tasks/consistency, implementation, AF-01 live policy, and donor/tool identities and records remaining AF-03/AF-04/CF work without scope laundering;
10. convergence and final closeout merge from exact qualified heads, and canonical post-merge main/tree are re-read before `CLOSED_CANONICAL` is claimed.

Implementation merge alone is insufficient for closure.
