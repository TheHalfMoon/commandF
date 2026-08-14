# CF-07 Implementation Plan

Status: Implementation authorized

## Architecture

CF-07 is a parallel slice directly above converged CF-04:

```text
base branch: feat/cf-04-compatibility-rules
base SHA: ae33586a925023d92b4d58db01663bf26f3bd9a3
```

No new workspace crate is planned. `commandf-pkg` owns terminology parsing/proof/reconciliation and `commandf` owns the explicit CLI wiring.

Implementation boundaries:

1. reuse CF-03 package scanning and CF-04 classification authority;
2. add a read-only verified-lock terminology closure index;
3. prove only finite closed membership sets;
4. expose `indeterminate` for semantically unsupported but well-formed cases;
5. emit separate CF-07 binding refinements without mutating embedded CF-04 findings.

## Modules

Add narrowly scoped modules under `crates/commandf-pkg/src/`:

```text
terminology_model.rs
terminology_error.rs
terminology_index.rs
terminology_set.rs
terminology_binding.rs
terminology.rs
```

Names may be consolidated if implementation remains clearer, but no new crate is justified by CF-07.

## Public model

```text
TerminologyProofMode
  code_system_complete
  value_set_expansion

TerminologyRelation
  equal
  narrowed
  widened
  incomparable
  indeterminate

TerminologyMember
  system
  version
  code

TerminologySetDelta
  resource
  resource_type
  proof_mode
  relation
  binding_proof_eligible
  reason
  before_count
  after_count
  added
  removed

BindingRefinement
  resource
  element_id
  view
  before_value_set
  after_value_set
  relation
  direction
  severity
  rule_id
  message

TerminologyDiffReport
  schema
  ruleset
  package_name
  before
  after
  compatibility
  code_systems
  value_sets
  binding_refinements
```

Public constants:

```text
schema: 1
ruleset: cf07-terminology-v1
```

All collections use deterministic ordering before serialization.

## Verified lock-closure index

The terminology command loads before and after states independently.

For each state:

1. parse the supplied CF-01 lockfile;
2. verify every lockfile cache object before consuming it;
3. read each content-addressed package archive;
4. reuse the bounded package-root scanner;
5. parse only FHIR JSON package-root resources;
6. index `ValueSet` and `CodeSystem` resources by canonical URL and optional business version;
7. reject duplicate exact `url|version` identities;
8. preserve all resources sharing a bare URL so bare-reference ambiguity can be detected explicitly.

No registry/source object is constructed on this path. There is no package acquisition.

### Canonical resolver

Parse references as:

```text
url
url|version
```

Rules:

- exact version -> exactly one matching canonical/version;
- bare URL -> exactly one resource with that URL across the state;
- malformed empty URL/version -> fail closed;
- multiple bare matches -> `AmbiguousCanonical`;
- missing -> `UnresolvedCanonical` for report-level indeterminate binding evidence.

The resolver never silently selects “latest”.

## CodeSystem finite-set proof

For a matched CodeSystem pair:

1. validate interpreted fields (`content`, `compositional`, `caseSensitive`, `count`, concept code shapes);
2. require `content == complete` on both sides;
3. reject proof when either is compositional;
4. require unchanged `caseSensitive` semantics;
5. recursively flatten all concept codes;
6. reject duplicate or empty codes;
7. if `count` exists, require exact flattened-count equality;
8. compare code sets.

Relations:

```text
after == before                  -> equal
after proper subset before       -> narrowed
before proper subset after       -> widened
otherwise                         -> incomparable
```

Unsupported completeness cases return `indeterminate`, not errors, unless the interpreted resource is malformed.

Output members for CodeSystem deltas use:

```text
system = canonical CodeSystem URL
version = side business version when present
code = exact code
```

For set comparison itself, codes are compared within the matched system URL; before/after business versions remain evidence and do not prevent relation proof.

## ValueSet expansion finite-set proof

For each matched ValueSet pair:

