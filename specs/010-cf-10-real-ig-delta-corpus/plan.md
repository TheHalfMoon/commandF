# CF-10 Implementation Plan — Public Real-IG Delta Corpus

Status: planned / selection rules frozen

## Architecture decision

CF-10 is a benchmark/evidence layer above existing commandF authorities. It does not add a new semantic engine.

The implementation is split into four boundaries:

1. **Corpus manifest model** — validates frozen case metadata, digests, rights/provenance evidence, ordering, and v1 R4 constraints.
2. **Acquisition/attestation** — uses CF-01 resolver/cache verification in isolated state; package bytes are ephemeral and never committed.
3. **Case evaluation** — invokes the existing CF-03/04/07 evidence stack and CF-06 oracle adapter without modifying their rules.
4. **Deterministic summary** — emits compact commandF-owned benchmark evidence and explicit operational failure states.

The first executable gate is digest discovery. Result counts/severities are deliberately unknown until after the cases and package identities are frozen.

## Phase A — Freeze selection and provenance

Canonical v1 cases are fixed before result discovery:

```text
C001 hl7.fhir.us.core  8.0.1 -> 9.0.0
C002 hl7.fhir.uv.ips   1.1.0 -> 2.0.1
C003 hl7.fhir.us.mcode 3.0.0 -> 4.0.0
```

Selection is based on published/stable R4 status, public package identity, explicit version/change evidence, and domain diversity. commandF output is not a selection input.

The donor/provenance record must distinguish:

- publication metadata rights;
- package/runtime access;
- terminology/IP statements inside the IG;
- repository redistribution rights.

CF-10 repository mode is metadata-only. No upstream NPM tarball is checked in.

## Phase B — Digest discovery

Add a one-shot/guarded GitHub Actions discovery workflow or equivalent isolated execution tool that:

1. checks out the exact CF-10 planning head;
2. resolves every selected exact package version into cache A;
3. verifies cache A;
4. resolves the same package/version into independent cache B;
5. verifies cache B;
6. requires archive bytes and SHA-256 to match A == B;
7. records byte size and digest in a machine-readable artifact;
8. fails if any selected version is unavailable or non-reproducible.

The discovery artifact is reviewed before digest metadata is copied into the canonical corpus manifest.

No expected commandF semantic result is generated or frozen in this phase.

## Phase C — Manifest model

Preferred ownership: `commandf-pkg` typed model and validation, not ad-hoc shell parsing.

Proposed modules:

```text
crates/commandf-pkg/src/corpus_model.rs
crates/commandf-pkg/src/corpus_error.rs
crates/commandf-pkg/src/corpus.rs
```

Public model concepts:

```text
RealIgCorpus
CorpusSelectionPolicy
RealIgCase
CorpusPackageState
CorpusRightsEvidence
CorpusOracleMode
```

Validation requirements:

- schema exactly 1;
- bounded input bytes and case count;
- stable unique case ids;
- canonical lexicographic case ordering;
- exact package names/versions;
- R4 `4.0.1` only in v1;
- SHA-256 lowercase/64-hex;
- positive bounded archive size;
- before != after version;
- publication/change/rights evidence URLs required;
- `metadata_only_no_redistribution` only in v1;
- `changed_structure_definitions_only` oracle mode only in v1;
- unknown enum/schema values fail closed.

The model does not fetch the network.

## Phase D — Deterministic evaluator

Add a thin evaluator that consumes an already validated manifest and explicit package state.

Do not introduce another matcher/classifier/terminology engine/oracle implementation.

For each case:

1. require attested before/after archive bytes matching manifest digest/size;
2. call `diff_package_archives`;
3. call canonical compatibility classification;
4. call canonical terminology diff against the same verified states;
5. call the CF-06 oracle path only for changed matched StructureDefinitions;
6. reduce those reports into deterministic aggregate counts without losing the underlying report identities/hashes.

Raw detailed reports remain CI artifacts; the committed/public corpus summary remains compact commandF-owned evidence.

## Phase E — User-visible execution surface

Prefer one narrow CLI rather than benchmark shell glue:

```text
commandf corpus run \
  --manifest corpus/real-ig/v1/corpus.json \
  --work-root <path> \
  --oracle-adapter <path> \
  --oracle-java <path> \
  --format json
```

