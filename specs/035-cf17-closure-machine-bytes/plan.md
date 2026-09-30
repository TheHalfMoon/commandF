# CF-17 Closure Machine Bytes Plan

Status: SPEC_CANDIDATE

## Why this unit

`specs/034-cf17-lifecycle-machine-bytes/` is merged as `f53b13616be9b4074a720aa30b997def248e687f`. Its plan names canonical machine bytes for one published closure pair as the next CF-17 package. Each closure already has its own digest. This slice exposes those two documents.

## Scope in

- Canonical encode and decode of the package-dependency document and the canonical-reference document.

## Scope out

- Live registry ingestion.
- Compatibility classification.
- A new wrapper digest that would fork the two existing closure identities.
- Issue #100 byte recovery and issue #15 pin qualification.
- CF-18 coverage.

## Exit

Normal merge after exact-head required checks. The next CF-17 package is canonical machine bytes for one closure query, using the same verified-byte rule. It still does not ingest a registry or classify compatibility. Issue #100 and issue #15 stay open.
