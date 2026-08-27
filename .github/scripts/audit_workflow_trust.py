#!/usr/bin/env python3
"""Deterministic repository-owned GitHub workflow trust audit for AF-01."""

from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

FULL_SHA_RE = re.compile(r"^[0-9a-fA-F]{40}$")
DIGEST_IMAGE_RE = re.compile(r"@sha256:[0-9a-fA-F]{64}$")
JOB_RE = re.compile(r"^  ([A-Za-z0-9_.-]+):\s*(?:#.*)?$")
USES_RE = re.compile(r"^(\s*)(?:-\s*)?uses:\s*(.+?)\s*$")
STEP_LIST_RE = re.compile(r"^(\s*)-\s+\S")
PERMISSION_RE = re.compile(r"^([A-Za-z0-9_-]+):\s*(read|write|none)\s*$")
FLOW_USES_RE = re.compile(r"[\[{,]\s*[\"']?uses[\"']?\s*:")
QUOTED_USES_RE = re.compile(r"^\s*(?:-\s*)?[\"']uses[\"']\s*:")
QUOTED_JOB_CONTAINER_RE = re.compile(r"^\s{4}[\"'](?:container|services)[\"']\s*:")
QUOTED_IMAGE_RE = re.compile(r"^\s*(?:[\"']image[\"'])\s*:")
QUOTED_PERMISSION_KEY_RE = re.compile(r"^(?: {4})?[\"']permissions[\"']\s*:")
FLOW_SERVICES_RE = re.compile(r"^\s{4}services\s*:\s*[\[{]")
BLOCK_SCALAR_RE = re.compile(r":\s*[|>][+-]?\s*(?:#.*)?$")
SHELL_SEPARATOR_RE = re.compile(r"(?:\r?\n|&&|\|\||;|(?<!\|)\|(?!\|))")
SHELL_ASSIGNMENT_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=.*$")
BARE_DYNAMIC_COMMAND_RE = re.compile(r"^\$(?:[A-Za-z_][A-Za-z0-9_]*|\{[A-Za-z_][A-Za-z0-9_]*\})$")
CARGO_WORD_RE = re.compile(r"(?<![A-Za-z0-9_])cargo(?![A-Za-z0-9_])")
HEREDOC_RE = re.compile(r"<<(?P<tabs>-)?\s*(?P<quote>['\"]?)(?P<delimiter>[A-Za-z_][A-Za-z0-9_]*)\2")
DYNAMIC_EXECUTABLE_RE = re.compile(
    r"""^\s*
    (?:(?:[A-Za-z_][A-Za-z0-9_]*=[^\s;|&]+)\s+)*
    (?:(?:!|do|if|then|until|while)\s+)*
    (?:(?:env|command|exec|nohup|retry)(?:\s+-[^\s]+|\s+[A-Za-z_][A-Za-z0-9_]*=[^\s;|&]+)*\s+)*
    [\"']?
    (?:
        \$(?:[A-Za-z_][A-Za-z0-9_]*(?=[\"']?(?:\s|$))|\{[A-Za-z_][A-Za-z0-9_]*\}(?=[\"']?(?:\s|$)))
        |\$\(
        |`
    )
    """,
    re.VERBOSE,
)
DYNAMIC_SHELL_RE = re.compile(
    r"^\s*(?:(?:[A-Za-z_][A-Za-z0-9_]*=[^\s;|&]+)\s+)*(?:(?:!|do|if|then|until|while)\s+)*(?:eval\b|(?:bash|dash|ksh|sh|zsh)\b[^\n]*\s-c(?:\s|$))"
)

