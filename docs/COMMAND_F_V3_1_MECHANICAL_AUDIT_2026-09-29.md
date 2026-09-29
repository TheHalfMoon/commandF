# commandF V3.1 — Mechanical Planning Audit — 2026-09-29

Status: **PLANNING_EVIDENCE / NO_KNOWN_UNOWNED_PLANNING_GAPS**

This document records a bounded mechanical/structural audit of the V3 + V3.1 planning package. It is not implementation evidence, does not close any product gap, does not supersede V2, and does not authorize a merge or product implementation by itself.

## 1. Audit identity and boundary

Audited repository and planning state:

```text
repository: TheHalfMoon/commandF
canonical base observed for PR #91: 18819c7fdaee618f4aa86c6c3279b1acb70ec9f2
forward merge of later canonical main: 90456dbcda782d95e83c20859ec6b24e3f4f4118
merge commit: 3fc0878bdd38357052401071360128d0478e3a8b
planning branch: docs/commandf-v3-plan-research
pre-audit planning head: e18ee1a555ea03be35fc88fac1e31361a5860bbb
planning PR: #91
```

The audit checks planning structure and explicit ownership. It does not claim that roadmap capabilities are implemented, that all future unknowns have been discovered, or that exact-head CI/review is green.

The correct completeness claim is:

> No material planning concern identified by this review remains knowingly unowned or without an explicit closure path.

If a later review discovers a new material concern not covered by an existing gap/owner/closure contract, it MUST receive a new gap identity or an explicit amendment before the affected implementation proceeds.

## 2. Changed-path scope

Before adding this audit file, PR #91 changed exactly the following planning/provenance paths:

```text
docs/COMMAND_F_DISCOVERY_COVERAGE_2026-08-13.md
docs/COMMAND_F_MASTER_ARCHITECTURE_V3_CANDIDATE.md
docs/COMMAND_F_OPEN_SOURCE_QUALIFICATION_2026-09-12.md
docs/COMMAND_F_PLAN_GAP_REVIEW_2026-09-12.md
docs/COMMAND_F_PLAN_INDEX.md
docs/COMMAND_F_V3_1_COMPLETENESS_AUDIT_2026-09-29.md
docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md
docs/COMMAND_F_V3_1_IMPLEMENTATION_RUNBOOK.md
docs/COMMAND_F_V3_1_REVIEW_GOVERNANCE.md
docs/COMMAND_F_V3_EXECUTION_PLAYBOOK.md
docs/commandf-v3_1-internal-pattern-sources-2026-09-29.yaml
```

This audit adds one further `docs/**` file. No product source, Cargo manifest/lockfile, workflow, runtime dependency, schema implementation, rule implementation, test corpus, or active Spec Kit is changed by this planning package.

## 3. Gap continuity audit

### V3 base gaps

`COMMAND_F_V3_EXECUTION_PLAYBOOK.md` contains one closure-contract row for every gap from `G01` through `G40` inclusive.

For each row the playbook states:

- gap identity;
- owning slice(s) or explicit shared/cross-cutting owner;
- executable closure contract.

Observed result:

```text
G01..G40 expected: 40
G01..G40 present:  40
missing IDs:        none identified
unowned rows:       none identified
rows without closure contract: none identified
```

The playbook also explicitly states that PLANNED is not CLOSED and that executable evidence is required for closure.

### V3.1 gaps

`COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` contains `G41` through `G50` inclusive.

Observed result:

```text
G41..G50 expected: 10
G41..G50 present:  10
missing IDs:        none identified
unowned rows:       none identified
rows without closure contract: none identified
```

The document explicitly states that none of G41-G50 is closed because the planning document exists.

### Combined ledger

```text
V3:       G01..G40 = 40
V3.1:     G41..G50 = 10
combined: G01..G50 = 50 explicit planning gaps
```

No duplicate gap identity or intentional hole was identified in the reviewed tables.

## 4. Build-card continuity audit

The V3 execution playbook contains sequential build cards for:

```text
CF-17 Ecosystem Observatory
CF-18 Compatibility Coverage Model
CF-19 Consumer Contract Scanner
CF-20 Compatibility Lab
CF-21 Interoperability BOM and Reproducible Build Evidence
CF-22 Deterministic TestGen
CF-23 Developer Platform
CF-24 Capability-Scoped Plugin and Policy SDK
CF-25 Terminology Gap and Federation Intelligence
CF-26 Transformation Evidence and Certificates
CF-27 Public Compatibility Observatory
CF-28 Bench and Research Convergence
```

Observed result:

```text
expected cards: CF-17..CF-28 = 12
present cards:  12
missing card identity: none identified
```

