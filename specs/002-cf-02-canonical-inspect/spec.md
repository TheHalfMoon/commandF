# CF-02 — Canonical Package Inspection

Status: Draft

## Outcome

From a package already locked and cached by CF-01:

```bash
commandf inspect hl7.fhir.r4.core@4.0.1 --format json
```

returns a deterministic, offline inventory of FHIR resources, canonical identities, exact resource hashes, and StructureDefinition element addresses.

## Requirements

1. `inspect` performs no package acquisition. It reads `commandf.lock` and verifies the CF-01 cache object before parsing.
2. Rebuild the resource inventory from actual package JSON resources. `package/.index.json` is optional derived metadata, never authority.
3. Exclude package metadata and `package/examples/**` from the conformance inventory by default.
4. Record for each resource: archive-relative filename, `resourceType`, optional `id`, optional top-level string `url`, optional top-level string `version`, and SHA-256 over exact archived resource bytes.
5. Canonical identity is `url`; when declared, canonical version qualifies it as `url|version`. Do not use `meta.versionId` and do not invent missing identity fields.
6. Distinct explicit versions may share one URL. Duplicate fully-qualified canonical identities, or duplicate unversioned canonical URLs, are explicit ambiguity/errors.
7. For StructureDefinition, inspect existing `snapshot.element[]` and `differential.element[]` only. Do not generate snapshots.
8. Exact `ElementDefinition.id` is the primary stable element address; preserve the view plus optional `path` and `sliceName`. Missing or duplicate ids in one view fail indexing; commandF does not synthesize ids.
9. JSON is the authoritative machine output. Identical verified inputs must serialize byte-identically with stable ordering.
10. Reuse CF-01 archive safety bounds or stricter equivalents. Malformed JSON, invalid identity-field types, and indexing ambiguity fail explicitly.
11. Repository tests use synthetic/public fixtures only; no PHI.

## Success criteria

- repeat inspection is byte-identical;
- inspection works offline;
- one real published FHIR package is inspected end-to-end;
- a sliced StructureDefinition fixture yields stable addresses from exact element ids;
- missing `.index.json` does not block inspection;
- no FHIR validation, snapshot generation, diff, risk classification, terminology execution, mapping execution, graph construction, or AI runtime is introduced.

## Standards constraints

HL7 FHIR NPM package layout; CanonicalResource/canonical URL and optional `|version`; StructureDefinition/ElementDefinition exact element ids including slicing/reslicing.
