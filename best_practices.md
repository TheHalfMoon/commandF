# commandF Review Best Practices

- Preserve source meaning and evidence across transformations.
- Flag silent information loss, hidden coercion, invented defaults, and precision reduction.
- Require exact source/target dialect and version where behavior depends on a standard version.
- Treat AI output as a proposal; require deterministic validation or explicit human approval for authoritative outcomes.
- Keep external validator identity and version visible in evidence.
- Fail closed when required compatibility or verification is unknown.
- Require tests for happy paths, malformed inputs, unsupported features, and loss/recovery behavior.
- Keep package and donor inputs content-addressed for reproducible certification.
- Prefer small, reviewable, stackable changes over broad mixed-purpose PRs.
- Do not convert research hypotheses into product guarantees.
