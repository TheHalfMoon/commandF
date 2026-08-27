#!/usr/bin/env python3
"""Complement AF-01 trust auditing for executable shell surfaces.

The primary workflow audit intentionally uses a constrained parser. This companion gate closes
shell-authority boundaries that require source-aware handling: shell-interpreter heredocs,
composite Action run steps, and recursively referenced tracked local Action shell scripts.
"""

from __future__ import annotations

import json
import re
import shlex
import sys
from pathlib import Path
from typing import Iterable

import audit_workflow_trust as core

HEREDOC_OPERATOR_RE = re.compile(
    r"<<-?\s*[\"']?[A-Za-z_][A-Za-z0-9_]*[\"']?"
)
REDIRECTION_TOKEN_RE = re.compile(r"^(?:\d*)?(?:>>?|<<?|<>|>&|<&).+$")
DOUBLE_BRACKET_RE = re.compile(r"\[\[(?:(?!\]\]).)*\]\]", re.DOTALL)
ACTION_LOCAL_SCRIPT_RE = re.compile(
    r"^\$(?:GITHUB_ACTION_PATH|\{GITHUB_ACTION_PATH\})/"
    r"(?P<path>[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*)$"
)
VARIABLE_COMMAND_RE = re.compile(
    r"^[\"']?\$(?:([A-Za-z_][A-Za-z0-9_]*)|\{([A-Za-z_][A-Za-z0-9_]*)\})[\"']?$"
)
ASSIGNMENT_RE = re.compile(r"^(?P<name>[A-Za-z_][A-Za-z0-9_]*)=(?P<value>.*)$")
ASSIGNMENT_BUILTINS = frozenset({"declare", "export", "local", "readonly", "typeset"})
ACTION_SOURCE_BUILTINS = frozenset({".", "source"})
EXECUTION_WRAPPERS = frozenset(
    {"command", "env", "exec", "nice", "nohup", "stdbuf", "sudo", "timeout"}
)


def _finding(code: str, path: str, scope: str, detail: str) -> core.Finding:
    return core.Finding(code, path, scope, detail)


def _scripts(lines: list[str]) -> list[str]:
    return core._run_scripts(lines, -1, len(lines))


def _basename(token: str) -> str:
    return token.rsplit("/", 1)[-1]


def _raw_executable_index(tokens: list[str]) -> int | None:
    index = 0
    while index < len(tokens) and core.SHELL_ASSIGNMENT_RE.fullmatch(tokens[index]):
        index += 1
    while index < len(tokens) and tokens[index] in core.SHELL_CONTROL_WORDS:
        index += 1
    return index if index < len(tokens) else None


def _wrapper_hides_cargo(tokens: list[str]) -> bool:
    """Detect Cargo behind an execution wrapper the core constrained normalizer did not resolve."""
    raw_index = _raw_executable_index(tokens)
    if raw_index is None or _basename(tokens[raw_index]) not in EXECUTION_WRAPPERS:
        return False
    return any(_basename(token) == "cargo" for token in tokens[raw_index + 1 :])


def _only_redirections(tokens: list[str]) -> bool:
    """Allow fixed shell redirections after a Cargo information flag."""
    return all(REDIRECTION_TOKEN_RE.fullmatch(token) is not None for token in tokens)


def _heredoc_prefix_executes_shell(prefix: str) -> bool:
    """Recognize fixed, absolute-path, and wrapped shell interpreters before a heredoc."""
    try:
        tokens = shlex.split(prefix, comments=True, posix=True)
    except ValueError:
        # A shell-looking executable prefix that cannot be parsed is executable authority and must
        # not silently become heredoc data.
        return bool(
            re.search(
                r"(?:^|\s|/)(?:bash|dash|ksh|sh|zsh)(?:\s|$)",
                prefix,
            )
        )
    if not tokens:
        return False

    command_index = core._command_token_index(tokens)
    if command_index is not None and command_index < len(tokens):
        command = tokens[command_index]
        if _basename(command) in core.SHELL_INTERPRETERS:
            return True
        # Unknown executable indirection can select a shell and the heredoc body would otherwise be
        # removed by the core Cargo scanner.
        if "$" in command or "`" in command:
            return True

    raw_index = _raw_executable_index(tokens)
    if raw_index is None:
        return False
    if _basename(tokens[raw_index]) not in EXECUTION_WRAPPERS:
        return False
    # Fail closed on wrapper forms the core constrained normalizer does not fully understand (for
    # example `env -u NAME bash` or an absolute `/usr/bin/env` wrapper).
    return any(_basename(token) in core.SHELL_INTERPRETERS for token in tokens[raw_index + 1 :])


def _shell_heredoc_findings(path: str, scope: str, script: str) -> list[core.Finding]:
    findings: list[core.Finding] = []
    for line in script.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        matched = HEREDOC_OPERATOR_RE.search(stripped)
        if matched is None:
            continue
        prefix = stripped[: matched.start()]
        if _heredoc_prefix_executes_shell(prefix):
            findings.append(
                _finding(
                    "unsupported_shell_heredoc",
                    path,
                    scope,
                    f"shell-interpreter heredoc is executable authority and is not supported: {stripped}",
                )
            )
    return findings


