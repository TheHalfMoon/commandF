# AF-02 Closed Verification Protocol

Status: PLANNING_CANDIDATE

This file is normative and has higher AF-02 precedence than `evidence-contracts.md`, `spec.md`, `plan.md`, `tasks.md`, `consistency.md`, and donor/provenance prose. Machine-readable schemas and checked-in policy instances are co-authoritative for structure and fixed policy values. Any disagreement fails qualification.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

## 1. Planning and temporal closure

T005/T006 are temporal gates and cannot be embedded into the commit whose future CI/review/merge they prove.

Planning closes only when one unchanged exact head:
1. passes every path-applicable workflow;
2. has unique successful `assurance-proof`, `rust`, and `scorecard` check-runs on that exact head;
3. proves those checks against `required-check-policy.json`, including GitHub Actions app id 15368, repository, workflow id/path/base blob, job, run, attempt, head/base and conclusion;
4. receives fresh Qodo and CodeRabbit review when available, with zero unresolved substantive findings;
5. merges with an expected-head guard; and
6. survives post-merge `main`/tree plus both AF-01 live-ruleset read-backs.

Only step 6 closes T006 and authorizes Stack A0. Any head mutation makes earlier-head qualification stale.

## 2. Canonical JSON and deterministic hashing

All machine AF-02 JSON uses UTF-8 without BOM, no floats, compact separators, lowercase JSON literals, recursively UTF-8-byte-sorted object keys, schema-defined array order and no trailing newline. SHA-256 is lowercase hex over exact canonical bytes.

Unknown fields, missing fields, wrong type/range/pattern/cardinality/order, duplicate semantic keys, invalid path normalization, source disagreement or digest mismatch fail before hashing.

`schemas/af02-adversarial-proof-v1.schema.json` is the sole final proof schema. It is an envelope over the preserved pre-amendment structural proof schema copied byte-for-byte to:

```text
schemas/af02-adversarial-proof-core-v1.schema.json
```

The core retains the original 25 contract roles and deterministic/stochastic structure. The core already contains the `enforcement_inventory` contract role. The envelope therefore adds exactly 17 ordered extension roles and MUST NOT repeat that instance role:

```text
proof_core_schema
retained_authority_sources_schema
waiver_policy
waiver_policy_schema
required_check_policy
required_check_policy_schema
required_check_provenance_schema
semantic_contract
semantic_contract_schema
verifier_input_policy
verifier_input_policy_schema
surface_policy_schema
resource_policy_schema
corpus_manifest_schema
coverage_policy_schema
mutation_policy_schema
enforcement_inventory_schema
```

Thus final proof binds 42 distinct contract files across the core and extension sets. Paths and roles are unique across both sets. `extension_authority.enforcement_inventory_sha256` MUST equal the raw SHA-256 of the core contract file whose role is `enforcement_inventory`; `extension_authority.enforcement_inventory_schema_sha256` MUST equal the raw SHA-256 of the extension contract file whose role is `enforcement_inventory_schema`. The semantic verifier applies the same exact digest cross-binding to every extension-authority field and its corresponding contract role.

`core.af02_adversarial_sha256` retains the historical core deterministic digest. The authoritative final `af02_adversarial_sha256` hashes the independently reconstructed deterministic envelope consisting of:
- `core.deterministic`;
- `extension_contract_files`;
- `extension_authority`; and
- `required_check_provenance`.

`core.stochastic_observations` is excluded from the final deterministic digest. A producer-computed digest is compared only after independent reconstruction.

## 3. Preserved authority: AF-01, CF-06 and CF-10

### AF-01

Live authority is read from:

```text
GET /repos/TheHalfMoon/commandF/rulesets/21652953
GET /repos/TheHalfMoon/commandF/rulesets/21652974
```

The assurance ruleset must remain active on `refs/heads/main`, with no bypass actors, deletion/non-fast-forward protection, and exactly `assurance-proof`, `rust`, `scorecard`, each integration id 15368. The review-governance ruleset must remain active, merge-only, code-owner/review-thread/last-push protected, with only RepositoryRole actor 5 in `pull_request` bypass mode.

Candidate edits cannot establish live authority.

### CF-06

The verifier reconstructs production-oracle identity from canonical-base:

```text
crates/commandf-pkg/src/oracle_model.rs
donors/hl7-fhir-validator-6.10.2.yaml
.github/workflows/cf06-oracle.yml
```

Required identity remains HAPI FHIR `6.10.2`, source commit `d06577dbc5c62c74a2a8823fbc4830a3024d5b0b`, validator digest `a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6`, and `hl7.fhir.r4.core@4.0.1`.

