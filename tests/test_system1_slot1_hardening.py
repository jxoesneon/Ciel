#!/usr/bin/env python3
"""Slot 1 Verification & Hardening Suite: PreTool Safety Integration.

Verifies:
1. Active intercept logic on unmodeled destructive or dangerous actions.
2. Fail-open behavior strictly maintained when endpoint is down, with latency < 5ms.
3. allow_privileged override functions properly when System-1 intercepts.
4. Antigravity and Devin runtime outputs strictly conform to their respective stdout JSON contracts.
"""

import http.server
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "ciel.skill" / "init" / "hooks" / "lib"
CIEL_BIN = os.environ.get(
    "CIEL_RS_BIN",
    str(ROOT / "ciel.skill" / "init" / "ciel-rs" / "target" / "release" / "ciel"),
)

sys.path.insert(0, str(LIB))
import risk_policy  # noqa: E402
import system1      # noqa: E402


class MockLayaServer:
    """Mock Laya/System-1 server for intercept testing."""

    def __init__(self, answer_fn_or_dict):
        self.answer_fn = (
            answer_fn_or_dict
            if callable(answer_fn_or_dict)
            else (lambda req: answer_fn_or_dict)
        )
        self.requests = []
        self._lock = threading.Lock()

        class Handler(http.server.BaseHTTPRequestHandler):
            def do_POST(s):
                clen = int(s.headers.get("content-length", "0"))
                body = s.rfile.read(clen)
                data = json.loads(body.decode("utf-8")) if body else {}
                with self._lock:
                    self.requests.append(data)
                resp_payload = self.answer_fn(data)

                s.send_response(200)
                s.send_header("content-type", "application/json")
                s.send_header("connection", "close")
                s.end_headers()
                s.wfile.write(json.dumps(resp_payload).encode("utf-8"))

            def log_message(s, *a):
                pass

        self.server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
        self.port = self.server.server_address[1]
        self.url = f"http://127.0.0.1:{self.port}"
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def count(self):
        with self._lock:
            return len(self.requests)

    def shutdown(self):
        self.server.shutdown()
        self.server.server_close()


class BaseSlot1Test(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="ciel-s1-slot1-"))
        self.ciel = self.tmp / ".ciel"
        self.ciel.mkdir(parents=True)
        self._saved = {
            k: os.environ.get(k)
            for k in (
                "CIEL_HOME",
                "HOME",
                "CIEL_BIN",
                "CIEL_SYSTEM1_MODE",
                "CIEL_SYSTEM1_URL",
                "CIEL_SYSTEM1_KEY",
                "CIEL_SYSTEM1_DISABLED",
            )
        }
        os.environ["HOME"] = str(self.tmp)
        os.environ["CIEL_HOME"] = str(self.ciel)
        os.environ["CIEL_BIN"] = CIEL_BIN
        os.environ.pop("CIEL_SYSTEM1_DISABLED", None)
        self.env = dict(os.environ)

    def tearDown(self):
        for k, v in self._saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
        shutil.rmtree(self.tmp, ignore_errors=True)

    def run_rust(self, *args, stdin="", env=None):
        proc = subprocess.run(
            [CIEL_BIN, *args],
            input=stdin,
            capture_output=True,
            text=True,
            env=env or self.env,
            timeout=15,
        )
        return proc.returncode, proc.stdout, proc.stderr


