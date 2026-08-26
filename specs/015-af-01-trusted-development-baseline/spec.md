# AF-01 Specification — Trusted Development Baseline

Status: PLANNING_CANDIDATE

## Identity

`AF-01` is the first commandF Assurance Foundation unit. It is cross-cutting development/release assurance, not a replacement or renumbering of product slice CF-14.

Canonical planning base:

```text
main: 8a45857bf31c4acae57fdfb1e3cdde3d0f7d0361
tree: ffaa14fdc7a738a771ac872e566ad1609eedf2cc
CF-13: CLOSED_CANONICAL
```

## User problem

commandF's product evidence is increasingly rigorous, but the repository does not yet enforce the same trust level uniformly across source control, workflows, third-party Actions, dependency policy, and security checks.

Today a maintainer can observe green exact-head workflows, but GitHub does not itself prevent a direct or insufficiently qualified update to `main`. The general CI workflow also still contains mutable Action/tool references that are weaker than commandF's newer proof workflows.

The result is a mismatch: commandF can prove interoperability evidence more strongly than it proves the integrity of the development path that produced that evidence.

## Outcome

After AF-01 closes, commandF has an independently executable and reviewable trusted-development baseline that:

1. detects mutable or unsafe GitHub Actions references and mutable proof execution-container identities;
2. minimizes workflow token/checkout authority and rejects permissions broader than a checked-in documented need;
3. audits Rust dependency vulnerabilities, licenses, sources, and banned/duplicate dependency policy;
4. statically audits workflow definitions for known CI/CD security hazards;
5. records exact-head assurance evidence;
6. requires repository-level source-control policy for canonical `main` so green checks are enforced rather than advisory.

## Functional requirements

### FR-001 — repository workflow trust audit

Provide a repository-owned deterministic audit that scans every tracked GitHub workflow and every tracked Action metadata file, including both supported metadata names `action.yml` and `action.yaml`, and fails if an external `uses:` reference is not pinned to a full 40-hex Git commit SHA unless an explicit bounded exception is recorded in the AF-01 policy.

The audit must also detect at least:

- checkout steps that persist credentials without an explicit reviewed exception;
- workflow/job permissions broader than their documented need, using checked-in AF-01 policy metadata that makes the allowed permission set machine-checkable rather than relying on prose-only review;
- proof-critical job or service container images that use a mutable tag/reference instead of an accepted digest identity, unless an explicit bounded exception is recorded;
- use of `ubuntu-latest`, `windows-latest`, or `macos-latest` in proof-critical jobs where AF-01 requires a fixed runner label;
- new workflow files or Action metadata files, regardless of whether they are named `action.yml` or `action.yaml`, that are outside the audit scope.

The audit result must be deterministic for the same repository tree and the same checked-in AF-01 policy.

### FR-002 — workflow hardening

All existing commandF workflows must be reconciled to a documented minimum baseline:

- external Actions use full commit SHAs;
- checkout uses `persist-credentials: false` unless a documented write operation requires otherwise;
- top-level/job `permissions` are explicit and no broader than the machine-checkable documented need for the workflow/job;
- proof-critical workflows use fixed runner labels and retain or improve digest-pinned execution containers where already present;
- proof-critical job/service containers use digest identities when a container is part of proof identity; mutable tags alone are insufficient;
- `cargo` commands that consume the lockfile use `--locked`;
- no product semantic gate is removed or weakened to make hardening pass.

Fixed GitHub-hosted runner labels are not claimed to make the underlying runner image byte-immutable. Proof workflows that require stronger execution identity must use an explicit container/toolchain identity and record it.

### FR-003 — dependency policy

Add checked-in `cargo-deny` policy covering:

- advisories;
- accepted licenses;
- crate source policy;
- duplicate/banned crate policy;
- explicit exceptions with rationale.

Policy must default to fail closed for unknown git/registry sources not intentionally authorized.

License policy must be derived from the actual dependency graph and commandF donor/provenance policy rather than copied blindly from another repository.

### FR-004 — RustSec vulnerability audit

Add an independently executable `cargo audit` gate against the exact `Cargo.lock` used by the candidate tree.

A vulnerability/advisory waiver must include:

- advisory or crate identity;
- reason;
- affected scope;
- compensating evidence where applicable;
- revisit/removal condition.

Absence of a vulnerability finding is not a claim that the dependency graph is generally secure.

### FR-005 — GitHub Actions static analysis

Add a pinned `zizmor` audit over repository workflow/composite-action definitions.

The commandF-owned workflow-trust audit remains authoritative for commandF's explicit pinning/credential/permission/container-identity rules. `zizmor` is an independent static-analysis signal and does not replace repository-owned policy.

### FR-006 — OpenSSF Scorecard evidence

Add or document an OpenSSF Scorecard run for the public repository, retaining exact tool/action identity and result provenance where the integration permits it.

Scorecard is supplemental posture evidence. No minimum aggregate Scorecard number is a commandF correctness gate in AF-01.

### FR-007 — main source-control enforcement

Canonical `main` must have repository-level branch/ruleset enforcement that, at minimum:

- blocks non-PR direct changes except a narrowly defined administrator/break-glass path;
- requires the current canonical CI/assurance checks selected by AF-01;
- selects as required only checks that produce a terminal result for **every** protected-branch pull request at the latest candidate SHA; a workflow-level `paths`, `branches`, or commit-message skip that leaves the required check pending is prohibited for a selected required check;
- if expensive validation is path-conditional, uses an always-triggered lightweight required gate/job whose terminal conclusion reflects the applicable conditional jobs, or keeps the path-filtered check non-required;
- requires review according to repository governance;
- rejects unresolved required review conversations where supported;
- prevents force-push and branch deletion for `main`;
- does not allow a stale prior-head PASS to satisfy a moved PR head.

