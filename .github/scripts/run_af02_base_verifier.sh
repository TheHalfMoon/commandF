#!/usr/bin/env bash
set -euo pipefail

SCRIPT_PATH="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)/$(basename -- "${BASH_SOURCE[0]}")"

python3 - "$SCRIPT_PATH" "$@" <<'PY'
from __future__ import annotations

import json
import os
import subprocess
import sys
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path, PurePosixPath

SCHEMA = "commandf.af02-base-gate-bootstrap/v1"
SELF_TEST_SCHEMA = "commandf.af02-base-gate-self-test/v1"
MAX_CHANGED_FILES = 3000
MAX_PATH_BYTES = 4096
MAX_API_BYTES = 4 * 1024 * 1024
GITHUB_API_ROOT = "https://api.github.com"

AUTHORITY_EXACT = frozenset(
    {
        ".github/main-review-ruleset.json",
        ".github/main-ruleset.json",
        ".github/required-checks.json",
        ".github/workflow-trust-policy.json",
        "Cargo.lock",
        "Cargo.toml",
        "crates/commandf-pkg/src/oracle_model.rs",
    }
)
AUTHORITY_PREFIXES = (
    ".github/scripts/",
    ".github/workflows/",
    "donors/",
    "specs/016-af-02-adversarial-test-strength/",
    "tools/af02-verifier/",
)
BOOTSTRAP_EXACT = frozenset(
    {
        ".github/scripts/run_af02_base_verifier.sh",
        ".github/workflow-trust-policy.json",
        ".github/workflows/af02-base-verifier.yml",
    }
)
BOOTSTRAP_PREFIXES = (
    "tools/af02-verifier/src/",
    "tools/af02-verifier/tests/",
)
ALLOWED_FILE_STATUSES = frozenset(
    {"added", "changed", "copied", "modified", "removed", "renamed", "unchanged"}
)


def fail(message: str) -> "None":
    raise SystemExit(f"AF02_BASE_GATE_FAIL: {message}")


