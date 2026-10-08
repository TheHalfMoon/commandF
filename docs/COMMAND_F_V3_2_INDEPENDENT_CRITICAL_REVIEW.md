# commandF V3.2 — Critical Review (Adversarial Self-Challenge)

Status: **PLANNING_CANDIDATE / AUTHOR SELF-CRITIQUE / NOT AN INDEPENDENT VERIFICATION GATE**

**Disclosure.** This review was written by the same AI agent that drafted the other nine V3.2 documents. It is an adversarial self-challenge, not independent review. Repository governance requires independent review (OCR, Jev where authorized, and a human Code Owner). This document is not a substitute for any of them.

## 1. The strongest reasons V3.2 could fail

| # | Failure mode | Likelihood | Why it is plausible | Mitigation in the plan | Residual concern |
| --- | --- | --- | --- | --- | --- |
| R1 | **Governance overhead keeps outrunning product.** | High | 22 CF-17 grains shipped without a user surface. All ten of the most recent `main` merges (PRs #175–#184) were edits to manuscript text that state no measured result. The V3.2 plan itself adds ten documents. | G52 rule: every grain names a user surface. The `review` journey comes first. One grain per user-visible change. | The plan cannot enforce discipline; only the founder can. **This is the dominant risk.** |
| R2 | **Consumer contracts never get written by users.** | High | Engineers rarely maintain machine-readable consumer declarations. | `contract init` from CapabilityStatements, ViewDefinitions, and FHIRPath. An honest `NO_PROTECTED_CONSUMERS` result. | CapabilityStatements in practice often over-declare or under-declare. Witness quality is unproven. |
| R3 | **`INSUFFICIENT`/`UNSUPPORTED` floods make `review` useless.** | Medium–High | A strict fail-closed design plus R4-only support plus missing contracts can mean most results end up as exit 4. | Coverage-matrix work in P2, sliced metrics, and pilot measurement of the abstention share. | No target abstention rate can be set before pilot data exists. |
| R4 | **The independent label supply is the true bottleneck for CommandFBench.** | High | It needs volunteer engineers, and zero-failure bounds need at least 299 `ALLOW` items for a 1% claim. | Bench plan §7 states the arithmetic honestly. Claims are bounded by what the corpus size supports. | Without funding for labelers, the strongest claim may stay "descriptive only" for a long time. |
| R5 | **D1 adds no measurable value.** | Medium–High | DAL's own study saw a model equal to its control and degenerate abstention. The vendor claims are GPU-only. The use case is narrow. | B1/B2 heuristics as baselines, DAL F1–F5 checks, an explicit NO-GO path, and a model-free core. | None. A NO-GO is an acceptable outcome. |
| R6 | **The LFM threshold makes any model distribution commercially toxic for the target buyers.** | High for enterprise | Hospitals, payers, and large vendors usually exceed USD 10M in revenue. | Opt-in, user-acquired pack only. FD-7. | Even an opt-in pack may confuse enterprise users. Legal review is needed. |
| R7 | **The llama.cpp `/v1/systemone` surface is immature.** | Medium | d1-3B support exists only on master after v0.6.0, and the hardening PR #30027 was closed unmerged. | Pin by commit, client-side limits, and boundary fuzzing (I8). | The upstream API could change. Re-qualify on every runtime pin. |
| R8 | **The amendment-record bootstrap is rejected or misused.** | Medium | It needs one founder override, and the two-PR friction may tempt a standing bypass. | Narrow FD-1 wording, exact SHAs, post-merge proof, and FD-2. | If FD-1 is declined, every dependency and workflow update stays in limbo. |
| R9 | **Oracle drift** (HL7 core 7.0.0 moved to an R6 internal basis). | Medium | Divergence counts may shift with any repin. | FD-4 deferral and full requalification. | 6.10.2 ages. Staying pinned becomes its own risk (security fixes in the JVM dependency chain). |
| R10 | **Studio scope creep becomes a second engine.** | Medium | UI teams tend to add "quick" logic. | S0 is a static file. A contract test requires CLI/UI output parity. | Requires review discipline. |

## 2. Adversarial checklist

| Question | Finding | Owner or answer |
| --- | --- | --- |
| What is missing? | (1) A **cost model** for founder time: the plan has no capacity estimate. (2) **Localization** is mentioned (RTL readiness) but not planned. (3) **Security disclosure / incident process** for a released binary: `SECURITY.md` exists, but release-time key compromise handling is not planned. (4) **Telemetry-free usage insight**: how the founder learns about adoption without telemetry. | (1) Founder. (2) Product plan follow-up grain after S1. (3) AF-03 Spec Kit must include key-compromise and revocation. (4) Pilot program and voluntary issue templates only. |
| What is duplicated? | CF-17 canonical emitters (G50). Two migration kits would duplicate, so V3.2 amends spec 018 instead. Sentrdel's canonicalizer is rejected for import to avoid a second canonical form. | Covered |
| What is impossible to prove? | "No break for all consumers": consumers outside the declared protected set are unknowable. Clinical safety, which commandF never claims. Semantic equivalence across standards. Completeness of the public registry history, which is undefined (spec 048). | All are stated as non-claims |
| What depends on unlicensed data? | `FHIR/ig-registry` (no LICENSE file) → FD-5. SQL-on-FHIR (HL7 contribution grant, not a public license per `CASE_CLASS_MATRIX.md`). Terminology content in Synthea output and in some IG packages. NPHIES (unknown rights). Firely SDK and fhirpath.js show NOASSERTION. | Each is gated in the source matrix. None is redistributed. |
| What cannot run offline? | `pkg resolve` (network by design; offline after lock). The registry catalog. Jev (hosted). An Inferno run needs a server under test. Everything in the `review` journey runs offline once inputs are locked. | Stated |
| What breaks on Windows CPU-only? | **Observed:** the `MAX_PATH` failure in the nested verifier build (audit §2). **Likely:** path canonicalization and file locking (Defender scans during atomic renames), Java discovery for the oracle, long-path package cache entries, and D1 RAM on 8 GB machines. | AF-03 W1. D1 platform W1 budget. |
| What is overengineered? | (1) The CF-17 per-document "machine bytes" grains, already built. (2) Three identities may be more than users need. (3) The full aggregation lattice before any multi-consumer user exists. | (2) and (3): keep them, because they are cheap in code and costly to retrofit, but expose only `did` and the verdict in default CLI output. |
| What could make commandF unsafe or misleading? | (1) An exit code 0 given while a protected consumer is missing from the set; mitigated by printing the protected-set digest and count in every human summary. (2) R4B/R5 inputs silently processed as R4; mitigated by P3's explicit `UNSUPPORTED`, but **this is true today** in shipped commands. (3) A model rank presented next to the verdict being read as the verdict. (4) Users trusting `LOCKED_DIGEST_MATCH` as publisher authenticity; it is trust-on-first-use only. | (2) deserves an interim warning in the current CLI. It is noted for the founder and not implemented here, because this is a planning-only assignment. |
| What would make an interoperability engineer stop using it? | Noise, Java as a hard requirement, slow start, hand-written contracts, unexplained unsupported states, and network calls in "offline" mode. | Product plan §8 watch-list. Pilot measurement. |
| Which claims lack representative tests? | Every compatibility classification lacks *independent* label tests. Current tests are author-written fixtures. There is no evidence for any version other than R4. There is no Windows/macOS CI. There is no performance evidence. Witness correctness is unmeasured. | Bench plan. AF-03. AF-04. |

## 3. Explicit unresolved concerns

1. **Whether the founder wants a product or a research artifact first.** The research manuscript (`research/manuscript/*`) is being written ahead of any measured result. V3.2 recommends product-first (the `review` journey) with research measurement in parallel. That priority call belongs to the founder.
2. **License for commandF itself (FD-6).** Without it, no external pilot participant can legally run a build. This blocks the pilot program.
3. **Whether the AF-02 gate's authority-path list is right.** Treating `Cargo.lock` as immutable authority is defensible for supply-chain integrity, but it may be disproportionate. FD-1 should decide deliberately.
4. **Spec 018's stale assertions.** They will be reconciled in M2, but until then any agent reading spec 018 alone will be misled.
5. **The audit was performed at README and module depth for the founder repositories,** and with targeted reads, not a line-by-line review, of commandF's 19.6k lines. Claims about code are limited to what was grepped, built, and tested, and are cited.

## 4. Tooling evidence for this planning pass (factual)

| Tool | Used? | Result |
| --- | --- | --- |
| `cargo test --workspace --all-features --locked` (Windows) | Yes | 143 passed, 1 failed (`MAX_PATH`). The failure passed from a short path. |
| `gh` live repository, ruleset, PR, and issue reads | Yes | Cited throughout |
| Alibaba OpenCodeReview (`ocr`) | Available locally. See the PR description for whether it was run and its result. | Markdown is `unsupported_ext` under the existing governance record, so it does not review prose. |
| Jev | The CLI is available locally, but it requires a TypeSafe API key (hosted, paid). | Not run. `BLOCKED_COST_NOT_AUTHORIZED`. Not a PASS. |
| Graft, pstack | Graft is installed locally but is not wired into this repository (PR #169 is open). pstack is not found. | Not used. |
| Hosted reviewers (CodeRabbit, Cubic, Qodo) | May run automatically on the PR | Not qualification evidence under current governance |
