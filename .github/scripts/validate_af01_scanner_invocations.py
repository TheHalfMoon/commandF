#!/usr/bin/env python3
"""Validate exact AF-01 scanner action refs, inputs, and direct cargo-audit commands."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

CARGO_DENY_USES = "EmbarkStudios/cargo-deny-action@3c6349835b2b7b196a839186cb8b78e02f7b5f25"
CARGO_DENY_INPUTS = {
    "arguments": "--all-features",
    "command": "check",
    "command-arguments": "advisories bans licenses sources",
    "log-level": "warn",
}
ZIZMOR_USES = "zizmorcore/zizmor-action@3dc1ecc9bcb9e94e9b2c709687979e1298497054"
ZIZMOR_INPUTS = {
    "advanced-security": "false",
    "annotations": "false",
    "collect": "all",
    "color": "false",
    "fail-on-no-inputs": "true",
    "inputs": ".",
    "min-severity": "medium",
    "online-audits": "false",
    "persona": "regular",
    "version": "1.29.0",
}
CARGO_AUDIT_INSTALL = "cargo install cargo-audit --version 0.22.2 --locked"
CARGO_AUDIT_RUN = 'cargo audit --file Cargo.lock --json > "$AF01_EVIDENCE_DIR/cargo-audit.json"'
FULL_SHA = re.compile(r"^[0-9a-f]{40}$")


class ScannerContractError(ValueError):
    """Raised when the workflow scanner invocation no longer matches the assurance contract."""


def _indent(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _scalar(value: str) -> str:
    value = value.strip()
    if " #" in value:
        value = value.split(" #", 1)[0].rstrip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        value = value[1:-1]
    return value


def _step_bounds(lines: list[str], uses_index: int) -> tuple[int, int, int]:
    uses_indent = _indent(lines[uses_index])
    step_indent = uses_indent - 2
    start = uses_index
    for index in range(uses_index, -1, -1):
        line = lines[index]
        if _indent(line) == step_indent and line.lstrip().startswith("- "):
            start = index
            break
    else:
        raise ScannerContractError("scanner uses entry is not inside a static workflow step")

    end = len(lines)
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if line.strip() and _indent(line) < step_indent:
            end = index
            break
        if _indent(line) == step_indent and line.lstrip().startswith("- "):
            end = index
            break
    return start, end, step_indent


def _step_inputs(lines: list[str], uses_index: int) -> dict[str, str]:
    start, end, step_indent = _step_bounds(lines, uses_index)
    with_indexes = [
        index
        for index in range(start + 1, end)
        if _indent(lines[index]) == step_indent + 2 and lines[index].strip() == "with:"
    ]
    if len(with_indexes) != 1:
        raise ScannerContractError("scanner step must contain exactly one static with mapping")

    result: dict[str, str] = {}
    with_index = with_indexes[0]
    for index in range(with_index + 1, end):
        line = lines[index]
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = _indent(line)
        if indent <= step_indent + 2:
            break
        if indent != step_indent + 4 or ":" not in line:
            raise ScannerContractError("scanner with mapping contains unsupported nested syntax")
        key, raw = line.strip().split(":", 1)
        if key in result:
            raise ScannerContractError(f"scanner with mapping contains duplicate input {key!r}")
        value = _scalar(raw)
        if not value:
            raise ScannerContractError(f"scanner input {key!r} is empty or dynamic")
        if "${{" in value:
            raise ScannerContractError(f"scanner input {key!r} is dynamic")
        result[key] = value
    return result


def _find_action(lines: list[str], repository: str) -> tuple[str, dict[str, str]]:
    prefix = f"{repository}@"
    matches: list[tuple[int, str]] = []
    for index, line in enumerate(lines):
        stripped = line.strip()
        if not stripped.startswith("uses:"):
            continue
        value = _scalar(stripped.split(":", 1)[1])
        if value.startswith(prefix):
            matches.append((index, value))
    if len(matches) != 1:
        raise ScannerContractError(
            f"expected exactly one {repository} scanner step, found {len(matches)}"
        )
    index, uses = matches[0]
    revision = uses.rsplit("@", 1)[1]
    if FULL_SHA.fullmatch(revision) is None:
        raise ScannerContractError(f"{repository} scanner is not pinned to a full commit SHA")
    return uses, _step_inputs(lines, index)


def _require_exact_shell_line(lines: list[str], command: str, label: str) -> None:
    accepted = {command, f"run: {command}"}
    count = sum(1 for line in lines if line.strip() in accepted)
    if count != 1:
        raise ScannerContractError(f"{label} exact command must appear once, found {count}")


def validate_workflow(path: Path) -> dict[str, Any]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as error:
        raise ScannerContractError(f"cannot read assurance workflow: {error}") from error

    cargo_deny_uses, cargo_deny_inputs = _find_action(lines, "EmbarkStudios/cargo-deny-action")
    zizmor_uses, zizmor_inputs = _find_action(lines, "zizmorcore/zizmor-action")

    if cargo_deny_uses != CARGO_DENY_USES:
        raise ScannerContractError("cargo-deny executed action commit does not match assurance identity")
    if cargo_deny_inputs != CARGO_DENY_INPUTS:
        raise ScannerContractError("cargo-deny executed inputs do not match assurance policy")
    if zizmor_uses != ZIZMOR_USES:
        raise ScannerContractError("zizmor executed action commit does not match assurance identity")
    if zizmor_inputs != ZIZMOR_INPUTS:
        raise ScannerContractError("zizmor executed inputs do not match assurance policy")

    _require_exact_shell_line(lines, CARGO_AUDIT_INSTALL, "cargo-audit install")
    _require_exact_shell_line(lines, CARGO_AUDIT_RUN, "cargo-audit execution")

    contract = {
        "cargo_audit": {
            "install": CARGO_AUDIT_INSTALL,
            "run": CARGO_AUDIT_RUN,
            "version": "0.22.2",
        },
        "cargo_deny": {
            "inputs": cargo_deny_inputs,
            "uses": cargo_deny_uses,
        },
        "schema": 1,
        "zizmor": {
            "inputs": zizmor_inputs,
            "uses": zizmor_uses,
        },
    }
    rendered = json.dumps(contract, sort_keys=True, separators=(",", ":")).encode("utf-8")
    contract["sha256"] = hashlib.sha256(rendered).hexdigest()
    return contract


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workflow", type=Path)
    args = parser.parse_args()
    try:
        result = validate_workflow(args.workflow)
    except ScannerContractError as error:
        print(json.dumps({"error": str(error), "ok": False}, sort_keys=True))
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main())
