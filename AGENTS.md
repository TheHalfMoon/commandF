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

Keep changes small and stackable. One independently reviewable, user-visible slice per PR. Do not merge a stack until CI and required reviewers are green for the exact candidate state.

<!-- graft:start -->
## Graft — repository context layer

Use Graft (https://github.com/trailhq/Graft, `@nanonets/graft`) as the default developer/agent codebase context and navigation layer. Graft output is advisory context only; deterministic validators, tests, and project policy remain authoritative evidence.

If Graft is unavailable or the local `graft/` graph is absent/stale, run `graft init`, select the active agent(s), then run `graft build`. Before broad source exploration prefer `graft check`, `graft map`, `graft ask "<question>" --source`, `graft skeleton <file>`, `graft callers <symbol>`, and `graft grep "<literal>"`. After material code changes, run `graft build` again.

Treat `graft/` as a local regenerable cache and do not commit it. Keep usage zero-cost: do not introduce paid model/API usage; any model-backed enrichment must use an already-authorized local or free provider.

Continue all repository-required tests, Jev review/qualification where applicable, Alibaba Open Code Review, CI, and security checks. Never fabricate Graft output, tool execution, CI, reviews, or evidence.
<!-- graft:end -->
