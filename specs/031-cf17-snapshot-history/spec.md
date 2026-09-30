# CF-17 Snapshot History

Status: SPEC_CANDIDATE

Slice: seventh CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- A history is an ordered list of already projected snapshot comparisons plus an explicit history engine schema.
- Each comparison digest must recompute from its own document.
- Step N+1 must start at the snapshot SHA-256 where step N ended.
- The same ordered list and engine schema replay to the same history SHA-256.
- A reversed chain that still connects has a different history identity.
- An empty list is an explicit history with no steps. It does not invent a snapshot.
- A broken chain, a tampered comparison digest, or more than 1000 steps fails closed.
- The history does not assign compatibility or consumer impact.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
