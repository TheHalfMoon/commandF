# AF-02 Retained Authority Durability Repair Tasks

Status: ACTIVE_REPAIR_CANDIDATE
Issue: #100

- [x] **R001** Re-establish live failure on PR #91 exact head and identify the failing AF-02 test.
- [x] **R002** Confirm live run-artifacts collection no longer exposes historical artifact `9255732702`.
- [x] **R003** Confirm retained workflow-run, retained contract, manifest blob, and donor blob identities remain independently reconstructible.
- [x] **R004** Freeze bootstrap repair safety contract in `spec.md` and `plan.md`.
- [x] **R005** Keep shared candidate-input `verify_artifacts` semantics unchanged.
- [x] **R006** Add a live-only historical-unavailability boundary after canonical retained-contract and workflow-run verification.
- [x] **R007** Add regression proving the live-only path does not weaken the shared verifier.
- [ ] **R008** Exact-head relevant test qualification.
- [ ] **R009** Exact-head full CI and AF-02 authority-baseline equality.
- [ ] **R010** Confirm any remaining red AF-01 checks are solely owned by #99 and not this repair.
- [ ] **R011** Jev independent review where technically available; record exact limitation if unavailable.
- [ ] **R012** Alibaba Open Code Review where technically available; record exact limitation if unavailable.
- [ ] **R013** Resolve/disposition every substantive review finding and rerun affected qualification.
- [ ] **R014** Guarded normal merge under live canonical governance.
- [ ] **R015** Fresh-main post-merge verification.
- [ ] **R016** Close #100 only after canonical evidence proves durable reconstruction with the historical artifact absent and no verifier weakening.

No unchecked task may be reported as complete without exact evidence.
