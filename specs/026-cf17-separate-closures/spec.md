# CF-17 Separate Closures

Status: SPEC_CANDIDATE

Slice: second CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- The input universe is one already projected ecosystem snapshot.
- Package-dependency closure and canonical-reference closure are separate documents and separate SHA-256 identities.
- A package edge changes the package-closure identity and leaves the canonical-closure identity unchanged.
- A canonical edge changes the canonical-closure identity and leaves the package-closure identity unchanged.
- Reordering the same edges does not change either identity.
- Package-edge endpoints must already exist in the snapshot. A missing endpoint fails closed and is not invented.
- A self-dependency or a duplicate edge fails closed.
- Snapshot packages that are not endpoints of a package-dependency edge stay in `unrelated_packages`. A canonical edge does not move a package into the dependency member set.
- Snapshot unresolved canonicals stay unresolved. An edge that resolves the same canonical fails closed as conflicting status.
- `RESOLVED`, `UNRESOLVED`, and `AMBIGUOUS` remain distinct witness classes.
- `MUTABLE_CI` cannot enter a published closure.
- Edge count is bounded by 10000 per closure kind. A canonical token is bounded by 2048 characters.
- This slice does not download a registry, walk package bytes, or build the CF-11 context extractor.
- It does not close issue #100 or issue #15, and it does not change the CF-06 production pin or `verify_artifacts`.
