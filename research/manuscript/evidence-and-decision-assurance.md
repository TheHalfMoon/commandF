# Evidence and Decision Assurance

Status: RESEARCH_PLANNING. It describes the evidence contract that already fails closed, and the decision envelope that is not implemented. It reports no sufficiency rate.

## Shipped identity

A locked package carries a name, a version, and a SHA-256. `Lockfile::verify_cache` asks the cache to verify every recorded digest. `PackageCache::read_verified` reads the object stored for that digest, hashes those bytes, and returns them only when the hash equals the digest. A mismatch is `PackageError::CacheDigestMismatch`. A missing object is `PackageError::CacheMissing`. An unsupported lock schema is `PackageError::UnsupportedLockSchema`. `read_verified_bounded` refuses an object above the caller-supplied byte limit before accepting the hash.

The context graph and the terminology closure load packages through that check. A replaced cache object fails closed. The classifier does not treat a digest failure as compatibility.

Provenance on a structural report is the package name plus the before and after lock identities that the diff recorded. That is not a decision receipt.

## The acquisition equality

`bytes_consumed == bytes_digest_verified` is the required equality in `specs/043-cf17-registry-acquisition/spec.md` and in `docs/COMMAND_F_V3_1_IMPLEMENTATION_RUNBOOK.md`. Spec 043 says the specification pull request does not contain that acquisition implementation. No Rust field is named `bytes_consumed`. The shipped cache check is the narrower equality of stored bytes to the digest already recorded on the lock. It is not official-registry acquisition, and it does not authorize CF-17.

Spec 043, if implemented later, bounds the read, allows only the two FHIR package hosts already named in `crates/commandf-pkg/src/registry.rs`, rejects a floating version and the token `latest`, and rejects an unverifiable body. Those rules are not a running acquisition command.

## Planned assurance

The V3.1 envelope in `docs/COMMAND_F_V3_1_DECISION_ASSURANCE_PLAN.md` places evidence assembly before a decision assurance step and a policy gate. Sufficiency and completeness are separate fields there, as `research/manuscript/problem-definition.md` already records. The planned invalid states reject `PROVEN_COMPATIBLE` when required evidence was not run or when unsupported witnesses remain inside the protected scope. A model probability cannot create that class.

No `DecisionReceipt` type is implemented. Unsupported input stays an error or an explicit coverage entry. It is not rewritten as `CheckDecision.passed = true`.

## What this section does not claim

It does not claim every command re-hashes every byte it later prints. It does not claim a receipt, a completeness score, or a measured fail-closed rate.
