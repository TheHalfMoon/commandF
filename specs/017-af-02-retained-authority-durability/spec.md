# AF-02 Retained Authority Durability Repair

Status: BOOTSTRAP_REPAIR_CANDIDATE
Issue: #100
Parent assurance program: AF-02 / `016-af-02-adversarial-test-strength`

## Problem

AF-02 reconstructs historical CF-10 authority from a canonical retained contract plus immutable Git objects and live GitHub metadata. The historical Actions artifact `9255732702` from workflow run `31916124080` has aged out of GitHub artifact retention. Fresh CI therefore fails even though the retained run identity and the immutable manifest/donor Git objects remain verifiable.

The defect is not that the historical artifact identity changed. The defect is that a time-bounded hosting availability observation was accidentally made a perpetual prerequisite for reconstructing an otherwise durable historical authority projection.

## Safety objective

Restore durable AF-02 reconstruction without turning artifact absence into authority and without weakening the shared verifier used for candidate-supplied retained-artifact evidence.

The semantic CF-10 projection remains derived from exact immutable Git objects:

- retained contract blob `f9c0bc16ac742238c93ff77a85486cd1db5dbcf3` at canonical authority base `54b9772a3b86464da6f395f8ba8371f364c9bb38`;
- retained manifest blob `655949a8a30d67502dffd624a175d2e8e02b1d1f` at head `5fe10d9859407272acf6649fc3e868d3eb2fbd12`;
- retained donor blob `566b46f4e6f467a1ccae3ac810b31956309173b6` at the same retained head;
- live workflow run metadata for run `31916124080`, preserving its historical `failure` conclusion.

The historical artifact record remains exactly:

```text
id: 9255732702
name: cf10-real-corpus-evidence
sha256: 9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612
workflow_run_id: 31916124080
```

No replacement artifact identity is authorized.

## Authorized behavior

The live canonical reconstruction test MAY classify the historical artifact collection as `HISTORICAL_UNAVAILABLE` only when the GitHub run-artifacts response is structurally valid and reports exactly:

```json
{"total_count": 0, "artifacts": []}
```

In that state:

1. the artifact identity/digest remain historical retained metadata from the pinned canonical retained-contract Git blob;
2. no artifact bytes are claimed available or reverified;
3. the live workflow-run identity still MUST verify;
4. the immutable retained manifest and donor Git objects still MUST verify and reproduce the same three deltas / six states;
5. the generated AF-02 authority baseline MUST remain byte-identical to the canonical snapshot;
6. the state is logged explicitly and must not be described as a live artifact PASS.

If the live artifact collection is non-empty, the existing strict `verify_artifacts` contract remains authoritative and MUST verify the exact expected id/name/digest/run binding.

## Candidate-input boundary

The shared `verify_artifacts` function is NOT changed by this repair.

Candidate-supplied or fixture-supplied empty artifact collections MUST continue to fail with the existing retained-authority mismatch. This prevents an untrusted candidate from self-authorizing by deleting artifact evidence.

The historical-unavailability exception exists only in the live canonical reconstruction test after the retained contract has already been loaded from the pinned canonical authority base and the live workflow run has verified.

## Fail-closed cases

The repair MUST fail when any of the following occurs:

- live response omits or malforms `total_count` or `artifacts`;
- `total_count == 0` while the artifact array is non-empty;
- a non-empty collection lacks the exact retained artifact;
- artifact id/name/digest/run binding differs;
- workflow-run identity differs;
- retained contract blob differs;
- retained manifest or donor blob differs;
- projected CF-10 states differ;
- authority-baseline bytes differ.

## Bootstrap rationale

A planning-only repair PR cannot obtain a green canonical CI while the broken live-retention dependency itself remains in that CI. This bounded repair therefore carries its frozen contract and implementation together. It does not broaden AF-02 product semantics, change CF-06, change CF-10 history, alter shared candidate-input verification, or authorize V3 implementation.

## Review

Exact-head repository CI and assurance remain required. Jev and Alibaba Open Code Review are required independent review inputs where technically available; inability to execute either must be recorded rather than fabricated. CodeRabbit, Qodo, Cubic, or similar hosted statuses are not qualification evidence for this repair.

## Exit condition

The repair closes only when fresh exact-head CI proves the canonical AF-02 reconstruction succeeds with the historical artifact absent, the shared candidate artifact verifier remains strict, the authority baseline remains unchanged, and all other applicable gates are reconciled honestly.
