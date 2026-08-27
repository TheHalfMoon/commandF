# AF-02 Specification — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Identity

`AF-02` is the second commandF Assurance Foundation unit. It measures whether commandF's existing deterministic product core fails safely under generated, malformed, mutated, adversarial, and reordered inputs.

The Spec Kit directory sequence `016` is the next available repository planning sequence. It does **not** rename or consume product identity `CF-16`.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
CF-13: CLOSED_CANONICAL
```

AF-02 does not depend on the separately blocked CF-06/CF-10 production-oracle path and does not authorize a CF-06 production-pin change.

## User problem

commandF has strong example-based tests, deterministic exact-head CI, fail-closed validators, authoritative-oracle checks, and the canonical AF-01 trusted-development baseline. That proves that known scenarios behave as intended and that the development path is protected.

It does not yet prove that the tests themselves are strong enough against plausible implementation defects or hostile input families.

The current product processes security- and correctness-sensitive structures including compressed FHIR package archives, JSON manifests/resources, lockfiles, source-map indexes, graph evidence, compatibility reports, suppressions, fingerprints, and canonical references. Many of these surfaces already enforce explicit limits and canonical ordering. Without property testing, fuzzing, mutation adequacy, coverage diagnostics, and explicit flaky-test policy, commandF can still have blind spots that ordinary regression examples do not reveal.

The risk is not merely a crash. For commandF, a dangerous test gap can produce:

- a false compatible/non-breaking result;
- a non-deterministic report or fingerprint;
- acceptance of malformed or non-canonical retained evidence;
- path traversal or source-escape behavior;
- silent graph/reference ambiguity;
- a regression that passes only after retry;
- a plausible logic mutation that the test suite fails to detect.

## Outcome

After AF-02 closes, commandF has an independently executable adversarial-testing evidence layer that:

1. maintains a reviewed inventory of high-risk parser, validation, canonicalization, graph, report, and evidence boundaries;
2. fuzzes raw and structure-aware inputs with exact tool identities and bounded harnesses;
3. expresses algebraic/canonical invariants as property tests with shrinking and reproducible failing seeds;
4. promotes every discovered crash or invariant violation into a minimized deterministic regression corpus before the fix is considered closed;
5. measures targeted mutation adequacy and leaves no required surviving mutant unexplained;
6. records coverage as a diagnostic floor derived from a measured canonical baseline rather than a vanity target;
7. detects flaky retry-pass behavior as a failure rather than converting it to green;
8. retains an exact-head adversarial proof artifact while clearly separating deterministic replay evidence from stochastic fuzz-discovery evidence;
9. preserves all existing product semantics, `cargo test` authority, AF-01 source-control policy, and oracle identities.

## Functional requirements

### FR-001 — adversarial surface inventory

Add a checked-in AF-02 surface manifest or equivalent machine-checkable policy that enumerates every required adversarial-testing boundary and its evidence mode.

The initial required inventory must cover at least these existing commandF areas:

- compressed package/archive and manifest ingestion through a public product path such as package inspection;
- `Lockfile` V1/V2 JSON parsing, validation, canonical serialization, and dependency-edge evidence;
- SUSHI/FSH source-map index and serialized source-mapping validation, including portable path handling and source-escape defenses;
- context-graph construction/canonical-reference resolution and deterministic ordering;
- compatibility/check/quality-gate retained evidence, canonical fingerprints, suppressions, and report validation;
- deterministic machine-readable serializers used by shipped commands where malformed retained evidence can affect policy.

The inventory must classify each surface as one or more of:

```text
RAW_FUZZ
STRUCTURED_FUZZ
PROPERTY
DIFFERENTIAL_OR_CROSS_PATH
MUTATION_TARGET
COVERAGE_CRITICAL
CORPUS_REPLAY
```

A future parser or instance-data boundary, especially any CF-14 source-profiler input boundary, must explicitly reconcile this AF-02 inventory before that later feature can close canonically.

### FR-002 — isolated fuzz workspace and exact tool identity

Add a dedicated fuzz workspace/harness that does not become a normal commandF product runtime dependency.

AF-02 planning freezes these initial tooling identities for implementation review:

```text
cargo-fuzz: 0.13.2
upstream commit: 984c861c8dfea28055254c5f1d2659ab2cd63f76

