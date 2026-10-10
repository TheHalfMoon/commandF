# UX-01 — Bounded Review Preview Implementation Plan

Status: IMPLEMENTED_CANDIDATE / NOT_EXECUTION_AUTHORITY

1. Add a CLI `ReviewPreview` variant using only existing before/after package lock/cache inputs and current check policy settings. Do not create a separate evaluator.
2. Reuse `build_diff_report`, `classify_structural_diff`, and `evaluate_compatibility_policy` for authoritative **existing** policy semantics.
3. Reuse `impact::run` for the **existing** graph report. If either operation fails, return operational exit 1 without emitting an incomplete bundle.
4. Combine only internally serialized JSON using a size-bounded composer. Include explicit false claim flags; no borrowed code, new dependencies or format-version negotiations.
5. Write output through existing atomic replacement helper; return the existing 0/2 policy exit codes after both reports are complete. Normalize malformed `review-preview` args to 1.
6. Add test fixtures from existing synthetic locked package archives. Convert v1 fixtures to v2 only in the preview tests. Verify both reports, policy status 0/2, output-file replacement, unsupported lock fail-closed and invalid-use 1.
7. On the exact candidate SHA, run rustfmt, focused tests, broad Rust CI, assurance checks, GitHub actions and independent review. The sample implementation is not production release authority.
8. Follow up separately with actual consumer contracts, coherent single-snapshot receipts, SARIF/Studio/report ergonomics, evidence preservation, and any newly adopted V3.2 architecture. Do not silently equate preview with those features.

Review checklist: validate source trusted boundaries, valid JSON composition, deterministic serializations, output atomicity, memory bounds, error exit semantics, no unverified archive read, and truthful capability labels.