LOCKFILE_CARGO_SUBCOMMANDS = frozenset(
    {"bench", "build", "check", "clippy", "doc", "metadata", "run", "test"}
)
CARGO_INFO_FLAGS = frozenset({"--version", "-V"})
DYNAMIC_COMMAND_BUILTINS = frozenset({"eval", "alias"})
SHELL_COMMAND_WRAPPERS = frozenset({"command", "exec", "nohup", "retry"})
SHELL_INTERPRETERS = frozenset({"bash", "dash", "ksh", "sh", "zsh"})
SHELL_CONTROL_WORDS = frozenset({"!", "do", "if", "then", "until", "while"})
BOOLEAN_RULES = frozenset(
    {
        "require_container_digest",
        "require_checkout_credentials_disabled",
        "require_external_uses_full_sha",
    }
)
SUPPORTED_RULE_KEYS = frozenset({"cargo_locked_subcommands", *BOOLEAN_RULES})
SUPPORTED_RUNNERS = frozenset({"ubuntu-24.04"})
MAX_JOB_TIMEOUT_MINUTES = 30
SUPPORTED_JOB_POLICY_KEYS = frozenset({"permissions", "runner", "timeout_minutes"})
SUPPORTED_TOP_LEVEL_POLICY_KEYS = frozenset(
    {"schema", "rules", "rationales", "workflows", "exceptions"}
)


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
    jobs_index = next(
        (
            index
            for index, line in enumerate(lines)
            if line.strip() == "jobs:" and _indent(line) == 0
        ),
        None,
    )
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
    """Return job-container and service-container image scalars only."""
    images: list[str] = []
    service_indent: int | None = None
    in_job_container = False

    for index in range(start + 1, end):
        line = lines[index]
        stripped = line.strip()
        indent = _indent(line)
        if not stripped or stripped.startswith("#"):
            continue

        if indent == 4:
            in_job_container = False
            service_indent = None
            if stripped.startswith("container:"):
                value = _scalar(stripped.split(":", 1)[1])
                if value:
                    images.append(value)
                else:
                    in_job_container = True
            elif stripped == "services:":
                service_indent = 4
            continue

        if in_job_container and indent == 6 and stripped.startswith("image:"):
            value = _scalar(stripped.split(":", 1)[1])
            if value:
                images.append(value)
            continue

        if service_indent is not None and indent == 8 and stripped.startswith("image:"):
            value = _scalar(stripped.split(":", 1)[1])
            if value:
                images.append(value)

    return images


def _all_uses(
    lines: list[str], start: int = 0, end: int | None = None
) -> list[tuple[int, str]]:
    if end is None:
        end = len(lines)
    result: list[tuple[int, str]] = []
    for index in range(start, end):
        matched = USES_RE.match(lines[index])
        if matched:
            result.append((index, _scalar(matched.group(2))))
    return result


def _block_scalar_line_indexes(lines: list[str]) -> set[int]:
    indexes: set[int] = set()
    for index, line in enumerate(lines):
        if not BLOCK_SCALAR_RE.search(line):
            continue
        indent = _indent(line)
        cursor = index + 1
        while cursor < len(lines):
            child = lines[cursor]
            if child.strip() and _indent(child) <= indent:
                break
            indexes.add(cursor)
            cursor += 1
    return indexes


def _unsupported_trust_syntax(lines: list[str]) -> list[str]:
    """Reject valid YAML forms that the constrained trust parser cannot safely normalize."""
    block_lines = _block_scalar_line_indexes(lines)
    unsupported: list[str] = []
    in_job_container = False
    in_services = False

    for index, line in enumerate(lines):
        if index in block_lines:
            continue
        stripped = line.lstrip()
        indent = _indent(line)
        if not stripped or stripped.startswith("#"):
            continue
        if stripped.startswith("run:") or re.match(r"^-\s+run:\s*", stripped):
            continue

        if (
            QUOTED_USES_RE.search(line)
            or FLOW_USES_RE.search(line)
            or QUOTED_PERMISSION_KEY_RE.search(line)
        ):
            unsupported.append(line.strip())
            continue

        if indent == 4:
            in_job_container = False
            in_services = False
            if QUOTED_JOB_CONTAINER_RE.search(line) or FLOW_SERVICES_RE.search(line):
                unsupported.append(line.strip())
                continue
            if stripped == "container:":
                in_job_container = True
            elif stripped == "services:":
                in_services = True
            continue

        if in_job_container and indent == 6 and QUOTED_IMAGE_RE.search(line):
            unsupported.append(line.strip())
            continue
        if in_services and indent == 8 and QUOTED_IMAGE_RE.search(line):
            unsupported.append(line.strip())

    return unsupported


def _external_ref_is_immutable(reference: str) -> bool:
    if reference.startswith("./"):
        return True
    if reference.startswith("docker://"):
        return bool(DIGEST_IMAGE_RE.search(reference))
    if "@" not in reference:
        return False
    _, revision = reference.rsplit("@", 1)
    return bool(FULL_SHA_RE.fullmatch(revision))