class TestPreToolSafetyHardening(BaseSlot1Test):
    """Exhaustive tests covering all 4 Slot 1 requirements."""

    # -------------------------------------------------------------------------
    # Requirement 1: Active intercept logic on unmodeled dangerous actions
    # -------------------------------------------------------------------------

    def test_01_active_intercept_unmodeled_action_antigravity(self):
        """Unmodeled destructive action is actively intercepted under Antigravity."""
        server = MockLayaServer({
            "answers": {
                "risk": {
                    "choice": "dangerous",
                    "confidence": 0.94,
                    "probabilities": {"dangerous": 0.94, "safe": 0.06},
                }
            },
            "model": "laya-slot1",
        })
        self.addCleanup(server.shutdown)

        self.env["CIEL_SYSTEM1_MODE"] = "active"
        self.env["CIEL_SYSTEM1_URL"] = server.url

        # Unmodeled action (passes deterministic regex)
        payload = json.dumps({
            "toolCall": {"name": "exec", "args": {"CommandLine": "wipe_database --force --all"}},
            "conversationId": "conv-unmodeled-ag",
        })

        rc, stdout, stderr = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        self.assertEqual(rc, 0)
        data = json.loads(stdout)

        # JSON contract checks
        self.assertEqual(data["decision"], "deny")
        self.assertIn("system1_semantic_risk", data["reason"])
        self.assertIn("flagged as dangerous (94% confidence)", data["reason"])

        # Audit logs check
        act_log = (self.ciel / "activity.log").read_text(encoding="utf-8")
        self.assertIn("PreToolUse+System1Intercept", act_log)
        self.assertIn('"risk": "critical"', act_log)

        s1_log = (self.ciel / "system1" / "events.jsonl").read_text(encoding="utf-8")
        rec = json.loads(s1_log.splitlines()[-1])
        self.assertEqual(rec["surface"], "pre_tool_risk")
        self.assertEqual(rec["flag"], "flag")
        self.assertEqual(rec["meta"]["pipeline"], "active")

    def test_02_active_intercept_unmodeled_action_devin(self):
        """Unmodeled destructive action is actively intercepted under Devin."""
        server = MockLayaServer({
            "answers": {
                "risk": {
                    "choice": "dangerous",
                    "confidence": 0.92,
                    "probabilities": {"dangerous": 0.92, "safe": 0.08},
                }
            },
            "model": "laya-slot1",
        })
        self.addCleanup(server.shutdown)

        self.env["CIEL_SYSTEM1_MODE"] = "active"
        self.env["CIEL_SYSTEM1_URL"] = server.url

        payload = json.dumps({
            "tool_name": "exec",
            "tool_input": {"command": "killall -9 python3"},
            "session_id": "sess-devin-1",
            "prompt_id": "p-1",
        })

        rc, stdout, stderr = self.run_rust("pretool", "--runtime", "devin", stdin=payload)
        self.assertEqual(rc, 0)
        data = json.loads(stdout)

        # Devin JSON contract: decision is 'block'
        self.assertEqual(data["decision"], "block")
        self.assertIn("system1_semantic_risk", data["reason"])
        self.assertIn("Run it manually outside the agent", data["reason"])

        act_log = (self.ciel / "activity.log").read_text(encoding="utf-8")
        self.assertIn("PreToolUse+System1Intercept", act_log)

    # -------------------------------------------------------------------------
    # Requirement 2: Fail-open behavior maintained when endpoint down (< 5ms)
    # -------------------------------------------------------------------------

    def test_03_fail_open_when_endpoint_down_antigravity(self):
        """When endpoint is offline, Antigravity fails open with pre-flight passed."""
        # Unbound port
        self.env["CIEL_SYSTEM1_MODE"] = "active"
        self.env["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:49151"

        payload = json.dumps({
            "toolCall": {"name": "exec", "args": {"CommandLine": "ls -la"}},
            "conversationId": "conv-failopen-ag",
        })

        t0 = time.perf_counter()
        rc, stdout, stderr = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        elapsed_ms = (time.perf_counter() - t0) * 1000

        self.assertEqual(rc, 0)
        data = json.loads(stdout)
        self.assertEqual(data["decision"], "allow")
        self.assertEqual(data["reason"], "Ciel pre-flight check passed.")

        # Ensure no shadow task or duplicate event was spawned
        s1_events = (self.ciel / "system1" / "events.jsonl")
        self.assertTrue(s1_events.exists())
        lines = s1_events.read_text(encoding="utf-8").strip().splitlines()
        self.assertEqual(len(lines), 1, "Expected exactly 1 active attempt record, no duplicate shadow records")
        rec = json.loads(lines[0])
        self.assertIsNone(rec["system1"])
        self.assertEqual(rec["flag"], "pass")

    def test_04_fail_open_when_endpoint_down_devin(self):
        """When endpoint is offline, Devin fails open silently (exit 0, empty stdout)."""
        self.env["CIEL_SYSTEM1_MODE"] = "active"
        self.env["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:49151"

        payload = json.dumps({
            "tool_name": "exec",
            "tool_input": {"command": "echo test"},
            "session_id": "sess-devin-failopen",
        })

        rc, stdout, stderr = self.run_rust("pretool", "--runtime", "devin", stdin=payload)
        self.assertEqual(rc, 0)
        self.assertEqual(stdout.strip(), "", "Devin must remain silent on allow")

    def test_05_internal_evaluate_risk_offline_latency_under_5ms(self):
        """Direct Rust unit test verification that evaluate_risk offline is < 5ms."""
        cmd = [
            "cargo",
            "test",
            "--",
            "--test-threads=1",
            "evaluate_risk_offline_latency_under_5ms",
        ]
        cargo_env = dict(os.environ)
        real_home = self._saved.get("HOME") or "/data/data/com.termux/files/home"
        cargo_env["HOME"] = real_home
        cargo_env["CARGO_HOME"] = f"{real_home}/.cargo"
        proc = subprocess.run(
            cmd,
            cwd=str(ROOT / "ciel.skill" / "init" / "ciel-rs"),
            capture_output=True,
            text=True,
            env=cargo_env,
        )
        self.assertEqual(proc.returncode, 0, f"Latency test failed: {proc.stdout}\n{proc.stderr}")
        self.assertIn("evaluate_risk_offline_latency_under_5ms ... ok", proc.stdout)

    # -------------------------------------------------------------------------
    # Requirement 3: allow_privileged override functions properly on S1 intercepts
    # -------------------------------------------------------------------------

    def test_06_allow_privileged_overrides_system1_intercept_antigravity(self):
        """allow_privileged permits unmodeled dangerous actions with override notice."""
        server = MockLayaServer({
            "answers": {
                "risk": {"choice": "dangerous", "confidence": 0.96}
            },
            "model": "laya-slot1",
        })
        self.addCleanup(server.shutdown)

        self.env["CIEL_SYSTEM1_MODE"] = "active"
        self.env["CIEL_SYSTEM1_URL"] = server.url
        (self.ciel / "allow_privileged").touch()

        payload = json.dumps({
            "toolCall": {"name": "exec", "args": {"CommandLine": "wipe_database --force"}},
            "conversationId": "conv-override-ag",
        })

        rc, stdout, stderr = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        self.assertEqual(rc, 0)
        data = json.loads(stdout)

        self.assertEqual(data["decision"], "allow")
        self.assertIn("System-1 semantic warning overridden by allow_privileged", data["reason"])

        act_log = (self.ciel / "activity.log").read_text(encoding="utf-8")
        self.assertIn("PreToolUse+System1Override", act_log)
        self.assertIn('"system1_override": true', act_log)

    def test_07_allow_privileged_overrides_system1_intercept_devin(self):
        """allow_privileged permits unmodeled dangerous actions silently on Devin."""
        server = MockLayaServer({
            "answers": {
                "risk": {"choice": "dangerous", "confidence": 0.96}
            },
            "model": "laya-slot1",
        })
        self.addCleanup(server.shutdown)

        self.env["CIEL_SYSTEM1_MODE"] = "active"
        self.env["CIEL_SYSTEM1_URL"] = server.url
        (self.ciel / "allow_privileged").touch()

        payload = json.dumps({
            "tool_name": "exec",
            "tool_input": {"command": "wipe_database --force"},
            "session_id": "sess-devin-override",
        })

        rc, stdout, stderr = self.run_rust("pretool", "--runtime", "devin", stdin=payload)
        self.assertEqual(rc, 0)
        self.assertEqual(stdout.strip(), "", "Devin must remain silent on overridden actions")

        act_log = (self.ciel / "activity.log").read_text(encoding="utf-8")
        self.assertIn("PreToolUse+System1Override", act_log)

    def test_08_allow_privileged_does_not_override_hard_regex_denials(self):
        """Hard-tier catastrophic actions (e.g. mkfs) remain blocked even with allow_privileged."""
        (self.ciel / "allow_privileged").touch()

        payload_ag = json.dumps({
            "toolCall": {"name": "exec", "args": {"CommandLine": "mkfs.ext4 /dev/sda1"}},
            "conversationId": "conv-hard-ag",
        })
        rc, stdout, _ = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload_ag)
        self.assertEqual(rc, 0)
        data_ag = json.loads(stdout)
        self.assertEqual(data_ag["decision"], "deny")
        self.assertIn("mkfs", data_ag["reason"])

        payload_devin = json.dumps({
            "tool_name": "exec",
            "tool_input": {"command": "mkfs.ext4 /dev/sda1"},
            "session_id": "sess-hard-devin",
        })
        rc, stdout, _ = self.run_rust("pretool", "--runtime", "devin", stdin=payload_devin)
        self.assertEqual(rc, 0)
        data_devin = json.loads(stdout)
        self.assertEqual(data_devin["decision"], "block")
        self.assertIn("mkfs", data_devin["reason"])

    # -------------------------------------------------------------------------
    # Requirement 4: Strict JSON contract verification across all branches
    # -------------------------------------------------------------------------

    def test_09_full_json_contract_antigravity(self):
        """Antigravity must emit valid single-line JSON with decision and reason."""
        cases = [
            # Benign
            ({"toolCall": {"name": "read", "args": {"path": "/tmp/a.txt"}}}, "allow", "pre-flight check passed"),
            # Regex hard deny
            ({"toolCall": {"name": "exec", "args": {"command": "rm -rf /"}}}, "deny", "Ciel safety gate [rm_root_or_home]"),
        ]
        for payload_dict, expected_decision, reason_substr in cases:
            rc, stdout, stderr = self.run_rust(
                "pretool", "--runtime", "antigravity", stdin=json.dumps(payload_dict)
            )
            self.assertEqual(rc, 0)
            lines = stdout.strip().splitlines()
            self.assertEqual(len(lines), 1, "Must be exactly single-line JSON")
            data = json.loads(lines[0])
            self.assertEqual(data.get("decision"), expected_decision)
            self.assertIn(reason_substr, data.get("reason", ""))

    def test_10_full_json_contract_devin(self):
        """Devin must emit block JSON on denies, and silence on allows."""
        # 1. Block case
        payload_block = json.dumps({
            "tool_name": "exec",
            "tool_input": {"command": "rm -rf /"},
        })
        rc, stdout, _ = self.run_rust("pretool", "--runtime", "devin", stdin=payload_block)
        self.assertEqual(rc, 0)
        lines = stdout.strip().splitlines()
        self.assertEqual(len(lines), 1)
        data = json.loads(lines[0])
        self.assertEqual(data.get("decision"), "block")
        self.assertIn("rm_root_or_home", data.get("reason", ""))

        # 2. Allow case
        payload_allow = json.dumps({
            "tool_name": "read",
            "tool_input": {"path": "/tmp/b.txt"},
        })
        rc, stdout, _ = self.run_rust("pretool", "--runtime", "devin", stdin=payload_allow)
        self.assertEqual(rc, 0)
        self.assertEqual(stdout.strip(), "")


if __name__ == "__main__":
    unittest.main()
