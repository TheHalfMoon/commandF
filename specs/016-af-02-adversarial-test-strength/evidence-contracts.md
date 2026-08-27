# AF-02 Normative Evidence Contracts

Status: PLANNING_CANDIDATE

This file is normative for `AF-02 Adversarial Test Strength`. It resolves implementation details that must not be selected opportunistically after adversarial results are known.

If this file conflicts with a looser statement in `spec.md`, `plan.md`, or `tasks.md`, the stricter fail-closed rule applies. A future change that weakens these contracts requires an explicit reviewed policy-change PR evaluated against the previously canonical contract.

## 1. Canonical planning authority snapshot

AF-02 planning begins from:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

The following external/canonical authorities are frozen inputs, not AF-02-owned semantics.

### 1.1 AF-01 live source-control policy

The semantic baseline deliberately excludes timestamps, API links, node IDs, and `current_user_can_bypass`; those are observations, not policy semantics.

Assurance ruleset:

```text
id: 21652953
name: commandF main assurance
target: branch
source_type: Repository
source: TheHalfMoon/commandF
enforcement: active
ref include: refs/heads/main
ref exclude: []
bypass_actors: []
rules:
  deletion
  non_fast_forward
  required_status_checks:
    strict_required_status_checks_policy: true
    do_not_enforce_on_create: false
    rust / integration_id 15368
    assurance-proof / integration_id 15368
    scorecard / integration_id 15368
semantic canonical-json sha256:
  7a6d13ea8b63d247f97eb091c6b189b116640bfdeb21f42cd22e873ab404f8f4
```

Review-governance ruleset:

```text
id: 21652974
name: commandF main review governance
target: branch
source_type: Repository
source: TheHalfMoon/commandF
enforcement: active
ref include: refs/heads/main
ref exclude: []
bypass_actors:
  RepositoryRole actor_id=5 / pull_request only
pull_request rule:
  required_approving_review_count: 1
  dismiss_stale_reviews_on_push: true
  required_reviewers: []
  require_code_owner_review: true
  require_last_push_approval: true
  required_review_thread_resolution: true
  require_extra_approval_for_unattributed_changes: true
  allowed_merge_methods: [merge]
semantic canonical-json sha256:
  a72fd9bcb1fe2584ff544d7f09981cb012dcf4d7f50f9dd8cb7d1559c64e5320
```

Every AF-02 stack and final proof MUST perform a live GitHub API read-back and recompute these semantic projections. Drift fails AF-02 qualification and requires reconciliation. This assertion is always run; it is not conditional on AF-02 proposing a new required check.

### 1.2 CF-06 production oracle authority

AF-02 MUST retain and fail closed on drift from this production contract unless a separately authorized CF-06 unit has already changed it canonically:

```text
project: hapifhir/org.hl7.fhir.core
release: 6.10.2
source_commit: d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
validator_cli_jar_sha256: a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6
R4_core_context: hl7.fhir.r4.core@4.0.1
```

The AF-02 proof MUST parse/derive these values from the canonical repository authority/configuration where they are represented and compare them to an authority-baseline record. A changed production pin is not an AF-02 test improvement.

### 1.3 CF-10 frozen corpus authority

The retained frozen corpus identity is:

```text
C001  hl7.fhir.us.core   8.0.1 -> 9.0.0
C002  hl7.fhir.uv.ips    1.1.0 -> 2.0.1
C003  hl7.fhir.us.mcode  3.0.0 -> 4.0.0
```

Retained six-state reproof identity:

```text
PR: 11
head: 5fe10d9859407272acf6649fc3e868d3eb2fbd12
base: 5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
run: 31916124080
artifact_id: 9255732702
artifact_name: cf10-real-corpus-evidence
artifact_sha256: 9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612
```

AF-02 MUST retain an authority-baseline record containing the six package states and the immutable retained-evidence identity above. It MUST fail if AF-02 changes, replaces, drops, or reinterprets those cases. AF-02 is not authorized to merge PR #11 or change the CF-06 production pin.

## 2. `commandf.af02-authority-baseline/v1`

Implementation MUST add a checked-in authority-baseline file with this logical schema:

