# CF-11 Plan — Multi-Version Package Graph

Status: implementation authorized by CF-10 foundation evidence

## Base

```text
repository: TheHalfMoon/commandF
canonical base: 4c72f4a21aca757fbdadd2fe34384b8d0c746b85
branch: fix/cf-11-multi-version-package-graph
blocked consumer: CF-10 / PR #11
```

## Evidence trigger

CF-10 run `31856586654` attempted two clean resolutions for every frozen package state and proved 5/6 states fail repeatably under CF-01's name-keyed selected map. The evidence artifact was uploaded before the workflow's final eligibility assertion failed.

This plan corrects that foundation model without using any semantic benchmark result to guide implementation.

## Implementation shape

### 1. Resolver identity

Replace the selected closure key:

```text
package name
```

with:

```text
(package name, selected concrete version)
```

The selected version is still computed before deduplication using the request's existing exact/patch-wildcard rules.

### 2. Expansion rule

For each queued request:

1. select the request-local concrete version deterministically;
2. form exact identity `(name, version)`;
3. if that identity is already selected, skip re-expansion;
4. otherwise download, verify manifest identity, cache by digest, lock the concrete package, and enqueue its declared dependency requests.

Different concrete versions of the same name therefore coexist rather than conflict.

### 3. Lock schema

Keep `commandf.lock` schema v1 unchanged for this slice. Its `packages: Vec<LockedPackage>` already carries name, concrete version, digest, source, and declared dependency constraints and already sorts by `(name, version)`.

CF-11 does not claim schema-v1 records resolved dependency edges. A future explicit graph schema may add those only if a shipped command requires them.

### 4. Downstream guard

Do not weaken name-only ambiguity checks in diff/check/terminology/oracle. These commands may continue rejecting a lock where the requested package name maps to more than one version unless the caller already supplies an exact identity (`inspect`).

### 5. Tests

Update the former `incompatible_versions_fail_closed` regression into a positive multi-version graph test and add:

- same concrete identity dedup across branches;
- wildcard + exact request producing two concrete identities;
- equivalent root-order byte stability with multi-version closure;
- cycle terminating on exact-identity revisit.

Preserve all previous exact identity/manifest/cache tests.

### 6. Real proof

Add a bounded real-network workflow dedicated to this correction. It must:

- use immutable action SHAs and `persist-credentials: false`;
- resolve + verify one previously failing frozen CF-10 state from clean cache;
- assert the lock contains at least one package name at multiple concrete versions;
- assert the exact root package/version is present;
- remain evidence-only for CF-11 and not execute CF-10 semantic diff/classification.

Preferred proof state: `hl7.fhir.us.core@8.0.1`, because the prior frozen sweep recorded its exact old-resolver conflict in both A/B runs.

## Review focus

Reviewers should prioritize:

1. traversal/order dependence;
2. duplicate exact identity or repeated archive expansion;
3. silent global version coercion;
4. lock nondeterminism;
5. cycles / unbounded re-expansion;
6. downstream accidental first-match behavior;
7. regression of archive/registry/cache trust boundaries.

## Exit

CF-11 may converge only after exact-head CI, oracle, real multi-version proof, and returned reviewer findings are dispositioned. CF-10 stays Draft and frozen until CF-11 is merged to canonical main.
