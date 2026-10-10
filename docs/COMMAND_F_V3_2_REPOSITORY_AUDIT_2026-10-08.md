# commandF V3.2 — Whole Repository Audit

Status: **PLANNING_CANDIDATE / AUDIT RECORD / NOT EXECUTION AUTHORITY**

**Current-state warning:** this Oct 8 snapshot predates the founder's Oct 10 zero-human-approval G53 policy, README, review-preview, G51 planning merge, and multiple signed technical PRs. See `docs/COMMAND_F_V3_2_FRONTIER_DELTA_2026-10-10.md` for current-state reconciliation; the observations below are retained for historical provenance.

This record describes the repository as observed on 2026-10-08. It does not change any canonical status, close any issue, or authorize implementation. V2 (`docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`) remains execution authority.

## 1. Observed live state

Read live from GitHub and from a fresh clone on 2026-10-08. These values were not copied from older records.

```text
repository:          TheHalfMoon/commandF (public)
default branch:      main
main:                f82565cca917d119e1c774b2c470e2ac20e0d6dd
main tree:           7b8a13949d8e52e2b0bdb952bca010d83450aeb5
last main merge:     PR #184, 2026-10-01T16:14:06Z
branches:            194
open pull requests:  20
open issues:         2 (#100, #15)
releases / tags:     none / none
repository license:  none detected by GitHub (no LICENSE file at root)
```

Active rulesets:

| Ruleset | ID | Enforced content |
| --- | --- | --- |
| commandF main assurance | 21652953 | Blocks deletion and non-fast-forward. Requires the `rust`, `assurance-proof`, and `scorecard` status checks with the strict policy. No bypass actors. |
| commandF main review governance | 21652974 | Merge commits only. One approval, Code Owner review, stale-review dismissal, last-push approval, and thread resolution are required. Repository role 5 (admin) may bypass in `pull_request` mode. |

`af02-base-verifier` is **not** a ruleset-required check. `.github/required-checks.json` lists the same three checks as the live ruleset.

## 2. Local verification performed for this audit

| Check | Result | Evidence |
| --- | --- | --- |
| `cargo test --workspace --all-features --locked` on Windows 11 x64 (Rust 1.97.1, MSVC) from a deep checkout path | 143 tests passed and 1 failed, across 57 suites. | The failing suite is `af02_a1_corpus_bijection`, test `af02_t037_checked_in_corpus_proves_bijection_bounds_and_no_phi`. It failed with `LNK1104: cannot open file` on the nested build of `tools/af02-verifier` at a 261-character path. |
| The same test from a short checkout path | 2 passed and 0 failed. | This is consistent with Windows `MAX_PATH` (260 characters). The failure is reproducible and depends on path length, not on behavior. |
| CI runner families | All 16 workflow `runs-on` entries are `ubuntu-24.04`. | `.github/workflows/*.yml` |

This is the first recorded Windows execution evidence for the workspace. It is a single machine, not a portability qualification.

## 3. Capability inventory

Classification vocabulary:

- `IMPLEMENTED_AND_VERIFIED` means code exists, tests exist, and the canonical record states closure with exact-head evidence.
- `IMPLEMENTED_BUT_NOT_QUALIFIED` means code and tests exist, but closure evidence is missing, inconsistent, or limited to a narrower scope than the name suggests.
- `PLANNING_ONLY`, `BLOCKED`, `FUTURE_RESEARCH`, and `OUT_OF_SCOPE` keep their plain meanings.

