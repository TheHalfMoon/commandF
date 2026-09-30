# CF-17 Offline Replay

Status: SPEC_CANDIDATE

Slice: seventeenth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- One frozen published observation is projected twice from the same logical inputs.
- The second projection produces the same snapshot, closure, lifecycle, workspace, and cache digests.
- Reordering packages, lifecycle sources, or workspace members before replay does not change those digests.
- A changed archive digest changes the snapshot digest.
- Mutable CI cannot be replayed as published authority.
- The replay schema is `commandf.ecosystem-replay/v1`. It names the existing digests. It does not add a second hashed wrapper.
- The replay does not read the clock, a branch name, the filesystem order, a locale, or the network.
- This slice does not download a registry or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
