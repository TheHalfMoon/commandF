# AF-02 Closed Verification Protocol

Status: PLANNING_CANDIDATE

This file is normative and has higher AF-02 precedence than `evidence-contracts.md`, `spec.md`, `plan.md`, `tasks.md`, `consistency.md`, and donor/provenance prose. Structural schemas under `schemas/` are co-authoritative for machine structure. A disagreement between this protocol and a machine schema fails qualification until reconciled; implementation may not choose the weaker interpretation.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

## 1. Temporal planning gate

T005/T006 exact-head review, merge, and post-merge evidence cannot be embedded into the commit whose future qualification they prove.

Therefore planning closure is a temporal GitHub gate:

1. one exact final planning head passes every path-applicable workflow;
2. required check contexts are unique, exact-head, successful, and GitHub-Actions-app bound;
3. fresh Qodo and CodeRabbit review that exact head when available;
4. every substantive finding is fixed or explicitly dispositioned without inventing PASS;
5. zero substantive review threads remain unresolved;
6. merge uses an expected-head guard;
7. post-merge canonical `main`/tree and both AF-01 live rulesets are re-read.

Only step 7 may close T006 and authorize Stack A0. No prior-head evidence carries forward after a head mutation.

## 2. Canonical JSON

All deterministic semantic projections and proof objects use:

- UTF-8;
- no BOM;
- no floats;
- recursively lexicographically sorted object keys by UTF-8 byte sequence;
- schema-defined array order, never generic array sorting unless named below;
- JSON separators `,` and `:` without extra whitespace;
- lowercase JSON literals;
- no trailing newline in hashed canonical bytes.

SHA-256 is lowercase hexadecimal over the exact canonical bytes.

Unknown semantic fields, missing required fields, duplicate set members, wrong cardinality, invalid type/range/pattern, or source disagreement fail before hashing.

## 3. AF-01 live authority projection

Authoritative sources:

```text
GET /repos/TheHalfMoon/commandF/rulesets/21652953
GET /repos/TheHalfMoon/commandF/rulesets/21652974
```

Observation-only API fields such as timestamps, URLs, node IDs and `current_user_can_bypass` are excluded. Every semantic ruleset field is either projected or explicitly rejected as unknown drift.

### 3.1 Assurance projection

Canonical object:

```json
{"bypass_actors":[],"deletion":true,"enforcement":"active","id":21652953,"name":"commandF main assurance","non_fast_forward":true,"ref_exclude":[],"ref_include":["refs/heads/main"],"required_status_checks":{"checks":[{"context":"assurance-proof","integration_id":15368},{"context":"rust","integration_id":15368},{"context":"scorecard","integration_id":15368}],"do_not_enforce_on_create":false,"strict_required_status_checks_policy":true},"source":"TheHalfMoon/commandF","source_type":"Repository","target":"branch"}
```

Expected SHA-256:

```text
6177b1b8777665506797d5e0cb3f48da81cc748e31bb2c9b53b4b1da777df00a
```

Validation requires exactly the rules `deletion`, `non_fast_forward`, and `required_status_checks`; no bypass actors; exactly the three checks above sorted by context.

### 3.2 Review-governance projection

Canonical object:

```json
{"bypass_actors":[{"actor_id":5,"actor_type":"RepositoryRole","bypass_mode":"pull_request"}],"enforcement":"active","id":21652974,"name":"commandF main review governance","pull_request":{"allowed_merge_methods":["merge"],"dismiss_stale_reviews_on_push":true,"require_code_owner_review":true,"require_extra_approval_for_unattributed_changes":true,"require_last_push_approval":true,"required_approving_review_count":1,"required_review_thread_resolution":true,"required_reviewers":[]},"ref_exclude":[],"ref_include":["refs/heads/main"],"source":"TheHalfMoon/commandF","source_type":"Repository","target":"branch"}
```

Expected SHA-256:

```text
9a54cd03bbf04449e1a00ff86701dfce98a700d392485f849774a63234347d0d
```

Validation requires exactly one pull-request rule and exactly one PR-only RepositoryRole actor-id 5 bypass. `allowed_merge_methods` is exactly `merge`.

The older AF-01 semantic hashes in historical AF-02 planning prose are superseded by these closed projections. Live rules are unchanged by that projection correction.

## 4. CF-06 production-oracle projection

Authority comes from canonical-base repository files, not candidate AF-02 text:

```text
crates/commandf-pkg/src/oracle_model.rs
donors/hl7-fhir-validator-6.10.2.yaml
.github/workflows/cf06-oracle.yml
```

Derivation:

1. parse exact Rust constants `HL7_ORACLE_PROJECT`, `HL7_ORACLE_RELEASE`, `HL7_ORACLE_SOURCE_COMMIT`, `HL7_VALIDATOR_JAR_SHA256`;
2. require donor record `hl7-fhir-core-validator` to agree on repository/ref/tag/release artifact digest;
3. inspect every workflow token matching `hl7.fhir.r4.core@<version>` and require the non-empty observed set to equal `{4.0.1}`;
4. retain raw SHA-256 and Git blob SHA for all three source files;
5. construct exactly:

```json
{"project":"hapifhir/org.hl7.fhir.core","r4_core_context":"hl7.fhir.r4.core@4.0.1","release":"6.10.2","source_commit":"d06577dbc5c62c74a2a8823fbc4830a3024d5b0b","validator_cli_jar_sha256":"a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6"}
```

Expected SHA-256:

```text
236d71b6816978a4f7c9ea587d70301801b91a1d8a038f93ac7940203dc62787
```

Candidate edits to these authority files are evaluated from canonical base first and cannot self-authorize AF-02.

## 5. CF-10 retained authority

Current `main` is not required to contain the CF-10 retained files. Exact retained locators are machine-readable in:

```text
retained-authority-sources.json
```

Required retained identity:

```text
PR: 11
head: 5fe10d9859407272acf6649fc3e868d3eb2fbd12
base: 5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
manifest path: corpus/real-ig/v1/corpus.json
manifest Git blob: 655949a8a30d67502dffd624a175d2e8e02b1d1f
donor path: donors/cf-10-real-ig-delta-corpus.yaml
donor Git blob: 566b46f4e6f467a1ccae3ac810b31956309173b6
run: 31916124080
run conclusion: failure
artifact id: 9255732702
artifact name: cf10-real-corpus-evidence
artifact SHA-256: 9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612
```

The projector fetches both retained blobs from the retained head/blob APIs, verifies Git blob identity, computes raw SHA-256 itself, and only then parses them. Missing/expired downloadable artifact bytes do not erase GitHub-recorded artifact identity; however artifact metadata read-back must still agree with retained id/name/digest when available from GitHub API.

The manifest must contain exactly three ordered case IDs `C001`, `C002`, `C003`, each with `before` and `after`, producing exactly these six ordered states:

```text
C001-after  hl7.fhir.us.core  9.0.0  d7b54d2ec2a48cea94ffea5d939ad67a681f80b94d69594a08cebac36da9e059  2749959
C001-before hl7.fhir.us.core  8.0.1  3c02eef48ef10617021bee95e58cbc66d596ceda8cada24b72000d33ad67c464  2713046
C002-after  hl7.fhir.uv.ips   2.0.1  7183242b70fb2a9058aa3701fb607517a3c2fd0e3100d1d8c538d744c2adf799  725312
C002-before hl7.fhir.uv.ips   1.1.0  403c4141101810e924f2928287985084819d8a5cc3a62e2b3840a557129840ef  1065103
C003-after  hl7.fhir.us.mcode 4.0.0  e603283bafa508a3705ad022bce95bba1fbd0b8b3b87b978e7412813b7bc1778  1003918
C003-before hl7.fhir.us.mcode 3.0.0  c94c91971747efeae760aa037d168e4df992cefb6dacece08217c464b9d39214  1014084
```

The donor must independently agree on package/version/digest/byte-size provenance and six-state selection. The retained workflow conclusion stays `failure`; AF-02 never converts it to a production-oracle PASS.

## 6. Authority baseline v2

The obsolete `commandf.af02-authority-baseline/v1` planning example MUST NOT be implemented.

The only accepted baseline schema is:

```text
commandf.af02-authority-baseline/v2
schemas/af02-authority-baseline-v2.schema.json
```

The schema requires exact 3-delta/6-state cardinality and the full retained identity. Repository semantic validation additionally requires sorted uniqueness, source-file digest agreement, projection recomputation, and candidate/base anti-self-authorization.

## 7. Proof schema and semantic invariants

The only proof schema is:

```text
commandf.af02-adversarial-proof/v1
schemas/af02-adversarial-proof-v1.schema.json
```

The schema file itself and its SHA-256 are mandatory `contract_files[]` evidence.

The proof verifier enforces these semantic invariants in addition to JSON Schema:

### 7.1 Contract files

`contract_files[]` is sorted by `path`; path and role are unique. Required roles are exactly one each for:

```text
spec
plan
tasks
consistency
evidence_contracts
verification_protocol
donor_manifest
retained_authority_sources
authority_baseline_schema
proof_schema
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

Every entry's Git blob and raw SHA-256 are recomputed from source SHA.

### 7.2 Tool lock

Entries are sorted by `id`, IDs unique, feature arrays sorted/unique. Executable and registry-package shapes cannot mix fields. Executable SHA/version output are recomputed from acquired tool. Registry checksum comes from exact locked Cargo metadata.

### 7.3 Counter relations

Required equalities:

```text
properties.case_count = properties.passed_count + properties.failed_count
canonical_cargo_test.test_count = passed_count + failed_count + ignored_count
mutation.required_count = killed_count + survived_count + timeout_count + unviable_or_build_failure_count + waived_count
coverage.workspace_covered_lines <= coverage.workspace_total_lines
coverage.workspace_floor_percent = (covered * 100) // total
```

All arithmetic is checked integer arithmetic. Overflow, zero coverage total, negative count, or mismatch fails.

### 7.4 Required checks

At planning and until live policy changes, contexts are exactly:

```text
assurance-proof
rust
scorecard
```

Each must appear exactly once, have candidate `head_sha`, `integration_id=15368`, and `conclusion=success`. A foreign-app same-name check fails uniqueness/provenance.

If a later canonical AF-02 base-verifier context is added to the live assurance ruleset, exact live read-back determines the additional required context; the candidate cannot add/remove it through proof data.

### 7.5 Inventory objects referenced by digests

`file_metrics_sha256`, `critical_surface_metrics_sha256`, mutation inventories, source universe, corpus fixture inventory, property counterexample inventory, and replay result digests refer to separate closed repository-owned JSON schemas frozen no later than their design stack. Unknown fields and malformed identities fail. Their schema/digests join `contract_files[]` through the relevant policy role before they can affect qualification.

### 7.6 Path rules

Repository paths use slash-separated, relative UTF-8 paths; no empty component, `.`/`..`, absolute prefix, drive/UNC prefix, NUL, or symlink escape. Output-mount paths must remain below the dedicated AF-02 output root. Canonicalization is containment verification, not a way to accept an originally non-portable path.

## 8. Enforcement inventory and base-controlled verifier

AF-02 maintains `commandf.af02-enforcement-inventory/v1` with exact repository paths/entry symbols for every acceptance-authority role:

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
AF02_SCHEMA
```

Each entry records role, path, canonical-base blob SHA, language/format, entry symbol/job, and owned tests.

### 8.1 Bootstrap exception

Stack A0 is the one bootstrap unit allowed to introduce the first executable AF-02 verifier/base gate because no earlier executable AF-02 verifier exists. A0 may contain only policy/schema/verifier/gate infrastructure and tests; it cannot carry dependent fuzz/property/coverage/mutation outcomes used for closure.

A0 itself is qualified by these canonical planning contracts, existing AF-01 gates, static/unit security tests, external review, guarded merge, then post-merge live read-back.

### 8.2 Canonical base gate after A0

After A0 merge, the base-controlled gate is a `pull_request_target` workflow that exists on canonical default branch. It executes only canonical-base workflow/script/verifier blobs with read-only permissions.

It obtains base and candidate trees into separate directories, with persisted GitHub credentials disabled. Candidate files are data only; the gate MUST NOT source, import, execute, build, run hooks from, or evaluate code from the candidate checkout.

The workflow itself—not candidate configuration—derives PR base/head SHAs from GitHub event/API data, validates changed paths, selects the canonical-base verifier command, and records:

```text
base SHA/tree
candidate SHA/tree
base workflow blob SHA
base verifier blob SHA
base schemas blob SHAs
base enforcement-inventory blob SHA
candidate evidence digests
fixed verifier argv digest
result digest
```

The base workflow runs for every PR and performs its own changed-path classification; candidate path filters cannot disable it. If no acceptance-authority path changed it emits an explicit terminal `NOT_APPLICABLE_NO_AUTHORITY_CHANGE` result. If an authority path changed, missing base verifier, parse incompatibility, candidate execution attempt, unknown authority path, or comparison ambiguity is failure.

A deliberate incompatible strengthening of verifier/schema authority must be a dedicated policy/verifier PR with no dependent harness/product change. Once canonical, later work may depend on it.

