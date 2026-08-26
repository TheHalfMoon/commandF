# AF-01 Plan — Trusted Development Baseline

Status: PLANNING_CANDIDATE

## Entry condition

AF-01 planning begins from canonical `main` after CF-13 closeout merged:

```text
main: 8a45857bf31c4acae57fdfb1e3cdde3d0f7d0361
tree: ffaa14fdc7a738a771ac872e566ad1609eedf2cc
CF-13: CLOSED_CANONICAL
```

AF-01 does not depend on the externally blocked CF-10/CF-06 production-oracle path.

Implementation starts only after this planning package and the assurance-program/index/architecture reconciliation are reviewed, exact-head green, and merged to canonical `main`.

## Design summary

AF-01 introduces no product semantic engine. It adds an independently executable development-assurance layer around the repository that makes workflow immutability, dependency policy, security analysis, and canonical-branch enforcement explicit.

The design deliberately has two classes of evidence:

1. **checked-in deterministic policy evidence** — repository scripts/configuration/workflows that can be reviewed and repeated from the source tree;
2. **live source-control evidence** — GitHub ruleset/branch state that must be queried from the hosting platform and cannot be inferred from files alone.

AF-01 cannot be `CLOSED_CANONICAL` unless both classes are satisfied.

## Baseline findings to remediate

### General CI

Current `.github/workflows/ci.yml` uses:

```text
runs-on: ubuntu-latest
actions/checkout@v4
dtolnay/rust-toolchain@1.97.1
```

These are less reproducible than later commandF proof workflows, which already use full action SHAs, fixed runner labels, credentialless checkout, and in some cases digest-pinned containers.

### Repository settings

Live GitHub state at planning creation:

```text
main protected: false
branch protection enabled: false
required status check enforcement: off
repository rulesets: none observed
```

### Missing assurance surfaces

No checked-in configuration/workflow currently exists for:

- cargo-deny;
- cargo-audit;
- zizmor;
- OpenSSF Scorecard;
- repository-owned verification that all workflow references stay immutable/credential-minimal.

## Implementation architecture

### Stack A — repository-owned workflow trust audit + baseline hardening

Add a small repository-owned audit script, preferably under `.github/scripts/`, with no new runtime dependency in the product crates.

Expected responsibilities:

1. enumerate tracked `.github/workflows/*.yml|*.yaml` and repository composite `action.yml` files;
2. parse or conservatively inspect all external `uses:` references;
3. permit local `./` references;
4. require external action refs to end in a full 40-hex commit SHA;
5. inspect checkout steps and require `persist-credentials: false` unless the policy file names a specific bounded exception;
6. flag proof-critical `*-latest` runner labels according to the AF-01 policy;
7. ensure newly added workflow files cannot escape the audit by using a complete tracked-file enumeration rather than a hard-coded subset;
8. emit deterministic machine-readable result plus a concise human-readable failure report.

Implementation language should minimize new dependencies. A small Python standard-library or shell+Python verifier is acceptable because it is repository CI tooling, not commandF trusted product runtime. The product core remains Rust-owned.

Workflow updates in the same stack must:

- replace mutable external action tags with verified full SHAs;
- set `persist-credentials: false` on read-only checkout;
- change proof-critical runner labels from `*-latest` to explicit supported labels such as `ubuntu-24.04` where the job semantics allow it;
- preserve all existing steps/assertions/path filters;
- add timeouts if a touched job lacks one and an appropriate bound can be established;
- keep `cargo --locked` semantics.

Do not bulk-reformat unrelated YAML.

### Stack B — Rust dependency and workflow security gates

#### cargo-deny

Add `deny.toml` with policy derived from the actual current graph.

Required checks:

```text
licenses
bans
advisories
sources
```

Policy rules:

- unknown registry/git sources denied by default;
- accepted licenses explicitly enumerated after inspecting current dependency metadata;
- duplicate versions are reported and reviewed rather than globally denied until the current graph is understood;
- wildcards disallowed for direct dependencies where supported by policy;
- advisory ignores require structured rationale and revisit condition;
- exceptions should be narrow package/version identities, not broad families.

Run `cargo deny check` from an AF-01 workflow with an exact pinned cargo-deny version or immutable execution image identity.

#### cargo-audit

Run a pinned `cargo audit` against exact `Cargo.lock`.

Keep cargo-audit separately visible even though cargo-deny can consume RustSec advisories because:

- RustSec audit is a recognizable independent vulnerability surface;
- results can be compared against cargo-deny advisory policy;
- disagreement itself becomes useful evidence.

No duplicate PASS is claimed if both tools depend on the same advisory database state; evidence records their relationship.

#### zizmor

Run a pinned zizmor version over repository workflows/actions.

AF-01 freezes a severity policy in the plan implementation commit. Initial target:

- fail on high/medium findings unless explicitly dispositioned;
- report lower severities as retained evidence until enough baseline data exists to justify stricter policy.

If the first real run shows a different severity calibration is appropriate, update the AF-01 plan/tasks before merging implementation rather than silently weakening the command.

### Stack C — posture evidence, proof artifact, and source-control enforcement

#### OpenSSF Scorecard

Add Scorecard in the least-authority supported mode for this public repository. Pin the action to a verified full commit SHA. If publishing results requires `id-token: write` or `security-events: write`, scope those permissions only to the Scorecard job.

Do not gate on a single aggregate score. Retain per-check evidence and specifically inspect at least:

- Branch-Protection;
- Dangerous-Workflow;
- Pinned-Dependencies;
- Token-Permissions;
- Security-Policy where applicable;
- Vulnerabilities.

#### AF-01 proof workflow

Add `.github/workflows/af01-assurance-proof.yml` with complete path coverage for:

