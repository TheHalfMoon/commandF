# AF-02 Closed Verification Protocol

Status: PLANNING_CANDIDATE

This file is a **normative companion** to `evidence-contracts.md` for AF-02. It closes verifier-design choices that may not be selected after adversarial results are known.

If this file conflicts with a looser statement in `spec.md`, `plan.md`, `tasks.md`, `consistency.md`, or `evidence-contracts.md`, this stricter fail-closed protocol controls. A weakening requires a dedicated policy/verifier PR evaluated under the previously canonical contract and merged before dependent work.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

## 1. Temporal planning-gate boundary

T005/T006 exact-head qualification is not embedded as a self-referential PASS inside the commit being qualified. A commit cannot contain immutable evidence of its own future review completion, merge result, or post-merge read-back without changing its head.

The temporal sequence is therefore fixed:

1. qualify the exact current PR head with all path-applicable workflows;
2. obtain fresh Qodo and CodeRabbit review on that same head;
3. disposition every substantive finding and require zero unresolved substantive threads;
4. merge only with an expected-head guard;
5. re-read canonical `main`/tree and both live AF-01 rulesets after merge;
6. only then close T006 and authorize Stack A0 design-freeze work.

Absence of future merge/post-merge evidence before merge is an open temporal gate, not a waiver and not an implementation PASS.

## 2. Canonical JSON used by this protocol

For every deterministic projection/object in this file:

- schema/type/range validation occurs before hashing;
- floats are prohibited;
- object keys are recursively sorted by UTF-8 byte order;
- schema-defined set arrays are sorted by their stated stable key and deduplicated;
- order-significant arrays retain schema order;
- strings are exact parsed Unicode values;
- integers use minimal base-10 JSON form;
- booleans/null use JSON literals;
- output is compact UTF-8 JSON with no insignificant whitespace and no trailing newline.

`SHA256_CANONICAL(x)` means lowercase hex SHA-256 over those canonical bytes.

## 3. Closed authority projection protocol

Schema: `commandf.af02-authority-projection/v1`.

Unknown semantic fields, missing required fields, wrong cardinality, duplicate set members, source disagreement, or source unavailability fail closed.

### 3.1 AF-01 assurance ruleset

Authoritative live source:

```text
GET /repos/TheHalfMoon/commandF/rulesets/21652953
```

Observation-only API metadata excluded from the projection is limited to timestamps, API/HTML links, node IDs, and `current_user_can_bypass`. No semantic rule field is silently ignored.

The normalized object is exactly:

```json
{
  "id": 21652953,
  "name": "commandF main assurance",
  "target": "branch",
  "source_type": "Repository",
  "source": "TheHalfMoon/commandF",
  "enforcement": "active",
  "ref_include": ["refs/heads/main"],
  "ref_exclude": [],
  "bypass_actors": [],
  "deletion": true,
  "non_fast_forward": true,
  "required_status_checks": {
    "strict_required_status_checks_policy": true,
    "do_not_enforce_on_create": false,
    "checks": [
      {"context": "assurance-proof", "integration_id": 15368},
      {"context": "rust", "integration_id": 15368},
      {"context": "scorecard", "integration_id": 15368}
    ]
  }
}
```

Validation requires exactly three live rules: `deletion`, `non_fast_forward`, and `required_status_checks`. The check array is sorted by `context` before hashing and must contain exactly those three entries.

Closed-v1 canonical digest:

```text
6177b1b8777665506797d5e0cb3f48da81cc748e31bb2c9b53b4b1da777df00a
```

The older `7a6d13...` digest in `evidence-contracts.md` belongs to the earlier looser projection and is superseded for `commandf.af02-authority-projection/v1`.

### 3.2 AF-01 review-governance ruleset

Authoritative live source:

```text
GET /repos/TheHalfMoon/commandF/rulesets/21652974
```

The normalized object is exactly:

