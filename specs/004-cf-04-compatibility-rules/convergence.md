# CF-04 Convergence Review

Status: Implementation and convergence complete — founder review candidate
Date: 2026-08-14

## Scope result

CF-04 stays inside the authorized slice: deterministic compatibility classification over CF-03 structural facts with explicit producer/consumer direction and `BREAKING`, `RISKY`, or `ADDITIVE` severity.

It does **not** add SARIF or finding-based quality-gate exit codes, validator-oracle judgments, terminology expansion/set inclusion, GitHub annotations, FSH source mapping, ecosystem blast radius, mapping execution, or AI authority. Those remain CF-05 or later work.

## Stack and architecture

CF-04 is stacked directly on exact CF-03 head:

```text
aa212b108e05fa0e22312f244f393c59602192b9
```

No new workspace crate was required.

The implementation uses:

- CF-03 `StructuralDiffReport` as the only structural-fact input;
- a public CF-04 validation/indexing layer for fail-closed code-value checks, duplicate-key checks, byte-fact subsumption, and indexed snapshot/differential deduplication;
- the versioned compatibility rule engine in `commandf-pkg`;
- `commandf classify` in the existing CLI, sharing the exact CF-03 `build_diff_report` two-state loader.

The classifier never reacquires packages.

## Public contract achieved

```text
commandf classify <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --format json
```

The emitted report contains schema `1`, ruleset `cf04-rules-v1`, before/after package evidence, and stable ordered findings. Every finding carries a stable rule id, severity, direction, source structural kind, resource evidence, and optional view/element/field/before/after values.

No `SAFE` state exists and no model-authored compatibility judgment exists.

## Directional rule result

### Cardinality and maximum length

- `min` increase -> producer `BREAKING`;
- `min` decrease -> consumer `BREAKING`;
- `max` decrease -> producer `BREAKING`;
- `max` increase -> consumer `BREAKING`;
- `maxLength` uses the same directional tightening/relaxation logic;
- `*` is treated as unbounded.

### Type choices

The implementation deliberately avoids a false-positive trap discovered during convergence:

- type-code set narrowing -> producer `BREAKING`;
- type-code set widening -> consumer `BREAKING`;
- incomparable code sets -> `BREAKING` both;
- unchanged code set with changed profile/targetProfile/aggregation qualifiers -> `RISKY` both;
- unavailable direct code comparison -> `RISKY` both.

CF-04 therefore does not claim profile-subset semantics it has not proven.

### Fixed, pattern, and value bounds

- fixed add/remove/change has deterministic directional/bilateral `BREAKING` rules;
- pattern add/remove is directional `BREAKING`, while pattern replacement is `RISKY` both;
- generic minValue/maxValue add/remove is directional, while bound rewrite is `RISKY` both without a typed value-ordering proof.

### Terminology binding

FHIR R4 strength ordering is implemented as:

```text
example < preferred < extensible < required
```

Strengthening is producer-breaking; weakening is consumer-breaking. A ValueSet canonical change is `RISKY` both because set membership/subset proof belongs to CF-07.

Present non-string or unrecognized binding strengths fail closed.

### Constraints

Constraint semantics are keyed by `constraint.key`:

- error invariant add -> producer `BREAKING`;
- error invariant remove -> consumer `BREAKING`;
- warning add/remove -> directional `RISKY`;
- warning -> error -> producer `BREAKING`;
- error -> warning -> consumer `BREAKING`;
- same-key expression/metadata rewrite -> `RISKY` both unless a later oracle proves implication/equivalence.

Duplicate constraint keys fail closed rather than allowing a map overwrite.

### Must Support, modifiers, slicing

Must Support changes remain `RISKY` both because R4 support obligations are contextual and separate from cardinality.

New modifier semantics are consumer `BREAKING` plus producer `RISKY`; other modifier changes are `RISKY` both.

Slicing uses `open < openAtEnd < closed`: restriction is producer-breaking, relaxation is consumer-risky, unordered->ordered is producer-breaking, and other/discriminator changes remain risky. Unknown present slicing rules fail closed.

## False-positive controls

CF-04 convergence intentionally prefers `RISKY` over unsupported `BREAKING` claims for:

- profile/targetProfile/aggregation qualifier rewrites with unchanged type codes;
- generic pattern rewrites;
- generic value-bound rewrites;
- ValueSet changes without terminology-set proof;
- same-key invariant rewrites without FHIRPath implication proof;
- Must Support changes;
- profile/context/slicing semantics without a proven subset relation.

