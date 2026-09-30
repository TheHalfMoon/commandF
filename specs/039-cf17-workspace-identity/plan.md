# CF-17 Workspace Identity Plan

Status: SPEC_CANDIDATE

## Why this unit

Cache-identity machine bytes are merged as `dc479b0806979710999b0fdb119dc264aa795ea4`. The CF-17 playbook still requires explicit multi-package workspace identity. Existing machine-byte packages do not record which snapshot packages form one workspace. Live registry ingestion remains a separate later package.

## Scope in

- Deterministic workspace membership bound to one published snapshot digest.

## Scope out

- Live registry ingestion.
- Branch names as authority.
- Compatibility classification.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is canonical machine bytes for one workspace identity. It still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