def _step_bounds(lines: list[str], uses_index: int) -> tuple[int, int, int] | None:
    uses_line = lines[uses_index]
    uses_indent = _indent(uses_line)
    if STEP_LIST_RE.match(uses_line) and uses_line.lstrip().startswith("- uses:"):
        start = uses_index
        step_indent = uses_indent
    else:
        start = -1
        step_indent = -1
        for index in range(uses_index - 1, -1, -1):
            line = lines[index]
            if not line.strip():
                continue
            indent = _indent(line)
            if indent >= uses_indent:
                continue
            if STEP_LIST_RE.match(line):
                start = index
                step_indent = indent
                break
            if indent < uses_indent and line.strip().endswith(":"):
                break
        if start < 0:
            return None

    end = len(lines)
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if not line.strip():
            continue
        if _indent(line) == step_indent and STEP_LIST_RE.match(line):
            end = index
            break
        if _indent(line) < step_indent:
            end = index
            break
    return start, end, step_indent


def _checkout_has_credentials_disabled(lines: list[str], uses_index: int) -> bool:
    bounds = _step_bounds(lines, uses_index)
    if bounds is None:
        return False
    start, end, step_indent = bounds
    with_indexes = [
        index
        for index in range(start + 1, end)
        if _indent(lines[index]) == step_indent + 2 and lines[index].strip() == "with:"
    ]
    if len(with_indexes) != 1:
        return False

    with_index = with_indexes[0]
    entries: list[str] = []
    for index in range(with_index + 1, end):
        line = lines[index]
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = _indent(line)
        if indent <= step_indent + 2:
            break
        if indent == step_indent + 4 and line.strip().startswith("persist-credentials:"):
            entries.append(_scalar(line.strip().split(":", 1)[1]))
    return entries == ["false"]


def _run_scripts(lines: list[str], start: int, end: int) -> list[str]:
    """Extract inline and block step `run:` scripts from a statically structured job."""
    scripts: list[str] = []
    index = start + 1
    while index < end:
        line = lines[index]
        stripped = line.strip()
        indent = _indent(line)
        if indent < 6 or not stripped.startswith("run:"):
            index += 1
            continue

        value = _scalar(stripped.split(":", 1)[1])
        if value in {"|", "|-", "|+", ">", ">-", ">+"}:
            block: list[str] = []
            cursor = index + 1
            while cursor < end:
                child = lines[cursor]
                if child.strip() and _indent(child) <= indent:
                    break
                if not child.strip():
                    block.append("")
                else:
                    child_indent = _indent(child)
                    if child_indent < indent + 2:
                        break
                    block.append(child[indent + 2 :])
                cursor += 1
            scripts.append("\n".join(block))
            index = cursor
            continue
        if value:
            scripts.append(value)
        index += 1
    return scripts


def _without_heredoc_bodies(script: str) -> str:
    """Remove heredoc bodies because their contents are data, not shell commands."""
    kept: list[str] = []
    pending: list[tuple[str, bool]] = []
    for line in script.splitlines():
        if pending:
            delimiter, strip_tabs = pending[0]
            candidate = line.lstrip("\t") if strip_tabs else line
            if candidate == delimiter:
                pending.pop(0)
            continue

        kept.append(line)
        for matched in HEREDOC_RE.finditer(line):
            pending.append((matched.group("delimiter"), matched.group("tabs") is not None))
    return "\n".join(kept)


def _logical_shell_segments(script: str) -> list[str]:
    """Join backslash continuations, strip heredoc data, then split shell boundaries."""
    command_text = _without_heredoc_bodies(script)
    joined = re.sub(r"\\[ \t]*\r?\n[ \t]*", " ", command_text)
    return [segment.strip() for segment in SHELL_SEPARATOR_RE.split(joined) if segment.strip()]


def _command_token_index(tokens: list[str]) -> int | None:
    """Locate a statically visible executable token in the supported shell subset."""
    index = 0
    while index < len(tokens) and SHELL_ASSIGNMENT_RE.fullmatch(tokens[index]):
        index += 1
    while index < len(tokens) and tokens[index] in SHELL_CONTROL_WORDS:
        index += 1
    if index >= len(tokens):
        return None

    if tokens[index] == "env":
        index += 1
        while index < len(tokens) and (
            tokens[index].startswith("-") or SHELL_ASSIGNMENT_RE.fullmatch(tokens[index])
        ):
            index += 1
        if index >= len(tokens):
            return None

    if tokens[index] in SHELL_COMMAND_WRAPPERS:
        index += 1
        while index < len(tokens) and tokens[index].startswith("-"):
            index += 1
        if index >= len(tokens):
            return None
    return index


