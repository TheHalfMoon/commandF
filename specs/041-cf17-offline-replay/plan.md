# CF-17 Offline Replay Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/040-cf17-workspace-machine-bytes/` is merged as `7d38d073286bd72277bb165f42ec30ac0b7f925c`. The CF-17 playbook exit asks two independent replays of the same frozen snapshot to produce equivalent identities. Live registry ingest remains unauthorized, so this package replays inputs that are already in memory.

## Scope in

- One deterministic replay of snapshot, closure, lifecycle, workspace, and cache identity.

## Scope out

- Live registry ingestion.
- Compatibility classification.
- Scale measurement.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The playbook must-ship item that still remains is official registry or feed acquisition. That item stays unauthorized until a later spec says otherwise. Issue #100 and issue #15 stay open. CF-18 stays closed.