```json
{
  "id": 21652974,
  "name": "commandF main review governance",
  "target": "branch",
  "source_type": "Repository",
  "source": "TheHalfMoon/commandF",
  "enforcement": "active",
  "ref_include": ["refs/heads/main"],
  "ref_exclude": [],
  "bypass_actors": [
    {"actor_id": 5, "actor_type": "RepositoryRole", "bypass_mode": "pull_request"}
  ],
  "pull_request": {
    "required_approving_review_count": 1,
    "dismiss_stale_reviews_on_push": true,
    "required_reviewers": [],
    "require_code_owner_review": true,
    "require_last_push_approval": true,
    "required_review_thread_resolution": true,
    "require_extra_approval_for_unattributed_changes": true,
    "allowed_merge_methods": ["merge"]
  }
}
```

Validation requires exactly one live `pull_request` rule and exactly one bypass actor equal to the object above.

Closed-v1 canonical digest:

```text
9a54cd03bbf04449e1a00ff86701dfce98a700d392485f849774a63234347d0d
```

The older `a72fd9...` digest in `evidence-contracts.md` belongs to the earlier looser projection and is superseded for this closed schema.

### 3.3 CF-06 production-oracle projection

AF-02 derives CF-06 from canonical-base repository sources, never candidate AF-02 prose.

Required sources:

```text
crates/commandf-pkg/src/oracle_model.rs
donors/hl7-fhir-validator-6.10.2.yaml
.github/workflows/cf06-oracle.yml
```

Closed derivation:

1. Parse exact Rust constants `HL7_ORACLE_PROJECT`, `HL7_ORACLE_RELEASE`, `HL7_ORACLE_SOURCE_COMMIT`, and `HL7_VALIDATOR_JAR_SHA256`.
2. Parse donor source `id=hl7-fhir-core-validator`; require repository/ref/tag/release-artifact SHA-256 to agree with those constants.
3. Parse every command token matching `hl7.fhir.r4.core@<version>` in canonical `cf06-oracle.yml`; the non-empty version set must be exactly `{4.0.1}`.
4. Construct only:

```json
{
  "project": "hapifhir/org.hl7.fhir.core",
  "release": "6.10.2",
  "source_commit": "d06577dbc5c62c74a2a8823fbc4830a3024d5b0b",
  "validator_cli_jar_sha256": "a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6",
  "r4_core_context": "hl7.fhir.r4.core@4.0.1"
}
```

Closed canonical digest:

```text
236d71b6816978a4f7c9ea587d70301801b91a1d8a038f93ac7940203dc62787
```

The verifier also records raw SHA-256 for the three authoritative source files. Candidate edits to them are authority changes and cannot self-authorize AF-02.

### 3.4 CF-10 frozen-corpus projection

Authority is read from retained CF-10 evidence, not candidate AF-02 files:

```text
PR: 11
head: 5fe10d9859407272acf6649fc3e868d3eb2fbd12
base: 5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
manifest: corpus/real-ig/v1/corpus.json
donor: donors/cf-10-real-ig-delta-corpus.yaml
run: 31916124080
artifact_id: 9255732702
artifact_name: cf10-real-corpus-evidence
artifact_sha256: 9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612
observed_run_conclusion: failure
```

The run conclusion remains `failure`; AF-02 must not relabel retained evidence as a successful CF-10 production gate.

Manifest requirements:

```text
schema == 1
selection_policy == frozen_pre_result_v1
cases length == 3
case ids exactly [C001,C002,C003]
ids sorted and unique
```

The three deltas are exactly:

```text
C001 hl7.fhir.us.core   8.0.1 -> 9.0.0
C002 hl7.fhir.uv.ips    1.1.0 -> 2.0.1
C003 hl7.fhir.us.mcode  3.0.0 -> 4.0.0
```

The verifier expands them into exactly six state records sorted by `state_id`:

```text
C001-after  hl7.fhir.us.core   9.0.0  d7b54d2ec2a48cea94ffea5d939ad67a681f80b94d69594a08cebac36da9e059  2749959
C001-before hl7.fhir.us.core   8.0.1  3c02eef48ef10617021bee95e58cbc66d596ceda8cada24b72000d33ad67c464  2713046
C002-after  hl7.fhir.uv.ips    2.0.1  7183242b70fb2a9058aa3701fb607517a3c2fd0e3100d1d8c538d744c2adf799   725312
C002-before hl7.fhir.uv.ips    1.1.0  403c4141101810e924f2928287985084819d8a5cc3a62e2b3840a557129840ef  1065103
C003-after  hl7.fhir.us.mcode  4.0.0  e603283bafa508a3705ad022bce95bba1fbd0b8b3b87b978e7412813b7bc1778  1003918
C003-before hl7.fhir.us.mcode  3.0.0  c94c91971747efeae760aa037d168e4df992cefb6dacece08217c464b9d39214  1014084
```

Each state schema is exactly:

```text
state_id
case_id
side
package
version
archive_sha256
archive_bytes
```

The retained donor must independently agree on package/version pairs, archive SHA-256, byte sizes, `fhir_version=4.0.1`, metadata-only/no-redistribution intent, and six selected states.

Live GitHub read-back must prove the retained PR/head/base/run/artifact identity and the full artifact digest above. Artifact expiry is metadata only.

`commandf.af02-authority-baseline/v1` therefore requires:

```text
schema
captured_from_main_sha
af01_rulesets
cf06 projection + authoritative_source_sha256s
cf10.deltas[3]
cf10.states[6]
cf10.retained_pr
cf10.retained_head
cf10.retained_base
cf10.retained_run
cf10.retained_run_conclusion
cf10.retained_artifact_id
cf10.retained_artifact_name
cf10.retained_artifact_sha256
cf10.retained_manifest_sha256
cf10.retained_donor_sha256
```

No listed field is optional. Candidate edits to the baseline are compared against independently reconstructed authority and cannot self-authorize.

## 4. Closed proof object

Schema: `commandf.af02-adversarial-proof/v1`.

Every object rejects unknown fields. Every field below is mandatory unless named `*_or_null`. Set arrays use the stated sort key and reject duplicates.

Top-level object is exactly:

```text
schema
deterministic
stochastic_observations[]
af02_adversarial_sha256
```

Only `deterministic` is hashed. `af02_adversarial_sha256 = SHA256_CANONICAL(deterministic)`.

`deterministic` contains exactly these keys:

```text
source
contract_files[]
authority
tool_lock[]
surface
resources
corpus
properties
nextest
coverage
mutation
canonical_cargo_test
required_checks[]
```

### 4.1 source

Exactly:

```text
sha
tree
canonical_base_sha
canonical_base_tree
```

### 4.2 contract_files[]

Each entry:

```text
path
blob_sha
sha256
role
```

Exactly one role each:

```text
spec
plan
tasks
consistency
evidence_contracts
verification_protocol
donor_manifest
authority_baseline
surface_policy
resource_policy
tool_lock
corpus_manifest
assertion_registry
coverage_policy
mutation_policy
enforcement_inventory
```

Sorted by `path`. Duplicate role/path fails.

### 4.3 authority

Exactly:

```text
baseline_file_sha256
af01_assurance_projection_sha256
af01_review_projection_sha256
cf06_projection_sha256
cf10_projection_sha256
live_readback_completed
```

`live_readback_completed` must be true and all projections must match independently derived authority.

### 4.4 tool_lock[]

Executable entry:

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

Registry-package entry instead contains exact package/version/source/checksum. Mixed shapes fail. Sorted by `id`; features sorted/deduped.

### 4.5 surface

Exactly:

```text
policy_sha256
scanner_source_sha256
scanner_dependency_lock_sha256
source_universe_sha256
discovery_inventory_sha256
classified_boundary_count
reviewed_exclusion_count
stale_entry_count
unclassified_boundary_count
```