```json
{
  "schema": "commandf.af02-authority-baseline/v1",
  "captured_from_main_sha": "40 lowercase hex",
  "af01_rulesets": {
    "assurance": {"ruleset_id": 0, "semantic_sha256": "64 lowercase hex"},
    "review_governance": {"ruleset_id": 0, "semantic_sha256": "64 lowercase hex"}
  },
  "cf06": {
    "project": "string",
    "release": "string",
    "source_commit": "40 lowercase hex",
    "validator_cli_jar_sha256": "64 lowercase hex",
    "r4_core_context": "string"
  },
  "cf10": {
    "cases": [
      {"id": "C001", "package": "string", "before": "semver", "after": "semver"}
    ],
    "retained_head": "40 lowercase hex",
    "retained_run": 0,
    "retained_artifact_id": 0,
    "retained_artifact_sha256": "64 lowercase hex"
  }
}
```

The file is not self-authorizing. The verifier MUST derive current values independently from canonical repository/live GitHub state and compare them. Editing the baseline and the protected authority in one candidate never makes the candidate green.

## 3. `commandf.af02-surface-policy/v1`

AF-02 MUST add one machine-readable surface policy owned by repository validation code.

Required top-level fields:

```text
schema
source_roots[]
discovery_matchers[]
reviewed_exclusions[]
critical_surfaces[]
resource_profiles{}
```

### 3.1 Source roots

Initial discovery roots MUST include all production Rust sources that can accept, decode, acquire, cache, persist, read, validate, map, resolve, or execute external/untrusted data:

```text
crates/**/src/**/*.rs
tools/**/src/**/*.rs
```

A narrower implementation-time root list is allowed only when every omitted path is proven non-production/generated/test-only and recorded as a reviewed exclusion.

### 3.2 Boundary discovery matchers

The repository-owned validator MUST deterministically discover candidate boundaries. Exact implementation may use AST-aware scanning or a conservative lexical scanner, but the frozen categories are:

```text
SERDE_OR_TEXT_PARSE
  serde_json::from_*
  serde_yaml::from_*
  toml::*from_*
  parse::<...>() at retained-evidence/input seams where classified

ARCHIVE_OR_COMPRESSION
  flate/gzip decoder construction
  tar archive construction/entry iteration
  package resource scanning

FILESYSTEM
  fs::canonicalize
  fs::read / read_to_string
  File::open / create
  path containment / strip_prefix authority checks

NETWORK_OR_ACQUISITION
  registry/source fetch clients
  URL acquisition helpers
  HTTP client construction or request methods

CACHE_OR_PERSISTENCE
  package-cache reads/writes/verification
  lock/report/corpus persisted-evidence readers

SUBPROCESS
  std::process::Command::new
  Java/oracle/tool process boundaries
```

The policy records concrete implementation-time match strings/AST node kinds for each category. A new discovered production boundary that is not assigned to one `critical_surfaces` entry or one reviewed exclusion fails validation.

A stale `critical_surfaces` entry whose source path, symbol/test seam, or declared corpus path no longer resolves also fails validation.

### 3.3 Reviewed exclusions

Every exclusion requires:

```text
id
category
source_path
matcher_or_symbol
reason
non_production_or_compensating_evidence
review_reference
revisit_condition
removal_condition
```

Broad directory exclusions over product source are prohibited.

### 3.4 Critical-surface entry

Every initial surface MUST contain:

```text
id                       stable `AF02-SURFACE-*`
source_paths[]
entrypoint_or_test_seam
discovery_categories[]
evidence_modes[]
accepted_outcomes[]
resource_profile
corpus_namespace
mutation_scope
coverage_scope
independent_model_or_oracle
```

Required evidence modes are drawn only from:

```text
RAW_FUZZ
STRUCTURED_FUZZ
PROPERTY
DIFFERENTIAL_OR_CROSS_PATH
MUTATION_TARGET
COVERAGE_CRITICAL
CORPUS_REPLAY
```

Initial inventory MUST include package acquisition/cache plus archive/manifest ingestion, Lockfile V1/V2, source-map/path validation, context/canonical graph, compatibility/check/gate/fingerprint/suppression evidence, and deterministic serializers.

## 4. Outcome classes

Adversarial harnesses and deterministic replay MUST use these normalized result classes:

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

`ACCEPT_CANONICAL`, `REJECT_INVALID`, and `FAIL_CLOSED_LIMIT` are only acceptable where the surface policy explicitly permits them.

