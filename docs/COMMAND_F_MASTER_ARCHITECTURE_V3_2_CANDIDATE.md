# commandF Master Architecture V3.2 Candidate

Status: **PLANNING_CANDIDATE / PROPOSED DESIGN / NOT EXECUTION AUTHORITY**

Observed canonical main: `f82565cca917d119e1c774b2c470e2ac20e0d6dd` (tree `7b8a13949d8e52e2b0bdb952bca010d83450aeb5`).

V2 remains execution authority. V3 and V3.1 remain planning candidates. V3.2 amends them; it does not replace them. Where V3.2 is silent, V3 and V3.1 apply. Nothing in this document is implemented unless §11 says so with a file path.

## 1. North star

> **What will this healthcare interoperability change break, which protected consumers will be affected, what is proven, what remains uncertain, and what evidence makes the release decision independently reproducible?**

commandF is a **change-assurance tool for interoperability engineers**. It is not:

- a FHIR server;
- a terminology server;
- an integration engine;
- an agent platform;
- a chatbot.

## 2. What V3.2 changes relative to V3/V3.1

| # | Change | Why |
| --- | --- | --- |
| A1 | **One first complete journey before breadth:** `review` = compare two IG package versions against declared protected consumers, then emit a Decision Envelope, a Decision Receipt, and SARIF, all offline. Every new slice must shorten or strengthen this journey until it ships. | The audit found that about 37% of the code (CF-17) has no user surface (G52). |
| A2 | Three separate identities: semantic decision, source input, and execution receipt (§5). | V3.1 had one receipt identity, which conflates "same decision" with "same run". |
| A3 | Explicit aggregation algebra across many changes and consumers (§6). | V3.1 defined a per-decision envelope but no join rule. Averaging or "majority" would hide unsafe allows. |
| A4 | The model advisor is an **out-of-process, loopback-only, optional evidence provider** that sits strictly below policy (§8). | The D1 evaluation requirement. It keeps the core free of models. |
| A5 | Package-source authenticity and namespace-confusion defense (§7) become a first-class contract (G54). | Official registries provide digests but not publisher signatures. 17 of 78 official feeds are plain HTTP. |
| A6 | Studio and IDE are **projections of the same JSON contracts**. They contain no semantics (§9). | V3 G16, made concrete. |
| A7 | A healthcare AI data-contract consumer is **one more consumer-contract family**, not a new plane (§10). It is classified as FUTURE_RESEARCH. | Bounded scope. |
| A8 | FHIR R4 (4.0.1) is the only `SUPPORTED` version for the first journey. R4B and R5 start as `PARSE_ONLY` / `UNSUPPORTED` with explicit witnesses. | The audit found no version dispatch in the source. |

## 3. Planes and ownership

The V3 seven planes are kept. Ownership, mapped to code that exists today:

```text
 Plane 0  Artifact truth      CF-01/02/03, CF-11      crates/commandf-pkg/src/{resolver,lock,cache,archive,artifact_*}.rs   [exists]
 Plane 1  Context graph       CF-11G, CF-12, CF-17    context*.rs, impact*.rs, ecosystem_*.rs                               [exists; CF-17 not surfaced]
 Plane 2  Compatibility       CF-04, CF-07, CF-18     compatibility*.rs, terminology in check/*                             [rules exist; coverage/envelope planned]
 Plane 3  Consumer contracts  CF-19                   —                                                                     [planned]
 Plane 4  Review/UX           CF-05/08/09/13, CF-23   check*.rs, gate*.rs, cli/main.rs, action.yml                          [exists; app layer planned]
 Plane 5  Lab/evidence        CF-06, CF-20, CF-21     oracle_*.rs, tools/hl7-oracle                                         [oracle exists]
 Plane 6  Semantic research   CF-16, CF-26            —                                                                     [research]
 Advisory (optional)          CF-24 + D1 study        separate process `commandf-advisor`                                   [planned, gated]
```

## 4. Trust boundaries

```text
┌───────────────────────────── user machine (offline-capable) ─────────────────────────────┐
│                                                                                          │
│  [untrusted inputs]  package archives · lockfiles · consumer-contract files · IG text   │
│        │ bounded reads (G49 primitives) · digest verify · no extraction to disk          │
│        ▼                                                                                 │
│  ┌──────────── commandF core (Rust, no network unless `pkg resolve`) ────────────┐       │
│  │ artifact truth → diff → rules → consumer witnesses → sufficiency → policy      │       │
│  │                     │                          ▲                               │       │
│  │                     │ typed request (JSON)     │ typed advisory (schema-valid) │       │
│  │                     ▼                          │                               │       │
│  │   B1: process boundary, loopback only, token, no network egress, timeouts     │       │
│  └─────────────────────┼──────────────────────────┼──────────────────────────────┘       │
│                        ▼                          │                                      │
│        [optional] commandf-advisor (llama.cpp server + pinned GGUF; user-installed pack) │
│                                                                                          │
│  B2: JVM process boundary → HL7 validator (pinned jar) — existing CF-06 boundary         │
│  B3: Studio / IDE read JSON files or the local API; they never run rules                 │
└──────────────────────────────────────────────────────────────────────────────────────────┘
   network (only when explicitly invoked): packages.fhir.org / packages2.fhir.org (spec 043)
   never: hosted inference, telemetry, a commandF cloud
```

