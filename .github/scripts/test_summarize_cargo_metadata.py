#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("summarize_cargo_metadata.py")
SPEC = importlib.util.spec_from_file_location("summarize_cargo_metadata_target", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
SUMMARY = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = SUMMARY
SPEC.loader.exec_module(SUMMARY)

CRATES_IO_SOURCE = "registry+https://github.com/rust-lang/crates.io-index"


def package(
    package_id: str,
    name: str,
    version: str,
    *,
    source: str | None = CRATES_IO_SOURCE,
    license_expr: str | None = "MIT",
    dependencies: object | None = None,
) -> dict[str, object]:
    return {
        "id": package_id,
        "name": name,
        "version": version,
        "source": source,
        "license": license_expr,
        "dependencies": [] if dependencies is None else dependencies,
    }


def valid_metadata() -> dict[str, object]:
    root_id = "path+file:///workspace/commandf#0.1.0"
    old_id = "registry+https://github.com/rust-lang/crates.io-index#getrandom@0.2.17"
    new_id = "registry+https://github.com/rust-lang/crates.io-index#getrandom@0.4.3"
    return {
        "packages": [
            package(
                root_id,
                "commandf",
                "0.1.0",
                source=None,
                license_expr=None,
                dependencies=[
                    {"name": "rand_old"},
                    {"name": "rand_new"},
                ],
            ),
            package(old_id, "getrandom", "0.2.17"),
            package(new_id, "getrandom", "0.4.3"),
        ],
        "workspace_members": [root_id],
        "resolve": {
            "nodes": [
                {
                    "id": root_id,
                    "dependencies": [old_id, new_id],
                    "deps": [
                        {"name": "rand_old", "pkg": old_id, "dep_kinds": []},
                        {"name": "rand_new", "pkg": new_id, "dep_kinds": []},
                    ],
                },
                {"id": old_id, "dependencies": [], "deps": []},
                {"id": new_id, "dependencies": [], "deps": []},
            ]
        },
    }


class CargoMetadataSummaryTests(unittest.TestCase):
    def test_resolved_edges_preserve_exact_selected_package_identity(self) -> None:
        result = SUMMARY.summarize(valid_metadata())
        self.assertEqual(result["schema"], 2)
        root = next(item for item in result["packages"] if item["name"] == "commandf")
        self.assertEqual(
            [
                (edge["name"], edge["package_name"], edge["version"], edge["package_id"])
                for edge in root["dependencies"]
            ],
            [
                (
                    "rand_new",
                    "getrandom",
                    "0.4.3",
                    "registry+https://github.com/rust-lang/crates.io-index#getrandom@0.4.3",
                ),
                (
                    "rand_old",
                    "getrandom",
                    "0.2.17",
                    "registry+https://github.com/rust-lang/crates.io-index#getrandom@0.2.17",
                ),
            ],
        )
        self.assertEqual(result["duplicates"], {"getrandom": ["0.2.17", "0.4.3"]})

    def test_manifest_dependencies_must_be_an_array(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][0]["dependencies"] = {"name": "hidden"}
        with self.assertRaisesRegex(SUMMARY.InventoryError, "not an array"):
            SUMMARY.summarize(metadata)

    def test_manifest_dependency_record_must_be_an_object(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][0]["dependencies"] = ["hidden"]
        with self.assertRaisesRegex(SUMMARY.InventoryError, "is not an object"):
            SUMMARY.summarize(metadata)

    def test_manifest_dependency_name_must_be_a_string(self) -> None:
        metadata = valid_metadata()
        metadata["packages"][0]["dependencies"] = [{"name": None}]
        with self.assertRaisesRegex(SUMMARY.InventoryError, "invalid name"):
            SUMMARY.summarize(metadata)

    def test_resolved_edge_requires_known_exact_package_id(self) -> None:
        metadata = valid_metadata()
        root = metadata["resolve"]["nodes"][0]
        missing = "registry+https://github.com/rust-lang/crates.io-index#getrandom@9.9.9"
        root["dependencies"] = [missing]
        root["deps"] = [{"name": "missing", "pkg": missing, "dep_kinds": []}]
        with self.assertRaisesRegex(SUMMARY.InventoryError, "unknown package id"):
            SUMMARY.summarize(metadata)

    def test_resolved_dependency_representations_must_agree(self) -> None:
        metadata = valid_metadata()
        root = metadata["resolve"]["nodes"][0]
        root["dependencies"] = root["dependencies"][:1]
        with self.assertRaisesRegex(SUMMARY.InventoryError, "representations disagree"):
            SUMMARY.summarize(metadata)

    def test_every_package_requires_a_resolved_node(self) -> None:
        metadata = valid_metadata()
        metadata["resolve"]["nodes"].pop()
        with self.assertRaisesRegex(SUMMARY.InventoryError, "missing from resolved dependency graph"):
            SUMMARY.summarize(metadata)


if __name__ == "__main__":
    unittest.main()
