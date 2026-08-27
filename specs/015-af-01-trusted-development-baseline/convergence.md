# AF-01 Convergence Record — Trusted Development Baseline

Status: `CONVERGENCE_CANDIDATE`

This document is the Phase 4 convergence record required by T050–T053. It records exact canonical identities and retained evidence without changing commandF product semantics. T054–T056 remain open until this convergence candidate is independently qualified, merged, and post-merge canonical evidence is complete.

## Phase 4 entry identity

```text
repository: TheHalfMoon/commandF
canonical main: a683dfaba7feb607145400eaa75d771e5df3c608
canonical tree: 623a5b20eba83c618d4da288677c1cd3d2826f61
Stack C PR: #45
Stack C final head: c82ef6e6f137805074cc5e0c453d47e0d2799839
Stack C merge commit: a683dfaba7feb607145400eaa75d771e5df3c608
T043: CLOSED_CANONICAL
```

At Phase 4 entry GitHub reports `main` as protected by active repository rulesets.

## Planning and stack identities

### Planning package — PR #34

```text
pre-AF-01 canonical base: 8a45857bf31c4acae57fdfb1e3cdde3d0f7d0361
pre-AF-01 tree: ffaa14fdc7a738a771ac872e566ad1609eedf2cc
planning final head: e2d6a26188e9c375aac8006d672e131c9859bbef
planning merge/main: eeecb0bc03c7040bb18b70bce8b69d618384f783
exact-head ci run: 32982942931 — success
exact-head cf06-oracle run: 32982942969 — success
```

The planning package was documentation-only and established the AF-01 spec/plan/tasks/consistency authority before implementation.

### Stack A — PR #43

```text
canonical input main: eeecb0bc03c7040bb18b70bce8b69d618384f783
final head: 28d9e39da1bc1bc6059a8b9f9c46327fe47ad99f
merge/main: 48587578e2d9167ac1c96b51c9942edb2aa74d8c
merge tree: 4f2ccef845321a7abba8ce5388e78281a1514436
exact-head ci run: 33043285246 — success
exact-head cf06-oracle run: 33043285220 — success
exact-head registry-download-smoke run: 33043285224 — success
```

Stack A established the repository-owned workflow-trust baseline, complete workflow/Action metadata discovery, immutable external Action references, least-authority permission policy, checkout credential policy, fixed proof runner policy, executable-surface checks, and deterministic counterexamples.

### Stack B — PR #44

```text
canonical input main: 48587578e2d9167ac1c96b51c9942edb2aa74d8c
final head: cc5607bff3e7c20a069e4f7d666005e74fe75b48
merge/main: 301aa5e66089859e938145870dc4a9300a25692a
merge tree: d8557e6992ea82c0d2bb36178cf85961243e0691
```

Exact-head required/path-applicable workflow evidence:

```text
af01-security: 33051903674 — success
ci: 33051903704 — success
cf06-oracle: 33051903643 — success
cf11-multi-version-proof: 33051903712 — success
cf11g-context-proof: 33051903645 — success
cf12-impact-proof: 33051903653 — success
cf13-quality-gate-proof: 33051903655 — success
```

Final Stack B security artifacts:

| Evidence | Artifact ID | GitHub digest |
|---|---:|---|
| `af01-t020-dependency-inventory` | `9637979875` | `sha256:483717f5921c3b4524546cd37a77eedc282f10ae1de406f95779a710f1b08b01` |
| `af01-cargo-deny-proof` | `9637995548` | `sha256:089e84016fe6ee6ff594aff24b9f359dd3dabd5174599d93ddae1fe987d762e4` |
| `af01-rustsec-audit` | `9638079695` | `sha256:8ea0770d681b557c97d5b33a3e08e3cc43b31733314123f15a2c783335f72f19` |
| `af01-zizmor-proof` | `9637989126` | `sha256:042b829a714f602cc9675f821979666c839454217430b51e389b9b869812d864` |

Stack B exact tool/security identities retained by the repository include:

```text
cargo-deny action commit: 3c6349835b2b7b196a839186cb8b78e02f7b5f25
cargo-deny version: 0.20.2
cargo-audit version: 0.22.2
RustSec advisory database origin: https://github.com/RustSec/advisory-db.git
observed Stack B advisory database commit: a7bfe16948bf6f3ee25bdee4822209f87da21b80
zizmor action commit: 3dc1ecc9bcb9e94e9b2c709687979e1298497054
zizmor version: 1.29.0
zizmor threshold: min-severity=medium
online zizmor audits: disabled
security waivers: zero
```