The command may perform acquisition because corpus execution is explicitly an integration/benchmark operation, but acquisition and evaluation remain internally separated so digest verification happens before semantic analysis.

Alternative if implementation pressure reveals the CLI is unnecessary: keep the typed evaluator public in `commandf-pkg` and use a repository-owned executable harness. This decision must be made before implementation and reflected in `spec.md`; do not silently create both.

## Phase F — Summary schema

Proposed deterministic v1 summary:

```text
schema
manifest_sha256
cases[]
  case_id
  package
  before { version, sha256 }
  after { version, sha256 }
  structural
    changes
  compatibility
    findings
    breaking
    risky
    additive
    producer
    consumer
    both
  terminology
    code_system_changes
    value_set_changes
    binding_refinements
  oracle
    compared
    agreement
    commandf_only
    authority_only
    both_changed
    uncomparable
  status
```

Exact field vocabulary must reuse existing public enum names where possible rather than invent parallel terms.

The summary does not claim clinical safety, semantic equivalence, or universal benchmark coverage.

## Phase G — First-result freeze

After implementation and digest lock:

1. run the entire v1 corpus from a clean environment;
2. preserve raw reports as CI artifact;
3. repeat from another clean cache;
4. require byte-identical deterministic summary;
5. review disagreements/unsupported evidence as findings, not failures to hide;
6. only then freeze exact summary digest/counts as a regression baseline in corpus v1 metadata or a separate expected-results file.

Any future upstream-package byte change at the same version is a hard attestation failure, not an automatic baseline update.

## CI layout

Preserve current `ci` and `cf06-oracle` workflows.

Add a dedicated corpus job/workflow with explicit resource/time bounds because three public IG pairs plus the Java oracle are heavier than ordinary unit tests.

Recommended stages:

```text
manifest-validation
package-attestation
structural-classification-terminology
oracle-evidence
repeat-determinism
summary-verification
```

CI must upload raw reports and the deterministic summary as short-retention review artifacts. Artifacts are evidence, not repository-vendored benchmark data.

## Test strategy

### Manifest tests

- wrong schema;
- empty corpus;
- too many cases;
- duplicate/out-of-order ids;
- malformed package/version;
- same before/after version;
- non-R4 FHIR version;
- malformed digest/size;
- missing publication/change/rights metadata;
- unsupported rights/oracle mode;
- deterministic JSON round trip.

### Attestation tests

Use synthetic/local package fixtures for unit tests:

- matching digest/size passes;
- digest mismatch fails;
- size mismatch fails;
- corrupted cache fails;
- before/after state cannot alias unexpectedly.

### Evaluation tests

Use small synthetic packages to verify aggregation equals canonical underlying reports. Do not make ordinary unit tests depend on public network or Java/Maven.

### Real integration tests

The dedicated CF-10 job uses the three frozen public cases and the pinned CF-06 adapter.

## Rights controls

No package bytes enter git history.

The corpus manifest records upstream legal/IP evidence but does not collapse mixed terminology rights into a single permissive license label. In particular, IPS contains terminology with separate upstream licensing statements; metadata-only benchmarking avoids redistribution and does not grant downstream terminology rights.

Any future proposal to publish/download-bundle the source packages as a commandF benchmark dataset is a separate founder/legal authorization gate and is not CF-10.

## Reviewer priorities

1. no benchmark cherry-picking after result discovery;
2. no semantic leakage from benchmark expectations into CF-03/04/07/06;
3. no package-content redistribution;
4. exact digest attestation before analysis;
5. no silent skipped cases;
6. deterministic result bytes;
7. bounded untrusted manifest/package metadata;
8. rights evidence remains conservative and source-specific;
9. real integration failures remain visible evidence;
10. no golden expected result authored before first verified run.

## Convergence condition

CF-10 converges only after:

- selection policy and case set remain unchanged from the pre-result spec unless an eligibility/right fact is proven false;
- exact digests/sizes are independently discovered and frozen;
- all three cases resolve and attest;
- typed manifest/evaluator tests pass;
- real corpus run completes without silent case removal;
- two clean runs produce byte-identical summary;
- current `ci` and `cf06-oracle` remain green;
- dedicated corpus workflow is green on the exact final head;
- substantive reviewer findings are fixed or explicitly rejected with evidence;
- `spec.md`, `plan.md`, `tasks.md`, donor record, corpus manifest, and `convergence.md` agree on the final truth.
