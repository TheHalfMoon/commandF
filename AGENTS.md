# commandF Engineering Rules

commandF is interoperability infrastructure. Review and implementation must preserve evidence, determinism, and explicit semantics.

## Non-negotiable rules

- FHIR is a first-class dialect, not the sole internal canonical model.
- Never silently discard known source information; preserve it or emit an explicit loss record.
- Never invent clinical facts merely to satisfy a target schema.
- AI may propose mappings, fixes, or findings; deterministic validators, tests, and policies provide verifiable evidence.
- Keep source identity, exact dialect/version, provenance, and content digests explicit.
- Treat mutable tags or floating donor references as insufficient for production certification.
- External validators remain independently identified; do not collapse one implementation into universal truth.
- Preserve lexical precision for numeric and temporal source values when possible.
- Prefer fail-closed behavior when required verification is unknown or not run.
- Every new contract needs positive and negative tests.
- Avoid panics in library code for externally supplied data.
- Research hypotheses and literature claims belong under `research/` and must not become product guarantees without evidence.

## Review priorities

1. Silent semantic or information loss.
2. Incorrect conformance or certification claims.
3. Provenance/version ambiguity.
4. Non-deterministic behavior.
5. Unsafe auto-fixes or AI-as-authority paths.
6. Missing compatibility and failure-path tests.
7. API/contract changes that can break downstream consumers.

## Change discipline

Keep changes small and stackable. Prefer one independently reviewable contract or behavior per PR. Do not merge a stack until CI and required reviewers are green for the exact candidate state.
