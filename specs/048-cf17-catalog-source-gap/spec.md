# CF-17 Catalog Source Gap

Status: NOT_EXECUTION_AUTHORITY

Observed canonical main: `d14c5ecebf211c6cb45f22498e7d0cf9cabbe1d9`

This record states why a bounded official multi-package catalog Spec Kit cannot be shaped from the sources already authorized for CF-17 acquisition. It does not authorize a catalog endpoint, a crawl, AF-03, AF-04, CF-18, issue #100, or issue #15.

## What is already authorized

`specs/043-cf17-registry-acquisition/` authorizes two hosts only:

- `packages.fhir.org`
- `packages2.fhir.org`

Those hosts are authorized for one exact package at a time, and, in `specs/045-cf17-advertised-versions/`, for the version listing of one package name. Spec 043 sets feed entries in that unit to 0. Spec 045 says no other host, mirror, or branch is a source.

Neither spec names a catalog URL, a feed index URL, or a package-name listing document on those hosts.

## Why a new catalog spec would be a guess

A multi-package catalog needs an exact document the official source itself returns: one URL, one response, one digest. Canonical CF-17 acquisition authority does not contain that URL.

`docs/COMMAND_F_OPEN_SOURCE_QUALIFICATION_2026-09-12.md` names `FHIR/ig-registry` and `package-feeds.json` as an official-reference candidate for CF-17, in pinned-data mode. The same review says candidates are not adopted dependencies. No canonical Spec Kit pins a commit, a blob digest, or a license record for that file. A mutable GitHub branch is not an acquisition identity.

Using that candidate to discover further feed URLs would be a second hop, not one bounded official response. This record does not adopt the candidate and does not invent a path on `packages.fhir.org`.

## Coverage that must not be claimed

`COMPLETE_REGISTRY_HISTORY` remains undefined. `OFFICIAL_CATALOG_SNAPSHOT`, `OFFICIAL_FEED_WINDOW`, `PARTIAL_OFFICIAL_CATALOG`, and `SOURCE_DECLARED_COMPLETE` are not issued here, because no source document was acquired.

## AF path, recorded and not opened

The playbook dependency graph is AF-02, then AF-03, then AF-04. `specs/016-af-02-adversarial-test-strength/` is `PLANNING_CANDIDATE`. Its tasks T001 through T006 are unchecked. T006 is the gate that would mark AF-02 planning canonical and authorize Stack A0 only. That gate has not been passed. AF-03 has no Spec Kit because it is not the first open unit on that chain. This record does not implement AF-02 and does not create AF-03 or AF-04. Scale measurement stays deferred.

## Next action

Do not implement catalog code from this record. A later catalog Spec Kit needs an exact official document identity that is already inside an authorized host, or a separate adoption record that pins commit, blob digest, and rights before any request. Until then the catalog row stays `NOT_AUTHORIZED`.
