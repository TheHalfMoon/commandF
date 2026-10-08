# commandF V3.2 — Product and UX Plan

Status: **PLANNING_CANDIDATE / PROPOSED PRODUCT DESIGN / NO USER RESEARCH PERFORMED YET**

Everything below is a hypothesis about users until the pilot program (§8) produces evidence. No persona has been validated with real interoperability engineers.

## 1. Personas (hypotheses to validate)

| Persona | Job to be done | Current pain (hypothesis) | Where commandF must fit |
| --- | --- | --- | --- |
| **P1 IG author** (HL7 work group, national program, vendor) | Publish a new IG version without breaking implementers unknowingly. | IG Publisher QA reports validation errors, not compatibility with the previous release or with implementers. | A PR check on the FSH or IG repository, plus a local CLI. |
| **P2 Interoperability engineer** (integration team at a provider, payer, or vendor) | Decide whether an upstream IG or package upgrade breaks their interfaces. | Manual reading of change logs, and diffing of StructureDefinitions by eye. | A local review against *their* declared consumer contracts, offline. |
| **P3 Release or QA lead** | Get a defensible go/no-go record. | Decisions live in email and spreadsheets. | A Decision Receipt that is verifiable later. |
| **P4 Platform or CI owner** | Gate merges with deterministic, low-noise checks. | Flaky or opaque tools get disabled. | GitHub Action + SARIF + baselines (exists: CF-05, CF-08, CF-13). |
| **P5 Healthcare-AI data engineer** (research persona) | Know whether a FHIR change invalidates a feature-extraction pipeline. | Breakage is discovered after model drift. | The `ai_feature_contract` consumer family (future research). |

## 2. The first complete product journey: `review`

This is the journey V3.2 commits to shipping before breadth. It reuses existing commands as its stages.

```text
commandf review \
  --old hl7.fhir.us.core@6.1.0 --new hl7.fhir.us.core@7.0.0 \
  --protected consumers/*.contract.json \
  --policy policy.json \
  --out review/
```

| Stage | Existing implementation | New work |
| --- | --- | --- |
| Resolve and lock both closures | `pkg resolve` and `pkg verify` (CF-01, CF-11) | Authenticity states (G54) |
| Inspect and diff | `inspect`, `diff` (CF-02, CF-03) | Change-set contract |
| Classify | `classify` (CF-04, CF-07) | Coverage matrix and an `UNSUPPORTED` witness |
| Consumer impact | `impact`, `context` (CF-11G, CF-12) | Consumer Contract v1 matching (CF-19 subset) |
| Sufficiency, policy, and decision | `gate` (CF-13) | Decision Envelope v1, aggregation |
| Outputs | `check --format sarif` (CF-05) | `envelope.json`, `receipt.json`, `report.html` (static, offline), SARIF |
| Verify later | — | `commandf verify-receipt review/receipt.json` (offline replay) |

**Exit codes.** These extend the existing exit contracts and are frozen in the owning Spec Kit.

| Code | Meaning |
| --- | --- |
| 0 | `ALLOW` |
| 1 | operational error |
| 2 | `BLOCK` |
| 3 | `WARN` |
| 4 | needs evidence: `ABSTAIN`, `REQUEST_*`, `INSUFFICIENT`, `UNSUPPORTED`, `CONFLICTING`, or `INDETERMINATE` |

Code 4 is deliberately distinct so that CI can never mistake "unknown" for "pass".

These codes are consistent with what ships today: `check` and `gate` return 1 for operational and usage errors and 2 for a policy failure (`crates/commandf-cli/src/main.rs:246-267`, `gate.rs:124`). One collision must be avoided. For other subcommands, clap's own usage-error code (2) is passed through. `review` must therefore join the `check`/`gate` special case, so that a usage error can never read as `BLOCK`.

**Consumer contract onboarding.** Most users will not have contracts on day one. The plan offers four routes:

1. `commandf contract init --from-capability-statement cs.json` extracts declared profiles, interactions, and search parameters.
2. `commandf contract init --from-view-definitions views/` extracts element paths from SQL-on-FHIR ViewDefinitions.
3. `--from-fhirpath expressions.txt` extracts paths from FHIRPath expressions.
4. With no contracts, `review` still runs. It reports producer-side findings and returns truth class `INSUFFICIENT_EVIDENCE` with reason `NO_PROTECTED_CONSUMERS`, and it says exactly how to add contracts. This is the honest default, and it avoids pretending to know the blast radius.

