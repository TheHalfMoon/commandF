# Local Mirror Archive Bound

Status: SPEC_CANDIDATE

Issue: #38

## Problem

`LocalMirrorSource::archive` read `package.tgz` with unbounded `fs::read`. Registry acquisition already stops a compressed archive body at 128 MiB.

## Contract

- Local mirror acquisition uses the same 128 MiB compressed limit as the registry.
- The read stops after that limit plus one byte.
- A file of exactly the tested bound is returned.
- One extra byte fails closed with a size error and is not parsed as an archive manifest.
- A missing archive stays `PackageNotFound`.
- The size error does not include the host path.
- Cache verified-byte paths and retained historical artifact bytes are out of scope.
- Issue #100 stays open. This candidate does not authorize CF-17 and does not change `verify_artifacts`.
