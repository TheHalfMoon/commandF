# CF-17 Advertised Version Acquisition Plan

Status: AUTHORIZATION_SPEC

## Why this unit

`specs/044-cf17-historical-package-graph/` can order versions the caller supplies. It cannot say those versions were advertised by an official registry. G01 still needs that listing for one package name, with the verified listing bytes bound to every accepted archive.

## Scope in

- One package name, the official listing body, exact advertised versions, same-host archives, a 256-version bound, and the coverage marker `ADVERTISED_BY_OFFICIAL_SOURCE`.

## Scope out of this pull request

- Implementation code, a registry-wide crawl, telemetry, scale numbers, issue #100, issue #15, and CF-18.

## Exit

Normal merge after exact-head required checks. The merge authorizes the implementation package. CF-17 and CF-18 stay open.
