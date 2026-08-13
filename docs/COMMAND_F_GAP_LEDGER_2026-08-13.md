# commandF Interoperability Gap Ledger — 2026-08-13

Status: plan input / hypothesis ledger. These gaps motivate commandF features and research; they are not all claims of universal impossibility.

1. FHIR-valid does not necessarily mean clinically or semantically equivalent.
2. No generally adopted Semantic Loss metric exists for heterogeneous healthcare transformations.
3. Bidirectional mappings do not automatically guarantee round-trip preservation.
4. FHIR profile proliferation creates review and compatibility burden.
5. Implementation Guides can contain overlapping or incompatible constraints.
6. There is no universal safe profile harmonizer.
7. Extensions can duplicate or localize semantics in incompatible ways.
8. Terminology interoperability remains incomplete even when structures align.
9. Wearable, nursing, genomics, patient-reported, and local concepts can expose terminology gaps.
10. FHIR search behavior can vary across implementations.
11. CapabilityStatement claims can differ from observed server behavior.
12. There is no single universal empirical server compatibility matrix.
13. Cross-version FHIR conversion can be structurally valid while semantically risky.
14. FHIR conversion operations are not a universal solution for all source/target models.
15. There is no universal mapping registry spanning healthcare models and local/vendor mappings.
16. FHIR↔openEHR mappings remain incomplete and domain-dependent.
17. FHIR↔OMOP mappings remain incomplete and purpose-dependent.
18. There is no established neutral typed clinical IR accepted across FHIR, openEHR, OMOP, HL7v2, CDA, and DICOM.
19. Semantic ambiguity can exist inside otherwise valid FHIR representations.
20. Clinical validation practices across IGs and organizations are inconsistent.
21. FHIR GraphQL remains a secondary/evolving interoperability surface rather than a universal query solution.
22. SQL-on-FHIR and related analytics interfaces continue to evolve.
23. There is no universal cross-model clinical query language with proven semantic equivalence.
24. Real-time eventing/subscription interoperability remains implementation- and version-sensitive.
25. Bulk/analytics workloads do not map cleanly to ordinary FHIR REST interaction patterns.
26. FHIR REST is not optimized as a general large-scale analytics engine.
27. Consent/Permission and organizational authorization semantics remain difficult to make portable across deployments.
28. SMART on FHIR is not a universal policy language for all healthcare authorization decisions.
29. Patient identity matching has no single universally correct algorithm.
30. Generic provenance does not by itself prove field-level transformation reasoning.
31. There is no generally adopted machine-verifiable healthcare Transformation Certificate proving what was checked and preserved/lost.
32. Cross-system concurrency and race conditions can invalidate otherwise correct-looking interoperability workflows.
33. AI-generated FHIR or mappings are unreliable if treated as semantic authority without deterministic verification.
34. AI reasoning directly over large FHIR/openEHR/OMOP graphs can be inefficient and representation-sensitive.
35. There is no comprehensive benchmark that jointly measures cross-standard semantic preservation, loss, reversibility, terminology fidelity, and reproducibility.

## commandF response map

- Semantic correctness gap → Semantic Diff, deterministic validators/oracles, later Semantic Conservation evidence.
- Loss gap → explicit loss vocabulary, Loss Ledger, research measurement method.
- Round-trip gap → reversibility classes and round-trip verifier/benchmarks.
- Profile/IG conflict gap → normalized profile graph and conflict witnesses.
- Terminology gap → terminology evidence and Gap Registry.
- Server behavior gap → Compatibility Lab and claimed-vs-observed evidence.
- Version gap → version-aware FHIR diff/conversion evidence using official and independent oracles.
- Mapping fragmentation → import/analyze FML/StructureMap, FHIRconnect, Whistle, Liquid, FHIRPath mapping, OMOCL, and other mapping forms.
- Cross-model gap → Mapping Analysis IR first; any broader CSIR only after evidence from implemented dialects.
- Query fragmentation → query-impact analysis first; Clinical Query IR remains research/future.
- Provenance gap → Transformation Evidence Graph plus OpenLineage/in-toto/SLSA/Sigstore-compatible evidence plumbing.
- Certificate gap → reproducible Transformation/Conformance Certificates.
- Identity/policy gap → adapters to mature identity/policy systems plus explicit evidence-bearing policy.
- Runtime/streaming gap → gateway/CDC/event adapters using mature runtime components rather than a new message broker.
- AI reliability gap → AI proposes/explains; deterministic systems validate, verify, and gate.
- Benchmark gap → commandF Bench.