**Time to first actionable finding** is a tracked product metric (bench plan). The target is set from pilot baselines, not guessed.

## 3. CLI and CI workflows

| Workflow | Surface | Status |
| --- | --- | --- |
| PR check on an IG repository | `action.yml` → SARIF → annotations | Exists (CF-05/08/09). Gains envelope and receipt artifacts in P4. |
| Legacy debt | `gate` baselines and suppressions | Exists (CF-13) |
| Pre-commit / pre-push | `commandf review --staged` (FSH source) | Planned (CF-23, after the CF-21 SUSHI evidence) |
| Air-gapped review | `commandf bundle export` / `import`, then `review --offline` | Planned (G19/G35) |
| Release record | `verify-receipt` + signed release artifacts (AF-03) | Planned |

## 4. Release experience (G20/G52)

1. **v0.1.0: "existing CLI, honestly labeled".**
   - Tagged release. Windows, macOS, and Linux binaries built in CI. SHA-256 sums. Sigstore bundle and GitHub attestation. SBOM.
   - A capability table generated from the status ledger (each capability `SUPPORTED`, `EXPERIMENTAL`, or `NOT_AVAILABLE`).
   - Blocked on FD-6 (the license) and AF-03 minimal.
2. **v0.2.0:** `review` with Decision Envelope v1 (R4, P0 artifact families).
3. **v0.3.0:** Decision Receipt, `verify-receipt`, offline bundle.
4. **v0.4.0:** Studio v0 (read-only viewer).
5. The optional model pack is versioned independently and never required by any release above.

Install paths, all verifiable offline:

- a direct binary download plus `cosign verify-blob --bundle`;
- `cargo install --locked` from the tag;
- a later winget/Homebrew manifest. Package-manager publication is a separate decision.

## 5. CommandF Studio

### 5.1 Principle

Studio is a **viewer and launcher**. It reads the following JSON contracts and renders them:

- `commandf.decision-envelope/v1`;
- `commandf.decision-receipt/v1`;
- `commandf.change-set/v1`;
- `commandf.coverage-matrix/v1`;
- consumer contracts;
- SARIF.

To compute anything new, Studio invokes the same `commandf` binary. It contains no rule, diff, or classification logic. A contract test proves this by rendering identical outputs from receipts produced by the CLI.

### 5.2 Staged delivery

| Stage | Form | Why |
| --- | --- | --- |
| S0 | `report.html` produced by `review`: a single self-contained static file with no network, no external fonts, no scripts beyond inline, and data embedded. | Zero install, works in CI artifacts and air gaps, and becomes a fixture for every later UI. |
| S1 | Local Studio: `commandf studio <review-dir>` serves the S0 UI on `127.0.0.1` with a per-launch token, and adds a "re-run with…" action that calls the CLI. | Interactivity without a desktop framework decision. |
| S2 | Desktop shell, after S1 usage evidence. Options: **Tauri** (WebView2 on Windows; reuses the S0/S1 web UI) or **Slint** (native and non-WebView; precedent in founder-owned `MedScale/crates/medscale-desktop`). | Decided by an ADR measuring install size, accessibility support (screen readers), Windows WebView2 availability in locked-down hospital desktops, and code reuse. |

Core CLI and CI users never install Studio or the model pack.

### 5.3 Screens (S0/S1)

1. **Overview.**
   - Aggregate truth class and action with its exit code.
   - Completeness and the protected-consumer count.
   - Counts by class (breaking, risky, additive, unsupported, conditional, insufficient).
   - The `did`, `sid`, and `rid` identities, with copy buttons.
2. **Changes.**
   - A table filterable by class, artifact, rule, and consumer.
   - Old/new side-by-side ElementDefinition view with the changed fields highlighted.
   - A FSH source link when CF-09 mapping exists.
3. **Consumers (blast radius).**
   - A per-consumer list with its failure witnesses: contract dependency → changed element → rule.
   - Dependency graph view limited to the affected subgraph.
4. **Evidence.**
   - Each evidence item with its truth class, producer, digest, and outcome.
   - Oracle observations with agreement or divergence and the independence group.
   - Missing evidence listed with whether it can change the verdict and how to obtain it.
