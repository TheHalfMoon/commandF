#!/usr/bin/env bash
set -euo pipefail

emit_operational_failure() {
  local message="$1"
  printf '::error title=commandF operational failure::%s\n' "$message"
  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'report-path=%s\n' "${COMMANDF_RESOLVED_REPORT_PATH:-}" >> "$GITHUB_OUTPUT"
    printf 'exit-code=1\n' >> "$GITHUB_OUTPUT"
    printf 'passed=false\n' >> "$GITHUB_OUTPUT"
  fi
  exit 1
}

require_nonempty() {
  local name="$1"
  local value="$2"
  if [[ -z "$value" ]]; then
    emit_operational_failure "required Action input is empty: $name"
  fi
}

reject_output_control_characters() {
  local value="$1"
  if [[ "$value" == *$'\n'* || "$value" == *$'\r'* ]]; then
    emit_operational_failure "report-path must not contain carriage return or line feed"
  fi
}

if [[ "${RUNNER_OS:-}" != "Linux" ]]; then
  emit_operational_failure "CF-08 source-backed Action supports Linux runners only"
fi

require_nonempty "package" "${COMMANDF_PACKAGE:-}"
require_nonempty "before-lock" "${COMMANDF_BEFORE_LOCK:-}"
require_nonempty "before-cache" "${COMMANDF_BEFORE_CACHE:-}"
require_nonempty "after-lock" "${COMMANDF_AFTER_LOCK:-}"
require_nonempty "after-cache" "${COMMANDF_AFTER_CACHE:-}"
require_nonempty "GITHUB_ACTION_PATH" "${GITHUB_ACTION_PATH:-}"
require_nonempty "RUNNER_TEMP" "${RUNNER_TEMP:-}"
require_nonempty "GITHUB_OUTPUT" "${GITHUB_OUTPUT:-}"

if [[ -n "${COMMANDF_REPORT_PATH:-}" ]]; then
  reject_output_control_characters "$COMMANDF_REPORT_PATH"
  COMMANDF_RESOLVED_REPORT_PATH="$COMMANDF_REPORT_PATH"
else
  COMMANDF_RESOLVED_REPORT_PATH="$RUNNER_TEMP/commandf/check-report.json"
  mkdir -p "$(dirname "$COMMANDF_RESOLVED_REPORT_PATH")" || \
    emit_operational_failure "unable to create default report directory"
fi
export COMMANDF_RESOLVED_REPORT_PATH

if ! command -v rustup >/dev/null 2>&1 || ! command -v cargo >/dev/null 2>&1; then
  emit_operational_failure "rustup and cargo are required to build the source-backed Action"
fi

if ! cargo +1.97.1 --version >/dev/null 2>&1; then
  if ! rustup toolchain install 1.97.1 --profile minimal --no-self-update; then
    emit_operational_failure "unable to install pinned Rust toolchain 1.97.1"
  fi
fi

export CARGO_TARGET_DIR="$RUNNER_TEMP/commandf-target"
if ! cargo +1.97.1 build --locked \
  --manifest-path "$GITHUB_ACTION_PATH/Cargo.toml" \
  -p commandf; then
  emit_operational_failure "unable to build exact commandF source with the pinned toolchain"
fi

binary="$CARGO_TARGET_DIR/debug/commandf"
if [[ ! -x "$binary" ]]; then
  emit_operational_failure "built commandF binary is missing or not executable"
fi

exec bash "$GITHUB_ACTION_PATH/scripts/github-action-run.sh" "$binary"
