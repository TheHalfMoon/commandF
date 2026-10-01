# Selective Oracle Escalation

Status: RESEARCH_PLANNING. Architecture description only. No oracle-performance result. B1 and B5 stay `NOT_PINNED`.

## What can run today

`commandf oracle` is a caller-invoked command. It is not a step that `commandf check` starts on its own. The caller supplies `--oracle-adapter` and may supply a Java path. The core package constant is `hl7.fhir.r4.core` at `4.0.1`. The command refuses the run when the before and after core digests differ. It stages archives through the verified cache. When the package digests differ, it asks the adapter for observations and passes them to `reconcile_hl7_oracle`.

The reconciliation report is `OracleDivergenceReport` schema 1. A resource status is one of `Agreement`, `CommandfOnly`, `AuthorityOnly`, `BothChanged`, or `Uncomparable`. Those names are not the planned oracle-outcome enum, and they do not change `CheckDecision.passed`. An unsupported oracle schema is an error.

`tools/hl7-oracle/pom.xml` declares validator coordinates. `research/BASELINE_IDENTITIES.md` records that a coordinate is not a built jar digest and that the Java runtime is not pinned. This section does not pin them.

## The planned progression

Section 7 of `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` requires an explicit escalation policy: run a required oracle, and do not skip one that the policy marks as required. The planned order, as a logical stack rather than a running pipeline, is static structural evidence, then the context graph, then a consumer witness, then an authoritative oracle, then a differential oracle, then a runtime probe, then a human decision.

The planned outcome names are `NOT_REQUIRED`, `REQUIRED_NOT_RUN`, `RUN_AGREEMENT`, `RUN_DIVERGENCE`, `UNAVAILABLE`, `FAILED`, `TIMED_OUT`, and `UNCOMPARABLE`. `REQUIRED_NOT_RUN`, `UNAVAILABLE`, `FAILED`, and `TIMED_OUT` may not become compatibility.

## What is not built

```text
IMPLEMENTED = caller-invoked oracle reconciliation against a structural diff
PLANNED = selective triggers, the planned outcome enum, and the later stages
NOT_IMPLEMENTED = automatic escalation, a runtime probe, a human-decision command, and any path that writes oracle evidence into the check decision
```

Static evidence exists as `commandf diff` and `commandf check`. The context graph exists as a separate command. A consumer witness, in the contract sense of `research/manuscript/consumer-contract-model.md`, is not implemented. A second independent oracle is not pinned, so a differential pair is not bound. No command is a runtime probe. No command records a human decision. No measurement of oracle agreement is in `research/RESULTS_LEDGER.md`.

## What this section does not claim

It does not report how often the adapter agrees with commandF. It does not treat an unrun oracle as compatibility.