## 9. Deterministic surface discovery

Source universe is generated from Git tree:

```text
tracked regular UTF-8 *.rs under crates/**/src/**
tracked regular UTF-8 *.rs under tools/**/src/**
minus exact previously canonical reviewed non-product exclusions
```

Sort by repository path and hash records `{path,blob_sha}`.

Canonical parser:

```text
syn =3.0.3
features = [full, visit]
registry source = crates.io canonical index
exact checksum = derived and retained from canonical-base Cargo.lock before implementation
```

Scanner rules:

- parse every source file or fail;
- comments/string literals do not create executable findings;
- visit cfg-disabled/dead tracked syntax;
- build module-local import/alias tables;
- glob imports for boundary crates/modules become candidate findings;
- macro definitions/invocations are conservatively inspected for frozen boundary tokens;
- unresolved receiver types use frozen constructor/import plus method-name matcher pairs;
- uncertain match becomes a candidate finding rather than omission.

Every finding records file, byte span, category, normalized matcher/callee, enclosing symbol when available. Every finding gets exactly one critical-surface or reviewed-exclusion disposition. Zero/multiple disposition, stale source/span/symbol, or unclassified finding fails.

Surface and coverage use the same production Rust source universe.

## 10. Mutation required-set derivation

Before listing mutants, Stack C0 freezes:

```text
targeted_source_paths[]
exact reviewed mutation exclusions[]
exact cargo-mutants tool/config/argv/test/timeout identity
```

Then pinned cargo-mutants JSON inventory is normalized. Every listed mutant whose source path is in target scope is REQUIRED unless it matches exactly one pre-frozen exact exclusion.

There is no top-N, percentage, operator preference, security-priority subset, or post-result selection.

Stable mutant ID hashes canonical fields:

```text
source_path
source_blob_sha
start_line/start_column/end_line/end_column
enclosing_function_or_null
mutation_description
mutant_diff_sha256
cargo_mutants_tool_lock_entry_sha256
mutation_policy_sha256
```

Inventory sorted by mutant ID; duplicate ID or missing disposition fails.

Required TIMEOUT or UNVIABLE/BUILD_FAILURE is retried under the frozen policy and diagnosed. At qualification, every required mutant must be KILLED or have a waiver that was already canonical before the implementation candidate. Survivor/timeout/unviable/unclassified counts are zero.

## 11. Assertion/replay registry

Schema id:

```text
commandf.af02-assertion-registry/v1
```

Top level contains only `schema` and `entries`.

Each entry contains exactly:

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

Runner kinds only `CARGO_TEST` or `AF02_REPLAY_BINARY`. Shell strings are prohibited. Scenario/assertion relation is bijective. Every referenced surface/path/config/target/test exists at source SHA.

Before replay, the runner inventories the selected test/target independently. Each assertion result records process exit, independently normalized outcome, stdout/stderr digests and structured result digest if any. Producer-supplied normalized outcome is not authority.

## 12. Canonical resource/offline enforcement

Canonical deterministic proof uses:

```text
image: docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f
machine: x86_64
stable Rust baseline: 1.97.1
--network none
--cpus 2
--memory 768m
--pids-limit 256
--read-only
--tmpfs /tmp:rw,noexec,nosuid,size=512m
source checkout: read-only mount
output: dedicated read-write /af02-output mount
```

No Docker socket, host network, privileged mode, device mount, or arbitrary host writable mount.

Preflight verifies runtime/image identity, network mode, `memory=805306368`, effective two-CPU quota (`NanoCPUs=2000000000` or semantically equivalent cgroup quota recorded canonically), PidsLimit 256, read-only root, tmpfs size/flags, source RO, output RW.

Negative probes inside the same container must prove public network unavailable, source write denied, output write succeeds, and write outside allowed roots denied. Missing probe or ambiguous runtime inspection fails.

Per-input timeout/input/decompressed/generated/temp-file/subprocess/artifact/corpus bounds come from checked-in resource policy. A host/runner kill not attributable to a defined surface limit is incomplete, never a clean rejection.

## 13. Coverage source accounting

Coverage source universe equals surface source universe:

```text
tracked Rust under crates/**/src/** and tools/**/src/**
minus exact previously canonical exclusions
```

Normalize repository-relative slash paths and reject traversal, absolute/drive/UNC forms, symlink ambiguity and duplicate normalization.