def _raw_indirect_cargo_finding(path: str, job: str, segment: str) -> Finding | None:
    """Reject executable-position indirection without parsing unrelated shell arguments."""
    if DYNAMIC_EXECUTABLE_RE.match(segment):
        return Finding(
            "unsupported_cargo_indirect",
            path,
            job,
            f"dynamic executable position could resolve to Cargo: {segment}",
        )
    if DYNAMIC_SHELL_RE.match(segment):
        return Finding(
            "unsupported_cargo_indirect",
            path,
            job,
            f"dynamic shell execution is not statically auditable for Cargo: {segment}",
        )
    return None


def _cargo_findings(
    path: str,
    job: str,
    lines: list[str],
    start: int,
    end: int,
    locked_subcommands: set[str],
) -> list[Finding]:
    findings: list[Finding] = []
    for script in _run_scripts(lines, start, end):
        for segment in _logical_shell_segments(script):
            indirect = _raw_indirect_cargo_finding(path, job, segment)
            if indirect is not None:
                findings.append(indirect)
                continue
            if "cargo" not in segment:
                continue
            try:
                tokens = shlex.split(segment, comments=True, posix=True)
            except ValueError as error:
                findings.append(
                    Finding(
                        "unsupported_shell_syntax",
                        path,
                        job,
                        f"cannot safely parse Cargo-containing shell segment: {error}: {segment}",
                    )
                )
                continue

            command_index = _command_token_index(tokens)
            if command_index is None:
                if any(CARGO_WORD_RE.search(token) for token in tokens):
                    findings.append(
                        Finding(
                            "unsupported_cargo_indirect",
                            path,
                            job,
                            f"Cargo appears without a statically executable command: {segment}",
                        )
                    )
                continue

            command = tokens[command_index]
            if "$" in command or "`" in command:
                findings.append(
                    Finding(
                        "unsupported_cargo_indirect",
                        path,
                        job,
                        f"dynamic Cargo executable path is not supported: {segment}",
                    )
                )
                continue

            is_direct_cargo = command == "cargo" or command.rsplit("/", 1)[-1] == "cargo"
            if not is_direct_cargo:
                if any(CARGO_WORD_RE.search(token) for token in tokens):
                    findings.append(
                        Finding(
                            "unsupported_cargo_indirect",
                            path,
                            job,
                            f"Cargo appears outside the statically executable command position: {segment}",
                        )
                    )
                continue

            index = command_index
            subcommand_index = index + 1
            if subcommand_index < len(tokens) and tokens[subcommand_index].startswith("+"):
                subcommand_index += 1
            if subcommand_index >= len(tokens):
                findings.append(
                    Finding(
                        "unsupported_cargo_syntax",
                        path,
                        job,
                        f"cannot identify Cargo subcommand: {segment}",
                    )
                )
                continue
            subcommand = tokens[subcommand_index]
            if subcommand in CARGO_INFO_FLAGS and subcommand_index == len(tokens) - 1:
                continue
            if subcommand.startswith("-"):
                findings.append(
                    Finding(
                        "unsupported_cargo_syntax",
                        path,
                        job,
                        f"Cargo global-option syntax requires explicit auditor support: {segment}",
                    )
                )
                continue
            if subcommand not in locked_subcommands:
                continue
            invocation = tokens[index:]
            if "--locked" not in invocation:
                findings.append(
                    Finding(
                        "cargo_unlocked",
                        path,
                        job,
                        f"cargo {subcommand} invocation omits --locked: {' '.join(invocation)}",
                    )
                )
    return findings


def _valid_exception(exception: object) -> bool:
    if not isinstance(exception, dict):
        return False
    required = {"rule", "path", "reason", "revisit"}
    if not required.issubset(exception):
        return False
    if not all(
        isinstance(exception[key], str) and exception[key].strip() for key in required
    ):
        return False
    for optional in ("job", "detail"):
        if optional in exception and (
            not isinstance(exception[optional], str) or not exception[optional].strip()
        ):
            return False
    if len(exception["reason"].strip()) < 10 or len(exception["revisit"].strip()) < 5:
        return False
    return set(exception).issubset(required | {"job", "detail"})


