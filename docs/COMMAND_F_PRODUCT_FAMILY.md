# commandF Product Family

Status: long-term product-plan coverage, not immediate build authorization.

The names below preserve the product family discussed during commandF discovery. They are capability groupings, not separate repositories or services that must exist now. The V2 execution sequence remains authoritative and may ship several of these capabilities from one binary/product before any packaging split is justified.

## commandF Core

Universal interoperability foundation: package/artifact identity, parsers, normalized analysis models, deterministic rules, mapping analysis, semantic/loss evidence, and reproducible transformation/verification primitives.

## commandF Studio

Human-centered visual workspace for mapping, debugging, semantic diff, blast radius, evidence inspection, terminology decisions, review, and safe recipe authoring.

## commandF Registry

Versioned/content-addressed profiles, schemas, mappings, terminology metadata, recipes, rules, benchmark packages, compatibility evidence, and later signed package/certificate bundles.

## commandF Verify

Conformance, structural, terminology, semantic, round-trip, consumer, differential-oracle, and policy verification with evidence-backed findings and quality gates.

## commandF Gateway

Future real-time healthcare interoperability delivery plane for bounded adapters, HL7v2/FHIR/vendor feeds, eventing, CDC, replay, policy enforcement, and transformation evidence. It should reuse mature transport/runtime components rather than invent a broker.

## commandF Query

Cross-model query analysis and later compilation where evidence justifies it: FHIR Search/FHIRPath, SQL-on-FHIR, CQL, openEHR AQL, OMOP SQL, and related analytics interfaces. A universal Clinical Query IR remains a research/future hypothesis rather than a current prerequisite.

## commandF Bench

Reproducible interoperability benchmark and research harness for semantic conservation, round-trip behavior, terminology fidelity, differential server/tool behavior, mappings, and transformation evidence.

## commandF Copilot

Optional AI-assisted mapping/review/test/remediation plane. AI may propose, explain, retrieve, and triage; deterministic compilers, terminology services, validators, policies, tests, and human authority remain the proof/gating layer.

## commandF Trust

Interoperability quality, data-quality, provenance, compatibility, consumer-readiness, and future AI-readiness assessment. Trust outputs must remain multidimensional and evidence-backed rather than one unexplained score.

## Shared product experiences retained

Across the family, the plan retains:

- Context Graph
- Semantic Diff
- Blast Radius
- Review / Deep Review
- Transformation Risk Analysis
- Rules and Quality Gates
- Verified AutoFix / Recipe Engine
- TestGen
- Transformation Stacks
- Certification Queue
- Interoperability Inbox
- Living Interoperability Wiki
- Continuous Standards/Vendor Drift
- Consumer Compatibility Matrix
- `can-i-certify`
- Loss Ledger
- Transformation Evidence Graph
- Transformation / Conformance Certificates

Positioning retained from discovery: **review healthcare interoperability like code**, while proving more than code review can prove about healthcare standards, terminology, mappings, consumers, and semantic loss.
