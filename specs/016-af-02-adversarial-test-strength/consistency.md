# AF-02 Consistency Analysis — Adversarial Test Strength

Status: PLANNING_CANDIDATE

## Canonical planning base

```text
main: 2b4033e237a5c74f3c45c12fbc7e7bfdc88067b1
tree: 804ce63c15edb501574bd4aba9a9aadc5bfb07f3
AF-01: CLOSED_CANONICAL
```

This analysis checks the AF-02 planning package for contradictions, false-PASS paths, identity drift, hidden implementation choices, and downstream roadmap collisions.

## Normative document consistency

The authoritative precedence is now explicit and singular:

1. repository governance/constitution/`AGENTS.md`;
2. `verification-protocol.md` and machine-readable schemas;
3. non-superseded `evidence-contracts.md` requirements;
4. `spec.md`;
5. `plan.md`;
6. `tasks.md`;
7. this analysis and donor/provenance records.

The earlier illustrative `commandf.af02-authority-baseline/v1` section in `evidence-contracts.md` is deprecated planning history. It cannot be used as an implementation schema. The only closed baseline is:

```text
commandf.af02-authority-baseline/v2
schemas/af02-authority-baseline-v2.schema.json
```

The proof schema remains:

```text
commandf.af02-adversarial-proof/v1
schemas/af02-adversarial-proof-v1.schema.json
```

This resolves the prior one-schema-name/two-shape ambiguity.

## Preserved external authority

### AF-01

AF-02 does not weaken live source-control assurance. The expected live rulesets remain:

```text
21652953 commandF main assurance
21652974 commandF main review governance
```

The closed projection schemas/digests are defined by `verification-protocol.md`, not the older looser examples in `evidence-contracts.md`.

Any future AF-02 base-verifier required check is added only after its exact workflow has universal terminal topology and after separate live-ruleset reconciliation/read-back.

### CF-06

Production oracle identity remains:

```text
hapifhir/org.hl7.fhir.core
release 6.10.2
source d06577dbc5c62c74a2a8823fbc4830a3024d5b0b
validator_cli.jar sha256 a3addadfa18dfa23146a0a243b6ede68eaad92157a5407738c468bb3d7e4ccd6
R4 context hl7.fhir.r4.core@4.0.1
```

AF-02 reconstructs this from canonical-base source files and does not authorize a pin change.

### CF-10

CF-10 remains Draft/unmerged and its final production gate is still separately blocked by the current CF-06 production comparator contract.

Frozen deltas remain:

```text
C001 hl7.fhir.us.core   8.0.1 -> 9.0.0
C002 hl7.fhir.uv.ips    1.1.0 -> 2.0.1
C003 hl7.fhir.us.mcode  3.0.0 -> 4.0.0
```

Retained source/evidence locators are machine-readable in `retained-authority-sources.json`. They point to exact retained commit/blob identities rather than assuming CF-10 files exist on current `main`.

Retained workflow run `31916124080` conclusion remains `failure`. AF-02 uses its retained package/corpus evidence without relabeling the production gate successful.

### CF-14/15/16

Spec Kit directory `016` does not consume product CF-16 identity. AF-02 does not grant implementation authority to CF-14/15/16.

## Reviewer-driven consistency closure

### First planning review

Initial reviewers identified open design choices in tool provenance, deterministic boundary discovery, resource/offline policy, nextest override resistance, coverage descriptor/floor governance, mutation timeout closure, proof construction, no-PHI provenance, and same-candidate policy weakening.

Those were reconciled into `evidence-contracts.md` and the initial closed protocol.

### Qodo round 2

Qodo then identified that the authority baseline, authority projection recomputation, proof schema, verifier anti-forgery, mutation selection, scanner semantics, assertion registry, resource proof, coverage accounting and nextest fixture were still not closed enough.

`verification-protocol.md` and the machine schemas close those design choices.

### Exact-head follow-up findings

The final planning amendments address the later findings explicitly:

- **CF-10 retained source availability:** exact retained commit/path/blob/API locators are checked in; current-main presence is not assumed.
- **Mutation-selection contradiction:** `spec.md`, `plan.md`, and `tasks.md` now all require every listed mutant in the frozen target scope except exact pre-frozen exclusions. There is no later “choose required mutants” step.
- **Authority baseline duplicate schema:** baseline v1 is deprecated; one closed v2 machine schema is authoritative.
- **Nextest JUnit provenance:** the output mount starts empty; JUnit path is absent/non-symlink before run; base-controlled runner binds wait/exit/JUnit/stdout/stderr in one result envelope and validates owner/location/link count.
- **Normative authority omission:** `verification-protocol.md` and schemas are explicitly in the spec/task authority set.
- **Base verifier candidate control:** Stack A0 uses canonical-base `pull_request_target` workflow code, separate base/candidate trees, candidate-as-data-only, exact blob recording and fail-closed behavior.
- **Proof type/range ambiguity:** the proof JSON Schema freezes structural validation and the protocol freezes semantic/cross-field validation; negative tests cover malformed types/formats/ranges/shapes/relations.
- **Coverage/surface scope mismatch:** both use tracked Rust under `crates/**/src/**` and `tools/**/src/**` minus exact prior exclusions.

