# G51 — Governance Amendment Path: Implementation Plan Candidate

Status: PLANNING_CANDIDATE / DOES_NOT_AUTHORIZE_PROTECTED_EDITS

## 1. Existing execution boundary

At the observed `main` SHA, the current `.github/workflows/af02-base-verifier.yml` checks out a canonical base and a candidate as data; it builds the verifier exclusively from the base. The base-gate authority path set includes existing `Cargo.*`, workflows, donor and verifier files. The verifier does not execute newly added unreferenced modules from the candidate. This is the intended anti-self-modification boundary.

Current GitHub ruleset `21652974` (founder G53, 2026-10-10) requires a pull request and **merge commits only**, with **zero human/Code Owner approvals** and no mandatory review-thread/last-push gates. Independent assurance ruleset `21652953` still requires exact-head `rust`, `assurance-proof` and `scorecard`, and forbids deletion/non-fast-forward. `af02-base-verifier` is not currently in the ruleset-required context list; it must not be bypassed or silently ignored if it reports a substantive failure. Read both live rulesets again immediately before any subsequent mutation.

## 2. Proposed protocol

### Phase A: planning and review

- Merge this planning record only with signed+DCO commits, successful exact-head required and applicable checks, a verified canonical base and an ordinary expected-head merge. Human approval is not required by current G53 review rules. Optional tool/human reviews must be recorded as actual outcomes, not invented.
- Review `specs/049`, `specs/050`, issue #100, and the live provenance/waiver logic before implementing.
- Decide whether the two-PR amendment model is proportionate for all protected paths. A future *explicit* supply-chain-policy carve-out for `Cargo.lock` can be studied separately; it is **not** part of this protocol.

### Phase B: one-time bootstrap, not yet approved

- Create an implementation PR limited to base-gate Rust implementation, its module registration, machine schema and regression tests, plus strictly necessary new governed paths. The exact changed-file inventory must be known first.
- Build the exact PR candidate and retain its base SHA, head SHA, old/new blob OIDs, CI run IDs, signer identity, explicit founder FD-1 evidence, and any recorded defects. Record optional independent review identities only if reviews truly occurred.
- **Then**, not before, obtain an explicit founder decision naming these exact identities. A generic `go ahead` is not a permitted bypass.
- If the existing `af02-base-verifier` rejects the bootstrap as expected, capture the real failure. All other required checks must satisfy active repository policy; no human review is mandatory under G53. An administrator may consider a *single exact-head founder-authorized* exception only if the platform actually permits it and the independent admission/assurance evidence is retained. This document grants no exception. Never disable a workflow, fake a result, or alter the assurance ruleset.
- The bootstrap itself **cannot** be admitted by the amendment mechanism it introduces.

### Phase C: base-recorded amendment

- For each later protected change, prepare the candidate **content bytes** outside the protected path on `main`, compute Git blob OIDs, and prepare an amendment PR adding a new numbered `governance/amendments/AMD-XXXX.json` file under a newly frozen schema.
- The record binds the exact preimage and postimage blob OIDs for every protected path, the planned affected scope, reason, protected test set, ticket, expiry or revocation conditions, and GitHub review/founder-decision references.
- Review and merge the amendment PR. The admission of this record itself is handled under the currently approved base verifier and normal branch rules. New records must be unique and bounded; existing records are immutable.
- Create the follow-up authority-change PR *after* the amendment is part of its base. The candidate may not introduce or edit an admission record to authorize its own change.
- The new base verifier compares changed protected paths to the canonical base record, verifies exact old/new OIDs, distinguishes absent from deleted, and refuses any undocumented extra path, reused or stale record.
- The follow-up change gets its own independently qualified exact-head CI and separate signed/founder authorization proof where required. Admission of the prior amendment record does **not** waive any second-PR technical or exact-identity gate; optional independent review is not a mandatory approval.
- After merge, an amendment with a nonmatching preimage is consumed; it must never authorize a later unrelated edit.

### Phase D: mandatory status and recovery

- Use disposable test PRs to demonstrate ordinary docs passes, exact authorized Cargo update passes, and unauthorized Cargo, workflows, verifier and donor edits fail. Record real check-run IDs.
- Verify positive and negative checks with the base-built implementation on a fresh PR. Then FD-2 can authorize a separate ruleset update adding `af02-base-verifier` as a required status.
- Read the ruleset back and run one new positive and one new negative PR. Do not claim closure until both behave as expected.
- On an incorrect bootstrap or partial rollout, stop all protected merges. Use reviewed forward corrective commits and explicit rollback/amendment governance. Never force-push, rewrite history, forge passing statuses, or add standing bypass actors.

## 3. Data contract proposal

An amendment record must contain:

- `schema_id` and `schema_version`;
- unique `amendment_id`, creation timestamp (metadata only, not proof), and purpose;
- canonical repository identity and scoped protected-path list;
- `path`, `before_blob_oid` or `ABSENT`, `after_blob_oid` or `DELETED`, file mode/type disposition;
- base-object applicability and one-time-consumption semantics;
- separate GitHub PR URI, immutable qualified head, actual signed commit identities, exact CI/check run evidence, and reviewer identities **only where independent review actually occurred**;
- a specific founder-decision reference when required;
- expected required status names and evidence policies;
- revocation/supersession reference where applicable.

A candidate cannot validate its own authority merely by supplying strings that look like a GitHub review or status. Freeze a schema only after the trust model specifies which fields are checked against canonical base Git objects and which require live authenticated metadata. Differentiate `METADATA_UNAVAILABLE` from `DISAPPROVED`; neither is PASS.

Do not bind future branch *commit SHA* prematurely to a yet-uncreated commit. Precompute target **blob OIDs** for approval; record exact candidate commit SHA and run evidence at the subsequent PR gate. The one-time bootstrap, unlike recurring amendments, must have an exact known head SHA before the founder can approve it.

## 4. Implementation seam

Prefer an isolated `amendment` module called by the already base-built `base_gate`. Review the actual verifier's bounded file walking, normalizer and Git porcelain behavior before editing it. Do not duplicate an entire verifier or run candidate code. Emit structured rejection codes that cannot be confused with successful qualification.

No new Rust dependency or workflow may be adopted solely because it is convenient; preserve the workspace's source/lock policy and existing process isolation.

## 5. Non-goals and escalation

- No issue #100 closure or recreation of original CF-10 artifact bytes.
- No CF-06 oracle repin through a dependency amendment.
- No general authority to bypass an AF/CF production gate.
- No claim that a green amendment check makes arbitrary new code clinically safe.

If a necessary platform permission, GitHub data source, founder authorization for a protected exception, or branch-rule update cannot be executed without its separate founder/platform gate, report `BLOCKED` and stop that mutation only.