libfuzzer-sys: =0.4.13
arbitrary: =1.4.2

fuzz compiler: nightly-2026-08-25
```

The normal commandF workspace remains on its canonical stable Rust declaration (`1.97.1`) unless separately changed by authorized roadmap work. The nightly toolchain is fuzz-only and must not leak into product/release compatibility claims.

The implementation must record registry checksums or equivalent exact package identity for fuzz-only crates when they are first locked.

### FR-003 — raw and structure-aware fuzzing

Provide bounded fuzz targets appropriate to the shape of each required boundary.

Raw-byte fuzzing is required where malformed bytes are itself the security boundary, including archive/compression and JSON parsing surfaces.

Structure-aware fuzzing is required where random bytes would spend most executions in trivial parser rejection and would fail to exercise meaningful invariants. Structured generators must favor semantically interesting combinations such as:

- valid and invalid Lockfile V2 package/dependency graphs;
- canonical references with versions/fragments/empty components;
- source-map line ranges and portable path components;
- report/suppression/fingerprint combinations;
- graph node/edge order permutations and ambiguity cases.

Fuzz harnesses must bound generated input sizes, collection lengths, recursion/depth, filesystem footprint, and per-case work so adversarial testing does not turn existing large product limits into routine multi-hundred-MiB CI allocations.

No harness may use `unsafe` merely to increase fuzz throughput without a separately reviewed need.

### FR-004 — differential and cross-path invariants

Where commandF has two independently meaningful computation paths or an existing external/reference contract, AF-02 must compare them rather than merely assert "no panic".

Eligible examples include:

- constructor canonicalization followed by serialization/parsing versus persisted-evidence validation;
- package resource inventory paths that are expected to agree on identity/count/order for the same synthetic archive;
- report construction followed by independent report validation/recomputation;
- canonical graph/fingerprint outputs produced from equivalent input permutations;
- existing CF-06 oracle adapter/reconciliation contracts **without changing the frozen production oracle identity**.

A comparison that uses the same underlying implementation twice must not be described as an independent oracle.

Every discovered divergence must be classified. Unknown divergence fails closed until minimized and understood.

### FR-005 — property-test layer

Adopt `proptest = =1.11.0` as a development/test-only dependency where property generation and shrinking materially improve evidence. The upstream crate currently declares Rust MSRV `1.86`, compatible with commandF's current `1.97.1` workspace.

Initial property families must include, where the current APIs permit them without semantic redesign:

- valid Lockfile V2 canonical round-trip and byte-stable serialization;
- constructor normalization versus rejection of non-canonical persisted V2 evidence;
- deterministic set/order semantics for context graph and comparable report structures;
- JSON object-key-order independence for canonical fingerprint identities;
- quality-gate evaluate-then-validate consistency and rejection of tampered retained evidence;
- suppression/baseline membership equivalence independent of caller input ordering;
- portable-path rejection for absolute, traversal, empty-component, and drive/prefix forms;
- deterministic serializers returning byte-identical output for equivalent canonical input.

Property runs must use checked-in case-count/shrinking configuration. A failing seed/reproducer must be retained before the defect is marked fixed.

### FR-006 — minimized regression corpus promotion

Every fuzz crash, timeout caused by an input defect, invariant violation, unexpected acceptance, or property counterexample found by AF-02 must be reduced to a deterministic reproducer before closure of the corresponding fix.

The promoted corpus must:

- contain only synthetic or publicly redistributable non-PHI data;
- preserve the smallest practical reproducer rather than an entire discovery corpus;
- have stable scenario identity and digest;
- name the affected boundary and expected fail-closed/accepted behavior;
- execute through deterministic regression tests or an equivalent corpus-replay gate;
- remain reviewable in repository size.

Default committed regression payloads should remain at or below `256 KiB` each. A larger minimized reproducer requires explicit rationale, provenance, and a bounded aggregate-corpus review.

Discovery corpora and generated coverage artifacts are not automatically committed.

### FR-007 — targeted mutation adequacy

Adopt targeted mutation testing with:

```text
cargo-mutants: 27.1.0
upstream commit: 8ab1dc786a1f61a4e370416cc6c68b81a704e917
```

The initial required mutation set must prioritize code where a plausible logic error could create false compatibility, evidence inconsistency, unsafe input acceptance, or non-determinism. Candidate modules include archive/lock/source-map/context/gate and compatibility validation surfaces after exact implementation-time inventory.

AF-02 does **not** define one aggregate mutation percentage as correctness authority.

For the required mutation target set:

- killed mutants are retained as measured evidence;
- timeouts/build failures are classified separately rather than silently counted as test kills;
- every surviving mutant must either be killed by a new/strengthened test or be entered in a checked-in narrow waiver with exact source/mutation identity, rationale, compensating evidence, and revisit/removal condition;
- broad file-level "skip everything" exclusions are prohibited for critical modules;
- equivalent/trivial/generated mutations may be waived only after review.

No unclassified required survivor is compatible with AF-02 closure.

### FR-008 — coverage diagnostics and floors

Adopt source-based coverage with:

```text
cargo-llvm-cov: 0.9.0
upstream commit: be59056988acd54c7f984b7c85643daea3711b29
```

Coverage is evidence of exercised code, not proof of correctness.

The implementation must first measure a canonical baseline on the exact planning/implementation base. Only then may it freeze initial floors. Floors must be derived from measured reality rather than guessed percentages.

At minimum retain:

- workspace line coverage;
- function and region coverage as diagnostics;
- line coverage for AF-02 critical modules/surfaces where the tool can report it reliably;
- the exact tool/compiler/source identities used for the baseline.

A candidate that lowers a frozen floor must not weaken the floor in the same implementation change merely to regain green. A floor reduction requires an explicit reviewed policy change with rationale and revisit condition.

Generated/vendor/fuzz harness code exclusions must be explicit; product modules may not disappear from coverage by broad glob accident.

### FR-009 — flaky-as-failure execution

Adopt nextest for additive flaky-test evidence with:

```text
cargo-nextest: 0.9.143
upstream commit: 60fa45f638ffc3f35e74afa65737f45fcd32db2a
```

AF-02 must configure a CI profile that allows bounded retry for diagnosis but treats a test that passes only after retry as failure. Initial intended semantics are:

```toml
[profile.ci]
retries = 2
flaky-result = "fail"
```

Any final timeout/slow-test profile must be based on observed repository behavior and be bounded.

Canonical `cargo test --workspace --all-features` remains mandatory. `cargo nextest` adds flaky/retry evidence and must not replace `cargo test` or hide doctest/test-harness differences.

A synthetic AF-02 self-test must prove that retry-pass is reported as failure without introducing a permanently flaky test into the ordinary product suite.

### FR-010 — deterministic PR qualification versus stochastic discovery

AF-02 must explicitly separate two evidence classes.

**Deterministic/replayable qualification** may include:

- compilation of every required fuzz target;
- deterministic replay of every committed regression corpus input;
- property tests with retained configuration and reproducible failing seeds;
- ordinary `cargo test` and nextest no-flake result;
- coverage against a fixed test/corpus set;
- targeted mutation results for a frozen mutation set and exclusions.

**Stochastic discovery evidence** includes bounded fuzz campaigns whose explored paths can differ between runs. Such campaigns must record exact source/tool/seed/budget/corpus identities where available, but a no-crash fuzz duration is not a proof of correctness and is not byte-deterministic product evidence.

PR qualification must not claim that "fuzzed for N minutes with no crash" proves a candidate safe.

### FR-011 — CI topology and bounded execution

AF-02 CI must keep expensive adversarial work independently diagnosable and bounded.

The intended topology is:

1. **PR adversarial qualification** — fuzz target build, committed corpus replay, property tests, nextest flaky-as-failure, and coverage checks needed by the candidate;
2. **bounded discovery campaign** — scheduled/manual fuzz discovery with retained crash/corpus metadata; failure means a real discovered defect, while no crash is only a bounded observation;
3. **targeted mutation lane** — path/scope-aware mutation adequacy with exact survivor/waiver evidence;
4. **AF-02 proof lane** — exact-head summary over the required deterministic evidence and retained stochastic observations.

Heavy AF-02 workflows need not become universal `main` required checks. If any new check is proposed for the live AF-01 ruleset, it must first prove universal terminal topology and undergo separate checked-in/live-policy reconciliation. AF-02 must not weaken or silently replace existing required contexts `rust`, `assurance-proof`, or `scorecard`.

Every new job must have an explicit timeout and least GitHub token authority consistent with canonical AF-01 policy.

### FR-012 — exact-head adversarial proof artifact

Add a dedicated retained AF-02 proof workflow or equivalent artifact with a stable schema recording at least:

- exact source SHA and tree SHA;
- hashes of AF-02 spec/plan/tasks/consistency and donor/policy files;
- fuzz target inventory and exact build/toolchain identities;
- committed corpus manifest, scenario IDs, and digests;
- property-test configuration/outcome;
- nextest configuration and proof that no retry-pass was converted to green;
- coverage tool identity, baseline identity, frozen floors, and observed values;
- mutation tool identity, target set, killed/surviving/timeout/build-failure classes, and waiver identities;
- stochastic fuzz campaign bounds/results labeled as discovery evidence;
- a deterministic final summary digest over normalized deterministic evidence, for example:

```text
AF02_ADVERSARIAL_SHA256=<64 lowercase hex>
```

Timestamps must not be used as evidence identity. Stochastic observations may carry timestamps as metadata, but the deterministic summary must not pretend their execution order/time is a semantic identity.

### FR-013 — product and assurance authority non-regression

AF-02 must not change or weaken:

- CF-03 structural-diff semantics;
- CF-04 compatibility classification/rule semantics;
- CF-05 check/SARIF policy semantics;
- CF-06 production oracle identity;
- CF-07 terminology semantics;
- CF-09 authored-source mapping semantics;
- CF-10 frozen corpus;
- CF-11/11G package/context graph identities;
- CF-12 impact semantics;
- CF-13 baseline/suppression/gate semantics;
- AF-01 workflow-trust, dependency-security, required-check, or live source-control enforcement.

A minimal internal refactor made solely to expose a pure test seam is allowed only if it preserves public API and behavior and receives full semantic regression qualification. AF-02 must not add public product API solely for fuzz harness convenience.

### FR-014 — reviewer truth

Every AF-02 planning and implementation stack must request CodeRabbit and Qodo when available. Findings are dispositioned against the exact current head. Reviewer timeout/rate limit/summary-only output is not a PASS.

## Non-functional requirements

### NFR-001 — determinism

All canonical product outputs, corpus replay, property assertions, coverage policy calculation, mutation policy parsing, and AF-02 deterministic summary construction must be reproducible from retained exact inputs.

Fuzz discovery itself is stochastic and must be labeled accordingly rather than laundering randomness into deterministic evidence claims.

### NFR-002 — fail closed

Malformed retained evidence, unknown mutation survivor classification, missing required corpus replay, missing critical surface coverage, or missing required proof input fails AF-02 qualification.

### NFR-003 — bounded resources

Fuzz/property/mutation/coverage/nextest jobs require explicit timeout, case/input bounds, and finite corpus policies. Harnesses must not routinely exercise product maximums that imply hundreds of MiB per iteration unless a separate stress scenario explicitly requires and bounds that work.

### NFR-004 — no hidden retries

A retry-pass is evidence of flakiness and remains a failed AF-02 result. Retry exists to diagnose, not to manufacture green.

### NFR-005 — no PHI

No patient data or PHI is introduced. Synthetic/public conformance metadata only.

### NFR-006 — exact provenance

Tool versions, upstream commits where applicable, Rust toolchains, crate registry checksums, corpus digests, policies, and source identities are retained.

### NFR-007 — stackability

Implementation is split into small independently reviewable stacks. Do not combine every fuzz target, coverage policy, mutation program, and proof workflow into one opaque PR.

### NFR-008 — no vanity metric

Coverage percentage, mutation score, number of fuzz executions, and fuzz duration are separate evidence dimensions. No single aggregate number becomes a commandF trust/correctness score.

## Acceptance scenarios

1. Malformed/truncated gzip/tar input enters the archive fuzz boundary -> commandF returns a bounded error or accepted result and does not panic or escape configured harness limits.
2. A valid generated Lockfile V2 built through canonical constructors -> serialize -> parse -> serialize produces equivalent object state and byte-identical canonical output.
3. Persisted Lockfile V2 evidence with unsorted/duplicate roots, packages, or dependency edges -> validation rejects it rather than silently canonicalizing hostile retained evidence.
4. A SUSHI source-map path containing `..`, absolute/root form, empty component, Windows drive/prefix form, or source escape -> fails closed.
5. Equivalent context-graph inputs differing only in caller ordering -> canonical report bytes/normalized graph evidence remain identical.
6. A finding whose embedded JSON objects have different object-key insertion order but identical semantic content -> `finding_fingerprint_v1` identity remains identical.
7. A quality-gate report is built and then one retained fingerprint/disposition/evidence field is tampered -> validator rejects it.
8. A controlled AF-02 test fixture passes only on a retry -> nextest AF-02 profile returns failure; ordinary product tests do not contain a permanently flaky fixture.
9. A known plausible mutation in a required critical module survives -> mutation gate remains non-green until a test kills it or an exact reviewed waiver classifies it.
10. Coverage on a candidate drops below a frozen floor -> gate fails; the same candidate cannot silently lower the floor to pass.
11. A fuzz/property failure is fixed without a committed minimized deterministic reproducer -> AF-02 task remains incomplete.
12. A bounded discovery fuzz run finds no crash -> result is recorded as a bounded observation, not a correctness PASS claim.
13. A future parser boundary is added without AF-02 surface-manifest classification -> coverage audit fails or later feature closure remains blocked.
14. Existing mandatory `cargo fmt`, `cargo clippy`, `cargo test`, path-applicable proof/oracle workflows, and AF-01 required checks remain green on every implementation head.

## Edge cases

- A fuzz target that only reaches trivial JSON syntax rejection is not sufficient structure-aware coverage for a structured invariant surface.
- A fuzzer timeout caused by an intentionally tiny harness timeout is not automatically a product defect; timeout policy and reproducer must distinguish harness configuration from input-triggered pathological behavior.
- A mutation that cannot compile is not a killed mutant unless the frozen mutation policy explicitly classifies compiler-rejected transformations separately.
- An equivalent mutant is not silently deleted from results; it receives a narrow reviewed waiver.
- Coverage can change because compiler/tool instrumentation changes. Tool/compiler identity must therefore be part of the baseline before interpreting drift.
- Property-test shrinking can discover a smaller input than the original fuzz artifact; the smallest stable reproducer should be promoted.
- Filesystem/source-map fuzzing must use isolated temporary directories and must never follow generated paths outside the harness root.
- A scheduled fuzz workflow that is skipped/unavailable is not a clean fuzz result.
- Cross-path comparison is not called "differential oracle" when both sides use the same implementation/semantic authority.
- Existing very large legitimate package limits are product behavior, not a requirement that every fuzz iteration allocate near those limits.

## Explicit non-goals

AF-02 does not implement:

- CF-14 source-profiler product behavior or real patient-instance ingestion;
- CF-15 AutoFix recipes;
- CF-16 mapping IR;
- CF-06 production oracle pin changes;
- CF-10 corpus mutation;
- Linux/Windows/macOS release qualification, MSRV release gate, SBOM, signing, provenance, or public API/SemVer guard — AF-03;
- benchmark performance budgets, large-scale resource claims, or external-service sentinel separation — AF-04;
- a stable public release claim;
- a universal trust score;
- AI/model/agent authority.