Qodo material findings about resolved dependency-edge identity and malformed dependency metadata were fixed before the final head. CodeRabbit's material wildcard-path policy finding was also fixed by disabling the global private-path wildcard exception and pinning the repository-owned `commandf -> commandf-pkg` path dependency to `version = "=0.0.0"`. Final exact-head review and merge evidence closed T028/T029.

### Stack C — PR #45

```text
canonical input main: 301aa5e66089859e938145870dc4a9300a25692a
final head: c82ef6e6f137805074cc5e0c453d47e0d2799839
final head tree: 623a5b20eba83c618d4da288677c1cd3d2826f61
merge/main: a683dfaba7feb607145400eaa75d771e5df3c608
merge tree: 623a5b20eba83c618d4da288677c1cd3d2826f61
```

Exact-head workflow evidence:

```text
af01-scorecard: 33072451121 — success
cf06-oracle: 33072451114 — success
ci: 33072451162 — success
af01-security: 33072451128 — success
af01-assurance-proof: 33072451125 — success
```

Selected exact-head retained artifacts:

| Evidence | Artifact ID | GitHub digest |
|---|---:|---|
| `af01-scorecard` | `9646377101` | `sha256:7ee44633234035e3635380c14aa3941083bfa3977103d601f689b02857b6a41f` |
| `af01-t020-dependency-inventory` | `9646368036` | `sha256:39558e976e67ec58042eceecd08784b63c0ccc4ba2923fe478ad6a81a44fcf7b` |
| `af01-cargo-deny-proof` | `9646369305` | `sha256:6d3d28bebc077d51e47b1aeaf77dc84f1a7244c79dcae43fd7694f36e9b9a9b8` |
| `af01-rustsec-audit` | `9646469600` | `sha256:8b50c4eab65e915094f44ce58753297a2da93478a10606a1879b322c5713404e` |
| `af01-zizmor-proof` | `9646365145` | `sha256:18f0c5544183b8608ba08da20ccd27a5903e8b3fc1bb7c3cfc6de7fa2de804bb` |
| `af01-assurance-proof` | `9646480632` | `sha256:ac7aacb20c6d40abceb49738a05fc0d25208ae98f2f2a97e4ddbf3bb7afa692d` |

The exact-head assurance artifact independently recomputed:

```text
AF01_ASSURANCE_SHA256=8f01ae41ef552ce69a5094682c06364aa0a9a2ddcbc43aa76c219e003b8ec8e7
assurance-summary.json sha256=8f01ae41ef552ce69a5094682c06364aa0a9a2ddcbc43aa76c219e003b8ec8e7
source SHA=c82ef6e6f137805074cc5e0c453d47e0d2799839
source tree=623a5b20eba83c618d4da288677c1cd3d2826f61
```

OpenSSF Scorecard execution identity for the hardened final Stack C implementation:

```text
scorecard version: 5.5.0
scorecard source commit reported by result: c395761df6afe1a69e476bc60a013a94bcbc153f
release asset: scorecard_5.5.0_linux_amd64.tar.gz
release asset sha256: 83b90a05c1540ef1390db1cd5711e5fd04be9c1d8537fb84d39d02092d6a8dff
```

Fresh exact-head Qodo and CodeRabbit reviews found no remaining substantive false-PASS, security, correctness, governance, ruleset-layering, or sole-administrator maintenance-deadlock issue after remediation. The historical inline Qodo scanner finding was resolved; zero substantive unresolved review threads remained before merge.

## Live source-control enforcement

The checked-in ruleset payloads are configuration intent; the following live GitHub read-back is closure evidence.

### Assurance layer

```text
ruleset id: 21652953
name: commandF main assurance
enforcement: active
target: refs/heads/main
bypass actors: none
current_user_can_bypass: never
rules:
  - deletion prohibited
  - non-fast-forward prohibited
  - strict required status checks
```

Required checks are exactly:

```text
rust              integration_id=15368
assurance-proof   integration_id=15368
scorecard         integration_id=15368
```

Each selected context had exactly one terminal successful producer on the exact final Stack C PR head from GitHub Actions app/integration `15368`.

### Review-governance layer

