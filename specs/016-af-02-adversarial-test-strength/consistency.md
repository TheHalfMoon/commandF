# AF-02 Consistency Analysis — Adversarial Test Strength

Status: PLANNING_CANDIDATE

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

## Authority consistency

AF-02 precedence is singular:

1. repository governance/constitution/`AGENTS.md`;
2. `verification-protocol.md`, checked-in machine policies and schemas;
3. non-superseded `evidence-contracts.md`;
4. `spec.md`;
5. `plan.md`;
6. `tasks.md`;
7. this analysis and donor/provenance records.

Authority-baseline v1 remains deprecated and non-implementable. Baseline v2 is the only authority-baseline schema.

The final proof path and schema id remain `schemas/af02-adversarial-proof-v1.schema.json` / `commandf.af02-adversarial-proof/v1`. During planning it was strengthened into an envelope over the byte-identical prior schema now retained as `af02-adversarial-proof-core-v1.schema.json`. The core preserves the previous 25 contract roles and already contains the `enforcement_inventory` instance role. The envelope therefore adds 17 required extension roles, including only the standalone enforcement-inventory schema, so the final deterministic proof binds 42 distinct contract files without discarding or duplicating earlier structural constraints. The extension authority separately cross-binds the enforcement-inventory instance digest to the core role and the schema digest to the extension role.

## Preserved external authority

AF-02 does not weaken the active AF-01 assurance/review rulesets. Required contexts remain `assurance-proof`, `rust`, `scorecard`, each GitHub Actions integration id 15368.

CF-06 remains HAPI FHIR `6.10.2`, source `d06577dbc5c62c74a2a8823fbc4830a3024d5b0b`, validator digest `a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6`, and R4 `hl7.fhir.r4.core@4.0.1`.

CF-10 retained run `31916124080` remains `failure`. Its manifest/donor/run/artifact locators are validated against a closed schema and reconstructed from GitHub identity fields. AF-02 never relabels that production gate as PASS.

## Reviewer finding closure

The latest planning round is addressed as follows:

- **Waiver authority:** `waiver-policy.json` + closed schema; zero initial waivers; canonical ancestry and mutant binding are named semantic-verifier algorithms.
- **Required-check provenance:** `required-check-policy.json` freezes repository, GitHub Actions app, workflow ids/paths/base blobs and job names; runtime provenance has a dedicated schema including run/job/check-suite/head/base identities.
- **Retained locator semantics:** exact repository/commit/blob/run/artifact URL relationships have a closed retained-authority schema; supplied URLs are reconstructed rather than trusted.
- **Prose-only semantic relations:** `semantic-contract.json` freezes algorithm ids, verifier package/path/entrypoint and required negative-fixture ids. Missing implementation/test mapping is non-green.
- **Candidate parser/resource boundary:** `verifier-input-policy.json` freezes preparse size/containment/symlink limits, JSON/YAML depth/record/string constraints, YAML safe-loader restrictions, aggregate wall-time/memory bounds, separate stdout/stderr byte ceilings, and observed-byte/overflow evidence.
- **Proof-critical policy schemas:** separate closed schemas now exist for surface, resource, corpus, coverage and mutation policy instances before dependent execution.
- **Enforcement-inventory closure:** `enforcement-inventory.json` and `schemas/af02-enforcement-inventory-v1.schema.json` freeze the 27-role activation inventory; the aggregate evidence schema requires exactly one runtime entry for each role.
- **Proof binding:** the proof envelope retains the original 25-role core, where `enforcement_inventory` already exists, and adds 17 exact extension contract roles including only `enforcement_inventory_schema`; final authority digests cross-bind the core instance and extension schema without duplicating a contract path.

## Anti-self-forgery consistency

A candidate cannot define both the acceptance rule and its own success. A0 bootstraps policy/schema/verifier infrastructure only. After A0 canonicalization, the base-controlled `pull_request_target` gate executes canonical-base workflow/verifier/schema blobs, never candidate code, and parses candidate data only under the input-limit policy.

A same-candidate waiver, source exclusion, mutation exclusion, coverage floor reduction, policy weakening, verifier weakening or locator substitution cannot make dependent evidence green.

## Source and coverage consistency

Surface discovery and coverage share the same Git-derived tracked Rust universe:

```text
crates/**/src/**/*.rs
tools/**/src/**/*.rs
```

minus canonical-base source exclusions only. Missing, unknown, duplicate-normalized or out-of-root paths fail instead of becoming implicit exclusions.

Coverage policy is frozen before percentages. Coverage remains diagnostic evidence, not semantic authority.

## Mutation consistency

Mutation target paths, tool identity, test command, timeout/retry/diagnosis, exclusion policy and waiver policy freeze before listing.

Required membership is deterministic:

```text
all listed mutants in target scope
minus exactly matched pre-frozen exclusions
```

There is no top-N, percentage, operator preference or post-result manual subset. A waiver resolves only through canonical waiver authority and cannot be introduced by the candidate it greens.

## Required-check consistency

Integration id 15368 alone is insufficient. Qualification binds each context to repository, GitHub Actions app identity, canonical-base workflow id/path/blob, workflow run/attempt, job name/id, exact head/base, pull_request event and success.

## Parser and resource consistency

The privileged/base-controlled verifier treats candidate files as hostile data. Size is checked before parsing. Symlinks/path escapes are rejected. JSON and YAML have explicit depth/item/string bounds. YAML aliases, merge keys and custom tags are prohibited. Aggregate file/record/byte, parser wall-time and memory limits are fixed. Parent-enforced stdout/stderr byte ceilings are fixed separately, and evidence retains observed bytes plus overflow classification for both streams. Parser exhaustion or output-limit breach is failure, not neutral topology.

## Tool and corpus consistency

Executable tools retain immutable source and installed-binary identity. Registry packages cannot activate with unresolved checksums.

Corpus entries remain synthetic/publicly redistributable non-PHI only, <=256 KiB by default and <=8 MiB aggregate. Corpus/assertion/replay membership is bijective and independently reconstructed.

## Nextest consistency

The fixed retry-pass fixture uses retries 2 and flaky-result fail. First failure then retry pass remains failed AF-02 evidence with non-zero process exit. JUnit/stdout/stderr/exit are bound to one waited-for process and dedicated clean output mount.

## Temporal boundary

T005/T006 evidence lives in GitHub because a commit cannot contain proof of its own future exact-head CI/review/merge. No planning merge occurs until the current head qualifies, and no implementation authority exists until post-merge main/tree plus live AF-01 rulesets are re-read.

## Current decision

```text
AF-02: PLANNING_CANDIDATE
T005: OPEN
T006: OPEN
IMPLEMENTATION AUTHORITY: NOT_GRANTED
NEXT AUTHORITY AFTER T006: STACK A0 ONLY
```
