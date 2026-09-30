# CF-17 Snapshot Comparison

Status: SPEC_CANDIDATE

Slice: sixth CF-17 package. It does not finish the Ecosystem Observatory.

## Contract

- The inputs are two published snapshot identities, an explicit comparison engine schema, and optional closure and lifecycle witnesses.
- The comparison SHA-256 covers the canonical comparison document and does not include itself.
- Repeating the same before snapshot, after snapshot, engine schema, and witnesses returns the same comparison identity.
- `compare(before, after)` and `compare(after, before)` differ when membership differs.
- Comparing one snapshot with itself records every package as unchanged and records no added or removed membership.
- A package name and version present only on the after side is added exactly once.
- A package name and version present only on the before side is removed exactly once.
- The same name and version with a different archive digest, source identity, or mutability is changed. The record keeps both sides. It does not assign compatibility.
- Unresolved canonical membership is introduced, removed, or retained exactly once.
- Reordering the same snapshot records does not change the comparison after snapshot identity verification.
- A snapshot whose stored digest does not match its document fails closed.
- A stored comparison that does not match a fresh projection fails closed.
- A different comparison engine schema changes the comparison identity.
- Closure witnesses are both present or both absent. A closure bound to the other snapshot fails closed.
- When both closures are present, the comparison records their package and canonical closure digests and resolution status changes. Absent closure evidence stays explicit.
- Lifecycle witnesses are both present or both absent. The comparison records the recorded source states and does not read a clock.
- Duplicate package identity fails closed.
- More than 10000 packages, unresolved canonicals, closure records, or lifecycle sources on one side fails closed before comparison work.
- Comparison JSON above 16 MiB fails closed.
- The comparison vocabulary is membership and evidence state. It does not emit compatibility or consumer judgments.
- This slice does not download a registry, run an oracle, or change the CF-06 production pin.
- It does not close issue #100 or issue #15.