Trust boundary rules:

1. B1 failures, including absence, timeout, invalid schema, or crash, map to `advisory.state = UNAVAILABLE | FAILED | INVALID`. They never change truth class or action.
2. No component may fall back to a remote model, a remote oracle, or a remote terminology server unless the user has an explicit per-invocation flag. Such evidence is classed `REMOTE_OBSERVATION` and is never `PROVEN_*`.

## 5. Identity model (amends V3.1 §9, owned by G43)

| Identity | Formed from | Must change when | Must NOT change when |
| --- | --- | --- | --- |
| **Source-input identity** `sid` | SHA-256 over the canonical list of exact input bytes: package archive digests, lockfile digest, consumer-contract file digests, and policy file digest. | Any input byte changes. | Never invariant to edits. |
| **Semantic decision identity** `did` | SHA-256 over the canonical JSON of: envelope schema version; normalized change set (rule-relevant fields only); protected consumer set (contract IDs + versions); rule IDs + rule versions; policy identity; engine semantic version; required-evidence outcomes (semantic content, not timings); and model-advisory identity **only if** policy consumed it. | Any material change, rule, policy, consumer, or required evidence changes. | Irrelevant edits as declared by the change-space model: formatting, key order, narrative text classed non-semantic, timestamps, host paths. |
| **Execution receipt identity** `rid` | SHA-256 over the full receipt: `sid`, `did`, stage statuses, tool and oracle binary digests, platform tuple, and start/finish times. | Every run. | — |

Replay contract:

- An offline replay of a receipt bundle must reproduce `sid` and `did` byte-for-byte. It produces a new `rid`.
- A replay whose `did` differs is a defect. It is reported as `REPLAY_DIVERGENCE`.

Irrelevant-edit stability (G46) is tested on `did`.

Cache authority:

- A cache entry is keyed by `(sid, engine identity, rule-set identity, policy identity)`.
- Cache hits are re-verified through `read_verified` before use.
- A cache entry is never evidence of anything by itself.

## 6. Aggregation across changes and consumers (amends V3.1 §4, owned by G41)

A decision is computed per `(change cluster, protected consumer)` pair. An aggregate decision uses lattice joins, never averages, counts, or majorities.

**Truth-class join.** The first matching row wins.

| Order | Condition on the set of pair decisions | Aggregate truth class |
| --- | --- | --- |
| 1 | Any `PROVEN_BREAKING` | `PROVEN_BREAKING` (all other pair results still listed) |
| 2 | Any `CONFLICTING_EVIDENCE` | `CONFLICTING_EVIDENCE` |
| 3 | Any `INDETERMINATE` | `INDETERMINATE` |
| 4 | Any `INSUFFICIENT_EVIDENCE` | `INSUFFICIENT_EVIDENCE` |
| 5 | Any `UNSUPPORTED` | `UNSUPPORTED` |
| 6 | Any `RISK_DETECTED` | `RISK_DETECTED` |
| 7 | Any `CONDITIONALLY_COMPATIBLE` | `CONDITIONALLY_COMPATIBLE`, with the union of conditions |
| 8 | All `PROVEN_COMPATIBLE` and the protected set is non-empty | `PROVEN_COMPATIBLE` |
| 9 | Empty protected set | `INSUFFICIENT_EVIDENCE`, reason `NO_PROTECTED_CONSUMERS` |

**Completeness join.** Take the minimum, with `COMPLETE` > `MATERIALLY_INCOMPLETE` > `UNKNOWN_DUE_TO_ERROR`.

**Action join.** The most restrictive of `BLOCK` > `REQUEST_HUMAN` > `{REQUEST_ORACLE, REQUEST_RUNTIME_PROBE}` (union) > `ABSTAIN` > `WARN` > `ALLOW`. `ALLOW` is possible only when every pair is `ALLOW`.

**Witness preservation.** Each pair keeps its own failure witness: consumer contract ID, dependency record, changed element, rule ID, and evidence references. The aggregate never replaces witnesses with a count.

**Cross-change interaction.** Two changes that are individually compatible may combine into a break (for example, a slice discriminator change plus a cardinality change). Clusters are formed from changes that touch the same element path or its slicing ancestry. If a rule cannot evaluate the combination, the cluster is `UNSUPPORTED`, reason `COMBINED_CHANGE_NOT_MODELED`. This is never inferred as compatible.

