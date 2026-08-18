import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest


MODULE_PATH = Path(__file__).with_name("verify_closure_binding.py")
SPEC = importlib.util.spec_from_file_location("verify_closure_binding", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


def package(name, version, sha, dependencies=None, source=None):
    value = {
        "name": name,
        "version": version,
        "sha256": sha,
        "source": source or f"https://packages.example/{name}/{version}",
        "dependencies": dependencies or {},
    }
    return value


def normalized(packages):
    values = [
        {
            "name": p["name"],
            "version": p["version"],
            "sha256": p["sha256"],
            "dependencies": {
                key: p["dependencies"][key] for key in sorted(p["dependencies"])
            },
        }
        for p in packages
    ]
    return sorted(
        values,
        key=lambda p: (
            p["name"],
            p["version"],
            p["sha256"],
            tuple(p["dependencies"].items()),
        ),
    )


def summary_for(before_packages, after_packages):
    before = normalized(before_packages)
    after = normalized(after_packages)
    return {
        "schema": 1,
        "manifest_sha256": "f" * 64,
        "cases": [
            {
                "case_id": "C001",
                "package": "root.pkg",
                "before": {
                    "version": "1.0.0",
                    "sha256": "a" * 64,
                    "closure_sha256": MODULE.closure_sha256(before),
                    "closure": before,
                },
                "after": {
                    "version": "2.0.0",
                    "sha256": "b" * 64,
                    "closure_sha256": MODULE.closure_sha256(after),
                    "closure": after,
                },
                "status": "oracle_failed",
                "structural": None,
                "compatibility": None,
                "terminology": None,
                "oracle": None,
            }
        ],
    }


class BindingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.evidence = self.root / "evidence" / "C001"
        self.evidence.mkdir(parents=True)

        self.before = [
            package(
                "root.pkg",
                "1.0.0",
                "a" * 64,
                {"same.dep": "1.0.0", "leaf.dep": "3.0.0"},
            ),
            package("same.dep", "1.0.0", "c" * 64),
            package("same.dep", "2.0.0", "d" * 64),
            package("leaf.dep", "3.0.0", "e" * 64),
        ]
        self.after = [
            package("root.pkg", "2.0.0", "b" * 64, {"same.dep": "2.0.0"}),
            package("same.dep", "2.0.0", "d" * 64),
        ]
        self.summary = summary_for(self.before, self.after)
        self._write_all(self.before, self.after)

    def tearDown(self):
        self.temp.cleanup()

    def _write_lock(self, side, root_version, packages):
        value = {
            "schema": 1,
            "roots": [f"root.pkg@{root_version}"],
            "packages": packages,
        }
        (self.evidence / f"{side}.commandf.lock").write_text(
            json.dumps(value, indent=2) + "\n", encoding="utf-8"
        )

    def _write_summary(self, summary=None):
        path = self.root / "summary.json"
        path.write_text(json.dumps(summary or self.summary), encoding="utf-8")
        return path

    def _write_all(self, before, after):
        self._write_lock("before", "1.0.0", before)
        self._write_lock("after", "2.0.0", after)

    def _verify(self, summary=None):
        MODULE.verify_summary(self._write_summary(summary), self.root / "evidence")

    def test_exact_binding_with_same_name_multi_version_passes(self):
        self._verify()

    def test_source_only_change_is_excluded_from_closure_contract(self):
        changed = [dict(value) for value in self.before]
        changed[1] = dict(
            changed[1], source="https://mirror.example/same.dep/1.0.0"
        )
        self._write_lock("before", "1.0.0", changed)
        self._verify()

    def test_dependency_map_only_tampering_fails(self):
        changed = [dict(value) for value in self.before]
        changed[0] = dict(
            changed[0],
            dependencies={"same.dep": "2.0.0", "leaf.dep": "3.0.0"},
        )
        self._write_lock("before", "1.0.0", changed)
        with self.assertRaisesRegex(MODULE.BindingError, "does not match summary closure"):
            self._verify()

    def test_missing_package_partial_binding_fails(self):
        self._write_lock("before", "1.0.0", self.before[:-1])
        with self.assertRaisesRegex(MODULE.BindingError, "does not match summary closure"):
            self._verify()

    def test_digest_mismatch_fails_even_when_closure_matches(self):
        changed_summary = json.loads(json.dumps(self.summary))
        changed_summary["cases"][0]["before"]["closure_sha256"] = "0" * 64
        with self.assertRaisesRegex(MODULE.BindingError, "closure digest mismatch"):
            self._verify(changed_summary)

    def test_root_identity_mismatch_fails(self):
        value = {
            "schema": 1,
            "roots": ["other.pkg@1.0.0"],
            "packages": self.before,
        }
        (self.evidence / "before.commandf.lock").write_text(
            json.dumps(value), encoding="utf-8"
        )
        with self.assertRaisesRegex(MODULE.BindingError, "roots must contain exactly one"):
            self._verify()

    def test_summary_closure_order_tampering_fails(self):
        changed_summary = json.loads(json.dumps(self.summary))
        closure = changed_summary["cases"][0]["before"]["closure"]
        closure[0], closure[1] = closure[1], closure[0]
        with self.assertRaisesRegex(MODULE.BindingError, "not canonically sorted"):
            self._verify(changed_summary)


if __name__ == "__main__":
    unittest.main()