Any `UNEXPECTED_ACCEPTANCE`, `INVARIANT_VIOLATION`, `ORACLE_DIVERGENCE`, or `PANIC_OR_ABORT` fails immediately and requires minimization plus deterministic regression promotion.

A required deterministic run ending in any `HARNESS_*` class is incomplete and fails qualification until diagnosed. A stochastic discovery campaign ending in `HARNESS_*` is not a clean no-crash observation; it is `INCOMPLETE_RESOURCE_OR_HARNESS_FAILURE`.

## 5. `commandf.af02-resource-policy/v1`

Resource and network limits are checked-in executable policy, not prose.

The policy MUST include, per lane/profile:

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

Initial upper bounds, unless a reviewed design-freeze measurement proves a smaller value is necessary:

```text
PR deterministic replay/property:
  job timeout: 30 min
  offline_required: true
  max committed corpus aggregate: 8 MiB
  max promoted fixture: 256 KiB by default

fuzz target execution:
  per-input timeout: 5 s
  max input: 256 KiB
  process RSS/memory limit: 768 MiB
  cpu_count: 2
  pids_limit: 256
  temporary filesystem: 512 MiB

scheduled discovery per target:
  campaign wall: 900 s
  per-input timeout: 5 s
  max input: 256 KiB
  process memory: 768 MiB
  max retained crash artifact: 1 MiB
  max retained metadata/artifact bundle: 32 MiB

subprocess/reference-model calls:
  default subprocess timeout: 60 s unless the surface policy freezes a smaller existing product bound
```

Large-product stress limits are AF-04, not routine AF-02 fuzz iteration budgets.

### 5.1 Offline enforcement

Network denial is two-layered for deterministic AF-02 execution:

1. dependencies/tools are acquired in a separate acquisition phase with exact provenance verification;
2. deterministic qualification executes with `CARGO_NET_OFFLINE=true` and OS/container network denial.

The intended Linux mechanism is a digest-pinned container invoked with `--network none`, explicit CPU/memory/PID/tmpfs limits, a read-only source mount where practical, and dedicated writable output mounts. If the implementation platform cannot enforce the second layer, the design-freeze PR MUST select and prove an equivalent mechanism before dependent harness code is written.

Missing effective offline control when `offline_required=true` fails qualification.

## 6. `commandf.af02-tool-lock/v1`

Every adopted AF-02 tool or test-only crate has an exact retained identity.

Required fields for executable tools:

```text
id
version
upstream_repository
upstream_commit
acquisition_mode
install_command
source_lock_sha256_or_release_asset_sha256
installed_executable
installed_executable_sha256
version_output
build_rustc
build_cargo
build_target
features[]
```

Allowed executable acquisition modes are:

```text
LOCKED_GIT_REV_SOURCE_BUILD
IMMUTABLE_RELEASE_ASSET_WITH_SHA256
```

Mutable `latest`, branch-only, or tag-only acquisition is not proof identity.

For source-build mode the command MUST resolve the exact reviewed Git commit and use locked dependency resolution. CI verifies the checked-out/resolved commit, builds the executable, records the executable SHA-256, and verifies expected `--version` output.

Initial reviewed source commits are:

```text
cargo-fuzz 0.13.2
  984c861c8dfea28055254c5f1d2659ab2cd63f76

cargo-mutants 27.1.0
  8ab1dc786a1f61a4e370416cc6c68b81a704e917

cargo-llvm-cov 0.9.0
  be59056988acd54c7f984b7c85643daea3711b29

cargo-nextest 0.9.143
  60fa45f638ffc3f35e74afa65737f45fcd32db2a
```

For library/test crates (`proptest =1.11.0`, `libfuzzer-sys =0.4.13`, `arbitrary =1.4.2`) the exact crates.io package version plus Cargo registry checksum from the locked dependency graph is the package identity. The donor upstream commit is research/provenance context and does not replace the registry checksum.

Tool-lock changes are policy changes and must pass AF-01 dependency/workflow trust gates.

## 7. Property and structured-generation contract

Every property family MUST record:

```text
property_id
surface_id
generator_model
validity_domain
invalidity_mutations
max_collection_lengths
max_depth
case_count
seed_policy
shrink_policy
expected_outcome_model
independent_oracle_or_model
```

A property that invokes the same production implementation twice is not an independent differential oracle.

