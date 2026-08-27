#!/usr/bin/env python3
"""Fail closed on environment authority that can alter shell execution across steps."""

from __future__ import annotations

import json
import re
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Iterable

import audit_workflow_trust as core

CHANNEL_NAME_RE = re.compile(
    r"(?<![A-Za-z0-9_])(?P<channel>GITHUB_PATH|GITHUB_ENV)(?![A-Za-z0-9_])"
)
SHELL_STARTUP_NAME_RE = re.compile(
    r"(?<![A-Za-z0-9_])(?P<startup>BASH_ENV|ENV|ZDOTDIR)(?![A-Za-z0-9_])"
)
INDIRECT_PARAMETER_RE = re.compile(r"\$\{!")
GITHUB_PREFIX_FRAGMENT_RE = re.compile(r"(?<![A-Za-z0-9_])GITHUB_(?![A-Za-z0-9_])")
SHELL_STARTUP_FRAGMENT_RE = re.compile(
    r"(?<![A-Za-z0-9_])(?:BASH_|ZDOT)(?![A-Za-z0-9_])"
)
SHELL_SHEBANG_RE = re.compile(r"^#![^\n]*\b(?:bash|dash|ksh|sh|zsh)\b")
ASSIGNMENT_BUILTINS = frozenset({"declare", "export", "local", "readonly", "typeset"})
VARIABLE_TARGET_BUILTINS = frozenset({"read", "mapfile", "readarray", "unset"})
COMMAND_BUILTIN_WRAPPERS = frozenset({"builtin", "command"})


def _tracked_files(root: Path) -> list[str]:
    completed = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z"],
        check=False,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise RuntimeError("unable to enumerate tracked repository files")
    return sorted(
        item.decode("utf-8")
        for item in completed.stdout.split(b"\0")
        if item
    )


def _is_yaml_authority(path: str) -> bool:
    candidate = Path(path)
    return (
        path.startswith(".github/workflows/") and candidate.suffix in {".yml", ".yaml"}
    ) or candidate.name in {"action.yml", "action.yaml"}


def _read_authority_text(root: Path, path: str) -> tuple[str | None, str | None]:
    candidate = root / path
    try:
        raw = candidate.read_bytes()
    except OSError as error:
        if _is_yaml_authority(path) or candidate.suffix == ".sh":
            return None, f"unable to read tracked authority file: {error}"
        return None, None

    is_shell = candidate.suffix == ".sh"
    if not is_shell:
        prefix = raw[:256].decode("utf-8", errors="ignore")
        is_shell = SHELL_SHEBANG_RE.match(prefix) is not None
    if not (_is_yaml_authority(path) or is_shell):
        return None, None

    try:
        return raw.decode("utf-8"), None
    except UnicodeDecodeError:
        return None, "tracked authority file is not valid UTF-8"


def _dynamic_name(value: str) -> bool:
    return "$" in value or "`" in value


def _basename(token: str) -> str:
    return token.rsplit("/", 1)[-1]


def _raw_command_index(tokens: list[str]) -> int | None:
    index = 0
    while index < len(tokens) and core.SHELL_ASSIGNMENT_RE.fullmatch(tokens[index]):
        index += 1
    while index < len(tokens) and tokens[index] in core.SHELL_CONTROL_WORDS:
        index += 1
    return index if index < len(tokens) else None


def _normalized_writer(tokens: list[str]) -> tuple[str, list[str]] | None:
    index = _raw_command_index(tokens)
    if index is None:
        return None
    command = _basename(tokens[index])
    index += 1
    if command in COMMAND_BUILTIN_WRAPPERS and index < len(tokens):
        while index < len(tokens) and tokens[index].startswith("-"):
            index += 1
        if index >= len(tokens):
            return None
        command = _basename(tokens[index])
        index += 1
    return command, tokens[index:]


def _before_redirection(args: list[str]) -> list[str]:
    result: list[str] = []
    for token in args:
        if token in {"<", ">", ">>", "<<", "<<<", "<>", ">&", "<&"}:
            break
        if re.match(r"^(?:\d*)?(?:>>?|<<?|<>|>&|<&)", token):
            break
        result.append(token)
    return result


def _dynamic_writer_detail(tokens: list[str]) -> str | None:
    normalized = _normalized_writer(tokens)
    if normalized is None:
        return None
    command, args = normalized

    if command in ASSIGNMENT_BUILTINS:
        for arg in args:
            if arg.startswith("-"):
                continue
            name = arg.split("=", 1)[0]
            if _dynamic_name(name):
                return f"{command} writes a dynamically constructed variable name: {arg}"
        return None

    if command == "printf":
        for index, arg in enumerate(args):
            if arg == "-v" and index + 1 < len(args) and _dynamic_name(args[index + 1]):
                return f"printf -v writes a dynamically constructed variable name: {args[index + 1]}"
        return None

    if command in VARIABLE_TARGET_BUILTINS:
        candidates = _before_redirection(args)
        for arg in candidates:
            if arg.startswith("-"):
                continue
            if _dynamic_name(arg):
                return f"{command} writes a dynamically constructed variable target: {arg}"
        return None

    if command == "getopts":
        positional = [arg for arg in args if not arg.startswith("-")]
        if len(positional) >= 2 and _dynamic_name(positional[1]):
            return f"getopts writes a dynamically constructed variable target: {positional[1]}"
        return None

    if command == "env":
        for arg in args:
            if arg.startswith("-"):
                continue
            if "=" in arg:
                name = arg.split("=", 1)[0]
                if _dynamic_name(name):
                    return f"env constructs a dynamic environment variable name: {arg}"
                continue
            if _dynamic_name(arg):
                return f"env has an unresolved dynamic environment/command operand: {arg}"
            break
    return None


