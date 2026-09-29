# V3.1 Authority Migration and Reconciliation

Status: SPEC_CANDIDATE
Canonical base: `d0015357fd569f25f161168e759037eaeff2f487`

## Identity

This Spec Kit is the V2 to V3.1 migration and reconciliation gate required by `docs/COMMAND_F_V3_1_IMPLEMENTATION_RUNBOOK.md` section 3.

It does not implement CF-17 through CF-28.
It does not renumber CF-01 through CF-16.
It does not close AF-01, AF-02, or issue #100.
It does not rewrite historical evidence.

Until this kit is itself canonical on `main` and its exit condition is proven, `COMMAND_F_MASTER_ARCHITECTURE_V2.md` plus the already-canonical Spec Kits `001` through `017` remain execution authority. Merging the V3.1 planning documents in PR #91 did not flip that authority.

## Live reconciliation captured for this candidate

Observed on 2026-09-29 after PR #91 merged. A later implementation pass must re-read live GitHub state. These identities are the input observation, not a permanent pin.

```text
canonical main: d0015357fd569f25f161168e759037eaeff2f487
parents: 90456dbcda782d95e83c20859ec6b24e3f4f4118 eb53d317dfe36c9395e64e00e7452b8bf0c25639
planning head merged: eb53d317dfe36c9395e64e00e7452b8bf0c25639
prior product main: 90456dbcda782d95e83c20859ec6b24e3f4f4118
```

Post-merge workflows on `d0015357fd569f25f161168e759037eaeff2f487`:

```text
af01-scorecard run 36633606511: SUCCESS
af01-assurance-proof run 36633606572: SUCCESS
ci / rust on push to main: NOT_RUN (workflow does not trigger on main)
```

Pre-merge exact-head `eb53d317dfe36c9395e64e00e7452b8bf0c25639` recorded SUCCESS for `rust` run `36633044827`, `assurance-proof` run `36633044738`, `scorecard` run `36633044765`, and `af02-base-verifier` run `36633043057`. Those results qualify the planning candidate. They are not post-merge `rust` evidence.

## History that must remain intact

- Spec Kits `001` through `017` keep their directory identities.
- CF-01 through CF-16 and AF-01 through AF-02 historical conclusions stay historical.
- Issue #99 remains closed only on the evidence recorded at closure: locked `rustls 0.23.45`, empty advisory ignore list, and assurance-proof cargo-deny/cargo-audit success on `90456dbcda782d95e83c20859ec6b24e3f4f4118`.
- Issue #100 stays open. PR #101 classified a structurally empty live Actions artifact collection as `HISTORICAL_UNAVAILABLE`. It did not create durable content-addressed artifact bytes or offline replay independent of live GitHub metadata.
- Issues #15, #35, #36, #37, #38, and #40 stay open until their underlying invariants are proven. Issue numbers are not closure evidence.
- Open pull requests that predate this kit are not absorbed, rebased, or force-updated by this migration.

V3 and V3.1 documents did not exist as execution authority before PR #91. This kit must not describe them as if they governed earlier CF or AF work.

## Governance reconciliation

Live rulesets at observation time:

- `commandF main assurance` (`21652953`): no bypass; required checks `rust`, `assurance-proof`, and `scorecard`; deletion and non-fast-forward blocked.
- `commandF main review governance` (`21652974`): merge commits only; one approval; Code Owner review; stale review dismissal; last-push approval; resolved threads; administrator bypass mode `pull_requests_only`.

The review-layer bypass exists for the sole-administrator self-approval deadlock documented in `specs/015-af-01-trusted-development-baseline/stack-c-governance-layering.md`. It does not authorize bypass of the assurance ruleset.

Independent review for later source-changing units:

- Jev is required where a zero-cost authorized execution path exists. On the #91 head, Jev was `BLOCKED_EXTERNAL_NO_ZERO_COST_AUTHORIZED_PATH` because the available TypeSafe path has monetary cost and spending it was not authorized. That block is not a PASS.
- Alibaba Open Code Review delegation mode is required for supported files. Markdown planning files are `unsupported_ext` and must be recorded as excluded, not as reviewed.
- CodeRabbit, Cubic, Qodo, Greptile, and similar hosted statuses are not qualification evidence.

## What becomes authoritative if this kit closes

If and only if the exit condition below is met, the following becomes the forward execution map:

1. Wave -1 integrity work remains blocking for any claim of production-trust readiness and for starting CF-17 implementation.
2. The next dependency-eligible implementation unit is the durable offline retained-authority successor for issue #100, followed by proof-based disposition of issues #35, #36, #37, #38, and #40.
3. CF-17 through CF-28 may be specified and implemented only after the Wave -1 unit that a slice actually depends on is closed or explicitly deferred by a later canonical Spec Kit.
4. Gap ownership stays the ownership already written in `docs/COMMAND_F_V3_EXECUTION_PLAYBOOK.md` (`G01` through `G40`) and `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` (`G41` through `G50`). This kit does not assign a second owner.
5. Deterministic Rust core authority remains separate from any optional advisory model layer. A model probability cannot create `PROVEN_COMPATIBLE` or erase `PROVEN_BREAKING`.

## Fail closed

This kit fails closed, and does not flip execution authority, when any of the following is true:

- canonical `main` is not descended from `d0015357fd569f25f161168e759037eaeff2f487` without an explicit later reconciliation;
- a required assurance check on the candidate is red or reused from an older head;
- an open integrity issue is marked closed without its own exit evidence;
- a public schema identity already used by `001` through `017` is reused for a new V3 contract;
- donor source is copied without the qualification record required by `docs/commandf-v3_1-internal-pattern-sources-2026-09-29.yaml`;
- Jev or OCR unavailability is recorded as PASS;
- CF-17 implementation is started inside this Spec Kit.

## Exit condition

This Spec Kit may close only when:

- its exact candidate is a normal merge onto canonical `main`;
- fresh post-merge `assurance-proof` and `scorecard` are green on the merge SHA;
- the diff is limited to this Spec Kit plus an index pointer, with no product behavior change;
- the document still matches live issue and ruleset state at merge time, or a same-PR amendment records the delta;
- no issue is closed by this merge.
