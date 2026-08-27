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
STATIC_KEY_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_-]*$")
AUDIT_SHELL_SEQUENCE = (
    "set -euo pipefail",
    "set +e",
    CARGO_AUDIT_RUN,
    "audit_status=$?",
    "set -e",
    'python3 - "$audit_status" <<\'PY\'',
    'test "$audit_status" -eq 0',
)


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


def _step_bounds(lines: list[str], anchor_index: int) -> tuple[int, int, int]:
    anchor_indent = _indent(lines[anchor_index])
    start = -1
    step_indent = -1
    for index in range(anchor_index, -1, -1):
        line = lines[index]
        indent = _indent(line)
        if line.strip() and indent < anchor_indent and line.lstrip().startswith("- "):
            start = index
            step_indent = indent
            break
        if index == anchor_index and line.lstrip().startswith("- "):
            start = index
            step_indent = indent
            break
    if start < 0:
        raise ScannerContractError("scanner command is not inside a static workflow step")

    end = len(lines)
    for index in range(start + 1, len(lines)):
        line = lines[index]
        if not line.strip():
            continue
        indent = _indent(line)
        if indent < step_indent:
            end = index
            break
        if indent == step_indent and line.lstrip().startswith("- "):
            end = index
            break
    return start, end, step_indent


def _step_fields(
    lines: list[str], start: int, end: int, step_indent: int, label: str
) -> dict[str, tuple[int, str]]:
    fields: dict[str, tuple[int, str]] = {}
    for index in range(start, end):
        line = lines[index]
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = _indent(line)
        if index == start:
            raw = line.strip()
            if not raw.startswith("- "):
                raise ScannerContractError(f"{label} step does not start with a static list item")
            raw = raw[2:]
        elif indent == step_indent + 2:
            raw = line.strip()
        else:
            continue
        if ":" not in raw:
            raise ScannerContractError(f"{label} step contains unsupported top-level syntax: {raw!r}")
        key, value = raw.split(":", 1)
        if not STATIC_KEY_RE.fullmatch(key):
            raise ScannerContractError(
                f"{label} step contains quoted or unsupported top-level key {key!r}"
            )
        if key in fields:
            raise ScannerContractError(f"{label} step contains duplicate top-level key {key!r}")
        fields[key] = (index, _scalar(value))
    return fields


def _require_step_shape(
    lines: list[str],
    anchor_index: int,
    *,
    label: str,
    required_fields: set[str],
) -> tuple[int, int, int, dict[str, tuple[int, str]]]:
    start, end, step_indent = _step_bounds(lines, anchor_index)
    fields = _step_fields(lines, start, end, step_indent, label)
    if set(fields) != required_fields:
        extra = sorted(set(fields) - required_fields)
        missing = sorted(required_fields - set(fields))
        raise ScannerContractError(
            f"{label} step shape mismatch: missing={missing} unsupported={extra}"
        )
    return start, end, step_indent, fields


def _step_inputs(
    lines: list[str], start: int, end: int, step_indent: int, with_index: int, label: str
) -> dict[str, str]:
    result: dict[str, str] = {}
    for index in range(with_index + 1, end):
        line = lines[index]
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = _indent(line)
        if indent <= step_indent + 2:
            break
        if indent != step_indent + 4 or ":" not in line:
            raise ScannerContractError(f"{label} with mapping contains unsupported nested syntax")
        key, raw = line.strip().split(":", 1)
        if not STATIC_KEY_RE.fullmatch(key):
            raise ScannerContractError(f"{label} input key is quoted or unsupported: {key!r}")
        if key in result:
            raise ScannerContractError(f"{label} with mapping contains duplicate input {key!r}")
        value = _scalar(raw)
        if not value or "${{" in value:
            raise ScannerContractError(f"{label} input {key!r} is empty or dynamic")
        result[key] = value
    return result


