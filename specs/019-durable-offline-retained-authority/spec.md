# Durable Offline Retained Authority Recovery

Status: SPEC_CANDIDATE
Issue: #100
Canonical base: `73885a95200327db8a066a1d4381add7c570cb81`

## Authority boundary

This kit does not implement CF-17.
It does not modify `verify_artifacts` in `tools/af02-verifier/src/retained.rs`.
It does not add a path under `tools/af02-verifier/`, `specs/016-af-02-adversarial-test-strength/`, or `donors/`, because those prefixes are existing AF-02 authority and the A0 gate rejects unknown or mutated authority paths.
It does not claim the historical Actions artifact bytes were recovered.

Historical identity remains:

```text
artifact_id: 9255732702
artifact_name: cf10-real-corpus-evidence
workflow_run: 31916124080
recorded_sha256: 9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612
workflow_head: 5fe10d9859407272acf6649fc3e868d3eb2fbd12
workflow_conclusion: failure
```

## Forensic recovery result

Searched on 2026-09-30, after re-reading live `main` `73885a95200327db8a066a1d4381add7c570cb81`.

| Source | Result |
| --- | --- |
| Live run artifacts API for `31916124080` | `total_count = 0` |
| Artifact object `9255732702` | HTTP 404 |
| Repository Actions caches | `total_count = 0` |
| Releases | none |
| Git tags | none |
| Git history and remote branches | digest appears only as written metadata, not as an archive blob |
| Tracked `*.zip` / evidence archives | none matching this artifact |
| Local Downloads, Desktop, Documents, Cursor project and app data filename scan | no matching archive |
| WSL | not present |

Conclusion: `HISTORICAL_ARTIFACT_BYTES_UNAVAILABLE`. Identity metadata remains `HISTORICAL_ARTIFACT_IDENTITY_KNOWN`.

## Regeneration analysis

The generating workflow at `5fe10d9859407272acf6649fc3e868d3eb2fbd12` is `.github/workflows/cf10-real-corpus.yml`. It uploaded `cf10-real-corpus-evidence` with `actions/upload-artifact` `ea165f8d65b6e75b540449e92b4886f43607fa02` and `retention-days: 3`. Inputs were runner-local files under `/tmp`, including two full corpus runs, evidence directories, and status files. The runner image was `ubuntu-latest`. No checked-in archive format, compression level, file order, timestamp policy, or zip implementation pin was recorded beyond the Actions action SHA.

A newly generated zip is not recovery unless its SHA-256 equals `9fdde985bb5abbe53ec2bce2dadc5f65c95557f8848c9af68755fc81a45af612`. Logical similarity is classified only as `RECONSTRUCTED_SEMANTIC_EQUIVALENT_CANDIDATE`.

## Protocol

Packet schema: `commandf.durable-retained-authority/v1`.
Projection schema: `commandf.durable-retained-projection/v1`.

The checked-in historical packet is `historical-packet.json`. Its byte state is `HISTORICAL_ARTIFACT_BYTES_UNAVAILABLE` and `durable_bytes_sha256` is null.

These states stay separate:

- `HISTORICAL_ARTIFACT_IDENTITY_KNOWN`
- `HISTORICAL_ARTIFACT_BYTES_VERIFIED`
- `HISTORICAL_ARTIFACT_BYTES_UNAVAILABLE`
- `DURABLE_PACKET_VERIFIED`
- `OFFLINE_REPLAY_VERIFIED`

Trusted head, base, artifact, workflow, and Git blob identities are caller-supplied authority. A packet cannot authorize itself by repeating edited constants. Live artifact identity and digest mismatches fail closed. `verify_artifacts` remains the candidate-input verifier and is not used as a substitute for missing durable bytes.

## Exit

This kit closes only after exact-head CI is green, `verify_artifacts` is unchanged, the historical packet still says bytes are unavailable, and post-merge assurance checks are green. It does not close issue #100's historical byte verification, because those bytes were not recovered.