The frozen llvm-cov JSON parser requires one merged dataset and uses physical production file line summaries. Every authoritative production file appears exactly once, including zero-covered files. An unknown production-root report path, missing source path, duplicate path, non-integer count, or `covered > total` fails.

Workspace totals are integer sums. Critical-surface totals are independently summed from their exact frozen source scopes. Macro expansion uses physical tracked source attribution; untracked build output is excluded only by frozen rule.

Floor:

```text
(covered * 100) // total
```

Zero-total critical scope fails. Descriptor/floor/scope/exclusion/command/test-selection changes are base-controlled authority changes and cannot self-green.

## 14. Nextest retry-pass provenance

Frozen fixture:

```text
root: tests/assurance/af02-nextest-flake-fixture/
manifest: tests/assurance/af02-nextest-flake-fixture/Cargo.toml
target: retry_pass_policy
test: af02_retry_pass_is_failure
workspace membership: prohibited
network: denied
```

Invocation argv:

```text
cargo nextest run
--manifest-path tests/assurance/af02-nextest-flake-fixture/Cargo.toml
--profile ci
--retries 2
--flaky-result fail
-E test(af02_retry_pass_is_failure)
```

The dedicated output mount is created empty by the base-controlled runner with mode 0700 and expected unprivileged UID/GID. Before invocation the runner proves:

- mount empty except runner-owned directories;
- configured JUnit target does not exist;
- no component is symlink;
- state file does not exist and has no symlink component.

Fixture uses only `AF02_NEXTEST_STATE_FILE`. First attempt atomically `create_new(true)` a regular fixed-byte file and intentionally fails. Retry verifies the same regular non-symlink file and bytes, then passes. No clock, RNG, sleep, PID ordering, network, scheduler timing or previous-run state controls the transition.

After the waited-for nextest process returns, the same runner opens the configured JUnit path directly using no-follow semantics, verifies regular file, expected owner, link count one, containment on dedicated mount, and creation/change timestamp not predating the preflight marker where the platform exposes trustworthy monotonic metadata. It hashes JUnit/stdout/stderr and process envelope together. A wrapper-supplied alternate path is rejected.

Parser requires exactly one selected testcase and valid nextest retry-history representation (`flakyFailure`/`flakyError` according to the pinned 0.9.143 fixture schema), first failure then retry pass, while process exit remains non-zero because `--flaky-result fail` is forced.

Required normalized values:

```text
first_attempt_class=FAIL
retry_attempt_class=PASS
normalized_class=FLAKY_RETRY_PASS
selected_test_count=1
process_exit_code != 0
```

Missing/malformed/pre-existing JUnit, zero exit, wrong test count/history, state mismatch, path/owner/link failure, or process-binding mismatch fails.

## 15. Proof reconstruction order

Independent verifier executes:

1. derive candidate source SHA/tree and canonical base SHA/tree from GitHub/Git;
2. load canonical-base enforcement inventory/schemas for anti-forgery classification;
3. classify candidate authority changes before trusting candidate policy;
4. validate candidate contract/schema files under base rules;
5. reconstruct AF-01 live projections;
6. reconstruct CF-06 from canonical-base source files;
7. fetch and reconstruct CF-10 from retained source locators;
8. verify exact tool acquisition/package identities;
9. derive source universe and boundary inventory;
10. verify resource/offline runtime evidence;
11. validate corpus/assertion inventory and raw replays;
12. parse raw property/nextest/coverage/mutation/cargo-test evidence;
13. verify exact-head required-check uniqueness/provenance;
14. validate the proof JSON Schema and semantic invariants;
15. construct deterministic object itself from raw evidence;
16. canonicalize deterministic object and compute `AF02_ADVERSARIAL_SHA256`;
17. compare producer artifact only after reconstruction; any mismatch or extra/missing deterministic field fails.

Stochastic observations are validated structurally but excluded from deterministic digest.

Negative tests must cover at least unknown field, malformed type, malformed identity/digest, invalid range, wrong enum, wrong array cardinality/order/uniqueness, mixed conditional tool shape, counter mismatch, path escape, changed raw digest, forged normalized result, forged producer summary, candidate verifier substituted for base verifier, changed base ref, skipped base gate, and stochastic field affecting deterministic digest.

## 16. Planning-review closure target

This planning PR does not claim implementation PASS. It is ready to merge only if a fresh exact-head review confirms no remaining normative hidden choice or false-PASS path, all path-applicable workflows are green, required contexts are unique/provenant, and the guarded merge/post-merge read-back sequence in section 1 completes.
