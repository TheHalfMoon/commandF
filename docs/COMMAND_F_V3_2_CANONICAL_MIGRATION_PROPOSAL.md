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
| C7 | Open issues #15 and #100 are dispositioned as in roadmap §5. | Issue comments |

## 3. AF-02 governance deadlock: legitimate resolution path (G51)

### 3.1 The problem, precisely

- The canonical-base verifier (`tools/af02-verifier/src/base_gate.rs`) rejects any candidate that changes an existing authority path. Those paths include its own sources, all workflows, `donors/`, `Cargo.lock`, and `Cargo.toml`.
- A new file under `tools/af02-verifier/src/` is not compiled unless an existing module or manifest is edited (spec 050). So no candidate can teach the gate to accept anything.
- The gate is **not** a ruleset-required check. It has already been merged past once, for a security fix (#101), and it currently fails every Cargo and Actions Dependabot PR.

### 3.2 Why no purely mechanical path exists

Any change to the gate is a change to an authority path, which the gate rejects. Adding the gate's acceptance logic in data is impossible, because data is never executed (spec 050). Therefore **a stronger authority than the gate is required exactly once**. In this repository that authority is the founder: the admin role with Code Owner review under ruleset 21652974. That is decision **FD-1**.

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
2. **FD-1 recorded**: a founder comment on the bootstrap PR naming the exact head SHA, the exact changed paths with before/after blob SHAs, and the statement that the AF-02 base-gate rejection on this head is overridden once.
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

## 4. Issue #100: a durable, content-addressed authority mechanism

**Historical facts (immutable).**

- Artifact `9255732702` (`cf10-real-corpus-evidence`).
- Run `31916124080`, conclusion `failure`.
- Digest `sha256:9fdde985…af612`.
- The bytes are unavailable (spec 019).

These are never re-labeled and never regenerated as a substitute.

**Forward mechanism**, designed in a new grain and not in spec 017 or 019, which are kept as they are:

1. **Deterministic evidence bundles.** A proof workflow writes a tar with:
   - sorted entries;
   - mtime 0;
   - uid and gid 0;
   - no extended attributes;
   - either no compression or a pinned gzip implementation and level.

   The bundle digest is the authority identity.
2. **Durable storage in Git itself.**
   - The bundle is committed to a dedicated ref namespace, `refs/evidence/<workflow>/<run-id>`, through an evidence commit.
   - A ruleset blocks deletion and non-fast-forward on `refs/evidence/**`.
   - Git objects are content-addressed and replicated by every clone. No dependence on Actions artifact retention remains.
   - Bundles above a size cap (proposed: 25 MiB) store a manifest plus the digests of large inputs. Those large inputs must be independently re-acquirable by pinned identity.
3. **Reconstructibility.** Every bundle carries a regeneration recipe: exact inputs, tool identities, and command. A verifier accepts either:
   - (a) bytes whose digest matches; or
   - (b) a regeneration that produces byte-identical output, recorded as `REGENERATED_IDENTICAL`.

   A regeneration that is only semantically equivalent is `RECONSTRUCTED_SEMANTIC_EQUIVALENT_CANDIDATE` (spec 019's term). It is never accepted as the original.
4. **Write path.**
   - Only a dedicated workflow with `contents: write` may push `refs/evidence/**`, triggered on `main` pushes only and never on `pull_request_target`.
   - The workflow is pinned by SHA and statically analyzed by zizmor.
5. **Issue #100 terminal state (FD-3).** Close the issue as `HISTORICAL_ARTIFACT_BYTES_UNAVAILABLE`, with a link to the forward mechanism. Retained-authority reconstruction for that historical run stays permanently classified as unavailable. It is not reinterpreted as a pass.

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
