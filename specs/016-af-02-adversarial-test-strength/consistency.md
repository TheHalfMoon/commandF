# AF-02 Consistency Analysis

Status: PLANNING_CANDIDATE

## Scope

This analysis reconciles:

- `.specify/memory/constitution.md`;
- `AGENTS.md`;
- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`;
- `docs/COMMAND_F_PLAN_INDEX.md`;
- `docs/COMMAND_F_ASSURANCE_PROGRAM_2026-08-26.md`;
- canonical AF-01 closeout and live source-control rulesets;
- canonical CF-06 production-oracle authority;
- retained CF-10 frozen-corpus authority;
- current commandF parser/acquisition/cache/path/graph/report/evidence boundaries;
- `donors/af-02-adversarial-testing.yaml`;
- AF-02 `spec.md`, normative `evidence-contracts.md`, `plan.md`, and `tasks.md`;
- exact upstream tool identities inspected during planning;
- Qodo and CodeRabbit findings on the initial PR #54 planning head.

Canonical planning base:

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

## Identity and ordering consistency

### AF-02 does not consume CF-16

`016-af-02-*` is a Spec Kit directory sequence only. Product identities remain CF-14, CF-15 and CF-16.

### AF-02 planning is authorized after AF-01

The canonical assurance program retained AF-02 as the next adversarial-test-strength unit. AF-01 explicitly handed off fuzzing, property tests, mutation adequacy, coverage, flaky-as-failure and minimized corpus work. Planning is authorized; implementation is not authorized until T006 is canonical.

### Design-freeze PRs do not violate vertical-capability rules

They are independently executable assurance-policy verification results and exist to prevent hidden post-result acceptance design. Dependent implementation cannot begin until each design freeze is canonical.

## Authority consistency

### AF-01

AF-02 does not replace or weaken the live required contexts. `evidence-contracts.md` freezes semantic read-back for:

- assurance ruleset `21652953`, including no bypass, deletion/non-fast-forward and strict GitHub Actions checks `rust`, `assurance-proof`, `scorecard`;
- review-governance ruleset `21652974`, including one review, code-owner/latest-push/thread-resolution protections, merge-only and PR-only repository-role bypass.

Every Stack and final proof performs live read-back. This is always-run non-regression, not conditional on proposing a new required check.

### CF-06

The production oracle remains independent advisory authority:

```text
hapifhir/org.hl7.fhir.core 6.10.2
d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
validator_cli.jar sha256 a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6
hl7.fhir.r4.core@4.0.1
```

AF-02 does not reinterpret blocked oracle outcomes or authorize a pin change.

### CF-10

The frozen corpus remains:

```text
C001 hl7.fhir.us.core   8.0.1 -> 9.0.0
C002 hl7.fhir.uv.ips    1.1.0 -> 2.0.1
C003 hl7.fhir.us.mcode  3.0.0 -> 4.0.0
```

The retained six-state evidence identity is PR #11 head `5fe10d9859407272acf6649fc3e868d3eb2fbd12`, run `31916124080`, artifact `9255732702`, digest `9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612`.

PR #11 remains open/draft/unmerged. AF-02 does not merge it, replace a case, or convert the CF-06 production-oracle blocker into semantic success.

## Tool/provenance consistency

AF-02 tools remain development/test assurance only. They are not product semantic authority and do not enter commandF runtime.

Planning source identities are exact commits. Implementation additionally requires exact executable/package artifact identity through `commandf.af02-tool-lock/v1`.

Allowed executable acquisition is only locked exact-revision source build or immutable release asset with SHA-256. `latest`, branch-only, tag-only and self-update proof identity are rejected.

Crates.io test/fuzz packages use exact package version plus Cargo registry checksum. This resolves the distinction between an upstream donor commit and the actual package artifact consumed by Cargo.

## Deterministic versus stochastic consistency

AF-02 deliberately does not claim fuzz discovery is reproducible correctness proof.

- deterministic proof includes surface-policy validation, corpus replay, properties, canonical cargo test, nextest no-flake truth, fixed-descriptor coverage, frozen-inventory mutation classification and authority verification;
- stochastic discovery records bounded campaign configuration and observations separately;
- no-crash duration is `NO_CRASH_OBSERVED_WITHIN_BOUND`, never PASS-by-time;
- interrupted/resource/harness failure is incomplete, not clean.

Only the deterministic object contributes to `AF02_ADVERSARIAL_SHA256`.

## Exact proof consistency

The original planning package named `AF02_ADVERSARIAL_SHA256` without a complete canonicalization contract. The amended normative contract now freezes:

- schema `commandf.af02-adversarial-proof/v1`;
- parsed/schema-validated JSON;
- no floating-point deterministic fields;
- recursive UTF-8 object-key ordering;
- schema-defined array order and separately verified sorted/deduplicated sets;
- minimal decimal integers;
- compact UTF-8 JSON, no insignificant whitespace or trailing newline;
- SHA-256 only over the deterministic object.

An independent repository verifier reconstructs the object from raw evidence and recomputes the digest. A producer-supplied summary is not trusted.

## Policy authenticity consistency

A PR can edit policy and evidence together, so candidate policy is not automatically authority.

The base-policy anti-forgery rule compares candidate SHA/tree with canonical base SHA/tree and classifies weakening changes. Coverage-floor/exclusion weakening, surface removal, discovery exclusion, resource/offline relaxation, flaky-pass overrides, mutation required-set reduction/new waivers, corpus assertion removal, provenance relaxation and authority-baseline changes cannot make the same candidate green.

A weakening either passes prior canonical policy too or is isolated into a dedicated reviewed policy PR merged before implementation. Strengthening applies the stricter base/candidate policy immediately.

This prevents hand-authored or self-serving green evidence from becoming proof.

## Coverage consistency

Coverage design is frozen before measurement:

```text
Linux/x86_64
Rust 1.97.1 + llvm-tools-preview
cargo llvm-cov --workspace --all-features --locked --json
production source crates/*/src/**
```

Only explicit non-product exclusions are initially allowed. Baseline descriptor binds exact source/tree/tool/compiler/lock/manifests/command/platform/raw report/corpus/property configuration.

Floors are mechanically derived after design freeze: integer floor of measured workspace production line percentage and integer floor independently for each critical surface. No averaging. Function/region values are retained diagnostics initially.

Rebaseline/floor lowering requires a dedicated prior-policy-reviewed PR; it cannot rescue its own candidate.

## Mutation consistency

The amended contract freezes mutation semantics before execution:

- exact cargo-mutants 27.1.0 tool lock;
- exact command/config/profile/test command/parallelism/timeouts/source scope;
- baseline required;
- JSON inventory via `--list --json` or exact-version equivalent;
- inventory digest and stable mutant identity bound to source/tool/config;
- explicit KILLED/SURVIVED/TIMEOUT/UNVIABLE_OR_BUILD_FAILURE/WAIVED classes;
- bounded retry and diagnosis for every TIMEOUT/UNVIABLE;
- unresolved required results need exact reviewed waiver and never count as killed;
- new waiver or reduced required set cannot self-green a candidate.

This removes aggregate-score and incomplete-run false-PASS paths.

## Nextest consistency

Pinned nextest documentation confirms:

- `--retries 2` means two retries after the first attempt;
- retry-pass is green by default unless flaky-result failure is selected;
- `--flaky-result fail` makes retry-pass fail;
- command-line `--retries` and `--flaky-result` disable weaker per-test overrides.

AF-02 therefore freezes both repository profile and CLI authority. The deterministic isolated self-test uses an AF-02-owned state file so first-fail/retry-pass behavior does not depend on time, RNG, scheduler or network.

Canonical `cargo test --workspace --all-features --locked` remains separately mandatory.

## Fuzz/resource/network consistency

Resource limits are executable policy rather than broad advice. Routine fuzzing is bounded below AF-04 stress scope with explicit per-input timeout, input bytes, memory, CPU, PID, tmpfs, generated/decompressed bytes, temporary files, subprocess timeout and artifact/corpus limits.

Deterministic qualification separates networked immutable acquisition from offline execution using Cargo offline mode plus OS/container network denial. Missing effective offline enforcement fails when the policy requires it.

Expected outcomes are normalized and surface-specific. Unexpected acceptance, invariant violation, oracle divergence or panic fails immediately; harness/resource failure is incomplete rather than clean.

## Property/oracle consistency

The original plan correctly prohibited calling the same implementation twice an independent oracle but did not assign concrete alternatives. The normative contract now freezes test-owned independent models for archive manifests, Lockfile graph/canonicalization, portable source paths, canonical reference resolution and quality-gate/fingerprint set/truth-table behavior.

Property records include generator validity/invalidity domain, bounds, case count, seed/shrink policy and expected model.

## Corpus consistency

`commandf.af02-corpus/v1` now freezes:

- stable `AF02-<SURFACE-SLUG>-NNNN` namespace;
- SHA-256 over raw stored bytes;
- provenance class and public redistribution basis;
- expected normalized outcome;
- assertion and replay IDs;
- <=256 KiB default single fixture and <=8 MiB aggregate corpus;
- machine-checkable assertion registry and actual execution of every entry.

Metadata alone cannot pretend a failure was promoted.

## No-PHI and artifact consistency

The primary no-PHI boundary is provenance classification: synthetic or public redistributable only. Unknown/private/patient-derived provenance is rejected. Scanner checks are defense in depth rather than an impossible claim that patterns alone prove absence of PHI.

Generated fuzz artifacts are opaque bytes, never executed. Retention rejects path escape, symlink/device/FIFO/socket/executable files and unsafe names; logs retain only bounded escaped preview plus digest/size.

## CI consistency

The amended logical job topology separates immutable acquisition, deterministic adversarial execution, nextest, coverage, mutation, stochastic discovery and exact-head proof. Every job follows AF-01 immutable Action/permission/checkout/timeout requirements.

Cancellation or incomplete current-head execution is not PASS. Scheduled discovery interruption is incomplete, not no-crash. Artifact retention is explicit and bounded.

AF-02 does not make a new live main required check by default. Any such proposal requires separate universal-terminal topology proof and live ruleset reconciliation.

## Review findings reconciliation

Initial PR #54 planning head:

```text
3224098403f6bfb64525bfab002e94d5c3d82e69
```

All five existing workflows completed successfully on that head, but this did **not** satisfy planning because Qodo and CodeRabbit returned substantive findings.

### Qodo — 16 accepted gaps

1. **Temporal exact-head/live evidence not yet retained.** Correct: T006 remains open and requires amended-head CI/reviews/merge/post-merge live read-back. Planning text does not fabricate future evidence.
2. **Artifact-level tool provenance incomplete.** Resolved in normative `commandf.af02-tool-lock/v1` and donor immutable acquisition contract.
3. **Surface policy/discovery underspecified.** Resolved with source roots, boundary categories, matcher ownership, exclusions, stale-entry and unclassified-boundary failure.
4. **Proof digest canonicalization underspecified.** Resolved with exact JSON rules and independent recomputation.
5. **Evidence authenticity/same-PR gaming.** Resolved with canonical-base anti-forgery and dedicated weakening-policy PR rule.
6. **Coverage scope/floors open.** Resolved with pre-measurement descriptor, explicit source/exclusion scope and independent floor calculation.
7. **Mutation qualification underbounded.** Resolved with exact design-freeze config/inventory/timeouts/stable IDs/result classes.
8. **Fuzz resource/outcome semantics incomplete.** Resolved with executable resource/offline policy and normalized result classes.
9. **Structured independent oracles absent.** Resolved with test-owned independent models per required surface family.
10. **Nextest self-test/override semantics open.** Resolved with deterministic state-file fixture, CLI `--retries 2 --flaky-result fail` and bounded slow timeout.
11. **Corpus promotion not enforceable.** Resolved with raw digest, stable IDs, size bounds and assertion/replay binding.
12. **AF-01 preservation too prose-like.** Resolved with exact semantic ruleset snapshots/digests and always-run live read-back.
13. **CF-06/CF-10 preservation not machine-checked.** Resolved with exact authority baseline schema and independent derivation.
14. **No-PHI enforcement missing.** Resolved with provenance gate, defense-in-depth scan and opaque artifact handling.
15. **CI topology/partial-run semantics open.** Resolved with fixed logical jobs, network separation, cancellation/incomplete semantics and artifact limits.
16. **Hidden implementation decisions could leak forward.** Resolved by mandatory separate Stack A0/B0/C0 design-freeze PRs before dependent implementation.

### CodeRabbit — 6 accepted actionable comments

1. **Deterministic boundary discovery and stale entries.** Resolved in surface-policy contract and T011-T013.
2. **Immutable tool acquisition/executable verification.** Resolved in donor contract, tool-lock schema and T016/T020/T030/T033/T050.
3. **Executable resource/offline policy.** Resolved in resource schema, explicit initial limits and T014-T015.
4. **Per-test flaky-result override can weaken config.** Resolved by mandatory nextest CLI `--retries 2 --flaky-result fail` plus self-test.
5. **Coverage baseline descriptor/rebaseline policy missing.** Resolved in fixed descriptor, anti-gaming and Stack B0 tasks.
6. **TIMEOUT/UNVIABLE mutation closure too weak.** Resolved with bounded retry+diagnosis and exact waiver/anti-self-green rules.

No prior reviewer finding is waived as “just planning.” The design is amended.

## Remaining planning truth

The amendments themselves still require fresh exact-head evidence. The prior workflow successes and reviews belong to the superseded head and cannot qualify the amended head.

T005/T006 remain open until:

- amended exact head passes all path-applicable workflows;
- fresh Qodo and CodeRabbit review the amended exact head;
- zero unresolved substantive review threads remain;
- PR #54 merges from exact qualified head with expected-head guard;
- canonical main/tree and live AF-01 rulesets are re-read post-merge.

Only then:

```text
AF-02 PLANNING: CANONICAL
IMPLEMENTATION AUTHORITY: GRANTED FOR STACK A0 DESIGN FREEZE ONLY
```

Until then:

```text
AF-02: PLANNING_CANDIDATE
IMPLEMENTATION AUTHORITY: NOT_GRANTED
```