def canonical_json(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def validate_repo_path(raw: object) -> str:
    if not isinstance(raw, str) or not raw:
        fail("changed path must be a non-empty string")
    if "\x00" in raw or "\\" in raw:
        fail("changed path contains a prohibited NUL or backslash")
    if len(raw.encode("utf-8")) > MAX_PATH_BYTES:
        fail("changed path exceeds the bounded UTF-8 length")
    path = PurePosixPath(raw)
    if path.is_absolute() or raw.startswith("/"):
        fail("changed path must be repository-relative")
    parts = path.parts
    if not parts or any(part in {"", ".", ".."} for part in parts):
        fail("changed path is not normalized")
    normalized = path.as_posix()
    if normalized != raw:
        fail("changed path is not canonical POSIX form")
    return normalized


def is_authority_path(path: str) -> bool:
    return path in AUTHORITY_EXACT or any(path.startswith(prefix) for prefix in AUTHORITY_PREFIXES)


def is_bootstrap_path(path: str) -> bool:
    return path in BOOTSTRAP_EXACT or any(path.startswith(prefix) for prefix in BOOTSTRAP_PREFIXES)


def classify_paths(paths: set[str]) -> str:
    authority = {path for path in paths if is_authority_path(path)}
    if not authority:
        return "NOT_APPLICABLE"
    if all(is_bootstrap_path(path) for path in paths):
        return "BOOTSTRAP_T025_STRENGTHENING"
    if all(is_bootstrap_path(path) for path in authority) and authority != paths:
        return "BLOCKED_MIXED_SCOPE"
    return "BLOCKED_PENDING_T025"


def run_self_test() -> None:
    cases = 0

    def expect(paths: set[str], expected: str) -> None:
        nonlocal cases
        observed = classify_paths(paths)
        if observed != expected:
            fail(f"self-test expected {expected}, observed {observed}")
        cases += 1

    expect({"README.md"}, "NOT_APPLICABLE")
    expect(
        {
            ".github/workflows/af02-base-verifier.yml",
            ".github/scripts/run_af02_base_verifier.sh",
            "tools/af02-verifier/src/base_gate.rs",
            "tools/af02-verifier/tests/base_gate_t025.rs",
        },
        "BOOTSTRAP_T025_STRENGTHENING",
    )
    expect(
        {
            ".github/workflows/af02-base-verifier.yml",
            "README.md",
        },
        "BLOCKED_MIXED_SCOPE",
    )
    expect(
        {"specs/016-af-02-adversarial-test-strength/semantic-contract.json"},
        "BLOCKED_PENDING_T025",
    )
    expect({"Cargo.lock"}, "BLOCKED_PENDING_T025")

    if validate_repo_path("tools/af02-verifier/src/main.rs") != "tools/af02-verifier/src/main.rs":
        fail("self-test normalized path changed unexpectedly")
    cases += 1
    try:
        validate_repo_path("../escape")
    except SystemExit:
        cases += 1
    else:
        fail("self-test accepted a parent traversal")

    print(canonical_json({"schema": SELF_TEST_SCHEMA, "test_count": cases, "result": "PASS"}))


def read_event() -> tuple[dict, str, int, str, str]:
    event_path = os.environ.get("GITHUB_EVENT_PATH")
    if not event_path:
        fail("GITHUB_EVENT_PATH is missing")
    try:
        event = json.loads(Path(event_path).read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        fail(f"cannot parse GitHub event: {exc}")
    if not isinstance(event, dict):
        fail("GitHub event root is not an object")
    pull_request = event.get("pull_request")
    repository = event.get("repository")
    number = event.get("number")
    if not isinstance(pull_request, dict) or not isinstance(repository, dict):
        fail("event is not a pull_request_target event")
    full_name = repository.get("full_name")
    if not isinstance(full_name, str) or full_name != "TheHalfMoon/commandF":
        fail("unexpected repository identity in GitHub event")
    if not isinstance(number, int) or isinstance(number, bool) or number <= 0:
        fail("invalid pull request number in GitHub event")
    base = pull_request.get("base")
    head = pull_request.get("head")
    if not isinstance(base, dict) or not isinstance(head, dict):
        fail("event pull request is missing base/head objects")
    base_sha = base.get("sha")
    head_sha = head.get("sha")
    if not isinstance(base_sha, str) or not isinstance(head_sha, str):
        fail("event pull request is missing base/head SHA identity")
    for label, sha in (("base", base_sha), ("head", head_sha)):
        if len(sha) != 40 or any(ch not in "0123456789abcdef" for ch in sha):
            fail(f"{label} SHA is not lowercase 40-hex")
    if base_sha == head_sha:
        fail("base and head SHA must differ")
    return event, full_name, number, base_sha, head_sha


def api_json(url: str) -> object:
    request = urllib.request.Request(
        url,
        headers={
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "commandf-af02-base-gate",
        },
        method="GET",
    )
    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            if response.status != 200:
                fail(f"GitHub API returned HTTP {response.status}")
            body = response.read(MAX_API_BYTES + 1)
    except (urllib.error.URLError, TimeoutError, OSError) as exc:
        fail(f"GitHub API request failed: {exc}")
    if len(body) > MAX_API_BYTES:
        fail("GitHub API response exceeded the bounded size")
    try:
        return json.loads(body)
    except (UnicodeError, json.JSONDecodeError) as exc:
        fail(f"GitHub API returned invalid JSON: {exc}")


def github_truth(repository: str, number: int, event_base: str, event_head: str) -> set[str]:
    owner, repo = repository.split("/", 1)
    owner_q = urllib.parse.quote(owner, safe="")
    repo_q = urllib.parse.quote(repo, safe="")
    pr_url = f"{GITHUB_API_ROOT}/repos/{owner_q}/{repo_q}/pulls/{number}"
    pr = api_json(pr_url)
    if not isinstance(pr, dict):
        fail("GitHub pull request API root is not an object")
    api_base = pr.get("base")
    api_head = pr.get("head")
    if not isinstance(api_base, dict) or not isinstance(api_head, dict):
        fail("GitHub pull request API is missing base/head objects")
    if api_base.get("sha") != event_base or api_head.get("sha") != event_head:
        fail("GitHub event/API base or head SHA disagreement")
    base_repo = api_base.get("repo")
    if not isinstance(base_repo, dict) or base_repo.get("full_name") != repository:
        fail("GitHub API base repository identity mismatch")

    paths: set[str] = set()
    seen_files: set[str] = set()
    file_count = 0
    for page in range(1, 31):
        files_url = f"{pr_url}/files?per_page=100&page={page}"
        payload = api_json(files_url)
        if not isinstance(payload, list):
            fail("GitHub pull request files response is not an array")
        for entry in payload:
            if not isinstance(entry, dict):
                fail("GitHub pull request file entry is not an object")
            status = entry.get("status")
            if status not in ALLOWED_FILE_STATUSES:
                fail(f"unsupported GitHub file status {status!r}")
            filename = validate_repo_path(entry.get("filename"))
            if filename in seen_files:
                fail("GitHub API returned a duplicate changed filename")
            seen_files.add(filename)
            paths.add(filename)
            previous = entry.get("previous_filename")
            if previous is not None:
                paths.add(validate_repo_path(previous))
            file_count += 1
            if file_count > MAX_CHANGED_FILES:
                fail("GitHub changed-file count exceeded the bounded maximum")
        if len(payload) < 100:
            break
    else:
        fail("GitHub changed-file pagination exceeded the bounded maximum")
    if file_count == 0:
        fail("GitHub API returned no changed files for the pull request")
    return paths


def git_head(root: Path) -> str:
    try:
        completed = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        fail(f"cannot read checkout Git identity: {exc}")
    value = completed.stdout.strip()
    if len(value) != 40 or any(ch not in "0123456789abcdef" for ch in value):
        fail("checkout HEAD is not lowercase 40-hex")
    return value


def main() -> None:
    argv = sys.argv[1:]
    if not argv:
        fail("script path argument is missing")
    script_path = Path(argv[0]).resolve(strict=True)
    args = argv[1:]
    if args == ["--self-test"]:
        run_self_test()
        return
    if len(args) != 2:
        fail("expected canonical-base and candidate checkout paths")

    base_root = Path(args[0]).resolve(strict=True)
    candidate_root = Path(args[1]).resolve(strict=True)
    if not base_root.is_dir() or not candidate_root.is_dir():
        fail("base and candidate roots must be directories")
    if base_root == candidate_root or base_root in candidate_root.parents or candidate_root in base_root.parents:
        fail("base and candidate roots must be disjoint")
    if base_root not in script_path.parents:
        fail("executing gate script is not inside the canonical-base checkout")
    if candidate_root in script_path.parents:
        fail("executing gate script resolves inside the candidate checkout")

    _, repository, number, base_sha, head_sha = read_event()
    observed_base = git_head(base_root)
    observed_head = git_head(candidate_root)
    if observed_base != base_sha:
        fail("canonical-base checkout SHA differs from GitHub event base SHA")
    if observed_head != head_sha:
        fail("candidate checkout SHA differs from GitHub event head SHA")

    changed_paths = github_truth(repository, number, base_sha, head_sha)
    mode = classify_paths(changed_paths)
    if mode == "BLOCKED_MIXED_SCOPE":
        fail("dedicated T025 verifier strengthening is mixed with unrelated changes")
    if mode == "BLOCKED_PENDING_T025":
        fail("AF-02 acceptance-authority change is blocked until T025 full verifier proof is canonical")

    print(
        canonical_json(
            {
                "base_sha": base_sha,
                "candidate_code_executed": False,
                "changed_path_count": len(changed_paths),
                "head_sha": head_sha,
                "mode": mode,
                "pull_request": number,
                "repository": repository,
                "schema": SCHEMA,
            }
        )
    )


if __name__ == "__main__":
    main()
PY
