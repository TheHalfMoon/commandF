# CF-17 Exit Gap Record

Status: NOT_EXECUTION_AUTHORITY

This record does not implement code. It does not authorize official registry acquisition. It does not start CF-18. It does not close issue #100 or issue #15. It does not change the CF-06 production pin.

## Why official registry acquisition is must-ship

`docs/COMMAND_F_V3_EXECUTION_PLAYBOOK.md` owns gap G01 on CF-17. The closure contract is: reproducible immutable registry/feed snapshots and a historical package graph exist with provenance and replay.

The same card's must-ship list includes immutable IG-registry/package-feed snapshots. Its minimum work packages include: ingest official registry/feed sources with exact digest and retrieval evidence.

`docs/COMMAND_F_PLAN_INDEX.md` states that V3 planning text is not execution authority by itself. `specs/018-v3-1-authority-migration/spec.md` is the gate that makes the playbook the forward gap map. Point 3 of that gate allows a CF-17 slice only after the Wave -1 unit that slice depends on is closed or explicitly deferred by a later canonical Spec Kit.

## Why it is not executable on this main

`specs/001-cf-01-package-resolution/convergence.md` already records bounded public FHIR registry acquisition and local-mirror acquisition for package resolution. That is CF-01 lock/cache acquisition. It is not a CF-17 immutable registry/feed snapshot with retrieval provenance.

Every CF-17 package from `specs/025-cf17-snapshot-identity/` through `specs/040-cf17-workspace-machine-bytes/` lists live registry ingestion as scope out.

`specs/041-cf17-offline-replay/plan.md`, merged as `eb83dce4a7e4430c8adc2a9ae37a9ecf8ba84648`, is the current exit. It says official registry or feed acquisition stays unauthorized until a later spec says otherwise.

No later spec exists. This record is not that later spec.

Issue #100 remains open because historical Actions artifact bytes are unavailable. `specs/019-durable-offline-retained-authority/` does not authorize CF-17. The repository does not say that issue #100 is the Wave -1 dependency of registry snapshot acquisition. That link is not invented here.

Issue #15 remains open. It qualifies one CF-06 production pin. It is not the registry-snapshot authorization record.

Scale measurement stays deferred by the playbook sentence that says measure scale only after AF-04.

## Missing authorization dependency

The missing artifact is a later canonical Spec Kit whose scope in is official registry/feed snapshot acquisition, and which names the Wave -1 unit that slice depends on as closed or explicitly deferred.

Until that kit exists, registry acquisition is must-ship and not executable.

## Checklist

| CF17_REQUIREMENT | STATE | EVIDENCE | BLOCKER | NEXT_ACTION |
| --- | --- | --- | --- | --- |
| Snapshot identity schema | CLOSED_FOR_THIS_REQUIREMENT | `specs/025-cf17-snapshot-identity/` merged as `1f4e45405505406d3d57d47d4773bbb23559f366` | none for the schema | none |
| Separate package and canonical closures | CLOSED_FOR_THIS_REQUIREMENT | `specs/026-cf17-separate-closures/` merged as `5846a9742ba21b6bbd45138dd7c31ff25792fe60` | none | none |
| Closure query identity | CLOSED_FOR_THIS_REQUIREMENT | `specs/027-cf17-closure-query/` merged as `296641b021269adb35506867b186a6b79cfa799a` | none | none |
| Source lifecycle and adoption refusal | CLOSED_FOR_THIS_REQUIREMENT | `specs/028-cf17-source-lifecycle/` merged as `2384be6483943dd8ba3311bc27c63aefeb98e266` | none | none |
| Cache identity without scale claim | CLOSED_FOR_THIS_REQUIREMENT | `specs/029-cf17-cache-identity/` merged as `11e3bf80bf8ac168cd1c85b9ffce5958c8fbe87d` | scale measurement is a different requirement | none for identity |
| Snapshot comparison without compatibility labels | CLOSED_FOR_THIS_REQUIREMENT | `specs/030-cf17-snapshot-comparison/` merged as `4ca49836e7f1419f7555eb3504ee3694af494025` | none | none |
| Ordered comparison history | CLOSED_FOR_THIS_REQUIREMENT | `specs/031-cf17-snapshot-history/` merged as `87676ac4d9b584d20be2948d9ac2339cd8dfe2b3` | none | none |
| History, snapshot, lifecycle, closure, query, comparison, cache, and workspace machine bytes | CLOSED_FOR_THIS_REQUIREMENT | `specs/032` through `specs/040` | none for those documents | none |
| Workspace membership identity | CLOSED_FOR_THIS_REQUIREMENT | `specs/039-cf17-workspace-identity/` merged as `33ac488018c36b538482ddbf8d513d216f0b795a` | none | none |
| Offline replay of one in-memory published observation | CLOSED_FOR_THIS_REQUIREMENT | `specs/041-cf17-offline-replay/` merged as `eb83dce4a7e4430c8adc2a9ae37a9ecf8ba84648` | replay does not fetch a registry | none |
| Mutable CI cannot be published authority | CLOSED_FOR_THIS_REQUIREMENT | `require_published_authority` in `crates/commandf-pkg/src/ecosystem_snapshot.rs` | none | none |
| Unresolved canonicals survive serialization | CLOSED_FOR_THIS_REQUIREMENT | snapshot identity tests retain unresolved canonicals | none | none |
| Official registry/feed snapshot acquisition | MUST_SHIP_NOT_AUTHORIZED | playbook G01 and the CF-17 ingest work package; spec 041 exit | no canonical Spec Kit scopes this acquisition in | do not implement from this record |
| Historical registry package graph with retrieval provenance | OPEN | G01 closure contract is not met by CF-01 single-resolution acquisition | same missing Spec Kit as the row above | do not implement from this record |
| Incremental cache scale measurement | DEFERRED_BY_PLAYBOOK | playbook says measure scale only after AF-04 | AF-04 evidence is not this slice | do not claim scale |
| Extension, terminology, and availability telemetry partition | OPEN | G22 names the partition; specs 025-041 do not implement it | no current spec exit names it as the next executable unit | do not invent that unit here |
| Branch-range comparison | OUT_OF_CURRENT_CONTRACT | playbook says branch names are not semantic authority | not named as the next unit by spec 041 | do not invent it here |
| CF-18 decision contract | NOT_STARTED | playbook order is CF-18 after CF-17 | CF-17 exit is not proven while G01 acquisition is open | do not start CF-18 |
| Issue #100 historical bytes | OPEN | artifact `9255732702`, digest `sha256:9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612`, bytes unavailable | bytes are not in the repository | do not fabricate bytes |
| Issue #15 production pin | OPEN | production pin remains unchanged | upstream qualification, frozen CF-10, and review are not recorded as complete | do not change the pin |
