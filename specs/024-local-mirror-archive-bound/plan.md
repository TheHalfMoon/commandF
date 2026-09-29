# Local Mirror Archive Bound Plan

Status: SPEC_CANDIDATE

## Scope in

- Share `MAX_COMPRESSED_PACKAGE_ARCHIVE_BYTES` between the registry and the local mirror.
- Bound the local mirror file read at that limit plus one byte.

## Scope out

- Historical retained artifact bytes (issue #100).
- CF-06 upstream comparator qualification (issue #15).
- CF-17 implementation.

## Exit

Normal merge after exact-head required checks. Issue #38 may close only after post-merge assurance evidence. Issue #100 stays open. CF-17 stays unauthorized until a later canonical Spec Kit authorizes it. The durable historical bytes remain unavailable, so closing #38 does not by itself authorize CF-17.
