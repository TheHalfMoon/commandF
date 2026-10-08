# UX-01 — Bounded CLI Review Preview

Status: IMPLEMENTED_CANDIDATE_ON_PR / NOT_CANONICAL / NOT_COMPLETE_REVIEW
Base: `f82565cca917d119e1c774b2c470e2ac20e0d6dd`.
Parent authority: `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`; V3.2 PR #187 is a candidate, not an adopted execution plan.

## Problem

Current `commandf check` and `commandf impact` are independently implemented but users must run separate commands to inspect both the deterministic compatibility policy and the declared package-graph impact. Creating a facade can improve usability without inventing consumer-contract assurance, signed receipts, scientific measurement, or a learned compatibility verdict.

## Scope

Add **`commandf review-preview`**, clearly distinguished from the future fully qualified `commandf review` product. Read exact before/after lockfiles and cached FHIR package archives, then compose the existing deterministic structural compatibility check and existing package-graph impact report into one bounded JSON output. No network calls are required when lock/cache inputs exist.

Input: package name, before/after locks, before/after caches, optional `--direction`, `--fail-on`, and `--output`. Requires lockfile schema 2 on both sides because the existing impact implementation requires it.

Output: JSON containing `schema: 1`, `scope: structural-and-declared-graph-preview`, `complete_consumer_contract_review: false`, `atomic_cross_step_snapshot: false`, `signed_receipt: false`, and the existing typed `check` and `impact` JSON reports. Neither component is a new semantic authority.

Exit codes: 0 = existing policy passes and both subreports are complete; 2 = existing policy fails and both subreports are complete; 1 = invalid CLI use, missing/unsupported input, malformed input, missing digest-verified cache object, impact failure, or failed publication. Never publish a partial preview on failure.

## Non-claims

No complete consumer-contract checking, graph-wide proof of safety, oracle escalation, terminology completeness, cryptographic signature, replayable proof receipt, privacy certification, D1/LLM input, or production release. The two internal reads are not a frozen atomic cross-step snapshot. A passed threshold does not mean general FHIR compatibility. Preview output is not a clinical safety decision.

## Security and reproducibility

- Reuse existing cached-archive verification; do not deserialize arbitrary untrusted JSON fragments into authority.
- Compose only JSON produced by the current typed internal library; no user-supplied raw JSON insertion.
- Cap each serialized part at 64 MiB; detect sum overflow before allocating output bytes.
- Publish only after both reports have successfully completed; existing atomic replacement semantics apply for `--output`.
- No new dependency, donor import, external model, paid service, workflow, protected AF-02 file, ruleset, or pinned oracle change.
- Reject v1 locks via the existing impact contract rather than silently skipping impact.
- Retain error/exit distinctions and report absence as evidence of operational failure, not PASS.

## Exit conditions

Four focused integration tests pass for: (1) nonempty structural policy failure, (2) permissive policy retaining findings, (3) v1 lock operational failure without partial output, (4) invalid policy usage exit 1. All existing `check` behavior continues to pass. Windows plus real exact-head CI are necessary but not sufficient; applicable human approval and normal merge commit are required before canonical adoption. This candidate may be rejected or replaced when full V3.2 `commandf review` is independently implemented.
