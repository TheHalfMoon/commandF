# CF-17 Comparison Machine Bytes

Status: SPEC_CANDIDATE

Slice: thirteenth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- Machine bytes are the compact UTF-8 JSON document already hashed as `comparison_sha256`.
- Field order follows the existing comparison document: added packages, closure digests, snapshots, changed packages, evidence flags, engine schema, unresolved canonicals, lifecycle, resolution, schema, and unchanged packages.
- The encoder identity is `commandf.ecosystem-comparison-bytes/v1`. It is recorded beside the bytes and is not a second hashed wrapper.
- Reversing the two snapshots changes the bytes.
- Parsing those bytes and writing them again produces the same bytes.
- Whitespace, an extra field, a digest mismatch, or a document larger than 16 MiB fails closed.
- The document records membership and identity facts only. It has no compatibility label, clock, path, or locale field.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