def _mask_double_bracket_tests(
    path: str, scope: str, script: str
) -> tuple[str, list[core.Finding]]:
    """Mask non-executable [[ expressions while rejecting dynamic command substitution inside them."""
    findings: list[core.Finding] = []

    def replace(matched: re.Match[str]) -> str:
        expression = matched.group(0)
        if "$(" in expression or "`" in expression:
            findings.append(
                _finding(
                    "unsupported_cargo_indirect",
                    path,
                    scope,
                    f"command substitution inside [[ ... ]] is not statically auditable: {expression}",
                )
            )
        return "[[ true ]]"

    return DOUBLE_BRACKET_RE.sub(replace, script), findings


def _assignment_class(value: str) -> str:
    """Classify an executable assignment as Cargo, statically non-Cargo, or unknown."""
    candidate = value.strip().strip("\"'")
    if not candidate:
        return "unknown"
    basename = candidate.rsplit("/", 1)[-1]
    if basename == "cargo":
        return "cargo"
    if "$(" in candidate or "`" in candidate:
        return "unknown"
    # A fixed final path component cannot turn into Cargo even when its parent path is expanded.
    if basename and "$" not in basename:
        return "non_cargo"
    return "unknown"


def _record_assignment_tokens(tokens: list[str], states: dict[str, str]) -> None:
    """Record simple shell assignments, including export/readonly/local/declare/typeset forms."""
    if not tokens:
        return
    candidates: list[str] = []
    if tokens[0] in ASSIGNMENT_BUILTINS:
        for token in tokens[1:]:
            if token.startswith("-"):
                continue
            candidates.append(token)
    else:
        for token in tokens:
            if ASSIGNMENT_RE.fullmatch(token):
                candidates.append(token)
            else:
                break
    for token in candidates:
        matched = ASSIGNMENT_RE.fullmatch(token)
        if matched:
            states[matched.group("name")] = _assignment_class(matched.group("value"))


def _direct_cargo_findings(
    path: str, scope: str, script: str, locked_subcommands: set[str]
) -> list[core.Finding]:
    """Audit direct Cargo and fail closed on executable indirection that may resolve to Cargo."""
    findings = _shell_heredoc_findings(path, scope, script)
    command_script, expression_findings = _mask_double_bracket_tests(path, scope, script)
    findings.extend(expression_findings)
    variable_states: dict[str, str] = {}

    for segment in core._logical_shell_segments(command_script):
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

        _record_assignment_tokens(tokens, variable_states)

        command_index = core._command_token_index(tokens)
        if command_index is None or command_index >= len(tokens):
            if _wrapper_hides_cargo(tokens):
                findings.append(
                    _finding(
                        "unsupported_cargo_indirect",
                        path,
                        scope,
                        f"execution wrapper hides Cargo from static command normalization: {segment}",
                    )
                )
            continue
        command = tokens[command_index]

        variable = VARIABLE_COMMAND_RE.fullmatch(command)
        if variable:
            name = variable.group(1) or variable.group(2)
            state = variable_states.get(name, "unknown")
            if state != "non_cargo":
                findings.append(
                    _finding(
                        "unsupported_cargo_indirect",
                        path,
                        scope,
                        f"variable-expanded executable is not proven non-Cargo ({name}={state}): {segment}",
                    )
                )
            continue

        if command.startswith("$(") or command.startswith("`"):
            findings.append(
                _finding(
                    "unsupported_cargo_indirect",
                    path,
                    scope,
                    f"command-substituted executable can resolve to Cargo: {segment}",
                )
            )
            continue

        if "$" in command or "`" in command:
            findings.append(
                _finding(
                    "unsupported_cargo_indirect",
                    path,
                    scope,
                    f"dynamic executable path can resolve to Cargo: {segment}",
                )
            )
            continue

        command_basename = _basename(command)
        if command_basename in core.DYNAMIC_COMMAND_BUILTINS:
            findings.append(
                _finding(
                    "unsupported_cargo_indirect",
                    path,
                    scope,
                    f"dynamic shell execution can hide Cargo: {segment}",
                )
            )
            continue
        if command_basename in core.SHELL_INTERPRETERS and "-c" in tokens[command_index + 1 :]:
            findings.append(
                _finding(
                    "unsupported_cargo_indirect",
                    path,
                    scope,
                    f"nested shell execution can hide Cargo: {segment}",
                )
            )
            continue

        is_direct_cargo = command == "cargo" or command_basename == "cargo"
        if not is_direct_cargo:
            if _wrapper_hides_cargo(tokens):
                findings.append(
                    _finding(
                        "unsupported_cargo_indirect",
                        path,
                        scope,
                        f"execution wrapper hides Cargo from static command normalization: {segment}",
                    )
                )
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
        if subcommand in core.CARGO_INFO_FLAGS:
            trailing = tokens[subcommand_index + 1 :]
            if not trailing or _only_redirections(trailing):
                continue
            findings.append(
                _finding(
                    "unsupported_cargo_syntax",
                    path,
                    scope,
                    f"Cargo information flag has unsupported trailing syntax: {segment}",
                )
            )
            continue
        if subcommand.startswith("-"):
            findings.append(
                _finding(
                    "unsupported_cargo_syntax",
                    path,
                    scope,
                    f"Cargo global-option syntax requires explicit auditor support: {segment}",
                )
            )
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


