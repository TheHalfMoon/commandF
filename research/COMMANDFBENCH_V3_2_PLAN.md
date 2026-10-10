# CommandFBench — V3.2 Plan

Status: **RESEARCH_PLANNING / NOT FROZEN / NO RESULT CLAIMED**

This plan **builds on** the existing research records. It does not replace them:

- `BENCHMARK_ADMISSION_LIFECYCLE.md`
- `CANDIDATE_CORPUS.md` (3 items, digest `e92d734a…`)
- `CASE_CLASS_MATRIX.md`
- `LABEL_AUTHORITY.md`
- `SPLIT_POLICY.md` (sp-1)
- `STATISTICAL_ANALYSIS_PLAN.md` (sap-1)
- `PROTOCOL_FREEZE.md` (`NOT_FROZEN`)

Any change to the split or analysis rules requires a new version (sp-2 or sap-2) while every result is still `RESULT_PENDING`.

## 1. Why the current state is inadequate

These shortcomings are taken from the records themselves:

- Three items, all from one HAPI repository. The result is one leakage group, so sp-1 refuses to assign a split, and sap-1 intervals are `NOT_APPLICABLE`.
- 14 of the 17 case classes in `CASE_CLASS_MATRIX.md` are `blocked`. The other 3 have only `low` admission probability.
- There is no executed baseline and no frozen protocol.

The binding constraint is **independently labeled, rights-qualified, multi-source items**. It is not tooling.

## 2. Corpus families and their unblock path

| Family | Source (see the source matrix) | Label authority | Rights path | Unblocks case classes |
| --- | --- | --- | --- | --- |
| F1 Official reference tests | `FHIR/fhir-test-cases` 1.8.0 (Apache-2.0) | Published test expectations, authored outside commandF | Pin the release | Malformed evidence, FHIRPath, correct abstention |
| F2 Historical real-IG deltas | Official-registry package pairs acquired through the existing spec 043/045 paths. Each package's own `package.json` `license` must permit research reuse. | Two independent interoperability engineers plus an adjudicator. `DISAGREEMENT` is retained. | A per-package `license` field. HL7 International packages are expected to declare CC0-1.0; **this must be verified per package** and is not assumed. | FHIR package/IG evolution, profiles, extensions, SearchParameters |
| F3 Publisher diversity | At least 4 publishers or realms, for example HL7 International (UV), a US realm, an AU realm, and a European realm | As F2 | As F2. Exclude any package without an explicit reuse-permitting license. | Removes the single-leakage-group block. Gives external validity. |
| F4 Synthetic consumer breakage | Generator committed to the repository. It applies declared mutations to a consumer contract and pairs each with a producer change. | The **generator's declared ground truth** is the label. This is a *constructed* label: it is reported separately and never pooled with F2 for the primary endpoint. | First-party | Consumer dependencies, irrelevant and adversarial mutations |
| F5 Terminology and FHIRPath | ValueSet/CodeSystem changes **only from packages whose content license permits it**. SNOMED CT, LOINC, and similar content stay excluded unless separately licensed. FHIRPath cases from F1. | F1 expectations; F2 process | T rights are recorded per item | Terminology, FHIRPath |
| F6 Malformed, malicious, unavailable, and conflicting evidence | First-party fuzz-minimized regressions (AF-02 corpus), injected oracle unavailability, and contradictory evidence fixtures | Specification-defined expected outcome, for example `INDETERMINATE` or `CONFLICTING_EVIDENCE` | First-party | Conflicting evidence, malformed evidence |
| F7 Oracle disagreement | HL7 validator 6.10.2 compared with an independent engine (Firely .NET or FHIR Candle), on F1/F2 inputs | Each oracle's own published output. Disagreement is the label. Shared-code groups are recorded. | Per-tool license | Oracle disagreement |
| F8 Unsupported versions | R4B/R5 package pairs | The expected `UNSUPPORTED` follows from the declared coverage matrix | As F2 | Unsupported surface detection |
| F9 Counterfactual groups | For each F2 item, a material-edit variant and an irrelevant-edit variant built by a committed generator | Generator ground truth, reported separately | First-party | Counterfactual sensitivity, irrelevant-edit stability (G46) |
| F10 Model-advisor cases | The text-change corpus and the injection families from the D1 plan, §6 and §9 | Two-labeler process | As F2 | Advisor failure and injection |
| F11 Negative controls | Version pairs with **identical** package bytes, and pairs that differ only in non-semantic metadata | Byte identity | As F2 | False-block baseline |

