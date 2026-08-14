# CF-03 Convergence Review

Status: Implementation and convergence complete — founder review candidate
Date: 2026-08-14

## Scope result

CF-03 remains inside its authorized boundary: deterministic structural facts between two separately locked and cached states of the same FHIR package.

It does **not** classify a change as breaking, risky, additive, safe, producer-facing, or consumer-facing. It does not run the FHIR Validator, generate snapshots, execute terminology, compute ecosystem blast radius, execute mappings, or introduce AI authority. Those remain CF-04 or later work.

## Architecture reconciliation

CF-03 required no new workspace crate.

The final design uses:

- `commandf-pkg` for CF-01 lock/cache authority, the shared bounded package-root resource scanner, CF-02 inspection, and CF-03 deterministic structural diff;
- `commandf` for the explicit two-state `diff` CLI surface.

The CF-02 scanner was factored into one internal helper so inspect and diff consume identical package-root filtering and safety bounds. The committed dependency graph remains unchanged and CI continues to run with the existing `Cargo.lock` and `--locked`.

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
- upgrades a canonical URL group to exact `url|version` matching when multiplicity exists on either side;
- matches non-canonical resources by unique `resourceType/id`, then filename fallback;
- fails closed on ambiguous resource keys and duplicate package-root resource filenames;
- emits deterministic resource add/remove and filename/version/resourceType/id/byte-hash facts;
- compares StructureDefinition metadata plus snapshot/differential views separately;
- matches StructureDefinition elements by exact `ElementDefinition.id`;
- emits deterministic view/element additions/removals and selected structural-field changes;
- suppresses editorial-only fields from element structural-field changes;
- normalizes known set-like arrays to avoid ordering-only false deltas;
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
- ambiguous non-canonical resource keys fail rather than guessing;
- duplicate package-root filenames fail before raw structural lookup can become ambiguous;
- malformed or duplicate StructureDefinition element ids fail;
- no external-archive lookup uses a panic path.

## Deterministic synthetic evidence

CF-03 contract tests prove:

1. self-diff produces an empty change list and byte-stable JSON;
2. one unique canonical URL stays matched across canonical-version change;
3. multi-version canonical groups use exact `url|version` keys;
4. duplicate non-canonical `resourceType/id` keys fail closed;
5. duplicate package-root resource filenames fail closed;
6. cardinality, type, slicing, binding, and fixed-value changes are emitted as structural facts;
7. editorial `short` changes are excluded from element structural-field changes;
8. representation, condition, and constraint reordering is normalized;
9. type-entry ordering and nested profile/targetProfile/aggregation ordering is normalized;
10. snapshot/differential view additions and removals are explicit;
11. element additions and removals are explicit.

## Green implementation evidence

Implementation candidate `2a4ece32313ba7b92b8dde038bf5e231a12b2dff` passed GitHub Actions run `31759991866`:

- Format — PASS;
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` — PASS;
- `cargo test --locked --workspace --all-features` — PASS;
- real registry resolve/verify into independent before/after states — PASS;
- real `commandf inspect hl7.fhir.r4.core@4.0.1 --format json` smoke — PASS;
- real `commandf diff hl7.fhir.r4.core ... --format json` self-diff — PASS with an empty change list.

Documentation-only convergence commits follow that implementation candidate. The exact final PR head must also pass the same complete gate set; that exact-head result is recorded in PR metadata rather than creating an endless self-referential documentation chain.

## Reviewer evidence

### CodeRabbit

At implementation convergence, the GitHub status context named `CodeRabbit` reported success, but the CodeRabbit PR comment explicitly states that the actual review was **skipped because the pull request is Draft**. No CodeRabbit review PASS is claimed. There were no inline review threads and no submitted review object at the latest inspection.

Disposition: **STATUS SUCCESS / REVIEW SKIPPED — DRAFT**.

### Qodo

No Qodo review result or finding was present at implementation convergence. No Qodo PASS is claimed.

Disposition: **NO EVIDENCE / NOT RETURNED**.

Reviewer unavailability does not replace deterministic CI evidence and is recorded explicitly rather than silently treated as success.

## Convergence decision

**CF-03 implementation is converged and may be presented for founder review once the exact final PR head is green.**

PR #4 remains Draft and unmerged. No merge or auto-merge is authorized. CF-04 compatibility/severity behavior is not part of this PR.

## Explicit deferrals

- BREAKING/RISKY/ADDITIVE/SAFE classification;
- producer/consumer directionality and compatibility modes;
- FHIR Validator or differential-oracle judgments;
- snapshot generation;
- terminology expansion or semantic binding comparison;
- ecosystem dependency graph and blast radius;
- mapping execution, CSIR, semantic loss, and round-trip proof;
- AI/agent runtime features.
