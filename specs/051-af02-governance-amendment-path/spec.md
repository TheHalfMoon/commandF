# G51 — AF-02 Governance Amendment Path

Status: PLANNING_CANDIDATE / NOT_EXECUTION_AUTHORITY

G53 live-policy delta (2026-10-10): GitHub rule 21652974 now demands
**zero mandatory human approvals**, a pull request and normal merge
commit. Rule 21652953 independently enforces strict rust,
assurance-proof and scorecard plus branch deletion/force-push
protection. This current-state record does not overwrite the
historical AF-02 v2 one-approval snapshot. See
specs/053-af02-review-free-live-policy/ for exact live-policy proof.
The earlier observed main below remains historical provenance.

Observed canonical main: `f82565cca917d119e1c774b2c470e2ac20e0d6dd`
Related: proposed V3.2 PR #187; issue #100 remains open.

## Problem

The canonical-base AF-02 verifier rejects edits to its existing authority set, including Cargo manifests/lockfiles, workflows, donor records, and its own verifier sources. These edits include legitimate dependency security fixes. The check is currently not a repository-ruleset-required status; some protected changes have entered through a different merge-gate path, leaving an undocumented trust exception.

The known absence-of-executable-precursor problem is recorded in `specs/050-af02-precanonical-path-gap/`. A candidate cannot change the verifier, compile a future verifier module, and then use its own changed code to approve itself.

## Intent

Introduce a legitimate, narrow, reviewable way for the existing **canonical-base** verifier to evaluate authorized changes to authority paths, preserving current fail-closed tests and exact-head review. Use a one-time founder-approved bootstrap only after its candidate exact SHA and paths are known, then require the base-verifier status as an enforceable check **only after** it can correctly admit qualified maintenance.

This Spec Kit is a design proposal and test contract only. The AF-02 authority remains unchanged.

## Scope in

- The legitimate bootstrap authority and its exact-SHA approval protocol.
- Base-controlled amendment records committed **before** the authority-changing PR.
- Precise admission, replay prevention, threat boundaries, rights, and failure states.
- Distinct treatment of ordinary Cargo/dependency changes versus validator/oracle production-pin changes.
- Exact-head, base-built and post-merge verification; ruleset change ordering.
- Recovery from a failed or revoked amendment without history rewriting.

## Scope out

- Immediate modification of `.github/`, `Cargo.toml`, `Cargo.lock`, `donors/`, `tools/af02-verifier/`, or `specs/016-af-02-adversarial-test-strength/`.
- Any change to CF-06 production oracle identity, CF-10 historical interpretations, or issue #100 evidence.
- An administrator standing bypass or automatic approval of any future changed file.
- Activation of AF-02 A0, AF-03 implementation, AF-04, CF-18, or V3.2 execution authority.

## Binding architectural constraints (for future implementation design)

1. **Base-owned decision logic.** The verifier launched on an authority-changing PR must be built exclusively from the protected base SHA, never the candidate head.
2. **Admission record ordering.** An amendment record is reviewed and merged as a distinct previous PR, making it present in the later authority-changing PR's **base** tree. Candidate-only records have no authority.
3. **Exact file-level binding.** Each record enumerates normalized repository paths, original base Git blob OID or `ABSENT`, and proposed target blob OID or `DELETED`. These are Git blob object IDs, not SHA-256 digests of file bytes; do not conflate the two.
4. **Non-self-authorization.** Founder FD-1 and earlier amendment provenance, where required, must resolve to the actual GitHub evidence and exact qualified head; no candidate-supplied text or self-issued claim may grant protected-path authority. This is distinct from the current **zero-human-review** branch rule.
5. **No implicit broadening.** Any changed authority path not covered by a qualified live amendment fails. Deletes, renames, symlinks, submodules, type changes, path collisions and non-canonical paths are distinct, bounded and rejected unless an explicitly reviewed class is implemented.
6. **Single-use / stale-base refusal.** Existing base blob identities must match the amendment's preimage; stale, conflicting, copied, already-consumed, or ambiguous records fail closed.
7. **Truth separation.** A green Rust test, cargo-audit, or cargo-deny is not itself authorization to amend protected paths, and an optional reviewer opinion is not proof that code is secure. Required exact-head assurance checks remain mandatory.
8. **Protected repository rulesets.** The later check becomes required only after demonstrated positive and negative tests prove legitimate fixes no longer deadlock. No bypass actor is added to assurance rules.
9. **No fabricated historical data.** This design does not change AF-02 retained-evidence reconstruction, which remains governed by issue #100 and separate durable-evidence work.
10. **Governance precedence.** Existing V2/AF execution authority and live branch protections win over this candidate until a separately approved migration gate promotes it.

## Acceptance (planning, not implementation)

- Spec, plan, tasks, threat model and consistency ledger agree on the same non-circular two-PR mechanism and the one-time bootstrap boundary.
- No protected file or workflow appears in this PR's diff.
- A negative-test matrix covers tampering, stale records, candidate-only data, unexpected changed paths, and approval spoofing.
- Genuine exact-head required checks, signed+DCO commits, and applicable founder authorization are recorded. Independent human/agent reviews are optional under G53; when not executed they are recorded as NOT_RUN, never fabricated as PASS.
- Normal merge only after repository policy allows it. Merging this **planning** Spec Kit does not grant founder FD-1 or implementation authority.

## Explicit governance decision

**FD-1 is not granted here.** Once a future exact bootstrap PR is implemented and technically qualified against its specific candidate/base, the founder can decide whether to authorize **that specific head** with exact base SHA, head SHA, all before/after blob OIDs, and explicit scope. Optional review evidence may supplement but is not mandatory; if the head changes, the FD-1 authorization is invalid.

**FD-2 is not granted here.** The ruleset change making `af02-base-verifier` required is separately qualified after its new base implementation is proven.
