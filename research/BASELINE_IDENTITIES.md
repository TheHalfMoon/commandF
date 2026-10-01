# Baseline Identities

Status: RESEARCH_PLANNING. Identities for the planned B0–B7 families. Not a run. Not a score. Not a protocol freeze.

Observed source tree: main `616ef5d61762ae9870a3036268e33ea21a9cbcc5`. The crate version in `crates/commandf-cli/Cargo.toml` is `0.0.0`. The workspace `rust-version` is `1.97.1`. No binary digest is recorded, because this record does not build one.

```text
BASELINE_IDENTITIES = SPECIFIED
BASELINE_EXECUTION = NOT_EXECUTED
BUILT_BINARY_DIGEST = NOT_RECORDED
RESULT = RESULT_PENDING
```

## Commands that exist at that commit

| Command | What the source calls | Default flags in the CLI |
| --- | --- | --- |
| `commandf diff` | `build_diff_report` | JSON |
| `commandf classify` | `build_diff_report`, then `classify_structural_diff` | JSON |
| `commandf check` | that classification, then `evaluate_compatibility_policy` | `--direction both`, `--fail-on breaking`, JSON |
| `commandf context` | `build_context_graph` on one lock and one cache | JSON |
| `commandf oracle` | `run_hl7_oracle_adapter` | JSON, `--oracle-adapter` required |

`diff`, `classify`, `check`, and `oracle` take a package name plus a before lock, a before cache, an after lock, and an after cache. Their stdout is JSON. `check` returns process status 0 when `decision.passed` is true and 2 otherwise. This record does not treat either status as a benchmark label. CommandF output is not a label.

## Family map

| Family | Identity at this commit | Capability boundary |
| --- | --- | --- |
| B0 structural diff only | `commandf diff` | archive structural diff JSON. It does not call `evaluate_compatibility_policy`. `classify` is a later structural classification of that diff. It is not B0 and it is not B2. |
| B1 FHIR validation only | not pinned | `commandf oracle` is not validation-only. It requires a caller-supplied adapter path and an optional Java path. |
| B2 deterministic compatibility rules | `commandf check` with the defaults above | policy over the structural classification, including the diff those functions read. |
| B3 rules plus context graph | `NOT_IMPLEMENTED` | `context` is a separate command. No command feeds that graph into `check`. |
| B4 rules plus graph plus consumer contracts | `NOT_IMPLEMENTED` | `--direction consumer` is a check direction. It is not a consumer-contract document, and the crates do not contain one. |
| B5 plus authoritative or differential oracle evidence | `NOT_PINNED` | no command adds oracle evidence onto the `check` decision. |
| B6 plus sufficiency and abstention | `NOT_IMPLEMENTED` | the crates do not name abstention or sufficiency. |
| B7 plus calibrated advisory layer | `NOT_IMPLEMENTED` | the crates do not name an advisory layer. |

## Declared validator coordinate, not an adapter pin

`tools/hl7-oracle/pom.xml` at the same commit declares `ca.uhn.hapi.fhir:org.hl7.fhir.validation:6.10.2`, `ca.uhn.hapi.fhir:org.hl7.fhir.r5:6.10.2`, Jackson `2.22.1`, and UCUM `1.0.10`. The compiler release is 17. The CLI constant for the oracle core package is `hl7.fhir.r4.core` at `4.0.1`, and the command refuses the run when the before and after core digests differ.

That coordinate is not a built jar digest. The Java runtime is not pinned. B1 and B5 stay unpinned until a later record supplies the adapter bytes, their SHA-256, and the Java runtime identity. This record does not download those bytes.

## Fairness, still untested

A later execution must give every family that actually runs the same package bytes. It must record the built binary digest. It must not replace a missing family with another tool after a score is known. A family marked `NOT_IMPLEMENTED` or `NOT_PINNED` is not given a result of zero, and it is not dropped from the planned list because it has no score.

Local compilation of this tree does not call a paid model. This record does not compile it and does not run it.

## What this record does not do

It does not assign a split. It does not change `CORPUS_DIGEST`. It does not freeze the protocol. It does not compute a statistic. It does not authorize CF-17, AF-02, AF-03, AF-04, or CF-18.
