# CF-03 Convergence Review

Status: Implementation and convergence complete — founder review candidate
Date: 2026-08-14

## Scope result

CF-03 remains inside its authorized boundary: deterministic structural facts between two explicitly supplied locked/cache states of the same FHIR package.

It does **not** classify a change as breaking, risky, additive, safe, producer-facing, or consumer-facing. It does not run the FHIR Validator, generate snapshots, execute terminology, compute ecosystem blast radius, execute mappings, or introduce AI authority. Those remain CF-04 or later work.

## Architecture reconciliation

CF-03 required no new workspace crate. The final design uses `commandf-pkg` for CF-01 lock/cache authority, the shared bounded package-root scanner, CF-02 inspection, and CF-03 structural diff; `commandf` provides the explicit two-state `diff` CLI. The dependency graph remains unchanged and CI uses the committed `Cargo.lock` with `--locked`.

## Contract achieved

```text
commandf diff <package-name> \
  --before-lock <path> --before-cache <path> \
  --after-lock <path> --after-cache <path> \
  --format json
```

The command:

- validates one FHIR package name and requires exactly one selected version in each supplied lockfile;
- performs no package acquisition;
- verifies each selected CF-01 cache object and independently rechecks each archive SHA-256 before structural parsing;
- rebuilds inventory from package-root FHIR JSON rather than trusting `.index.json`;
- matches unique canonical resources by canonical URL;
- upgrades a multiplicity group to exact `url|version` matching and fails closed when a member lacks a usable version;
- matches non-canonical resources by unique `resourceType/id`, then filename;
- fails closed on ambiguity and duplicate package-root filenames;
- emits deterministic resource add/remove and filename/version/resourceType/id/byte-hash facts;
- compares StructureDefinition metadata plus snapshot/differential views separately and matches elements by exact `ElementDefinition.id`;
- validates CF-03-owned structural field shapes before normalization without becoming a general FHIR validator;
- preserves valid FHIR primitive `_field` metadata, including extension-only primitive metadata present in official R4 artifacts;
- normalizes only known set-like arrays and preserves ordering where semantics may depend on it, including `extension[]`;
- serializes stable ordered JSON without severity or compatibility labels.

## Safety and fail-closed behavior

CF-03 reuses the CF-02 archive limits: 512 MiB decompressed TAR traversal, 50,000 archive entries, and 64 MiB per package-root resource.

Additionally, cache/archive digest mismatches, malformed JSON, canonical multiplicity without usable versions, ambiguous non-canonical keys, duplicate filenames, malformed/duplicate element ids, malformed interpreted structural shapes, and internal inventory disagreement fail explicitly rather than being guessed or silently normalized.

## Deterministic synthetic and CLI evidence

Tests prove:

1. self-diff is empty and byte-stable;
2. unique canonical URLs remain matched across canonical-version changes;
3. multiplicity groups use exact `url|version` and missing versions fail closed;
4. duplicate non-canonical ids and duplicate archive filenames fail closed;
5. view/element additions and removals are explicit;
6. cardinality, type, slicing, binding, fixed/pattern, boolean, metadata, and related structural changes are emitted;
7. malformed CF-03-owned field shapes fail with `InvalidStructuralField`;
8. valid primitive `_code` metadata remains accepted while malformed/empty metadata is rejected;
9. editorial-only element fields are excluded from structural-field changes;
10. representation, condition, contextInvariant, constraint, and type/profile/targetProfile/aggregation reorderings are normalized;
11. `extension[]` order is preserved as structural;
12. CLI required arguments, absent packages, corrupted before/after caches, and successful offline diff are covered.

## Green implementation evidence

Exact implementation evidence head `a8fec0f2073abe1b9660457a493db6c2e437a2ec` passed GitHub Actions pull-request run `31761209998`:

- Format — PASS;
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` — PASS;
- `cargo test --locked --workspace --all-features` — PASS;
- first real registry resolve/verify for `hl7.fhir.r4.core@4.0.1` — PASS;
- real `commandf inspect hl7.fhir.r4.core@4.0.1 --format json` smoke — PASS;
- second independent registry resolve/verify for the same exact package into a distinct lock/cache state — PASS;
- real `commandf diff hl7.fhir.r4.core ... --format json` self-diff — PASS with an empty change list.

The second resolution is intentional. CF-03 acceptance requires reproducibility across two independently resolved explicit states; copying the first state would weaken that gate. Registry availability is therefore an accepted external dependency of this real-package reproducibility smoke.

Documentation-only convergence commits follow the implementation evidence head. The exact final documentation head must pass the same full CI gate; that exact-head result is recorded in PR metadata to avoid a self-referential documentation commit chain.

## Reviewer evidence

### CodeRabbit

A manual review while PR #4 remained Draft returned three actionable threads:

1. **CI repeatability — Minor.** The reviewer proposed copying the first resolved state to avoid a second registry dependency. **Not adopted by contract:** CF-03 acceptance explicitly requires two independently resolved states. CodeRabbit subsequently withdrew the finding after that acceptance requirement was clarified.
2. **Malformed structural shapes — Major.** **Fixed:** pre-normalization CF-03-owned shape validation plus regression coverage was added while preserving valid FHIR primitive `_field` metadata required by official R4 artifacts.
3. **Global `extension[]` sorting — Minor.** **Not adopted by design:** CF-03 preserves extension order because unordered semantics are profile/slicing-context dependent. A regression pins that behavior, and CodeRabbit withdrew the finding.

All three CodeRabbit review threads are resolved. The general CodeRabbit docstring-coverage warning is non-blocking for this slice and is not represented as a CF-03 correctness PASS requirement.

### Qodo

`/review` was requested. No Qodo review result or finding has been observed at convergence time.

Disposition: **NO EVIDENCE / NOT RETURNED**. No Qodo PASS is claimed.

### Cubic

Cubic has maintained an automated PR summary describing the CF-03 diff. No separate substantive Cubic blocking finding has been observed in the PR conversation at convergence time. The generated summary is not treated as certification.

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
