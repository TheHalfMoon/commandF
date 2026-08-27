# AF-02 Closed Verification Protocol

Status: PLANNING_CANDIDATE

This file is a **normative companion** to `evidence-contracts.md` for AF-02. It closes the remaining verifier-design choices identified during exact-head planning review of PR #54. It is not implementation evidence.

If this file conflicts with a looser statement in `spec.md`, `plan.md`, `tasks.md`, `consistency.md`, or `evidence-contracts.md`, the stricter fail-closed rule in this file controls. A later weakening requires a dedicated policy-change PR evaluated under the previously canonical contract and merged before any dependent candidate.

The planning base remains:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

## 1. Temporal planning-gate boundary

T005/T006 exact-head qualification is intentionally **not embedded as a self-referential PASS record in the planning commit**. A commit cannot contain immutable evidence of its own future review completion, merge result, or post-merge live-policy read-back without changing the head being qualified.

Therefore:

- planning documents define the acceptance contract;
- GitHub exact-head workflow/reviewer state supplies pre-merge temporal evidence;
- merge uses an expected-head guard;
- canonical `main` and live rulesets are re-read after merge;
- only that post-merge evidence may close T006.

A reviewer noting that post-merge evidence is absent before merge is recording an open temporal gate, not an implementation-design waiver. AF-02 remains `PLANNING_CANDIDATE` until the temporal gate actually completes.

## 2. Closed authority projection protocol

AF-02 authority verification uses schema `commandf.af02-authority-projection/v1`. Unknown semantic fields, missing required fields, duplicate set members, wrong cardinality, source disagreement, or source unavailability fail closed.

Canonical JSON for all authority projections follows the canonical JSON rules in `evidence-contracts.md`: no floats, recursively byte-sorted object keys, schema-defined array ordering, compact UTF-8 JSON, and no trailing newline.

### 2.1 AF-01 live ruleset projection

Authoritative live sources are exactly:

```text
GET /repos/TheHalfMoon/commandF/rulesets/21652953
GET /repos/TheHalfMoon/commandF/rulesets/21652974
```

Observation-only API fields such as timestamps, links, node IDs, and `current_user_can_bypass` are excluded. No other semantic rule field is silently ignored.

#### Assurance projection

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

Ruleset validation requires exactly three rules with types `deletion`, `non_fast_forward`, and `required_status_checks`. The check list is sorted by `context` before canonicalization and must contain exactly the three entries above. Any additional rule type, bypass actor, required context, or changed integration ID is authority drift.

Expected canonical SHA-256 remains:

```text
7a6d13ea8b63d247f97eb091c6b189b116640bfdeb21f42cd22e873ab404f8f4
```

#### Review-governance projection

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

Validation requires exactly one `pull_request` rule and exactly one bypass actor with the object above. `allowed_merge_methods` is a sorted set and must equal `['merge']`. Any additional semantic rule is drift.

Expected canonical SHA-256 remains:

```text
a72fd9bcb1fe2584ff544d7f09981cb012dcf4d7f50f9dd8cb7d1559c64e5320
```

### 2.2 CF-06 production-oracle projection

The verifier derives CF-06 from **canonical-base repository files**, never from candidate-edited AF-02 prose.

Required source set:

```text
crates/commandf-pkg/src/oracle_model.rs
donors/hl7-fhir-validator-6.10.2.yaml
.github/workflows/cf06-oracle.yml
```

Derivation is closed:

1. Parse the four exact Rust constant declarations in `oracle_model.rs`:
   - `HL7_ORACLE_PROJECT`
   - `HL7_ORACLE_RELEASE`
   - `HL7_ORACLE_SOURCE_COMMIT`
   - `HL7_VALIDATOR_JAR_SHA256`
2. Parse donor source `id=hl7-fhir-core-validator` and require `repository`, `ref`, `tag`, and `release_artifact.sha256` to agree exactly with those constants.
3. Parse every command token in canonical `.github/workflows/cf06-oracle.yml` matching `hl7.fhir.r4.core@<version>` and require the non-empty set of observed versions to be exactly `{4.0.1}`. A missing occurrence or mixed version fails.
4. Construct only this object:

