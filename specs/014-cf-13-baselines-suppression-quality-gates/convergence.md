# CF-13 Convergence — Baselines, Suppressions, and Quality Gates

Status: CONVERGENCE_CANDIDATE

This document closes the implementation evidence for CF-13 without changing the already qualified product tree. It is not canonical until this docs-only closeout qualifies on its own exact head and merges to `main`.

## Canonical sequence

CF-13 was delivered in the repository-authorized order:

| Unit | PR | Final head | Canonical merge |
| --- | ---: | --- | --- |
| Planning / Spec Kit | #30 | `33a0536d745d67ac6a094ce891293efa7e2204b9` | `cb3d0824d795b06d40bd121798030be15bba507c` |
| Stack A — deterministic quality-gate library | #31 | `8bdca1bc66539058310249f5841ece9fca2a437a` | `82bf9d69c8b574ba7f302296e08b416d7566a351` |
| Stack B — shipped `commandf gate` + proof | #32 | `06da4f3f61b47afe11525b2c33306b5952cd680e` | `4b2ddf7d9579e7dbed0759f69de56544e7ab8fb3` |

Post-Stack-B canonical main:

```text
main: 4b2ddf7d9579e7dbed0759f69de56544e7ab8fb3
tree: 6707735a3d3521380ab22a31d4a0865982fadd6a
implementation parent: 06da4f3f61b47afe11525b2c33306b5952cd680e
```

The merge commit is GitHub signature-verified and the canonical merge tree exactly equals the final qualified Stack B implementation tree.

## Final implementation exact-head qualification

Every path-applicable repository workflow observed on final implementation head `06da4f3f61b47afe11525b2c33306b5952cd680e` completed successfully:

| Workflow | Run | Result |
| --- | ---: | --- |
| `ci` | `32978131562` | `SUCCESS` |
| `cf06-oracle` | `32978131527` | `SUCCESS` |
| `cf11-multi-version-proof` | `32978131447` | `SUCCESS` |
| `cf11g-context-proof` | `32978131464` | `SUCCESS` |
| `cf12-impact-proof` | `32978131498` | `SUCCESS` |
| `cf13-quality-gate-proof` | `32978131520` | `SUCCESS` |

The exact-head `ci` run includes the mandatory workspace gates and configured real-FHIR/security regressions:

```text
cargo fmt --all -- --check                         PASS
cargo clippy --workspace --all-targets --all-features -- -D warnings  PASS
cargo test --workspace --all-features             PASS
```

No prior-head workflow result is substituted for this final implementation head.

## Dedicated deterministic proof identity

```text
CF13_SOURCE_SHA=06da4f3f61b47afe11525b2c33306b5952cd680e
CF13_SOURCE_TREE=6707735a3d3521380ab22a31d4a0865982fadd6a

workflow run=32978131520
proof job=98207812843
artifact id=9610321732
artifact digest=sha256:4e1f8e0cf4167e77153e2d5ff8749d146881a1a6c20608f743c2c44a71c5a8fe

CF13_GATE_SHA256=118fdd9e7606394d4abcbb39b51e0af81d303c95e3a513886acb1bedb95e93cf
```

The retained proof binds the evaluation to immutable repository and input provenance required by the CF-13 specification, including:

- exact source SHA and tree;
- governing CF-13 spec/plan/tasks and repository governance identities;
- relevant CF-04/CF-05 implementation authority identities;
- pinned Rust/toolchain and GitHub Action identities;
- `Cargo.lock` repository/blob/exact-byte identity;
- exact synthetic input/fixture digests;
- exact before/after package names, versions, and archive SHA-256 identities;
- canonical baseline and suppression evidence digests;
- final byte-stable gate report digest.

The final proof retained, among its machine evidence:

```text
baseline canonical digest=sha256:256ee30bc6c91b03b11301c047819949123365ad1ae6ef56e47bc69b3dd41209
suppression canonical digest=sha256:06758f7e631f7474577a8a3de26f3891d34f0f48cf6695dcce22ccb581c131a9
before archive sha256=ad2179da72996d681a46cc1dbf9b97ed31f9d38abccf1d945a55819e32100bd3
after archive sha256=a8c02de7ef919781808a8a66b89ecc5e6c259e4a8415b1d28022e5ed2897e61c
```

Those identities are proof evidence; they do not grant runtime policy or semantic authority.

## Independent review truth

### Qodo

Qodo's implementation review originally found multiple substantive defects across the Stack B development series, including source/proof identity, verified-read correctness, primary-input bounds, and missing oversized optional-input regression coverage. The valid findings were fixed and regression-tested before final qualification.

On exact final implementation head `06da4f3f61b47afe11525b2c33306b5952cd680e`, after the missing suppression-boundary regression was consolidated into `gate_bounds.rs`, Qodo performed a fresh review and reported no substantive issues. The corresponding finding thread was resolved only after that exact-head disposition.

### CodeRabbit

