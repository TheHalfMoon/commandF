# CommandF

**Healthcare interoperability change intelligence.**

> **CommandF helps engineers find and explain breaking FHIR conformance changes before deployment.** It reads versioned FHIR package archives, classifies supported structural changes, traverses the declared package graph, and emits machine-readable findings.

**Status: experimental source-checkout CLI.** The implemented Rust tools are usable for local interoperability engineering experiments. This repository does **not** yet publish supported binaries, a stable public release, a complete multi-consumer `review` command, a Studio application, clinical safety certification, or a general FHIR R4B/R5/R6 compatibility guarantee. See the roadmap for proposed capabilities; the plans are not implementation evidence.

## Start locally

Prerequisites: Rust toolchain **1.97.1**, Cargo, and Git. Java is required only when invoking the separately pinned HL7 oracle adapter. Default core package analysis does not require an LLM, a hosted API, or a cloud account.

```bash
git clone https://github.com/TheHalfMoon/commandF.git
cd commandF
cargo test --workspace --all-features --locked
cargo run --locked -p commandf -- --help
```

Windows note: a recorded local nested AF-02 verifier test fails under a deep checkout path with `LNK1104` / the Windows legacy path-length limit. Keep the checkout and `CARGO_TARGET_DIR` paths short while that issue is being qualified. Successful local tests are not, by themselves, a cross-platform release qualification.

There is currently no root `LICENSE` file. Do not infer external redistribution rights from the availability of the source.

## Package acquisition and verification

A single publicly available FHIR package can be acquired by exact name/version:

```bash
cargo run --locked -p commandf -- pkg resolve hl7.fhir.us.core@6.1.0 \
  --lock commandf.lock --cache .commandf/cache

cargo run --locked -p commandf -- pkg verify \
  --lock commandf.lock --cache .commandf/cache

cargo run --locked -p commandf -- inspect hl7.fhir.us.core@6.1.0 \
  --lock commandf.lock --cache .commandf/cache

cargo run --locked -p commandf -- context \
  --lock commandf.lock --cache .commandf/cache
```

Acquisition normally contacts a FHIR package registry. `pkg resolve --source-dir <local-mirror>` instead uses a prepared local mirror. `pkg verify` and the downstream commands use the lockfile and cached package bytes without a mandatory remote lookup.

Digest verification proves that local bytes match the recorded digest; **it does not independently prove publisher identity, package namespace ownership, or absence of malicious source content**. Package-authenticity and namespace-confusion analysis are future hardening work.

## Compare two package versions

The commands below use **illustrative placeholders**, not known registered package identities. Replace `example.ig@1.0.0`, `example.ig@2.0.0`, and `example.ig` with a real package and two real published versions.

```bash
cargo run --locked -p commandf -- pkg resolve example.ig@1.0.0 \
  --lock before.lock --cache .commandf/before-cache

cargo run --locked -p commandf -- pkg resolve example.ig@2.0.0 \
  --lock after.lock --cache .commandf/after-cache

cargo run --locked -p commandf -- diff example.ig \
  --before-lock before.lock --before-cache .commandf/before-cache \
  --after-lock after.lock --after-cache .commandf/after-cache

cargo run --locked -p commandf -- classify example.ig \
  --before-lock before.lock --before-cache .commandf/before-cache \
  --after-lock after.lock --after-cache .commandf/after-cache

cargo run --locked -p commandf -- impact example.ig \
  --before-lock before.lock --before-cache .commandf/before-cache \
  --after-lock after.lock --after-cache .commandf/after-cache

cargo run --locked -p commandf -- check example.ig \
  --before-lock before.lock --before-cache .commandf/before-cache \
  --after-lock after.lock --after-cache .commandf/after-cache \
  --direction both --fail-on breaking --format sarif --output findings.sarif
```

The `impact` command requires the version-2 lockfile schema on **both** sides. The `check` command reports status `0` for policy pass, `2` for policy fail, and `1` for operational errors; malformed command usage is normalized to `1` for `check`. A pass is bounded to implemented rules and inputs, **not** proof of universal FHIR compatibility. SARIF output is available from `check`; the separate `gate` command supports JSON reports and optionally supplied baseline and suppression files.

## Command map (implemented in the Rust CLI)

| Command | Current purpose | Important limitation |
| --- | --- | --- |
| `pkg resolve` | Resolve named FHIR packages to a content-addressed cache and lockfile | Registry or a separately prepared local mirror required for acquisition. |
| `pkg verify` | Recheck cached archive bytes against lockfile digests | Digest match is not a publisher signature. |
| `inspect` | Inspect an exact version in a lockfile | JSON output; exact locked version required. |
| `context` | Inspect the declared package context graph | JSON; not a complete public ecosystem catalog. |
| `diff` | Produce typed package-archive differences | Only classified modeled transitions are meaningful. |
| `classify` | Classify supported structural differences | Unsupported surfaces fail rather than silently becoming compatible. |
| `impact` | Analyze impact via package graph | Version-2 lockfile required on both sides. |
| `check` | Apply a direction/threshold policy; output JSON or SARIF | Policy-bound result, not full consumer contract verification. |
| `gate` | Apply a quality gate with optional baseline/suppressions | JSON output; explicit before/after states required. |
| `terminology` | Compare supported terminology artifacts | Content rights and coverage remain separate qualification concerns. |
| `oracle` | Invoke the pinned external HL7 oracle adapter | Requires exact adapter/Java setup and `hl7.fhir.r4.core@4.0.1` in both lock states. |
| `source-map` | Map check findings to FSH-source metadata | Requires a local SUSHI index and source-root paths. |
| `github-annotations` | Project check findings to GitHub annotations | Takes an existing report file; does not invent its findings. |

Run `cargo run --locked -p commandf -- <command> --help` for the current CLI argument contract.

## What is intentionally not shipped yet

The future integrated `commandf review` journey (comparison + protected consumer contracts + decision envelope + receipt + SARIF), CommandF Studio, complete compatibility coverage matrices, multi-package observatory catalog, signed reproducible release artifacts, and optional local D1 advisory integration are **planning or research items**, not present CLI features.

The model-free, deterministic Rust core remains authoritative. Any future model may propose review priorities but may not certify a compatible state or suppress a breaking finding.

## Development, security, and evidence

- `docs/COMMAND_F_MASTER_ARCHITECTURE_V2.md` is the current execution authority until a separately qualified migration.
- `docs/COMMAND_F_PLAN_INDEX.md` identifies the complete plan hierarchy and research sources.
- `docs/PROVENANCE_AND_DONOR_POLICY.md` governs borrowed source, terminology content, and evidence rights.
- `AGENTS.md` defines exact-head, fail-closed implementation discipline.
- `docs/COMMAND_F_ASSURANCE_PROGRAM_2026-08-26.md` describes current and candidate development/release assurance units.

Do not put patient information into tests, fixtures or issues. Report vulnerabilities privately using the repository's `SECURITY.md` guidance. Observations, plans, negative results and actual executed proofs must remain distinguishable.

**Product direction:** review healthcare interoperability changes like code—and make every asserted compatibility decision auditable rather than model-generated.