1. require `expansion` on both sides;
2. require offset absent or zero;
3. recursively flatten `expansion.contains`;
4. ignore entries with no code as navigation/grouping nodes;
5. require every coded entry to carry system + non-empty code;
6. identity is exact `(system, version?, code)`;
7. reject duplicate membership identities;
8. require `total` on each side and require it to equal the number of unique flattened coded members;
9. normalize expansion parameters as deterministic `(name, typed-value)` records and require exact multiset equality across sides;
10. reject unsupported/multiple parameter value shapes.

The hierarchy is never used for logical inference.

### Abstract expansion members

If any coded expansion member has `abstract=true`:

- set-delta evidence may still be emitted;
- `binding_proof_eligible=false`;
- reason includes `abstract_member_present`;
- no hard required-binding refinement is emitted from that relation.

### Expansion parameters

`identifier` and `timestamp` are not membership context and are excluded.

All `expansion.parameter` entries are retained for context comparison because unknown server parameters can affect membership. Conservative exact equality is preferred over guessing which parameters are semantic.

## Binding refinement

Start from the exact CF-04 `CompatibilityReport` and CF-03 structural evidence.

For each `ElementFieldChanged(field="binding")` with changed ValueSet canonical:

1. retain the existing CF-04 finding unchanged;
2. parse before/after binding strength and ValueSet reference using the same fail-closed code vocabulary as CF-04;
3. resolve the referenced ValueSets in the corresponding verified lock closures;
4. derive/probe the ValueSet relation;
5. emit relation evidence regardless of whether hard refinement is authorized;
6. emit hard refinement only if before strength == after strength == `required` and the proof is binding-eligible.

Hard rule ids:

```text
CF07-BIND-001  required ValueSet narrowed  -> producer BREAKING
CF07-BIND-002  required ValueSet widened   -> consumer BREAKING
CF07-BIND-003  required ValueSet replaced by incomparable finite set -> producer BREAKING
CF07-BIND-004  required ValueSet replaced by incomparable finite set -> consumer BREAKING
```

Equal finite membership emits no hard finding and does not create a SAFE state.

For extensible/preferred/example, relation evidence is retained but no hard refinement is emitted solely from set inclusion.

If binding strength changes simultaneously, CF-04 strength findings remain authoritative and CF-07 records `unsupported_binding_interaction` rather than layering a potentially contradictory hard membership judgment.

## Package-level terminology delta discovery

The public report must include direct CodeSystem/ValueSet deltas for terminology resources in the requested root package that are matched across the before/after archives by the existing CF-03 canonical matching rules.

Binding resolution may additionally inspect the verified dependency closure to resolve referenced ValueSets, but dependency resources are not automatically emitted as root-package direct deltas unless they belong to the requested package being compared.

This prevents a root package report from pretending that unchanged dependency resources are part of its own artifact delta while still permitting deterministic binding proof.

## CLI

Add:

```text
commandf terminology <package-name> \
  --before-lock <path> \
  --before-cache <path> \
  --after-lock <path> \
  --after-cache <path> \
  --format json
```

The CLI reuses the exact two-state package selection behavior already used by `diff`/`classify`.

Execution sequence:

1. load and verify explicit lock/cache states;
2. load selected package archives;
3. run CF-03 structural diff;
4. run CF-04 classification;
5. build verified terminology indexes for both lock closures;
6. derive direct root terminology deltas;
7. derive binding relation/refinement evidence;
8. serialize deterministic JSON.

No network object is created.

## Errors vs indeterminate

### Runtime/parser error

Use a hard error for evidence that is malformed relative to fields CF-07 actively interprets:

- non-string canonical URL/version/code/content;
- malformed boolean/count/offset/total;
- duplicate exact canonical identity;
- duplicate CodeSystem concept code in a complete proof candidate;
- duplicate ValueSet expansion member identity;
- expansion parameter with zero or multiple value[x] fields;
- corrupted cache;
- unsupported CF-03/CF-04 schema/ruleset.

### `indeterminate`

Use relation-level `indeterminate` for valid FHIR cases outside the authorized proof domain:

