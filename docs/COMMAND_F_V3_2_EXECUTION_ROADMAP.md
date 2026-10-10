# commandF V3.2 — Execution Roadmap and Dependency DAG

Status: **PLANNING_CANDIDATE / NOT EXECUTION AUTHORITY**

Every phase below names whether it is permitted **under current V2 authority** or needs the V3.2 migration first (`COMMAND_F_V3_2_CANONICAL_MIGRATION_PROPOSAL.md`). No phase is authorized by this document. The proposed spec-directory sequences (051 and later) are reservations only. The next free sequence must be re-read at creation time.

## 1. Founder decisions required

| ID | Decision | Blocks | Options | Recommendation |
| --- | --- | --- | --- | --- |
| **FD-1** | Rule for when the AF-02 base gate's rejection may be overridden, and how the gate becomes amendable | P0, every dependency and workflow PR | (a) Keep the status quo: ad-hoc merges past a non-required failing check (as in #101). (b) A one-time founder-ratified bootstrap PR that adds an *amendment-record* admission path to the verifier, followed by making the gate required (FD-2). (c) Retire the gate. | **(b).** (a) is undocumented authority. (c) loses real protection. |
| **FD-2** | Add `af02-base-verifier` to the required checks of ruleset 21652953, after FD-1(b) lands | Durable governance | — | Yes, but only after the amendment path is proven, or every dependency update deadlocks. |
| **FD-3** | Record permanent historical-byte unavailability and decide issue #100 disposition **only after** a forward evidence mechanism has been independently implemented and its retrieval/replay/retention tested | Retained-authority clean-up | Keep #100 open and linked to the proposed design until that qualification | Do not close #100 merely because the historical bytes are irrecoverable or a future design has been written. |
| **FD-4** | CF-06 oracle repin target (6.10.4, 7.0.0, or a later release containing the #2554 fix) | Issue #15 | — | Defer. Close Dependabot #93. Repin only after upstream merges the fix and a full qualification runs. |
| **FD-5** | Rights position for the `FHIR/ig-registry` data (no LICENSE file) | CF-17 catalog, G54 namespace masks | Ask HL7/FHIR infrastructure maintainers; treat as facts and not copyrightable expression (legal opinion needed); or do not use | Ask the maintainers in writing first. |
| **FD-6** | commandF repository license | First release | Apache-2.0 (consistent with the founder's other repositories), MPL-2.0, or other | Founder's choice. Required before any release. |
| **FD-7** | Model-pack distribution and the LFM threshold position | D1 adoption | See the D1 plan §4 | Model-free core by default. Optional user-acquired pack only. |
| **FD-8** | Spending authorization for the Jev (TypeSafe) research comparator | D1 comparator J | — | Optional. If declined, record `BLOCKED_COST_NOT_AUTHORIZED`. |
| **FD-9** | Ratify the numeric thresholds in the D1 go/no-go and the bench falsification tolerance | D1-0, B-5 | — | Ratify before any held-out access. |
| **FD-10** | Re-prioritize V2 CF-14 (on-prem profiler) and CF-16 (mapping IR) after the `review` journey | Migration | — | Defer both and retain them. CF-15 stays, after CF-19. |

## 2. Phases

```text
          ┌────────── P0 Governance & truth repair ──────────┐
          │ G51, G49 audit, status ledger, PR triage, FD-1..6 │
          └───────┬────────────────────────┬──────────────────┘
                  │                        │
          P1 AF-03 minimal + v0.1.0     research lane R (parallel from P0):
          (G20, G52)                     CommandFBench B-1..B-5, D1-0/D1-1
                  │                        │
          P2 CF-18a Decision contracts (G41,G42,G02,G08,G33,G46,G50)
                  │
          P3 CF-19a Consumer Contract v1 + `review` (G04,G05,G44,G54)
                  │
          P4 CF-21a Receipt + replay + bundle + Studio S0 (G43,G19,G35,G29,G55)
                  │
          P5 Studio S1 · CF-17 catalog (if FD-5) · AF-04 baseline
                  │
          P6 CF-20a selective oracle escalation + independent oracle (G06,G47,G10)
                  │
          P7 CF-15 recipes · CF-22 TestGen · scale (G15,G18,G24)
                  │
          P8 D1 adoption Spec Kit (only if GO) · CF-24 seam (G48,G17)
                  │
          P9 Bench held-out execution B-6/B-7 → first evidence-backed claims
```

Edges: P2 depends on P0 (the G51 rule) and on P1 (a release pipeline for distributing the contracts). P3 depends on P2. P4 depends on P3. Research lane R needs only P0 for its rights decisions. B-6 needs P3. P8 needs D1-4 and FD-7. **No phase consumes evidence from a later phase.**

## 3. Phase detail

### P0 — Governance and truth repair

Permitted under V2 now, except where FD-1 is required.

| Grain (proposed seq.) | Output | Acceptance | Touches AF-02 authority? |
| --- | --- | --- | --- |
| 051 governance-amendment-path (Spec Kit, docs only) | The design of the amendment-record admission (migration proposal §3) | Signed exact-head CI; G51 planning-only; FD-1 not granted and not needed for planning docs | No (the spec is under `specs/051-*`) |
| 051 implementation (bootstrap PR) | Verifier change, regression tests, workflow unchanged | Migration proposal §3 steps 3–7; **specific exact-SHA founder exception must first be explicitly approved** | **Yes.** This is a proposed one-time bootstrap, not a currently authorized bypass. |
| 052 status-ledger (docs + generator) | A generated `docs/STATUS_LEDGER.json` and `.md` deriving each Spec Kit's status from tree evidence (merge SHAs, closeout files) and live issue state at a recorded time | The generator is deterministic. The ledger diff is reviewed. It reconciles the audit §5.3 contradictions **without editing historical files**, by superseding them in the ledger. | No |
| 053 equivalent-pattern audit (G49) | A record proving no remaining unverified-byte or unbounded-read consumer: a grep-based inventory plus tests | Each consumer path is listed with its verified-read evidence | No |
| PR triage | Per audit §6 | Each closure is commented with a reason. Nothing is merged mechanically. | Some PRs (after FD-1) |

### P1 — AF-03 minimal and first release

AF-03 planning is permitted now (audit §5.7). Implementation edits workflows, so it needs FD-1.

| Grain | Output | Acceptance |
| --- | --- | --- |
| 054 AF-03 Spec Kit | Plan: Windows/macOS/Linux CI, MSRV, release artifacts, SBOM, attestation | Consistent with the Assurance Program |
| AF-03 W1 | Cross-platform `cargo test` on `windows-2025` and `macos-15` (pinned images) | Green. The Windows `MAX_PATH` failure (audit §2) is fixed by using a short `CARGO_TARGET_DIR` for the nested verifier build or by enabling long paths, with a regression test. |
| AF-03 W2 | Release workflow: binaries, sha256, Sigstore bundle, GitHub attestation, Syft SBOM | Offline `cosign verify-blob` instructions verified on all three OSes |
| G52 | README regenerated from the status ledger; capability table | Every claim links to a test or spec |
| v0.1.0 | First tagged pre-release of the existing CLI, labeled "experimental" | FD-6 is decided. V2's "AF-03 must close before a **stable** release claim" is respected: v0.1.0 is not a stable claim. |

### P2 — CF-18a Decision contract foundation (needs migration)

Outputs:

- the schemas (`change-set`, `coverage-matrix`, `evidence`, `decision-envelope`, `policy`, `error`);
- the truth/action/completeness algebra and aggregation (architecture §6);
- the coverage matrix for R4 P0 artifact families, generated from the rules;
- explanation codes (G55 part 1);
- one shared canonical-JSON module, which consolidates the CF-17 emitters (G50).

Acceptance:

- the invalid-state matrix;
- property tests for the joins (associativity, commutativity, monotonicity, no `ALLOW` without all `ALLOW`);
- counterfactual and irrelevant-edit fixtures (F9);
- the coverage-matrix CI check, which fails on any new uncovered transition;
- no CLI behavior change for existing commands (golden tests).

### P3 — CF-19a Consumer Contract v1 + `commandf review` (needs migration)

Outputs:

- the `consumer-contract/v1` schema;
- `contract init` from a CapabilityStatement, ViewDefinitions, or FHIRPath lists;
- per-consumer witnesses;
- the `review` command with exit codes 0, 1, 2, 3, and 4;
- authenticity states (G54);
- R4B and R5 explicitly `UNSUPPORTED`.

Acceptance:

- round-trip tests;
- a witness test per dependency family;
- adversarial namespace fixtures;
- a `NO_PROTECTED_CONSUMERS` path test;
- the pilot v0.2 sessions begin.

### P4 — CF-21a Receipt, replay, and bundle; Studio S0 (needs migration)

Outputs:

- `decision-receipt/v1` with the `sid`/`did`/`rid` identities;
- `verify-receipt`;
- `bundle export` and `bundle import`, with admission ported from `medscale-pack`;
- `report.html` (S0);
- G29 rights fields.

Acceptance:

- a cross-OS replay test: a receipt produced on Linux is verified on Windows with identical `sid` and `did`;
- a receipt that omits a stage fails validation;
- the report renders from fixtures only.

### P5 to P9

As in the diagram. Each needs its own Spec Kit, and each states its user-visible surface (G52 rule). The CF-17 catalog (P5) proceeds only after FD-5. It reuses the spec 043/045 acquisition code and adds the coverage label `OFFICIAL_FEED_LIST_AT_COMMIT` or `QUERY_RESULT_AT_TIME`.

## 4. Spec Kit grain rules (SpecGrain / Diffcipline)

- One grain = one independently reviewable user-visible change, or one evidence record. The CF-17 pattern of one "machine bytes" grain per document type is **not** repeated: serialization ships with the type it serializes.
- Each grain states:
  - its user-visible surface, or a `LIBRARY_ONLY_UNTIL` record;
  - the gaps it advances;
  - the evidence it produces;
  - what it does not claim.
- Review stack per grain:
  - required checks;
  - `af02-base-verifier` (after FD-2);
  - Alibaba OpenCodeReview (`ocr`) for supported source files, with Markdown recorded as `unsupported_ext`, not as "reviewed";
  - Jev where a zero-cost authorized path exists, otherwise recorded as blocked and not as PASS;
  - zero mandatory human/Code Owner approvals under G53; for protected bootstrap, separate founder FD-1 exact-head authorization.

  AI self-review is never a gate.

## 5. What stays blocked regardless of this roadmap

- Issue #15 / CF-06 repin: upstream #2554 is open, and FD-4 is required.
- The CF-10 corpus (PR #11): depends on issue #15.
- Issue #100 historical bytes: unavailable, with no fabricated substitute. Keep the issue open until the forward mechanism is implemented, independently qualified, and FD-3 explicitly decides the truthful closure/disposition.
- Any quantitative performance claim: AF-04.
- Any accuracy, safety, or superiority claim: P9 and the bench falsification criteria.
- D1 in any distribution: FD-7, D1-4 GO.
