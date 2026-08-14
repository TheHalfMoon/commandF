#!/usr/bin/env bash
set -euo pipefail

temp="$(mktemp -d)"
trap 'rm -rf "$temp"' EXIT

fake="$temp/fake-commandf"
cat > "$fake" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail
command="${1:-}"
shift || true
case "$command" in
  check)
    output=""
    while [[ "$#" -gt 0 ]]; do
      if [[ "$1" == "--output" ]]; then
        shift
        [[ "$#" -gt 0 ]] || exit 1
        output="$1"
      fi
      shift || true
    done
    code="${FAKE_CHECK_CODE:-0}"
    if [[ "$code" == "0" || "$code" == "2" ]]; then
      [[ -n "$output" ]] || exit 1
      printf '{"fake":true}\n' > "$output" || exit 1
    fi
    exit "$code"
    ;;
  source-map)
    : > "$FAKE_SOURCE_MAP_ARGV_LOG"
    output=""
    while [[ "$#" -gt 0 ]]; do
      printf '%s\n' "$1" >> "$FAKE_SOURCE_MAP_ARGV_LOG"
      if [[ "$1" == "--output" ]]; then
        shift
        [[ "$#" -gt 0 ]] || exit 1
        output="$1"
        printf '%s\n' "$1" >> "$FAKE_SOURCE_MAP_ARGV_LOG"
      fi
      shift || true
    done
    code="${FAKE_SOURCE_MAP_CODE:-0}"
    if [[ "$code" == "0" ]]; then
      [[ -n "$output" ]] || exit 1
      printf '{"mapped":true}\n' > "$output" || exit 1
    fi
    exit "$code"
    ;;
  github-annotations)
    : > "$FAKE_RENDER_ARGV_LOG"
    while [[ "$#" -gt 0 ]]; do
      printf '%s\n' "$1" >> "$FAKE_RENDER_ARGV_LOG"
      shift || true
    done
    printf '::notice title=fake::rendered\n'
    exit "${FAKE_RENDER_CODE:-0}"
    ;;
  *)
    exit 1
    ;;
esac
FAKE
chmod +x "$fake"

run_case() {
  local label="$1"
  local expected="$2"
  local check_code="$3"
  local source_map_code="$4"
  local render_code="$5"

  local case_dir="$temp/$label"
  mkdir -p "$case_dir/workspace/input/fsh" "$case_dir/runner/commandf"
  local output_file="$case_dir/github-output"
  local report_path="$case_dir/report.json"
  local source_map_path="$case_dir/runner/commandf/source-map.json"
  local fsh_index="$case_dir/fsh index;literal.json"
  local source_map_argv="$case_dir/source-map-argv"
  local render_argv="$case_dir/render-argv"
  : > "$output_file"
  : > "$fsh_index"

  set +e
  GITHUB_OUTPUT="$output_file" \
  GITHUB_WORKSPACE="$case_dir/workspace" \
  COMMANDF_RESOLVED_REPORT_PATH="$report_path" \
  COMMANDF_SOURCE_MAP_PATH="$source_map_path" \
  COMMANDF_FSH_INDEX="$fsh_index" \
  COMMANDF_FSH_ROOT='input/fsh; literal $(touch never)' \
  COMMANDF_PACKAGE='example.package' \
  COMMANDF_BEFORE_LOCK="$case_dir/before lock" \
  COMMANDF_BEFORE_CACHE="$case_dir/before cache" \
  COMMANDF_AFTER_LOCK="$case_dir/after lock" \
  COMMANDF_AFTER_CACHE="$case_dir/after cache" \
  COMMANDF_DIRECTION='both' \
  COMMANDF_FAIL_ON='breaking' \
  FAKE_CHECK_CODE="$check_code" \
  FAKE_SOURCE_MAP_CODE="$source_map_code" \
  FAKE_RENDER_CODE="$render_code" \
  FAKE_SOURCE_MAP_ARGV_LOG="$source_map_argv" \
  FAKE_RENDER_ARGV_LOG="$render_argv" \
  bash scripts/github-action-run.sh "$fake" > "$case_dir/stdout" 2> "$case_dir/stderr"
  actual=$?
  set -e

  [[ "$actual" -eq "$expected" ]] || {
    echo "$label: expected exit $expected, got $actual" >&2
    exit 1
  }

  case "$expected" in
    0)
      grep -Fx 'exit-code=0' "$output_file" >/dev/null
      grep -Fx 'passed=true' "$output_file" >/dev/null
      grep -Fx "report-path=$report_path" "$output_file" >/dev/null
      ;;
    2)
      grep -Fx 'exit-code=2' "$output_file" >/dev/null
      grep -Fx 'passed=false' "$output_file" >/dev/null
      grep -Fx "report-path=$report_path" "$output_file" >/dev/null
      ;;
    1)
      grep -Fx 'exit-code=1' "$output_file" >/dev/null
      grep -Fx 'passed=false' "$output_file" >/dev/null
      grep -Fx 'report-path=' "$output_file" >/dev/null
      ;;
  esac

  if [[ "$check_code" == "0" || "$check_code" == "2" ]]; then
    grep -Fx -- '--fsh-index' "$source_map_argv" >/dev/null
    grep -Fx -- "$fsh_index" "$source_map_argv" >/dev/null
    grep -Fx -- '--repo-root' "$source_map_argv" >/dev/null
    grep -Fx -- "$case_dir/workspace" "$source_map_argv" >/dev/null
    grep -Fx -- '--fsh-root' "$source_map_argv" >/dev/null
    grep -Fx -- 'input/fsh; literal $(touch never)' "$source_map_argv" >/dev/null
  fi

  if [[ "$source_map_code" == "0" && ( "$check_code" == "0" || "$check_code" == "2" ) ]]; then
    grep -Fx -- '--source-map' "$render_argv" >/dev/null
    grep -Fx -- "$source_map_path" "$render_argv" >/dev/null
  fi
}

run_case mapped-pass 0 0 0 0
run_case mapped-policy-fail 2 2 0 0
run_case source-map-fail 1 0 1 0
run_case mapped-render-fail 1 0 0 1

[[ ! -e "$temp/never" ]]
printf 'CF-09 Action source-map tests passed\n'
