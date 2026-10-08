# commandF V3.2 — Gap Reconciliation Ledger

Status: **PLANNING_CANDIDATE / NOT EXECUTION AUTHORITY**

Sources:

- G01–G40: `docs/COMMAND_F_PLAN_GAP_REVIEW_2026-09-12.md` and `docs/COMMAND_F_V3_EXECUTION_PLAYBOOK.md` §7.
- G41–G50: `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` §15.

Observed canonical main: `f82565cca917d119e1c774b2c470e2ac20e0d6dd`.

## Rules

1. Every gap has exactly one **primary owner**. Contributing slices are listed separately, so no gap is owned twice.
2. A gap is never closed by this document.
3. `V3.2 phase` refers to the roadmap (`COMMAND_F_V3_2_EXECUTION_ROADMAP.md`).
4. "Amended" means V3.2 adds a requirement to an existing gap instead of creating a new one.
5. Primary owners were chosen from the slice listed first in the source document. Where the source names several slices, the first is primary and the rest contribute. This makes the ownership deterministic.

State vocabulary:

| State | Meaning |
| --- | --- |
| `OPEN` | No closure work exists. |
| `PARTIAL` | Merged code covers part of the closure contract. Evidence is named. |
| `BLOCKED` | Cannot progress without a named decision or upstream change. |
| `DEFERRED` | Intentionally out of the first product journey. |

## G01–G40

