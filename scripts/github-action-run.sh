#!/usr/bin/env bash
set -euo pipefail

binary="${1:-}"
if [[ -z "$binary" || ! -x "$binary" ]]; then
  printf '::error title=commandF operational failure::commandF Action runner received no executable binary\n'
  exit 1
fi

report_path="${COMMANDF_RESOLVED_REPORT_PATH:-}"
if [[ -z "$report_path" ]]; then
  printf '::error title=commandF operational failure::commandF Action runner received no resolved report path\n'
  exit 1
fi

write_outputs() {
  local final_code="$1"
  local output_report_path=""
  local passed="false"

  if [[ "$final_code" == "0" || "$final_code" == "2" ]]; then
    output_report_path="$report_path"
  fi
  if [[ "$final_code" == "0" ]]; then
    passed="true"
  fi

  printf 'report-path=%s\n' "$output_report_path" >> "$GITHUB_OUTPUT"
  printf 'exit-code=%s\n' "$final_code" >> "$GITHUB_OUTPUT"
  printf 'passed=%s\n' "$passed" >> "$GITHUB_OUTPUT"
}

set +e
"$binary" check "$COMMANDF_PACKAGE" \
  --before-lock "$COMMANDF_BEFORE_LOCK" \
  --before-cache "$COMMANDF_BEFORE_CACHE" \
  --after-lock "$COMMANDF_AFTER_LOCK" \
  --after-cache "$COMMANDF_AFTER_CACHE" \
  --direction "$COMMANDF_DIRECTION" \
  --fail-on "$COMMANDF_FAIL_ON" \
  --format json \
  --output "$report_path"
check_code=$?
set -e

case "$check_code" in
  0|2)
    if [[ ! -f "$report_path" ]]; then
      printf '::error title=commandF operational failure::commandF check completed without a complete JSON report\n'
      write_outputs 1
      exit 1
    fi

    set +e
    "$binary" github-annotations --input "$report_path"
    renderer_code=$?
    set -e
    if [[ "$renderer_code" -ne 0 ]]; then
      printf '::error title=commandF operational failure::commandF could not safely render GitHub annotations from the complete report\n'
      write_outputs 1
      exit 1
    fi

    write_outputs "$check_code"
    exit "$check_code"
    ;;
  1)
    printf '::error title=commandF operational failure::commandF check failed before a valid report was produced; inspect step logs\n'
    write_outputs 1
    exit 1
    ;;
  *)
    printf '::error title=commandF operational failure::commandF check returned an unsupported exit code\n'
    write_outputs 1
    exit 1
    ;;
esac
