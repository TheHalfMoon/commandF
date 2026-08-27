#!/usr/bin/env python3
"""Complement AF-01 trust auditing for executable shell surfaces.

The primary workflow audit intentionally uses a constrained parser. This companion gate closes
shell-authority boundaries that require source-aware handling: shell-interpreter heredocs,
composite Action run steps, and statically referenced local Action shell scripts.
"""

from __future__ import annotations

import json
import re
import shlex
import sys
from pathlib import Path
from typing import Iterable

import audit_workflow_trust as core

SHELL_HEREDOC_RE = re.compile(
    r"(?:^|\s)(?:bash|dash|ksh|sh|zsh)\b[^\n;]*(?:<<-?\s*[\"']?[A-Za-z_][A-Za-z0-9_]*[\"']?)"
)
ACTION_LOCAL_SCRIPT_RE = re.compile(
    r"\$\{?GITHUB_ACTION_PATH\}?/(?P<path>[A-Za-z0-9_./-]+)"
)
VARIABLE_COMMAND_RE = re.compile(
    r"^[\"']?\$(?:([A-Za-z_][A-Za-z0-9_]*)|\{([A-Za-z_][A-Za-z0-9_]*)\})[\"']?$"
)
ASSIGNMENT_RE = re.compile(r"^(?P<name>[A-Za-z_][A-Za-z0-9_]*)=(?P<value>.*)$")


def _finding(code: str, path: str, scope: str, detail: str) -> core.Finding:
    return core.Finding(code, path, scope, detail)


def _scripts(lines: list[str]) -> list[str]:
    return core._run_scripts(lines, -1, len(lines))


def _shell_heredoc_findings(path: str, scope: str, script: str) -> list[core.Finding]:
    findings: list[core.Finding] = []
    for line in script.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if SHELL_HEREDOC_RE.search(stripped):
            findings.append(
                _finding(
                    "unsupported_shell_heredoc",
                    path,
                    scope,
                    f"shell-interpreter heredoc is executable authority and is not supported: {stripped}",
                )
            )
    return findings


def _direct_cargo_findings(
    path: str, scope: str, script: str, locked_subcommands: set[str]
) -> list[core.Finding]:
    """Audit direct/obviously indirect Cargo in a shell source without treating text as authority."""
    findings = _shell_heredoc_findings(path, scope, script)
    cargo_variables: set[str] = set()

    for segment in core._logical_shell_segments(script):
        try:
            tokens = shlex.split(segment, comments=True, posix=True)
        except ValueError as error:
            if "cargo" in segment:
                findings.append(
                    _finding(
                        "unsupported_shell_syntax",
                        path,
                        scope,
                        f"cannot safely parse Cargo-containing shell segment: {error}: {segment}",
                    )
                )
            continue
        if not tokens:
            continue

        if len(tokens) == 1:
            assignment = ASSIGNMENT_RE.fullmatch(tokens[0])
            if assignment:
                value = assignment.group("value").strip("\"'")
                if value == "cargo" or value.rsplit("/", 1)[-1] == "cargo":
                    cargo_variables.add(assignment.group("name"))
                continue

        command_index = core._command_token_index(tokens)
        if command_index is None or command_index >= len(tokens):
            continue
        command = tokens[command_index]

        variable = VARIABLE_COMMAND_RE.fullmatch(command)
        if variable:
            name = variable.group(1) or variable.group(2)
            if name in cargo_variables:
                findings.append(
                    _finding(
                        "unsupported_cargo_indirect",
                        path,
                        scope,
                        f"variable-expanded executable resolves to Cargo: {segment}",
                    )
                )
            continue

        if command.startswith("$(") or command.startswith("`"):
            if "cargo" in segment:
                findings.append(
                    _finding(
                        "unsupported_cargo_indirect",
                        path,
                        scope,
                        f"command-substituted executable can resolve to Cargo: {segment}",
                    )
                )
            continue

        if command in core.DYNAMIC_COMMAND_BUILTINS:
            if "cargo" in segment:
                findings.append(
                    _finding(
                        "unsupported_cargo_indirect",
                        path,
                        scope,
                        f"dynamic shell execution can hide Cargo: {segment}",
                    )
                )
            continue
        if command in core.SHELL_INTERPRETERS and "-c" in tokens[command_index + 1 :]:
            if "cargo" in segment:
                findings.append(
                    _finding(
                        "unsupported_cargo_indirect",
                        path,
                        scope,
                        f"nested shell execution can hide Cargo: {segment}",
                    )
                )
            continue

        if not (command == "cargo" or command.rsplit("/", 1)[-1] == "cargo"):
            continue

        subcommand_index = command_index + 1
        if subcommand_index < len(tokens) and tokens[subcommand_index].startswith("+"):
            subcommand_index += 1
        if subcommand_index >= len(tokens):
            findings.append(
                _finding(
                    "unsupported_cargo_syntax",
                    path,
                    scope,
                    f"cannot identify Cargo subcommand: {segment}",
                )
            )
            continue
        subcommand = tokens[subcommand_index]
        if subcommand in core.CARGO_INFO_FLAGS and subcommand_index == len(tokens) - 1:
            continue
        if subcommand.startswith("-"):
            # `command -v cargo` is normalized by the primary scanner and is not a Cargo invocation.
            continue
        if subcommand in locked_subcommands and "--locked" not in tokens[command_index:]:
            findings.append(
                _finding(
                    "cargo_unlocked",
                    path,
                    scope,
                    f"cargo {subcommand} invocation omits --locked: {' '.join(tokens[command_index:])}",
                )
            )
    return findings