Initial independent models:

- **Archive/manifest:** synthetic archive builder knows the exact inserted manifest/resource identities; accepted output is compared against the synthetic model, not a second product scan.
- **Lockfile:** a small test-owned BTree-set/map model independently checks canonical ordering, exact package identities, dependency coverage, and semver constraint satisfaction.
- **Source-map/path:** a test-owned portable-path model operates on normalized slash components and explicit root-containment rules without calling the production path parser.
- **Context graph:** a small synthetic canonical-index model maps `(url, optional version)` to zero/one/many candidate outcomes independently of production graph resolution.
- **Quality gate/fingerprint:** a test-owned set/truth-table model derives new/baseline/suppressed disposition and blocking counts; canonical fingerprint key-order tests independently normalize JSON keys before comparison.

Failure seeds and shrunk cases used to close a defect MUST become deterministic corpus/regression evidence.

## 8. `commandf.af02-corpus/v1`

The committed regression corpus manifest uses this schema:

```text
schema: commandf.af02-corpus/v1
entries[]:
  scenario_id
  surface_id
  relative_path
  raw_sha256
  byte_length
  provenance_class
  public_source_url_or_null
  license_or_null
  expected_outcome
  assertion_id
  replay_command_id
  discovered_by
  minimized_from_sha256_or_null
```

Scenario IDs are stable and match:

```text
AF02-<SURFACE-SLUG>-NNNN
```

The digest is over raw fixture bytes exactly as stored; no hidden normalization is performed before hashing.

Allowed provenance classes:

```text
SYNTHETIC
PUBLIC_REDISTRIBUTABLE
```

`PUBLIC_REDISTRIBUTABLE` requires a source URL and redistribution/license basis. Unknown/private/patient-derived provenance is rejected.

Default size limits:

```text
single promoted fixture <= 256 KiB
aggregate committed AF-02 corpus <= 8 MiB
```

Exceptions require a separate reviewed policy entry and cannot be used to make the same failing candidate green.

### 8.1 Assertion binding

A manifest entry is not considered promoted merely because metadata mentions a test. Implementation MUST maintain a machine-checkable assertion registry mapping `assertion_id` to an actual deterministic test/replay target. CI MUST:

1. validate every manifest scenario has exactly one assertion binding;
2. prove the bound test/replay target exists using the test/replay inventory;
3. execute every bound regression;
4. compare the observed normalized outcome to `expected_outcome`.

Orphan manifest entries or orphan assertion bindings fail closed.

## 9. No-PHI and artifact-safety gate

AF-02 data safety is executable policy.

Every committed fixture must pass the corpus provenance gate above. In addition:

- generated fuzz artifacts are untrusted bytes and are never executed as shell scripts, binaries, workflows, Actions, or helper programs;
- discovery crash inputs are not automatically uploaded from arbitrary paths;
- retention code copies only bounded regular files from the dedicated fuzz artifact directory, records SHA-256/size, and stores them as opaque data;
- artifact names are generated from safe scenario/digest identifiers, never directly from fuzz input;
- symlinks, devices, FIFOs, sockets, executable-mode files, path traversal, and files outside the dedicated artifact root are rejected;
- logs MUST not dump full arbitrary binary inputs; only bounded escaped prefixes plus digest/size may be logged;
- a repository scan rejects fixtures without provenance classification and detects forbidden real-patient/credential fixture paths or explicit PHI markers defined by policy.

The scanner is defense in depth, not a claim that pattern matching can prove absence of all PHI. Provenance classification is the primary authority.

## 10. Nextest flaky-as-failure contract

Initial repository configuration:

```toml
[profile.ci]
retries = 2
flaky-result = "fail"
slow-timeout = { period = "60s", terminate-after = 2 }
```

The AF-02 CI invocation MUST additionally pass:

```text
--retries 2 --flaky-result fail
```

The command-line values intentionally disable per-test retry/flaky-result overrides from weakening the AF-02 lane.

Canonical `cargo test --workspace --all-features --locked` remains independently required.

### 10.1 Deterministic retry-pass self-test

The self-test lives in an isolated fixture crate that is not a workspace member and is not executed by ordinary canonical `cargo test`.

Its single test receives an AF-02-owned temporary state-file path. First invocation atomically creates the state file and fails; the retry sees the file and passes. No wall clock, RNG, network, scheduler timing, or external service controls the outcome.