The exact GitHub ruleset configuration is operational repository metadata, not a substitute for checked-in workflow policy.

If the active automation connector cannot mutate rulesets, AF-01 must record that limitation and remains open until an authorized administrator applies the exact configuration and live GitHub evidence verifies it.

### FR-008 — exact-head assurance evidence

Add a dedicated AF-01 proof workflow or equivalent retained artifact that records at least:

- exact head SHA and tree SHA;
- hashes/identities for the AF-01 spec/plan/tasks/consistency files;
- exact workflow-trust audit result;
- exact dependency-policy result;
- exact RustSec audit result;
- exact workflow static-analysis result;
- tool/action/container versions or immutable refs used;
- repository ruleset/branch-protection observation when queryable;
- a final deterministic assurance summary digest.

No timestamp is used as evidence identity.

### FR-009 — no regression in product authority

AF-01 must not change:

- CF-03 structural semantics;
- CF-04 compatibility rules;
- CF-05 policy semantics;
- CF-06 production oracle identity;
- CF-07 terminology semantics;
- CF-09 source-mapping semantics;
- CF-10 frozen corpus;
- CF-11/11G graph identities;
- CF-12 impact semantics;
- CF-13 baseline/suppression/gate semantics.

### FR-010 — reviewer truth

AF-01 planning and implementation must request CodeRabbit and Qodo when available. Returned findings must be dispositioned against the exact candidate head. Reviewer absence, timeout, rate limit, or summary-only output is not a PASS.

Codex Review is not required for AF-01.

## Non-functional requirements

### NFR-001 Determinism

Repository-owned audits and proof summaries must be byte-stable for identical pinned inputs wherever external advisory-database content is not itself part of the input.

When an advisory database or external security service is used, its identity/update state must be recorded so a later result can explain drift.

### NFR-002 Least authority

Security workflows must not receive write permissions merely to report read-only findings unless the reporting mechanism itself requires a narrowly scoped write permission. Checked-in AF-01 policy must make intended workflow/job permissions reviewable and machine-checkable so later permission expansion cannot pass merely because the YAML remains syntactically valid.

### NFR-003 Bounded execution

Every new CI job must define a bounded timeout appropriate to its function. External network checks must have bounded retry/timeout policy and must not retry indefinitely.

### NFR-004 No PHI

No patient data or PHI is introduced.

### NFR-005 Stackability

Implementation is split into independently reviewable stacks; a single monolithic workflow rewrite is not acceptable if it obscures which security property changed.

## Acceptance scenarios

1. A workflow changes `actions/checkout@<full-sha>` to `actions/checkout@v5` -> repository workflow-trust audit fails.
2. A new Action metadata file named `action.yaml` contains a mutable external `uses:` ref -> the audit discovers it and fails; `action.yaml` cannot evade a scanner written only for `action.yml`.
3. A new workflow uses credential-persisting checkout without an allowed exception -> fails.
4. A workflow/job requests `contents: write` when checked-in AF-01 policy permits only `contents: read` -> fails.
5. A proof-critical job/service changes a digest-pinned container image to `image:vendor/tool:latest` or another mutable tag -> fails.
6. A dependency is added from an unapproved git source -> `cargo-deny` fails.
7. A dependency introduces a RustSec advisory -> vulnerability gate fails unless an explicit reviewed waiver exists.
8. A workflow contains a security issue detected by the configured `zizmor` severity policy -> audit fails or is explicitly dispositioned according to the frozen policy.
9. The same tree is audited twice with the same pinned tool/advisory inputs -> commandF-owned assurance summary bytes are identical.
10. A documentation-only PR that does not match a heavy proof workflow's `paths` still receives a terminal result for every check selected as required by the `main` ruleset; no required check remains indefinitely pending because its entire workflow was skipped.
11. Product test suites and all path-applicable existing proof workflows remain green.
12. Live GitHub query confirms the intended `main` source-control policy before AF-01 claims canonical closure.

## Edge cases

- GitHub Actions referenced through local `./` paths are local source, not external mutable tags.
- Action metadata discovery covers both GitHub-supported names: `action.yml` and `action.yaml`, including metadata below repository subdirectories rather than only a root file.
- Docker/OCI job or service images referenced by digest are acceptable immutable identities; mutable image tags alone are not proof identity.
- A non-proof workflow may use a container only under the explicit AF-01 container policy; omission from proof identity must be deliberate and machine-checkable rather than accidental.
- Reusable workflows require the same immutable-reference discipline as third-party Actions where GitHub supports commit-SHA references.
- Workflow/job permission inheritance and omission must be normalized by the audit so an absent local `permissions` block cannot silently gain broader authority from an unexamined parent/default.
- GitHub distinguishes a skipped **job**, which can report a terminal successful/skipped conclusion, from a skipped **workflow** caused by path/branch/commit-message filtering, whose required check can remain pending. Required-check design must account for that distinction.
- Scorecard or advisory-service unavailability must be distinguished from a clean security result.
- A security tool finding that is not applicable may be dispositioned, but the disposition and rationale become retained evidence.
- `cargo-deny` duplicate-version policy must not blindly reject legitimate unavoidable transitive duplication without review; exceptions are explicit and narrow.

## Explicit non-goals

AF-01 does not implement:

- fuzzing, mutation score, property testing, or coverage floors — AF-02;
- Windows/macOS portability matrices or release signing — AF-03;
- benchmark/performance budgets — AF-04;
- CF-14 profiler behavior;
- CF-15 AutoFix recipes;
- CF-16 mapping IR;
- CF-06 production-pin changes;
- CF-10 corpus changes;
- AI/model/agent authority.