### CF-10

`retained-authority-sources.json` must validate against `schemas/af02-retained-authority-sources-v1.schema.json` before any locator is used. The verifier reconstructs allowed GitHub URLs from fields instead of trusting supplied URLs, then verifies retained commit/blob/run/artifact identities.

Required retained truth remains PR 11, head `5fe10d9859407272acf6649fc3e868d3eb2fbd12`, base `5cb1a4c3445c0ebd86654cfb467a5e008e801c3e`, manifest blob `655949a8a30d67502dffd624a175d2e8e02b1d1f`, donor blob `566b46f4e6f467a1ccae3ac810b31956309173b6`, run `31916124080` conclusion `failure`, and artifact `9255732702` / `cf10-real-corpus-evidence` / digest `9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612`.

AF-02 never relabels the retained CF-10 failed production gate as PASS.

## 4. Closed machine authority

The obsolete `commandf.af02-authority-baseline/v1` is non-implementable. The closed baseline remains `commandf.af02-authority-baseline/v2`.

The following planning contracts are mandatory inputs:

```text
tool-policy.json
exclusion-policy.json
waiver-policy.json
required-check-policy.json
semantic-contract.json
verifier-input-policy.json
enforcement-inventory.json

schemas/af02-authority-baseline-v2.schema.json
schemas/af02-adversarial-proof-v1.schema.json
schemas/af02-adversarial-proof-core-v1.schema.json
schemas/af02-tool-policy-v1.schema.json
schemas/af02-tool-lock-v1.schema.json
schemas/af02-exclusion-policy-v1.schema.json
schemas/af02-evidence-inventories-v1.schema.json
schemas/af02-waiver-policy-v1.schema.json
schemas/af02-required-check-policy-v1.schema.json
schemas/af02-required-check-provenance-v1.schema.json
schemas/af02-retained-authority-sources-v1.schema.json
schemas/af02-semantic-contract-v1.schema.json
schemas/af02-verifier-input-policy-v1.schema.json
schemas/af02-surface-policy-v1.schema.json
schemas/af02-resource-policy-v1.schema.json
schemas/af02-corpus-v1.schema.json
schemas/af02-coverage-policy-v1.schema.json
schemas/af02-mutation-policy-v1.schema.json
schemas/af02-enforcement-inventory-v1.schema.json
```

Every schema uses fail-closed unknown-field behavior for its object contracts. Repository semantic validation remains mandatory where JSON Schema cannot express cross-object relations.

## 5. Waiver authority

`waiver-policy.json` validates against `schemas/af02-waiver-policy-v1.schema.json`. It starts with no waivers.

A mutation result with `WAIVED_EQUIVALENT_OR_OUT_OF_SCOPE` is green only when the base verifier resolves exactly one `AF02-W####` entry whose mutant id, source path, allowed result class, evidence digest and canonical ancestry match the mutation result and frozen mutation inventory.

`introduced_policy_sha` must already be an ancestor of the candidate canonical base. A waiver introduced by the same candidate cannot make that candidate green. Fabricated, duplicate, stale, cross-mutant or non-ancestor waivers fail.

## 6. Required-check provenance

`required-check-policy.json` and `schemas/af02-required-check-policy-v1.schema.json` freeze expected repository, GitHub Actions app, workflow ids/paths, canonical-base workflow blobs and job names.

Runtime evidence validates against `schemas/af02-required-check-provenance-v1.schema.json`.

For each context the verifier queries GitHub and proves exactly one matching check-run, repository `TheHalfMoon/commandF`, exact head/base, success, app id 15368/slug `github-actions`/owner `github`, expected workflow id/path/base blob, workflow run id/attempt, exact job name/id and `pull_request` event. App id alone is insufficient.

## 7. Semantic verifier and untrusted candidate parsing

`semantic-contract.json` validates against `schemas/af02-semantic-contract-v1.schema.json` and freezes verifier package/path/entrypoint plus named algorithm versions and negative-fixture ids.

The base verifier must implement every listed algorithm exactly once and own tests for each algorithm and negative fixture. Missing algorithm/test identity is non-green.

`verifier-input-policy.json` validates against `schemas/af02-verifier-input-policy-v1.schema.json`. Candidate files are untrusted data. Before semantic parsing the base gate enforces regular-file/no-symlink/containment and byte limits. JSON/YAML parsing enforces bounded depth, strings, properties/sequences/records and aggregate file/byte counts. YAML aliases, merge keys and custom tags are prohibited. Parser wall time and memory are bounded. Parent enforcement applies separate stdout/stderr byte ceilings; retained process evidence records observed bytes and per-stream overflow classification. Limit breach is explicit failure, never `NOT_APPLICABLE`.

