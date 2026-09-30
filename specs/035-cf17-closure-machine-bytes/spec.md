# CF-17 Closure Machine Bytes

Status: SPEC_CANDIDATE

Slice: eleventh CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- A published closure observation has two documents. They are not one wrapper.
- The package-dependency document is the compact JSON already hashed as `package_closure_sha256`.
- Its field order is `edges`, `kind`, `members`, `schema`, `snapshot_sha256`, `unrelated_packages`.
- Its `kind` is `package-dependency`.
- The canonical-reference document is the compact JSON already hashed as `canonical_closure_sha256`.
- Its field order is `ambiguous_canonicals`, `edges`, `kind`, `resolved_canonicals`, `schema`, `snapshot_sha256`, `unresolved_canonicals`.
- Its `kind` is `canonical-reference`.
- The encoder identity is `commandf.ecosystem-closure-bytes/v1`. It is recorded beside the bytes and is not hashed into either document.
- Reordering the same edges before projection does not change either document.
- A package edge changes only the package document when the canonical witness set is unchanged. This slice still treats a removed canonical edge as a canonical-document change.
- Whitespace, an extra field, a wrong kind, a digest mismatch, or a mutable snapshot fails closed.
- Either document larger than 16 MiB fails before parsing.
- The documents have no clock, path, locale, or compatibility field.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
