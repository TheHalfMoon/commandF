# CF-17 Historical Package Graph Plan

Status: AUTHORIZATION_SPEC

## Why this unit

`specs/043-cf17-registry-acquisition/` acquires one exact official package. The G01 closure still asks for a historical package graph with provenance and replay. This spec scopes that graph in for versions already identified by exact archive digests.

## Scope in

- Deterministic ordering, recomputed snapshot identity, a 256-version bound, and the explicit `SUPPLIED_VERSIONS_ONLY` coverage marker.

## Scope out of this pull request

- Implementation code, network access, a claim of registry completeness, telemetry, scale measurement, issue #100, issue #15, and CF-18.

## Exit

Normal merge after exact-head required checks. The merge authorizes the implementation package and does not itself build the graph. CF-17 and CF-18 stay open.