- AF-01 spec package;
- assurance program/index/architecture documents;
- workflow files;
- action.yml and repository CI scripts;
- Cargo manifests/lockfile;
- `deny.toml` and any AF-01 policy/config files;
- AGENTS/constitution when they affect authority.

Proof should run in a pinned environment consistent with the best existing commandF proof workflows.

Expected retained artifact contents:

```text
assurance-summary.json
workflow-trust.json
cargo-deny.txt/json if supported
cargo-audit.json or machine-readable equivalent
zizmor.sarif/json
tool-identities.txt
source-identities.txt
```

The final summary has a stable schema and records a deterministic digest, for example:

```text
AF01_ASSURANCE_SHA256=<64 lowercase hex>
```

Live GitHub ruleset state is kept separately because hosting-platform metadata can change without changing the repository tree. The convergence record binds the exact live observation used for closure.

#### Main ruleset

Target source-control policy:

- applies to `refs/heads/main`;
- branch deletion prohibited;
- force-push prohibited;
- pull request required;
- required review count at least 1 unless governance specifies stronger;
- required conversations resolved;
- stale approvals dismissed or latest-push approval semantics configured so moved heads cannot inherit stale approval;
- required status checks include the AF-01-selected canonical checks;
- administrator/bypass actors minimized and documented;
- no broad bypass based solely on actor type.

Exact check names must be derived from the final implementation workflows after they are canonical; do not guess names before jobs exist.

The current connector exposes ruleset/branch-protection reads but not writes. Therefore this configuration is an explicit external operational task. AF-01 cannot mark it complete until a live read proves it.

## Security and trust boundary

AF-01 security tooling runs against repository source/dependency metadata only.

- no PHI;
- no patient instances;
- no model/provider credentials;
- no CF-06 production identity mutation;
- no code execution from untrusted PR-supplied arbitrary scripts beyond the repository's existing CI model;
- third-party actions/tools are pinned and least-authority;
- network access used for advisory databases/Scorecard is explicit and bounded.

Fork PR security must be considered before any workflow is granted write permissions or secrets. `pull_request_target` is not introduced by AF-01 unless a separate threat-model amendment proves it necessary.

## Determinism model

### Fully deterministic inputs

- repository workflow trust audit;
- exact `Cargo.lock` graph inspection when dependency/advisory external state is excluded;
- source/config hashes;
- proof summary construction over retained normalized inputs.

### Externally versioned inputs

- RustSec advisory database;
- OpenSSF Scorecard service/action behavior;
- live GitHub ruleset state.

These must record version/commit/update identity where available. Their results are reproducible only relative to that external input identity and must not be described as timeless.

## Test plan

### Workflow-trust audit positive

- all existing hardened workflows pass;
- local `uses: ./` accepted;
- full 40-hex refs accepted;
- credentialless checkout accepted;
- fixed proof runner accepted.

### Workflow-trust audit negative

- `actions/checkout@v5` rejected;
- shortened SHA rejected;
- branch ref rejected;
- tag ref rejected;
- checkout without explicit `persist-credentials: false` rejected;
- new unscanned workflow path causes coverage test failure;
- proof-critical `ubuntu-latest` rejected according to policy;
- malformed workflow input fails closed rather than being skipped.

### Dependency policy

- current graph passes only after every accepted license/source is explicitly represented;
- synthetic/config fixture for unknown git source rejected where practical;
- waiver schema rejects missing rationale/revisit metadata if commandF wraps waiver validation;
- direct wildcard dependency policy is checked;
- cargo-audit output is retained even when clean.

### zizmor

- all workflows collected;
- baseline findings either fixed or explicitly documented;
- high/medium new finding makes AF-01 gate fail according to final frozen policy.

### Proof

- repeated exact-tree repository-owned audit outputs equal;
- summary digest recomputes exactly;
- source SHA/tree recorded correctly;
- proof fails if one required evidence file is missing;
- dirty repository or source mismatch fails where the proof environment supports the check.

### Regression

All existing mandatory repository gates remain:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

and every path-applicable existing proof/oracle workflow must remain green on each implementation head.

## Migration impact

Developer-visible changes:

- CI will reject mutable Action references and new unreviewed workflow authority;
- dependency additions may require license/source/advisory policy updates;
- canonical main will require PR/check/review policy once the ruleset is applied;
- emergency/break-glass changes become explicit governance events rather than ordinary direct pushes.

No commandF CLI or report schema changes are planned.

## Performance impact

AF-01 adds CI work. Keep it bounded:

- workflow-trust audit should complete in seconds;
- cargo-deny/audit may use caching but cache identity must not make the result authoritative;
- Scorecard/zizmor should be separate jobs so they can be diagnosed independently;
- do not put long-running AF-02 fuzz/mutation work into AF-01.

## Stack ordering

```text
Planning package
  -> Stack A workflow trust audit + workflow baseline hardening
  -> Stack B dependency/security static gates
  -> Stack C assurance proof + Scorecard + live main ruleset evidence
  -> convergence
```

Stack B may branch from canonical Stack A. Stack C may branch only after Stack B is canonical unless the PR stack explicitly preserves exact dependencies and the repository governance supports that stack.

## Closure criteria

AF-01 is `CLOSED_CANONICAL` only when:

1. planning package canonical;
2. every implementation stack merged from an exact green/reviewed head;
3. all AF-01 functional requirements proven or explicitly deferred by an amended canonical plan with rationale;
4. live `main` ruleset/branch-policy query proves required enforcement;
5. final AF-01 proof artifact retained with exact identities;
6. zero unresolved substantive review findings;
7. convergence document merged without semantic substitution;
8. canonical post-merge main/tree recorded.

Implementation merge alone is insufficient for closure.
