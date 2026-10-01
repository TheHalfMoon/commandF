# Reproducibility

Status: RESEARCH_PLANNING. Guarantees that exist, and requirements for a run that does not exist. There is no reproduction package.

## Identities that are already pinned

The candidate corpus digest is `e92d734ab2981691322074f9d963bcf6ffc9058dace3875fdea64d2558b417aa`. `research/CANDIDATE_CORPUS.md` defines it as the SHA-256 of the canonical JSON item lines in item order, joined by one newline and ending without a trailing newline. Each item line is the SHA-256 of UTF-8 canonical JSON with sorted keys. The three members cite `hapifhir/hapi-fhir` at `e307df6b64ff87c55af1607160f57141dbeb0360` under Apache-2.0.

Split policy version is `sp-1`. It does not assign roles. Statistical plan version is `sap-1`. It does not compute a rate. Baseline command identities were read at `616ef5d61762ae9870a3036268e33ea21a9cbcc5`. B0 is `commandf diff`. B2 is `commandf check` with the CLI defaults at that commit. B1 and B5 have no pinned bytes. No built-binary digest is recorded.

Package bytes used by the shipped commands are accepted only when `PackageCache::read_verified` finds the SHA-256 recorded on the lock. A mismatch is an error. A future reproduction cannot replace the corpus digest or a cited git revision with a floating tag. Spec 043's acquisition path is not implemented, so this is not a registry-replay package.

## What a later experiment would have to name

`research/REPRODUCIBILITY.md` describes a future public flow: obtain allowable corpora, verify digests, run pinned baselines, run commandF, compute metrics, and emit tables from immutable result files. That flow is not running. A result file would have to name the git revision it ran, the corpus digest, `sp-1`, `sap-1`, the baseline command identities or their later replacements, and the seed sap-1 says is recorded at execution. The seed is not chosen in the plan. Headline numbers must not be typed by hand when a generator exists. No generator output is in `research/RESULTS_LEDGER.md`.

Exact-head qualification of a manuscript pull request is the repository's required checks on that head. It is not a reproduction of an experiment.

## What this section does not claim

It does not claim a third party can regenerate a headline table. It does not claim the validator jar or the Java runtime. It does not freeze a protocol.
