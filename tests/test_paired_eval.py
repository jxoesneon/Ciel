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


def _run_eval(skill: Path, runner: str, *extra: str, env_extra: dict | None = None) -> tuple[int, dict]:
    report = skill.parent / "report.json"
    cmd = [
        sys.executable, str(PAIRED_EVAL),
        "--skill", str(skill),
        "--tasks", str(FIXTURE_TASKS),
        "--runner", runner,
        "--report", str(report),
        *extra,
    ]
    env = dict(os.environ)
    if env_extra:
        env.update(env_extra)
    proc = subprocess.run(cmd, env=env, capture_output=True, text=True, check=False)
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

    def test_completion_gate_shadow_records(self):
        import http.server
        import threading
        class H(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                self.send_response(200)
                self.send_header("content-type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({
                    "answers": {"done": {"type": "choice", "choice": "complete", "confidence": 0.95}},
                    "model": "stub"
                }).encode())
            def log_message(self, *a):
                pass
        srv = http.server.HTTPServer(("127.0.0.1", 0), H)
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        self.addCleanup(srv.shutdown)

        with tempfile.TemporaryDirectory() as tmp:
            skill = _skill_dir(Path(tmp))
            rc, report = _run_eval(
                skill, "touch marker.txt", "--completion-gate", "shadow",
                env_extra={
                    "CIEL_HOME": str(Path(tmp) / ".ciel"),
                    "CIEL_SYSTEM1_URL": f"http://127.0.0.1:{srv.server_port}",
                    "CIEL_SYSTEM1_KEY": "k",
                }
            )
            self.assertEqual(0, rc)
            self.assertEqual("shadow", report["completion_gate"])
            ctrl_check = report["tasks"][0]["control"].get("completion_check")
            self.assertIsNotNone(ctrl_check)
            self.assertEqual("complete", ctrl_check["choice"])

    def test_completion_gate_enforce_catches_false_pass(self):
        import http.server
        import threading
        class H(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                self.send_response(200)
                self.send_header("content-type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({
                    "answers": {"done": {"type": "choice", "choice": "incomplete", "confidence": 0.9}},
                    "model": "stub"
                }).encode())
            def log_message(self, *a):
                pass
        srv = http.server.HTTPServer(("127.0.0.1", 0), H)
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        self.addCleanup(srv.shutdown)

        with tempfile.TemporaryDirectory() as tmp:
            skill = _skill_dir(Path(tmp))
            # verify.sh succeeds because runner touches marker.txt, but System-1 returns incomplete
            rc, report = _run_eval(
                skill, "touch marker.txt", "--completion-gate", "enforce",
                env_extra={
                    "CIEL_HOME": str(Path(tmp) / ".ciel"),
                    "CIEL_SYSTEM1_URL": f"http://127.0.0.1:{srv.server_port}",
                    "CIEL_SYSTEM1_KEY": "k",
                }
            )
            # False pass was intercepted: verify_pass was True, but arm pass flipped to False
            self.assertEqual(2, rc)
            self.assertEqual("fail", report["verdict"])
            ctrl = report["tasks"][0]["control"]
            self.assertTrue(ctrl["verify_pass"])
            self.assertFalse(ctrl["pass"])
            self.assertTrue(ctrl.get("false_pass_detected"))

    def test_completion_gate_fail_open_offline(self):
        with tempfile.TemporaryDirectory() as tmp:
            skill = _skill_dir(Path(tmp))
            # System1 pointing to unreachable port with enforce gate
            rc, report = _run_eval(
                skill, "touch marker.txt", "--completion-gate", "enforce",
                env_extra={
                    "CIEL_HOME": str(Path(tmp) / ".ciel"),
                    "CIEL_SYSTEM1_URL": "http://127.0.0.1:9",
                    "CIEL_SYSTEM1_TIMEOUT": "0.2",
                }
            )
            # Fails open: returns pass because deterministic verify.sh passed
            self.assertEqual(0, rc)
            self.assertEqual("pass", report["verdict"])
            ctrl = report["tasks"][0]["control"]
            self.assertTrue(ctrl["pass"])


if __name__ == "__main__":
    unittest.main()
