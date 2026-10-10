# commandF Engineering Rules

commandF is interoperability infrastructure. Review and implementation must preserve evidence, determinism, and explicit semantics.

## Non-negotiable rules

- The V1 critical path is FHIR conformance change intelligence; do not make future semantic layers dependencies of shipped commands.
- Never silently discard, coerce, or invent known source information.
- AI may propose mappings, fixes, or findings; deterministic validators, tests, and policies provide authoritative evidence.
- Keep package identity, exact version, provenance, and content digests explicit.
- Mutable tags or floating references are insufficient for reproducible production evidence.
- Authoritative validators are oracles; do not rewrite them merely to remove JVM/runtime dependencies.
- Fail closed when a required compatibility state cannot be classified.
- Every public rule requires rationale, positive tests, negative/counterexample tests, and deterministic output.
- Avoid panics in library code for externally supplied data.
- Research hypotheses belong under `research/` and are not product guarantees.
- No new crate unless a shipped command or immediate executable test uses it.

## Review priorities

1. Silent compatibility or information-loss behavior.
2. Incorrect breaking-change claims or false-positive risk.
3. Provenance/version ambiguity.
4. Non-deterministic package resolution or output ordering.
5. Unsafe archive, cache, registry, or supply-chain behavior.
6. Missing failure-path and conflict tests.
7. API changes that make later CF slices harder to compose.

## Change discipline

Keep changes small and stackable. Each PR must have a bounded, testable
scope and independently reproducible evidence. Merge only after the
**current exact-head required CI checks are successful**, no known
substantive technical failures remain, and the signed/DCO candidate
qualifies under active GitHub rules. Use only a normal merge commit
with an expected-head guard; never use rebase, squash, direct push,
force-push, or bypass.

**Founder governance decision effective 2026-10-10:** CommandF requires
**zero human/Code Owner review approvals**. GitHub review ruleset
21652974 retains the pull-request and merge-commit requirement, but
no longer requires human approvals, last-push approval, stale-review
dismissal, or review-thread resolution. The separate assurance ruleset
21652953 still enforces strict rust, assurance-proof, and scorecard
checks, along with deletion and non-fast-forward protection.
Alibaba Open Code Review and Jev are independent engineering inputs
when truly executable; unavailable providers are NOT_RUN, never
fabricated as PASS. Historical AF-01 and AF-02 v2 one-approval records
describe the past and must not be treated as the live review policy.
See specs/053-af02-review-free-live-policy/ for pinned live rule
verification and immutable historical authority separation.

<!-- graft:start -->
## Graft — optional local repository context

When installed and current, prefer Graft as a **local, read-only
navigation aid** for source exploration; it never supplies merge,
clinical, source authenticity, or policy approval evidence.
Verified on the authorized Mac: `graft --version` = `0.21.1`
(2026-10-10). A generated graph is **not** included in Git and
must not be assumed fresh or present on every workstation.

- Check `graft --version` and `graft check` for the active checkout
  before treating indexed references as current. When the cache is
  missing or stale, the user may run ordinary `graft build` locally;
  this is the deterministic $0 code-graph mode, without `--deep`.
  Do not claim a build occurred if it was not executed.
- Helpful documented no-key commands: `graft map`,
  `graft ask "question" --source`, `graft skeleton <file>`,
  `graft callers <symbol>`, `graft grep "pattern"`, and
  `graft blast`. Validate critical findings against live source.
- `graft init` **modifies agent instruction/configuration files**;
  never silently run it merely to create a cache. Review its planned
  changes separately and preserve all existing AGENTS.md and
  project-specific rules.
- Keep regenerable `graft/` outputs out of commits; use a
  local-only exclusion or disposable graph if needed. Do not put
  credentials, research patient data, or repository secrets in
  generated graph content.
- Do not request paid external LLM calls or use `graft build --deep`
  absent an explicitly authorized zero-cost local model.
  No external enrichment, telemetry claims, provider results, or
  code-review PASS may be invented.

These Graft notes do not supersede exact-head signed/DCO
discipline, genuine GitHub CI, deterministic validators, founder
governance or the separately protected AF-02 authority rules.
Alibaba OCR and Jev are separate optional reviews when actually
available and executed; unavailable checks are NOT_RUN.
<!-- graft:end -->
