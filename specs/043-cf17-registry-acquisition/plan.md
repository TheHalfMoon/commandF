# CF-17 Official Registry Acquisition Plan

Status: AUTHORIZATION_SPEC

## Why this unit

`specs/042-cf17-exit-gap-record/` records that G01 official registry acquisition is must-ship and had no Spec Kit that scoped it in. `specs/041-cf17-offline-replay/plan.md` leaves that work unauthorized until a later spec says otherwise. This is that spec.

## Wave -1 dependency

The verified-byte units this acquisition depends on are already closed. Issue #100 is explicitly deferred: its unavailable historical artifact is not an input of this slice. This sentence is the deferral required by `specs/018-v3-1-authority-migration/spec.md` point 3.

## Scope in

- The authorized hosts, the single allowed redirect, the exact-version rule, the verified-byte sequence, the numeric bounds, the typed failures, and the offline replay rule for one package observation.

## Scope out of this pull request

- Registry client changes and new Rust modules.
- A catalog crawl, a historical package graph, telemetry partitions, scale numbers, issue #100, issue #15, and CF-18.

## Exit

Normal merge of this Spec Kit after exact-head required checks. The merge authorizes a following implementation package and does not itself perform acquisition. Issue #100 and issue #15 stay open. CF-17 stays open because G01 is not yet implemented. CF-18 stays closed.
