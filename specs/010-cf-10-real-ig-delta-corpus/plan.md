# CF-10 Implementation Plan — Public Real-IG Delta Corpus

Status: implementation in progress / eligibility and digest freeze complete

## Architecture decision

CF-10 is a benchmark/evidence layer above existing commandF authorities. It does not add a new semantic engine.

The implementation is split into four hard boundaries:

1. **Corpus manifest model** — validates frozen case metadata, digests, rights/provenance evidence, ordering, and v1 R4 constraints.
2. **Acquisition/attestation** — uses CF-01 resolver/cache verification in isolated state and refuses semantic work until digest/size attestation succeeds.
3. **Case evaluation** — invokes existing CF-03/04/07 evidence and CF-06 oracle behavior without changing their rules.
4. **Deterministic summary** — emits compact commandF-owned benchmark evidence and explicit operational failure states.

## Phase A — Frozen selection and provenance — COMPLETE

Canonical v1 cases remain fixed:

```text
C001 hl7.fhir.us.core  8.0.1 -> 9.0.0
C002 hl7.fhir.uv.ips   1.1.0 -> 2.0.1
C003 hl7.fhir.us.mcode 3.0.0 -> 4.0.0
```

Selection was frozen before commandF semantic result discovery. No case or version has been replaced.

The donor/provenance record distinguishes publication/change/rights evidence and keeps repository mode metadata-only. No upstream package payload is committed.

## Phase B — Foundation reconciliation and digest discovery — COMPLETE

CF-11 multi-version graph support is canonical at merge commit:

```text
5cb1a4c3445c0ebd86654cfb467a5e008e801c3e
```

It was reconciled into the frozen CF-10 branch by:

```text
5ec463f0ae53b76f9c2c151335d98598b53e5abc
```

The same six frozen package states were then resolved twice from independent clean caches and verified through CF-01. `cf10-digest-discovery` run `31890014888` succeeded and artifact `9248341586` was reviewed before digest/size metadata was frozen in `corpus/real-ig/v1/corpus.json`. Later exact-head rerun `31890859039` also succeeded.

No expected semantic result was generated or frozen in this phase.

## Phase C — Manifest model — COMPLETE

Ownership is `commandf-pkg` typed code:

```text
crates/commandf-pkg/src/corpus_model.rs
crates/commandf-pkg/src/corpus_error.rs
crates/commandf-pkg/src/corpus.rs
crates/commandf-pkg/tests/corpus_manifest.rs
```

Implemented validation includes:

- schema exactly 1;
- bounded pre-decode input bytes and bounded case count;
- stable unique lexicographically ordered case ids;
- exact package names and semver versions;
- R4 `4.0.1` only in v1;
- lowercase 64-hex SHA-256;
- positive bounded archive size;
- before != after version;
- publication/change/rights evidence required;
- closed rights/oracle enums;
- unknown fields fail closed;
- deterministic canonical JSON round trip;
- canonical manifest assertions against the reviewed frozen digests/sizes.

The manifest parser performs no network access.

## Phase D — Package attestation — COMPLETE

Reusable package-state attestation is owned by `commandf-pkg` and enforces this order:

```text
Lockfile::verify_cache(whole graph)
-> exact (package, version) selection
-> locked digest == manifest digest
-> verified target archive read
-> archive bytes and SHA-256 == manifest
```

Regression tests cover matching states, manifest digest mismatch, size mismatch, corrupted target cache, missing/ambiguous exact lock identity, and an unrelated unverified lock entry blocking the target attestation.

This makes CF-01 verification a mandatory authority rather than a caller convention.

## Phase E — Deterministic evaluator — NEXT

Add a thin evaluator that consumes a validated `RealIgCase` plus already resolved/attested package states.

For each case:

1. obtain attested before/after root bytes from verified state;
2. call `diff_package_archives` (CF-03);
3. call canonical compatibility classification (CF-04);
4. call canonical terminology evidence against the same verified lock/cache states (CF-07);
5. call the CF-06 oracle boundary only for changed matched StructureDefinitions;
6. reduce canonical sub-reports into deterministic aggregate counts without changing their semantic rules;
7. retain raw sub-report bytes/hashes as evidence for CI artifacts.

