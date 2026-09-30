# CF-17 Catalog Source Gap Plan

Status: NOT_EXECUTION_AUTHORITY

## Why this unit

`specs/047-cf17-exit-frontier/` left the multi-package catalog as `NOT_AUTHORIZED` because no Spec Kit scoped it in. This record checks whether that Spec Kit can be shaped from the hosts in `specs/043-cf17-registry-acquisition/`. It cannot, for the reasons in the spec. The record is the decision, not the missing authorization.

## Scope in

- The source-gap statement.
- The statement that AF-03 stays behind the unfinished AF-02 planning gate.

## Scope out

- Product code, a catalog client, a new host, AF-02 implementation, AF-03, AF-04, issue #100, issue #15, and CF-18.

## Exit

Normal merge after exact-head required checks. The merge does not authorize implementation.
