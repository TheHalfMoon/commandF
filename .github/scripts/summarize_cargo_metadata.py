#!/usr/bin/env python3
"""Summarize exact Cargo metadata for AF-01 T020 dependency-policy inspection."""

from __future__ import annotations

import json
import sys
from collections import defaultdict

CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


def main() -> int:
    try:
        metadata = json.load(sys.stdin)
    except (json.JSONDecodeError, OSError) as error:
        print(json.dumps({"error": f"invalid cargo metadata: {error}", "ok": False}, sort_keys=True))
        return 1

    packages = metadata.get("packages")
    workspace_members = metadata.get("workspace_members")
    if not isinstance(packages, list) or not isinstance(workspace_members, list):
        print(json.dumps({"error": "cargo metadata missing package/workspace arrays", "ok": False}, sort_keys=True))
        return 1

    workspace_ids = set(workspace_members)
    inventory: list[dict[str, object]] = []
    versions_by_name: dict[str, set[str]] = defaultdict(set)
    license_packages: dict[str, list[str]] = defaultdict(list)
    source_classes: dict[str, int] = defaultdict(int)
    unknown_license: list[str] = []
    non_crates_io: list[str] = []

    for package in packages:
        if not isinstance(package, dict):
            print(json.dumps({"error": "cargo metadata contains non-object package", "ok": False}, sort_keys=True))
            return 1
        package_id = package.get("id")
        name = package.get("name")
        version = package.get("version")
        source = package.get("source")
        license_expr = package.get("license")
        if not all(isinstance(value, str) for value in (package_id, name, version)):
            print(json.dumps({"error": "package identity is incomplete", "ok": False}, sort_keys=True))
            return 1
        if source is not None and not isinstance(source, str):
            print(json.dumps({"error": f"invalid source for {name} {version}", "ok": False}, sort_keys=True))
            return 1
        if license_expr is not None and not isinstance(license_expr, str):
            print(json.dumps({"error": f"invalid license for {name} {version}", "ok": False}, sort_keys=True))
            return 1

        identity = f"{name}@{version}"
        workspace = package_id in workspace_ids
        source_class = "workspace" if workspace else ("crates.io" if source == CRATES_IO_SOURCE else "other")
        source_classes[source_class] += 1
        versions_by_name[name].add(version)
        if not workspace:
            if license_expr:
                license_packages[license_expr].append(identity)
            else:
                unknown_license.append(identity)
            if source != CRATES_IO_SOURCE:
                non_crates_io.append(f"{identity}:{source or '<none>'}")

        dependencies = package.get("dependencies", [])
        direct_dependency_names = sorted(
            {
                dependency.get("name")
                for dependency in dependencies
                if isinstance(dependency, dict) and isinstance(dependency.get("name"), str)
            }
        )
        inventory.append(
            {
                "dependencies": direct_dependency_names,
                "license": license_expr,
                "name": name,
                "source": source,
                "source_class": source_class,
                "version": version,
                "workspace": workspace,
            }
        )

    inventory.sort(key=lambda item: (str(item["name"]), str(item["version"]), str(item["source"])))
    duplicates = {
        name: sorted(versions)
        for name, versions in sorted(versions_by_name.items())
        if len(versions) > 1
    }
    licenses = {
        expression: sorted(identities)
        for expression, identities in sorted(license_packages.items())
    }
    result = {
        "duplicates": duplicates,
        "licenses": licenses,
        "non_crates_io": sorted(non_crates_io),
        "ok": not unknown_license,
        "package_count": len(inventory),
        "packages": inventory,
        "schema": 1,
        "source_classes": dict(sorted(source_classes.items())),
        "unknown_license": sorted(unknown_license),
    }
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
