"""Tests for scripts/paired_eval.py using deterministic stub runners."""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PAIRED_EVAL = ROOT / "scripts" / "paired_eval.py"
FIXTURE_TASKS = ROOT / "tests" / "fixtures" / "eval_tasks"


def _skill_dir(base: Path) -> Path:
    skill = base / "cand-skill"
    skill.mkdir(parents=True)
    (skill / "SKILL.md").write_text("---\nname: cand-skill\ndescription: test\n---\n", encoding="utf-8")
    return skill


def _run_eval(skill: Path, runner: str, *extra: str) -> tuple[int, dict]:
    report = skill.parent / "report.json"
    cmd = [
        sys.executable, str(PAIRED_EVAL),
        "--skill", str(skill),
        "--tasks", str(FIXTURE_TASKS),
        "--runner", runner,
        "--report", str(report),
        *extra,
    ]
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    return proc.returncode, json.loads(report.read_text(encoding="utf-8"))


@unittest.skipUnless(shutil.which("bash"), "bash required")
class TestPairedEval(unittest.TestCase):
    def test_preserved_pass_verdict_pass(self):
        with tempfile.TemporaryDirectory() as tmp:
            skill = _skill_dir(Path(tmp))
            rc, report = _run_eval(skill, "touch marker.txt")
            self.assertEqual(0, rc)
            self.assertEqual("pass", report["verdict"])
            self.assertEqual("preserved-pass", report["tasks"][0]["outcome"])

    def test_regression_verdict_fail(self):
        # Runner succeeds only when the skill is NOT installed.
        runner = 'test ! -d .devin/skills && touch marker.txt'
        with tempfile.TemporaryDirectory() as tmp:
            skill = _skill_dir(Path(tmp))
            rc, report = _run_eval(skill, runner)
            self.assertEqual(2, rc)
            self.assertEqual("fail", report["verdict"])
            self.assertEqual(["probe"], report["regressions"])

    def test_improvement_verdict_pass(self):
        # Runner succeeds only when the skill IS installed.
        runner = 'test -d .devin/skills && touch marker.txt'
        with tempfile.TemporaryDirectory() as tmp:
            skill = _skill_dir(Path(tmp))
            rc, report = _run_eval(skill, runner)
            self.assertEqual(0, rc)
            self.assertEqual("improvement", report["tasks"][0]["outcome"])
            self.assertEqual(["probe"], report["improvements"])

    def test_require_improvement_fails_on_preserved(self):
        with tempfile.TemporaryDirectory() as tmp:
            skill = _skill_dir(Path(tmp))
            rc, report = _run_eval(skill, "touch marker.txt", "--require-improvement")
            self.assertEqual(2, rc)
            self.assertEqual("fail", report["verdict"])

    def test_skill_installed_only_in_treatment(self):
        # Stub runner records whether the skill dir exists in each arm.
        runner = (
            'if [ -d .devin/skills/cand-skill ]; then echo T > arm.txt; '
            'else echo C > arm.txt; fi; touch marker.txt'
        )
        with tempfile.TemporaryDirectory() as tmp:
            skill = _skill_dir(Path(tmp))
            out = Path(tmp) / "report.json"
            subprocess.run(
                [sys.executable, str(PAIRED_EVAL), "--skill", str(skill),
                 "--tasks", str(FIXTURE_TASKS), "--runner", runner,
                 "--report", str(out), "--keep-workspaces"],
                capture_output=True, text=True, check=False,
            )
            kept = Path(tempfile.gettempdir()) / "ciel-eval-probe"
            self.assertEqual("C\n", (kept / "control" / "arm.txt").read_text())
            self.assertEqual("T\n", (kept / "treatment" / "arm.txt").read_text())
            self.assertTrue((kept / "treatment" / ".devin" / "skills" / "cand-skill" / "SKILL.md").is_file())
            shutil.rmtree(kept, ignore_errors=True)


if __name__ == "__main__":
    unittest.main()
