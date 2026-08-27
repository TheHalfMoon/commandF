# AF-02 Closed Verification Protocol

Status: PLANNING_CANDIDATE

This file is normative and has higher AF-02 precedence than `evidence-contracts.md`, `spec.md`, `plan.md`, `tasks.md`, `consistency.md`, and donor/provenance prose. Machine-readable schemas under `schemas/` are co-authoritative for structure, and the policy instances `tool-policy.json` and `exclusion-policy.json` are canonical AF-02 inputs. Any disagreement between this protocol, a machine schema, or a policy instance fails qualification; implementation may not choose the weaker interpretation.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

## 1. Temporal planning gate

T005/T006 exact-head review, merge, and post-merge evidence cannot be embedded into the commit whose future qualification they prove.

Planning closure therefore requires, in order:

1. one exact final planning head passes every path-applicable workflow;
2. every live required check context is unique, exact-head, successful, and bound to the expected GitHub App;
3. fresh Qodo and CodeRabbit review that exact head when available;
4. every substantive finding is fixed or explicitly dispositioned without inventing PASS;
5. zero substantive review threads remain unresolved;
6. merge uses an expected-head guard;
7. canonical post-merge `main`/tree and both AF-01 live rulesets are re-read.

Only step 7 may close T006 and authorize Stack A0. Any head mutation makes earlier-head CI/review evidence stale.

## 2. Canonical JSON

Deterministic semantic projections and proof objects use UTF-8, no BOM, no floats, recursively UTF-8-byte-sorted object keys, schema-defined array ordering, compact separators, lowercase JSON literals, and no trailing newline. SHA-256 is lowercase hex over exact canonical bytes.

Unknown fields, missing fields, duplicate set members, wrong cardinality/order, invalid type/range/pattern, or source disagreement fail before hashing.

## 3. AF-01 live authority

Authoritative live sources are exactly:

```text
GET /repos/TheHalfMoon/commandF/rulesets/21652953
GET /repos/TheHalfMoon/commandF/rulesets/21652974
```

Observation-only fields such as timestamps, URLs, node IDs, and `current_user_can_bypass` are excluded; no semantic rule field is silently ignored.

### 3.1 Assurance projection

Canonical object:

```json
{"bypass_actors":[],"deletion":true,"enforcement":"active","id":21652953,"name":"commandF main assurance","non_fast_forward":true,"ref_exclude":[],"ref_include":["refs/heads/main"],"required_status_checks":{"checks":[{"context":"assurance-proof","integration_id":15368},{"context":"rust","integration_id":15368},{"context":"scorecard","integration_id":15368}],"do_not_enforce_on_create":false,"strict_required_status_checks_policy":true},"source":"TheHalfMoon/commandF","source_type":"Repository","target":"branch"}
```

Expected SHA-256:

```text
6177b1b8777665506797d5e0cb3f48da81cc748e31bb2c9b53b4b1da777df00a
```

Exactly three rules are permitted: deletion, non-fast-forward, required-status-checks. No bypass actor is permitted. The check set is exactly `assurance-proof`, `rust`, `scorecard`, each integration id 15368.

### 3.2 Review-governance projection

Canonical object:

```json
{"bypass_actors":[{"actor_id":5,"actor_type":"RepositoryRole","bypass_mode":"pull_request"}],"enforcement":"active","id":21652974,"name":"commandF main review governance","pull_request":{"allowed_merge_methods":["merge"],"dismiss_stale_reviews_on_push":true,"require_code_owner_review":true,"require_extra_approval_for_unattributed_changes":true,"require_last_push_approval":true,"required_approving_review_count":1,"required_review_thread_resolution":true,"required_reviewers":[]},"ref_exclude":[],"ref_include":["refs/heads/main"],"source":"TheHalfMoon/commandF","source_type":"Repository","target":"branch"}
```

Expected SHA-256:

```text
9a54cd03bbf04449e1a00ff86701dfce98a700d392485f849774a63234347d0d
```

Exactly one pull-request rule and one PR-only RepositoryRole actor-id 5 bypass are permitted. Allowed merge methods are exactly `merge`.

