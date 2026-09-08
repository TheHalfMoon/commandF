#!/usr/bin/env bash
set -euo pipefail

SCRIPT_PATH="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)/$(basename -- "${BASH_SOURCE[0]}")"

python3 - "$SCRIPT_PATH" "$@" <<'PY'
from __future__ import annotations

import hashlib
import json
import os
import selectors
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path, PurePosixPath

BOOTSTRAP_SCHEMA = "commandf.af02-base-gate-bootstrap/v1"
SELF_TEST_SCHEMA = "commandf.af02-base-gate-self-test/v1"
GATE_INPUT_SCHEMA = "commandf.af02-base-gate-input/v1"
RUNTIME_SCHEMA = "commandf.af02-base-controlled-gate/v1"
MAX_CHANGED_FILES = 3000
MAX_PATH_BYTES = 4096
MAX_API_BYTES = 4 * 1024 * 1024
STREAM_LIMIT = 1024 * 1024
PARSER_WALL_SECONDS = 20
PARSER_MEMORY_BYTES = 512 * 1024 * 1024
PARSER_PIDS = 64
GITHUB_API_ROOT = "https://api.github.com"
RUNTIME_IMAGE = "docker.io/library/rust@sha256:9146b0f62e1939989aa96fc8d89699a43c5635bf212819235a773e1a9e71a98f"

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
        "tools/af02-verifier/src/base_gate.rs",
        "tools/af02-verifier/src/main.rs",
        "tools/af02-verifier/tests/base_gate_t025.rs",
    }
)
ALLOWED_FILE_STATUSES = frozenset(
    {"added", "changed", "copied", "modified", "removed", "renamed", "unchanged"}
)


def fail(message: str) -> "None":
    raise SystemExit(f"AF02_BASE_GATE_FAIL: {message}")


def canonical_json(value: object) -> str:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"))


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while True:
            chunk = handle.read(1024 * 1024)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()


def git_blob_sha1_file(path: Path) -> str:
    data = path.read_bytes()
    payload = f"blob {len(data)}\0".encode("ascii") + data
    return hashlib.sha1(payload).hexdigest()


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
    parts = raw.split("/")
    if not parts or any(part in {"", ".", ".."} for part in parts):
        fail("changed path is not normalized")
    if path.as_posix() != raw:
        fail("changed path is not canonical POSIX form")
    return raw


def is_authority_path(path: str) -> bool:
    return path in AUTHORITY_EXACT or any(path.startswith(prefix) for prefix in AUTHORITY_PREFIXES)


def is_bootstrap_path(path: str) -> bool:
    return path in BOOTSTRAP_EXACT


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
            "tools/af02-verifier/src/main.rs",
            "tools/af02-verifier/tests/base_gate_t025.rs",
        },
        "BOOTSTRAP_T025_STRENGTHENING",
    )
    expect(
        {".github/workflows/af02-base-verifier.yml", "README.md"},
        "BLOCKED_MIXED_SCOPE",
    )
    expect(
        {"specs/016-af-02-adversarial-test-strength/semantic-contract.json"},
        "BLOCKED_PENDING_T025",
    )
    expect({"Cargo.lock"}, "BLOCKED_PENDING_T025")
    expect({"tools/af02-verifier/src/semantic.rs"}, "BLOCKED_PENDING_T025")
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


def read_event() -> tuple[str, int, str, str]:
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
    if full_name != "TheHalfMoon/commandF":
        fail("unexpected repository identity in GitHub event")
    if not isinstance(number, int) or isinstance(number, bool) or number <= 0:
        fail("invalid pull request number in GitHub event")
    base = pull_request.get("base")
    head = pull_request.get("head")
    if not isinstance(base, dict) or not isinstance(head, dict):
        fail("event pull request is missing base/head objects")
    base_sha = base.get("sha")
    head_sha = head.get("sha")
    for label, sha in (("base", base_sha), ("head", head_sha)):
        if not isinstance(sha, str) or len(sha) != 40 or any(ch not in "0123456789abcdef" for ch in sha):
            fail(f"{label} SHA is not lowercase 40-hex")
    if base_sha == head_sha:
        fail("base and head SHA must differ")
    return full_name, number, base_sha, head_sha


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


