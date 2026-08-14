# CF-03 Convergence Review

Status: Implementation and convergence complete — founder review candidate
Date: 2026-08-14

## Scope result

CF-03 remains inside its authorized boundary: deterministic structural facts between two explicitly supplied locked/cache states of the same FHIR package.

It does **not** classify a change as breaking, risky, additive, safe, producer-facing, or consumer-facing. It does not run the FHIR Validator, generate snapshots, execute terminology, compute ecosystem blast radius, execute mappings, or introduce AI authority. Those remain CF-04 or later work.

## Architecture reconciliation

CF-03 required no new workspace crate.

The final design uses:

- `commandf-pkg` for CF-01 lock/cache authority, the shared bounded package-root resource scanner, CF-02 inspection, and CF-03 deterministic structural diff;
- `commandf` for the explicit two-state `diff` CLI surface.

The CF-02 scanner is one internal helper so inspect and diff consume identical package-root filtering and safety bounds. The committed dependency graph remains unchanged and CI runs with the existing `Cargo.lock` and `--locked`.

## Contract achieved

```text
commandf diff <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --format json
```

now:

- validates one FHIR package name;
- requires exactly one selected version of that package in each explicit lockfile;
- performs no package acquisition;
- verifies each selected CF-01 cache object before reading;
- independently rechecks each archive SHA-256 before structural parsing;
- rebuilds inventory from package-root FHIR JSON rather than trusting `.index.json`;
- matches unique canonical resources by canonical URL;
- upgrades a canonical URL group to exact `url|version` matching when multiplicity exists and fails closed when a group member lacks a usable version;
- matches non-canonical resources by unique `resourceType/id`, then filename fallback;
- fails closed on ambiguous resource keys and duplicate package-root filenames;
- emits deterministic resource add/remove and filename/version/resourceType/id/byte-hash facts;
- compares StructureDefinition metadata plus snapshot/differential views separately;
- matches StructureDefinition elements by exact `ElementDefinition.id`;
- validates interpreted structural field shapes before normalization;
- preserves valid FHIR primitive `_field` metadata forms, including extension-only primitive metadata used by official R4 artifacts;
- emits deterministic view/element additions/removals and selected structural-field changes;
- suppresses editorial-only fields from element structural-field changes;
- normalizes only known set-like arrays and preserves ordering where semantics may depend on it, including `extension[]` in CF-03;
- serializes stable ordered JSON without severity or compatibility labels.

## Safety and fail-closed behavior

CF-03 reuses the CF-02 archive safety boundary:

- decompressed TAR traversal: 512 MiB maximum;
- archive entries: 50,000 maximum;
- one package-root resource: 64 MiB maximum.

Additionally:

- cache digest mismatch fails before archive comparison;
- archive digest mismatch fails before structural parsing;
- malformed resource or structural JSON fails explicitly;
- duplicate canonical identities continue to fail under CF-02 inspection rules;
- canonical multiplicity without usable versions fails rather than mixing bare and qualified identities;
- ambiguous non-canonical resource keys fail rather than guessing;
- duplicate package-root filenames fail before raw structural lookup can become ambiguous;
- malformed or duplicate StructureDefinition element ids fail;
- malformed CF-03-owned structural field shapes fail before normalization;
- primitive metadata is accepted only when structurally meaningful rather than as an empty escape hatch;
- no external-archive lookup uses a panic path.

## Deterministic synthetic and CLI evidence

CF-03 tests prove:

1. self-diff produces an empty change list and byte-stable JSON;
2. one unique canonical URL stays matched across canonical-version change;
3. multi-version canonical groups use exact `url|version` keys;
4. canonical multiplicity without usable versions fails closed;
5. duplicate non-canonical `resourceType/id` keys fail closed;
6. duplicate package-root resource filenames fail closed;
7. cardinality, type, slicing, binding, fixed-value, and selected metadata changes are emitted as structural facts;
8. malformed interpreted field shapes fail with `InvalidStructuralField` instead of becoming normal deltas;
9. valid primitive `_code` metadata without a primitive value remains accepted while empty/malformed metadata is rejected;
10. editorial `short` changes are excluded from element structural-field changes;
11. representation, condition, contextInvariant, constraint, and type/profile/targetProfile/aggregation reordering is normalized;
12. `extension[]` ordering remains structural rather than being globally sorted;
13. snapshot/differential view additions and removals are explicit;
14. element additions and removals are explicit;
15. CLI usage errors, missing packages, corrupted before/after caches, and offline successful diff behavior are covered.

## Green implementation evidence

Exact implementation head `0b5bb366e0cd3f8e2198f4e3ee3eb0841b618fdc` passed GitHub Actions run `31761159980`:

- Format — PASS;
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` — PASS;
- `cargo test --locked --workspace --all-features` — PASS;
- real registry resolve/verify for `hl7.fhir.r4.core@4.0.1` — PASS;
- real `commandf inspect hl7.fhir.r4.core@4.0.1 --format json` smoke — PASS;
- copied explicit after-state verification — PASS;
- real `commandf diff hl7.fhir.r4.core ... --format json` self-diff — PASS with an empty change list.

The real smoke performs one pinned registry resolution, then copies the verified content-addressed state into explicit after paths and verifies the copied after state before diff. This isolates CF-03 from a second registry call while still exercising the real two-path CLI and both cache-verification boundaries.

Documentation-only convergence commits follow that implementation head. The exact final PR docs head must also pass the same complete gate set; that exact-head run is recorded in PR metadata rather than creating a self-referential documentation commit chain.

## Reviewer evidence

### CodeRabbit

A manual review was triggered while PR #4 remained Draft. The actual review returned three actionable threads:

1. **CI repeatability — Minor:** second remote resolution made the self-diff smoke unnecessarily dependent on a second registry call. **Fixed** by one pinned resolve followed by copied explicit after-state paths and independent after-cache verification.
2. **Malformed structural shapes — Major:** interpreted fields could be normalized without sufficient type/shape checks. **Fixed** with pre-normalization structural validation plus regression coverage, while retaining valid FHIR primitive `_field` metadata support required by official R4 artifacts.
3. **Global `extension[]` sorting — Minor:** reviewer proposed treating all extension arrays as set-like. **Not applied by design.** CF-03 preserves extension order because unordered semantics are profile/slicing-context dependent; global sorting could erase a real structural change. CodeRabbit withdrew the finding after the contract was clarified.

All three CodeRabbit threads are resolved. CodeRabbit status on the implementation head is **SUCCESS**.

### Qodo

`/review` was requested. No Qodo review result or finding was returned at convergence time.

Disposition: **NO EVIDENCE / NOT RETURNED**. No Qodo PASS is claimed.

## Convergence decision

**CF-03 implementation is converged and is a founder-review candidate, subject only to the exact final documentation head passing the complete CI gate.**

PR #4 remains Draft and unmerged. No merge or auto-merge is authorized. CF-04 compatibility/severity behavior is not part of this PR.

## Explicit deferrals

- BREAKING/RISKY/ADDITIVE/SAFE classification;
- producer/consumer directionality and compatibility modes;
- FHIR Validator or differential-oracle judgments;
- snapshot generation;
- terminology expansion or semantic binding comparison;
- profile-aware extension ordering semantics;
- ecosystem dependency graph and blast radius;
- mapping execution, CSIR, semantic loss, and round-trip proof;
- AI/agent runtime features.
