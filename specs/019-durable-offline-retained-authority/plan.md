# Plan — Durable Offline Retained Authority

## In

- Forensic record that the historical zip is unavailable.
- Versioned packet and projection in `commandf-pkg`, outside the AF-02 authority prefixes.
- Tests for mismatch, partial, malformed, self-authorization, offline replay, unchanged CF-10 snapshot, and non-matching regeneration.
- Canonical historical packet with byte state unavailable.

## Out

- Editing `verify_artifacts`.
- Inventing artifact bytes.
- Relabeling run `31916124080` as a pass.
- CF-17 implementation.
- Closing #100 as byte-verified.

## Next after this kit

Keep #100 open for historical byte recovery if a digest-matching copy appears later. The mechanism can accept those bytes only when SHA-256 equals the recorded digest. Verified-byte issues #35, #36, #37, #38, and #40 remain the next integrity units and still block CF-17.