`stale_entry_count` and `unclassified_boundary_count` must be zero.

### 4.6 resources

Exactly:

```text
policy_sha256
runner_image_digest
runtime_identity_sha256
effective_enforcement_sha256
offline_probe_sha256
network_mode
memory_bytes
nano_cpus
pids_limit
tmpfs_bytes
source_mount_read_only
output_mount_path
output_mount_read_write
```

### 4.7 corpus

Exactly:

```text
manifest_sha256
assertion_registry_sha256
fixture_inventory_sha256
scenario_count
total_fixture_bytes
replay_result_sha256
orphan_manifest_count
orphan_assertion_count
provenance_violation_count
```

All three final counts must be zero.

### 4.8 properties

Exactly:

```text
config_sha256
model_registry_sha256
case_count
passed_count
failed_count
counterexample_inventory_sha256
```

`failed_count` must be zero.

### 4.9 nextest

Exactly:

```text
config_sha256
command_argv_sha256
selected_test_name
state_protocol_sha256
junit_sha256
process_exit_code
first_attempt_class
retry_attempt_class
normalized_class
ordinary_suite_result_sha256
```

The policy fixture requires `FAIL`, `PASS`, `FLAKY_RETRY_PASS`, and non-zero process exit.

### 4.10 coverage

Exactly:

```text
descriptor_sha256
raw_report_sha256
source_universe_sha256
file_metrics_sha256
workspace_covered_lines
workspace_total_lines
workspace_floor_percent
critical_surface_metrics_sha256
unknown_production_path_count
missing_production_path_count
duplicate_normalized_path_count
```

The three final counts must be zero.

### 4.11 mutation

Exactly:

```text
policy_sha256
inventory_sha256
required_set_sha256
exclusion_set_sha256
result_inventory_sha256
required_count
killed_count
survived_count
timeout_count
unviable_or_build_failure_count
waived_count
unclassified_count
```

After mandatory retry/diagnosis, `survived_count`, `timeout_count`, `unviable_or_build_failure_count`, and `unclassified_count` must all be zero. Waivers must predate the implementation candidate.

### 4.12 canonical_cargo_test

Exactly:

```text
command_argv_sha256
source_sha
exit_code
test_count
passed_count
failed_count
ignored_count
raw_result_sha256
```

Exit must be 0 and failed count 0.

### 4.13 required_checks[]

Exactly three entries sorted by `context`:

```text
context
integration_id
check_run_id
head_sha
conclusion
```

Contexts must be `assurance-proof`, `rust`, `scorecard`; each integration ID is `15368`, `head_sha` equals the exact candidate, and conclusion is `success`. Duplicate/same-name foreign-app contexts fail.

### 4.14 stochastic_observations[]

Closed metadata-only shape:

```text
campaign_id
target_id
source_sha
tool_lock_entry_sha256
campaign_config_sha256
seed_or_null
wall_seconds
executions_or_null
corpus_start_sha256
corpus_end_sha256
outcome_class
artifact_manifest_sha256_or_null
started_at_utc
completed_at_utc
```

Allowed outcomes only:

```text
NO_CRASH_OBSERVED_WITHIN_BOUND
DEFECT_DISCOVERED
INCOMPLETE_RESOURCE_OR_HARNESS_FAILURE
CANCELLED_OR_SUPERSEDED
```

These observations never enter the deterministic digest.

## 5. Enforcement-code anti-forgery

AF-02 requires `commandf.af02-enforcement-inventory/v1` because protecting policy files alone is insufficient.

Every acceptance-changing code/config path has one or more roles:

```text
AUTHORITY_PROJECTOR
SURFACE_SCANNER
SURFACE_POLICY_PARSER
RESOURCE_RUNNER
RESOURCE_POLICY_PARSER
TOOL_ACQUISITION_VERIFIER
CORPUS_MANIFEST_PARSER
ASSERTION_REGISTRY_PARSER
REPLAY_RUNNER
RESULT_NORMALIZER
NEXTEST_RESULT_PARSER
COVERAGE_REPORT_PARSER
COVERAGE_POLICY_PARSER
MUTATION_INVENTORY_PARSER
MUTATION_RESULT_PARSER
MUTATION_POLICY_PARSER
BASE_POLICY_COMPARATOR
PROOF_BUILDER
PROOF_VERIFIER
AF02_WORKFLOW
AF02_ACTION_OR_SCRIPT
```

Each inventory entry is exactly:

```text
role
path
blob_sha_at_policy_base
language_or_format
entry_symbol_or_job
owned_test_paths[]
```

Sorted by `(role,path)` and unique. Every role must be represented before final AF-02 proof.

Before candidate policy is evaluated, the verifier compares candidate/base trees. Any change to an inventoried path, AF-02 workflow, AF-02 contract/policy, verifier/discovery namespace, or result parser is an acceptance-authority change.

Such a change may not self-qualify solely under its changed implementation. It must either:

1. be accepted by the canonical-base verifier/parser suite and independently yield the same normalized result/digest under the candidate verifier; or
2. be a dedicated policy/verifier PR containing only authority changes plus tests, evaluated under previous canonical rules, independently reviewed, merged first, and used only by a later dependent candidate.

If the old verifier cannot parse a deliberate versioned strengthening, path 2 is mandatory.

Bootstrap: Stack A0 may introduce the first AF-02 executable enforcement implementation because none exists earlier, but A0 contains no dependent fuzz/property result used for closure. It is judged against these canonical planning contracts, ordinary repository gates, AF-01, and independent review. After A0 merges, executable base-verifier anti-forgery is mandatory.

The enforcement inventory itself cannot be reduced in the same candidate that removes/changes an enforcement path.

## 6. Deterministic surface discovery

AF-02 chooses AST discovery; implementation may not choose AST versus lexical scanning after seeing results.

### 6.1 Parser identity

Canonical scanner parser dependency is:

```text
crate: syn
version: =3.0.3
source: registry+https://github.com/rust-lang/crates.io-index
features: [full, visit]
```

`syn@3.0.3` is already present in the canonical locked graph. A0 derives its exact registry checksum from canonical-base `Cargo.lock`, records that checksum in the scanner dependency lock, and fails if name/version/source/checksum are ambiguous or disagree. Selecting another parser/version requires a dedicated policy change before scanner implementation.

### 6.2 Source universe

From the exact Git tree:

1. enumerate Git-tracked regular files;
2. retain UTF-8 `.rs` files under `crates/**/src/**` and `tools/**/src/**`;
3. remove only exact previously canonical reviewed-exclusion paths;
4. normalize to repository-relative slash paths;
5. sort by UTF-8 bytes;
6. hash the canonical `{path,blob_sha}` array.

Generated files outside Git are absent. Git-tracked generated/dead/cfg-disabled Rust under production roots remains included unless an exact prior exclusion proves it non-production. Parse failure is qualification failure.

### 6.3 Scanner semantics

Repository-owned Rust scanner using pinned `syn` must:

- parse every source-universe file;
- ignore comments/string literals as non-executable syntax;
- visit all items/expressions regardless of cfg/dead-code reachability;
- inspect macro definitions/invocations conservatively: frozen boundary tokens create a candidate boundary unless an exact exclusion classifies non-executable metadata;
- construct and resolve module-local `use`/`as` aliases;
- treat glob imports of a frozen boundary module as candidate boundaries requiring classification;
- discover method-call boundaries using frozen constructor/import + method-name pairs when receiver types cannot be resolved syntactically;
- record path, byte span, category, normalized callee/matcher, and enclosing symbol when available.

Frozen categories remain serde/text parse, archive/compression, filesystem, network/acquisition, cache/persistence, and subprocess.

Each finding has exactly one disposition:

