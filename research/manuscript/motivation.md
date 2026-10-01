# Motivation

Status: RESEARCH_PLANNING. Reasons for the distinctions already defined. Not a demonstration that the distinctions improve outcomes.

A package diff answers whether bytes changed. A validator answers whether an artifact meets a rule it was given. A consumer that reads one element can break when that element changes, while another consumer never reads it. `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` section 8 says popularity and registry presence do not stand in for an observed contract, and missing contract information stays missing. The manuscript consumer-contract section records that the versioned contract is not implemented. The reason for the section is that stopping at the artifact would hide that split.

An empty finding list is not coverage of the protected scope. The planned class `PROVEN_COMPATIBLE` requires that coverage, and the plan says it is never inferred from "no finding" alone. The shipped check can still report `passed` when the blocking count is zero. Treating that boolean as proof would erase unsupported fields, unrun oracles, and disagreements. The motivation for keeping those states visible is that a silent success is a different claim from the evidence that was supplied.

The charter's transformation hypotheses, including information-loss accounting, remain unmeasured. They are not the reason this manuscript gives for the current commands. The reason here is the decision boundary in the V3.1 planning candidate: say what a change can break, for which declared scope, with what evidence, and what remains unknown, before the change ships. That sentence is a target. It is not an outcome this repository has measured.

## What this section does not claim

It does not claim the target is met. It does not claim fewer unsafe allows than another tool.
