# G51 — AF-02 Amendment-Admission Threat Model

Status: PLANNING_CANDIDATE / NOT_SECURITY_PROOF

## Trust boundary

The **canonical base** contains trusted verifier code and admitted amendment records. The proposed authority-changing pull request supplies untrusted diff data. GitHub PR state, reviews, and check-run conclusions are external observations that must be bound to exact identities. Author-written GitHub URLs and JSON strings are not review evidence.

## Threat and required negative result matrix

| ID | Attack or failure | Required verifier / process outcome |
| --- | --- | --- |
| T01 | Candidate edits `base_gate.rs` to accept itself | Verifier built from base; candidate logic never executed; reject without prior qualified record. |
| T02 | Candidate adds an amendment record in its own diff | Candidate-only record ignored for permission; reject unapproved authority change. |
| T03 | Reuse a record with changed base blob OID | Reject `STALE_AMENDMENT` or equivalent explicit failure. |
| T04 | Change one authorized Cargo blob plus an unrecorded workflow blob | Reject entire candidate; never partial-admit. |
| T05 | Edit/delete an existing amendment record | Reject immutable-history violation unless a separately qualified forward revocation contract expressly allows a new superseding record. |
| T06 | Forge `reviewer_approved:true`, a URL, or status-name strings | Verify actual external approval/check identity or report `UNVERIFIED_PROVENANCE`; never trust supplied text. |
| T07 | Replace file with symlink, submodule, or wrong Git mode | Reject until explicit typed policy and tests prove support. |
| T08 | Rename or case-fold path, insert `../`, duplicate JSON key, Unicode confusable or Windows backslash | Reject canonical-path/schema ambiguity. |
| T09 | Two conflicting or copied amendment IDs authorize the same before/after pair | Reject ambiguous simultaneous grants; require unique stable identity. |
| T10 | Edit `Cargo.lock` to make an oracle/version repin look like a routine dependency update | Must still satisfy separate oracle/frozen-dependency authority; ordinary amendment is insufficient. |
| T11 | Review was approved on an earlier head before a later push | Reject stale reviewer binding; require exact-head approval. |
| T12 | GitHub API/ruleset source is down during required approval reconstruction | `UNAVAILABLE` / `BLOCKED`, not proof. |
| T13 | Admin uses a standing bypass without a specific exact-head founder exception | Not an authorized G51 process; report a governance incident rather than treating the green required checks as consent. |
| T14 | Record preauthorizes a future ambiguous branch SHA | Restrict to exact file blob OIDs and preimage; later candidate head independently reverified. |
| T15 | Multiple grants match same changed authority path | Reject conflict; no first-match-wins or last-match-wins semantics. |
| T16 | Resource-exhaustion record or path listing | Enforce base-controlled aggregate limits and fail closed. |
| T17 | Previously valid amendment PR is reverted but authority-change PR still references it | Base tree must contain a valid admitted record; otherwise reject. |
| T18 | Record merged by unexpected path or via a temporary host artifact that expires | Retain durable base Git record and independently qualified review provenance; no dependency on ephemeral artifact bytes for basic admission. |
| T19 | Required check added before base gate can admit maintenance | Stop ruleset mutation. Test legitimate positive and negative paths first. |
| T20 | User gives a generic `go ahead` | Not specific founder FD-1 approval; never treat it as a blanket bypass. |

## Independent recovery and rollback

Rejecting an unauthorized PR must not rewrite branch history. Preserve the failed run and changed-file evidence. Fix the candidate in a new normal commit, requalify exact head, and request fresh review. If a defective bootstrap reaches `main`, document the incident and use a separately reviewed forward corrective or revert commit under current rules, not a forced push.

## Residual risks to document at implementation

- An admin or compromised GitHub identity can misuse an available review-layer bypass; this plan cannot make that platform power disappear.
- The application of GitHub approval and ruleset metadata can change or become unavailable; authorization reconstruction must specify trusted source and fail closed.
- Two reviewed PRs add overhead. This should be measured on actual security-update latency before narrowing the policy.
- Git SHA-1 object IDs and independent SHA-256 integrity digests serve different purposes; do not treat a SHA-1 blob name as a cryptographic signature.
- A passing verifier test does not establish absence of unknown vulnerabilities in the Rust code or its dependencies.
