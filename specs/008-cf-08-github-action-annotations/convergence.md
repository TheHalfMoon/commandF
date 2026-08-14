# CF-08 Convergence Record

Status: Implementation in progress

This record is intentionally incomplete until the exact final CF-08 head passes all required gates. Do not treat its presence as certification.

## Exact stack

```text
repository: TheHalfMoon/commandF
branch: feat/cf-08-github-action-annotations
base branch: feat/cf-05-sarif-ci-gate
base SHA: 9f158f1dcb7d04bc5ee582eabd1ee8dd93bd5019
```

CF-06 and CF-07 are not dependencies. CF-09 has not started.

## Current implemented surface

- Spec Kit authority established;
- shared CF-05 direction-selection helper;
- deterministic bounded GitHub workflow-command projection module;
- workflow-command data/property escaping;
- no explicit source location before CF-09;
- 10 error / 10 warning / 10 notice presentation caps with overflow disclosure;
- `commandf github-annotations --input` with 64 MiB bounded report input;
- projection and CLI regression coverage in progress.

## Remaining before convergence

- exact Rust gates on the current implementation;
- root composite `action.yml`;
- action runner preserving CF-05 exit 0/1/2;
- action security regressions;
- real local Action self-check smoke;
- CodeRabbit/Qodo reconciliation;
- final Spec Kit reconciliation;
- exact-final-head CI and governance checks.

Correct current state:

```text
CF-08_IMPLEMENTATION_IN_PROGRESS
```
