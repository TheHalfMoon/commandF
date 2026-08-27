# AF-01 Stack A Workflow Trust Inventory

Status: IMPLEMENTATION_EVIDENCE / T010

Canonical inventory base:

```text
main: eeecb0bc03c7040bb18b70bce8b69d618384f783
tree: d5abe932f1436a9612f45bf130ba29aadbc5a133
AF-01 planning: CANONICAL
```

This inventory records the tracked GitHub workflow/action authority that AF-01 Stack A must make machine-checkable. It is not a substitute for the repository-owned discovery audit: the audit must discover future workflow and Action metadata files automatically.

## Tracked workflow files

1. `.github/workflows/ci.yml`
2. `.github/workflows/cf06-oracle.yml`
3. `.github/workflows/cf11-multi-version-proof.yml`
4. `.github/workflows/cf11g-context-proof.yml`
5. `.github/workflows/cf12-impact-proof.yml`
6. `.github/workflows/cf13-quality-gate-proof.yml`
7. `.github/workflows/registry-download-smoke.yml`

## Tracked Action metadata

- `action.yml`

No `action.yaml` metadata file is present at the inventory base. AF-01 discovery still treats both `action.yml` and `action.yaml` at any tracked path as authoritative scan inputs.

The source-backed root Action delegates through tracked repository-owned shell sources:

```text
action.yml
  -> scripts/github-action.sh
  -> scripts/github-action-run.sh
```

Because those scripts can execute lockfile-consuming Cargo commands or delegate additional shell authority, Stack A treats the statically exposed `$GITHUB_ACTION_PATH/...` chain as part of the Action trust surface. Delegated Action shell sources must be tracked, recursively auditable, cycle-bounded, and exact-path static. Dynamic, relative, command-substituted, shell-`-c`, or prefix/suffix-expanded script authority fails closed. The Action runner binds execution to the built `$CARGO_TARGET_DIR/debug/commandf` path rather than accepting a runtime-selected executable argument.

## Job authority inventory

| Workflow | Job | Current effective permission | Current runner | Current container | Current timeout | Stack A disposition |
|---|---|---|---|---|---|---|
| `ci.yml` | `rust` | `contents: read` | `ubuntu-latest` | none | none | pin runner/actions, disable checkout credentials, add timeout |
| `cf06-oracle.yml` | `oracle-self-smoke` | `contents: read` | `ubuntu-latest` | none | none | fixed runner, explicit job permission, timeout |
| `cf06-oracle.yml` | `oracle-changed-profile` | `contents: read` | `ubuntu-latest` | none | none | fixed runner, explicit job permission, timeout |
| `cf06-oracle.yml` | `oracle-proof` | inherited `contents: read` | `ubuntu-latest` | none | none | reduce to no repository permission, fixed runner, timeout |
| `cf11-multi-version-proof.yml` | `real-package-graph` | `contents: read` | `ubuntu-24.04` | Rust digest pinned | 20m | retain |
| `cf11g-context-proof.yml` | `deterministic-context-cli` | `contents: read` | `ubuntu-24.04` | Rust digest pinned | 15m | retain |
| `cf12-impact-proof.yml` | `deterministic-impact-cli` | `contents: read` | `ubuntu-24.04` | Rust digest pinned | 15m | retain |
| `cf13-quality-gate-proof.yml` | `deterministic-quality-gate` | `contents: read` | `ubuntu-24.04` | Rust digest pinned | 15m | retain |
| `registry-download-smoke.yml` | `registry-download` | `contents: read` | `ubuntu-latest` | none | 15m | fixed runner |

## External Action / reusable-workflow references

Immutable references already used by current proof workflows:

```text
actions/checkout@fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09
actions/setup-java@b6effb05e454b25005698d916606bdc6ffcbf961
dtolnay/rust-toolchain@032958afbdc797a9164d3bc0b56325c1308924a5
actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02
```

Current `ci.yml` is the outlier:

```text
actions/checkout@v4
dtolnay/rust-toolchain@1.97.1
```

`action.yml` is a local composite Action and contains no external `uses:` reference at this base.

## Checkout credential inventory

Every current checkout in the proof/oracle/registry workflows sets:

```yaml
persist-credentials: false
```

`ci.yml` does not and must be reconciled.

## Container identity inventory

Current containerized proof jobs use digest-bound Rust images. No service containers are present at this base. AF-01 policy applies the digest rule to every future job or service container that appears; a mutable tag is never accepted merely because it is new or non-proof-labeled.

## Cargo lockfile-consuming command inventory

Current workflow commands that build/check/test/run against the Rust dependency graph already use `--locked` in the proof/oracle/registry workflows and in the relevant `ci.yml` clippy/test/run invocations. `cargo fmt` and `cargo --version` are not lockfile-consuming commands and are outside this rule.

The source-backed Action also builds commandF from the repository lockfile. Its composite `run:` entries and recursively reachable tracked Action shell sources are therefore part of the same Cargo authority boundary. Variable-expanded executable positions must be proven statically non-Cargo; unknown or Cargo-resolving executable provenance fails closed.

AF-01 audit treats at least these cargo subcommands as lockfile-consuming when present in workflow or Action shell commands:

```text
bench
build
check
clippy
doc
metadata
run
test
```

## Machine-checkable target after T014/T015

- every discovered workflow has an exact policy entry for every discovered job;
- effective workflow/job permissions equal the policy declaration, with no unresolved GitHub default authority;
- all current runners become `ubuntu-24.04`;
- every job has an explicit bounded `timeout-minutes`;
- all external `uses:` references are full 40-hex commit SHAs;
- all checkout steps persist no credentials;
- every job/service container reference, if present, is digest-bound with `sha256`;
- all lockfile-consuming cargo invocations use `--locked`;
- every tracked `action.yml` and `action.yaml` is scanned for external `uses:` references and composite `run:` Cargo authority;
- every statically delegated `$GITHUB_ACTION_PATH/...` shell source is tracked and recursively audited, while dynamic or non-Action-root shell source selection fails closed.

## Scope boundary

Stack A changes development-assurance configuration and source-backed Action execution hardening only. It does not change commandF product semantics, CF-06 production oracle identity, the CF-10 frozen corpus, report schemas, or runtime classification authority.
