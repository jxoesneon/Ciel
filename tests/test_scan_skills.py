"""Tests for scripts/scan_skills.py (skillfrisk is stubbed via SCAN_SKILLS_CMD)."""

import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent.parent / "scripts" / "scan_skills.py"

FAKE = """#!/usr/bin/env bash
# Stub for `uvx skillfrisk scan <dir> --json`: echoes $FAKE_DIR/<basename>.json
cat "$FAKE_DIR/$(basename "$1").json"
"""


class ScanSkillsTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.work = Path(self.tmp.name)
        self.fake_dir = self.work / "canned"
        self.fake_dir.mkdir()
        self.fake = self.work / "fake_skillfrisk.sh"
        self.fake.write_text(FAKE)
        self.fake.chmod(self.fake.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        self.env = dict(os.environ, SCAN_SKILLS_CMD=str(self.fake),
                        FAKE_DIR=str(self.fake_dir))

    def _skill(self, name: str) -> Path:
        d = self.work / name
        d.mkdir()
        (d / "SKILL.md").write_text("---\nname: %s\n---\n" % name)
        return d

    def _canned(self, name: str, failed: bool, findings=None):
        (self.fake_dir / f"{name}.json").write_text(json.dumps({
            "root": name, "files_scanned": 1, "risk_score": 5 if failed else 0,
            "failed": failed, "findings": findings or [],
        }))

    def _run(self, *args, env=None):
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args],
            capture_output=True, text=True, env=env or self.env,
        )

    def test_pass_when_no_failures(self):
        ok = self._skill("ok-skill")
        self._canned("ok-skill", failed=False)
        report = self.work / "report.json"
        proc = self._run(str(ok), "--report", str(report))
        self.assertEqual(proc.returncode, 0, proc.stderr)
        data = json.loads(report.read_text())
        self.assertEqual(data["scanned"], 1)
        self.assertEqual(data["failed"], [])

    def test_exit_2_on_failed_skill(self):
        bad = self._skill("bad-skill")
        self._canned("bad-skill", failed=True,
                     findings=[{"severity": "high", "title": "x"}])
        proc = self._run(str(bad))
        self.assertEqual(proc.returncode, 2)
        self.assertIn("failed=1", proc.stdout)

    def test_fail_on_any_flag(self):
        mid = self._skill("mid-skill")
        self._canned("mid-skill", failed=False,
                     findings=[{"severity": "low", "title": "note"}])
        self.assertEqual(self._run(str(mid)).returncode, 0)
        self.assertEqual(self._run(str(mid), "--fail-on", "any").returncode, 2)

    def test_missing_tool_is_advisory(self):
        env = dict(os.environ)
        env.pop("SCAN_SKILLS_CMD", None)
        env["PATH"] = str(self.work / "empty-bin")  # no uvx on PATH
        (self.work / "empty-bin").mkdir()
        ok = self._skill("ok-skill")
        self.assertEqual(self._run(str(ok), env=env).returncode, 0)
        self.assertEqual(self._run(str(ok), "--strict", env=env).returncode, 2)


if __name__ == "__main__":
    unittest.main()