## 8. Policy schemas before dependent execution

The following policy instance schemas are frozen during planning and MUST validate their instances before a dependent stack executes:

```text
commandf.af02-surface-policy/v1 -> schemas/af02-surface-policy-v1.schema.json
commandf.af02-resource-policy/v1 -> schemas/af02-resource-policy-v1.schema.json
commandf.af02-corpus/v1 -> schemas/af02-corpus-v1.schema.json
commandf.af02-coverage-policy/v1 -> schemas/af02-coverage-policy-v1.schema.json
commandf.af02-mutation-policy/v1 -> schemas/af02-mutation-policy-v1.schema.json
commandf.af02-enforcement-inventory/v1 -> schemas/af02-enforcement-inventory-v1.schema.json
```

The base verifier validates each instance before accepting its digest. Same-candidate weakening cannot self-green dependent evidence.

Surface and coverage consume the same Git-derived tracked Rust universe under `crates/**/src/**/*.rs` and `tools/**/src/**/*.rs` minus canonical-base source exclusions only.

Mutation selection is exactly every cargo-mutants-listed mutant inside frozen target paths minus exact pre-frozen exclusions. No top-N, percentage, operator preference or post-result selection exists.

Coverage descriptor/floors, mutation target/timeout/waiver bindings, corpus limits/provenance and resource units/ranges are schema-bound before observations. Enforcement role membership and activation-stack closure are schema/semantic-contract bound before a role becomes qualification authority.

## 9. Tool identity and execution isolation

`tool-policy.json` remains expected-set authority. Final tool lock contains exactly `arbitrary`, `cargo-fuzz`, `cargo-llvm-cov`, `cargo-mutants`, `cargo-nextest`, `libfuzzer-sys`, `proptest`, and `syn-af02-scanner`.

No null registry checksum may remain once that tool's activation stack begins. Executable acquisition records immutable source identity and installed executable digest.

Canonical deterministic execution uses the pinned Linux Rust OCI image from the resource schema, network none, read-only root/source, dedicated writable output, CPU/memory/PID/tmpfs limits and negative network/write probes. Missing enforcement evidence is incomplete, never PASS.

## 10. Base-controlled anti-forgery gate

A0 is the only bootstrap unit permitted to introduce the initial AF-02 verifier and base gate. A0 may use policy/schema/verifier infrastructure and tests only; no dependent fuzz/property/coverage/mutation outcome proves A0 itself.

After A0 canonicalization, the `pull_request_target` base gate runs canonical-base workflow/verifier/schema blobs with read-only permissions; separates base and candidate trees with credentials disabled; never sources/imports/builds/executes candidate code; parses candidate authority only under the input-limit policy; derives base/head from GitHub event/API data; classifies acceptance-authority paths itself; records base workflow/verifier/schema/enforcement-inventory blobs; cannot be disabled by candidate path filters; and fails if the base verifier cannot parse or classify the candidate.

An incompatible verifier/schema strengthening is a dedicated policy/verifier PR canonicalized before dependent work.

## 11. Deterministic evidence rules

The existing evidence inventory schema plus the semantic contract jointly enforce sorted/unique source universe and exact Git blob reconstruction; corpus/assertion/replay bijection; no PHI and bounded corpus sizes; independently normalized replay outcomes; nextest first-fail/retry-pass with forced flaky failure and non-zero process exit; exact Git-derived coverage membership and integer floor arithmetic; complete mutation inventory/result membership; waiver ancestry and mutant binding; canonical cargo-test counter equality; policy/inventory/digest cross-links; exact enforcement-role membership and activation closure; and path containment/no-follow rules.

Schema validation alone is never sufficient for cross-object semantics.

## 12. Stochastic observations

Fuzz campaigns are observational only. Allowed classes remain `NO_CRASH_OBSERVED_WITHIN_BOUND`, `DEFECT_DISCOVERED`, `INCOMPLETE_RESOURCE_OR_HARNESS_FAILURE`, and `CANCELLED_OR_SUPERSEDED`.

“No crash observed” is not correctness PASS. Any discovered defect must be minimized into deterministic replay evidence before the relevant implementation stack can close.

## 13. Planning-review closure target

This planning PR grants no implementation authority before T006.

It is mergeable only after one exact final head has green path-applicable CI, unique/provenant required checks, fresh reviewer truth, and zero unresolved substantive findings. Merge uses expected-head protection. Post-merge main/tree and both live AF-01 rulesets are re-read before Stack A0 begins.