```text
CRITICAL_SURFACE:<surface_id>
REVIEWED_EXCLUSION:<exclusion_id>
```

Zero/multiple dispositions fail. Every surface/exclusion reverse-resolves to current findings unless explicitly `NON_SCANNER_POLICY_ENTRY` with independent rationale. Stale paths/spans/symbols fail.

Proof retains source-universe/scanner/dependency/matcher/raw-findings/classified-findings digests, file count, boundary count, unclassified count, and stale count. The last two must be zero.

## 7. Deterministic mutation selection

AF-02 uses **all-listed-within-frozen-scope**, not a discretionary prioritized subset.

Before execution Stack C0 freezes target source paths, exact reviewed exclusions, and exact cargo-mutants tool/config/command identity.

Then:

1. run pinned cargo-mutants JSON listing over exact source and target paths;
2. normalize every listed mutant;
3. every listed mutant inside target paths is `REQUIRED` unless it matches exactly one previously frozen exclusion;
4. every excluded mutant is `EXCLUDED:<exclusion_id>`;
5. every inventory record has exactly one disposition;
6. top-N, percentages, operator preference, post-result cherry-picking, or manual “interesting mutant” selection is prohibited.

Stable mutant ID is SHA-256 canonical JSON over:

```text
source_path
source_blob_sha
start_line
start_column
end_line
end_column
enclosing_function_or_null
mutation_description
mutant_diff_sha256
cargo_mutants_tool_lock_entry_sha256
mutation_policy_sha256
```

Missing span/function values are explicit null. Duplicate IDs fail. Required-set digest hashes sorted required IDs. Exclusion-set digest hashes sorted `{mutant_id,exclusion_id}`.

All required mutants remain required even if later TIMEOUT/UNVIABLE. Those outcomes close only through mandatory bounded retry + diagnosis and previously canonical waiver governance.

## 8. Assertion/replay registry

Schema: `commandf.af02-assertion-registry/v1`.

Top level exactly:

```text
schema
entries[]
```

Each entry exactly:

```text
assertion_id
scenario_id
surface_id
runner_kind
manifest_path
package_or_binary
cargo_target_or_null
test_name_or_null
argv[]
cwd_repo_relative
environment_allowlist{}
expected_outcome
result_parser_id
source_paths[]
config_sha256s[]
```

Runner kinds only `CARGO_TEST` or `AF02_REPLAY_BINARY`. Shell command strings are prohibited; argv is authority. Environment keys are explicit sorted allowlist entries and contain no secrets.

Binding is bijective:

- each corpus scenario has exactly one registry entry;
- each registry entry maps to exactly one scenario;
- assertion/scenario/runner-target-test identities are unique;
- every referenced path exists at source SHA;
- surface exists in policy;
- expected outcome is allowed by that surface.

For `CARGO_TEST`, a frozen inventory command lists exact tests and `test_name` must appear once. For replay binary, exact binary digest/argv plus repository-owned list/inventory mode proves target existence.

Inventory digest is bound to source SHA, Cargo.lock, relevant manifests/configs, and registry digest.

Replay result exactly:

```text
assertion_id
scenario_id
runner_kind
process_exit_code
normalized_outcome
raw_stdout_sha256
raw_stderr_sha256
structured_result_sha256_or_null
```

Repository-owned parser derives `normalized_outcome`. Orphans, missing targets, argv/parser/source/config drift, or outcome mismatch fail.

## 9. Canonical resource/offline proof

Canonical AF-02 qualification uses Linux OCI enforcement. “Equivalent mechanism” is not accepted for canonical proof lanes.

Proof-critical base image:

```text
docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f
machine: x86_64
stable Rust baseline: 1.97.1
```

Network-enabled acquisition occurs first with immutable provenance verification. Deterministic execution then uses acquired caches/tools with network denial.

Minimum canonical container controls:

