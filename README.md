# commandF

**Healthcare interoperability change intelligence.**

> **commandF tells you what your interoperability change will break — before you ship it.**

commandF starts with a CI-grade FHIR conformance package and breaking-change workflow, then grows into dependency impact, mapping review, loss analysis, and reproducible evidence.

## First slice: CF-01

```bash
commandf pkg resolve hl7.fhir.us.core@6.1.0
commandf pkg verify
```

CF-01 resolves FHIR NPM package dependencies, caches the original package archives by SHA-256, and writes a deterministic `commandf.lock`. Package archives are read directly rather than extracted to the filesystem.

## Build order

`pkg` → `inspect` → `diff` → breaking rules → `check`/SARIF → oracle differential testing → terminology diff → PR annotations → FSH source mapping → public IG delta corpus.

The long-term vision remains cross-standard, but a universal semantic IR is not a dependency of the first product. See `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md`.