AF-02 invokes nextest against that fixture with two retries and `--flaky-result fail`. Qualification requires:

```text
first attempt: FAIL
later attempt: PASS
normalized classification: FLAKY_RETRY_PASS
process exit: non-zero
```

A zero exit or a config/per-test override that converts this fixture to green fails the AF-02 policy self-test.

Any test-run cancellation, runner loss, timeout, or incomplete result is failure, not clean no-flake evidence.

## 11. Coverage baseline and floor contract

Coverage measurement is frozen before observed percentages are known.

Canonical baseline execution descriptor:

```text
platform: linux/x86_64
runner policy: AF-01 fixed supported runner
Rust: canonical 1.97.1 plus llvm-tools-preview
command family:
  cargo llvm-cov --workspace --all-features --locked --json
canonical cargo test remains separate and mandatory
source inclusion:
  production Rust under crates/*/src/**
explicit exclusions:
  target/**
  fuzz harness source
  generated build output
  isolated AF-02 fixture crates
no product-source wildcard exclusion is allowed
```

The implementation MUST record a normalized baseline descriptor containing:

```text
source_sha
source_tree
rustc_version
cargo_version
llvm_tools_identity
cargo_llvm_cov_tool_lock_entry_sha256
Cargo.lock_sha256
workspace_manifest_sha256s[]
coverage_command
features
platform
raw_coverage_report_sha256
source_inclusion_paths[]
exclusions[]
corpus_manifest_sha256
property_config_sha256
```

The raw workspace report is retained even when policy metrics use explicit source subsets.

Initial frozen floors are derived only after the descriptor is canonical:

- workspace production line floor = integer floor of measured production-line percentage;
- each `COVERAGE_CRITICAL` surface receives its own line floor = integer floor of that surface's measured percentage;
- function and region values are retained diagnostics initially, not hidden acceptance floors.

Each percentage is computed independently as `covered / total * 100`; there is no averaging of per-surface percentages.

A candidate may not change the descriptor, exclusions, or lower a floor and rely on that same change to become green. Re-baselining/floor reduction requires a dedicated reviewed policy PR evaluated under the previous canonical descriptor/floors and merged before dependent product changes.

Missing critical-source coverage or zero discovered source lines for a declared critical surface fails closed.

## 12. Mutation inventory and closure contract

Mutation evidence is bound to exact source, tool, config, and inventory.

Design-freeze MUST produce:

```text
source_sha
source_tree
cargo_mutants_tool_lock_entry_sha256
command
build_profile
test_command
jobs
baseline_mode
timeout_seconds_or_multiplier
minimum_test_timeout_seconds
build_timeout_seconds_or_multiplier
targeted_source_paths[]
reviewed_exclusions[]
mutants_json_sha256
required_mutant_ids[]
```

The frozen inventory is generated using cargo-mutants JSON listing (`--list --json`) or its exact-version equivalent. Each required mutant identity is derived from canonical JSON over the tool-provided source path/span/function/mutation description/diff plus source/tool/config identity.

Initial execution policy:

```text
parallelism: bounded and recorded; default 2 jobs
baseline: required, not skipped
minimum test timeout: 20 s
explicit maximum test timeout: 120 s per mutant unless the design-freeze proves a smaller bound
explicit maximum build timeout: 180 s per mutant unless the design-freeze proves a smaller bound
workspace test authority: canonical test command appropriate to the target crate/scope, fully recorded
```

Exact final flags/config keys MUST be verified against cargo-mutants 27.1.0 during the design-freeze and then frozen before mutation execution.

Result classes:

```text
KILLED
SURVIVED
TIMEOUT
UNVIABLE_OR_BUILD_FAILURE
WAIVED_EQUIVALENT_OR_OUT_OF_SCOPE
```

Closure rules:

- every `SURVIVED` required mutant is killed by stronger evidence or receives an exact reviewed waiver;
- every required `TIMEOUT` or `UNVIABLE_OR_BUILD_FAILURE` receives at least one bounded retry and documented diagnosis;
- after diagnosis, an unresolved timeout/unviable result must either become a classified executable result or receive the same exact reviewed waiver standard; it is never counted as killed;
- separate counts for every result class are retained;
- incomplete/cancelled mutation runs cannot qualify.

