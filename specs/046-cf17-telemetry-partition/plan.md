# CF-17 Telemetry Partition Plan

Status: AUTHORIZATION_SPEC

## Why this unit

G22 is still open after advertised-version acquisition. Extension, terminology, and availability observations are different evidence. This spec keeps them in separate documents and keeps those documents out of the compatibility vocabulary.

## Scope in

- Three schemas, evidence states `OBSERVED`, `MISSING`, and `UNSUPPORTED`, a 256-entry bound, and replay without network.

## Scope out of this pull request

- Implementation code, scale measurement, issue #100, issue #15, and CF-18.

## Exit

Normal merge after exact-head required checks. The merge authorizes the implementation package. CF-17 and CF-18 stay open.