Historical looser AF-01 semantic hashes in earlier AF-02 prose are superseded; live rules themselves are not changed by this projection correction.

## 4. CF-06 production-oracle authority

Authority is derived from canonical-base repository files, never candidate AF-02 prose:

```text
crates/commandf-pkg/src/oracle_model.rs
donors/hl7-fhir-validator-6.10.2.yaml
.github/workflows/cf06-oracle.yml
```

The verifier parses exact Rust constants, cross-checks donor repository/ref/tag/release digest, requires all workflow occurrences of `hl7.fhir.r4.core@<version>` to yield exactly `{4.0.1}`, records Git blob/raw SHA-256 for all three sources, and constructs exactly:

```json
{"project":"hapifhir/org.hl7.fhir.core","r4_core_context":"hl7.fhir.r4.core@4.0.1","release":"6.10.2","source_commit":"d06577dbc5c62c74a2a8823fbc4830a3024d5b0b","validator_cli_jar_sha256":"a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6"}
```

Expected SHA-256:

```text
236d71b6816978a4f7c9ea587d70301801b91a1d8a038f93ac7940203dc62787
```

Candidate changes to CF-06 authority files cannot self-authorize AF-02.

## 5. CF-10 retained authority

Current `main` is not required to contain retained CF-10 files. Exact source locators are machine-readable in `retained-authority-sources.json`.

Required identity:

```text
PR: 11
head: 5fe10d9859407272acf6649fc3e868d3eb2fbd12
base: 5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
manifest: corpus/real-ig/v1/corpus.json
manifest Git blob: 655949a8a30d67502dffd624a175d2e8e02b1d1f
donor: donors/cf-10-real-ig-delta-corpus.yaml
donor Git blob: 566b46f4e6f467a1ccae3ac810b31956309173b6
run: 31916124080
run conclusion: failure
artifact id: 9255732702
artifact name: cf10-real-corpus-evidence
artifact SHA-256: 9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612
```

The verifier fetches the retained blobs, verifies Git blob identities, computes raw SHA-256 itself, parses them only after identity validation, and proves exactly three ordered deltas and six ordered states:

```text
C001-after  hl7.fhir.us.core  9.0.0  d7b54d2ec2a48cea94ffea5d939ad67a681f80b94d69594a08cebac36da9e059  2749959
C001-before hl7.fhir.us.core  8.0.1  3c02eef48ef10617021bee95e58cbc66d596ceda8cada24b72000d33ad67c464  2713046
C002-after  hl7.fhir.uv.ips   2.0.1  7183242b70fb2a9058aa3701fb607517a3c2fd0e3100d1d8c538d744c2adf799  725312
C002-before hl7.fhir.uv.ips   1.1.0  403c4141101810e924f2928287985084819d8a5cc3a62e2b3840a557129840ef  1065103
C003-after  hl7.fhir.us.mcode 4.0.0  e603283bafa508a3705ad022bce95bba1fbd0b8b3b87b978e7412813b7bc1778  1003918
C003-before hl7.fhir.us.mcode 3.0.0  c94c91971747efeae760aa037d168e4df992cefb6dacece08217c464b9d39214  1014084
```

The retained workflow conclusion remains `failure`; AF-02 never relabels it as production-oracle success. Artifact expiry does not alter immutable recorded id/name/digest, but available GitHub metadata must agree.

## 6. Closed machine authority set

The obsolete illustrative `commandf.af02-authority-baseline/v1` MUST NOT be implemented.

The sole authority baseline is:

```text
commandf.af02-authority-baseline/v2
schemas/af02-authority-baseline-v2.schema.json
```

The following files are also canonical AF-02 machine authority:

```text
tool-policy.json
exclusion-policy.json
schemas/af02-tool-policy-v1.schema.json
schemas/af02-tool-lock-v1.schema.json
schemas/af02-exclusion-policy-v1.schema.json
schemas/af02-evidence-inventories-v1.schema.json
schemas/af02-adversarial-proof-v1.schema.json
```

`af02-evidence-inventories-v1.schema.json` contains closed, distinguishable schemas for:

```text
commandf.af02-source-universe/v1
commandf.af02-assertion-registry/v1
commandf.af02-replay-results/v1
commandf.af02-coverage-inventory/v1
commandf.af02-mutation-inventory/v1
commandf.af02-corpus-fixture-inventory/v1
commandf.af02-property-counterexample-inventory/v1
commandf.af02-enforcement-inventory/v1
```

No proof-critical inventory may invent a later ad-hoc shape. Any incompatible schema strengthening is an acceptance-authority change under the base-controlled policy-change process.

## 7. Tool policy and tool-lock completeness

`tool-policy.json` is the expected-set authority. It validates against `schemas/af02-tool-policy-v1.schema.json`.

The final AF-02 tool-lock member set is exactly, in UTF-8 id order:

```text
arbitrary             registry arbitrary 1.4.2, default features + derive, activates A1
cargo-fuzz            executable 0.13.2, rust-fuzz/cargo-fuzz@984c861c8dfea28055254c5f1d2659ab2cd63f76, activates A1
cargo-llvm-cov        executable 0.9.0, taiki-e/cargo-llvm-cov@be59056988acd54c7f984b7c85643daea3711b29, activates B0
cargo-mutants         executable 27.1.0, sourcefrog/cargo-mutants@8ab1dc786a1f61a4e370416cc6c68b81a704e917, activates C0
cargo-nextest         executable 0.9.143, nextest-rs/nextest@60fa45f638ffc3f35e74afa65737f45fcd32db2a, activates B0
libfuzzer-sys         registry 0.4.13 default features, activates A1
proptest              registry 1.11.0 default features, activates A1
syn-af02-scanner      registry syn 3.0.3 features=[full,visit], activates A0
```

Canonical-base `Cargo.lock` proves the current scanner checksum:

```text
syn 3.0.3 = 53e9bae58849f64dfa4f5d5ae372c8341f7305f82a3868709269343628b659a3
```

Registry packages whose checksum is null in the planning policy are **not yet active**. Their exact crates.io checksum must be frozen by a dedicated design-freeze policy change that is canonical before the activation stack executes dependent evidence. A null checksum is prohibited once its activation stack begins.

Executable acquisition mode is frozen to `LOCKED_GIT_REV_SOURCE_BUILD`. The actual lock records source-lock digest, executable path/SHA-256, version-output SHA-256, compiler, cargo, target, and features. Upstream commit alone is insufficient.

`commandf.af02-tool-lock/v1` validates against `schemas/af02-tool-lock-v1.schema.json`. At each stack, the base verifier derives the mandatory active subset from canonical-base `tool-policy.json`, never candidate proof data. It rejects missing, unexpected, duplicate, substituted-id/version/repository/commit/acquisition-mode/feature/checksum/target/executable-digest entries. Final C1 proof requires all eight entries and no others.

Negative tests include empty lock, omitted active member, unexpected member, substituted id, wrong version/upstream commit/features/checksum, and valid-looking wrong executable digest.

## 8. Exclusion authority

`exclusion-policy.json` is the sole source/mutation exclusion inventory and validates against `schemas/af02-exclusion-policy-v1.schema.json`.

The planning policy starts with:

```text
production_source_exclusions = []
mutation_exclusions = []
```

The same `production_source_exclusions` array governs both surface discovery and coverage. A source may not be excluded from one while remaining in the other.

A future exclusion requires a dedicated policy change that is canonical **before** dependent discovery/measurement/listing/execution. A candidate may not add an exclusion after seeing its own surface, coverage, or mutation result, and a same-candidate exclusion cannot make dependent evidence green.

Each future source exclusion requires exact id/path/reason/owner/introducing-policy-SHA/review reference/removal condition. Each mutation exclusion requires exact id/matcher digest/reason/owner/introducing-policy-SHA/review reference/removal condition. Unlisted or multiply matched exclusions fail.

The verifier independently hashes the canonical-base exclusion policy and requires the same digest in surface, coverage, and mutation proof sections.

## 9. Proof schema and contract-file closure