def _exact_action_local_target(token: str) -> str | None:
    matched = ACTION_LOCAL_SCRIPT_RE.fullmatch(token)
    if matched is None:
        return None
    relative = matched.group("path")
    if relative.startswith("/") or ".." in Path(relative).parts:
        return None
    return relative


def _action_local_targets(script: str) -> tuple[list[str], list[str]]:
    """Return exact static GITHUB_ACTION_PATH shell targets and unsupported delegation."""
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

        command = tokens[command_index]
        command_basename = _basename(command)
        args = tokens[command_index + 1 :]

        direct_target = _exact_action_local_target(command)
        if direct_target is not None:
            # Arguments can change delegated script behavior; keep the supported delegation shape
            # intentionally narrow and deterministic.
            if args:
                unsupported.append(segment)
            else:
                targets.append(direct_target)
            continue
        if "GITHUB_ACTION_PATH" in command:
            unsupported.append(segment)
            continue

        if command in ACTION_SOURCE_BUILTINS:
            if len(args) != 1:
                unsupported.append(segment)
                continue
            source_target = _exact_action_local_target(args[0])
            if source_target is None:
                unsupported.append(segment)
            else:
                targets.append(source_target)
            continue

        if command_basename not in core.SHELL_INTERPRETERS:
            continue
        if "-c" in args:
            unsupported.append(segment)
            continue
        if HEREDOC_OPERATOR_RE.search(segment):
            # Heredoc authority is rejected separately by _shell_heredoc_findings.
            continue
        script_positions = [index for index, arg in enumerate(args) if not arg.startswith("-")]
        if not script_positions:
            unsupported.append(segment)
            continue
        script_position = script_positions[0]
        script_arg = args[script_position]
        target = _exact_action_local_target(script_arg)
        if target is None:
            unsupported.append(segment)
            continue
        # Any remaining positional arguments can influence the delegated script and are not part of
        # the supported static delegation subset.
        if any(not arg.startswith("-") for arg in args[script_position + 1 :]):
            unsupported.append(segment)
            continue
        targets.append(target)
    return sorted(set(targets)), sorted(set(unsupported))


def _unsupported_action_script_findings(
    path: str, scope: str, scripts: Iterable[str]
) -> list[core.Finding]:
    findings: list[core.Finding] = []
    for script in scripts:
        _, unsupported = _action_local_targets(script)
        for segment in unsupported:
            findings.append(
                _finding(
                    "unsupported_action_script",
                    path,
                    scope,
                    f"Action shell source is dynamic or not statically repository-owned: {segment}",
                )
            )
    return findings


def audit_action_text(
    path: str,
    text: str,
    locked_subcommands: set[str],
) -> list[core.Finding]:
    lines = text.splitlines()
    scripts = _scripts(lines)
    findings: list[core.Finding] = []
    for script in scripts:
        findings.extend(_direct_cargo_findings(path, "composite-action", script, locked_subcommands))
    findings.extend(_unsupported_action_script_findings(path, "composite-action", scripts))
    return findings


def _action_target_path(action_path: str, relative: str) -> str | None:
    action_dir = Path(action_path).parent
    target_path = action_dir / relative
    if target_path.is_absolute() or ".." in target_path.parts:
        return None
    target = target_path.as_posix()
    return target[2:] if target.startswith("./") else target


def _audit_local_action_script(
    root: Path,
    action_path: str,
    relative: str,
    tracked: set[str],
    locked_subcommands: set[str],
    seen: set[str],
) -> list[core.Finding]:
    target = _action_target_path(action_path, relative)
    if target is None:
        return [
            _finding(
                "unsupported_action_script",
                action_path,
                "composite-action",
                f"Action shell source escapes the Action directory: {relative}",
            )
        ]
    if target in seen:
        return []
    seen.add(target)
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

    findings = _direct_cargo_findings(target, "action-script", source, locked_subcommands)
    targets, unsupported = _action_local_targets(source)
    for segment in unsupported:
        findings.append(
            _finding(
                "unsupported_action_script",
                target,
                "action-script",
                f"Action shell source delegates dynamically or outside GITHUB_ACTION_PATH: {segment}",
            )
        )
    for nested in targets:
        findings.extend(
            _audit_local_action_script(
                root,
                action_path,
                nested,
                tracked,
                locked_subcommands,
                seen,
            )
        )
    return findings


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
        action_scripts = _scripts(text.splitlines())
        findings.extend(audit_action_text(action_path, text, locked_subcommands))
        seen: set[str] = set()
        for script in action_scripts:
            targets, _ = _action_local_targets(script)
            for relative in targets:
                findings.extend(
                    _audit_local_action_script(
                        root,
                        action_path,
                        relative,
                        tracked,
                        locked_subcommands,
                        seen,
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