Each card has goal/must-ship/non-goal/work-package/exit-evidence structure. V3.1 amends these existing owners rather than inventing a second parallel roadmap.

## 5. Legacy problem preservation audit

The V3 playbook retains the 35 legacy interoperability hypotheses from `COMMAND_F_GAP_LEDGER_2026-08-13.md` through an explicit crosswalk.

The crosswalk permits a problem to be implemented, retained as bounded research, or explicitly bounded/non-goal. It does not permit silent disappearance.

Observed result:

```text
legacy hypotheses expected: 35
legacy hypotheses crosswalked: 35
silent-drop mechanism allowed: no
```

## 6. Decision-assurance contract audit

V3.1 assigns explicit ownership for the previously ambiguous decision layer.

Required contract families are now named and owned:

```text
Decision Envelope v1        -> CF-18 / CF-23 exposure
Decision Policy v1          -> CF-18
Consumer Contract v1        -> CF-19 / CF-23 exposure
Selective oracle escalation -> CF-20
Decision Receipt v1         -> CF-21 / CF-23 exposure
Counterexample/TestGen      -> CF-22
Model/plugin authority      -> CF-24
Terminology evidence        -> CF-25
Transformation evidence     -> CF-26
Public evidence view        -> CF-27
CommandFBench-Decision      -> CF-28
```

The reviewed plan explicitly separates:

- proof from probability/confidence;
- truth class from recommended action;
- decision truth from evidence completeness;
- deterministic evidence from model advisory output;
- completed execution from verified execution;
- missing/unsupported/unavailable evidence from compatibility.

No optional model/plugin path is allowed to self-promote into `PROVEN_*`, silently suppress findings, weaken policy, or grant privileged authority.

## 7. Failure/unknown/abstention audit

The reviewed V3.1 contracts make these outcomes explicit rather than treating them as exceptional prose:

```text
CONFLICTING_EVIDENCE
INSUFFICIENT_EVIDENCE
UNSUPPORTED
INDETERMINATE
ABSTAIN
REQUEST_ORACLE
REQUEST_RUNTIME_PROBE
REQUEST_HUMAN
```

Required oracle/probe states include required-not-run, unavailable, failed, timed-out, uncomparable, disagreement, and explicit execution-state transitions. A required missing stage cannot disappear from a receipt or become PASS by absence.

## 8. Consumer-scope audit

Consumer protection is no longer inferred from ecosystem popularity or graph centrality. The plan requires a portable, versioned Consumer Contract with exact protected consumer/version sets and source provenance.

The contract family covers the initial bounded interoperability dependencies needed by the roadmap, including:

- FHIR package/profile/extension and canonical dependencies;
- element/path usage;
- terminology system/version/binding;
- SearchParameter/search interaction;
- FHIRPath;
- CQL/ELM/Library;
- SQL-on-FHIR ViewDefinition;
- CapabilityStatement expectations;
- REST operations/interactions;
- SMART scopes/backend-service expectations;
- Bulk Data;
- subscriptions/eventing;
- executable conformance/test references.

A missing requested consumer contract is explicitly insufficient evidence or another fail-closed policy result, never implicit safety.

## 9. Protocol-scope audit

The V3 plan has an explicit maturity order for protocol compatibility:

1. FHIR REST interactions/operations;
2. Search/SearchParameter;
3. SMART App Launch/backend services;
4. Bulk Data;
5. Subscriptions/eventing;
6. bounded FHIRcast/CDS Hooks where a protected consumer/IG declares them;
7. GraphQL-on-FHIR retained as secondary/evolving unless evidence reprioritizes it.

Protocol support is therefore not silently reduced to resource/profile diffs.

## 10. Donor/provenance consistency audit

The pinned V3.1 founder-controlled pattern record contains five sources:

```text
GAX      a1432dc81d48407c551b1b8ddefefa478a2cd33f
Ascout   06b763d8ffb09538b812fc0b007424b482531256
Sentrdel f5747319a50831ef7cee983d253c0ca5503c9a64
Kernux   7e07035d8a29b234273fa84a496539cc685ad03e
Cotra    0b56d8b95f69f5797fb8edf7871ec84b9013321c
```

Each is classified `PATTERN_ONLY` at this planning stage. The runbook donor application map was reconciled on pre-audit head `e18ee1a555ea03be35fc88fac1e31361a5860bbb` so the same five sources are visible from the implementation entry point.

No donor pin authorizes runtime dependency adoption, source copying, redistribution, or correctness authority. Slice-specific adoption still requires exact source path/blob, license/notice evidence, third-party/data/model/content-rights disposition, destination/owner, tests, security review, and removal/upgrade path.

## 11. Internal-source sweep disposition