```json
{
  "project": "hapifhir/org.hl7.fhir.core",
  "release": "6.10.2",
  "source_commit": "d06577dbc5c62c74a2a8823fbc4830a3024d5b0b",
  "validator_cli_jar_sha256": "a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6",
  "r4_core_context": "hl7.fhir.r4.core@4.0.1"
}
```

The verifier records SHA-256 for each authoritative source file. Candidate edits to any of these authority files are evaluated from the canonical base first and cannot self-authorize an AF-02 PASS.

### 2.3 CF-10 frozen-corpus projection

CF-10 authority is derived from the retained CF-10 head, not from current AF-02 candidate files:

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
observed run conclusion: failure
```

The retained workflow conclusion is deliberately recorded as `failure`; AF-02 MUST NOT relabel it as successful. The retained evidence is foundation/corpus evidence whose final enforcement was blocked by the separately governed production CF-06 comparator behavior.

The manifest parser requires:

```text
schema == 1
selection_policy == frozen_pre_result_v1
cases length == 3
case ids exactly C001, C002, C003
case ids lexicographically ordered and unique
```

Each case must contain both `before` and `after`; each state must contain exact `version`, `archive_sha256`, and `archive_bytes`. The verifier expands the three deltas into **exactly six** state records, sorted by `state_id`:

```json
[
  {
    "state_id": "C001-after",
    "case_id": "C001",
    "side": "after",
    "package": "hl7.fhir.us.core",
    "version": "9.0.0",
    "archive_sha256": "d7b54d2ec2a48cea94ffea5d939ad67a681f80b94d69594a08cebac36da9e059",
    "archive_bytes": 2749959
  },
  {
    "state_id": "C001-before",
    "case_id": "C001",
    "side": "before",
    "package": "hl7.fhir.us.core",
    "version": "8.0.1",
    "archive_sha256": "3c02eef48ef10617021bee95e58cbc66d596ceda8cada24b72000d33ad67c464",
    "archive_bytes": 2713046
  },
  {
    "state_id": "C002-after",
    "case_id": "C002",
    "side": "after",
    "package": "hl7.fhir.uv.ips",
    "version": "2.0.1",
    "archive_sha256": "7183242b70fb2a9058aa3701fb607517a3c2fd0e3100d1d8c538d744c2adf799",
    "archive_bytes": 725312
  },
  {
    "state_id": "C002-before",
    "case_id": "C002",
    "side": "before",
    "package": "hl7.fhir.uv.ips",
    "version": "1.1.0",
    "archive_sha256": "403c4141101810e924f2928287985084819d8a5cc3a62e2b3840a557129840ef",
    "archive_bytes": 1065103
  },
  {
    "state_id": "C003-after",
    "case_id": "C003",
    "side": "after",
    "package": "hl7.fhir.us.mcode",
    "version": "4.0.0",
    "archive_sha256": "e603283bafa508a3705ad022bce95bba1fbd0b8b3b87b978e7412813b7bc1778",
    "archive_bytes": 1003918
  },
  {
    "state_id": "C003-before",
    "case_id": "C003",
    "side": "before",
    "package": "hl7.fhir.us.mcode",
    "version": "3.0.0",
    "archive_sha256": "c94c91971747efeae760aa037d168e4df992cefb6dacece08217c464b9d39214",
    "archive_bytes": 1014084
  }
]
```

The donor manifest at the retained head must independently agree on package names, version pairs, archive SHA-256 values, archive byte sizes, `fhir_version=4.0.1`, metadata-only/no-redistribution intent, and six selected states.

Live GitHub read-back must additionally prove:

- PR #11 head equals retained head;
- PR #11 base identity is the retained base for that evidence run;
- workflow run `31916124080` belongs to PR #11 head `5fe10d...` and base `5cb1a4...`;
- artifact list for that run contains exactly one artifact with id `9255732702`, name `cf10-real-corpus-evidence`, and recorded digest `sha256:9fdde985...`;
- artifact expiry is metadata only and does not rewrite its immutable GitHub-recorded identity.

The authority-baseline schema therefore contains both `deltas[3]` and `states[6]` with exact cardinality plus:

```text
retained_pr
retained_head
retained_base
retained_run
retained_run_conclusion
retained_artifact_id
retained_artifact_name
retained_artifact_sha256
retained_manifest_sha256
retained_donor_sha256
```

No field is optional.

## 3. Closed proof object schema

`commandf.af02-adversarial-proof/v1` is a **closed schema**. Every object rejects unknown fields. Every listed field is required unless explicitly named `*_or_null`. Arrays have the ordering stated below. No producer may add an unverified field to the deterministic object and thereby change the digest.

Top-level schema:

```json
{
  "schema": "commandf.af02-adversarial-proof/v1",
  "deterministic": {
    "source": {},
    "contract_files": [],
    "authority": {},
    "tool_lock": [],
    "surface": {},
    "resources": {},
    "corpus": {},
    "properties": {},
    "nextest": {},
    "coverage": {},
    "mutation": {},
    "canonical_cargo_test": {},
    "required_checks": []
  },
  "stochastic_observations": [],
  "af02_adversarial_sha256": "64 lowercase hex"
}
```

Only `deterministic` is hashed. The verifier constructs `deterministic`; it does not trust a producer-created copy.

### 3.1 `source`

Exactly:

```text
sha
tree
canonical_base_sha
canonical_base_tree
```

All are 40-lowercase-hex Git identities recomputed from Git/GitHub.

### 3.2 `contract_files[]`

Each entry is exactly:

```text
path
blob_sha
sha256
role
```

Required roles are exactly one each for:

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

Entries are sorted by UTF-8 `path`; duplicate paths or roles fail.

### 3.3 `authority`

Exactly:

```text
baseline_file_sha256
af01_assurance_projection_sha256
af01_review_projection_sha256
cf06_projection_sha256
cf10_projection_sha256
live_readback_completed: true
```

The four projections are reconstructed under section 2 and must equal the checked-in previously canonical baseline.

### 3.4 `tool_lock[]`

One entry per required tool/package, sorted by `id`. Executable-tool entry is exactly:

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

`features[]` is a sorted set. Registry-package entry instead uses exact package/version/registry checksum and no executable-only fields. Mixed shapes fail schema validation.

### 3.5 `surface`

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

Qualification requires both final counts to be zero.

### 3.6 `resources`

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

Qualification requires the exact canonical enforcement values defined in section 8.

### 3.7 `corpus`

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

All three violation/orphan counts must be zero. Fixture inventory is sorted by `scenario_id` and contains raw fixture SHA-256 and byte length.

### 3.8 `properties`

Exactly:

```text
config_sha256
model_registry_sha256
case_count
passed_count
failed_count
counterexample_inventory_sha256
```

Qualification requires `failed_count=0`. A historical counterexample retained as a promoted regression may appear in the inventory but not as an unclosed current failure.

### 3.9 `nextest`

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

For the policy fixture, required values are `FAIL`, `PASS`, `FLAKY_RETRY_PASS`, and a non-zero process exit. The ordinary no-flake suite has a separate result digest.

### 3.10 `coverage`

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

The three final counts must be zero. Percentages are integer floors derived by verifier from integer covered/total pairs; no float is stored.

### 3.11 `mutation`

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

Qualification requires `survived_count=0`, `timeout_count=0`, `unviable_or_build_failure_count=0`, and `unclassified_count=0` after the mandatory retry/diagnosis process. `waived_count` may include only waivers already canonical before the implementation candidate.

### 3.12 `canonical_cargo_test`

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

Qualification requires exit 0 and failed count 0.

### 3.13 `required_checks[]`

Exactly three entries sorted by `context`:

```text
context
integration_id
check_run_id
head_sha
conclusion
```

Contexts must be `assurance-proof`, `rust`, `scorecard`, each `integration_id=15368`, exact candidate head, and `conclusion=success`. Duplicate or same-name foreign-app contexts fail.

### 3.14 Stochastic observation schema

Stochastic entries are not hashed into `AF02_ADVERSARIAL_SHA256`, but their shape is still closed:

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

Allowed `outcome_class` is only:

```text
NO_CRASH_OBSERVED_WITHIN_BOUND
DEFECT_DISCOVERED
INCOMPLETE_RESOURCE_OR_HARNESS_FAILURE
CANCELLED_OR_SUPERSEDED
```

Timestamps are metadata only and remain outside the deterministic digest.

## 4. Verifier/enforcer anti-forgery inventory

Policy-file anti-forgery is insufficient if a candidate can weaken the code that interprets policy. AF-02 therefore requires `commandf.af02-enforcement-inventory/v1`.

The inventory contains exact repository paths and roles for **all** code/config capable of changing AF-02 acceptance:

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

At design freeze, each role has at least one exact path. Entries are sorted by `(role,path)`, unique, and contain:

```text
role
path
blob_sha_at_policy_base
language_or_format
entry_symbol_or_job
owned_test_paths[]
```

The candidate/base diff is scanned before candidate policy is evaluated. A change to any inventoried path, any AF-02 workflow, any AF-02 policy/contract, or any file matching repository-owned verifier/discovery/result-parser namespaces is an **acceptance-authority change**.

An acceptance-authority change may not qualify itself under only the changed implementation. It must satisfy one of these paths:

1. **Non-weakening compatibility:** the canonical-base verifier/parser suite successfully evaluates the candidate evidence and the candidate verifier independently produces the same normalized result/digest; or
2. **Dedicated policy/verifier change:** a PR containing only the authority change plus its tests is evaluated under the previous canonical contract, independently reviewed, merged first, and only a later candidate may depend on it.

If the old verifier cannot parse a deliberately versioned strengthening, the candidate is a dedicated policy/verifier change and cannot carry dependent product/harness work.

Bootstrap rule: Stack A0 may introduce the first repository-owned AF-02 enforcement implementation because no prior AF-02 executable verifier exists. That Stack contains **no dependent fuzz/property outcome used for closure**; it is judged against these canonical planning contracts, ordinary repository tests, AF-01 gates, and independent review. After A0 merges, all later changes are subject to the executable base-verifier comparison above.

The enforcement inventory itself is base-controlled and cannot be reduced in the same candidate that removes/changes an enforcement path.

## 5. Deterministic surface discovery algorithm

AF-02 selects one discovery model; implementation may not choose between AST and lexical scanning after seeing results.

### 5.1 Source universe

The source universe is derived from the candidate Git tree, not filesystem glob side effects:

1. enumerate Git-tracked regular files;
2. retain UTF-8 `.rs` files under `crates/**/src/**` and `tools/**/src/**`;
3. remove only exact previously canonical reviewed-exclusion paths;
4. sort repository-relative slash-normalized paths by UTF-8 bytes;
5. hash canonical JSON array `{path, blob_sha}` to obtain `source_universe_sha256`.

Generated files outside Git are not in the universe. Git-tracked generated/dead/cfg-disabled Rust under the production roots remains in the universe unless an exact reviewed exclusion proves it non-production. A source-universe path that cannot be parsed is failure, not omission.

### 5.2 AST scanner semantics

The canonical scanner is repository-owned Rust code using a Rust syntax parser whose exact package version/checksum is locked in the A0 dependency inventory. Scanner source and dependency-lock digests are proof inputs.

Scanner behavior is fixed:

- parse each source-universe file as Rust syntax;
- comments and string literals are not executable AST expressions and do not create boundaries;
- visit all items/expressions regardless of `cfg` or dead-code reachability because the source is production-tracked;
- inspect macro definitions and invocations conservatively: token streams containing a frozen boundary path/name produce a candidate boundary unless an exact reviewed exclusion classifies the macro as non-executable metadata;
- construct a module-local `use`/`as` alias table and resolve aliases before call/path matching;
- glob imports of a frozen boundary crate/module are a candidate boundary and require explicit classification; they may not be silently ignored;
- method-call boundaries whose receiver type cannot be resolved syntactically are discovered through frozen constructor/import plus method-name pairs; uncertain matches become candidate boundaries rather than omissions;
- every scanner match records file, byte span, category, normalized callee/matcher, and enclosing symbol when available.

Frozen boundary families are the categories already listed in `evidence-contracts.md`: serde/text parse, archive/compression, filesystem, network/acquisition, cache/persistence, and subprocess.

The A0 surface policy records exact AST node/macro/import matcher definitions. After A0 is canonical, matcher reduction is an acceptance-authority change subject to section 4.

### 5.3 Completeness checks

The scanner output has exactly one disposition for every discovered boundary:

```text
CRITICAL_SURFACE:<surface_id>
REVIEWED_EXCLUSION:<exclusion_id>
```

Zero or multiple dispositions fail. Every critical-surface/exclusion entry must reverse-resolve to at least one current scanner finding unless explicitly marked `NON_SCANNER_POLICY_ENTRY` with independent rationale. Stale paths/spans/symbols fail.

Proof retains:

```text
source_universe_sha256
source_file_count
source_blob_count
scanner_source_sha256
scanner_dependency_lock_sha256
matcher_policy_sha256
raw_findings_sha256
classified_findings_sha256
unclassified_count
stale_entry_count
```

No source file may disappear between source-universe construction and scanner completion.

## 6. Deterministic mutation required-set algorithm

“Prioritize critical mutants” is not a selection algorithm. AF-02 instead uses **all-listed-within-frozen-scope** semantics.

Before mutation execution, Stack C0 freezes:

```text
targeted_source_paths[]
exact reviewed mutation exclusions[]
exact cargo-mutants command/config/tool identity
```

Then:

1. run pinned cargo-mutants JSON listing over the exact source tree and frozen target paths;
2. normalize every listed mutant to a stable ID;
3. every listed mutant whose source path is in `targeted_source_paths[]` is `REQUIRED` unless it matches exactly one previously frozen reviewed exclusion;
4. every excluded mutant is `EXCLUDED:<exclusion_id>`;
5. every inventory record must have exactly one disposition;
6. no heuristic top-N, percentage, operator preference, “interesting” subset, or post-result manual selection exists.

Stable mutant ID is SHA-256 over canonical JSON containing:

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

The inventory is sorted by mutant ID. Missing span/function fields are represented as explicit null, never omitted. Duplicate IDs fail.

Required-set SHA-256 is over the sorted required ID array. Exclusion-set SHA-256 is over sorted `{mutant_id, exclusion_id}` records.

Any new exclusion or target-scope reduction cannot make the same candidate green; it follows section 4/dedicated-policy rules.

All required mutants remain required even if execution later yields TIMEOUT or UNVIABLE/BUILD_FAILURE. Those execution classes must close through the retry/diagnosis/previously-canonical-waiver process defined in `evidence-contracts.md`.

## 7. Assertion/replay registry protocol

AF-02 uses schema `commandf.af02-assertion-registry/v1`.

Top level is exactly:

```text
schema
entries[]
```

Each entry is exactly:

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

Allowed runner kinds are only:

```text
CARGO_TEST
AF02_REPLAY_BINARY
```

Shell command strings are prohibited; `argv[]` is the authority. Environment keys are explicit allowlist entries and sorted by name. Secret values are never registry content.

Bindings are bijective:

- every corpus scenario has exactly one registry entry;
- every registry entry refers to exactly one corpus scenario;
- `assertion_id`, `scenario_id`, and `(runner_kind,target,test_name)` identities are unique where applicable;
- every referenced manifest/source/config path exists at source SHA;
- every `surface_id` exists in the surface policy;
- `expected_outcome` is one of the surface's allowed normalized outcomes.

### 7.1 Test/replay inventory

For `CARGO_TEST`, AF-02 executes a frozen inventory command against the exact manifest/target and captures the machine-readable or deterministically normalized test list. `test_name` must appear exactly once before execution.

For `AF02_REPLAY_BINARY`, the binary path/digest and `argv[]` are tool/proof inputs; `--list` or an equivalent repository-owned inventory subcommand must prove the scenario/target exists before replay.

The inventory digest is bound to source SHA, Cargo.lock, relevant manifest/config digests, and assertion-registry digest.

### 7.2 Replay result record

Each executed assertion produces exactly:

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

The repository-owned result parser independently derives `normalized_outcome`. The producer cannot provide it as authority.

Orphan manifest entry, orphan registry entry, missing test/target, changed argv, parser mismatch, source/config digest drift, or observed-outcome mismatch fails qualification.

## 8. Canonical resource/offline enforcement protocol

Canonical AF-02 qualification uses Linux OCI/container enforcement. “Equivalent mechanism” is **not** accepted for canonical AF-02 proof lanes.

Frozen proof-critical base image:

```text
docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f
machine: x86_64
stable Rust identity: 1.97.1 proof baseline
```

Nightly fuzz tooling is acquired and verified separately, then mounted/provided to the network-denied execution phase; use of nightly does not change product compatibility authority.

Canonical deterministic harness invocation must enforce at least:

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

The runner additionally enforces policy per-input timeout, maximum input bytes, decompressed/generated byte counters, maximum temporary-file count in the AF-02 writable root, subprocess timeout, artifact size, and aggregate corpus bounds.

Subprocesses inherit the container network namespace and cgroup. Spawning a child outside the cgroup/container is prohibited. Docker socket, host network, privileged mode, device mounts, and arbitrary host-path writes are prohibited.

### 8.1 Enforcement evidence

Before tests, a repository-owned preflight records and verifies:

```text
container image repo digest
container runtime version
network mode == none
memory limit bytes == 805306368
NanoCPUs/effective CPU quota corresponding to 2 CPUs
PidsLimit == 256
root filesystem read-only == true
/tmp tmpfs size == 536870912 and noexec/nosuid
source mount read-only == true
output mount read-write == true and inside dedicated root
no privileged mode
no docker socket/device mount
```

Preflight also performs negative probes inside the same container:

- DNS/network connection attempt cannot reach the public network;
- source-tree write attempt fails;
- write to the dedicated output root succeeds;
- file outside allowed writable roots cannot be created.

Probe command, exit statuses, normalized result, and raw-output SHA-256 are retained. Missing/ambiguous runtime inspection or a skipped negative probe fails.

Per-input resource-limit outcomes are normalized using the existing AF-02 outcome classes. A harness killed by an unclassified host/runner failure is incomplete, never clean rejection.

## 9. Closed coverage source-accounting protocol

Coverage uses the frozen command family from `evidence-contracts.md`, but line accounting is now closed.

### 9.1 Authoritative source universe

For exact source SHA, derive tracked production source paths from the Git tree:

```text
all regular *.rs under crates/<crate>/src/** recursively
minus exact previously canonical non-product exclusions
```

Normalize to repository-relative slash paths, reject `..`, absolute paths, duplicate normalized paths, symlink ambiguity, or paths outside repository root. Sort by UTF-8 bytes and hash `{path,blob_sha}` array.

No product source may be omitted because it has zero hits. A tracked generated file inside the production roots counts unless a previously canonical exact exclusion exists.

### 9.2 Raw llvm-cov JSON shape

AF-02 line-floor authority uses the single merged JSON report emitted by the frozen command. The parser requires one logical merged data set and reads physical source-file `summary.lines.count` and `summary.lines.covered` integer fields after path normalization.

Rules:

- each normalized production source path appears exactly once;
- every authoritative production source-universe path appears in the report, including zero-covered files;
- any report path that normalizes into production roots but is absent from the Git source universe is failure;
- duplicate normalized report paths fail rather than sum twice;
- paths outside production roots are ignored only when they match an exact frozen exclusion such as target/generated output or isolated AF-02 fixtures;
- an unknown non-excluded path fails;
- `covered <= count`; negative/overflow/non-integer values fail;
- workspace totals are the integer sums over the authoritative production file set;
- critical-surface totals are sums only over the exact file/path scope frozen for that surface.

Macro/generated handling is physical-source based: if llvm-cov maps executed/expanded code to a tracked production `.rs` file, that file's reported line summary counts. Untracked compiler/build output is non-authoritative and may be ignored only under frozen exclusions. AF-02 does not invent synthetic source lines from macro expansion metadata.

Function/region data remains diagnostic and cannot alter the line denominator.

### 9.3 Floor derivation

For a pair `(covered,total)` with `total > 0`:

```text
floor_percent = (covered * 100) // total
```

All arithmetic is checked integer arithmetic. Each critical surface is calculated independently. Missing/zero-total critical scope fails rather than yielding 100% or 0% by convention.

Descriptor drift, source-universe drift, exclusions, and rebaseline rules remain base-controlled under section 4.

## 10. Closed nextest retry-pass fixture protocol

The deterministic policy fixture identity is frozen:

```text
fixture root: tests/assurance/af02-nextest-flake-fixture/
manifest: tests/assurance/af02-nextest-flake-fixture/Cargo.toml
integration test target: retry_pass_policy
single selected test name: af02_retry_pass_is_failure
workspace membership: prohibited
network: denied
```

Repository `.config/nextest.toml` must contain the canonical `[profile.ci]` retry/flaky settings plus a JUnit path for the CI profile. The AF-02 invocation is an argv vector, not shell text:

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

Before invocation the runner:

1. creates a dedicated AF-02 temporary directory owned by the current unprivileged UID;
2. removes any previous fixture state;
3. verifies the chosen state path does not exist and has no symlink component;
4. sets only the explicit `AF02_NEXTEST_STATE_FILE` environment variable in addition to the frozen environment allowlist.

Fixture behavior is exactly:

- validate the env path is absolute and beneath the assigned AF-02 temp root;
- attempt atomic `create_new(true)` of a regular state file containing fixed bytes `first-attempt\n`;
- if creation succeeds, intentionally fail the test;
- if the file already exists, verify it is a regular non-symlink file with exactly those bytes and pass;
- any other I/O state fails.

No clock, PID ordering, RNG, network, scheduler timing, sleep, external service, or previous run controls the result. Cleanup occurs after raw evidence is captured; cleanup failure fails the policy self-test.

### 10.1 Result parser

Nextest JUnit is the structured authority for attempt classification, while process exit is separate authority. The parser requires exactly one selected testcase corresponding to `af02_retry_pass_is_failure` and requires JUnit evidence that the test failed initially and later passed/flaked while final flaky policy is failure. The exact nextest JUnit schema observed from pinned 0.9.143 is frozen in Stack B0 test fixtures before enforcement and the parser itself becomes an acceptance-authority path under section 4.

Normalized required result:

```text
first_attempt_class = FAIL
retry_attempt_class = PASS
normalized_class = FLAKY_RETRY_PASS
process_exit_code != 0
selected_test_count = 1
```

A missing JUnit file, multiple selected tests, zero exit, malformed XML, result-parser ambiguity, or weaker override fails.

Raw evidence retains:

```text
command argv SHA-256
nextest executable SHA-256
nextest config SHA-256
fixture source/manifest SHA-256
state protocol SHA-256
JUnit SHA-256
stdout SHA-256
stderr SHA-256
process exit code
```

## 11. Proof-verifier reconstruction order

The independent verifier executes in this order and stops on first invalid prerequisite:

1. derive source SHA/tree and canonical base SHA/tree;
2. load the **canonical-base** contract/enforcement inventory for anti-forgery decisions;
3. classify candidate changes to policy/verifier/enforcer authority;
4. validate candidate contract files under base rules;
5. reconstruct AF-01, CF-06, and CF-10 authority projections from authoritative sources;
6. verify tool acquisition/executable/package identities;
7. derive source universe and boundary inventory;
8. verify effective resource/offline enforcement;
9. hash raw corpus fixtures and validate assertion registry/replay inventory;
10. parse raw property/nextest/coverage/mutation/cargo-test outputs;
11. verify required GitHub check-run uniqueness/provenance on exact candidate head;
12. construct the closed deterministic object defined in section 3;
13. canonicalize and compute `AF02_ADVERSARIAL_SHA256`;
14. compare against producer artifact and reject any mismatch or extra/missing deterministic field.

The proof builder and proof verifier are separate entry symbols/modules and appear separately in the enforcement inventory. Tests must demonstrate that a forged producer summary, forged normalized result, changed verifier code, changed base policy, unknown deterministic field, reordered set, omitted required field, or changed raw-output digest fails.

## 12. Round-2 finding disposition contract

This protocol closes the normative design issues returned by Qodo on PR #54 head `ce93767c7e4c3f569ed6c4575d2bbd4c7dda310b`:

- exact CF-10 three-delta/six-state cardinality and retained PR/base/run/artifact-name/digest identity;
- canonical AF-01/CF-06/CF-10 projection sources, schemas, serialization, and recomputation rules;
- closed deterministic proof object and stochastic metadata schema;
- anti-forgery coverage over verifier/scanner/parser/workflow enforcement code, not policy files alone;
- deterministic all-listed-within-frozen-scope mutation required-set algorithm;
- one reproducible AST-based surface-discovery algorithm with source-universe, aliases, macros, comments/strings, cfg/dead/generated handling, and scanner identity;
- closed assertion/replay registry, target identity, argv, parser, reverse/orphan, and source/config binding;
- canonical digest-pinned Linux OCI resource/offline enforcement protocol with runtime inspection and negative probes;
- complete Git-tracked production coverage source accounting and edge semantics;
- fixed nextest fixture path/test/state/invocation/result-evidence protocol.

The remaining planning-review observation that exact-head review/merge/post-merge evidence is not yet present is intentionally still open as the temporal T005/T006 gate described in section 1. This file does not convert that future gate into PASS.