Waiver fields:

```text
mutant_id
tool_identity
source_sha
source_path
span_or_function
mutation_description
result_class
rationale
compensating_evidence
review_reference
revisit_condition
removal_condition
```

A newly added waiver cannot make the same candidate green. New waivers require a dedicated reviewed policy PR evaluated against the previous canonical mutation policy.

## 13. CI topology and partial-run semantics

AF-02 workflows use independently diagnosable jobs and explicit timeouts.

Required logical topology:

```text
acquire-and-verify-tools
  network allowed only for frozen acquisition
  produces tool-lock evidence

adversarial-deterministic
  offline + OS network denied
  surface-policy validation
  corpus replay
  property tests
  fuzz target build
  canonical cargo test remains separate

nextest-flake
  offline
  --retries 2 --flaky-result fail
  isolated retry-pass policy self-test

coverage
  offline after dependency acquisition
  exact frozen baseline descriptor/floors

mutation
  offline after dependency acquisition where practicable
  frozen inventory/config

stochastic-discovery
  scheduled/manual
  bounded campaign
  no-crash => bounded observation only

adversarial-proof
  exact-head verifier over retained deterministic inputs
  live AF-01/CF-06/CF-10 authority checks
```

Every job has explicit `timeout-minutes`, least GitHub token permissions, credentialless checkout, AF-01 full-SHA Action references, and AF-01-compliant proof container identity where a container is proof-critical.

Concurrency/cancellation:

- PR qualification may cancel superseded older heads;
- cancellation of the current exact candidate is not PASS;
- scheduled discovery may be superseded, but an incomplete campaign is labeled incomplete, never no-crash clean;
- retained evidence records completion state explicitly.

Artifact retention policy is checked in. Only validated bounded artifacts are uploaded. Raw dependency caches are not evidence artifacts.

## 14. Base-policy anti-forgery rule

A candidate cannot establish its own weaker acceptance criteria and then cite itself as proof.

For every PR candidate the verifier identifies:

```text
candidate_sha
candidate_tree
canonical_base_sha
canonical_base_tree
```

Policy authority comes from the canonical base first.

The following changes are classified as potentially weakening:

```text
coverage floor reduction
coverage source/exclusion broadening
critical-surface removal
boundary-discovery exclusion addition
resource/offline limit relaxation
flaky-result pass/override
mutation required-set reduction
mutation waiver addition
corpus assertion removal
no-PHI/provenance relaxation
AF-01/CF-06/CF-10 authority-baseline change
```

A candidate containing one of those changes MUST NOT use the weakened candidate policy to satisfy its own qualification. It must either:

1. satisfy the previous canonical policy as well, proving the change is non-weakening in effect; or
2. be a dedicated policy-change PR with explicit rationale/revisit condition, independently reviewed, merged first, and only then used by a later implementation candidate.

Strengthening changes may be applied immediately, but the candidate is evaluated against the stricter of base and candidate rules.

Generated evidence is derived from raw tool/test output by repository-owned validators. Manually editing result JSON to change a failure classification is rejected by recomputation.

## 15. `AF02_ADVERSARIAL_SHA256` canonicalization

The deterministic AF-02 proof object uses schema:

```text
commandf.af02-adversarial-proof/v1
```

The top-level artifact has:

```json
{
  "schema": "commandf.af02-adversarial-proof/v1",
  "deterministic": {},
  "stochastic_observations": [],
  "af02_adversarial_sha256": "64 lowercase hex"
}
```

Only the `deterministic` object is hashed.

### 15.1 Canonical JSON rules

Repository-owned canonicalization is:

1. input is parsed JSON after schema/type/range validation;
2. floating-point numbers are prohibited from the deterministic object; percentages are stored as integer numerator/denominator pairs and optionally integer basis points derived by the verifier;
3. object keys are recursively sorted by UTF-8 byte order;
4. arrays preserve schema-defined order; set-like data MUST be sorted/deduplicated by the producer and independently checked by the verifier;
5. strings are exact Unicode values as parsed; no timestamp/path rewriting occurs inside the canonicalizer;
6. integers serialize in minimal base-10 JSON form;
7. booleans/null use JSON literals;
8. serialization is UTF-8 compact JSON with no insignificant whitespace and no trailing newline.

