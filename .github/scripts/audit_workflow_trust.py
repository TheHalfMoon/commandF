#!/usr/bin/env python3
"""Deterministic repository-owned GitHub workflow trust audit for AF-01."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

FULL_SHA_RE = re.compile(r"^[0-9a-fA-F]{40}$")
DIGEST_IMAGE_RE = re.compile(r"@sha256:[0-9a-fA-F]{64}$")
JOB_RE = re.compile(r"^  ([A-Za-z0-9_.-]+):\s*(?:#.*)?$")
USES_RE = re.compile(r"^(\s*)(?:-\s*)?uses:\s*(.+?)\s*$")
STEP_USES_RE = re.compile(r"^(\s*)-\s+uses:\s*(.+?)\s*$")
PERMISSION_RE = re.compile(r"^([A-Za-z0-9_-]+):\s*(read|write|none)\s*$")
CARGO_RE = re.compile(r"\bcargo\s+(bench|build|check|clippy|doc|metadata|run|test)\b")


@dataclass(frozen=True, order=True)
class Finding:
    code: str
    path: str
    job: str
    detail: str

    def as_dict(self) -> dict[str, str]:
        result = {"code": self.code, "path": self.path, "detail": self.detail}
        if self.job:
            result["job"] = self.job
        return result


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _scalar(value: str) -> str:
    value = value.strip()
    if " #" in value:
        value = value.split(" #", 1)[0].rstrip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        value = value[1:-1]
    return value


def _tracked_files(root: Path) -> list[str]:
    completed = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return sorted(path for path in completed.stdout.decode("utf-8").split("\0") if path)


def discover_security_files(paths: Iterable[str]) -> tuple[list[str], list[str]]:
    workflows: list[str] = []
    actions: list[str] = []
    for path in sorted(paths):
        name = Path(path).name
        if path.startswith(".github/workflows/") and name.endswith((".yml", ".yaml")):
            workflows.append(path)
        if name in {"action.yml", "action.yaml"}:
            actions.append(path)
    return workflows, actions


def _parse_permissions(
    lines: list[str], start: int, end: int, indent: int
) -> tuple[dict[str, str] | None, str | None]:
    prefix = " " * indent + "permissions:"
    for index in range(start, end):
        raw = lines[index]
        if not raw.startswith(prefix) or _indent(raw) != indent:
            continue
        value = _scalar(raw[len(prefix) :])
        if value == "{}":
            return {}, None
        if value:
            return None, f"unsupported permissions scalar {value!r}"
        permissions: dict[str, str] = {}
        cursor = index + 1
        while cursor < end:
            child = lines[cursor]
            if not child.strip() or child.lstrip().startswith("#"):
                cursor += 1
                continue
            child_indent = _indent(child)
            if child_indent <= indent:
                break
            if child_indent != indent + 2:
                return None, "permissions block contains unsupported nested syntax"
            parsed = PERMISSION_RE.match(child.strip())
            if not parsed:
                return None, f"unsupported permission entry {child.strip()!r}"
            key, permission = parsed.groups()
            if key in permissions:
                return None, f"duplicate permission key {key!r}"
            permissions[key] = permission
            cursor += 1
        return permissions, None
    return None, None


def _job_ranges(lines: list[str]) -> tuple[dict[str, tuple[int, int]], str | None]:
    jobs_index = next((i for i, line in enumerate(lines) if line.strip() == "jobs:" and _indent(line) == 0), None)
    if jobs_index is None:
        return {}, "workflow has no top-level jobs mapping"

    starts: list[tuple[str, int]] = []
    for index in range(jobs_index + 1, len(lines)):
        line = lines[index]
        if line.strip() and _indent(line) == 0:
            break
        matched = JOB_RE.match(line)
        if matched:
            starts.append((matched.group(1), index))
    if not starts:
        return {}, "workflow jobs mapping has no statically named jobs"
    if len({name for name, _ in starts}) != len(starts):
        return {}, "workflow contains duplicate statically named jobs"

    ranges: dict[str, tuple[int, int]] = {}
    for position, (name, start) in enumerate(starts):
        end = starts[position + 1][1] if position + 1 < len(starts) else len(lines)
        for index in range(start + 1, end):
            line = lines[index]
            if line.strip() and _indent(line) == 0:
                end = index
                break
        ranges[name] = (start, end)
    return ranges, None


def _job_scalar(lines: list[str], start: int, end: int, key: str) -> str | None:
    prefix = "    " + key + ":"
    for index in range(start + 1, end):
        line = lines[index]
        if _indent(line) == 4 and line.startswith(prefix):
            return _scalar(line[len(prefix) :])
    return None


def _container_images(lines: list[str], start: int, end: int) -> list[str]:
    images: list[str] = []
    for index in range(start + 1, end):
        line = lines[index]
        stripped = line.strip()
        if _indent(line) == 4 and stripped.startswith("container:"):
            value = _scalar(stripped.split(":", 1)[1])
            if value:
                images.append(value)
        if stripped.startswith("image:") and _indent(line) >= 6:
            value = _scalar(stripped.split(":", 1)[1])
            if value:
                images.append(value)
    return images


def _all_uses(lines: list[str], start: int = 0, end: int | None = None) -> list[tuple[int, str]]:
    if end is None:
        end = len(lines)
    result: list[tuple[int, str]] = []
    for index in range(start, end):
        matched = USES_RE.match(lines[index])
        if matched:
            result.append((index, _scalar(matched.group(2))))
    return result


def _external_ref_is_immutable(reference: str) -> bool:
    if reference.startswith("./"):
        return True
    if reference.startswith("docker://"):
        return bool(DIGEST_IMAGE_RE.search(reference))
    if "@" not in reference:
        return False
    _, revision = reference.rsplit("@", 1)
    return bool(FULL_SHA_RE.fullmatch(revision))


def _checkout_has_credentials_disabled(lines: list[str], uses_index: int) -> bool:
    matched = STEP_USES_RE.match(lines[uses_index])
    if not matched:
        return False
    step_indent = len(matched.group(1))
    end = len(lines)
    for index in range(uses_index + 1, len(lines)):
        line = lines[index]
        if re.match(rf"^ {{{step_indent}}}-\s+", line):
            end = index
            break
    for line in lines[uses_index + 1 : end]:
        if re.match(r"^\s*persist-credentials:\s*false\s*(?:#.*)?$", line):
            return True
    return False


def _valid_exception(exception: object) -> bool:
    if not isinstance(exception, dict):
        return False
    required = {"rule", "path", "reason", "revisit"}
    if not required.issubset(exception):
        return False
    if not all(isinstance(exception[key], str) and exception[key].strip() for key in required):
        return False
    if len(exception["reason"].strip()) < 10 or len(exception["revisit"].strip()) < 5:
        return False
    return set(exception).issubset(required | {"job", "detail"})


def _excepted(policy: dict, finding: Finding) -> bool:
    for exception in policy.get("exceptions", []):
        if exception.get("rule") != finding.code or exception.get("path") != finding.path:
            continue
        if exception.get("job", finding.job) != finding.job:
            continue
        if "detail" in exception and exception["detail"] != finding.detail:
            continue
        return True
    return False


def audit_workflow(path: str, text: str, expected: dict, policy: dict) -> list[Finding]:
    findings: list[Finding] = []
    if "\t" in text:
        findings.append(Finding("malformed_yaml", path, "", "tab indentation is not supported"))
        return findings
    lines = text.splitlines()
    jobs, jobs_error = _job_ranges(lines)
    if jobs_error:
        findings.append(Finding("malformed_yaml", path, "", jobs_error))
        return findings

    expected_jobs = expected.get("jobs")
    if not isinstance(expected_jobs, dict):
        findings.append(Finding("invalid_policy", path, "", "workflow policy must contain a jobs object"))
        return findings
    actual_names = set(jobs)
    expected_names = set(expected_jobs)
    for name in sorted(actual_names - expected_names):
        findings.append(Finding("unplanned_job", path, name, "job is not declared in workflow trust policy"))
    for name in sorted(expected_names - actual_names):
        findings.append(Finding("missing_job", path, name, "policy job is missing from workflow"))

    top_permissions, top_permission_error = _parse_permissions(lines, 0, len(lines), 0)
    if top_permission_error:
        findings.append(Finding("permissions_syntax", path, "", top_permission_error))

    for job in sorted(actual_names & expected_names):
        start, end = jobs[job]
        expected_job = expected_jobs[job]
        runner = _job_scalar(lines, start, end, "runs-on")
        if runner != expected_job.get("runner"):
            findings.append(
                Finding("runner_mismatch", path, job, f"expected runner {expected_job.get('runner')!r}, found {runner!r}")
            )
        if runner and runner.endswith("-latest"):
            findings.append(Finding("mutable_runner", path, job, f"runner {runner!r} is a mutable latest label"))

        timeout = _job_scalar(lines, start, end, "timeout-minutes")
        try:
            timeout_value = int(timeout) if timeout is not None else None
        except ValueError:
            timeout_value = None
        timeout_limit = expected_job.get("timeout_minutes")
        if not isinstance(timeout_limit, int) or timeout_limit <= 0:
            findings.append(Finding("invalid_policy", path, job, "timeout_minutes must be a positive integer"))
        elif timeout_value is None or timeout_value <= 0 or timeout_value > timeout_limit:
            findings.append(
                Finding("timeout_policy", path, job, f"timeout must be 1..{timeout_limit} minutes, found {timeout!r}")
            )

        job_permissions, job_permission_error = _parse_permissions(lines, start + 1, end, 4)
        if job_permission_error:
            findings.append(Finding("permissions_syntax", path, job, job_permission_error))
        effective_permissions = job_permissions if job_permissions is not None else top_permissions
        if effective_permissions is None:
            findings.append(
                Finding("unresolved_permissions", path, job, "job inherits undocumented GitHub default token permissions")
            )
        elif effective_permissions != expected_job.get("permissions"):
            findings.append(
                Finding(
                    "permission_mismatch",
                    path,
                    job,
                    f"expected effective permissions {expected_job.get('permissions')!r}, found {effective_permissions!r}",
                )
            )

        if policy["rules"].get("require_container_digest", False):
            for image in _container_images(lines, start, end):
                if not DIGEST_IMAGE_RE.search(image):
                    findings.append(
                        Finding("mutable_container", path, job, f"container image is not sha256 digest-bound: {image}")
                    )

        locked_subcommands = set(policy["rules"].get("cargo_locked_subcommands", []))
        for line in lines[start + 1 : end]:
            command = line.strip()
            for match in CARGO_RE.finditer(command):
                subcommand = match.group(1)
                if subcommand in locked_subcommands and "--locked" not in command[match.start() :]:
                    findings.append(
                        Finding("cargo_unlocked", path, job, f"cargo {subcommand} invocation omits --locked: {command}")
                    )

    for index, reference in _all_uses(lines):
        if policy["rules"].get("require_external_uses_full_sha", False) and not _external_ref_is_immutable(reference):
            findings.append(Finding("mutable_uses", path, "", f"uses reference is not immutable: {reference}"))
        if reference.startswith("actions/checkout@") and policy["rules"].get(
            "require_checkout_credentials_disabled", False
        ):
            if not _checkout_has_credentials_disabled(lines, index):
                findings.append(
                    Finding("checkout_credentials", path, "", "checkout step does not set persist-credentials: false")
                )

    return [finding for finding in findings if not _excepted(policy, finding)]


def audit_action_metadata(path: str, text: str, policy: dict) -> list[Finding]:
    findings: list[Finding] = []
    if "\t" in text:
        return [Finding("malformed_yaml", path, "", "tab indentation is not supported")]
    lines = text.splitlines()
    if not any(line.strip() == "runs:" for line in lines):
        findings.append(Finding("malformed_action_metadata", path, "", "Action metadata has no runs mapping"))
    for _, reference in _all_uses(lines):
        if policy["rules"].get("require_external_uses_full_sha", False) and not _external_ref_is_immutable(reference):
            findings.append(Finding("mutable_uses", path, "", f"uses reference is not immutable: {reference}"))
    return [finding for finding in findings if not _excepted(policy, finding)]


def audit_repository(root: Path, policy: dict, tracked_files: Iterable[str] | None = None) -> dict:
    findings: list[Finding] = []
    if policy.get("schema") != 1 or not isinstance(policy.get("rules"), dict):
        findings.append(Finding("invalid_policy", ".github/workflow-trust-policy.json", "", "unsupported policy schema"))
    exceptions = policy.get("exceptions", [])
    if not isinstance(exceptions, list) or any(not _valid_exception(item) for item in exceptions):
        findings.append(
            Finding("invalid_policy", ".github/workflow-trust-policy.json", "", "every exception requires bounded rule/path/reason/revisit fields")
        )

    paths = list(tracked_files) if tracked_files is not None else _tracked_files(root)
    workflows, actions = discover_security_files(paths)
    expected_workflows = policy.get("workflows", {})
    if not isinstance(expected_workflows, dict):
        expected_workflows = {}
        findings.append(Finding("invalid_policy", ".github/workflow-trust-policy.json", "", "workflows must be an object"))

    for path in sorted(set(workflows) - set(expected_workflows)):
        findings.append(Finding("unplanned_workflow", path, "", "tracked workflow is absent from policy"))
    for path in sorted(set(expected_workflows) - set(workflows)):
        findings.append(Finding("missing_workflow", path, "", "policy workflow is not tracked"))

    for path in sorted(set(workflows) & set(expected_workflows)):
        findings.extend(audit_workflow(path, (root / path).read_text(encoding="utf-8"), expected_workflows[path], policy))
    for path in actions:
        findings.extend(audit_action_metadata(path, (root / path).read_text(encoding="utf-8"), policy))

    filtered = sorted(finding for finding in findings if not _excepted(policy, finding))
    return {
        "schema": 1,
        "ok": not filtered,
        "workflows": workflows,
        "action_metadata": actions,
        "findings": [finding.as_dict() for finding in filtered],
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path("."))
    parser.add_argument("--policy", type=Path, default=Path(".github/workflow-trust-policy.json"))
    args = parser.parse_args()

    root = args.root.resolve()
    policy_path = args.policy if args.policy.is_absolute() else root / args.policy
    try:
        policy = json.loads(policy_path.read_text(encoding="utf-8"))
        result = audit_repository(root, policy)
    except (OSError, UnicodeError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        result = {
            "schema": 1,
            "ok": False,
            "workflows": [],
            "action_metadata": [],
            "findings": [
                {
                    "code": "audit_operational_failure",
                    "path": str(args.policy),
                    "detail": str(error),
                }
            ],
        }

    rendered = json.dumps(result, indent=2, sort_keys=True, separators=(",", ": ")) + "\n"
    sys.stdout.write(rendered)
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