```text
--network none
--cpus 2
--memory 768m
--pids-limit 256
--read-only
--tmpfs /tmp:rw,noexec,nosuid,size=512m
source checkout mount: read-only
AF-02 output mount: dedicated read-write
```

Docker socket, host network, privileged mode, device mounts, and arbitrary host-path writes are prohibited. Children remain inside the same network namespace/cgroup.

Preflight must inspect and prove:

```text
image repo digest exact
network mode == none
memory == 805306368 bytes
NanoCPUs == 2000000000 or equivalent inspect fields proving exactly 2 CPUs
PidsLimit == 256
rootfs read-only
/tmp tmpfs == 536870912 bytes + noexec + nosuid
source mount read-only
output mount read-write inside dedicated root
not privileged
no docker socket/device mount
```

Negative probes inside that same container must prove public-network/DNS reachability is denied, source-tree write fails, output-root write succeeds, and write outside allowed writable roots fails. Raw output SHA-256, argv, exit statuses, and normalized preflight result are evidence.

Harness policy additionally enforces per-input timeout, input bytes, generated/decompressed byte counter, temporary-file count, subprocess timeout, artifact size, aggregate corpus size, and retention. Missing/ambiguous enforcement is incomplete, never PASS.

## 10. Closed coverage accounting

Coverage source authority is Git, not whatever files the coverage producer chooses to report.

### 10.1 Source universe

Exact source SHA -> every tracked regular `.rs` under `crates/<crate>/src/**`, recursively, minus only exact previously canonical non-product exclusions. Normalize repo-relative slash paths; reject absolute/`..`/duplicate/symlink ambiguity. Sort and hash `{path,blob_sha}`.

A zero-hit production source still belongs to the denominator.

### 10.2 llvm-cov JSON parser

Using the frozen single merged `cargo llvm-cov --workspace --all-features --locked --json` report:

- require one logical merged data set;
- normalize each physical source-file path to repo-relative form;
- use integer `summary.lines.count` and `summary.lines.covered` only for line-floor authority;
- each production source path appears exactly once;
- every source-universe path appears, including zero-covered files;
- unknown production-root path absent from Git universe fails;
- duplicate normalized path fails rather than summing twice;
- outside-root paths are ignored only by exact frozen exclusion;
- unknown non-excluded path fails;
- require `0 <= covered <= count` with checked integer arithmetic.

Workspace totals are integer sums over authoritative production files. Critical-surface totals sum only exact frozen file/path scope.

Macro/generated behavior is physical-source based: if llvm-cov maps code to a tracked production `.rs`, it counts. Untracked compiler/build output is ignored only under a frozen exclusion. AF-02 invents no synthetic macro lines.

Function/region data remains diagnostic and cannot alter line denominator.

Floor for `total > 0`:

```text
floor_percent = (covered * 100) // total
```

Zero/missing total for a critical surface fails. Descriptor/source-universe/exclusion/floor weakening is base-controlled and cannot self-green.

## 11. Closed nextest retry-pass fixture

Frozen fixture identity:

```text
root: tests/assurance/af02-nextest-flake-fixture/
manifest: tests/assurance/af02-nextest-flake-fixture/Cargo.toml
integration target: retry_pass_policy
selected test: af02_retry_pass_is_failure
workspace member: no
network: denied
```

AF-02 argv exactly:

```text
cargo
nextest
run
--manifest-path
tests/assurance/af02-nextest-flake-fixture/Cargo.toml
--profile
ci
--retries
2
--flaky-result
fail
-E
test(af02_retry_pass_is_failure)
```

Before invocation the runner creates a dedicated unprivileged AF-02 temp root, removes prior state, proves the state path does not exist and has no symlink component, and sets `AF02_NEXTEST_STATE_FILE` plus only frozen allowlisted environment.

Fixture algorithm exactly:

1. validate state path is absolute and beneath assigned AF-02 temp root;
2. atomic `create_new(true)` regular file with bytes `first-attempt\n`;
3. successful creation -> intentionally fail test;
4. existing file -> require regular non-symlink file with exact bytes -> pass;
5. all other states -> fail.