| Capability | Owner | Surface | Classification | Evidence and qualification |
| --- | --- | --- | --- | --- |
| Package resolve, lock, verify, and content-addressed cache | CF-01 | `commandf pkg resolve` and `commandf pkg verify` | IMPLEMENTED_BUT_NOT_QUALIFIED | `crates/commandf-pkg/src/{resolver,lock,cache,registry,source}.rs` and the tests `resolution.rs`, `cache_and_constraints.rs`, and `lock_schema.rs`. Spec 001's convergence header still says "Draft readiness candidate". |
| Canonical inspect | CF-02 | `commandf inspect` | IMPLEMENTED_BUT_NOT_QUALIFIED | `artifact_inspect.rs` and `inspection.rs`. The convergence header says "founder review candidate". |
| Structural diff | CF-03 | `commandf diff` | IMPLEMENTED_BUT_NOT_QUALIFIED | Five `structural_diff*` test suites. Same header state as CF-02. |
| Compatibility rules (70 unique `CF04-*`/`CF07-*` rule IDs; Breaking, Risky, and Additive; producer and consumer direction) | CF-04 | `commandf classify` | IMPLEMENTED_BUT_NOT_QUALIFIED | `compatibility*.rs` and `compatibility_rule_coverage.rs`. Unsupported fields fail closed (`CompatibilityError::UnsupportedStructuralField`). No change-space coverage proof exists (G02). |
| SARIF / CI gate | CF-05 | `commandf check --format sarif` | IMPLEMENTED_BUT_NOT_QUALIFIED | `check*.rs` |
| HL7 validator differential oracle | CF-06 | `commandf oracle` and `tools/hl7-oracle` | IMPLEMENTED_AND_VERIFIED for the 6.10.2 pin. Pin change BLOCKED (issue #15). | `oracle_*.rs` and `cf06-oracle.yml`. The oracle core is hard-coded to FHIR `4.0.1` (`crates/commandf-cli/src/oracle.rs:14`). |
| Terminology diff | CF-07 | `commandf terminology` | IMPLEMENTED_BUT_NOT_QUALIFIED | Three `terminology_*` suites. The header says "Candidate". |
| GitHub annotations / Action | CF-08 | `commandf github-annotations` and `action.yml` | IMPLEMENTED_BUT_NOT_QUALIFIED | The header says "Candidate". |
| FSH source mapping | CF-09 | `commandf source-map` | IMPLEMENTED_BUT_NOT_QUALIFIED | `source_map*.rs` |
| Public real-IG delta corpus | CF-10 | — | BLOCKED | PR #11 has been a draft since 2026-08-16. It depends on the CF-06 production-oracle contract (issue #15). |
| Multi-version package graph | CF-11 | library | IMPLEMENTED_AND_VERIFIED | Convergence `CLOSED_CANONICAL`. |
| Ecosystem context graph | CF-11G | `commandf context` | IMPLEMENTED_AND_VERIFIED | Convergence `CLOSED_CANONICAL`. |
| Impact | CF-12 | `commandf impact` | IMPLEMENTED_BUT_NOT_QUALIFIED | The convergence record is `CONVERGENCE_CANDIDATE`, while the plan index treats later work as post-CF-13. |
| Baselines, suppression, and quality gates | CF-13 | `commandf gate` | IMPLEMENTED_BUT_NOT_QUALIFIED by its on-disk record | The plan index says `CF-13: CLOSED_CANONICAL`, but `specs/014-.../spec.md` says `PLANNING_CANDIDATE` and its convergence says `CONVERGENCE_CANDIDATE`. |
| Trusted development baseline | AF-01 | CI | IMPLEMENTED_AND_VERIFIED | `specs/015-.../closeout.md` is `CLOSED_CANONICAL`, and the rulesets are live. |
| Adversarial test strength | AF-02 | `tools/af02-verifier`, `fuzz/` | IMPLEMENTED_BUT_NOT_QUALIFIED (partial) and BLOCKED for further planning | The planning candidate (T001–T006) is unchecked. The gate deadlock is recorded in specs 049 and 050, and §5.1 here. |
| Portability / release evidence | AF-03 | — | PLANNING_ONLY | No Spec Kit and no release. CI runs on Linux only. |
| Performance evidence | AF-04 | — | PLANNING_ONLY | — |
| On-prem aggregate source profiler | CF-14 | — | PLANNING_ONLY | — |
| Verified dry-run recipes | CF-15 | — | PLANNING_ONLY | — |
| Mapping analysis IR (parse-only) | CF-16 | — | PLANNING_ONLY | — |
| Ecosystem observatory primitives | CF-17 (specs 025–046) | **library only, with no CLI surface** | IMPLEMENTED_BUT_NOT_QUALIFIED. The catalog is BLOCKED. | About 7,274 lines in `ecosystem_*.rs` and `durable_retained.rs`, roughly 37% of the 19,607 Rust source lines. `crates/commandf-cli/src/*.rs` has zero references to `ecosystem`. |
| Verified-byte and bounded-input repairs | specs 020–024 | existing commands | IMPLEMENTED_BUT_NOT_QUALIFIED by record | Issues #35, #36, #37, #38, and #40 were closed on 2026-09-29. The spec headers still say `SPEC_CANDIDATE`, and spec 018 says those issues "stay open until their underlying invariants are proven". No equivalent-pattern audit record exists (V3.1 G49). |
| Durable retained authority | spec 019 / issue #100 | — | BLOCKED | The historical bytes are unavailable. This is recorded honestly in `specs/019-.../historical-packet.json`. |
| Decision Envelope, Consumer Contract, and Decision Receipt | CF-18/19/21 (V3.1) | — | PLANNING_ONLY | — |
| CommandFBench | research | — | PLANNING_ONLY | `research/CANDIDATE_CORPUS.md` has 3 items, all from one HAPI repository. The protocol is `NOT_FROZEN` and the split is `NOT_ASSIGNED`. |
| FHIR version awareness (R4B, R5, R6) | G08/G09 | — | PLANNING_ONLY | No version-dispatch logic exists in `crates/*/src`. `fhirVersion` is treated as an opaque structural field. |
| Studio, model advisor, and public observatory | product family | — | PLANNING_ONLY / FUTURE_RESEARCH | — |

## 4. Existing strengths worth preserving

1. **Fail-closed semantics are real code, not prose.** Unsupported diff schemas and fields return typed errors (`compatibility_error.rs`) and do not produce a silent classification.
2. **Exact-head assurance discipline.** Required checks are strict, every action reference is pinned to a full SHA (`workflow-trust-policy.json`), and `zizmor`, `cargo-deny`, `cargo-audit`, and Scorecard all run.
3. **Verified-byte discipline at the cache boundary.** `PackageCache::read_verified` exists, and the V3.1 repairs bound reads before allocation.
4. **Oracle humility.** HL7 is pinned (6.10.2, source `d06577db…`, jar `a3addadf…`) and treated as an advisory oracle. Divergences are classified, not laundered.
5. **Honest negative records.** Specs 042, 047, 048, 049, and 050 and the issue #100 packet record what could not be done, without inventing artifacts.
6. **A deterministic CLI spine already covers most of the first product journey:** `pkg`, `inspect`, `diff`, `classify`, `check`, `gate`, `impact`, `context`, `terminology`, `oracle`, `source-map`, and `github-annotations`.

## 5. Important discoveries

### 5.1 The AF-02 base gate is socially blocking but not ruleset-enforced, and it freezes dependency maintenance

`tools/af02-verifier/src/base_gate.rs` treats the following as immutable authority:

- `AUTHORITY_EXACT`: `Cargo.lock`, `Cargo.toml`, `crates/commandf-pkg/src/oracle_model.rs`, and four `.github/*.json` policy files.
- `AUTHORITY_PREFIXES`: `.github/scripts/`, `.github/workflows/`, `donors/`, `specs/016-…/`, and `tools/af02-verifier/`.

Consequences observed on live PRs:

| PR | Change | `af02-base-verifier` | Required checks |
| --- | --- | --- | --- |
| #98 | clap 4.6.6 → 4.6.7 (Cargo.lock) | fail | rust, assurance-proof, and scorecard pass |
| #95 | ureq 3.4.0 → 3.4.2 | fail | pass |
| #97 | actions/setup-java 5.7.0 → 6.0.1 | fail | assurance-proof also fails |
| #101 | rustls security fix (RUSTSEC-2026-0285) | **fail, and the PR was merged** | pass |

So every Cargo dependency update, every workflow update, and every donor-record edit is rejected by a gate that is not part of the merge contract. The project has already merged past that rejection once, for a security fix (#101). That precedent shows the gate is de facto advisory. However, no written rule says when the gate's rejection may be overridden. That missing rule, not the gate itself, is the governance defect. G51 in the gap ledger owns it.

### 5.2 CF-17 consumed about 37% of the code base without producing a user-visible command

Specs 025 through 046 are 22 merged grains. Nine of them (032–040) serialize "machine bytes" for one document type each. None of these modules is reachable from `commandf-cli`. The planned multi-package catalog is still `NOT_AUTHORIZED` (spec 048). The engineering quality is high, but the cost-to-value ratio is the largest risk to the product: an interoperability engineer cannot use any of it. G52 owns this.

### 5.3 Canonical status cannot be derived mechanically from the tree

Status headers disagree with the plan index and with live issue state. Examples:

- CF-13 is `PLANNING_CANDIDATE` on disk and `CLOSED_CANONICAL` in the index.
- Specs 020–024 are `SPEC_CANDIDATE`, but their issues are closed.
- Spec 018 asserts that the now-closed issues stay open.
- Spec 049 says retained-source files are absent, and spec 050 corrects it.

A reader, agent, or migration gate that trusts a single file will reach a wrong conclusion. The V3.2 migration proposal therefore requires a generated status ledger as a precondition.

### 5.4 Distribution gap

There are no releases, no tags, and no root LICENSE file. The workspace sets `publish = false`. `README.md` still presents CF-01 as the first slice and does not mention `context`, `impact`, `gate`, or `check`. An external engineer has no supported way to obtain or legally reuse commandF. The license is a founder decision (FD-6).

### 5.5 Upstream oracle drift

`hapifhir/org.hl7.fhir.core` released `6.10.3` (2026-08-26), `6.10.4` (2026-09-04), and `7.0.0` (2026-10-06). The 7.0.0 notes say internal processing moves to R6 and that snapshot generation was reworked. Upstream fix PR #2554, the dependency of commandF issue #15, is still OPEN and unmerged. Its last update was 2026-09-05, after reviewer revision requests. Dependabot PR #93 proposes `hl7.fhir.version` 6.10.4. Merging it would change the CF-06 production pin without qualification, which is explicitly prohibited.

### 5.6 FHIR-version semantics are implicit R4

No source path dispatches on FHIR version. The oracle uses core `4.0.1`. Any R4B or R5 result today is an accident of structural similarity, not a supported capability. G08/G09 already own this. The finding raises their priority for the first product journey.

### 5.7 AF-03 is not ordered behind AF-02 by execution authority

Specs 047 and 048 justify deferring AF-03 with "the playbook dependency graph is AF-02, then AF-03, then AF-04". That serial chain comes from `docs/COMMAND_F_V3_EXECUTION_PLAYBOOK.md` §3, which is a planning candidate (authority rank 5 in the V3.1 runbook §1). V2 says AF-02/03/04 "are sequenced in the Assurance Program document". `docs/COMMAND_F_ASSURANCE_PROGRAM_2026-08-26.md` ("Ordering and product-roadmap relationship") draws AF-02, AF-03, and AF-04 as **parallel** children of AF-01. Its only ordering rules are:

- AF-03 must close before a stable public release claim.
- AF-04 must close before quantitative performance claims.

AF-01 is `CLOSED_CANONICAL`. Under the current execution authority, **an AF-03 planning Spec Kit is therefore eligible now** and does not wait for the AF-02 deadlock. Implementing AF-03 is a separate matter: it would edit `.github/workflows/`, which is an AF-02 authority prefix, and so it still meets the G51 governance defect in practice. This observation corrects the reasoning in specs 047 and 048. It does not edit those records.

## 6. Open pull request triage (recommendation only; nothing merged)

| PR | Recommendation | Reason |
| --- | --- | --- |
| #186 jackson-databind 2.22.3 (`tools/hl7-oracle`) | Review normally. | It touches no AF-02 authority path, all checks pass, and it is an oracle-tool dependency. Confirm CF-06 proof parity before merge. |
| #93 hl7.fhir.version 6.10.4 | **Do not merge.** Close it with a reference to issue #15. | It is a production oracle pin change without differential qualification. 7.0.0 now exists, and a later qualified repin should target an exact chosen release. |
| #78 maven-compiler-plugin | Review after #186. | Build-tool only. |
| #98, #95, #81 (cargo) | Hold until FD-1 (gate amendment rule) is decided, then batch. | Blocked only by the non-required AF-02 gate (§5.1). |
| #50 sha2 0.11, #48 thiserror 2 | Rebase or close. | Major-version API changes, and #50's `rust` check fails. Reopen deliberately under AF-03. |
| #97, #96, #47, #46 (actions) | Hold for FD-1, then a workflow-pin refresh grain. | Workflow paths are AF-02 authority. #97 also fails assurance-proof. |
| #185 docs/abstract-sap1-tokens | Review normally. | Research-text alignment. |
| #169 Graft context layer | Founder decision. | A tooling change, not product scope. Verify that it adds no authority paths. |
| #89 T031 archive fuzz target | Keep open. It is the AF-02 Stack A1 grain. | Re-qualify after the AF-02 governance migration (Phase 0). |
| #83 Tencent source study (draft) | Close or convert into the source-qualification matrix. | Superseded by the V3.2 matrix. |
| #42, #41, #39 (verified-bytes drafts) | Close as superseded, after confirming the merged repairs cover them. | The issues are closed. The drafts predate the merged fixes. |
| #11 CF-10 corpus (draft) | Keep, BLOCKED by #15. | The frozen corpus must not be mutated. |

## 7. Architectural debt

1. **Duplicated canonical-serialization helpers.** Each `ecosystem_*` module carries its own byte-emission code (specs 032–040). V3.1 G50 already names shared assurance primitives. The concrete debt is now measurable.
2. **`main.rs` concentration.** `crates/commandf-cli/src/main.rs` is 768 lines and contains command dispatch, archive selection, and policy assembly. CF-23 should extract a typed application layer before Studio or LSP surfaces exist, so that no second semantic engine appears.
3. **Status metadata as hand-written prose** (§5.3).
4. **A gate that cannot be amended by any path the project has written down** (§5.1).
5. **Implicit FHIR R4** (§5.6).
6. **Linux-only CI with a reproducible Windows path-length failure** (§2).

## 8. What this audit does not establish

It does not establish:

- correctness of any classification against independent labels;
- performance;
- clinical safety;
- release readiness;
- that closed issues #35–#40 are fully remediated, because no equivalent-pattern audit record was found;
- any property of a FHIR version other than R4.