def _shell_scripts(path: str, text: str) -> list[str]:
    if _is_yaml_authority(path):
        lines = text.splitlines()
        return core._run_scripts(lines, -1, len(lines))
    return [text]


def _dynamic_variable_write_findings(path: str, text: str) -> list[dict[str, str]]:
    findings: list[dict[str, str]] = []
    for script in _shell_scripts(path, text):
        for segment in core._logical_shell_segments(script):
            try:
                tokens = shlex.split(segment, comments=True, posix=True)
            except ValueError as error:
                if "$" in segment and any(
                    name in segment
                    for name in (
                        "export",
                        "declare",
                        "local",
                        "readonly",
                        "typeset",
                        "printf",
                        "read",
                        "mapfile",
                        "readarray",
                        "getopts",
                        "unset",
                        "env",
                    )
                ):
                    findings.append(
                        {
                            "channel": "",
                            "code": "unsupported_dynamic_variable_write",
                            "detail": f"cannot safely parse dynamic variable-writing shell segment: {error}: {segment}",
                            "path": path,
                        }
                    )
                continue
            if not tokens:
                continue
            detail = _dynamic_writer_detail(tokens)
            if detail is not None:
                findings.append(
                    {
                        "channel": "",
                        "code": "unsupported_dynamic_variable_write",
                        "detail": detail,
                        "path": path,
                    }
                )
    return findings


def _authority_findings(path: str, text: str) -> list[dict[str, str]]:
    findings: list[dict[str, str]] = []
    for matched in CHANNEL_NAME_RE.finditer(text):
        channel = matched.group("channel")
        findings.append(
            {
                "channel": channel,
                "code": "unsupported_github_environment_channel",
                "detail": (
                    f"{channel} can mutate later-step environment/command resolution and "
                    "is outside the AF-01 constrained shell authority"
                ),
                "path": path,
            }
        )
    for matched in SHELL_STARTUP_NAME_RE.finditer(text):
        startup = matched.group("startup")
        findings.append(
            {
                "channel": startup,
                "code": "unsupported_shell_startup_environment",
                "detail": (
                    f"{startup} can change shell startup-file authority before visible run commands "
                    "and is outside the AF-01 constrained shell authority"
                ),
                "path": path,
            }
        )
    if INDIRECT_PARAMETER_RE.search(text) is not None:
        findings.append(
            {
                "channel": "",
                "code": "unsupported_indirect_parameter_expansion",
                "detail": (
                    "indirect shell parameter expansion can resolve forbidden environment authority "
                    "and is outside AF-01 authority"
                ),
                "path": path,
            }
        )
    if GITHUB_PREFIX_FRAGMENT_RE.search(text) is not None:
        findings.append(
            {
                "channel": "",
                "code": "unsupported_github_environment_name_fragment",
                "detail": (
                    "standalone GITHUB_ name fragments can construct a forbidden GitHub "
                    "environment channel and are outside AF-01 authority"
                ),
                "path": path,
            }
        )
    if SHELL_STARTUP_FRAGMENT_RE.search(text) is not None:
        findings.append(
            {
                "channel": "",
                "code": "unsupported_shell_startup_name_fragment",
                "detail": (
                    "shell-startup variable name fragments can construct hidden startup-file "
                    "authority and are outside AF-01 authority"
                ),
                "path": path,
            }
        )
    findings.extend(_dynamic_variable_write_findings(path, text))
    return findings


def audit_repository_environment_channels(
    root: Path, tracked_files: Iterable[str]
) -> dict[str, object]:
    """Reject cross-step and shell-startup environment authority in tracked shell surfaces."""
    findings: list[dict[str, str]] = []
    for path in sorted(set(tracked_files)):
        text, read_error = _read_authority_text(root, path)
        if read_error is not None:
            findings.append(
                {
                    "channel": "",
                    "code": "unreadable_environment_channel_authority",
                    "detail": read_error,
                    "path": path,
                }
            )
            continue
        if text is None:
            continue
        findings.extend(_authority_findings(path, text))
    findings.sort(key=lambda item: (item["path"], item["channel"], item["code"], item["detail"]))
    return {"findings": findings, "ok": not findings, "schema": 1}


def main() -> int:
    root = Path(__file__).resolve().parents[2]
    try:
        tracked = _tracked_files(root)
        result = audit_repository_environment_channels(root, tracked)
    except RuntimeError as error:
        result = {
            "findings": [
                {
                    "channel": "",
                    "code": "environment_channel_inventory_failed",
                    "detail": str(error),
                    "path": "",
                }
            ],
            "ok": False,
            "schema": 1,
        }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