The planning review also inspected directly relevant founder repositories beyond the five selected pattern donors, including ProtocolWISE, commandMed, and MESC/MedScale.

Their useful verification, abstention, calibrated-decision, validator-grounding, provenance, and reproducibility principles were already represented by the chosen source set plus existing commandF V3 semantics. They were therefore not added merely to increase donor count.

Sentrdel was retained because it contributed a distinct useful pattern: explicit evidence-authority classes, a single canonical reconciliation boundary, contradiction preservation, and the rule that missing/failed/unsupported/unavailable coverage cannot become a clean result by absence.

## 12. Shared assurance/integrity audit

The runbook contains one reusable primitive backlog rather than allowing every future slice to invent a different version of the same trust invariant:

```text
bounded_read
verified_read
canonical_relative_path
canonical_json
schema_version_validation
evidence_digest
reason_code/status vocabulary
external_process_bounds
offline_bundle_manifest
redaction/truncation facts
exact source/comparison identity
```

Each primitive requires negative tests. A slice may strengthen a primitive, but duplicating incompatible semantics requires explicit rationale and compatibility evidence.

## 13. Known repository blockers are not hidden

The plan explicitly retains existing evidence/input-boundary issue families as migration blockers where still applicable:

```text
#35 verified terminology-cache bytes
#36 impact/oracle verified-byte staging
#37 bounded lockfile input
#38 bounded local-mirror archive allocation
#40 CLI root-package verified-byte consumption
```

A new dependency assurance failure discovered while qualifying the planning branch is independently owned by:

```text
#99 fix(deps): remediate RUSTSEC-2026-0285 in transitive rustls
```

The observed failing chain was `rustls 0.23.43 -> ureq 3.4.0 -> commandf-pkg -> commandf`, and the advisory requires a patched rustls version (`>=0.23.45`). The planning PR does not waive, suppress, or smuggle this dependency repair into docs-only scope.

These blockers do not make the planning architecture structurally incomplete; they block the relevant migration/merge/trust claim until their own exact-head evidence is green.

## 14. CI/review honesty

On planning head `47d9fe9d2ef738833fc75ce3f5ac4a3b1b47566d`, AF-01 assurance counterexample/checksum/summary tests passed before `cargo-deny` correctly failed on `RUSTSEC-2026-0285`. That failure is retained as a real assurance result, not converted into PASS.

Subsequent documentation commits change the exact planning head, so that earlier run is evidence of the repository dependency blocker, not final-head merge qualification. Final-head required CI/review must be re-evaluated and stale green/red status must not be reused as exact-head qualification.

Future V3/V3.1 review policy intends Jev plus Alibaba Open Code Review as independent review inputs, with human/protected merge authority. Current live canonical reviewer/ruleset authority remains controlling until a qualified migration gate adopts that topology. CodeRabbit/Qodo/Cubic status is not V3.1 qualification evidence.

## 15. Implementation-readiness verdict

For planning structure:

```text
G01..G50 ownership: COMPLETE_FOR_REVIEWED_PLAN
G01..G50 closure contracts: COMPLETE_FOR_REVIEWED_PLAN
CF-17..CF-28 build-card continuity: COMPLETE_FOR_REVIEWED_PLAN
legacy 35-gap preservation: COMPLETE_FOR_REVIEWED_PLAN
donor map/provenance planning: COMPLETE_FOR_REVIEWED_PLAN
decision/unknown/abstention semantics: EXPLICIT
consumer/protocol scope: EXPLICIT
integrity/security blockers: EXPLICIT_AND_SEPARATELY_OWNED
known unowned material planning concerns: NONE IDENTIFIED
```

The correct operational status is:

```text
NO_KNOWN_UNOWNED_PLANNING_GAPS
READY_FOR_SPEC_KIT_SHAPING
NOT_CURRENT_EXECUTION_AUTHORITY
NOT_IMPLEMENTED
NOT_MERGE_QUALIFIED_BY_THIS_AUDIT
```

A future implementation unit is `READY_FOR_IMPLEMENTATION` only after its own Spec Kit answers every applicable no-gap checklist item, its dependencies are canonical, its source/donor gates are satisfied, and its live exact-head review/CI requirements are met.

## 16. Anti-completeness-fabrication rule

No static plan can prove that no future unknown will ever be discovered.

Therefore commandF must preserve this invariant throughout implementation:

> A newly discovered material concern may be mapped to an existing gap only when its existing owner and closure contract genuinely cover it; otherwise create a new gap ID before implementation continues. Never hide a newly discovered gap in commentary, implementation detail, or an unrelated closure claim.

This rule is part of the plan's readiness, not a disclaimer from doing the work.