def github_truth(repository: str, number: int, event_base: str, event_head: str) -> list[dict]:
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

    changed: list[dict] = []
    seen: set[str] = set()
    for page in range(1, 31):
        payload = api_json(f"{pr_url}/files?per_page=100&page={page}")
        if not isinstance(payload, list):
            fail("GitHub pull request files response is not an array")
        for entry in payload:
            if not isinstance(entry, dict):
                fail("GitHub pull request file entry is not an object")
            status = entry.get("status")
            if status not in ALLOWED_FILE_STATUSES:
                fail(f"unsupported GitHub file status {status!r}")
            filename = validate_repo_path(entry.get("filename"))
            if filename in seen:
                fail("GitHub API returned a duplicate changed filename")
            seen.add(filename)
            previous = entry.get("previous_filename")
            if previous is not None:
                previous = validate_repo_path(previous)
            changed.append(
                {"status": status, "filename": filename, "previous_filename": previous}
            )
            if len(changed) > MAX_CHANGED_FILES:
                fail("GitHub changed-file count exceeded the bounded maximum")
        if len(payload) < 100:
            break
    else:
        fail("GitHub changed-file pagination exceeded the bounded maximum")
    if not changed:
        fail("GitHub API returned no changed files for the pull request")
    changed.sort(key=lambda item: item["filename"].encode("utf-8"))
    return changed


