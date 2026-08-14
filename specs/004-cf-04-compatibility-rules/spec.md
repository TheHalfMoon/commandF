# CF-04 Specification — Directional Compatibility Rules

Status: Approved for implementation

## Purpose

CF-04 turns CF-03 structural facts into deterministic compatibility findings. It answers **who is affected and how strongly** when a published FHIR conformance package changes.

CF-04 is deliberately rule-based. It does not ask a model to decide compatibility and it does not replace the FHIR Validator, IG Publisher, or later differential-oracle work.

## User-visible command

```text
commandf classify <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --format json
```

The command reuses the exact CF-03 two-state loading and structural diff path. It performs no package acquisition.

## Direction semantics

CF-04 uses two explicit directions:

- **producer** — asks whether output that was valid under the before contract can still be produced under the after contract. Tightening constraints primarily affects this direction.
- **consumer** — asks whether an implementation built to consume all before-valid output can safely consume all after-valid output. Expanding what producers may send primarily affects this direction.

A single structural change may produce findings in one or both directions.

## Severity semantics

CF-04 v1 emits only:

- `BREAKING` — the rule corpus can establish an incompatibility from deterministic structural evidence;
- `RISKY` — compatibility may be affected but CF-04 cannot prove a safe subset/superset relation without deferred semantics or contextual knowledge;
- `ADDITIVE` — the change adds a contract artifact without invalidating an existing contract under the rule represented by that finding.

There is no AI-authored severity and no implicit `SAFE` state.

## Versioned rule corpus

The public ruleset identifier is `cf04-rules-v1`.

Every public rule has a stable rule id, rationale, positive tests, counterexample tests where applicable, deterministic output, and an explicit direction.

### Cardinality

- increasing `min` is `BREAKING` for producers;
- decreasing `min` is `BREAKING` for consumers because after-valid output may omit data previously guaranteed;
- decreasing `max` is `BREAKING` for producers;
- increasing `max` is `BREAKING` for consumers because after-valid output may contain more repetitions than before allowed;
- the same directional rule applies to numeric `maxLength` changes.

### Type choices

CF-03-normalized type entries are treated as deterministic allowed-choice sets:

- narrowing the allowed type set is `BREAKING` for producers;
- widening the allowed type set is `BREAKING` for consumers;
- replacing incomparable type/profile/targetProfile choices is `BREAKING` in both directions.

### Fixed values and patterns

- adding a fixed value is `BREAKING` for producers;
- removing a fixed value is `BREAKING` for consumers;
- changing one fixed value to another is `BREAKING` in both directions;
- adding/removing a pattern follows the same directional constraint logic;
- replacing one non-identical pattern with another is `RISKY` in both directions because generic pattern subset proof is out of scope.

### Terminology bindings

FHIR R4 defines binding strengths `example`, `preferred`, `extensible`, and `required` with progressively stronger conformance obligations.

- strengthening the binding is `BREAKING` for producers;
- weakening the binding is `BREAKING` for consumers;
- changing a bound ValueSet without an independently proven set relation is `RISKY` in both directions.

ValueSet expansion/subset semantics are explicitly deferred to CF-07.

### Constraints

- adding an error-level invariant is `BREAKING` for producers;
- removing an error-level invariant is `BREAKING` for consumers;
- warning-level invariant additions/removals are `RISKY` in the affected direction;
- changing an existing invariant expression is `RISKY` in both directions unless a later oracle proves equivalence or implication.

### Must Support and modifiers

FHIR defines `mustSupport` separately from cardinality and leaves its concrete support obligation context-dependent. Any `mustSupport` change is therefore `RISKY` in both directions in CF-04 v1.

A change that newly marks an element as a modifier is `BREAKING` for consumers because unrecognized modifier semantics cannot be safely ignored. Other modifier-semantic changes are at least `RISKY` in both directions.

### Slicing

- constraining slicing from a more-open rule toward `closed`, or changing unordered to ordered, is `BREAKING` for producers;
- relaxing slicing is `RISKY` for consumers because after-valid instances may contain slice structures not previously allowed;
- discriminator changes are `RISKY` in both directions unless a later semantic layer proves equivalence.

### Resource and representation changes

- resource addition is `ADDITIVE` in both directions;
- resource removal is `BREAKING` in both directions at the package-contract level;
- resource type or FHIR-version identity changes are `BREAKING` in both directions;
- filename/id/version-only changes, view removal, structural metadata changes whose subset relation is not provable, and residual byte-only changes are `RISKY` rather than silently ignored;
- element additions/removals without enough CF-03 payload to establish cardinality/modifier semantics are conservatively `RISKY`, except snapshot element removal which is `BREAKING` for producers.

## Determinism and coverage

CF-04 must classify or explicitly fail every CF-03 structural change kind and every CF-03 interpreted structural field. A future CF-03 field added without a CF-04 rule must fail closed rather than silently disappear.

Equivalent snapshot/differential facts may be deduplicated deterministically so one effective contract change does not produce duplicate user findings.

Identical structural diff input must produce byte-identical compatibility JSON.

A no-op CF-03 report must produce an empty finding list.

## Standards rationale

CF-04 v1 is grounded in published FHIR R4 profiling semantics, including:

- cardinality restriction rules: `https://hl7.org/fhir/R4/profiling.html#cardinality`;
- binding-strength constraints and Must Support: `https://hl7.org/fhir/R4/profiling.html`;
- binding strength conformance meanings: `https://hl7.org/fhir/R4/terminologies.html`;
- modifier-extension safety semantics: `https://hl7.org/fhir/R4/extensibility.html`.

These sources constrain the deterministic rule rationale but do not imply that HL7 itself publishes commandF severity labels.

## Acceptance

- public report schema and ruleset id are stable and deterministic;
- producer/consumer direction is explicit on every finding;
- cardinality, maxLength, type-set, fixed/pattern, binding-strength, constraint, mustSupport, modifier, slicing, resource, view, and residual-change rules have regression coverage;
- all CF-03 structural fields are covered or fail closed;
- self-classification of a real independently resolved `hl7.fhir.r4.core@4.0.1` state produces zero findings;
- CI uses the committed `Cargo.lock` and `--locked`;
- no PHI fixtures;
- no terminology expansion, SARIF, oracle divergence, ecosystem graph, mapping runtime, or AI authority.

## Non-goals

- SARIF or quality-gate exit semantics — CF-05;
- HL7/FHIR Validator differential oracle — CF-06;
- ValueSet/CodeSystem membership and binding-set inclusion — CF-07;
- GitHub annotations — CF-08;
- FSH source mapping — CF-09;
- ecosystem blast radius — CF-11/12;
- transformation loss or mapping execution;
- AI-authored compatibility judgments.