def _excepted(policy: dict, finding: Finding) -> bool:
    exceptions = policy.get("exceptions", [])
    if not isinstance(exceptions, list):
        return False
    for exception in exceptions:
        if not isinstance(exception, dict):
            continue
        if exception.get("rule") != finding.code or exception.get("path") != finding.path:
            continue
        if exception.get("job", finding.job) != finding.job:
            continue
        if "detail" in exception and exception["detail"] != finding.detail:
            continue
        return True
    return False


def _uses_findings(path: str, lines: list[str], policy: dict) -> list[Finding]:
    findings: list[Finding] = []
    rules = policy.get("rules", {})
    if not isinstance(rules, dict):
        return findings

    for syntax in _unsupported_trust_syntax(lines):
        findings.append(
            Finding(
                "unsupported_trust_syntax",
                path,
                "",
                f"trust-sensitive YAML syntax is not supported by the constrained parser: {syntax}",
            )
        )

    for index, reference in _all_uses(lines):
        if rules.get("require_external_uses_full_sha", False) and not _external_ref_is_immutable(
            reference
        ):
            findings.append(
                Finding("mutable_uses", path, "", f"uses reference is not immutable: {reference}")
            )
        if reference.startswith("actions/checkout@") and rules.get(
            "require_checkout_credentials_disabled", False
        ):
            if not _checkout_has_credentials_disabled(lines, index):
                findings.append(
                    Finding(
                        "checkout_credentials",
                        path,
                        "",
                        "checkout step does not set with.persist-credentials: false exactly once",
                    )
                )
    return findings


