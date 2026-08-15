# CF-11 Convergence — Multi-Version Package Graph

Status: foundation behavior proven; final documentation-head gates and reviewer reconciliation pending

## Decision

```text
CF-11_FOUNDATION_BEHAVIOR_PROVEN_PENDING_FINAL_REVIEW_RECONCILIATION
```

CF-11 corrects only the package-closure identity model. It does not reinterpret compatibility, policy, terminology, oracle, source attribution, or CF-10 corpus semantics.

## Canonical base and implementation identity

```text
repository: TheHalfMoon/commandF
PR: #13
base: main
canonical base: 4c72f4a21aca757fbdadd2fe34384b8d0c746b85
branch: fix/cf-11-multi-version-package-graph
implementation evidence head: 7411cebaa3052ccd71e83a916eb8d02e8269912c
```

The implementation changes the resolver selected-closure key from package name alone to exact `(package name, concrete version)` identity. Same exact identities deduplicate; different concrete versions of the same name remain distinct closure nodes. Lock schema v1 is unchanged and retains manifest-declared dependency constraints rather than claiming explicit resolved edges.

## Synthetic regression evidence

The workspace regression suite proves:

- two branches may retain `acme.dep@1.0.0` and `acme.dep@2.0.0` simultaneously;
- repeated requests resolving to the same exact identity produce one locked package;
- exact and patch-wildcard requests for the same package name can resolve to distinct versions;
- equivalent root-order permutations produce byte-identical lockfiles;
- exact-identity cycles terminate through deduplication;
- existing stable-patch selection and cache digest verification remain intact.

## Real frozen-state foundation proof

CF-10 remains frozen. CF-11 reused one of its previously ineligible states without replacing or cherry-picking the case:

```text
state: C002-ips-after
package: hl7.fhir.uv.ips@2.0.1
prior CF-10 failure: hl7.terminology.r4 selected 7.2.0, requested 7.1.0
```

Dedicated workflow:

```text
cf11-multi-version-proof
run: 31857934955
implementation head: 7411cebaa3052ccd71e83a916eb8d02e8269912c
result: SUCCESS
artifact id: 9239621769
artifact digest: sha256:6c4c54856fad7124fabe5cd15fb8b46666268438da35f9fd71a2988936da9be9
```

The workflow performed two independent clean `pkg resolve` + `pkg verify` runs, required byte-identical lockfiles, and proved same-name multi-version identities in the resulting real FHIR package closure:

```text
hl7.fhir.uv.extensions.r4: 5.2.0, 5.3.0
hl7.terminology.r4:        6.2.0, 7.1.0, 7.2.0
```

This proves the old name-level flattening was insufficient for this frozen real graph. It does not claim every FHIR package graph is supported.

## Exact implementation-head regression gates

Implementation evidence head `7411cebaa3052ccd71e83a916eb8d02e8269912c` passed:

```text
ci                         31857934949  SUCCESS
cf06-oracle                31857934948  SUCCESS
cf11-multi-version-proof   31857934955  SUCCESS
```

The mainline workflow passed Format, locked Clippy with `-D warnings`, full workspace tests, CF-08 and CF-09 security regressions, real FHIR resolve/verify + inspect/diff/classify/check, terminology smoke, CF-09 fixture preparation, local composite Action source-map self-check, and output verification.

The dedicated oracle workflow passed the pinned HL7 adapter build, real R4 context, self-equivalence, `commandf oracle` self-diff, invalid-snapshot and corrupted-cache fail-closed gates, changed-profile determinism, and end-to-end reconciliation.

## Downstream ambiguity boundary

CF-11 does not add a name-only version choice rule. Existing commands that request one locked package by name and encounter multiple locked versions remain required to fail closed. Exact `inspect name@version` selection remains exact.

Terminology canonical ambiguity and duplicate protections are unchanged.

## CF-10 boundary

CF-10 / PR #11 remains frozen and `BLOCKED_BY_FOUNDATION` until CF-11 is canonical. No CF-10 case may be replaced merely because the previous resolver rejected it.

After CF-11 becomes canonical, the first CF-10 action must be to reconcile onto the new main and rerun the exact same six frozen package states. Semantic diff/classify/check/terminology/oracle execution remains blocked until that eligibility rerun establishes the new foundation state.

## Reviewer truth at convergence-document creation

- **Codex Code Review:** requested on implementation head `7411ceba...`; no returned substantive result was observed at this document's creation time. No PASS claimed.
- **Qodo:** accepted the request and reported that review agents were working; no final findings/result was available yet. No PASS claimed.
- **CodeRabbit:** exact-head review attempt was rate-limited and explicitly did not start a substantive review. A commit status alone is not treated as review certification. No fresh PASS claimed.
- **Greptile:** exact-head review requested; no returned result observed. No PASS claimed.
- **Cubic:** generated PR summary only; not treated as correctness certification.

Any substantive result returned after this document is created must be dispositioned before merge. Reviewer absence or rate limits must be recorded rather than replaced with invented approval.

## Research inputs explicitly outside CF-11

Recent research/donor inputs — CPGPrompt, PathWISE, and `reason-healthcare/rh-skills` — are not dependencies of this foundation correction and do not modify its acceptance criteria. They belong to later research/benchmark/clinical-knowledge roadmap work after the package/corpus foundation is stable.

## Final documentation-head rule

This convergence update changes documentation only. Its resulting repository head must pass all three configured CF-11 gates again:

```text
ci
cf06-oracle
cf11-multi-version-proof
```

The final documentation-head SHA and final run ids are recorded in PR #13 metadata/body after those workflows settle. A failure reopens convergence. This document intentionally does not self-reference its own future commit SHA/run ids.