This is a deliberate trust boundary, not missing severity logic.

## Fail-closed behavior

The public classifier rejects:

- unsupported CF-03 schema versions;
- unknown future interpreted structural fields;
- malformed rule evidence such as invalid booleans/integers/max forms;
- duplicate constraint keys;
- present non-string or unrecognized `binding.strength`;
- present non-string or unrecognized `slicing.rules`.

Corrupted CF-01 cache objects fail before classification, and the CLI regression asserts the digest-mismatch reason and expected digest.

## Deduplication and residual evidence

Equivalent snapshot/differential field facts are deduplicated with snapshot evidence winning. Snapshot identities are pre-indexed in a `BTreeSet`, avoiding the earlier public-path O(n²) nested scan.

A generic `resource_bytes_changed` finding is emitted only when the same resource lacks a more precise structural fact. More precise structural evidence subsumes the byte-only residual finding.

## Deterministic synthetic and CLI evidence

Regression coverage proves:

1. empty report -> empty findings;
2. repeated classification -> byte-identical JSON;
3. cardinality and maxLength variance;
4. type-code narrowing/widening and qualifier-only RISKY behavior;
5. fixed/pattern/value-bound rules;
6. binding-strength direction plus ValueSet RISKY behavior;
7. constraint add/remove/severity/rewrite behavior;
8. Must Support, modifier, and slicing behavior;
9. resource/view/element and residual rules;
10. snapshot/differential deduplication;
11. byte-fact subsumption;
12. unknown future field/schema failure;
13. duplicate constraint-key failure;
14. unknown/malformed binding-strength and slicing-rule failure;
15. CLI help, offline classify success, and corrupted-cache failure reason.

## Green implementation evidence

Exact implementation head:

```text
eccf64f8450cd50e72f771e1c2ade947fead7eb0
```

GitHub Actions run:

```text
31763844412
```

Result:

- Format — PASS;
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` — PASS;
- `cargo test --locked --workspace --all-features` — PASS;
- independent registry resolve/verify of published `hl7.fhir.r4.core@4.0.1` into explicit before state — PASS;
- real CF-02 inspect — PASS;
- second independent resolve/verify into explicit after state — PASS;
- real CF-03 self-diff — PASS with `changes == []`;
- real CF-04 self-classify — PASS with `findings == []`.

Documentation-only convergence follows this implementation head. The exact final documentation head must pass the same complete gate; that run is recorded in PR metadata to avoid a self-referential documentation commit chain.

## Reviewer evidence

### CodeRabbit

A manual review was triggered on a green implementation candidate and returned two actionable findings:

1. **Unknown binding/slicing codes could under-report as RISKY.** Accepted. The public classifier now rejects present non-string or unknown `binding.strength` and `slicing.rules` values before rule dispatch, with regression coverage.
2. **Differential deduplication used a nested O(n²) scan.** Accepted. The public path now pre-indexes snapshot field identities and performs indexed membership lookups.

Both findings were fixed, validated by run `31763844412`, replied to with exact-head evidence, and both review threads are resolved.

CodeRabbit also raised routine nitpicks. The corrupted-cache failure reason assertion, stronger determinism/rule-family coverage, temporary CI push-trigger cleanup, and documentation of byte-fact subsumption were adopted. Sharing repeated Clap path arguments between `diff` and `classify` was reviewed and intentionally not adopted because it was non-functional refactor churn with no correctness or contract benefit in CF-04.

### Qodo

`/review` was requested. No returned Qodo review result or finding is used as evidence. No Qodo PASS is claimed unless a result exists in the final PR timeline.

### Cubic

Cubic supplied automated PR summaries. No separate substantive blocking Cubic finding is treated as certification or as a convergence gate.

## Convergence decision

**CF-04 implementation is converged and is a founder-review candidate, subject only to the exact final documentation head passing the complete CI gate.**

PR #5 remains Draft and unmerged. No merge or auto-merge is authorized. CF-05 has not started.

## Explicit deferrals

- SARIF and finding-based quality-gate exit semantics — CF-05;
- FHIR Validator differential oracle — CF-06;
- terminology expansion and set inclusion — CF-07;
- GitHub annotations — CF-08;
- FSH source mapping — CF-09;
- ecosystem dependency graph and blast radius — CF-11/12;
- mapping execution and semantic-loss runtime;
- AI/agent authority.
