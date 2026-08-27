#!/usr/bin/env python3
"""Summarize exact Cargo metadata for AF-01 T020 dependency-policy inspection."""

from __future__ import annotations

import hashlib
import json
import sys
from collections import defaultdict
from typing import Any

CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


class InventoryError(ValueError):
    """Raised when Cargo metadata cannot support a complete exact inventory."""


def require_string(value: object, message: str) -> str:
    if not isinstance(value, str) or not value:
        raise InventoryError(message)
    return value


def validate_manifest_dependencies(package: dict[str, Any], identity: str) -> None:
    dependencies = package.get("dependencies")
    if not isinstance(dependencies, list):
        raise InventoryError(f"manifest dependencies for {identity} are not an array")
    for index, dependency in enumerate(dependencies):
        if not isinstance(dependency, dict):
            raise InventoryError(
                f"manifest dependency {index} for {identity} is not an object"
            )
        require_string(
            dependency.get("name"),
            f"manifest dependency {index} for {identity} has invalid name",
        )


def graph_sha256(packages: list[dict[str, object]]) -> str:
    rendered = json.dumps(
        packages,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")
    return hashlib.sha256(rendered).hexdigest()


def summarize(metadata: object) -> dict[str, object]:
    if not isinstance(metadata, dict):
        raise InventoryError("cargo metadata root is not an object")

    packages = metadata.get("packages")
    workspace_members = metadata.get("workspace_members")
    resolve = metadata.get("resolve")
    if not isinstance(packages, list) or not isinstance(workspace_members, list):
        raise InventoryError("cargo metadata missing package/workspace arrays")
    if not isinstance(resolve, dict) or not isinstance(resolve.get("nodes"), list):
        raise InventoryError("cargo metadata missing resolved dependency graph")

    workspace_ids: set[str] = set()
    for index, member in enumerate(workspace_members):
        workspace_ids.add(
            require_string(member, f"workspace member {index} is not a package id")
        )

    package_records: dict[str, dict[str, object]] = {}
    versions_by_name: dict[str, set[str]] = defaultdict(set)
    license_packages: dict[str, list[str]] = defaultdict(list)
    source_classes: dict[str, int] = defaultdict(int)
    unknown_license: list[str] = []
    non_crates_io: list[str] = []

    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            raise InventoryError(f"cargo metadata package {index} is not an object")

        package_id = require_string(package.get("id"), "package identity is incomplete")
        name = require_string(package.get("name"), "package identity is incomplete")
        version = require_string(package.get("version"), "package identity is incomplete")
        if package_id in package_records:
            raise InventoryError(f"duplicate package id in cargo metadata: {package_id}")

        source = package.get("source")
        license_expr = package.get("license")
        if source is not None and not isinstance(source, str):
            raise InventoryError(f"invalid source for {name} {version}")
        if license_expr is not None and not isinstance(license_expr, str):
            raise InventoryError(f"invalid license for {name} {version}")

        identity = f"{name}@{version}"
        validate_manifest_dependencies(package, identity)

        workspace = package_id in workspace_ids
        source_class = (
            "workspace"
            if workspace
            else ("crates.io" if source == CRATES_IO_SOURCE else "other")
        )
        source_classes[source_class] += 1
        versions_by_name[name].add(version)
        if not workspace:
            if license_expr:
                license_packages[license_expr].append(identity)
            else:
                unknown_license.append(identity)
            if source != CRATES_IO_SOURCE:
                non_crates_io.append(f"{identity}:{source or '<none>'}")

        package_records[package_id] = {
            "dependencies": [],
            "license": license_expr,
            "name": name,
            "package_id": package_id,
            "source": source,
            "source_class": source_class,
            "version": version,
            "workspace": workspace,
        }

    unknown_workspace_ids = sorted(workspace_ids - set(package_records))
    if unknown_workspace_ids:
        raise InventoryError(
            "workspace members reference unknown package ids: "
            + ", ".join(unknown_workspace_ids)
        )

    resolved_ids: set[str] = set()
    for index, node in enumerate(resolve["nodes"]):
        if not isinstance(node, dict):
            raise InventoryError(f"resolved node {index} is not an object")
        node_id = require_string(node.get("id"), f"resolved node {index} has invalid id")
        if node_id not in package_records:
            raise InventoryError(f"resolved node references unknown package id: {node_id}")
        if node_id in resolved_ids:
            raise InventoryError(f"duplicate resolved node for package id: {node_id}")
        resolved_ids.add(node_id)

        dependency_ids = node.get("dependencies")
        deps = node.get("deps")
        if not isinstance(dependency_ids, list) or not isinstance(deps, list):
            raise InventoryError(f"resolved node {node_id} has invalid dependency arrays")

        exact_dependency_ids: list[str] = []
        for dep_index, dependency_id in enumerate(dependency_ids):
            exact_dependency_ids.append(
                require_string(
                    dependency_id,
                    f"resolved dependency {dep_index} for {node_id} has invalid package id",
                )
            )

        resolved_edges: list[dict[str, object]] = []
        edge_package_ids: list[str] = []
        for dep_index, dep in enumerate(deps):
            if not isinstance(dep, dict):
                raise InventoryError(
                    f"resolved dependency edge {dep_index} for {node_id} is not an object"
                )
            edge_name = require_string(
                dep.get("name"),
                f"resolved dependency edge {dep_index} for {node_id} has invalid name",
            )
            target_id = require_string(
                dep.get("pkg"),
                f"resolved dependency edge {dep_index} for {node_id} has invalid package id",
            )
            target = package_records.get(target_id)
            if target is None:
                raise InventoryError(
                    f"resolved dependency edge for {node_id} references unknown package id: {target_id}"
                )
            edge_package_ids.append(target_id)
            resolved_edges.append(
                {
                    "name": edge_name,
                    "package_id": target_id,
                    "package_name": target["name"],
                    "source": target["source"],
                    "version": target["version"],
                }
            )

        if sorted(set(exact_dependency_ids)) != sorted(set(edge_package_ids)):
            raise InventoryError(
                f"resolved dependency representations disagree for package id: {node_id}"
            )

        resolved_edges.sort(
            key=lambda edge: (
                str(edge["name"]),
                str(edge["package_name"]),
                str(edge["version"]),
                str(edge["source"]),
                str(edge["package_id"]),
            )
        )
        package_records[node_id]["dependencies"] = resolved_edges

    missing_resolved_ids = sorted(set(package_records) - resolved_ids)
    if missing_resolved_ids:
        raise InventoryError(
            "packages missing from resolved dependency graph: "
            + ", ".join(missing_resolved_ids)
        )

    inventory = sorted(
        package_records.values(),
        key=lambda item: (
            str(item["name"]),
            str(item["version"]),
            str(item["source"]),
            str(item["package_id"]),
        ),
    )
    duplicates = {
        name: sorted(versions)
        for name, versions in sorted(versions_by_name.items())
        if len(versions) > 1
    }
    licenses = {
        expression: sorted(identities)
        for expression, identities in sorted(license_packages.items())
    }
    return {
        "duplicates": duplicates,
        "graph_sha256": graph_sha256(inventory),
        "licenses": licenses,
        "non_crates_io": sorted(non_crates_io),
        "ok": not unknown_license,
        "package_count": len(inventory),
        "packages": inventory,
        "schema": 2,
        "source_classes": dict(sorted(source_classes.items())),
        "unknown_license": sorted(unknown_license),
    }


def main() -> int:
    try:
        metadata = json.load(sys.stdin)
        result = summarize(metadata)
    except (json.JSONDecodeError, OSError, InventoryError) as error:
        print(
            json.dumps(
                {"error": f"invalid cargo metadata: {error}", "ok": False},
                sort_keys=True,
            )
        )
        return 1

    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