Digest:

```text
AF02_ADVERSARIAL_SHA256 = lowercase_hex(SHA256(canonical_json(deterministic)))
```

### 15.2 Deterministic object contents

At minimum:

```text
source {sha, tree, canonical_base_sha, canonical_base_tree}
policy_file_sha256s
normative_contract_sha256
authority_baseline + independently observed authority values
tool_lock entries and executable/package checksums
surface policy digest/discovery result
resource policy digest/effective enforcement result
corpus manifest + raw fixture digests + assertion/replay results
property configuration + deterministic outcome summary
nextest configuration + retry-pass self-test + no-flake result
coverage baseline descriptor + floors + exact measured numerator/denominator values
mutation inventory/config + result classes + waiver identities
canonical cargo-test result identity
AF-01 required-context provenance/read-back
```

Stochastic campaign timestamps, iteration order, coverage path discoveries, and no-crash observations live under `stochastic_observations` and are excluded from the deterministic digest. A deterministic hash may include the frozen campaign configuration digest, but not pretend the stochastic outcome is reproducible semantic state.

### 15.3 Independent verifier

The verifier MUST:

- recompute source/tree identity from Git;
- hash checked-in policy/spec/contract/corpus files itself;
- recompute corpus raw-byte digests;
- parse raw test/tool outputs and derive normalized classifications;
- recompute authority semantic projections from live/canonical sources;
- verify base-policy anti-forgery rules;
- construct the deterministic object itself;
- canonicalize and recompute `AF02_ADVERSARIAL_SHA256`;
- reject a producer-supplied summary whose fields or digest do not match recomputation.

A hand-authored green summary is not evidence.

## 16. Design-freeze gates

Implementation-time measurement is allowed only to fill explicitly measured fields, not to choose hidden semantics.

Each stack starts with a separate design-freeze candidate that is reviewed and merged before dependent implementation code.

### Stack A design freeze

Must canonicalize before fuzz/property implementation:

- authority baseline;
- surface policy and deterministic discovery rule;
- resource/offline policy;
- tool-lock acquisition procedure for fuzz/property tools;
- property generator/oracle models;
- corpus schema/assertion registry design;
- no-PHI/artifact safety policy.

### Stack B design freeze

Must canonicalize before nextest/coverage enforcement:

- exact nextest invocation and deterministic retry fixture;
- exact slow-timeout maximum;
- coverage command/platform/source scope/exclusions;
- complete normalized baseline descriptor schema;
- rebaseline policy.

The measured coverage percentage itself is recorded after this design is frozen.

### Stack C design freeze

Must canonicalize before mutation/proof enforcement:

- exact cargo-mutants 27.1.0 command/config;
- source scope;
- parallelism/timeouts/build profile/test command;
- JSON inventory format and mutant-ID derivation;
- timeout/unviable retry/diagnosis policy;
- waiver governance;
- proof schema/canonicalization/verifier topology;
- CI/artifact-retention topology.

A design-freeze PR changing these semantics and implementation code depending on the new semantics in the same candidate is prohibited.

## 17. Planning review findings addressed by this contract

This contract explicitly closes the planning-design gaps raised on PR #54 head `3224098403f6bfb64525bfab002e94d5c3d82e69`:

- exact-head/live authority evidence requirements are made explicit and remain temporal T006 obligations;
- executable tool provenance schema and verification procedure;
- deterministic critical-boundary discovery and stale-entry failure;
- reproducible proof normalization and independent digest recomputation;
- base-policy anti-forgery and dedicated weakening-policy PR rule;
- frozen coverage scope/descriptor/floor aggregation semantics;
- frozen mutation inventory/config/result/timeout closure semantics;
- executable resource/offline/artifact bounds;
- independent structured-generation models;
- deterministic nextest retry-pass fixture and CLI override resistance;
- enforceable corpus IDs/raw digests/assertion binding/aggregate limits;
- always-run AF-01 policy preservation;
- explicit CF-06 and CF-10 identity preservation;
- machine-checkable provenance/no-PHI/artifact-safety gate;
- explicit CI partial-run/cancellation/retention semantics;
- mandatory design-freeze PRs before dependent implementation.

Planning still does not claim these mechanisms have been implemented. T006 may complete only after the amended exact head passes all required CI/review gates and merges with post-merge live read-back.