**No patient data.** Synthea output is not used in repository fixtures until terminology rights are cleared. If it is ever used, generation happens locally on the user's side.

## 3. Labeling protocol

The protocol amends nothing in `LABEL_AUTHORITY.md`.

1. **Labelers.** Labelers are recruited from the pilot community (product plan §8). They must not be commandF authors, and they must not have seen commandF output for the item.
2. **Label schema.** For a (change, consumer contract) pair, the label is one of:
   - `BREAKS_CONSUMER`
   - `SAFE_FOR_CONSUMER`
   - `CONDITIONAL`
   - `CANNOT_DETERMINE`

   For a producer-only change, the label is one of:
   - `BREAKING`
   - `NON_BREAKING`
   - `EDITORIAL`
   - `CANNOT_DETERMINE`
3. **Rationale.** Each label records a free-text rationale and the evidence the labeler used.
4. **Agreement.** Report Cohen's κ (2 labelers) or Krippendorff's α. A third adjudicator resolves disagreements, and the original disagreement is retained.
5. **Admission.** An item is admitted only under the existing lifecycle: exact bytes, SHA-256, source identity, rights, label, a not-clinical-data statement, and a replay procedure.

## 4. Splits and leakage

- **Leakage group = (publisher, IG package name).** No package lineage appears in both development and held-out.
- Use sp-1 unchanged once there are at least 2 groups. sp-1 is deterministic: SHA-256 modulo 5, which sends about 20% of groups to held-out.
- **Temporal holdout (secondary).** Releases published after the protocol freeze date form a prospective set. Results on it are reported separately. Results on that set are also the main defense against model pretraining contamination for any model comparator.
- **No tuning on held-out.** Rule changes, policy defaults, and advisor calibration use development items only. Any rule change made after freeze invalidates the freeze. It requires a new protocol version and a fresh prospective set.

## 5. Systems compared

These match the families in `BASELINE_IDENTITIES.md`. All are frozen by digest at freeze time.

| ID | System |
| --- | --- |
| S0 | Trivial `ALLOW`-all. Lower bound on safety. |
| S1 | Trivial `BLOCK`-all / `ABSTAIN`-all. Lower bound on usefulness. |
| S2 | commandF CF-04 rules alone, with the existing `Breaking`/`Risky` → action mapping. |
| S3 | commandF `review` with the Decision Envelope (V3.2), no oracle. |
| S4 | S3 with selective oracle escalation (HL7 validator). |
| S5 | S3 with the D1 advisor (ranking only; under monotonicity the advisor cannot change the S3 action). |
| S6 | External comparator, where a reproducible one exists. |

For S6: no maintained tool known to this plan performs consumer-aware FHIR IG compatibility analysis. If none qualifies, the record says so. A comparator is not invented.

## 6. Metrics

**Primary (sap-1):** unsafe auto-allow rate on held-out, **reported with automation coverage**.

**Secondary (descriptive):**

