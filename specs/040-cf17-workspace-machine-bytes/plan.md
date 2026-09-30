# CF-17 Workspace Machine Bytes Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/039-cf17-workspace-identity/` is merged as `33ac488018c36b538482ddbf8d513d216f0b795a`. Its plan names canonical machine bytes for one workspace identity as the next CF-17 package. The bytes are the workspace document that package already hashes.

## Scope in

- Canonical encode and decode of one workspace identity bound to one published snapshot.

## Scope out

- Live registry ingestion.
- Branch names as authority.
- Compatibility classification.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. Re-read the CF-17 playbook before naming the next package. Live registry ingest stays unauthorized. Issue #100 and issue #15 stay open. CF-18 stays closed.