def git_output(root: Path, args: list[str]) -> str:
    try:
        completed = subprocess.run(
            ["git", "-C", str(root), *args],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        fail(f"cannot read canonical Git identity: {exc}")
    return completed.stdout.strip()


def git_head(root: Path) -> str:
    value = git_output(root, ["rev-parse", "HEAD"])
    if len(value) != 40 or any(ch not in "0123456789abcdef" for ch in value):
        fail("checkout HEAD is not lowercase 40-hex")
    return value


def git_tree(root: Path) -> str:
    value = git_output(root, ["rev-parse", "HEAD^{tree}"])
    if len(value) != 40 or any(ch not in "0123456789abcdef" for ch in value):
        fail("checkout tree is not lowercase 40-hex")
    return value


def git_blob(root: Path, path: str) -> str:
    value = git_output(root, ["rev-parse", f"HEAD:{path}"])
    if len(value) != 40 or any(ch not in "0123456789abcdef" for ch in value):
        fail(f"Git blob identity is invalid for {path}")
    return value


def git_tree_blobs(root: Path, prefix: str) -> dict[str, str]:
    output = git_output(root, ["ls-tree", "-r", "HEAD", "--", prefix])
    blobs: dict[str, str] = {}
    for line in output.splitlines():
        if not line:
            continue
        try:
            meta, path = line.split("\t", 1)
            mode, kind, sha = meta.split()
        except ValueError:
            fail(f"malformed git ls-tree record under {prefix}")
        if kind != "blob" or not mode:
            fail(f"non-blob canonical authority under {prefix}: {path}")
        path = validate_repo_path(path)
        if len(sha) != 40 or any(ch not in "0123456789abcdef" for ch in sha):
            fail(f"invalid Git blob under {prefix}: {path}")
        blobs[path] = sha
    if not blobs:
        fail(f"canonical base has no Git blobs under {prefix}")
    return dict(sorted(blobs.items(), key=lambda item: item[0].encode("utf-8")))


def canonical_inventory(root: Path) -> dict:
    path = root / "specs/016-af-02-adversarial-test-strength/enforcement-inventory.json"
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        fail(f"cannot parse canonical enforcement inventory: {exc}")
    if not isinstance(value, dict) or value.get("schema") != "commandf.af02-enforcement-inventory/v1":
        fail("canonical enforcement inventory has unexpected schema")
    entries = value.get("entries")
    if not isinstance(entries, list) or not entries:
        fail("canonical enforcement inventory has no entries")
    return value


def known_authority_paths(root: Path) -> list[str]:
    known = set(AUTHORITY_EXACT)
    output = git_output(
        root,
        [
            "ls-tree",
            "-r",
            "--name-only",
            "HEAD",
            "--",
            ".github/scripts",
            ".github/workflows",
            "donors",
            "specs/016-af-02-adversarial-test-strength",
            "tools/af02-verifier",
        ],
    )
    for line in output.splitlines():
        if line:
            known.add(validate_repo_path(line))
    for entry in canonical_inventory(root)["entries"]:
        if not isinstance(entry, dict):
            fail("canonical enforcement inventory entry is not an object")
        planned = entry.get("planned_path")
        if not isinstance(planned, str) or not planned:
            fail("canonical enforcement inventory has invalid planned path")
        if not planned.endswith("/"):
            known.add(validate_repo_path(planned))
    return sorted(known, key=lambda value: value.encode("utf-8"))


def build_base_identity(root: Path, base_sha: str) -> dict:
    observed = git_head(root)
    if observed != base_sha:
        fail("canonical-base checkout SHA differs from GitHub event base SHA")
    return {
        "schema": "commandf.af02-base-identity-proof/v1",
        "base_sha": base_sha,
        "base_tree": git_tree(root),
        "workflow_blob": git_blob(root, ".github/workflows/af02-base-verifier.yml"),
        "runner_blob": git_blob(root, ".github/scripts/run_af02_base_verifier.sh"),
        "cargo_manifest_blob": git_blob(root, "tools/af02-verifier/Cargo.toml"),
        "cargo_lock_blob": git_blob(root, "tools/af02-verifier/Cargo.lock"),
        "verifier_blobs": git_tree_blobs(root, "tools/af02-verifier/src"),
        "schema_blobs": git_tree_blobs(
            root, "specs/016-af-02-adversarial-test-strength/schemas"
        ),
        "enforcement_inventory_blob": git_blob(
            root, "specs/016-af-02-adversarial-test-strength/enforcement-inventory.json"
        ),
    }


def write_gate_input(
    base_root: Path,
    repository: str,
    number: int,
    base_sha: str,
    head_sha: str,
    changed_files: list[dict],
) -> tuple[Path, dict]:
    identity = build_base_identity(base_root, base_sha)
    payload = {
        "schema": GATE_INPUT_SCHEMA,
        "repository": repository,
        "pull_request": number,
        "base_sha": base_sha,
        "base_tree": identity["base_tree"],
        "head_sha": head_sha,
        "changed_files": changed_files,
        "base_identity": identity,
        "known_authority_paths": known_authority_paths(base_root),
    }
    output_dir = base_root / "target/af02-verifier"
    output_dir.mkdir(parents=True, exist_ok=True)
    path = output_dir / "af02-base-gate-input.json"
    path.write_text(canonical_json(payload), encoding="utf-8")
    return path, identity


def docker_create(workspace_root: Path) -> tuple[str, dict]:
    name = f"commandf-af02-base-gate-{os.getpid()}"
    command = [
        "docker",
        "create",
        "--pull=never",
        "--name",
        name,
        "--network=none",
        "--read-only",
        "--cap-drop=ALL",
        "--security-opt=no-new-privileges",
        "--cpus=2",
        "--memory=512m",
        "--pids-limit=64",
        "--tmpfs",
        "/tmp:rw,noexec,nosuid,nodev,size=64m",
        "--mount",
        f"type=bind,src={workspace_root},dst=/workspace,readonly",
        "--workdir",
        "/workspace/base",
        "--user",
        "65534:65534",
        RUNTIME_IMAGE,
        "/workspace/base/target/af02-verifier/release/commandf-af02-verifier",
        "verify-pr",
        "/workspace/base",
        "/workspace/candidate",
        "/workspace/base/target/af02-verifier/af02-base-gate-input.json",
    ]
    try:
        created = subprocess.run(
            command,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        fail(f"cannot create pinned verifier container: {exc}")
    container_id = created.stdout.strip()
    if not container_id:
        fail("docker create returned empty container id")
    try:
        inspected = subprocess.run(
            ["docker", "inspect", container_id],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=10,
        )
        value = json.loads(inspected.stdout)
    except (OSError, subprocess.SubprocessError, json.JSONDecodeError) as exc:
        fail(f"cannot inspect verifier container: {exc}")
    if not isinstance(value, list) or len(value) != 1 or not isinstance(value[0], dict):
        fail("docker inspect returned unexpected shape")
    return container_id, value[0]


def verify_container_inspect(inspect: dict) -> dict:
    config = inspect.get("Config")
    host = inspect.get("HostConfig")
    mounts = inspect.get("Mounts")
    if not isinstance(config, dict) or not isinstance(host, dict) or not isinstance(mounts, list):
        fail("docker inspection omits Config/HostConfig/Mounts")
    unprivileged = config.get("User") == "65534:65534"
    network_none = host.get("NetworkMode") == "none"
    root_read_only = host.get("ReadonlyRootfs") is True
    memory_limit = host.get("Memory") == PARSER_MEMORY_BYTES
    pid_limit = host.get("PidsLimit") == PARSER_PIDS
    cap_drop = host.get("CapDrop")
    security = host.get("SecurityOpt")
    if not isinstance(cap_drop, list) or "ALL" not in cap_drop:
        fail("verifier container does not drop all capabilities")
    if not isinstance(security, list) or not any("no-new-privileges" in item for item in security):
        fail("verifier container omits no-new-privileges")
    workspace_mount = [
        item
        for item in mounts
        if isinstance(item, dict) and item.get("Destination") == "/workspace"
    ]
    if len(workspace_mount) != 1 or workspace_mount[0].get("RW") is not False:
        fail("verifier workspace mount is not uniquely read-only")
    cgroup_v2 = Path("/sys/fs/cgroup/cgroup.controllers").is_file()
    if not all((unprivileged, network_none, root_read_only, memory_limit, pid_limit, cgroup_v2)):
        fail("verifier subprocess enforcement envelope is incomplete")
    return {
        "unprivileged": unprivileged,
        "network_none": network_none,
        "root_read_only": root_read_only,
        "memory_limit_enforced": memory_limit,
        "pid_limit_enforced": pid_limit,
        "cgroup_v2": cgroup_v2,
    }


def kill_container(container_id: str) -> None:
    subprocess.run(
        ["docker", "kill", container_id],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=5,
        check=False,
    )


def run_container_bounded(container_id: str) -> tuple[int, bytes, bytes, dict]:
    observation = {
        "wall_timeout_enforced": False,
        "stdout_exceeded": False,
        "stderr_exceeded": False,
        "termination": "CLEAN_EXIT",
    }
    try:
        process = subprocess.Popen(
            ["docker", "start", "--attach", container_id],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except OSError as exc:
        fail(f"cannot start verifier container: {exc}")
    assert process.stdout is not None and process.stderr is not None
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ, "stdout")
    selector.register(process.stderr, selectors.EVENT_READ, "stderr")
    streams = {"stdout": bytearray(), "stderr": bytearray()}
    deadline = time.monotonic() + PARSER_WALL_SECONDS
    observation["wall_timeout_enforced"] = True
    try:
        while selector.get_map():
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                observation["termination"] = "WALL_TIMEOUT_KILL"
                kill_container(container_id)
                process.kill()
                process.wait(timeout=5)
                return -1, bytes(streams["stdout"]), bytes(streams["stderr"]), observation
            events = selector.select(timeout=min(0.25, remaining))
            if not events and process.poll() is not None:
                events = [(key, selectors.EVENT_READ) for key in list(selector.get_map().values())]
            for key, _ in events:
                chunk = os.read(key.fileobj.fileno(), 65536)
                if not chunk:
                    selector.unregister(key.fileobj)
                    continue
                bucket = streams[key.data]
                bucket.extend(chunk)
                if len(bucket) > STREAM_LIMIT:
                    observation[f"{key.data}_exceeded"] = True
                    observation["termination"] = "SEMANTIC_REJECT"
                    kill_container(container_id)
                    process.kill()
                    process.wait(timeout=5)
                    return -1, bytes(streams["stdout"]), bytes(streams["stderr"]), observation
        code = process.wait(timeout=5)
        if code != 0:
            observation["termination"] = "SEMANTIC_REJECT"
    finally:
        selector.close()
    return code, bytes(streams["stdout"]), bytes(streams["stderr"]), observation


def validate_process_evidence(
    binary: Path,
    expected_binary_sha256: str,
    lock_path: Path,
    identity: dict,
    inspect: dict,
    stdout: bytes,
    stderr: bytes,
    observation: dict,
) -> tuple[dict, str]:
    enforcement = verify_container_inspect(inspect)
    after_binary = sha256_file(binary)
    observed_lock_blob = git_blob_sha1_file(lock_path)
    expected_lock_blob = identity["cargo_lock_blob"]
    cgroup_snapshot = {
        "controllers": Path("/sys/fs/cgroup/cgroup.controllers").read_text(encoding="utf-8").split(),
        "memory_bytes": inspect["HostConfig"].get("Memory"),
        "network_mode": inspect["HostConfig"].get("NetworkMode"),
        "pids_limit": inspect["HostConfig"].get("PidsLimit"),
        "read_only_root": inspect["HostConfig"].get("ReadonlyRootfs"),
        "user": inspect["Config"].get("User"),
    }
    evidence = {
        "binary_sha256": after_binary,
        "expected_binary_sha256": expected_binary_sha256,
        "cargo_lock_blob": observed_lock_blob,
        "expected_cargo_lock_blob": expected_lock_blob,
        "unprivileged": enforcement["unprivileged"],
        "cgroup_v2": enforcement["cgroup_v2"],
        "wall_timeout_enforced": observation["wall_timeout_enforced"],
        "memory_limit_enforced": enforcement["memory_limit_enforced"],
        "pid_limit_enforced": enforcement["pid_limit_enforced"],
        "network_none": enforcement["network_none"],
        "root_read_only": enforcement["root_read_only"],
        "stdout_observed": len(stdout),
        "stdout_limit": STREAM_LIMIT,
        "stdout_exceeded": observation["stdout_exceeded"],
        "stderr_observed": len(stderr),
        "stderr_limit": STREAM_LIMIT,
        "stderr_exceeded": observation["stderr_exceeded"],
        "termination": observation["termination"],
    }
    return evidence, sha256_bytes(canonical_json(cgroup_snapshot).encode("utf-8"))


def run_process_evidence_validator(binary: Path, evidence_path: Path) -> None:
    try:
        completed = subprocess.run(
            [str(binary), "validate-input-process-evidence", str(evidence_path)],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=5,
        )
    except (OSError, subprocess.SubprocessError) as exc:
        fail(f"cannot validate verifier process evidence: {exc}")
    if completed.returncode != 0:
        fail(
            "verifier process evidence failed semantic validation: "
            + completed.stderr.decode("utf-8", errors="replace")
        )


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
    if base_root.parent != candidate_root.parent:
        fail("base and candidate roots must be sibling directories")
    if base_root == candidate_root:
        fail("base and candidate roots must be disjoint")
    if base_root not in script_path.parents or candidate_root in script_path.parents:
        fail("executing gate script is not anchored inside canonical base")

    repository, number, base_sha, head_sha = read_event()
    observed_base = git_head(base_root)
    observed_head = git_head(candidate_root)
    if observed_base != base_sha:
        fail("canonical-base checkout SHA differs from GitHub event base SHA")
    if observed_head != head_sha:
        fail("candidate checkout SHA differs from GitHub event head SHA")

    changed_files = github_truth(repository, number, base_sha, head_sha)
    _, identity = write_gate_input(
        base_root, repository, number, base_sha, head_sha, changed_files
    )

    binary = base_root / "target/af02-verifier/release/commandf-af02-verifier"
    lock_path = base_root / "tools/af02-verifier/Cargo.lock"
    if not binary.is_file() or binary.is_symlink():
        fail("canonical verifier binary is missing or not a regular file")
    if git_blob_sha1_file(lock_path) != identity["cargo_lock_blob"]:
        fail("canonical verifier Cargo.lock bytes differ from Git identity")

    expected_binary_sha256 = sha256_file(binary)
    container_id = ""
    try:
        container_id, inspect = docker_create(base_root.parent)
        enforcement = verify_container_inspect(inspect)
        code, stdout, stderr, observation = run_container_bounded(container_id)

        evidence, cgroup_snapshot_sha256 = validate_process_evidence(
            binary,
            expected_binary_sha256,
            lock_path,
            identity,
            inspect,
            stdout,
            stderr,
            observation,
        )
        evidence_path = base_root / "target/af02-verifier/af02-process-evidence.json"
        evidence_path.write_text(canonical_json(evidence), encoding="utf-8")
        run_process_evidence_validator(binary, evidence_path)

        if code != 0:
            fail(
                "canonical verifier rejected candidate: "
                + stderr.decode("utf-8", errors="replace")
            )
        try:
            proof = json.loads(stdout)
        except (UnicodeError, json.JSONDecodeError) as exc:
            fail(f"canonical verifier returned invalid JSON: {exc}")
        if not isinstance(proof, dict) or proof.get("candidate_code_executed") is not False:
            fail("canonical verifier did not prove candidate-code non-execution")

        print(
            canonical_json(
                {
                    "schema": RUNTIME_SCHEMA,
                    "base_sha": base_sha,
                    "head_sha": head_sha,
                    "pull_request": number,
                    "repository": repository,
                    "candidate_code_executed": False,
                    "proof": proof,
                    "runtime": {
                        "image": RUNTIME_IMAGE,
                        "verifier_binary_sha256": evidence["binary_sha256"],
                        "cargo_lock_blob": evidence["cargo_lock_blob"],
                        "cgroup_snapshot_sha256": cgroup_snapshot_sha256,
                        "network_none": enforcement["network_none"],
                        "root_read_only": enforcement["root_read_only"],
                        "unprivileged": enforcement["unprivileged"],
                        "wall_timeout_enforced": evidence["wall_timeout_enforced"],
                        "stdout_bytes": len(stdout),
                        "stdout_exceeded": evidence["stdout_exceeded"],
                        "stderr_bytes": len(stderr),
                        "stderr_exceeded": evidence["stderr_exceeded"],
                        "termination": evidence["termination"],
                    },
                }
            )
        )
    finally:
        if container_id:
            subprocess.run(
                ["docker", "rm", "--force", container_id],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                timeout=10,
                check=False,
            )


if __name__ == "__main__":
    main()
PY
