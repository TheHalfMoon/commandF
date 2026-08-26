# commandF Assurance Program — 2026-08-26

Status: PLANNING_CANDIDATE

## Purpose

commandF has now shipped a substantial deterministic interoperability-analysis spine through canonical CF-13. The next risk is no longer "does the repository have tests?" The repository already has strong exact-head CI, deterministic proof workflows, fail-closed tests, real FHIR smoke, and independent review. The gap is that the quality of the development system itself is not yet uniformly measured or enforced.

This program makes commandF's development and release evidence as explicit as its product evidence.

It does **not** renumber or replace CF-14, CF-15, or CF-16. Those product identities remain governed by `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`.

## Live baseline at program creation

```text
canonical main: 8a45857bf31c4acae57fdfb1e3cdde3d0f7d0361
canonical tree: ffaa14fdc7a738a771ac872e566ad1609eedf2cc
CF-13: CLOSED_CANONICAL
open product blocker: CF-10 remains separately blocked by the CF-06 production-oracle contract
```

The live repository audit established the following concrete gaps.

### P0 — source and CI trust

1. `main` has no branch protection and no active repository ruleset. Required status checks are therefore policy-by-convention rather than repository-enforced source-control policy.
2. The general `.github/workflows/ci.yml` still uses mutable `actions/checkout@v4`, mutable `dtolnay/rust-toolchain@1.97.1`, and `ubuntu-latest`.
3. Some later proof workflows already use stronger immutable action SHAs, credentialless checkout, fixed runner labels, and digest-pinned containers. The repository therefore has two assurance levels rather than one uniform baseline.
4. There is no repository-wide machine gate proving that every external GitHub Action reference is a full commit SHA and every checkout disables credential persistence unless an explicit reviewed exception exists.
5. There is no GitHub Actions static-analysis gate such as `zizmor` and no OpenSSF Scorecard evidence retained by commandF.
6. Dependency policy is implicit. There is no checked-in `cargo-deny` policy for licenses, sources, advisories, duplicate/banned crates, or explicit reviewed exceptions.
7. There is no dedicated RustSec audit gate against `Cargo.lock`.

### P1 — test adequacy, not merely test count

1. No fuzz workspace or fuzz targets exist. This is material because commandF processes untrusted package archives, JSON, lockfiles, reports, source maps, and graph evidence.
2. There is no structure-aware or differential fuzzing against parser/validator invariants.
3. There is no mutation-testing program to measure whether tests actually kill plausible logic defects.
4. There is no coverage evidence or per-change coverage floor. Coverage will be treated as a diagnostic and floor, not as a correctness score.
5. There is no explicit flaky-test detector. A test that only passes after retry must be visible as a defect, because reproducibility is a commandF contract.
6. There is no property-test layer for algebraic invariants such as canonicalization idempotence, serialization round trips, set/order semantics, graph-path determinism, and baseline/suppression equivalence.

### P1 — portability and compatibility

1. Canonical CI executes on Linux only. There is no Windows/macOS workspace gate despite filesystem, path, process, shell, and atomic-file behavior being platform-sensitive.
2. The workspace declares Rust `1.97.1`, but there is no explicit MSRV job that proves the declared toolchain remains sufficient separately from the primary CI image.
3. There is no automated Rust public-API/SemVer compatibility guard for `commandf-pkg`.
4. There is no release artifact verification pipeline, SBOM, build provenance attestation, or signature bundle.

### P2 — performance and reliability evidence

1. There are no checked-in micro/macro benchmark baselines or regression budgets for package scanning, graph construction, impact traversal, fingerprinting, and gate validation.
2. Live registry smoke is valuable external-contract evidence but is mixed into the general CI critical path. Deterministic core qualification and external availability sentinels should have distinct semantics and failure reporting.
3. There is no retained resource-envelope evidence for CPU time, peak memory, archive traversal, large graph size, or pathological input families beyond existing local bounds tests.

### P2 — standards/version readiness and developer experience

1. The current product core is strongly R4-proven. The repository does not yet expose a release-awareness matrix for R4, R4B, R5, and draft R6 surfaces.
2. HL7's publication directory records R5 `5.0.0` as the current published release and R6 `6.0.0-ballot5` as a draft ballot published 2026-07-17. Draft R6 evidence must never be presented as equivalent to a published release.
3. `README.md` still describes CF-01 as the first slice and omits canonical capabilities such as `context`, `impact`, and `gate`.

## Program architecture

The assurance program uses `AF-*` identities. `AF` means **Assurance Foundation** and is not a product-slice renumbering.

### AF-01 — Trusted Development Baseline

Goal: make source-control, workflow, dependency, and exact-head evidence policy mechanically enforceable.

Ships independently executable verification results:

- repository workflow-trust audit;
- dependency/license/source/advisory audit;
- GitHub Actions static-analysis result;
- retained exact-head assurance evidence;
- repository ruleset/branch-policy evidence.

AF-01 is the immediate unit authorized by this program once its planning package is reviewed and merged.

### AF-02 — Adversarial Test Strength

Goal: measure whether commandF fails safely under generated, mutated, malformed, and adversarial inputs.

Planned surfaces:

- `cargo-fuzz` structure-aware targets for archive/JSON/report/lock/graph boundaries;
- differential fuzzing where an independent oracle or equivalent reference model exists;
- property tests for deterministic/canonical invariants;
- `cargo-mutants` targeted mutation score with reviewed exclusions;
- `cargo-llvm-cov` coverage diagnostics/floors;
- `cargo-nextest` CI profile with flaky retries reported as failure;
- minimized regression corpus promoted from every discovered crash or invariant violation.