The sole proof schema is `schemas/af02-adversarial-proof-v1.schema.json`.

`contract_files[]` is sorted by path; both path and role are unique. It contains exactly one each of these 25 roles:

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
tool_policy
tool_policy_schema
tool_lock_schema
exclusion_policy
exclusion_policy_schema
evidence_inventory_schema
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

Each contract entry's Git blob and raw SHA-256 are recomputed from the exact source head. The proof authority object binds SHA-256 for tool/exclusion/schema policy inputs as well as AF-01/CF-06/CF-10 projections.

The proof schema requires final `tool_lock` cardinality exactly eight and exact member identities. It also requires exact current required-check ordering/membership (`assurance-proof`, `rust`, `scorecard`), resource constants, zero non-green terminal counts, and exclusion-policy digests in surface/coverage/mutation.

### 9.1 Semantic relations

The base verifier additionally enforces checked-integer relations:

```text
properties.case_count = properties.passed_count + properties.failed_count
canonical_cargo_test.test_count = passed_count + failed_count + ignored_count
mutation.required_count = killed_count + survived_count + timeout_count + unviable_or_build_failure_count + waived_count
coverage.workspace_covered_lines <= coverage.workspace_total_lines
coverage.workspace_floor_percent = (workspace_covered_lines * 100) // workspace_total_lines
```

Zero coverage denominator, overflow, mismatch, duplicate/order violation, wrong digest relationship, or path containment failure is non-green.

If a future canonical live ruleset adds an AF-02 base-verifier required context, that live authority change requires a proof-schema/policy update before a later proof can qualify; candidate proof data cannot silently add or remove contexts.

## 10. Closed evidence inventories

Every digest-only proof-critical object is parsed through `schemas/af02-evidence-inventories-v1.schema.json` before its digest is accepted.

### Source universe

`commandf.af02-source-universe/v1` binds source SHA/tree, exclusion-policy digest, exact roots, and `{path,blob_sha}` records. Paths are sorted/unique by semantic validator.

### Assertion registry

`commandf.af02-assertion-registry/v1` closes scalar/nullability/runner/argv/environment/source/config fields. `CARGO_TEST` requires non-null target/test; `AF02_REPLAY_BINARY` requires both null. Shell command strings are not authority. Assertion/scenario IDs are unique and bijective with the corpus.

### Replay results

`commandf.af02-replay-results/v1` binds each assertion/scenario to runner kind, process exit, independently normalized outcome, stdout/stderr and structured-result digests. Producer-supplied normalized outcomes are ignored as authority.

### Coverage inventory

`commandf.af02-coverage-inventory/v1` binds source/exclusion/descriptor identities, every file's covered/total integers, and each critical surface's exact source paths and independently derived floor. Missing/unknown/duplicate paths and covered>total fail semantically.

### Mutation inventory

`commandf.af02-mutation-inventory/v1` binds source/tool/policy/exclusion identities, frozen target paths, every mutant record/disposition, and every result. REQUIRED implies null exclusion id; EXCLUDED requires an exact `MUT-X####` entry present in canonical-base exclusion policy. Result inventory and mutant inventory must have exact required membership.

### Corpus fixture inventory

`commandf.af02-corpus-fixture-inventory/v1` binds scenario/path/SHA-256/byte length/provenance/assertion id; each default fixture <=256 KiB, aggregate <=8 MiB.

### Property counterexamples

`commandf.af02-property-counterexample-inventory/v1` binds property id, counterexample id, raw/minimized digests and status. `PROMOTED_REGRESSION` requires a real scenario id; `OPEN_DEFECT` remains non-green for stack closure.

### Enforcement inventory

`commandf.af02-enforcement-inventory/v1` binds every acceptance-authority role to base blob, entry symbol/job, format, and owned tests.

Semantic validation adds array sort/uniqueness, bijections, cross-file existence, digest equality, path containment, counter equations, exact target/test inventory membership, and policy membership. Schema validation alone is never treated as sufficient.

## 11. Base-controlled anti-forgery gate

AF-02 enforcement inventory covers:

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

### 11.1 Bootstrap

