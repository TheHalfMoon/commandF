# AF-01 Consistency Analysis

Status: PLANNING_CANDIDATE

## Scope

This analysis reconciles:

- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`;
- `docs/COMMAND_F_PLAN_INDEX.md`;
- `docs/COMMAND_F_ASSURANCE_PROGRAM_2026-08-26.md`;
- `.specify/memory/constitution.md`;
- `AGENTS.md`;
- AF-01 `spec.md`;
- AF-01 `plan.md`;
- AF-01 `tasks.md`;
- live GitHub repository/ruleset state observed at the canonical planning base.

## Resolved consistency questions

### 1. Does AF-01 renumber CF-14/15/16?

No.

`AF` is a cross-cutting Assurance Foundation identity. CF product identities remain:

```text
CF-14 on-prem aggregate-only source profiler
CF-15 verified dry-run recipes
CF-16 mapping analysis IR, parse-only
```

Spec directory sequence `015` is only the next available Spec Kit package sequence after `014-cf-13-*`; it does not imply `CF-15`.

### 2. Does AF-01 violate the constitution's vertical-capability rule?

No.

The constitution permits a user-visible command, report, annotation, **or independently executable verification result**. AF-01 ships independently executable repository assurance results and retained proof artifacts. It is not an empty infrastructure scaffold.

### 3. Does AF-01 make future semantic layers dependencies of shipped commands?

No.

No commandF product command changes behavior. AF-01 wraps repository development assurance only.

### 4. Does AF-01 replace deterministic product gates with third-party security tools?

No.

The commandF-owned workflow-trust audit is explicit repository policy. cargo-deny, cargo-audit, zizmor, and Scorecard are independent evidence inputs. No third-party aggregate score becomes semantic or correctness authority.

### 5. Does cargo-deny conflict with the no-new-crate rule?

No product crate is added merely for cargo-deny. It is an external development/CI tool used immediately by an executable assurance gate. If implementation proposes a Rust helper crate solely for assurance configuration, it requires separate justification and should be avoided unless directly necessary.

### 6. Is using Python/shell for repository audit inconsistent with Rust ownership?

No.

AGENTS/constitution constrain commandF product/trusted interoperability core. Repository CI tooling may use bounded standard tooling. AF-01 explicitly keeps product runtime Rust-owned and avoids adding product dependencies for a CI-only parser.

### 7. Does requiring full-SHA Actions contradict GitHub's release guidance that tags are convenient?

No.

GitHub's secure-use guidance states full-length commit SHA is the immutable Action reference. AF-01 deliberately chooses the stronger reproducibility/security posture because commandF proof policy already rejects mutable aliases as sufficient evidence.

### 8. Is `ubuntu-24.04` itself immutable?

No, and the plan does not claim it is.

A fixed runner label narrows platform drift versus `ubuntu-latest`, but GitHub-hosted images still evolve. Proof-critical workflows that require stronger execution identity use digest-pinned containers/toolchain identities and retain them in evidence.

### 9. Are cargo-deny and cargo-audit redundant?

They overlap on RustSec advisory data, but serve different assurance roles.

- cargo-deny: repository-owned policy across licenses, sources, bans/duplicates, and advisories.
- cargo-audit: focused independent RustSec vulnerability audit of `Cargo.lock`.

AF-01 records that overlap and does not claim two independent vulnerability databases merely because two tools run.

### 10. Is Scorecard an acceptance score?

No.

AF-01 uses per-check posture evidence. Aggregate score is not a commandF correctness gate.

### 11. Can AF-01 close while `main` remains unprotected because the current connector cannot mutate rulesets?

No.

Connector capability is an execution limitation, not a waiver of the requirement. T038 is an external authorized administrator action and T039 requires a live read proving the resulting policy before AF-01 can close.

### 12. Does AF-01 authorize CF-14 implementation?

No.

AF-01's program-level ordering allows CF-14 **planning** to proceed in parallel, but CF-14 implementation requires its own Spec Kit authority. No profiler source/data behavior is authorized here.

### 13. Does AF-01 unblock CF-10 or change CF-06?

No.

CF-10 remains separately governed by the current CF-06 production-oracle contract. AF-01 must not mutate the HL7 production pin, frozen corpus, or semantic interpretation.

### 14. Does AF-01 solve test adequacy comprehensively?

No.

It establishes the trusted development baseline. Fuzzing, property tests, mutation testing, coverage, and flaky-as-failure execution are explicitly retained for AF-02 rather than being implied complete.

### 15. Does AF-01 make stable-release claims?

No.

SBOM, provenance/signing, public API compatibility, MSRV, and cross-platform release qualification are AF-03.

### 16. Does the current FHIR release status require a product behavior change in AF-01?

No.

The research correction (R5 published current; R6 `6.0.0-ballot5` draft as of 2026-07-17) informs future version-readiness assurance. AF-01 does not expand FHIR semantic support.

### 17. Is scanning only `action.yml` complete GitHub Action metadata coverage?

No.

GitHub supports both `action.yml` and `action.yaml`. AF-01 therefore requires recursive tracked-tree discovery of both names in T010/T012/T016 and AF-01 proof coverage. A future composite/JavaScript/Docker Action must not evade trust auditing merely by using the other supported filename.

### 18. Can a path-filtered workflow be selected directly as a required `main` check?

Not safely when the whole workflow may be skipped.

GitHub documents that a workflow skipped by path/branch filtering or commit-message skip can leave the associated required check pending and block merging. A skipped job inside an always-triggered workflow can reach a terminal conclusion, which is a different topology.

AF-01 therefore requires T035/T036 to prove that every check selected by the ruleset is universally terminal for every protected-branch PR at its latest candidate SHA. Heavy path-specific validation may remain conditional, but an always-triggered lightweight aggregation gate must represent it if it contributes to universal merge policy. Otherwise the path-filtered check remains non-required.

### 19. Did the first Qodo planning review findings remain unaddressed?

No.

The first Qodo review found missing task enforcement for machine-checkable workflow/job permissions and missing audit coverage for proof-critical container identities. Both were accepted and amended into `spec.md`, `plan.md`, and `tasks.md` with positive/negative tests.

The re-review then found the `action.yaml` metadata-name gap and required-check/path-filter topology gap. Those were also accepted and added to the planning contract. Any CI/review state from heads before these amendments is superseded.

## Requirement-to-task trace

| Requirement | Tasks |
|---|---|
| FR-001 workflow trust audit | T010-T013, T016 |
| FR-002 workflow hardening | T014-T019 |
| FR-003 cargo-deny policy | T020-T022, T027-T029 |
| FR-004 cargo-audit | T023-T024, T028-T029 |
| FR-005 zizmor | T025-T029 |
| FR-006 Scorecard | T030-T031 |
| FR-007 main source-control enforcement + universally terminal required checks | T035-T040, T043 |
| FR-008 exact-head assurance proof | T032-T034, T041-T043 |
| FR-009 product authority unchanged | T014-T019, T028-T029, T041, T052 |
| FR-010 reviewer truth | T005, T018, T029, T042, T054 |
| NFR-001 determinism | T012-T013, T033-T034, T041 |
| NFR-002 least authority | T011-T016, T030-T032 |
| NFR-003 bounded execution | T014-T015, T022-T025, T030-T032 |
| NFR-004 no PHI | all implementation tasks |
| NFR-005 stackability | phase ordering T010-T056 |

## Known planning risks retained explicitly

1. **Ruleset mutation is external to the current connector.** Closure remains blocked until live policy is applied and observed.
2. **Security-tool version pinning requires exact implementation-time selection.** Planning names tools, not mutable `latest` versions.
3. **Initial cargo-deny policy cannot be safely guessed from generic examples.** T020 requires actual current dependency/license inspection first.
4. **Initial zizmor severity threshold may need calibration from real findings.** Any change is an explicit plan/task amendment, not silent weakening.
5. **Scorecard external availability/authentication can fail independently of repository correctness.** Operational failure is reported separately from clean posture.
6. **Fixed GitHub runner labels still drift internally.** Strong proof continues to prefer digest-pinned environments.
7. **Required-check topology can deadlock merges if whole path-filtered workflows are selected directly.** T035/T036 require a universal terminal-result proof before ruleset selection.

## Final planning consistency result

No unresolved architecture contradiction is known in the authored package after the accepted planning-review corrections.

```text
AF-01 PLANNING CONSISTENCY: CANDIDATE / REQUIRES EXACT-HEAD CI + INDEPENDENT REVIEW
PRODUCT IDENTITIES CF-14/15/16: PRESERVED
CF-06/CF-10 AUTHORITY: UNCHANGED
IMPLEMENTATION AUTHORITY: NOT YET GRANTED
```
