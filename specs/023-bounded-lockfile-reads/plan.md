# Bounded Lockfile Reads Plan

Status: SPEC_CANDIDATE

## Scope in

- Add `Lockfile::from_bounded_slice`.
- Route pkg, inspect, context, diff, classify, check, terminology, impact, oracle, and gate lock loads through one bounded reader.

## Scope out

- Local mirror archive allocation (issue #38).
- Historical retained artifact bytes (issue #100).
- CF-17 implementation.

## Exit

Normal merge after exact-head required checks. Issue #37 may close only after post-merge assurance evidence. Issues #38 and #100 stay open.