```text
ruleset id: 21652974
name: commandF main review governance
enforcement: active
target: refs/heads/main
required approvals: 1
require Code Owner review: true
dismiss stale reviews on push: true
require last-push approval: true
require resolved review threads: true
allowed merge methods: merge only
sole bypass: RepositoryRole actor_id=5, bypass_mode=pull_request
current_user_can_bypass: pull_requests_only
```

GitHub's import read-back additionally materialized `required_reviewers=[]` and `require_extra_approval_for_unattributed_changes=true`. The latter is stricter than the checked-in minimum, does not weaken the reviewed boundary, and is retained as an observed live-platform strengthening rather than silently normalized away.

The separation is deliberate: the assurance ruleset has no bypass, so required checks/deletion/non-fast-forward protections remain unbypassable through the review-layer administrator exception; the review-layer exception is usable only through a pull request and prevents sole-administrator governance maintenance from deadlocking.

## Non-destructive negative governance proof

T040 was satisfied from authoritative live ruleset semantics plus exact-head check topology; no destructive force push, direct main mutation, or branch deletion was attempted.

The observed configuration proves:

- the assurance layer cannot be bypassed by the current administrator;
- branch deletion and non-fast-forward/force updates are prohibited by that unbypassable layer;
- the review-layer admin exception cannot be used as a direct-push exception because its bypass mode is `pull_request` only;
- stale approvals are invalidated and latest-push approval is required;
- unresolved review conversations block merge;
- required checks are strict and integration-bound;
- repository regressions prove every selected required context remains terminal on docs-only/path-nonmatching protected-branch PRs instead of depending on a whole workflow that can be skipped.

## Post-merge Stack C applicability

After PR #45 merged, canonical `main` became `a683dfaba7feb607145400eaa75d771e5df3c608` with tree `623a5b20eba83c618d4da288677c1cd3d2826f61`. Both rulesets remained active and GitHub reported `main` as protected.

Push-triggered post-merge evidence on that exact canonical SHA:

```text
af01-scorecard run: 33075836359 — success
af01-assurance-proof run: 33075836350 — success
post-merge assurance artifact: 9647922871
post-merge artifact digest: sha256:1c16ced5aad807a1214d864bf56f79395cabf0b600067ecf650fa36c65c97731
```

Every substantive post-merge assurance job step completed successfully, including source assertion/counterexamples, workflow-trust evidence, exact locked dependency evidence, cargo-deny, pinned cargo-audit/RustSec, zizmor, deterministic summary construction, and artifact retention.

The `rust` required context is pull-request scoped and is therefore not expected as a push check on the merge commit. Universal required-check authority applies to protected-branch pull requests; post-merge applicability is demonstrated separately by the push-triggered AF-01 proof and Scorecard workflows.

## T050 drift reconciliation

Phase 4 re-read the canonical AF-01 spec, plan, tasks, assurance-program document, consistency analysis, constitution, AGENTS, live GitHub policy, and the implementation diff from the pre-AF-01 base.

Observed drift and disposition:

1. `tasks.md` still showed T028–T043 incomplete despite exact-head/merge evidence. This convergence change reconciles those task states to completed.
2. Planning documents retain `Status: PLANNING_CANDIDATE` because they are the immutable authored planning contract snapshots. Current lifecycle state is carried by this convergence record and the task ledger rather than rewriting historical planning authority after implementation.
3. The Stack C Scorecard posture document intentionally preserves development-time findings, including the pre-ruleset `Branch-Protection=0` observation. Live ruleset evidence above supersedes that observation for closure without rewriting the historical scan.
4. Dependabot activated after Stack C and opened dependency-update PRs #46–#50. They are post-Stack-C drift inputs, not AF-01 closure dependencies and not implicitly trusted upgrades. They remain separate candidates requiring their own exact dependency/oracle/workflow qualification before merge.
5. Older open PRs #11, #39, #41, and #42 remain governed by their own CF/cache/oracle sequencing and are not pulled into AF-01 convergence.

## T052 product-semantic freeze proof

A repository compare from the pre-AF-01 canonical base
`8a45857bf31c4acae57fdfb1e3cdde3d0f7d0361` to Phase 4 entry main
`a683dfaba7feb607145400eaa75d771e5df3c608` is 186 commits ahead and contains no changed Rust source (`*.rs`) file.

No CF semantic fixture, CF-06 production oracle identity, CF-10 frozen corpus, structural/compatibility/policy/terminology/source-map/impact/gate semantic implementation file was changed by AF-01.

The only product manifest mutation is:

