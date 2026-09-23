"""Tests for scripts/migrate_skill_sidecar.py."""

import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import yaml

SCRIPT = (
    Path(__file__).resolve().parent.parent / "scripts" / "migrate_skill_sidecar.py"
)

OLD_SKILL = """---
name: widget
version: 2.1.0
format: skill/1.0
description: A test widget skill.
runtimes: ["claude_code", "generic"]
license: MIT
tags: ["ciel", "domain:test"]
triggers:
  - pattern: "widget.*"
    confidence: 0.9
source: { tier: 1, origin: test }
dependencies: { skills: [], mcp: [], system: [] }
side_effects: ["filesystem"]
---

# Widget

Body text preserved byte-for-byte.
"""


def _load_module():
    spec = importlib.util.spec_from_file_location("migrate_skill_sidecar", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class MigrateSidecarTests(unittest.TestCase):
    def setUp(self):
        self.mod = _load_module()
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.skill = self.root / "skills" / "widget"
        self.skill.mkdir(parents=True)
        (self.skill / "SKILL.md").write_text(OLD_SKILL, encoding="utf-8")

    def _run(self, *args):
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--root", str(self.root), *args],
            capture_output=True, text=True,
        )

    def test_check_fails_before_migration(self):
        proc = self._run("--check")
        self.assertEqual(proc.returncode, 1)

    def test_apply_splits_frontmatter(self):
        self.assertEqual(self._run("--apply").returncode, 0)

        skill_text = (self.skill / "SKILL.md").read_text(encoding="utf-8")
        fm = yaml.safe_load(skill_text.split("---", 2)[1])
        self.assertEqual(set(fm) - {"metadata"},
                         {"name", "description", "license"})
        self.assertEqual(fm["metadata"]["ciel-extension"], "ciel.yaml")
        self.assertEqual(fm["metadata"]["ciel-version"], "2.1.0")

        sidecar = yaml.safe_load(
            (self.skill / "ciel.yaml").read_text(encoding="utf-8")
        )
        old_fm = yaml.safe_load(OLD_SKILL.split("---", 2)[1])
        for key in ("version", "format", "runtimes", "tags", "triggers",
                    "source", "dependencies", "side_effects"):
            self.assertEqual(sidecar[key], old_fm[key], key)
        self.assertEqual(sidecar["schema"], 1)

        body = skill_text.split("---", 2)[2]
        self.assertEqual(body, OLD_SKILL.split("---", 2)[2])

    def test_apply_is_idempotent(self):
        self.assertEqual(self._run("--apply").returncode, 0)
        before_md = (self.skill / "SKILL.md").read_bytes()
        before_yaml = (self.skill / "ciel.yaml").read_bytes()
        self.assertEqual(self._run("--apply").returncode, 0)
        self.assertEqual((self.skill / "SKILL.md").read_bytes(), before_md)
        self.assertEqual((self.skill / "ciel.yaml").read_bytes(), before_yaml)

    def test_check_passes_after_migration(self):
        self.assertEqual(self._run("--apply").returncode, 0)
        proc = self._run("--check")
        self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main()
