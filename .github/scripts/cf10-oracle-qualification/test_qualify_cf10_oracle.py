from __future__ import annotations

import importlib.util
import io
import json
import os
import sys
import tarfile
import tempfile
import unittest
from pathlib import Path
from unittest import mock


SCRIPT = Path(__file__).with_name("qualify_cf10_oracle.py")
SPEC = importlib.util.spec_from_file_location("qualify_cf10_oracle", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
qualification = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = qualification
SPEC.loader.exec_module(qualification)


def package(name: str, version: str, dependencies: dict[str, str] | None = None) -> dict:
    return {
        "name": name,
        "version": version,
        "sha256": (name + version).encode().hex()[:64].ljust(64, "0"),
        "source": f"https://example.test/{name}/{version}",
        "dependencies": dependencies or {},
    }


def state_for(lock: dict, root: dict, core: dict) -> qualification.VerifiedState:
    spec = qualification.StateSpec(
        "T001",
        "before",
        root["name"],
        root["version"],
        root["sha256"],
        1,
        "a" * 64,
    )
    return qualification.VerifiedState(spec, Path("."), lock, root, core)


def probe_result(status: str, exception: str | None = None, message: str | None = None) -> dict:
    return {
        "status": status,
        "phase": "comparison",
        "exception_class": exception,
        "exception_message": message,
    }


def state_with_resources(
    parent: Path,
    side: str,
    version: str,
    resources: list[tuple[str, dict]],
) -> qualification.VerifiedState:
    root = package("example.root", version)
    core = package("hl7.fhir.r4.core", "4.0.1")
    state_root = parent / side
    archive = state_root / "cache" / "sha256" / f"{root['sha256']}.tgz"
    archive.parent.mkdir(parents=True)
    with tarfile.open(archive, mode="w:gz") as package_file:
        for filename, value in resources:
            raw = qualification.canonical_json_bytes(value)
            info = tarfile.TarInfo(f"package/{filename}")
            info.size = len(raw)
            package_file.addfile(info, io.BytesIO(raw))
    spec = qualification.StateSpec(
        "T001",
        side,
        root["name"],
        root["version"],
        root["sha256"],
        archive.stat().st_size,
        "a" * 64,
    )
    return qualification.VerifiedState(
        spec, state_root, {"packages": [root, core]}, root, core
    )


def structural_report(*keys: str) -> dict:
    return {
        "schema": 1,
        "changes": [
            {"resource": {"kind": "canonical", "value": key}} for key in keys
        ],
    }


class QualificationTests(unittest.TestCase):
    def test_unique_canonical_version_change_uses_unversioned_production_key(self) -> None:
        url = "https://example.test/StructureDefinition/unique"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            before = state_with_resources(
                root,
                "before",
                "1.0.0",
                [("before.json", {"resourceType": "StructureDefinition", "url": url, "version": "1"})],
            )
            after = state_with_resources(
                root,
                "after",
                "2.0.0",
                [("after.json", {"resourceType": "StructureDefinition", "url": url, "version": "2"})],
            )
            pairs = qualification.matched_changed_profiles(
                before, after, structural_report(url)
            )
        self.assertEqual([pair.resource_key for pair in pairs], [url])
        self.assertIsNone(pairs[0].lookup_version)

    def test_multi_version_canonical_identities_are_not_collapsed(self) -> None:
        url = "https://example.test/StructureDefinition/versioned"
        before_resources = [
            (
                f"before-{version}.json",
                {
                    "resourceType": "StructureDefinition",
                    "url": url,
                    "version": version,
                    "status": "draft",
                },
            )
            for version in ("1", "2")
        ]
        after_resources = [
            (
                f"after-{version}.json",
                {
                    "resourceType": "StructureDefinition",
                    "url": url,
                    "version": version,
                    "status": "active",
                },
            )
            for version in ("1", "2")
        ]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            before = state_with_resources(root, "before", "1.0.0", before_resources)
            after = state_with_resources(root, "after", "2.0.0", after_resources)
            pairs = qualification.matched_changed_profiles(
                before,
                after,
                structural_report(f"{url}|1", f"{url}|2"),
            )
        self.assertEqual(
            [(pair.resource_key, pair.lookup_version) for pair in pairs],
            [(f"{url}|1", "1"), (f"{url}|2", "2")],
        )

    def test_full_closure_preserves_versions_excludes_r5_and_orders_dependencies_first(self) -> None:
        core = package("hl7.fhir.r4.core", "4.0.1")
        r5_core = package("hl7.fhir.r5.core", "5.0.0")
        extension_v1 = package(
            "hl7.fhir.uv.extensions.r4", "1.0.0", {core["name"]: core["version"]}
        )
        extension_v2 = package(
            "hl7.fhir.uv.extensions.r4", "5.2.0", {core["name"]: core["version"]}
        )
        neutral_leaf = package("hl7.fhir.r4.examples", "4.0.1")
        r5_branch = package(
            "hl7.fhir.uv.extensions", "5.1.0", {r5_core["name"]: r5_core["version"]}
        )
        bridge = package(
            "example.bridge",
            "1.0.0",
            {
                core["name"]: core["version"],
                extension_v1["name"]: extension_v1["version"],
                neutral_leaf["name"]: neutral_leaf["version"],
                r5_branch["name"]: r5_branch["version"],
            },
        )
        root = package(
            "example.root",
            "1.0.0",
            {
                core["name"]: core["version"],
                bridge["name"]: bridge["version"],
                extension_v2["name"]: extension_v2["version"],
            },
        )
        lock = {
            "schema": 1,
            "roots": ["example.root@1.0.0"],
            "packages": [
                bridge,
                core,
                extension_v1,
                extension_v2,
                neutral_leaf,
                r5_branch,
                r5_core,
                root,
            ],
        }
        contexts = qualification.full_context_packages(state_for(lock, root, core))
        identities = [qualification.package_identity(item) for item in contexts]
        self.assertIn((extension_v1["name"], extension_v1["version"]), identities)
        self.assertIn((extension_v2["name"], extension_v2["version"]), identities)
        self.assertIn((neutral_leaf["name"], neutral_leaf["version"]), identities)
        self.assertNotIn((r5_branch["name"], r5_branch["version"]), identities)
        self.assertLess(
            identities.index((extension_v1["name"], extension_v1["version"])),
            identities.index((bridge["name"], bridge["version"])),
        )
        self.assertLess(
            identities.index((neutral_leaf["name"], neutral_leaf["version"])),
            identities.index((bridge["name"], bridge["version"])),
        )

    def test_duplicate_dependency_identity_fails_closed(self) -> None:
        root = package("example.root", "1.0.0", {"example.dep": "1.0.0"})
        first = package("example.dep", "1.0.0")
        second = {**first, "sha256": "f" * 64}
        lock = {"packages": [root, first, second]}
        with self.assertRaisesRegex(ValueError, "matched 2 locked packages"):
            qualification.select_dependency(lock, root, "example.dep", "1.0.0")

    def test_named_slice_before_local_slicing_is_preserved(self) -> None:
        profile = qualification.ProfileResource(
            "StructureDefinition-profile.json",
            {
                "url": "https://example.test/StructureDefinition/profile",
                "version": "1.0.0",
                "baseDefinition": "http://hl7.org/fhir/StructureDefinition/Observation",
                "derivation": "constraint",
                "snapshot": {
                    "element": [
                        {"id": "Observation", "path": "Observation"},
                        {
                            "id": "Observation.category",
                            "path": "Observation.category",
                            "slicing": {"rules": "open"},
                        },
                        {
                            "id": "Observation.category:lab",
                            "path": "Observation.category",
                            "sliceName": "lab",
                        },
                    ]
                },
                "differential": {
                    "element": [
                        {"id": "Observation", "path": "Observation"},
                        {
                            "id": "Observation.category:lab",
                            "path": "Observation.category",
                            "sliceName": "lab",
                        },
                        {
                            "id": "Observation.category",
                            "path": "Observation.category",
                            "slicing": {"rules": "open"},
                        },
                    ]
                },
            },
        )
        shape = qualification.differential_shape(profile, "Observation.category")
        differential = shape["differential"]
        snapshot = shape["snapshot"]
        self.assertTrue(differential["named_slice_precedes_slicing"])
        self.assertFalse(differential["slicing_before_first_named_slice"])
        self.assertEqual(differential["first_named_slice_index"], 1)
        self.assertEqual(differential["slicing_indices"], [2])
        self.assertTrue(snapshot["slicing_before_first_named_slice"])
        self.assertEqual(snapshot["slicing_indices"], [1])

    def test_self_definition_exception_in_both_modes_classifies_pinned_limit(self) -> None:
        probes = []
        failure = probe_result(
            "exception",
            "org.hl7.fhir.exceptions.DefinitionException",
            "Found a slice at 'Observation.category' but there was no definition for the slicing",
        )
        for label in ("self_before", "self_after", "cross"):
            for mode in ("DIRECT", "FULL_CLOSURE"):
                result = probe_result("completed")
                if label in {"self_before", "cross"}:
                    result = failure
                probes.append(
                    {"case_id": "C001", "probe": label, "context_mode": mode, "result": result}
                )
        self.assertEqual(
            qualification.case_classification("C001", probes, failure),
            qualification.ROOT_CAUSE_PINNED,
        )

    def test_full_closure_completion_after_direct_failure_classifies_context(self) -> None:
        probes = []
        failure = probe_result(
            "exception",
            "org.hl7.fhir.exceptions.DefinitionException",
            "Found a slice at 'Observation.category' but there was no definition for the slicing",
        )
        for label in ("self_before", "self_after", "cross"):
            for mode in ("DIRECT", "FULL_CLOSURE"):
                result = probe_result("completed")
                if label == "cross" and mode == "DIRECT":
                    result = failure
                probes.append(
                    {"case_id": "C001", "probe": label, "context_mode": mode, "result": result}
                )
        self.assertEqual(
            qualification.case_classification("C001", probes, failure),
            qualification.ROOT_CAUSE_CONTEXT,
        )

    def test_exception_cannot_be_coerced_to_oracle_status(self) -> None:
        payload = {
            "schema": 1,
            "oracle": qualification.ORACLE,
            "status": "uncomparable",
        }
        completed = qualification.ProcessResult(0, qualification.canonical_json_bytes(payload), b"")
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.object(qualification, "run_bounded", return_value=completed), mock.patch(
                "builtins.print"
            ):
                with self.assertRaisesRegex(ValueError, "coerced"):
                    qualification.run_probe(
                        Path(directory),
                        {"case_id": "C001"},
                        ["java"],
                    )

    def test_classpath_component_cannot_leak_into_evidence(self) -> None:
        payload = {
            "schema": 1,
            "oracle": qualification.ORACLE,
            "status": "exception",
            "phase": "context_load",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            classes = root.parent / "probe-classes"
            process = qualification.ProcessResult(
                2,
                qualification.canonical_json_bytes(payload),
                str(classes).encode(),
            )
            classpath = os.pathsep.join((str(classes), str(root.parent / "oracle.jar")))
            with mock.patch.object(
                qualification, "run_bounded", return_value=process
            ), mock.patch("builtins.print"):
                with self.assertRaisesRegex(ValueError, "host-absolute path"):
                    qualification.run_probe(
                        root,
                        {"case_id": "C001"},
                        ["java", "-cp", classpath],
                    )

    def test_resolved_resource_identity_mismatch_fails_closed(self) -> None:
        expected = {
            "url": "https://example.test/StructureDefinition/expected",
            "version": "1",
            "id": "expected",
            "type": "Observation",
        }
        actual = {**expected, "id": "different"}
        payload = {
            "schema": 1,
            "oracle": qualification.ORACLE,
            "status": "completed",
            "phase": "comparison",
            "left_resource": actual,
            "right_resource": expected,
        }
        process = qualification.ProcessResult(
            0, qualification.canonical_json_bytes(payload), b""
        )
        invocation = {
            "case_id": "C001",
            "left": {"resource": expected},
            "right": {"resource": expected},
        }
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.object(
                qualification, "run_bounded", return_value=process
            ), mock.patch("builtins.print"):
                with self.assertRaisesRegex(ValueError, "resolved left"):
                    qualification.run_probe(Path(directory), invocation, ["java"])

    def test_unrelated_discovery_exception_cannot_classify_root_cause(self) -> None:
        probes = []
        for label in ("self_before", "self_after", "cross"):
            for mode in ("DIRECT", "FULL_CLOSURE"):
                probes.append(
                    {
                        "case_id": "C001",
                        "probe": label,
                        "context_mode": mode,
                        "result": probe_result("completed"),
                    }
                )
        unrelated = probe_result("exception", "example.ContextException", "missing")
        self.assertEqual(
            qualification.case_classification("C001", probes, unrelated),
            qualification.ROOT_CAUSE_NOT_PROVEN,
        )

    def test_duplicate_canonical_profile_fails_closed(self) -> None:
        root = package("example.root", "1.0.0")
        core = package("hl7.fhir.r4.core", "4.0.1")
        with tempfile.TemporaryDirectory() as directory:
            state_root = Path(directory)
            archive = state_root / "cache" / "sha256" / f"{root['sha256']}.tgz"
            archive.parent.mkdir(parents=True)
            resource = json.dumps(
                {
                    "resourceType": "StructureDefinition",
                    "url": "https://example.test/StructureDefinition/profile",
                }
            ).encode()
            with tarfile.open(archive, mode="w:gz") as package_file:
                for name in ("package/first.json", "package/second.json"):
                    info = tarfile.TarInfo(name)
                    info.size = len(resource)
                    package_file.addfile(info, io.BytesIO(resource))
            spec = qualification.StateSpec(
                "T001",
                "before",
                root["name"],
                root["version"],
                root["sha256"],
                archive.stat().st_size,
                "a" * 64,
            )
            state = qualification.VerifiedState(spec, state_root, {"packages": [root]}, root, core)
            with self.assertRaisesRegex(ValueError, "matched 2 profiles"):
                qualification.find_profile(
                    state, "https://example.test/StructureDefinition/profile"
                )

    def test_output_and_bounding_are_deterministic(self) -> None:
        value = {"b": 2, "a": [3, 1]}
        self.assertEqual(
            qualification.canonical_json_bytes(value), qualification.canonical_json_bytes(value)
        )
        bounded = qualification.bounded_text("x" * (qualification.MAX_DIAGNOSTIC_CHARS + 10))
        self.assertTrue(bounded.endswith("... [diagnostic truncated]"))
        self.assertLessEqual(
            len(bounded),
            qualification.MAX_DIAGNOSTIC_CHARS + len("... [diagnostic truncated]"),
        )


if __name__ == "__main__":
    unittest.main()
