# commandF V3.2 — Canonical Migration Proposal

Status: **PLANNING_CANDIDATE / PROPOSAL / REQUIRES FOUNDER DECISIONS FD-1, FD-2, FD-3**

This proposal describes how execution authority could move from V2 to a qualified successor without rewriting history. It changes nothing by being merged. V2 remains execution authority until the exit conditions in §5 are met on canonical `main`.

## 1. Principles

1. **Append, never rewrite.**
   - Historical Spec Kits, closeouts, issue records, merge SHAs, and failure records stay byte-identical.
   - Corrections are new records that supersede by reference.
   - Merge commits only. No squash, rebase, or force-push.
2. **One migration kit, not two.**
   - `specs/018-v3-1-authority-migration/` already exists as `SPEC_CANDIDATE`.
   - V3.2 proposes amending it with a dated delta section, instead of creating a parallel migration kit.
   - Spec 018 is not under an AF-02 authority prefix.
   - Spec 018 currently states that issues #35–#40 "stay open". They were closed on 2026-09-29. The delta must record that fact, and the G49 equivalent-pattern audit must supply the missing proof.
3. **No self-approval.** No candidate may be admitted by logic that the same candidate introduces.
4. **Founder authority is explicit and narrow.** Where human authority must break a deadlock, the decision names exact SHAs and paths. It is a one-time action, not a standing waiver.

## 2. Preconditions (all must hold on live `main` at migration time)

| # | Precondition | Evidence |
| --- | --- | --- |
| C1 | A generated status ledger (roadmap grain 052) reconciles every Spec Kit's state from tree and issue evidence. | Ledger file plus generator test |
| C2 | The G49 equivalent-pattern audit (grain 053) is merged. | Audit record |
| C3 | The G51 amendment path is bootstrapped and `af02-base-verifier` is a required check. | §3 complete; ruleset readback |
| C4 | G01–G54 each have exactly one primary owner. | `COMMAND_F_V3_2_GAP_RECONCILIATION_LEDGER.md` re-verified against live specs |
| C5 | No dependency cycle: no phase consumes later-phase evidence. | Roadmap §2 re-checked |
| C6 | The live rulesets match the recorded snapshot, or the delta is recorded. | `gh api` readback committed as text |
| C7 | Open issues #15 and #100 have accurate blocking statuses and linked evidence; #100 is not closed by an unimplemented forward proposal. | Live issue readback and issue comments |

## 3. AF-02 governance deadlock: legitimate resolution path (G51)

### 3.1 The problem, precisely