def _action_local_targets(script: str) -> tuple[list[str], list[str]]:
    """Return statically exposed GITHUB_ACTION_PATH shell targets and unsupported dynamic targets."""
    targets: list[str] = []
    unsupported: list[str] = []
    for segment in core._logical_shell_segments(script):
        try:
            tokens = shlex.split(segment, comments=True, posix=True)
        except ValueError:
            continue
        command_index = core._command_token_index(tokens)
        if command_index is None or command_index >= len(tokens):
            continue
        if tokens[command_index] not in core.SHELL_INTERPRETERS:
            continue
        args = tokens[command_index + 1 :]
        if "-c" in args or "<<" in segment:
            continue
        script_arg = next((arg for arg in args if not arg.startswith("-")), None)
        if script_arg is None:
            unsupported.append(segment)
            continue
        matched = ACTION_LOCAL_SCRIPT_RE.search(script_arg)
        if matched:
            relative = matched.group("path")
            if relative.startswith("/") or ".." in Path(relative).parts:
                unsupported.append(segment)
            else:
                targets.append(relative)
            continue
        if "$" in script_arg or "`" in script_arg or script_arg.startswith("$("):
            unsupported.append(segment)
    return sorted(set(targets)), sorted(set(unsupported))


def audit_action_text(
    path: str,
    text: str,
    locked_subcommands: set[str],
) -> list[core.Finding]:
    lines = text.splitlines()
    findings: list[core.Finding] = []
    for script in _scripts(lines):
        findings.extend(_direct_cargo_findings(path, "composite-action", script, locked_subcommands))
        _, unsupported = _action_local_targets(script)
        for segment in unsupported:
            findings.append(
                _finding(
                    "unsupported_action_script",
                    path,
                    "composite-action",
                    f"Action shell source is dynamic or not statically exposed: {segment}",
                )
            )
    return findings


def _audit_local_action_script(
    root: Path,
    action_path: str,
    relative: str,
    tracked: set[str],
    locked_subcommands: set[str],
) -> list[core.Finding]:
    action_dir = Path(action_path).parent
    target = (action_dir / relative).as_posix()
    if target.startswith("./"):
        target = target[2:]
    if target not in tracked:
        return [
            _finding(
                "untracked_action_script",
                action_path,
                "composite-action",
                f"statically referenced Action shell source is not tracked: {target}",
            )
        ]
    try:
        source = (root / target).read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        return [
            _finding(
                "unreadable_action_script",
                action_path,
                "composite-action",
                f"cannot read tracked Action shell source {target}: {error}",
            )
        ]
    return _direct_cargo_findings(target, "action-script", source, locked_subcommands)


def audit_repository_surface(
    root: Path,
    policy: dict,
    tracked_files: Iterable[str] | None = None,
) -> dict:
    paths = list(tracked_files) if tracked_files is not None else core._tracked_files(root)
    tracked = set(paths)
    workflows, actions = core.discover_security_files(paths)
    rules = policy.get("rules", {}) if isinstance(policy, dict) else {}
    raw_subcommands = rules.get("cargo_locked_subcommands", []) if isinstance(rules, dict) else []
    locked_subcommands = set(raw_subcommands) if isinstance(raw_subcommands, list) else set()
    findings: list[core.Finding] = []

    for workflow_path in workflows:
        text = (root / workflow_path).read_text(encoding="utf-8")
        lines = text.splitlines()
        jobs, _ = core._job_ranges(lines)
        for job, (start, end) in sorted(jobs.items()):
            for script in core._run_scripts(lines, start, end):
                findings.extend(_shell_heredoc_findings(workflow_path, job, script))

    for action_path in actions:
        text = (root / action_path).read_text(encoding="utf-8")
        findings.extend(audit_action_text(action_path, text, locked_subcommands))
        for script in _scripts(text.splitlines()):
            targets, _ = _action_local_targets(script)
            for relative in targets:
                findings.extend(
                    _audit_local_action_script(
                        root,
                        action_path,
                        relative,
                        tracked,
                        locked_subcommands,
                    )
                )

    ordered = sorted(set(findings))
    return {
        "schema": 1,
        "ok": not ordered,
        "findings": [finding.as_dict() for finding in ordered],
    }


def main() -> int:
    root = Path(".").resolve()
    policy_path = root / ".github/workflow-trust-policy.json"
    try:
        policy = json.loads(policy_path.read_text(encoding="utf-8"))
        result = audit_repository_surface(root, policy)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        result = {
            "schema": 1,
            "ok": False,
            "findings": [
                {
                    "code": "surface_audit_operational_failure",
                    "path": str(policy_path.relative_to(root)),
                    "detail": str(error),
                }
            ],
        }
    sys.stdout.write(json.dumps(result, indent=2, sort_keys=True) + "\n")
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