| ID | Primary | Contributing | State | Evidence today | V3.2 phase | V3.2 amendment |
| --- | --- | --- | --- | --- | --- | --- |
| G01 | CF-17 | — | PARTIAL / BLOCKED | Specs 025–046 merged. The catalog is `NOT_AUTHORIZED` (spec 048). | P5 (catalog) | The catalog source must be one of: row 2 (`package-feeds.json` at a pinned commit+blob, after FD-5 rights) or row 3 (registry `catalog` query with a `QUERY_RESULT_AT_TIME` label). |
| G02 | CF-18 | — | PARTIAL | `compatibility_rule_coverage.rs` and fail-closed `UnsupportedStructuralField` | P2 | The coverage matrix is a generated, CI-checked artifact (architecture §7). |
| G03 | CF-18 | — | OPEN | — | P2 | — |
| G04 | CF-19 | — | OPEN | Context-graph edges only | P3 | The first extraction methods are `DECLARED_FILE` + `CAPABILITY_STATEMENT` + `VIEW_DEFINITION`. |
| G05 | CF-19 | — | OPEN | — | P3 | Surfaced through `commandf review --protected <contracts>`. |
| G06 | CF-20 | — | PARTIAL | CF-06 single-oracle divergence | P6 | The `independence_group` field is required (HAPI and the HL7 validator share a core). |
| G07 | CF-17 | CF-20, CF-22 | PARTIAL | `require_published_authority` and the telemetry partition | P2 | Add the class `ADVISORY_MODEL_OBSERVATION`, which is never upgradable. |
| G08 | CF-18 | — | OPEN | No version dispatch (audit §5.6) | P2 | R4 `SUPPORTED`. R4B and R5 are `PARSE_ONLY`/`UNSUPPORTED` with witnesses. |
| G09 | CF-18 | CF-23 | OPEN | — | P2 | — |
| G10 | CF-20 | CF-21 | PARTIAL | The CF-06 oracle | P6 | — |
| G11 | CF-21 | — | OPEN | — | P6 | — |
| G12 | CF-21 | — | OPEN | — | P4 | The Interoperability BOM is a section of the receipt bundle, not a separate product. |
| G13 | CF-18 | — | OPEN | Rules are versioned. No lifecycle exists. | P2 | — |
| G14 | CF-18 | CF-23 | PARTIAL | SARIF and fingerprints | P2 | — |
| G15 | CF-17 | — | PARTIAL | `ecosystem_cache_identity.rs` (identity only) | P7 | Scale is measured only after AF-04. |
| G16 | CF-23 | — | OPEN | `main.rs` holds the dispatch logic | P4 | Extract the application layer before Studio or LSP exist. |
| G17 | CF-24 | — | DEFERRED | — | P8 | — |
| G18 | CF-15 | CF-19, CF-22 | DEFERRED | — | P7 | — |
| G19 | CF-21 | — | OPEN | Offline replay of one observation (spec 041) | P4 | The bundle admission errors port MedScale `medscale-pack` admission. |
| G20 | CF-23 | AF-03 | OPEN | No releases | P1 | The first release (v0.1.0 of the existing CLI) is promoted ahead of new semantics. See also G52. |
| G21 | CF-27 | — | DEFERRED | — | P8+ | — |
| G22 | CF-17 | — | PARTIAL | Spec 046 telemetry partition | — | — |
| G23 | CF-20 | CF-21 | OPEN | CF-09 source map | P6 | — |
| G24 | CF-22 | — | DEFERRED | — | P7 | — |
| G25 | CF-18 | CF-28 (research lane) | OPEN | AF-02 property tests are planned | P2 | Includes the aggregation-algebra property tests (architecture §6) and advisory monotonicity. |
| G26 | V3 reconciliation gate | all slices | PARTIAL | Donor policy; this matrix | P0 | Donor records cannot be written until G51 closes, because `donors/` is AF-02 authority. |
| G27 | CF-17 | — | PARTIAL / closed for the requirement | Spec 026 | — | — |
| G28 | CF-17 | donor policy | PARTIAL | Spec 028 | — | — |
| G29 | CF-21 | CF-25 | OPEN | — | P4 | **Amended** to six rights dimensions (S, C, T, D, M, TM), including model-weight rights (LFM v1.0 threshold) and trademarks. |
| G30 | CF-18 | CF-23 | OPEN | — | P2 | — |
| G31 | CF-18 | CF-19, CF-20 | DEFERRED beyond P3 | — | P6 | Only CapabilityStatement-declared REST and search is in P3. |
| G32 | CF-18 | CF-23 | PARTIAL | `gate_model.rs` policy | P2 | — |
| G33 | CF-18 | CF-23 | PARTIAL | Typed error enums per slice | P2 | Unified as `commandf.error/v1` (architecture §7). |
| G34 | CF-19 | CF-20 | DEFERRED | — | P6 | — |
| G35 | CF-21 | CF-23 | OPEN | — | P4 | — |
| G36 | CF-17 | CF-19, CF-23 | PARTIAL | Spec 039 workspace identity | P3 | — |
| G37 | CF-27 | — | DEFERRED | — | P8+ | — |
| G38 | CF-24 | — | DEFERRED | — | P8 | — |
| G39 | CF-20 | CF-28 | DEFERRED | — | research | — |
| G40 | CF-19 | CF-24 | OPEN (scope statement only) | — | P3 | — |

## G41–G50

| ID | Primary | Contributing | State | V3.2 phase | V3.2 amendment |
| --- | --- | --- | --- | --- | --- |
| G41 | CF-18 | CF-23 | OPEN | P2 | **Amended:** the aggregation algebra across (change cluster × consumer) pairs, plus the `COMBINED_CHANGE_NOT_MODELED` rule (architecture §6). |
| G42 | CF-18 | CF-20 | OPEN | P2 | — |
| G43 | CF-21 | CF-23 | OPEN | P4 | **Amended:** the separation of `sid`, `did`, and `rid`, plus the replay contract (architecture §5). |
| G44 | CF-19 | CF-23 | OPEN | P3 | **Amended:** the `ai_feature_contract` family is added as FUTURE_RESEARCH, not P3 scope. |
| G45 | CF-28 | — | OPEN | P1 → P9 | Owned by the CommandFBench plan. |
| G46 | CF-18 | CF-22, CF-28 | OPEN | P2 | Measured on `did`. |
| G47 | CF-20 | — | OPEN | P6 | — |
| G48 | CF-18 | CF-23, CF-24, CF-28 | OPEN | P8 (D1 study runs in parallel from P1) | **Amended:** process isolation, the monotonicity invariant, injection testing, exact model identity, and the license-aware distribution (D1 plan). |
| G49 | V3 migration gate | — | PARTIAL | P0 | Issues #35, #36, #37, #38, and #40 are closed, but no equivalent-pattern audit record exists. P0 produces one. |
| G50 | AF program | all slices | PARTIAL | P2 | Measured debt: per-module canonical byte emission in specs 032–040. |

