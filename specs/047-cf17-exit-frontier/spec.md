# CF-17 Exit Frontier

Status: NOT_EXECUTION_AUTHORITY

Observed canonical main: `93a182428533ae8c7b2ab5569fcb07d445f880cd`

This record recomputes the CF-17 requirement table after `specs/046-cf17-telemetry-partition/`. It does not implement code. It does not authorize a registry catalog, a scale measurement, or CF-18. It does not close issue #100 or issue #15. It does not change the CF-06 production pin. It does not replace `specs/042-cf17-exit-gap-record/`. That earlier record remains the historical statement that acquisition was unauthorized before `specs/043-cf17-registry-acquisition/`.

## Coverage that stays bounded

One exact official package is closed for that requirement. A historical graph of caller-supplied exact versions is closed for that requirement and remains `SUPPLIED_VERSIONS_ONLY`. The official version listing for one package name, with a verified archive for every advertised exact version, is closed for that requirement and remains `ADVERTISED_BY_OFFICIAL_SOURCE`.

None of those markers means `COMPLETE_REGISTRY_HISTORY`. A multi-package catalog remains outside `specs/043-cf17-registry-acquisition/` and `specs/045-cf17-advertised-versions/`.

## Why scale is not the next executable unit

The playbook dependency graph is AF-02, then AF-03, then AF-04. `specs/016-af-02-adversarial-test-strength/` is still `PLANNING_CANDIDATE`. It is not `CLOSED_CANONICAL`. No AF-03 Spec Kit exists. Quantitative scale claims stay deferred. This record does not create AF-03 or AF-04.

## Why CF-17 is not closed

The observatory must-ship list still includes an immutable registry/feed snapshot beyond one package name, and it still requires scale measurement only after AF-04. Those two requirements are not proven. CF-18 stays closed.

## Checklist

| CF17_REQUIREMENT | STATE | EVIDENCE | BLOCKER | NEXT_ACTION |
| --- | --- | --- | --- | --- |
| Snapshot identity schema | CLOSED_FOR_THIS_REQUIREMENT | `specs/025-cf17-snapshot-identity/` merged as `1f4e45405505406d3d57d47d4773bbb23559f366` | none for the schema | none |
| Separate package and canonical closures | CLOSED_FOR_THIS_REQUIREMENT | `specs/026-cf17-separate-closures/` merged as `5846a9742ba21b6bbd45138dd7c31ff25792fe60` | none | none |
| Closure query identity | CLOSED_FOR_THIS_REQUIREMENT | `specs/027-cf17-closure-query/` merged as `296641b021269adb35506867b186a6b79cfa799a` | none | none |
| Source lifecycle and adoption refusal | CLOSED_FOR_THIS_REQUIREMENT | `specs/028-cf17-source-lifecycle/` merged as `2384be6483943dd8ba3311bc27c63aefeb98e266` | none | none |
| Cache identity without a scale claim | CLOSED_FOR_THIS_REQUIREMENT | `specs/029-cf17-cache-identity/` merged as `11e3bf80bf8ac168cd1c85b9ffce5958c8fbe87d` | scale measurement is a different requirement | none for identity |
| Snapshot comparison without compatibility labels | CLOSED_FOR_THIS_REQUIREMENT | `specs/030-cf17-snapshot-comparison/` merged as `4ca49836e7f1419f7555eb3504ee3694af494025` | none | none |
| Ordered comparison history | CLOSED_FOR_THIS_REQUIREMENT | `specs/031-cf17-snapshot-history/` merged as `87676ac4d9b584d20be2948d9ac2339cd8dfe2b3` | none | none |
| History, snapshot, lifecycle, closure, query, comparison, cache, and workspace machine bytes | CLOSED_FOR_THIS_REQUIREMENT | `specs/032` through `specs/040` | none for those documents | none |
| Workspace membership identity | CLOSED_FOR_THIS_REQUIREMENT | `specs/039-cf17-workspace-identity/` merged as `33ac488018c36b538482ddbf8d513d216f0b795a` | none | none |
| Offline replay of one frozen published observation | CLOSED_FOR_THIS_REQUIREMENT | `specs/041-cf17-offline-replay/` merged as `eb83dce4a7e4430c8adc2a9ae37a9ecf8ba84648` | replay does not fetch a registry | none |
| Mutable CI cannot be published snapshot authority | CLOSED_FOR_THIS_REQUIREMENT | `require_published_authority` in `crates/commandf-pkg/src/ecosystem_snapshot.rs` | none | none |
| Unresolved canonicals survive serialization | CLOSED_FOR_THIS_REQUIREMENT | snapshot identity tests retain unresolved canonicals | none | none |
| One exact official package | CLOSED_FOR_THIS_REQUIREMENT | `specs/043-cf17-registry-acquisition/` implementation merged as `9adc556fdc02faf8e99661a23da78cc37ddec526` | not a catalog | none for this one package |
| Historical graph of supplied exact versions | CLOSED_FOR_THIS_REQUIREMENT | `specs/044-cf17-historical-package-graph/` implementation merged as `684eba49258e46d809d68259cd8029bd70659dd2`; coverage `SUPPLIED_VERSIONS_ONLY` | not an official listing | none for supplied versions |
| Advertised exact versions for one package name | CLOSED_FOR_THIS_REQUIREMENT | `specs/045-cf17-advertised-versions/` implementation merged as `e48f3c119bda127765cb3e648a76f29b3f202d99`; coverage `ADVERTISED_BY_OFFICIAL_SOURCE` | not `COMPLETE_REGISTRY_HISTORY` | none for that one name |
| Extension, terminology, and availability telemetry partition | CLOSED_FOR_THIS_REQUIREMENT | `specs/046-cf17-telemetry-partition/` implementation merged as `93a182428533ae8c7b2ab5569fcb07d445f880cd` | telemetry is not a compatibility verdict | none for the partition |
| Multi-package registry or feed catalog | NOT_AUTHORIZED | playbook must-ship still names an immutable registry/feed snapshot; specs 043 and 045 exclude enumeration | no canonical Spec Kit scopes a bounded catalog in, and whole-registry enumeration is not authorized | do not crawl a catalog from this record |
| Incremental cache scale measurement | DEFERRED_BY_PLAYBOOK | playbook: measure scale only after AF-04 | AF-02 is `PLANNING_CANDIDATE`, not `CLOSED_CANONICAL`; AF-03 has no Spec Kit | do not claim scale and do not start AF-04 |
| Branch-range comparison | OUT_OF_CURRENT_CONTRACT | playbook: branch names are not semantic authority | not named as the next unit | do not invent it here |
| CF-18 decision contract | NOT_STARTED | playbook order is CF-18 after CF-17 | CF-17 exit is not proven | do not start CF-18 |
| Issue #100 historical bytes | OPEN | artifact `9255732702`, digest `sha256:9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612`, bytes unavailable | bytes are not in the repository | do not fabricate bytes |
| Issue #15 production pin | OPEN | production pin remains unchanged | upstream qualification, frozen CF-10, and review are not recorded as complete | do not change the pin |