## 7. Contracts

Each contract below is a **proposed** schema. The owning Spec Kit freezes the exact fields. Every contract must have:

- `schema` (a string ID) and `schema_version`;
- a JSON Schema under `schemas/`, a directory that does not exist yet;
- an invalid-state test matrix;
- round-trip tests;
- a migration or rejection rule for older versions (G30, G35).

| Contract | Schema ID | Owner | Key content (beyond V3.1) | Reuses existing code |
| --- | --- | --- | --- | --- |
| Change identity | `commandf.change-set/v1` | CF-18 | Old and new package `name@version#sha256`, artifact canonical URL + version + resource digest, normalized element address (incl. slice name), and change kind from the CF-03 `StructuralChangeKind`. | `artifact_diff_model.rs` |
| Canonical source identity | (inside change-set) | CF-01/CF-09 | Package archive digest, and an optional FSH source location (`source-map`). | `lock.rs`, CF-09 |
| Coverage matrix | `commandf.coverage-matrix/v1` | CF-18 (G02, G08, G09) | Row = (FHIR version, artifact class, field path, change kind). State ∈ `CLASSIFIED(rule_ids)`, `IGNORED_NON_SEMANTIC(rationale)`, `VERSION_INAPPLICABLE`, `UNSUPPORTED_FAIL_CLOSED`. A generated artifact, checked in CI for every rule-set change. | `compatibility_rule_coverage.rs` seeds it |
| Consumer Contract | `commandf.consumer-contract/v1` | CF-19 (G44) | V3.1 §8 fields, plus `extraction_method` ∈ `DECLARED_FILE`, `CAPABILITY_STATEMENT`, `VIEW_DEFINITION`, `FHIRPATH_EXPRESSION_SET`, `SEARCH_USAGE_LOG_AGGREGATE`. Plus `ai_feature_contract` (§10, research). | `context_model.rs` edge kinds |
| Evidence item | `commandf.evidence/v1` | CF-18/CF-21 | `truth_class` (V3 eight classes), `producer`, `digest`, `required_by[]`, `outcome` ∈ `PRESENT`, `ABSENT`, `UNAVAILABLE`, `CONTRADICTED`, `STALE_IDENTITY`. | — |
| Sufficiency | (inside envelope) | CF-18 (G42) | Per decision class: required evidence classes, which are present, and `can_change_verdict: bool` per missing item. | — |
| Decision Envelope | `commandf.decision-envelope/v1` | CF-18 (G41) | V3.1 §5 fields, plus `did`, `pair_decisions[]`, `aggregate`, and `explanation_codes[]` (§9). | — |
| Policy | `commandf.policy/v1` | CF-13/CF-18 (G32) | Maps (truth, completeness, sufficiency, oracle state) → action. Ships with a conservative default: no `ALLOW` without `PROVEN_COMPATIBLE` + `COMPLETE`. | `gate_model.rs` |
| Oracle observation | `commandf.oracle-observation/v1` | CF-20 (G47) | V3.1 §7 states, plus `independence_group`. HAPI and the HL7 validator share `org.hl7.fhir.core`, so their agreement counts as one group. | `oracle_model.rs` (AF-02 authority path: an edit needs Phase 0) |
| Model advisory | `commandf.model-advisory/v1` | CF-24 + D1 (G48) | `state` ∈ `NOT_CONFIGURED`, `UNAVAILABLE`, `FAILED`, `INVALID`, `ABSTAINED`, `ANSWERED`. `question_id`, `answer_type` ∈ `noul`, `choice`, `score`. `probabilities`. `model_identity` {gguf sha256, quantization, tokenizer digest, runtime commit, calibration-map digest}. `input_digest`. `consumed_by_policy: bool`. | — |
| Decision Receipt | `commandf.decision-receipt/v1` | CF-21 (G43) | V3.1 §9, plus `sid`, `did`, `rid`, the platform tuple, and `replay_bundle_digest`. | — |
| Error / partial result | `commandf.error/v1` | CF-18/CF-23 (G33) | Codes: `INPUT_INVALID`, `INPUT_TOO_LARGE`, `UNSUPPORTED_VERSION`, `UNSUPPORTED_ARTIFACT`, `UNCLASSIFIED_TRANSITION`, `POLICY_INVALID`, `ORACLE_UNAVAILABLE`, `ORACLE_DISAGREEMENT`, `RESOURCE_LIMIT`, `RIGHTS_DENIED`, `SOURCE_AUTHENTICITY_FAILED`, `ADVISOR_FAILED`, `INTERNAL_INVARIANT`. Every partial report lists `stages_completed[]` and `stages_not_run[]`. | Existing typed error enums |