```toml
# before
commandf-pkg = { path = "../commandf-pkg" }

# after
commandf-pkg = { path = "../commandf-pkg", version = "=0.0.0" }
```

This was the reviewed Stack B remediation for a cargo-deny private-path wildcard policy defect. It tightens package-source/version authority for the existing local workspace edge and does not select a different package or alter runtime semantics. The exact final Stack B head ran `ci`, `cf06-oracle`, CF-11/11G/12/13 proofs, and `af01-security` successfully before merge; Stack C later re-ran the product/assurance gates on its exact final head.

Workflow/action shell wrappers changed only to bind execution to commandF-owned built artifacts and harden executable authority. Their product-facing semantic claims remained governed by the unchanged Rust implementation and passed the existing CF-08/CF-09 regressions in exact-head CI.

T052 conclusion:

```text
UNAUTHORIZED_CF_SEMANTIC_CHANGE: NONE OBSERVED
CF-06 PRODUCTION ORACLE IDENTITY: UNCHANGED
CF-10 FROZEN CORPUS: UNCHANGED
PRODUCT RUST SOURCE DIFF: NONE
INCIDENTAL PRODUCT MANIFEST CHANGE: REVIEWED SUPPLY-CHAIN IDENTITY TIGHTENING, FULLY QUALIFIED
```

## T053 retained assurance work and explicit limits

AF-01 establishes a trusted-development baseline only. It does not claim the following work complete:

### AF-02 — Adversarial Test Strength

Retained for a separate Spec Kit package:

- structure-aware/differential `cargo-fuzz`;
- property tests;
- `cargo-mutants` mutation adequacy;
- `cargo-llvm-cov` diagnostics/floors;
- `cargo-nextest` flaky-as-failure policy;
- minimized regression corpus promotion.

### AF-03 — Portability and Release Evidence

Retained for a separate Spec Kit package:

- Windows/macOS qualification alongside Linux;
- explicit MSRV proof;
- Rust public API/SemVer guard;
- SBOM;
- SLSA/GitHub artifact provenance;
- Sigstore/offline verification where adopted;
- stable release verification.

### AF-04 — Performance and Reliability Evidence

Retained for a separate Spec Kit package:

- benchmark/resource corpus;
- measured regression budgets;
- large package/graph stress;
- external-service sentinel separation;
- trend evidence reusable by future commandF Bench.

Additional explicit AF-01 limits:

- no repository license was selected by AF-01; legal-license selection is not inferred from cargo-deny dependency-license policy;
- Scorecard's aggregate score is not correctness authority;
- the two retained Scorecard Java/oracle vulnerability signals remain a separate exact-oracle dependency requalification concern; they are not converted into a Rust waiver or an unauthorized CF-06 pin mutation;
- AF-01 does not authorize CF-14/15/16 implementation, PHI, AI/model authority, a stable release claim, or a production-oracle identity change.

## Post-Stack-C dependency-update drift

The following live Dependabot PRs appeared after Stack C enabled update discovery:

```text
#46 actions/checkout 5.1.0 -> 7.0.1
#47 actions/upload-artifact 4.6.2 -> 7.0.1
#48 thiserror 1.0.69 -> 2.0.20
#49 jackson-databind 2.22.1 -> 2.22.2 in tools/hl7-oracle
#50 sha2 0.10.9 -> 0.11.0
```

Disposition: `SEPARATE_QUALIFICATION_REQUIRED`.

Reasons:

- #46/#47 mutate third-party workflow execution identities used by reviewed assurance/proof workflows;
- #48/#50 are semver-major Rust dependency changes and therefore cannot be treated as routine AF-01 metadata cleanup;
- #49 changes the external HL7 oracle dependency graph and must preserve the frozen CF-06/CF-10 authority boundary through exact oracle qualification.

None is merged or folded into this convergence candidate.

## Phase 4 gate state

At creation of this convergence record:

```text
T043 CLOSED_CANONICAL
T050 COMPLETE
T051 COMPLETE_IN_THIS_CANDIDATE
T052 COMPLETE
T053 COMPLETE
T054 OPEN — exact convergence head CI + Qodo + CodeRabbit required
T055 OPEN — exact qualified convergence PR merge + post-merge verification required
T056 OPEN — AF-01 must not be marked CLOSED_CANONICAL before T055 evidence
```

No `AF-01=CLOSED_CANONICAL` claim is made by this document before T055.