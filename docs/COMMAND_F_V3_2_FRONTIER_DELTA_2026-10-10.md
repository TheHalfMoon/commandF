# CommandF V3.2 — Canonical Frontier Delta (2026-10-10)

Status: **PLANNING_CANDIDATE / CURRENT-STATE RECONCILIATION / NOT EXECUTION AUTHORITY**

This is a dated reconciliation of the original V3.2 **2026-10-08 audit**.
The earlier audit's main SHA, rule settings and observations remain
historically accurate for their timestamp; they must **not** be used
as claims of current GitHub state. Canonical main at preparation of
this signed candidate: `8afa0ca8c0b94f2aa21ce7fed38fa9f94be66a54`.
Before any action re-read live main/rulesets/CI.

## Actual integrated frontier, not proposed work

- Founder change G53 was incorporated through PR #223 and rule
  21652974: **zero required human approvals**, no mandatory Code
  Owner or last-push reviews, pull request + merge-only. The
  independently enforced ruleset 21652953 still requires strict
  `rust`, `assurance-proof` and `scorecard`, and blocks branch
  deletion and non-fast-forward edits. Historical AF-02 v2
  approval=1 authority remains frozen.
- Actual bounded production/test improvements have been normally
  merged in PRs #206-#213, #217-#218 and #220; never infer the
  universe of promised capabilities from these increments.
- PR #211 shipped an intentionally **partial** `review-preview`
  command that composes deterministic check + graph-impact
  JSON/SARIF. It is **not** a complete multi-consumer `review`
  command, atomic execution receipt, Studio, or clinical proof.
- PR #224 shipped a signed, honest CLI README. G52's stale
  onboarding dimension improved, while the complete CF-17 CLI
  observatory surface, a stable supported release, publisher
  source authentication, and full consumer contracts remain
  unqualified or unimplemented.
- G51's separate signed five-file amendment-path Spec Kit was
  merged in PR #225. It is **planning only**, neither executable
  verifier amendments nor a founder FD-1/FD-2 decision. Protected
  AF-02 edits, including issue #215, remain independently gated.

## Reconciliation: proposed V3.2 numbering

The original 2026-10-08 V3.2 planning draft used `G53` for
explanation templates and accessibility. The canonical repository
subsequently assigned `G53` to the implemented review-free governance
policy. The **unimplemented accessibility/explanation candidate**
is hereby renamed **G55** across the proposed V3.2 plan.
G51, G52 and G54 are retained as their pre-existing proposal
identifiers. G55 is only a proposed new gap, not adopted authority.
Future activation requires an explicit, separate conflict-free
Spec Kit and plan migration; editing this document alone
cannot allocate runtime semantics.

## Active blockers and non-claims

- #100: durable AF-02 authority evidence migration remains open;
  an expiring Actions artifact is not a replacement for base
  provenance and no self-authorizing change is permitted.
- #215: macOS AF-02 verifier Clippy failure is in a protected path,
  requiring a separately scoped G51/FD-1 exact-head decision.
- #214: two independently retrieved official FHIR registry
  archives may have different raw SHA-256 identities despite
  matching apparent package identity. Preserve both source
  digests and lockfiles; do not normalize or forge equivalence.
- #221: live registry DNS acquisition intermittently fails in
  GitHub CI. A single documented retry succeeding is NOT proof
  of a permanently resolved network problem.
- #15: upstream HL7 differential-oracle core source qualification
  remains open; no pin migration is implied.
- No claim of stable release, root redistribution license,
  FHIR R4B/R5/R6 general support, certified patient safety,
  D1 model sufficiency, complete public registry catalog, or
  full multi-consumer review applies to the present code.

## Authority and admission contract

`docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md` and adopted canonical
Spec Kits remain the execution authority. These V3.2 documents
are a **candidate** only; no FD-1, FD-2 or later founder
decision is granted. Future source/protected changes must use
signed+DCO normal merge commits, exact-head required CI,
original AF-02 base evidence and explicitly scoped founder
authorization when the protected gate demands it. Optional
Alibaba OCR/Jev/human reviews must be reported as PASS only
if truly performed; non-execution or an exhausted commercial
review quota must be labeled NOT_RUN/NEUTRAL, not an approval.
Nothing in this planning package permits removing technical
checks, direct pushes, an undocumented bypass, force-push,
or rewriting frozen evidence.