- compose-only ValueSet;
- no expansion;
- paged/incomplete expansion;
- expansion context mismatch;
- non-complete CodeSystem content;
- compositional CodeSystem;
- case-sensitivity semantics changed;
- unresolved referenced ValueSet;
- abstract expansion members for binding proof;
- simultaneous binding-strength/value-set interaction.

## Bounds

Reuse CF-02/03 package archive bounds.

Additional CF-07 bounds:

```text
max terminology resources indexed per state: 100,000
max flattened members per terminology resource: 1,000,000
max normalized expansion parameters per ValueSet: 10,000
max public added members per delta: 100,000
max public removed members per delta: 100,000
```

If the full relation cannot be computed within configured hard bounds, fail closed. Do not truncate and still claim subset/superset.

## Tests

### CodeSystem matrix

- complete self-equivalence;
- one removed code -> narrowed;
- one added code -> widened;
- add+remove -> incomparable;
- nested concepts flatten deterministically;
- duplicate code fails closed;
- count mismatch -> indeterminate or hard malformed evidence per parser contract;
- fragment/example/not-present/supplement -> indeterminate;
- compositional true -> indeterminate;
- caseSensitive change -> indeterminate;
- display/designation/property-only changes do not become membership changes.

### ValueSet matrix

- complete expansion self-equivalence;
- added/removed/incomparable tuples;
- nested hierarchy flattening;
- navigation node without code ignored;
- coded entry without system fails closed;
- duplicate tuple fails closed;
- missing expansion -> indeterminate;
- total mismatch / nonzero offset -> indeterminate incomplete proof;
- expansion parameter reordering normalizes;
- parameter semantic mismatch -> indeterminate;
- identifier/timestamp differences do not alter membership context;
- abstract coded member disables hard binding proof.

### Binding matrix

- required equal -> no hard refinement;
- required narrowed -> producer BREAKING;
- required widened -> consumer BREAKING;
- required incomparable -> both BREAKING;
- extensible/preferred/example retain relation evidence without hard break;
- simultaneous strength+ValueSet change -> unsupported interaction / no duplicate hard claim;
- unresolved/ambiguous ValueSet behavior;
- embedded CF-04 report remains byte-for-byte semantically unchanged.

### Closure index matrix

- root-package canonical resolution;
- dependency-package canonical resolution;
- exact `url|version` selection;
- bare unique selection;
- bare ambiguity;
- duplicate exact identity;
- corrupted dependency cache fails before proof;
- no source/registry/network use.

### CLI matrix

- `terminology --help`;
- required arguments;
- self-diff deterministic empty deltas;
- synthetic changed package reports expected set/refinement evidence;
- corrupted before/after/dependency caches;
- repeated byte-identical JSON.

## Real integration gate

Extend CI for the CF-07 branch while preserving all existing CF-01..04 gates.

Use two independently resolved `hl7.fhir.r4.core@4.0.1` states and run:

```text
commandf terminology hl7.fhir.r4.core ... --format json
```

Acceptance:

- schema/ruleset correct;
- embedded compatibility findings empty;
- no false direct terminology deltas on self-comparison;
- no acquisition during terminology command.

Positive membership/refinement behavior remains synthetic/public because repository CI must not redistribute proprietary terminology content.

## Review priorities

1. no false complete-set proof;
2. no remote terminology dependency;
3. bare canonical ambiguity cannot choose silently;
4. finite-set relation direction matches CF-04 producer/consumer semantics;
5. required-only hard binding refinement boundary is enforced;
6. extensible human applicability is not reduced to set math;
7. expansion paging/context cannot create false equality/subset;
8. CodeSystem completeness/compositional/case sensitivity gates are enforced;
9. embedded CF-04 evidence is preserved;
10. output is deterministic and bounded;
11. no proprietary terminology fixtures.

## Convergence

CF-07 converges only after exact-final-head fmt/clippy/tests/real R4 terminology smoke pass, reviewer truth is dispositioned, spec/plan/tasks/convergence match implementation, PR remains Draft/open/unmerged with auto-merge disabled, and CF-08 remains unstarted.