Stack A0 is the only bootstrap unit allowed to introduce the first executable AF-02 verifier/base gate. A0 may contain policy/schema/verifier/gate infrastructure and tests only; no dependent fuzz/property/coverage/mutation outcome may be used to prove A0 itself.

### 11.2 Post-A0 canonical base gate

After A0 merges, a canonical-default-branch `pull_request_target` workflow with read-only permissions executes only canonical-base workflow/script/verifier/schema blobs. Base and candidate trees are separate; persisted credentials are disabled. Candidate files are data only and MUST NOT be sourced, imported, executed, built, hooked, or evaluated as code.

The base workflow derives base/head from GitHub event/API data, runs for every PR, classifies changed paths itself, selects its own base verifier command, and records base/head SHA/tree plus base workflow/verifier/schema/enforcement-inventory blob identities, fixed argv digest, candidate evidence digests, and result digest.

No candidate path filter or workflow edit can disable the base gate. No authority change yields explicit `NOT_APPLICABLE_NO_AUTHORITY_CHANGE`; missing/incompatible base verifier, candidate execution attempt, unknown authority path, parse ambiguity, or base/candidate identity mismatch fails.

Incompatible verifier/schema strengthening must be a dedicated policy/verifier PR with no dependent harness/product change, canonicalized before later dependent work.

## 12. Deterministic surface discovery

Source universe is Git-derived tracked UTF-8 Rust under:

```text
crates/**/src/**/*.rs
tools/**/src/**/*.rs
```

minus only entries in canonical-base `exclusion-policy.json.production_source_exclusions`.

Canonical parser is `syn=3.0.3`, features `[full,visit]`, exact checksum `53e9bae58849f64dfa4f5d5ae372c8341f7305f82a3868709269343628b659a3`.

Scanner parses every source file, ignores comments/string literals as executable findings, visits tracked cfg/dead syntax, resolves module aliases/imports, treats relevant glob imports as candidates, conservatively inspects macros, and converts syntactically uncertain boundary matches into candidates rather than omissions.

Every finding records path/span/category/matcher/enclosing symbol and has exactly one disposition: critical surface or exact source exclusion. Stale, multiply classified, unclassified, or policy-unlisted exclusion fails.

Surface and coverage consume the same source-universe object/digest and the same exclusion-policy digest.

## 13. Mutation required-set derivation

Before listing, C0 freezes exact target paths, cargo-mutants tool/config/argv/test/timeout identity, and canonical-base exclusion policy.

Every listed mutant in target scope is REQUIRED unless it matches exactly one canonical-base `mutation_exclusions` entry. No top-N, percentage, operator preference, security-interest subset, or post-result manual selection exists.

Stable mutant id hashes source path/blob/span/enclosing function/null, mutation description, mutant diff SHA-256, cargo-mutants tool-lock entry digest, and mutation-policy digest.

Every inventory record has exactly one disposition. Required TIMEOUT or UNVIABLE/BUILD_FAILURE is retried and diagnosed; at qualification every required mutant is KILLED or covered by a waiver already canonical before the implementation candidate. Survivor/timeout/unviable/unclassified counts are zero.

## 14. Resource/offline enforcement

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
source: read-only mount
output: dedicated read-write /af02-output mount
```

No Docker socket, host network, privileged mode, device mount, or arbitrary host writable mount is permitted.

Preflight verifies runtime/image identity, network, memory=805306368, effective two CPU quota, PidsLimit=256, read-only root, tmpfs size/flags, source RO/output RW, then proves negative network/source-write/outside-root-write probes and positive output-write probe. Missing/ambiguous evidence fails.

Per-input timeout/input/decompressed/generated/temp-file/subprocess/artifact/corpus limits come from checked-in resource policy. Unclassified host/runner termination is incomplete, never clean rejection.

## 15. Coverage accounting

Coverage uses the same source universe and exact canonical-base `production_source_exclusions` as surface discovery. Every production path appears exactly once in the raw merged llvm-cov data, including zero-hit files. Unknown, missing, duplicate-normalized, non-integer, or covered>total records fail.

Workspace totals are integer sums; critical surfaces are independently summed from exact frozen source scopes. Macro attribution is to physical tracked production source. Floor is `(covered*100)//total`; zero-total critical scope fails.