AF-02 must not weaken current `cargo test` authority. It adds evidence; it does not replace existing canonical gates.

### AF-03 — Portability and Release Evidence

Goal: prove that the shipped CLI/library and release artifacts are portable, versioned, and traceable.

Planned surfaces:

- Linux + Windows + macOS workspace qualification for platform-relevant code;
- explicit MSRV job for Rust `1.97.1` until deliberately changed;
- public-API/SemVer compatibility checks for the library surface;
- deterministic release artifact inventory;
- SBOM;
- SLSA-compatible build provenance;
- GitHub artifact attestation and/or Sigstore bundle verification where appropriate;
- release verification instructions that work without trusting a mutable tag alone.

### AF-04 — Performance and Reliability Evidence

Goal: prevent correctness-preserving changes from causing unbounded performance/resource regressions and separate deterministic qualification from external-service availability.

Planned surfaces:

- benchmark corpus and stable scenario IDs;
- wall-time/CPU/memory/size metrics with environment identity;
- regression budgets based on measured baselines, not guessed percentages;
- large-package and large-graph stress scenarios;
- external registry/oracle sentinel classification distinct from deterministic local gates;
- trend artifacts suitable for later commandF Bench reuse.

AF-04 is internal assurance evidence. It is not the full future `commandF Bench` product.

## Ordering and product-roadmap relationship

```text
CF-13 CLOSED_CANONICAL
        |
        +--> AF-01 Trusted Development Baseline
        |       |
        |       +--> AF-02 Adversarial Test Strength
        |       +--> AF-03 Portability / Release Evidence
        |       +--> AF-04 Performance / Reliability Evidence
        |
        +--> CF-14 planning may proceed from canonical roadmap authority
```

Execution rule:

- AF-01 must close before any new product implementation is merged after CF-13.
- CF-14 planning may proceed in parallel because planning does not create runtime authority.
- Any CF-14 parser/instance-data boundary introduced later must enter AF-02 fuzz/property coverage before that boundary can close canonically.
- AF-03 must close before commandF makes a stable public release claim.
- AF-04 must close before commandF makes quantitative performance/scalability claims.

## Assurance principles

1. **No vanity metric.** Line coverage, mutation score, fuzz time, Scorecard score, and benchmark speed are evidence dimensions, not a single trust score.
2. **Exact-head evidence.** Assurance results bind to exact source/tree/tool identities just as CF proof workflows do.
3. **Fail closed.** A missing required assurance result is not silently treated as pass.
4. **No hidden retries.** Flaky tests remain defects; retry may diagnose but must not convert a flaky result into canonical green.
5. **Reproduce before widening.** Fuzz crashes and mutation gaps become minimized deterministic regressions before a fix is considered closed.
6. **Pinned external tooling.** Tool versions, actions, containers, advisory databases, and benchmark corpora are recorded explicitly where they affect proof.
7. **Separation of external availability.** Network service outages are reported distinctly from deterministic product failure.
8. **No PHI.** Assurance corpora remain synthetic/public and license-governed.
9. **No oracle laundering.** Independent tools inform commandF; they do not become hidden semantic authority.
10. **Small stacks.** Each assurance implementation PR must be independently reviewable and preserve existing product semantics.

## Current external evidence that constrains the program

- GitHub Secure Use: full-length commit SHA is the immutable way to pin an Action; repository/organization policies can require this.
- SLSA v1.2 is the current approved SLSA specification and includes Build and Source tracks plus provenance/verification guidance.
- Sigstore bundles carry verification material and signature content required for offline-capable verification.
- Rust Fuzz Book recommends `cargo-fuzz`; structure-aware fuzzing is appropriate for structured domains and supports differential fuzzing.
- `cargo-mutants` measures whether plausible source mutations are caught by the existing test suite.
- `cargo-nextest` can detect retries as flaky and can be configured so a flaky result still fails CI.
- `cargo-deny` checks licenses, bans, advisories, and crate sources.
- RustSec `cargo-audit` audits `Cargo.lock` for known Rust ecosystem vulnerabilities.
- `cargo-llvm-cov` provides source-based coverage with enforceable line/function/region floors.
- `zizmor` statically analyzes GitHub Actions for injection, credential, permission, and reference risks.
- OpenSSF Scorecard provides an additional repository security posture view; it remains supplemental evidence, not a substitute for commandF-owned gates.
- HL7 FHIR publication history currently distinguishes R5 `5.0.0` (published) from R6 `6.0.0-ballot5` (draft ballot).

## Primary references

- https://docs.github.com/en/actions/reference/security/secure-use
- https://slsa.dev/spec/v1.2/
- https://docs.sigstore.dev/about/bundle/
- https://rust-fuzz.github.io/book/cargo-fuzz.html
- https://rust-fuzz.github.io/book/cargo-fuzz/structure-aware-fuzzing.html
- https://mutants.rs/
- https://nexte.st/docs/features/retries/
- https://embarkstudios.github.io/cargo-deny/checks/
- https://rustsec.org/
- https://github.com/taiki-e/cargo-llvm-cov
- https://docs.zizmor.sh/
- https://github.com/ossf/scorecard-action
- https://hl7.org/fhir/directory.html

## Explicitly not authorized by this document

This program document alone does not authorize:

- production code changes outside an approved AF/CF Spec Kit unit;
- a CF-06 production oracle pin change;
- mutation of the frozen CF-10 corpus;
- PHI or real patient-instance fixtures;
- AI/model authority;
- a stable release claim;
- a universal trust score;
- implementation of CF-14, CF-15, or CF-16 ahead of their own Spec Kit packages.