def audit_workflow(path: str, text: str, expected: dict, policy: dict) -> list[Finding]:
    findings: list[Finding] = []
    if "\t" in text:
        return [Finding("malformed_yaml", path, "", "tab indentation is not supported")]
    lines = text.splitlines()
    jobs, jobs_error = _job_ranges(lines)
    if jobs_error:
        return [Finding("malformed_yaml", path, "", jobs_error)]

    expected_jobs = expected.get("jobs") if isinstance(expected, dict) else None
    if not isinstance(expected_jobs, dict):
        return [
            Finding(
                "invalid_policy",
                path,
                "",
                "workflow policy must contain a jobs object",
            )
        ]

    actual_names = set(jobs)
    expected_names = set(expected_jobs)
    for name in sorted(actual_names - expected_names):
        findings.append(
            Finding("unplanned_job", path, name, "job is not declared in workflow trust policy")
        )
    for name in sorted(expected_names - actual_names):
        findings.append(Finding("missing_job", path, name, "policy job is missing from workflow"))

    top_permissions, top_permission_error = _parse_permissions(lines, 0, len(lines), 0)
    if top_permission_error:
        findings.append(Finding("permissions_syntax", path, "", top_permission_error))

    rules = policy.get("rules", {})
    if not isinstance(rules, dict):
        rules = {}

    for job in sorted(actual_names & expected_names):
        start, end = jobs[job]
        expected_job = expected_jobs[job]
        if not isinstance(expected_job, dict):
            findings.append(
                Finding("invalid_policy", path, job, "job policy must be an object")
            )
            continue

        runner = _job_scalar(lines, start, end, "runs-on")
        if runner != expected_job.get("runner"):
            findings.append(
                Finding(
                    "runner_mismatch",
                    path,
                    job,
                    f"expected runner {expected_job.get('runner')!r}, found {runner!r}",
                )
            )
        if runner and runner.endswith("-latest"):
            findings.append(
                Finding("mutable_runner", path, job, f"runner {runner!r} is a mutable latest label")
            )

        timeout = _job_scalar(lines, start, end, "timeout-minutes")
        try:
            timeout_value = int(timeout) if timeout is not None else None
        except ValueError:
            timeout_value = None
        timeout_limit = expected_job.get("timeout_minutes")
        if not isinstance(timeout_limit, int) or isinstance(timeout_limit, bool) or timeout_limit <= 0:
            findings.append(
                Finding("invalid_policy", path, job, "timeout_minutes must be a positive integer")
            )
        elif timeout_value is None or timeout_value <= 0 or timeout_value > timeout_limit:
            findings.append(
                Finding(
                    "timeout_policy",
                    path,
                    job,
                    f"timeout must be 1..{timeout_limit} minutes, found {timeout!r}",
                )
            )

        job_permissions, job_permission_error = _parse_permissions(lines, start + 1, end, 4)
        if job_permission_error:
            findings.append(Finding("permissions_syntax", path, job, job_permission_error))
        effective_permissions = job_permissions if job_permissions is not None else top_permissions
        if effective_permissions is None:
            findings.append(
                Finding(
                    "unresolved_permissions",
                    path,
                    job,
                    "job inherits undocumented GitHub default token permissions",
                )
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

        if rules.get("require_container_digest", False):
            for image in _container_images(lines, start, end):
                if not DIGEST_IMAGE_RE.search(image):
                    findings.append(
                        Finding(
                            "mutable_container",
                            path,
                            job,
                            f"container image is not sha256 digest-bound: {image}",
                        )
                    )

        locked_subcommands = set(rules.get("cargo_locked_subcommands", []))
        findings.extend(
            _cargo_findings(path, job, lines, start, end, locked_subcommands)
        )

    findings.extend(_uses_findings(path, lines, policy))
    return [finding for finding in findings if not _excepted(policy, finding)]


def audit_action_metadata(path: str, text: str, policy: dict) -> list[Finding]:
    if "\t" in text:
        return [Finding("malformed_yaml", path, "", "tab indentation is not supported")]
    lines = text.splitlines()
    findings: list[Finding] = []
    if not any(line.strip() == "runs:" for line in lines):
        findings.append(
            Finding(
                "malformed_action_metadata",
                path,
                "",
                "Action metadata has no runs mapping",
            )
        )
    findings.extend(_uses_findings(path, lines, policy))
    return [finding for finding in findings if not _excepted(policy, finding)]


def _policy_errors(policy: object) -> list[Finding]:
    findings: list[Finding] = []
    policy_path = ".github/workflow-trust-policy.json"
    if not isinstance(policy, dict):
        return [Finding("invalid_policy", policy_path, "", "policy root must be an object")]

    unknown_top = set(policy) - SUPPORTED_TOP_LEVEL_POLICY_KEYS
    if unknown_top:
        findings.append(
            Finding(
                "invalid_policy",
                policy_path,
                "",
                f"unsupported top-level policy keys: {sorted(unknown_top)!r}",
            )
        )
    if policy.get("schema") != 1:
        findings.append(Finding("invalid_policy", policy_path, "", "unsupported policy schema"))

    rules = policy.get("rules")
    if not isinstance(rules, dict):
        findings.append(Finding("invalid_policy", policy_path, "", "rules must be an object"))
    else:
        if set(rules) != SUPPORTED_RULE_KEYS:
            findings.append(
                Finding(
                    "invalid_policy",
                    policy_path,
                    "",
                    f"rules must contain exactly {sorted(SUPPORTED_RULE_KEYS)!r}",
                )
            )
        cargo_rules = rules.get("cargo_locked_subcommands")
        if (
            not isinstance(cargo_rules, list)
            or any(not isinstance(item, str) for item in cargo_rules)
            or len(cargo_rules) != len(set(cargo_rules))
            or set(cargo_rules) != LOCKFILE_CARGO_SUBCOMMANDS
        ):
            findings.append(
                Finding(
                    "invalid_policy",
                    policy_path,
                    "",
                    "cargo_locked_subcommands must list the complete supported lockfile-consuming command set exactly once",
                )
            )
        for key in BOOLEAN_RULES:
            if rules.get(key) is not True:
                findings.append(
                    Finding(
                        "invalid_policy",
                        policy_path,
                        "",
                        f"security rule {key!r} must be boolean true; use a reviewed exception for a narrow waiver",
                    )
                )

    rationales = policy.get("rationales")
    if not isinstance(rationales, dict):
        findings.append(
            Finding("invalid_policy", policy_path, "", "rationales must be an object")
        )
    elif set(rationales) != SUPPORTED_RULE_KEYS or any(
        not isinstance(value, str) or len(value.strip()) < 20 for value in rationales.values()
    ):
        findings.append(
            Finding(
                "invalid_policy",
                policy_path,
                "",
                "rationales must provide a substantive string for every supported rule",
            )
        )

    workflows = policy.get("workflows")
    if not isinstance(workflows, dict) or not workflows:
        findings.append(
            Finding("invalid_policy", policy_path, "", "workflows must be a non-empty object")
        )
    else:
        for workflow_path, workflow_policy in workflows.items():
            if not isinstance(workflow_path, str) or not workflow_path:
                findings.append(
                    Finding("invalid_policy", policy_path, "", "workflow paths must be non-empty strings")
                )
                continue
            if not isinstance(workflow_policy, dict) or set(workflow_policy) != {"jobs"}:
                findings.append(
                    Finding(
                        "invalid_policy",
                        policy_path,
                        "",
                        f"workflow {workflow_path!r} policy must contain only a jobs object",
                    )
                )
                continue
            jobs = workflow_policy.get("jobs")
            if not isinstance(jobs, dict) or not jobs:
                findings.append(
                    Finding(
                        "invalid_policy",
                        policy_path,
                        "",
                        f"workflow {workflow_path!r} jobs must be a non-empty object",
                    )
                )
                continue
            for job_name, job_policy in jobs.items():
                if not isinstance(job_name, str) or not job_name:
                    findings.append(
                        Finding("invalid_policy", policy_path, "", "job names must be non-empty strings")
                    )
                    continue
                if not isinstance(job_policy, dict) or set(job_policy) != SUPPORTED_JOB_POLICY_KEYS:
                    findings.append(
                        Finding(
                            "invalid_policy",
                            policy_path,
                            job_name,
                            f"job policy must contain exactly {sorted(SUPPORTED_JOB_POLICY_KEYS)!r}",
                        )
                    )
                    continue
                permissions = job_policy.get("permissions")
                if not isinstance(permissions, dict) or any(
                    not isinstance(key, str)
                    or not key
                    or value not in {"read", "write", "none"}
                    for key, value in permissions.items()
                ):
                    findings.append(
                        Finding(
                            "invalid_policy",
                            policy_path,
                            job_name,
                            "permissions must be a string-to-read/write/none object",
                        )
                    )
                elif permissions not in ({}, {"contents": "read"}):
                    findings.append(
                        Finding(
                            "invalid_policy",
                            policy_path,
                            job_name,
                            "AF-01 Stack A permits only no token permissions or contents: read",
                        )
                    )
                runner = job_policy.get("runner")
                if runner not in SUPPORTED_RUNNERS:
                    findings.append(
                        Finding(
                            "invalid_policy",
                            policy_path,
                            job_name,
                            f"runner must be one of {sorted(SUPPORTED_RUNNERS)!r}",
                        )
                    )
                timeout = job_policy.get("timeout_minutes")
                if (
                    type(timeout) is not int
                    or timeout <= 0
                    or timeout > MAX_JOB_TIMEOUT_MINUTES
                ):
                    findings.append(
                        Finding(
                            "invalid_policy",
                            policy_path,
                            job_name,
                            f"timeout_minutes must be 1..{MAX_JOB_TIMEOUT_MINUTES}",
                        )
                    )

    exceptions = policy.get("exceptions", [])
    if not isinstance(exceptions, list) or any(not _valid_exception(item) for item in exceptions):
        findings.append(
            Finding(
                "invalid_policy",
                policy_path,
                "",
                "every exception requires bounded rule/path/reason/revisit fields",
            )
        )
    return findings


def audit_repository(
    root: Path, policy: dict, tracked_files: Iterable[str] | None = None
) -> dict:
    findings = _policy_errors(policy)
    if findings:
        filtered = sorted(findings)
        return {
            "schema": 1,
            "ok": False,
            "workflows": [],
            "action_metadata": [],
            "findings": [finding.as_dict() for finding in filtered],
        }

    paths = list(tracked_files) if tracked_files is not None else _tracked_files(root)
    workflows, actions = discover_security_files(paths)
    expected_workflows = policy["workflows"]

    for path in sorted(set(workflows) - set(expected_workflows)):
        findings.append(
            Finding("unplanned_workflow", path, "", "tracked workflow is absent from policy")
        )
    for path in sorted(set(expected_workflows) - set(workflows)):
        findings.append(
            Finding("missing_workflow", path, "", "policy workflow is not tracked")
        )

    for path in sorted(set(workflows) & set(expected_workflows)):
        findings.extend(
            audit_workflow(
                path,
                (root / path).read_text(encoding="utf-8"),
                expected_workflows[path],
                policy,
            )
        )
    for path in actions:
        findings.extend(
            audit_action_metadata(path, (root / path).read_text(encoding="utf-8"), policy)
        )

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
    parser.add_argument(
        "--policy", type=Path, default=Path(".github/workflow-trust-policy.json")
    )
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