Descriptor/floor/scope/exclusion/command/test-selection change is acceptance-authority change and cannot self-green.

## 16. Nextest retry-pass provenance

Frozen fixture:

```text
root: tests/assurance/af02-nextest-flake-fixture/
manifest: tests/assurance/af02-nextest-flake-fixture/Cargo.toml
target: retry_pass_policy
test: af02_retry_pass_is_failure
workspace membership: prohibited
network: denied
```

Invocation is exactly:

```text
cargo nextest run --manifest-path tests/assurance/af02-nextest-flake-fixture/Cargo.toml --profile ci --retries 2 --flaky-result fail -E test(af02_retry_pass_is_failure)
```

The base-controlled runner creates an empty 0700 output mount, proves JUnit target/state file absent with no symlink components, and sets only the frozen environment allowlist plus `AF02_NEXTEST_STATE_FILE`.

First attempt atomically `create_new(true)` writes fixed state bytes then fails; retry verifies the same regular non-symlink file/bytes then passes. No clock/RNG/sleep/PID/scheduler/network/previous-run state controls behavior.

After the waited-for process returns, the same runner opens the configured JUnit path with no-follow semantics, verifies regular file/owner/link-count/containment/fresh creation evidence where trustworthy, and binds JUnit/stdout/stderr/process exit into one envelope. Alternate/pre-existing wrapper output is rejected.

Exactly one testcase must show first failure then retry pass via pinned nextest retry-history representation, while process exit is non-zero due to forced flaky failure:

```text
first_attempt_class=FAIL
retry_attempt_class=PASS
normalized_class=FLAKY_RETRY_PASS
selected_test_count=1
process_exit_code != 0
```

## 17. Proof reconstruction order

The independent verifier executes:

1. derive candidate and canonical-base SHA/tree;
2. load canonical-base enforcement inventory, tool policy, exclusion policy, and all schemas;
3. classify candidate acceptance-authority changes before trusting candidate policy;
4. validate candidate contract/schema files under base rules;
5. reconstruct AF-01, CF-06, and CF-10 authority;
6. derive expected active tool set from canonical-base tool policy and verify tool-lock membership/provenance;
7. derive source universe using canonical-base exclusion policy and run boundary discovery;
8. verify resource/offline runtime evidence;
9. parse corpus/assertion/fixture/replay objects using closed inventory schemas and verify bijection/outcomes;
10. parse property counterexample inventory;
11. parse nextest raw evidence;
12. parse coverage inventory/raw report using same source/exclusion authority;
13. parse mutation inventory/results using canonical-base mutation exclusions;
14. parse canonical cargo-test evidence;
15. verify exact-head required-check uniqueness/provenance;
16. validate proof JSON schema and all semantic invariants/cross-digest relationships;
17. independently construct deterministic object from raw evidence;
18. canonicalize and compute `AF02_ADVERSARIAL_SHA256`;
19. compare producer artifact only after reconstruction; mismatch or extra/missing deterministic field fails.

Stochastic observations are structurally validated but excluded from deterministic digest.

Negative tests cover at least empty/omitted/substituted tool lock members, wrong tool digest/checksum/features, unlisted/same-candidate exclusion, malformed inventory/assertion types/nullability/order/uniqueness, orphan assertion/scenario/result, malformed identity/digest/range/enum/cardinality, mixed conditional shape, counter mismatch, path escape/symlink ambiguity, changed raw digest, forged normalized result/producer summary, candidate verifier substituted for base, base-ref swap, skipped base gate, and stochastic field affecting deterministic digest.

## 18. Planning-review closure target

This planning PR does not claim implementation PASS. It is ready to merge only when a fresh exact-head review finds no remaining substantive hidden choice or false-PASS path, every path-applicable workflow is green, required contexts are unique/provenant, zero substantive threads remain unresolved, and the guarded merge/post-merge live read-back sequence in section 1 completes.