## Anti-self-forgery model

The candidate is not allowed to define both the rule and its own success in one change.

After A0 canonicalization:

- acceptance-authority code/config paths are inventoried;
- a candidate diff touching those paths is classified before candidate policy is trusted;
- the canonical-base verifier runs from base-controlled workflow code;
- candidate workflow changes cannot select the base verifier or skip the comparison;
- if old verifier cannot parse a deliberate strengthening, the change is a dedicated policy/verifier PR and cannot carry dependent product/harness work.

This model is stricter than ordinary candidate-self-testing and is necessary because AF-02 verifies the verifier itself.

## Source-universe consistency

Surface discovery and coverage share a single production Rust universe:

```text
tracked *.rs under crates/**/src/**
tracked *.rs under tools/**/src/**
minus exact previously canonical non-product exclusions
```

The scanner may classify a boundary excluded from a particular fuzz target, but the source file does not disappear from coverage merely because the target uses an offline seam.

Unknown/missing/duplicate paths fail instead of becoming implicit exclusions.

## Mutation consistency

Target source paths and exact exclusions freeze before cargo-mutants inventory generation.

The required set is a deterministic function:

```text
required = all listed mutants inside frozen target source paths
           minus exactly matched pre-frozen exclusions
```

Therefore mutation results cannot influence membership. Required timeout/unviable/build-failure results remain non-green until retry/diagnosis and a closure-eligible prior waiver or executable kill result.

## Nextest consistency

Top-level and command-line flaky policy agree:

```text
retries = 2
flaky-result = fail
--retries 2
--flaky-result fail
```

Command-line authority prevents per-test pass overrides from weakening the result.

The deterministic fixture is isolated from clocks/RNG/network/scheduler timing and uses atomic state-file creation. The JUnit file cannot be pre-seeded because the base-controlled runner starts with a clean dedicated output mount and proves the file absent before the waited-for nextest process.

## Coverage consistency

Coverage is diagnostic evidence, not semantic correctness authority.

Before percentage observation, descriptor/source scope/exclusions/commands/inputs are frozen. Source authority comes from Git. Each tracked production source appears exactly once, including zero-hit files.

A floor or exclusion cannot be lowered in the same implementation candidate. Rebaseline is a dedicated policy-only change under the prior policy and cannot modify product source, tests, or measurement command.

## Resource/offline consistency

Network-enabled immutable acquisition and network-denied deterministic execution are separate phases.

Canonical deterministic proof uses a digest-pinned OCI image plus cgroup/read-only/tmpfs/network controls and negative probes. Missing runtime inspection is incomplete evidence rather than assumed enforcement.

## Tool-provenance consistency

An upstream Git commit alone is not an installed executable identity. AF-02 retains acquisition source/asset/package identity, exact install/build command, compiler/target/features, installed executable SHA-256 and version output. Registry crates retain exact Cargo checksum.

## Proof consistency

`schemas/af02-adversarial-proof-v1.schema.json` closes structural field/type/enum/cardinality/unknown-field semantics. `verification-protocol.md` closes canonicalization and semantic relationships that require repository-aware logic.

The verifier constructs the deterministic object from raw evidence. Producer summaries are compared only after reconstruction.

The schema and its SHA-256 are themselves proof contract inputs; changing the schema is an acceptance-authority change.

## Temporal T005/T006 boundary

Planning source cannot contain immutable proof of its own future review completion, merge result or post-merge read-back. Therefore exact-head CI/reviewer evidence lives in GitHub and post-merge authority is re-read after the guarded merge.

This is not a loophole: no merge occurs until current-head CI/reviewer gates are green, and no planning-canonical claim occurs until post-merge read-back.

## Current decision

```text
AF-02: PLANNING_CANDIDATE
T005: OPEN UNTIL ONE EXACT FINAL HEAD QUALIFIES
T006: OPEN UNTIL GUARDED MERGE + POST-MERGE READ-BACK
IMPLEMENTATION AUTHORITY: NOT_GRANTED
NEXT AUTHORITY AFTER T006: STACK A0 DESIGN FREEZE ONLY
```
