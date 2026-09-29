# V3.1 Authority Migration Tasks

Status: ACTIVE_SPEC_CANDIDATE

- [x] **M001** Re-read canonical `main` after PR #91 and record `d0015357fd569f25f161168e759037eaeff2f487`.
- [x] **M002** Re-read issues #99 (closed), #100, #15, #35, #36, #37, #38, and #40 (open).
- [x] **M003** Re-read both active `main` rulesets and the sole-administrator review exception.
- [x] **M004** Write the reconciliation spec, plan, and this task list without product changes.
- [x] **M005** Add the index pointer and confirm the diff stays inside this kit plus that pointer.
- [ ] **M006** Open a pull request with exact base `d0015357fd569f25f161168e759037eaeff2f487`.
- [ ] **M007** Qualify the exact head: `rust`, `assurance-proof`, `scorecard`, and `af02-base-verifier` where the changed paths trigger them.
- [ ] **M008** Run Alibaba Open Code Review delegation mode. Record exclusions. Do not claim Markdown was OCR-reviewed if the CLI excludes it.
- [ ] **M009** Record Jev as executed or `BLOCKED_EXTERNAL_NO_ZERO_COST_AUTHORIZED_PATH`. Do not record PASS for a blocked call.
- [ ] **M010** Merge with a normal merge commit only after non-bypassable assurance checks are green.
- [ ] **M011** Verify post-merge `main` and fresh `assurance-proof` plus `scorecard` on the merge SHA.
- [ ] **M012** Do not close #100 or the verified-byte issues from this merge.

No unchecked task is complete.