No clock, PID ordering, RNG, network, scheduler timing, sleep, external service, or previous run controls behavior. Cleanup after evidence capture is mandatory; cleanup failure fails.

### 11.1 JUnit/result parser

Repository `.config/nextest.toml` freezes profile `ci` with retries 2, `flaky-result="fail"`, bounded slow timeout, and a deterministic JUnit output path.

Pinned nextest 0.9.143 JUnit parser acceptance is closed to these predicates:

- exactly one selected `<testcase>` corresponding to `af02_retry_pass_is_failure`;
- no `<skipped>` classification;
- JUnit contains failure-on-flaky evidence: the selected testcase has final failure representation plus at least one `<flakyFailure>` or `<flakyError>` retry-history element;
- state file exists with exact fixed bytes after the run, proving the first-attempt transition occurred;
- process exit is non-zero;
- selected-test count is exactly 1.

The repository-owned parser maps only that evidence to:

```text
first_attempt_class = FAIL
retry_attempt_class = PASS
normalized_class = FLAKY_RETRY_PASS
```

Missing JUnit, ambiguous/multiple testcase, zero exit, malformed XML, missing flaky history, unexpected state file, or weaker override fails.

Raw evidence retains command argv SHA-256, nextest executable/config SHA-256, fixture source/manifest SHA-256, state-protocol SHA-256, JUnit/stdout/stderr SHA-256, and process exit.

The result parser itself is an anti-forgery inventoried authority path.

## 12. Independent verifier reconstruction order

The proof verifier executes and fails closed in this order:

1. derive source SHA/tree and canonical base SHA/tree;
2. load canonical-base contracts/enforcement inventory;
3. classify candidate policy/verifier/enforcer changes;
4. validate candidate contracts under base anti-forgery rules;
5. reconstruct AF-01/CF-06/CF-10 projections from authoritative sources;
6. verify immutable tool/package identities;
7. derive source universe and boundary inventory;
8. verify resource/offline runtime enforcement;
9. hash raw corpus fixtures and validate assertion/replay registry;
10. parse raw property/nextest/coverage/mutation/cargo-test outputs;
11. verify required GitHub check-run uniqueness/provenance on exact candidate head;
12. construct the closed deterministic proof object itself;
13. canonicalize and recompute `AF02_ADVERSARIAL_SHA256`;
14. compare producer artifact and reject any extra/missing/mismatched field or digest.

Proof builder and verifier are separate inventoried entry symbols/modules.

Required negative tests include forged producer summary, forged normalized result, verifier-code change, base-policy change, unknown deterministic field, missing field, duplicate/reordered set, changed raw-output digest, authority drift, assertion orphan, coverage missing path, mutation omitted required ID, and retry-pass converted to zero exit.

## 13. Round-2 review disposition boundary

This protocol closes the normative Qodo findings returned on PR #54 head `ce93767c7e4c3f569ed6c4575d2bbd4c7dda310b`:

- exact CF-10 3-delta/6-state cardinality and retained PR/base/run/artifact-name/digest identity;
- exact AF-01/CF-06/CF-10 projection sources/schema/canonicalization;
- self-consistent closed-v1 AF-01 projection digests;
- closed deterministic proof and stochastic metadata schemas;
- verifier/scanner/parser/workflow anti-forgery inventory;
- all-listed-within-frozen-scope mutation selection;
- one pinned AST scanner model using canonical `syn@3.0.3` plus exact lock checksum;
- assertion/replay registry with bijection/argv/parser/source/config binding;
- fixed digest-pinned OCI resource/offline enforcement and negative probes;
- complete Git-derived coverage source accounting;
- fixed nextest fixture/state/argv/JUnit/process-exit protocol.

The only intentionally open planning observation is temporal T005/T006 exact-head review/merge/post-merge evidence. This file does not convert that future gate to PASS.
