#!/usr/bin/env bash
set -euo pipefail

temp="$(mktemp -d)"
trap 'rm -rf "$temp"' EXIT

fake_target="$temp/fake-target"
mkdir -p "$fake_target/debug"
fake="$fake_target/debug/commandf"
cat > "$fake" <<'FAKE'
#!/usr/bin/env bash
set -euo pipefail
command="${1:-}"
shift || true
case "$command" in
  check)
    : > "$FAKE_ARGV_LOG"
    output=""
    while [[ "$#" -gt 0 ]]; do
      printf '%s\n' "$1" >> "$FAKE_ARGV_LOG"
      if [[ "$1" == "--output" ]]; then
        shift
        [[ "$#" -gt 0 ]] || exit 1
        output="$1"
        printf '%s\n' "$1" >> "$FAKE_ARGV_LOG"
      fi
      shift || true
    done
    code="${FAKE_CHECK_CODE:-0}"
    if [[ "$code" == "0" || "$code" == "2" ]]; then
      [[ -n "$output" ]] || exit 1
      if ! printf '{"fake":true}\n' > "$output"; then
        exit 1
      fi
    fi
    exit "$code"
    ;;
  github-annotations)
    printf 'renderer\n' >> "$FAKE_RENDER_LOG"
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
  local render_code="$4"
  local report_path="$5"
  local package="$6"

  local case_dir="$temp/$label"
  mkdir -p "$case_dir"
  local output_file="$case_dir/github-output"
  local argv_log="$case_dir/argv"
  local render_log="$case_dir/render"
  : > "$output_file"
  : > "$render_log"

  set +e
  CARGO_TARGET_DIR="$fake_target" \
  GITHUB_OUTPUT="$output_file" \
  COMMANDF_RESOLVED_REPORT_PATH="$report_path" \
  COMMANDF_PACKAGE="$package" \
  COMMANDF_BEFORE_LOCK="$case_dir/before lock" \
  COMMANDF_BEFORE_CACHE="$case_dir/before cache" \
  COMMANDF_AFTER_LOCK="$case_dir/after lock" \
  COMMANDF_AFTER_CACHE="$case_dir/after cache" \
  COMMANDF_DIRECTION="both" \
  COMMANDF_FAIL_ON="breaking" \
  FAKE_CHECK_CODE="$check_code" \
  FAKE_RENDER_CODE="$render_code" \
  FAKE_ARGV_LOG="$argv_log" \
  FAKE_RENDER_LOG="$render_log" \
  bash scripts/github-action-run.sh > "$case_dir/stdout" 2> "$case_dir/stderr"
  actual=$?
  set -e

  [[ "$actual" -eq "$expected" ]] || {
    echo "$label: expected exit $expected, got $actual" >&2
    exit 1
  }

  grep -Fx -- "$package" "$argv_log" >/dev/null
  grep -Fx -- "$case_dir/before lock" "$argv_log" >/dev/null
  grep -Fx -- "$case_dir/before cache" "$argv_log" >/dev/null

  case "$expected" in
    0)
      grep -Fx 'exit-code=0' "$output_file" >/dev/null
      grep -Fx 'passed=true' "$output_file" >/dev/null
      grep -Fx "report-path=$report_path" "$output_file" >/dev/null
      [[ -s "$render_log" ]]
      ;;
    2)
      grep -Fx 'exit-code=2' "$output_file" >/dev/null
      grep -Fx 'passed=false' "$output_file" >/dev/null
      grep -Fx "report-path=$report_path" "$output_file" >/dev/null
      [[ -s "$render_log" ]]
      ;;
    1)
      grep -Fx 'exit-code=1' "$output_file" >/dev/null
      grep -Fx 'passed=false' "$output_file" >/dev/null
      grep -Fx 'report-path=' "$output_file" >/dev/null
      ;;
  esac
}

mkdir -p "$temp/pass" "$temp/fail" "$temp/operational" "$temp/render-fail"
run_case pass 0 0 0 "$temp/pass/report.json" 'example.package'
run_case fail 2 2 0 "$temp/fail/report.json" 'example.package'
run_case operational 1 1 0 "$temp/operational/report.json" 'example.package'
run_case render-fail 1 0 1 "$temp/render-fail/report.json" 'example.package'

pwned="$temp/pwned"
package="example.package; touch $pwned"
mkdir -p "$temp/argv-safe"
run_case argv-safe 0 0 0 "$temp/argv-safe/report.json" "$package"
[[ ! -e "$pwned" ]]

missing_parent="$temp/missing/parent/report.json"
mkdir -p "$temp/missing-case"
run_case missing-parent 1 0 0 "$missing_parent" 'example.package'
[[ ! -d "$temp/missing/parent" ]]

printf 'CF-08 Action runner tests passed\n'