def _find_action(lines: list[str], repository: str, label: str) -> tuple[str, dict[str, str]]:
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
    uses_index, uses = matches[0]
    revision = uses.rsplit("@", 1)[1]
    if FULL_SHA.fullmatch(revision) is None:
        raise ScannerContractError(f"{repository} scanner is not pinned to a full commit SHA")

    start, end, step_indent, fields = _require_step_shape(
        lines,
        uses_index,
        label=label,
        required_fields={"name", "uses", "with"},
    )
    if fields["uses"][0] != uses_index or fields["uses"][1] != uses:
        raise ScannerContractError(f"{label} uses entry is not the exact top-level step field")
    with_index, with_value = fields["with"]
    if with_value:
        raise ScannerContractError(f"{label} with field must be a block mapping")
    return uses, _step_inputs(lines, start, end, step_indent, with_index, label)


def _without_heredoc_bodies(lines: list[str]) -> list[str]:
    kept: list[str] = []
    delimiter: str | None = None
    for line in lines:
        stripped = line.strip()
        if delimiter is not None:
            if stripped == delimiter:
                delimiter = None
            continue
        kept.append(stripped)
        matched = re.search(r"<<['\"]?([A-Za-z_][A-Za-z0-9_]*)['\"]?", stripped)
        if matched is not None:
            delimiter = matched.group(1)
    if delimiter is not None:
        raise ScannerContractError("cargo-audit run step contains an unterminated heredoc")
    return [line for line in kept if line]


def _find_run_step(
    lines: list[str],
    command: str,
    *,
    label: str,
    block: bool,
) -> tuple[int, int, int, dict[str, tuple[int, str]]]:
    anchors = [
        index
        for index, line in enumerate(lines)
        if line.strip() in {command, f"run: {command}"}
    ]
    if len(anchors) != 1:
        raise ScannerContractError(f"{label} exact command must appear once, found {len(anchors)}")
    anchor = anchors[0]
    start, end, step_indent, fields = _require_step_shape(
        lines,
        anchor,
        label=label,
        required_fields={"name", "run"},
    )
    run_index, run_value = fields["run"]
    if block:
        if run_value not in {"|", "|-", "|+", ">", ">-", ">+"}:
            raise ScannerContractError(f"{label} must use a static block run script")
        body: list[str] = []
        for index in range(run_index + 1, end):
            line = lines[index]
            if not line.strip():
                body.append("")
                continue
            if _indent(line) <= step_indent + 2:
                break
            body.append(line[step_indent + 4 :])
        shell_lines = tuple(_without_heredoc_bodies(body))
        if shell_lines != AUDIT_SHELL_SEQUENCE:
            raise ScannerContractError(
                f"{label} run script does not match the exact fail-closed audit sequence"
            )
    elif run_value != command:
        raise ScannerContractError(f"{label} must be the exact inline run command")
    return start, end, step_indent, fields


def validate_workflow(path: Path) -> dict[str, Any]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as error:
        raise ScannerContractError(f"cannot read assurance workflow: {error}") from error

    cargo_deny_uses, cargo_deny_inputs = _find_action(
        lines, "EmbarkStudios/cargo-deny-action", "cargo-deny"
    )
    zizmor_uses, zizmor_inputs = _find_action(lines, "zizmorcore/zizmor-action", "zizmor")

    if cargo_deny_uses != CARGO_DENY_USES:
        raise ScannerContractError("cargo-deny executed action commit does not match assurance identity")
    if cargo_deny_inputs != CARGO_DENY_INPUTS:
        raise ScannerContractError("cargo-deny executed inputs do not match assurance policy")
    if zizmor_uses != ZIZMOR_USES:
        raise ScannerContractError("zizmor executed action commit does not match assurance identity")
    if zizmor_inputs != ZIZMOR_INPUTS:
        raise ScannerContractError("zizmor executed inputs do not match assurance policy")

    _find_run_step(
        lines,
        CARGO_AUDIT_INSTALL,
        label="cargo-audit install",
        block=False,
    )
    _find_run_step(
        lines,
        CARGO_AUDIT_RUN,
        label="cargo-audit execution",
        block=True,
    )

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
