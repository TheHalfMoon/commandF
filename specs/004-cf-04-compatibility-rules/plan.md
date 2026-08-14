# CF-04 Implementation Plan

Status: Approved for implementation

## Architecture

CF-04 remains inside the existing `commandf-pkg` crate and `commandf` CLI. No new workspace crate is required.

The classifier consumes the in-memory `StructuralDiffReport` produced by CF-03. It does not reopen archives or duplicate structural parsing. This keeps CF-03 as the single authority for deterministic artifact matching and structural facts.

## Public model

Add a versioned compatibility report with:

- schema version;
- ruleset id (`cf04-rules-v1`);
- package name and before/after package evidence copied from CF-03;
- ordered compatibility findings.

Each finding contains:

- stable `rule_id`;
- `severity` (`BREAKING`, `RISKY`, `ADDITIVE`);
- `direction` (`producer`, `consumer`);
- source structural change kind;
- resource key and filenames;
- optional view, element id, and field;
- normalized before/after evidence;
- deterministic explanatory message.

## Classifier boundary

Expose:

```text
classify_structural_diff(&StructuralDiffReport) -> Result<CompatibilityReport, CompatibilityError>
```

The classifier accepts only CF-03 schema v1. Unsupported input schema fails closed.

The engine must exhaustively handle every `StructuralChangeKind`. `ElementFieldChanged` and `StructureFieldChanged` are further dispatched by the CF-03 field name. An unknown future structural field returns `CompatibilityError::UnsupportedStructuralField` rather than receiving a fallback severity.

## Directional rules

The engine uses contract variance rather than a single undirected severity:

- producer compatibility detects tightening of what may be emitted;
- consumer compatibility detects widening of what may be received.

Cardinality and finite max/maxLength changes are therefore classified by comparing ordered bounds.

Type arrays are already normalized by CF-03. Convert their entries to canonical JSON set identities and compare set inclusion:

- after strict subset of before -> producer BREAKING;
- after strict superset of before -> consumer BREAKING;
- incomparable replacement -> BREAKING in both directions.

## Binding rules

Parse binding strength when available using the R4 order:

```text
example < preferred < extensible < required
```

Strengthening is producer-breaking; weakening is consumer-breaking.

If the ValueSet canonical changes, emit RISKY findings for both directions because terminology set inclusion is CF-07 work. If binding payload changes in another unmodeled way, emit RISKY in both directions.

## Constraint rules

For `constraint` arrays, inspect error/warning severity when additions/removals are unambiguous:

- added error constraint -> producer BREAKING;
- removed error constraint -> consumer BREAKING;
- warning-only additions/removals -> directional RISKY;
- non-identical replacement when implication is unknown -> RISKY both.

No FHIRPath equivalence solver is introduced.

## Slicing rules

Read `slicing.rules` using `open < openAtEnd < closed` as increasing restrictiveness and inspect `ordered` when present.

- increasing restrictiveness or false->true ordering -> producer BREAKING;
- relaxing the rule or true->false ordering -> consumer RISKY;
- discriminator or other slicing payload changes -> RISKY both.

## Must Support and modifiers

Any `mustSupport` change is RISKY both because R4 defines support obligations as contextual and distinct from cardinality.

Newly setting `isModifier=true` emits consumer BREAKING plus producer RISKY. Other modifier-semantic changes emit RISKY both.

## Residual structural facts

To avoid silent gaps:

- resource byte changes remain explicit RISKY findings;
- package filename/id/version changes are RISKY;
- resource additions are ADDITIVE;
- resource removals and type/FHIR-version identity changes are BREAKING where the contract implication is direct;
- fields whose semantic subset relation is not established by CF-04 are RISKY rather than guessed safe.

## Duplicate snapshot/differential facts

Before classification, a differential element-field change that is byte-for-byte equivalent to a snapshot element-field change for the same resource/element/field/kind/before/after tuple may be skipped. Snapshot evidence wins. View add/remove changes are never deduplicated this way.

## CLI

Add:

```text
commandf classify <package-name> \
  --before-lock before.lock --before-cache before-cache \
  --after-lock after.lock --after-cache after-cache \
  --format json
```

Refactor the existing Diff CLI loading into one internal helper returning `StructuralDiffReport`, then use it for both `diff` and `classify`.

CF-04 does not set policy exit codes based on findings. That quality-gate behavior belongs to CF-05.

## Validation

Synthetic tests cover:

- empty report -> empty findings and byte-stable JSON;
- min/max/maxLength directionality;
- type narrowing/widening/incomparable replacement;
- fixed and pattern add/remove/change;
- binding strength direction and ValueSet-change RISKY behavior;
- constraint addition/removal/change behavior;
- Must Support and modifier behavior;
- slicing restriction/relaxation;
- resource/view/element and residual byte rules;
- unknown future field fail-closed behavior;
- snapshot/differential deduplication;
- CLI help, missing-path errors, corrupted cache behavior, and offline classify success.

Real CI extends the current official R4 smoke by running `commandf classify` against the same independently resolved before/after states and requiring an empty finding list for self-equivalent content.

## Deferred semantics

Do not add terminology expansion, validator judgments, SARIF, source mapping, baselines, graph impact, mapping execution, or AI authority in CF-04.