A CodeRabbit finding on an intermediate temporary test file identified a real rustfmt failure. That temporary file was subsequently deleted and the regression was consolidated into the existing `gate_bounds.rs`; exact-head CI then passed format, Clippy, and tests. CodeRabbit confirmed that the intermediate finding no longer applied and withdrew/resolved the thread.

A fresh CodeRabbit incremental review completed on exact final head `06da4f3f61b47afe11525b2c33306b5952cd680e` with status `success / Review completed` and reported:

```text
No actionable comments were generated in the recent review.
```

CodeRabbit also retained two non-blocking reviewer metadata items that are not represented as repository behavioral PASS gates:

1. docstring coverage warning (`28.81%` against CodeRabbit's configured `80%` threshold);
2. a bounded operational note that an invocation failing before successful output publication can leave a pre-existing report file in place, so consumers must bind machine evidence to the command exit status and/or verify report freshness.

The second note does not change the CF-13 atomic-publication contract: complete reports are published before completed policy-failure exit `2`; operational/input failures remain exit `1` and are not evidence of a completed gate evaluation.

### Thread state

```text
UNRESOLVED_SUBSTANTIVE_REVIEW_THREADS=0
CODEX_REVIEW_USED=NO
```

No unavailable reviewer or automated status is promoted into a stronger approval claim than the returned evidence supports.

## V1 acceptance convergence

The canonical implementation proves the CF-13 V1 contract required by `spec.md`:

- `commandf gate` exposes exact before/after package state, CF-05 direction/threshold, optional baseline/suppressions, JSON output, and optional output path;
- exact historical baseline membership and exact suppressions alter only adoption-layer disposition/gate blocking, never CF-04 severity or embedded CF-05 evidence;
- suppressions are explicit fingerprint-version-aware evidence with mandatory rationale and no wildcard authority;
- stale suppressions remain visible and cannot hide an unmatched current finding;
- malformed, oversized, unsupported-version, duplicate, inconsistent, or tampered evidence fails closed;
- persisted reports retain enough baseline/suppression membership to revalidate `baseline` and `suppressed` dispositions without trusting unseen external files;
- semantic JSON object-key reordering is canonicalized while array order remains identity-bearing;
- repeated identical evaluation produces byte-identical report bytes;
- primary lock/cache inputs and optional baseline/suppression inputs are explicitly bounded;
- CF-05 `commandf check` behavior/schema/SARIF/exit semantics remain regression-stable;
- the dedicated proof records immutable source/input/toolchain authority and clean-worktree evidence.

## Security and authority boundary preserved

CF-13 introduces no:

- package acquisition or network lookup in `commandf gate`;
- PHI or instance-data requirement;
- arbitrary suppression predicate or executable policy;
- current-time/expiry authority;
- model/agent compatibility authority;
- wildcard/rule-wide/severity-wide/resource-wide suppression;
- CF-04 ruleset reinterpretation;
- CF-05 public report-schema mutation;
- CF-06 production-oracle identity change;
- frozen CF-10 corpus change;
- lock-schema change;
- new dependency.

The deterministic engine and retained evidence remain authority. Reviewers and future AI/agent systems may propose or explain changes but do not become semantic authority.

## Explicit V1 limits and deferrals

CF-13 V1 intentionally defers:

- wildcard/regex/rule/resource/severity-wide suppressions;
- clock-based suppression expiry;
- shared or remote baseline stores;
- GitHub issue/reference validation;
- organization-wide policy/profile languages such as CEL/Rego/CUE;
- multi-package aggregate gate profiles;
- SARIF disposition overlays that hide accepted findings;
- automatic suppression generation;
- AI/model/agent waiver authority;
- impact-informed compatibility severity;
- any reinterpretation of CF-12 reachability as BREAKING/RISKY/ADDITIVE truth.

The operational stale-output note from CodeRabbit is retained as a consumer integration boundary: a prior output file is not evidence that a later operationally failed invocation completed. Consumers must use exit truth and expected input/report identity rather than filesystem existence alone.

## Closeout qualification gate

This convergence branch is documentation-only. It changes no Rust source, test code, workflow logic, dependency, lockfile, fixture, oracle identity, corpus, runtime behavior, or product semantics.

Before this closeout may merge:

1. identify the exact final closeout head and tree;
2. inspect only workflows actually triggered/applicable to that exact docs head;
3. require every triggered mandatory workflow to be terminal and successful;
4. request/inspect Qodo and CodeRabbit when available;
5. disposition every returned substantive finding;
6. require unresolved substantive review threads = `0`;
7. merge only with an exact expected-head guard;
8. verify post-merge `main` SHA/tree.

No workflow is invented as a closeout requirement merely because it ran on the implementation head.

## Closure decision

```text
IMPLEMENTATION_QUALIFIED=YES
IMPLEMENTATION_MERGED=YES
CONVERGENCE_RECORDED=YES
CLOSEOUT_EXACT_HEAD_QUALIFIED=PENDING
CLOSED_CANONICAL=NO
```

After this exact docs-only closeout is independently qualified and merged without content-changing substitution, canonical repository truth may classify:

```text
CF-13=CLOSED_CANONICAL
```

Only then may the next repository-authorized architectural/planning unit begin.