## New gaps justified by V3.2

Before adding each gap, it was checked against G01–G50 for an existing owner. Each is new only because no existing closure contract covers it.

| ID | Sev | Gap | Why no existing G covers it | Primary owner | Closure contract |
| --- | --- | --- | --- | --- | --- |
| **G51** | P0 | Governance gates have no written, independently reviewable amendment path. The AF-02 base gate rejects every change to `Cargo.lock`, workflows, and donor records, yet it is not a required check and has already been merged past once (PR #101). | G26 covers donor qualification. G50 covers shared primitives. Neither covers how a gate admits changes to itself or its protected paths. | **Governance migration Spec Kit (proposed sequence 051)** | (1) A founder-ratified amendment rule (FD-1). (2) A verifier capability that admits a change to authority paths only when it is bound to an amendment record naming exact base and head blob SHAs, the rationale, an independent review reference, and the founder approval. (3) Regression tests showing every edit class rejected before is still rejected without such a record. (4) After the bootstrap, `af02-base-verifier` becomes a ruleset-required check (FD-2). |
| **G52** | P0 | Implemented capability is unreachable by users: no release, no CF-17 command, and a stale README. | G20 covers install and update UX for a release; it does not require that merged library capability is user-reachable or that slices deliver user value. | **CF-23** (with AF-03 contributing) | Every merged product slice states its user-visible surface or an explicit `LIBRARY_ONLY_UNTIL:<slice>` record. A first tagged release ships the existing CLI with verification instructions. The README is regenerated from the capability ledger. |
| **G53** | P1 | Decisions lack deterministic, accessible human explanations independent of any model, and UI surfaces lack accessibility and error-recovery requirements. | G14 covers the finding schema and G16 the surfaces. Neither covers template-rendered explanations, the model/no-model text separation, or accessibility. | **CF-23** | Stable `explanation_codes` with versioned templates. Snapshot tests across CLI, SARIF, and Studio. Advisory text visually and structurally separated. WCAG 2.2 AA checks in the Studio CI. |
| **G54** | P0 | Package-source authenticity and namespace or dependency confusion are not modeled. | G28 covers source lifecycle and staleness. G26 covers donors. Neither covers runtime package authenticity. | **CF-01** (amendment Spec Kit), with CF-17 contributing | Authenticity states as evidence (architecture §7). Fail closed on `NAMESPACE_RESTRICTION_VIOLATED`. `UNAUTHENTICATED_TRANSPORT` is never sufficient alone. Adversarial fixtures: a look-alike name, a same name from a different feed, and an http-only feed. |

## Items checked and deliberately not made into new gaps

| Candidate | Folded into |
| --- | --- |
| Model rights and license threshold | G29 (amended) |
| Prompt injection via IG text | G48 (amended) |
| Identity separation | G43 (amended) |
| Aggregation | G41 (amended) |
| Windows path-length portability | AF-03 (existing unit) |
| Healthcare AI data contracts | G44 (amended, research) |
| Benchmark label independence | G45 + `research/LABEL_AUTHORITY.md` |
| Stale status headers | G51 closure evidence (the generated status ledger in the migration proposal) |

## Non-G blockers tracked here for completeness

| Blocker | Owner | Required decision |
| --- | --- | --- |
| Issue #100 (historical bytes unavailable) | Spec 019 | FD-3: accept `HISTORICAL_ARTIFACT_BYTES_UNAVAILABLE` as a permanent, honest terminal state, with a durable forward mechanism (migration proposal §4). |
| Issue #15 (oracle pin) | CF-06 | FD-4 only after qualification. Upstream PR #2554 is still open. |
| Repository license | — | FD-6 |