- The canonical-base verifier (`tools/af02-verifier/src/base_gate.rs`) rejects any candidate that changes an existing authority path. Those paths include its own sources, all workflows, `donors/`, `Cargo.lock`, and `Cargo.toml`.
- A new file under `tools/af02-verifier/src/` is not compiled unless an existing module or manifest is edited (spec 050). So no candidate can teach the gate to accept anything.
- The gate is **not** a ruleset-required check. It has already been merged past once, for a security fix (#101), and it currently fails every Cargo and Actions Dependabot PR.

### 3.2 Why no purely mechanical path exists

Any change to the gate is a change to an authority path, which the gate rejects. Adding the gate's acceptance logic in data is impossible, because data is never executed (spec 050). Therefore **a stronger authority than the gate is required exactly once**. The repository offers a founder-admin bypass capability, but use of that capability is **not** automatically authorized merely because it exists. A single exceptional action requires a separate exact-base-and-head, exact-path, human-reviewed FD-1 approval recorded against the specific bootstrap PR; this plan is not that approval.

### 3.3 Proposed protocol: amendment records that live in the base, not the candidate

After a one-time bootstrap, the gate admits a change to authority path `P` only if **the base tree** contains an amendment record that:

1. names `P`;
2. names the exact base blob SHA of `P` (or `ABSENT`);
3. names the exact candidate blob SHA of `P` (or `DELETED`);
4. gives a rationale, an independent-review reference (PR URL plus reviewer), and a founder approval reference;
5. has not already been consumed, meaning the base blob of `P` still equals the record's "before" SHA.

The record path is `governance/amendments/AMD-<NNNN>.json`. It is a new prefix that the bootstrap makes **append-only authority**: existing records are immutable, and new records may be added.

Why this is not circular:

- The record must be merged **before** the change, in its own PR, and reviewed by a Code Owner.
- The candidate PR cannot carry its own record. A record present only in the candidate is ignored, because the gate reads records from the base.
- Exact blob binding prevents a record from authorizing a broader edit than the one reviewed.
- A record is consumed automatically, because after merge the base blob no longer matches its "before" SHA.

Cost: every authority change takes two PRs. This is deliberate friction for authority paths. For routine `Cargo.lock` updates, FD-1 may choose a narrower variant: admit `Cargo.lock` changes when the base-built `cargo-deny` and `cargo-audit` pass. That variant is recorded as a **policy relaxation**, decided explicitly, never by default.

### 3.4 Bootstrap procedure (one time)

1. **Spec Kit 051** (docs only; touches no authority path; passes every gate): the design above, the threat model, and the test list.
2. **FD-1 requested, not presumed**: after the exact candidate commit exists and independent review is available, request a separate founder comment on that specific bootstrap PR naming exact base and head commit SHAs, every changed authority path with before/after blob SHAs, and the one-time override scope. The founder must explicitly approve the precise exception before an admin bypass may be used. Any later head change invalidates that exception.
3. **Bootstrap PR contents, limited to:**
   - `tools/af02-verifier/src/base_gate.rs` and a new `amendment.rs`, plus its module declaration;
   - tests;
   - the `governance/amendments/README.md` schema.

   No workflow, ruleset JSON, `Cargo.lock`, or product source is changed.
4. **Required tests in the bootstrap PR:**
   - (a) every rejection fixture that the current gate rejects is still rejected when no record exists;
   - (b) a record in the candidate only → rejected;
   - (c) a blob mismatch → rejected;
   - (d) a consumed record → rejected;
   - (e) an exact match → admitted;
   - (f) an edit to an existing record → rejected;
   - (g) an authority path outside the record's path list → rejected.
5. **Independent review:**
   - OCR on the Rust diff;
   - Jev if a zero-cost path is authorized, otherwise recorded as blocked;
   - human Code Owner approval at the exact head.

   Required checks must be green on the exact head. The `af02-base-verifier` failure is expected and is cited by run ID in the FD-1 comment.
6. **Merge** as a normal merge commit.
7. **Post-merge proof:** open a no-op PR and a deliberately rejected authority-edit PR. The new base gate must pass the first and reject the second. Record both run IDs.
8. **FD-2:** add `af02-base-verifier` to ruleset 21652953's required checks. Read the ruleset back and commit the readback text.

After step 8, the class of decision in step 2 should never be needed again. All future authority changes go through amendment records.

### 3.5 What must not be done

- Disabling or editing the workflow to skip the gate.
- Removing paths from the authority lists without an amendment record.
- Adding a bypass actor to ruleset 21652953.
- Reopening PR #146.
- Rewriting specs 049 or 050.

## 4. Issue #100: historical unavailability and a forward durable-evidence design

**Historical facts remain binding and are not repaired by planning.**

- Artifact `9255732702` (`cf10-real-corpus-evidence`), run `31916124080`, historical conclusion `failure`.
- Original artifact digest: `sha256:9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612`.
- The original artifact bytes are unavailable. The only truthful reconstruction result for those specific unavailable bytes is `HISTORICAL_ARTIFACT_BYTES_UNAVAILABLE`.
- A new execution, an author-authored fixture, a semantically similar output, or a new manifest is **not** a replacement for those bytes.
- Issue #100 **remains open** while the forward protocol is only a plan. FD-3 is a future disposition decision, not approval to close it now. A later issue closure can acknowledge the historical loss as permanent **only after** forward controls are implemented, tested, and independently reviewed; it must never claim that the historical proof succeeded or that its bytes were recovered.

### 4.1 Storage and reachability are different requirements

The previous proposal of using `refs/evidence/**` alone is **withdrawn**. A standard clone usually fetches ordinary heads via its configured refspec; it does **not** imply fetching arbitrary `refs/evidence/**` namespaces. Git object hashing proves identity of bytes that are present, not availability, completeness, access control, retention, or independent authority. GitHub is not an unconditional permanent archive.

**Preferred candidate for small, redistributable evidence:** reviewed, bounded content-addressed packets under a normal, protected canonical-branch path such as `evidence/retained/<digest>/` with a manifest, exact bytes, provenance, verification recipe, and rights/disclosure disposition. Each packet enters through a separate reviewed PR, not a CI self-write that edits `main` directly. File size, repository-growth limits, redaction, and data-rights checks are admission gates. Frozen packet paths are append-only under separately proven governance.

For large or restricted inputs, a manifest or Git LFS pointer **alone** is insufficient for an offline-replay claim. Before admitting an evidence identity, qualify a separate rights-compliant durable byte store and an independently mirrored recovery route, each digest-verified and retrieval-tested. If bytes cannot legally be retained or recovered, record `UNAVAILABLE` and do not issue reproducibility or proof claims.

An optional dedicated evidence branch/ref may be evaluated later, but it is not durable proof until its fetch refspec, branch protection, non-deletion, clone behavior, authorization, independent mirror, retention, and disaster-recovery behavior have been demonstrated on the actual hosting provider.

### 4.2 Retained-evidence record and verifier

1. Each retained packet binds exact source/input identity, run identity and outcome (including failures), tool/build/rule versions, relevant policy, original artifact SHA-256, and an immutable Git tree/blob identity for the retained bytes.
2. The packet is generated deterministically where applicable (sorted entries, fixed metadata, bounded archive layout, no environmental timestamps in semantic content). The verifier hashes **actual available bytes**, checks the declared source chain independently of candidate-authored expectations, and rejects conflicting evidence.
3. The verification tool is built from an already-authorized base; a candidate's own new verifier logic cannot approve the candidate's packet. An independently reviewed adoption PR binds the bytes and the verifier identity.
4. Reproduction produces a new execution receipt. Only byte-identical reconstruction can satisfy a byte-identity requirement; semantic similarity is an explicitly lower-authority research observation.
5. The design must distinguish `HISTORICAL_UNAVAILABLE`, `PRESENT_VERIFIED`, `REGENERATED_BYTE_IDENTICAL`, `MISMATCH`, `UNSUPPORTED`, and `RETRIEVAL_FAILED`, without collapsing a missing required item into `PASS`.
6. The proposal must explicitly address Git history growth, rights to redistribute source and benchmark artifacts, protected storage writes, key compromise, deletion and availability threats, and independent offline backup.

### 4.3 Required end-to-end qualification before FD-3

Prove all of these with retained execution evidence:

- historical Actions artifact deleted, genuine retained bytes available in the durable store -> verified offline reconstruction;
- historical artifact deleted and retained bytes absent -> `HISTORICAL_UNAVAILABLE`, never a green proof;
- normal full clone, shallow clone, single-branch clone, and air-gapped copy -> each reports the actual fetch/replay requirements truthfully; missing objects are not assumed present;
- removed/unreachable ref, tampered packet, mismatched digest, truncated/oversized archive, and rights-restricted item -> explicit fail-closed outcomes;
- candidate-authored packet or verifier cannot self-certify past historical authority;
- independent recovery path tested after simulated primary-host unavailability;
- repeatable retention across the provider's Actions artifact expiry window;
- new proof workflow does not silently gain privileged write authority on untrusted PR triggers.

No storage change, workflow privilege increase, ruleset change, or history reinterpretation is authorized by this document. Those actions require their own Spec Kit, exact-head tests, rights disposition, independent review, and founder approval when applicable.

## 5. Migration steps and exit conditions

| Step | Action | Exit evidence |
| --- | --- | --- |
| M1 | Preconditions C1–C7 | Each linked |
| M2 | Amend spec 018 with a "V3.2 delta" section: reconciled issue facts, G51–G54, and the V3.2 documents as the successor plan | Review + required checks + gate |
| M3 | Append to `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md` a single dated paragraph: "Execution order superseded by V3.2 as of merge `<sha>`; V2 remains the historical record and its invariants remain binding" | Diff is append-only (verified mechanically) |
| M4 | Update `docs/COMMAND_F_PLAN_INDEX.md` section A to name V3.2 | Same |
| M5 | Fresh post-merge `assurance-proof` and `scorecard` on the merge SHA | Run IDs |

**Identity preservation.** These identities keep their meaning:

- CF-01 to CF-28;
- CF-11G;
- AF-01 to AF-04;
- G01 to G50;
- every spec directory number;
- every rule ID;
- every schema ID.

V3.2 adds G51–G54, schema IDs under `commandf.*/v1` that are not used by specs 001–050 (checked at M2), and spec sequences from 051 onward.

**Rollback.** If a migration merge proves defective, a new superseding record restores V2 order, through a revert commit plus a record. History is never rewritten.
