# AF-02 + Dependency Security Dual-Bootstrap Repair Tasks

Status: ACTIVE_DUAL_BOOTSTRAP_CANDIDATE
Issues: #100, #99

- [x] **R001** Re-establish live failure on PR #91 exact head and identify the failing AF-02 test.
- [x] **R002** Confirm live run-artifacts collection no longer exposes historical artifact `9255732702`.
- [x] **R003** Confirm retained workflow-run, retained contract, manifest blob, and donor blob identities remain independently reconstructible.
- [x] **R004** Freeze AF-02 repair safety contract.
- [x] **R005** Keep shared candidate-input `verify_artifacts` semantics unchanged.
- [x] **R006** Add a live-only historical-unavailability boundary after canonical retained-contract and workflow-run verification.
- [x] **R007** Add regression proving the live-only path does not weaken the shared verifier.
- [x] **R008** Prove the AF-02-only exact head through full `ci` run `36616122386` and dependent proof workflows; remaining red checks were isolated to #99.
- [x] **R009** Prove the current main assurance ruleset creates a no-bypass required-check cycle between #99 and #100.
- [x] **R010** Generate the patched lockfile with Cargo/Rust `1.97.1`, require byte-exact minimal change, and record SHA-256 `d7e3050b3ff81fc39db61a0345875de6c0b7bc7ca48cc7f432a1f2ba14653648`.
- [x] **R011** Fast-forward the generated `Cargo.lock` onto the bootstrap branch with exact target-head guard; commit `3ebe793` changed one file, two insertions/two deletions.
- [x] **R012** Reconcile `spec.md` and `plan.md` so the dual-bootstrap scope is explicit and neither issue silently absorbs the other.
- [ ] **R013** Exact-final-head compare proves only the bounded five-path repair set and exact lockfile delta.
- [ ] **R014** Exact-final-head full CI succeeds, including AF-02 reconstruction and unchanged authority baseline.
- [ ] **R015** Exact-final-head AF-01 security and assurance-proof succeed with `RUSTSEC-2026-0285` absent.
- [ ] **R016** Confirm all other triggered proof workflows succeed on the same final head.
- [ ] **R017** Run Jev independent review where technically available; record exact limitation if unavailable.
- [ ] **R018** Run Alibaba Open Code Review where technically available; record exact limitation if unavailable.
- [ ] **R019** Resolve/disposition every substantive review finding and rerun affected qualification.
- [ ] **R020** Re-read current review/protected-merge requirements and satisfy them without bypass.
- [ ] **R021** Mark PR ready for review only after final-head qualification supports that transition.
- [ ] **R022** Guarded normal merge with exact expected head; no rebase, force-push, or history rewrite.
- [ ] **R023** Fresh-main post-merge verification proves the required checks remain green canonically.
- [ ] **R024** Close #99 only after canonical lock/security evidence proves the advisory is remediated.
- [ ] **R025** Close #100 only after canonical AF-02 evidence proves durable reconstruction with the historical artifact absent and no verifier weakening.

No unchecked task may be reported as complete without exact evidence. Automatic CodeRabbit/Qodo/Cubic status is not qualification evidence.