Do not introduce another matcher, classifier, terminology engine, or oracle implementation.

Unit tests use synthetic/local package fixtures. Ordinary tests must not require public network, Java, or Maven.

## Phase F — Execution surface — DECIDED

CF-10 v1 exposes exactly one corpus execution surface:

```text
commandf corpus run \
  --manifest corpus/real-ig/v1/corpus.json \
  --work-root <path> \
  --oracle-adapter <path> \
  --oracle-java <path> \
  --format json
```

Do not add a second public repository harness.

The CLI may acquire packages because this is an explicit integration/benchmark operation, but acquisition and semantic evaluation remain separated internally:

```text
acquire -> CF-01 verify -> manifest attest -> evaluate
```

A selected case must never be silently skipped. Operational failure is represented explicitly and causes the real-corpus gate to fail while preserving evidence.

## Phase G — Deterministic summary

V1 summary reuses existing vocabulary where possible and contains no timestamps/absolute paths/random ids/network timing/temp paths.

Logical shape:

```text
schema
manifest_sha256
cases[]
  case_id
  package
  before { version, sha256 }
  after { version, sha256 }
  structural { changes }
  compatibility { findings, breaking, risky, additive, producer, consumer, both }
  terminology { code_system_changes, value_set_changes, binding_refinements }
  oracle { compared, agreement, commandf_only, authority_only, both_changed, uncomparable }
  status
```

Exact counting rules must be direct reductions over canonical report fields, not new compatibility policy.

## Phase H — Real corpus workflow and first-result freeze

Add one bounded dedicated workflow for the three frozen cases. Required stages:

```text
manifest-validation
acquisition-and-attestation
structural-classification-terminology
oracle-evidence
repeat-clean-run
deterministic-summary-equality
artifact-upload
repository-payload-scan
```

The workflow must:

1. run all three cases from a clean work root;
2. upload raw structural/compatibility/terminology/oracle reports plus deterministic summary as short-retention artifacts;
3. run the full corpus a second time from another clean work root/cache;
4. require byte-identical deterministic summary output;
5. preserve failures/divergences as evidence rather than removing cases;
6. verify no upstream IG tarball/terminology payload enters git history;
7. only after two-run equality, freeze observed summary hash/count regression evidence.

Any upstream byte change at the same package/version is a hard attestation failure, not an automatic baseline update.

## Test strategy

Manifest tests and attestation tests are already implemented as described above.

Evaluator tests must use small synthetic packages and prove aggregation equals canonical underlying reports. Add explicit failure-path tests for malformed/unsupported sub-reports and no silent case removal.

Real integration testing belongs only in the dedicated CF-10 workflow.

## Rights controls

No package bytes enter git history.

The corpus manifest records upstream legal/IP evidence conservatively and does not collapse mixed terminology rights into one permissive license label. Publishing or bundling source packages as a dataset is outside CF-10 and requires a separate founder/legal authorization gate.

## Reviewer priorities

1. no benchmark cherry-picking after result discovery;
2. no semantic leakage into CF-03/04/07/06;
3. no package-content redistribution;
4. exact digest attestation before analysis;
5. no silent skipped cases;
6. deterministic result bytes;
7. bounded untrusted manifest/package metadata;
8. source-specific rights evidence;
9. real integration failures remain visible evidence;
10. no golden expected result authored before first verified two-run proof.

## Convergence condition

CF-10 converges only after:

- frozen selection remains unchanged;
- exact digests/sizes remain frozen from reviewed independent discovery;
- typed manifest/attestation/evaluator tests pass;
- all three real cases execute without silent removal;
- two clean runs produce byte-identical deterministic summary;
- `ci`, `cf06-oracle`, CF-11 proof, and dedicated corpus workflow are green on the exact final head;
- substantive reviewer findings are fixed or explicitly rejected with evidence;
- `spec.md`, `plan.md`, `tasks.md`, donor record, corpus manifest, implementation, and `convergence.md` agree on final truth.