- Breaking-change detection sensitivity: recall of `BREAKING` and `BREAKS_CONSUMER` labels in `BLOCK` ∪ `REQUEST_*` ∪ `WARN`.
- False-block rate: the share of `SAFE`/`NON_BREAKING`/`EDITORIAL` items that receive `BLOCK`.
- Decision coverage and abstention rate, split by abstention reason (`UNSUPPORTED`, `INSUFFICIENT`, `CONFLICTING`, `INDETERMINATE`).
- Risk-coverage at sap-1's fixed points 0.50, 0.80, 0.90, and 0.95, with `NOT_REACHED` reported honestly.
- Calibration (Brier and ECE), only for systems that emit probabilities (S5).
- Consumer witness correctness: for each `BREAKS_CONSUMER` item, whether the reported witness (contract dependency → element → rule) matches the labeler's rationale. Two-rater agreement on witness correctness is reported.
- Oracle escalation utility: the change in unsafe-allow and false-block from S3 to S4, against escalation cost (oracle runs, CPU-seconds).
- Counterfactual sensitivity (F9 material edits move the action in the expected direction) and irrelevant-edit stability (F9 irrelevant edits leave `did` unchanged).
- Receipt reproducibility: the fraction of receipts whose offline replay reproduces `sid` and `did` byte-for-byte on a different OS.
- Time to first actionable finding: wall-clock time from command start to the first `BLOCK`/`WARN` finding with a witness. Measured on the AF-04 reference machines.
- Resources: CPU-seconds, peak RSS, wall time, install size, and cross-platform failure rate (Windows, macOS, Linux).

**Slice reporting is mandatory.** Results are reported per artifact class, per FHIR version, per publisher, and per family. An aggregate may not hide a slice with an unsafe allow.

## 7. Statistical honesty and sample-size reality

- **Zero-event bounds.** If no unsafe allow is observed among n bound `ALLOW` items, the exact one-sided 95% upper bound on the rate is 1 − 0.05^(1/n), roughly 3/n. To claim an unsafe-allow rate below 1% with 95% confidence and zero observed failures needs **at least 299 bound `ALLOW` items in held-out**. Below 2% needs at least 149. This is arithmetic, not a target. The protocol freeze must state which bound the corpus size can support. Any claim stronger than that bound is prohibited.
- Bootstrap intervals use sap-1: 10,000 resamples over leakage groups, with the seed recorded at execution.
- **Falsification criteria, pre-registered.** The claim "commandF `review` is safe to gate releases on for R4 P0 artifacts" is **falsified** if either:
  - the held-out unsafe auto-allow upper bound exceeds the pre-registered tolerance; or
  - any `PROVEN_COMPATIBLE` item carries a `BREAKS_CONSUMER` label that adjudication confirms.

  Either outcome is published as a negative result.
- No statistical-significance or superiority claim is made unless sap-2 names a confirmatory paired comparison before results exist.

## 8. Execution sequence

| Step | Output | Gate |
| --- | --- | --- |
| B-1 | Rights survey of candidate packages: `license` field per package, recorded in `CASE_CLASS_MATRIX.md` | — |
| B-2 | Bind at least 4 publishers × at least 3 version pairs (F2/F3) by exact digest via spec 043/045 acquisition | B-1 |
| B-3 | Recruit labelers; run a label pilot on 20 development items; report κ | B-2, pilot program |
| B-4 | F4/F9 generators committed with tests; F6 fixtures from the AF-02 corpus | Phase 0 (generators touch no authority path) |
| B-5 | sp-1 assignment computed; held-out manifest bound; `PROTOCOL_FREEZE.md` frozen | B-3, at least 2 leakage groups |
| B-6 | S0–S4 executed once on held-out; results to `RESULTS_LEDGER.md`, including nulls | S3 shipped (Phase 3) |
| B-7 | S5 (D1) only per the D1 plan's go/no-go protocol | D1-0…D1-3 |

## 9. Threats to validity

These extend `THREATS_TO_VALIDITY.md`:

- **Labeler expertise bias** toward US realm conventions. Mitigated by F3 diversity.
- **Constructed-label circularity.** F4/F9 generator labels are written by commandF authors, so they are reported separately and never pooled into the primary endpoint.
- **Oracle non-independence.** HAPI and the HL7 validator share `org.hl7.fhir.core`, so their agreement counts as one group.
- **Survivorship.** Only published releases are visible, and withdrawn ballots are missing.
- **Model contamination** for S5 and any hosted comparator. Mitigated by the prospective temporal holdout.
