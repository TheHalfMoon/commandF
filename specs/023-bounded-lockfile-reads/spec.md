# Bounded Lockfile Reads

Status: SPEC_CANDIDATE

Issue: #37

## Problem

CLI lockfile paths called `fs::read` and then `Lockfile::from_slice`. The parser has no byte limit, so an oversized caller-selected lockfile was allocated before validation.

## Contract

- The shared maximum is 16 MiB, the same bound the quality gate already used.
- A CLI read stops after that limit plus one byte.
- `Lockfile::from_bounded_slice` rejects a larger slice before JSON parsing.
- `Lockfile::from_slice` remains the parser for a caller that already holds a bounded slice. Its documentation states that responsibility.
- A file of exactly 16 MiB that is otherwise valid JSON is parsed.
- One extra byte fails with a stable size error and not a JSON error.
- A malformed file under the limit still fails as a parse error.
- This candidate does not change cache verified-byte behavior, local mirror bounds, CF-06 identity, or `verify_artifacts`.
- Issue #38 and issue #100 stay open. This candidate does not authorize CF-17.