**Contradictory or unavailable evidence.**

- Two qualified sources that disagree produce `CONFLICTING_EVIDENCE`.
- Required evidence that is unavailable produces `INSUFFICIENT_EVIDENCE`. If it was required and not run, it is reported as `REQUIRED_NOT_RUN`.
- No precedence rule may silently pick a winner unless it is declared in the policy and recorded in the envelope.

### Package source authenticity (G54)

FHIR NPM registries publish tarballs and version metadata, but no publisher signatures. commandF therefore records authenticity as an evidence state, not a boolean:

| State | Meaning |
| --- | --- |
| `LOCKED_DIGEST_MATCH` | The archive bytes match the lockfile digest. This is trust-on-first-use. |
| `MULTI_SOURCE_AGREEMENT` | packages.fhir.org and packages2.fhir.org serve identical bytes. |
| `NAMESPACE_RESTRICTION_SATISFIED` | The package name matches an official `package-restrictions` mask and its declared feed. This requires the FD-5 rights clearance for the data. |
| `NAMESPACE_RESTRICTION_VIOLATED` | Fail closed: `SOURCE_AUTHENTICITY_FAILED`. |
| `UNAUTHENTICATED_TRANSPORT` | The feed or package was obtained over plain HTTP. It is never `PROVEN_*` evidence on its own. |
| `LOCAL_MIRROR_UNVERIFIED` | A local mirror without a prior lock. |

Policy decides which states are acceptable. The default is that `LOCKED_DIGEST_MATCH` is required, and `NAMESPACE_RESTRICTION_VIOLATED` always blocks.

## 8. Model advisory lane (summary; full plan in `COMMAND_F_V3_2_D1_QUALIFICATION_PLAN.md`)

- **Position.** Below policy. It is an evidence item of class `PATTERN_OR_RESEARCH_INPUT` until qualified. After qualification it becomes `ADVISORY_MODEL_OBSERVATION`, a new class that is never upgradable.
- **Allowed effects.**
  - Reorder human-review queues.
  - Propose `REQUEST_HUMAN` for free-text definition changes that deterministic rules classify only as "text changed".
  - Propose which oracle to run.
- **Prohibited effects.** It may never:
  - produce or upgrade a `PROVEN_*` class;
  - change `BLOCK` to anything else;
  - suppress a finding;
  - satisfy a sufficiency requirement;
  - change `did` unless policy consumed it, and policy may only make the decision *more* restrictive.
- **Monotonicity invariant.** For every input, `action(with advisory)` is greater than or equal to `action(without advisory)` in the restrictiveness order of §6. This is property-tested.
- **Process boundary.** `commandf-advisor` runs as a child process bound to `127.0.0.1` on an ephemeral port with a per-run token. It is started only when the user configures a model pack.

## 9. Explanations without model text (G53)

Every finding and decision carries `explanation_codes[]`, each with a stable code, parameters, and evidence references. Human text is rendered from **versioned templates** shipped with the engine, in English first; the localization catalog is a later grain. Studio, SARIF messages, and PR annotations all render from the same templates.

- Model output may appear only in a visually distinct "advisory" region, labeled with the model identity.
- Model output is never copied into SARIF `message.text` or into the receipt's decision section.

## 10. Healthcare AI data-contract readiness (FUTURE_RESEARCH)

This is modeled as a consumer-contract family `ai_feature_contract`. It declares:

- the element paths read by the feature extraction;
- the required terminologies and value sets, with versions;
- the expected cardinalities and units;
- the FHIR version;
- the provenance requirements.

The existing rules then evaluate whether a change can invalidate the **data interface**. Examples:

- a removed or renamed path;
- binding narrowed or broadened;
- a code system version change;
- a units change;
- a version transition with an `UNSUPPORTED` witness.

Explicit non-claim: commandF never asserts anything about the downstream model's clinical performance. The envelope reason code is `DATA_INTERFACE_ONLY`.

## 11. What is implemented today versus proposed

**Implemented** (see the audit, §3):

- Plane 0 and Plane 1 primitives;
- 70 rule IDs with Breaking, Risky, and Additive classifications;
- SARIF, gate, impact, and context;
- the HL7 oracle at 6.10.2;
- cache verified reads.

**Proposed only:** every contract in §7, the identities in §5, the aggregation in §6, the advisory lane in §8, explanation templates, and the coverage matrix.

## 12. Non-negotiables carried forward

- V2 oracle rule.
- No patient data in repository tests.
- No cloud dependency.
- AI never has semantic authority.
- Fail closed on unclassified states.
- No new crate unless a shipped command uses it (the advisor process is a separate binary only when the D1 gate passes).
- Merge commits only.
- No history rewrite.