5. **Coverage.**
   - The support state for the artifact classes and FHIR version in this review.
   - Unsupported transitions are listed explicitly.
6. **Tests and remediation (later).**
   - Generated tests and dry-run recipe diffs (CF-15/CF-22), clearly labeled `DRY_RUN`.
7. **Advisory (only if a model pack is configured).**
   - A separate panel with the model identity.
   - Re-ranking only. A banner states that it does not affect the verdict.
8. **Export.**
   - SARIF, envelope, receipt, and bundle.
   - A "How to verify this offline" panel showing the exact `commandf verify-receipt` command.

### 5.4 Explanation, accessibility, and error recovery (G53)

- All text is rendered from versioned explanation templates keyed by `explanation_codes`, never from model output.
- WCAG 2.2 AA is the target. Required behaviors: keyboard-complete navigation, no color-only encoding (class icons plus text), screen-reader labels on graph nodes with an equivalent table view, and respect for reduced-motion settings.
- Right-to-left layout readiness is required for future localization (Arabic is a likely regional need); translation itself is a later grain.
- Every error state shows the `commandf.error/v1` code, which stages completed, and one concrete next action. Examples:
  - `ORACLE_UNAVAILABLE` → "install Java 17+ or run without `--oracle`; the result will be `INSUFFICIENT_EVIDENCE` for rules that require it".
  - `UNSUPPORTED_VERSION` → "R5 is parse-only in this version".
- First-run onboarding offers three paths: a bundled synthetic example IG pair (public, rights-clear), "compare two packages", and "add a consumer contract".

## 6. IDE and PR workflows

- **PR:** the existing Action adds an envelope summary comment generated from templates, and uploads the receipt as an artifact. The receipt **must not** depend on Actions artifact retention for authority, which is the lesson of issue #100: the receipt is self-contained and re-derivable offline from pinned inputs.
- **IDE (CF-23, later):** an LSP over the same engine for FSH files, offering diagnostics from `review --staged` and hover explanations from templates.

## 7. Product scope discipline

| Capability family (from `docs/COMMAND_F_PRODUCT_FAMILY.md`) | V3.2 classification |
| --- | --- |
| Core (pkg, diff, rules, check, gate, impact, context) | IMPLEMENTED_BUT_NOT_QUALIFIED (audit), with the first journey on top |
| Verify (receipts, replay) | PLANNING_ONLY → P4 |
| Studio | PLANNING_ONLY → P5 (S0 in P4) |
| Bench | PLANNING_ONLY → parallel research lane |
| Registry / observatory (CF-17, CF-27) | Primitives implemented; catalog BLOCKED; public observatory DEFERRED |
| Query (consumer contracts over FHIRPath, SQL-on-FHIR, CQL) | CF-19 subset in P3; CQL DEFERRED |
| Copilot (model assistance) | FUTURE_RESEARCH, D1 gate |
| Gateway (runtime interception) | OUT_OF_SCOPE for this horizon |
| Trust (signing, certificates) | AF-03 for releases; transformation certificates are FUTURE_RESEARCH |
| Cross-standard transformation (openEHR, OMOP, Loss Ledger) | FUTURE_RESEARCH, deferred until there is measured demand |

## 8. Pilot program (before any product-market-fit claim)

1. Recruit 5–8 practitioners across P1 and P2. At least two must come from outside the US realm (for example an AU or EU IG team), and one from the Saudi ecosystem if available.
2. Hold a 60-minute moderated session with the v0.2 `review` on *their* chosen public IG version pair. No PHI and no proprietary contracts leave their machine; sessions are run locally on their side.
3. Measure:
   - time to first actionable finding;
   - findings they judge wrong, which is a false-block signal;
   - findings they say are missing, which is a missed-break signal;
   - whether they would gate a release on it.
4. Run a two-week diary study with 2–3 teams using it in CI.
5. Record the results in `research/` as qualitative evidence with consent. They are not statistical claims.

"Would stop using it" triggers to watch for:

- noisy `RISKY` floods;
- slow cold start;
- having to write contracts by hand;
- opaque unsupported states;
- a Java requirement for basic use (the oracle must stay optional);
- any network call during an "offline" run.
