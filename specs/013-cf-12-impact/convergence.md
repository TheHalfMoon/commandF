# CF-12 Convergence — Deterministic Impact Analysis

Status: CONVERGENCE_CANDIDATE — this record is not canonical until the docs-only closeout PR is itself qualified and merged.

## Canonical implementation merges

- Stack A PR: `#26` — `feat(impact): add deterministic blast-radius library`
  - exact qualified head: `9fa948cb2ad0110cd4288c330a5bc8b977472418`
  - merge commit: `d46591f0f7224d49fda0d89a6a79cc418fba534e`
- Stack B PR: `#27` — `feat(impact): ship deterministic commandf impact`
  - exact qualified head: `6d8e22b1d8c999256692052d473ba3c27effc972`
  - merge commit: `9e462cbb5c0bd05cf2219e2283f09bfbc8a51720`
- T022 follow-up evidence PR: `#28` — `test(impact): prove reverse package exposure through CLI`
  - exact final qualified head: `c874c8c665a053d3022b6592a6dcf2a9f9c88349`
  - exact tree: `0ab82f0d8fb19d88ddcb0af1fbc5a4cd8535b765`
  - merge commit: `71c5c4372a829ca6b26846acad0a8ded44f1e1ba`
  - merged main tree: `0ab82f0d8fb19d88ddcb0af1fbc5a4cd8535b765`

The T022 follow-up changes tests only. No production behavior changed after Stack B.

## Exact-head workflow qualification

Final implementation/evidence head:

```text
head  c874c8c665a053d3022b6592a6dcf2a9f9c88349
tree  0ab82f0d8fb19d88ddcb0af1fbc5a4cd8535b765
```

Applicable workflows on that exact head:

```text
ci                       run 32942924918  SUCCESS
cf11-multi-version-proof run 32942924926  SUCCESS
cf11g-context-proof      run 32942924928  SUCCESS
cf12-impact-proof        run 32942924956  SUCCESS
cf06-oracle              run 32942924969  SUCCESS
```

`ci` job `98097504281` passed the mandatory workspace gates and the configured integrated regressions:

- `cargo fmt --all -- --check`;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
- `cargo test --workspace --all-features`;
- CF-08 Action runner security regression;
- CF-09 Action source-map security regression;
- real FHIR registry/inspect/self-diff/self-classify/self-check smoke;
- real FHIR self-terminology smoke;
- local GitHub Action source-map self-check and output verification.

No post-merge workflow run was emitted for merge commit `71c5c4372a829ca6b26846acad0a8ded44f1e1ba`; repository workflows qualified the exact PR head before merge, and the merge commit retained the same tree.

## Deterministic impact proof identity

Dedicated proof:

```text
workflow  cf12-impact-proof
run       32942924956
job       98097504274
head      c874c8c665a053d3022b6592a6dcf2a9f9c88349
tree      0ab82f0d8fb19d88ddcb0af1fbc5a4cd8535b765
result    SUCCESS
```

The job passed the pinned-toolchain assertion, byte-identical repeated `commandf impact` execution, repository-cleanliness assertion, and evidence upload.

Retained evidence:

```text
artifact name    cf12-impact-proof
artifact id      9597183002
artifact digest  sha256:1cf1fc14c84f35a84c00c01ed2cc475a0c310e374dd14f3105ae4ac08bb79c1f
CF12_IMPACT_SHA256=e75f54cefc9af93819fb11b437418c04f6fe8036bef3e4be1ccf6523170c84b1
```

## Independent review

Final evidence PR `#28` received independent Qodo review. One substantive correctness finding identified that the initial CLI regression assertions did not bind the declared constraints to the specific `acme.subject` package-impact relation. The finding was fixed in commit `6210db22c08a5e9a0b6e9f9b5c7653b771da0795`; the regression then selected the exact impacted package name/version relation and asserted side-local declared constraints. The thread is resolved and outdated on the final head.

CodeRabbit commit status on the final head is `success`; its status description reports review rate limiting. No CodeRabbit PASS beyond that recorded status is invented.

Unresolved substantive review findings at convergence: `0`.

## Acceptance-contract convergence

CF-12 now ships the V1 `commandf impact` vertical slice and preserves the frozen authority boundary:

- deterministic structural-diff-derived change seeds;
- side-aware reverse traversal over exact CF-11G `resolved` canonical-reference edges only;
- deterministic shortest-path evidence with lexicographic equal-length tie-breaking;
- cycle termination and deterministic deduplication;
- exact schema-v2 multi-version reverse package exposure with declared constraints;
- explicit `external` and `ambiguous` unresolved boundaries without traversal or network completion;
- before/after evidence preservation with `both` only for exactly identical normalized relations;
- JSON-only V1 CLI with explicit before/after lock/cache inputs;
- fail-closed unsupported-schema and corrupt/missing local evidence behavior;
- deterministic retained CLI proof identity;
- no compatibility severity inferred from reachability.

The final T022 CLI regression additionally proves reverse package exposure for `acme.subject` through a changed `acme.shared` dependency and binds exact before/after constraints to the corresponding package-impact relations.

## Coverage limits and explicit deferrals

The following remain intentionally outside CF-12 V1 and are not gaps in its accepted contract:

- SQL-on-FHIR, CQL, SearchParameter-expression, and FHIRPath-invariant impact extraction;
- persistent graph storage or graph databases;
- network canonical completion;
- AI/model/agent impact authority;
- runtime or clinical breakage claims derived from reachability;
- CF-06 production oracle identity changes;
- mutation of the frozen CF-10 corpus;
- PHI/instance-data analysis.

Artifact-level blast-radius evidence is limited to relation kinds extracted by CF-11G V1. Package-level exposure remains independent of artifact extractor coverage.

## Convergence decision

Implementation and evidence satisfy CF-12's frozen V1 acceptance contract on the exact qualified implementation tree. This docs-only closeout may be merged only after its own exact-head, path-applicable CI/review gates are terminal and clean.

After that merge, CF-12 may be recorded as `CLOSED_CANONICAL` and the repository may proceed to the next dependency-eligible slice under Master Architecture V2.