# Introduction

Status: RESEARCH_PLANNING. This frames the manuscript. It is not a result.

commandF is being specified to decide what an interoperability change supports, for which protected scope, and with what evidence. A byte difference, a successful parse, and a proof of compatibility are different questions. `research/manuscript/problem-definition.md` keeps them apart. `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` states that proof is not probability and that absence of evidence is not compatibility. That document is a planning candidate. It does not by itself implement the decision envelope.

The commands that exist record a structural diff and, when asked, a policy result over `Breaking`, `Risky`, and `Additive` findings. `CheckDecision.passed` is true when the blocking count is zero. That boolean is not a planned truth class. Consumer-specific impact, evidence sufficiency, and selective oracle escalation are specified further in the later sections. Several of those pieces are planned or only partly implemented.

`research/RESEARCH_CHARTER.md` records a longer candidate question: whether a typed intermediate representation would improve transformation fidelity among FHIR, openEHR, and OMOP. The charter says those hypotheses are not product claims, and that the intermediate representation is not a prerequisite for the first product stack. This manuscript does not treat that question as a measured thesis. The account here is the change decision and the CommandFBench plan.

The candidate corpus has three members from one source family. The protocol is not frozen. No headline rate is computed. Sections of this manuscript that would report accuracy, calibration, or runtime stay `RESULT_PENDING`.

## What this introduction does not claim

It does not claim a safety improvement, a coverage, or a ranking against other tools.
