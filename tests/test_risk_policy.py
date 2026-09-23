"""Unit tests for hooks/lib/risk_policy.py and the policy compile step."""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "ciel.skill" / "init" / "hooks" / "lib"))

import risk_policy

POLICY_YAML = ROOT / "ciel.skill" / "risk" / "policy.yaml"
POLICY_JSON = ROOT / "ciel.skill" / "risk" / "policy.json"


class TestPolicyLoading(unittest.TestCase):
    def test_policy_file_resolves_via_ancestors(self):
        rules, source = risk_policy.load_policy()
        self.assertEqual("file", source)
        self.assertGreaterEqual(len(rules), 10)

    def test_fallback_rules_when_policy_missing(self):
        original = risk_policy._candidate_policy_files
        risk_policy._candidate_policy_files = list
        try:
            rules, source = risk_policy.load_policy()
        finally:
            risk_policy._candidate_policy_files = original
        self.assertEqual("fallback", source)
        self.assertTrue(all(r["tier"] == "hard" for r in rules))


class TestEvaluate(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.home = Path(self.tmp.name)
        self._old_home = os.environ.get("CIEL_HOME")
        os.environ["CIEL_HOME"] = str(self.home / ".ciel")

    def tearDown(self):
        if self._old_home is None:
            os.environ.pop("CIEL_HOME", None)
        else:
            os.environ["CIEL_HOME"] = self._old_home
        self.tmp.cleanup()

    def _eval(self, **kwargs):
        return risk_policy.evaluate(home=self.home, **kwargs)

    def test_benign_allows(self):
        self.assertEqual("allow", self._eval(tool="exec", command="ls -la")["decision"])
        self.assertEqual("allow", self._eval(tool="exec", command="cat sudoers")["decision"])
        self.assertEqual("allow", self._eval(tool="exec", command="grep sudo")["decision"])

    def test_soft_deny_without_override(self):
        v = self._eval(tool="exec", command="sudo apt update")
        self.assertEqual("deny", v["decision"])
        self.assertEqual("privilege_escalation", v["rule_id"])
        self.assertEqual("soft", v["tier"])

    def test_soft_override_allows(self):
        (self.home / ".ciel").mkdir(parents=True)
        (self.home / ".ciel" / "allow_privileged").touch()
        v = self._eval(tool="exec", command="sudo apt update")
        self.assertEqual("allow_overridden", v["decision"])

    def test_hard_deny_beats_override(self):
        (self.home / ".ciel").mkdir(parents=True)
        (self.home / ".ciel" / "allow_privileged").touch()
        for cmd in ("rm -rf /", ":(){ :|:& };:", "mkfs.ext4 /dev/sda1", "dd if=x of=/dev/sda"):
            v = self._eval(tool="exec", command=cmd)
            self.assertEqual("deny", v["decision"], cmd)
            self.assertEqual("hard", v["tier"], cmd)

    def test_path_rules_only_apply_to_matching_tools(self):
        denied = self._eval(tool="write", path="/etc/hosts")
        self.assertEqual("deny", denied["decision"])
        allowed = self._eval(tool="read", path="/etc/hosts")
        self.assertEqual("allow", allowed["decision"])

    def test_home_paths_normalize_to_tilde(self):
        for raw in (str(self.home / ".ssh" / "id"), "$HOME/.ssh/id", "~/.ssh/id"):
            v = self._eval(tool="write", path=raw)
            self.assertEqual("deny", v["decision"], raw)
            self.assertEqual("secret_store_write", v["rule_id"], raw)

    def test_hook_self_tamper(self):
        v = self._eval(tool="edit", path="~/.ciel/risk/policy.yaml")
        self.assertEqual("deny", v["decision"])
        self.assertEqual("hook_self_tamper", v["rule_id"])


class TestPolicySources(unittest.TestCase):
    """Policy loading via $CIEL_POLICY and the compiled fallback path."""

    def test_yaml_source_loads(self):
        try:
            import yaml  # noqa: F401
        except ImportError:
            self.skipTest("PyYAML not installed")
        env = dict(os.environ, CIEL_POLICY=str(POLICY_YAML))
        proc = subprocess.run(
            [sys.executable, str(ROOT / "ciel.skill" / "init" / "hooks" / "lib" / "risk_policy.py"), "--check"],
            capture_output=True, text=True, env=env, check=False,
        )
        self.assertEqual(0, proc.returncode, proc.stderr)
        self.assertIn("source=file", proc.stdout)

    def test_corrupt_json_falls_through(self):
        with tempfile.TemporaryDirectory() as tmp:
            bad = Path(tmp) / "policy.json"
            bad.write_text("{corrupt")
            env = dict(os.environ, CIEL_POLICY=str(bad))
            proc = subprocess.run(
                [sys.executable, str(ROOT / "ciel.skill" / "init" / "hooks" / "lib" / "risk_policy.py"), "--check"],
                capture_output=True, text=True, env=env, check=False,
            )
            # corrupt explicit file falls through to the ancestor-found repo policy
            self.assertIn("source=file", proc.stdout)

    def test_cli_stdin_verdict(self):
        proc = subprocess.run(
            [sys.executable, str(ROOT / "ciel.skill" / "init" / "hooks" / "lib" / "risk_policy.py")],
            input='{"tool":"exec","command":"mkfs.ext4 /dev/sda1"}',
            capture_output=True, text=True, check=False,
        )
        verdict = json.loads(proc.stdout)
        self.assertEqual("deny", verdict["decision"])
        self.assertEqual("mkfs", verdict["rule_id"])

    def test_cli_bad_json_payload(self):
        proc = subprocess.run(
            [sys.executable, str(ROOT / "ciel.skill" / "init" / "hooks" / "lib" / "risk_policy.py")],
            input="{not json",
            capture_output=True, text=True, check=False,
        )
        self.assertEqual("allow", json.loads(proc.stdout)["decision"])


class TestEvaluateBranches(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.home = Path(self.tmp.name)
        self._old_home = os.environ.get("CIEL_HOME")
        os.environ["CIEL_HOME"] = str(self.home / ".ciel")

    def tearDown(self):
        if self._old_home is None:
            os.environ.pop("CIEL_HOME", None)
        else:
            os.environ["CIEL_HOME"] = self._old_home

    def test_explicit_rules_skip_file_load(self):
        rules = [{"id": "x", "tier": "soft", "match": "command",
                  "pattern": "danger", "reason": "r"}]
        v = risk_policy.evaluate(tool="exec", command="danger now",
                                 home=self.home, rules=rules)
        self.assertEqual("deny", v["decision"])

    def test_invalid_regex_skipped(self):
        rules = [{"id": "bad", "tier": "hard", "match": "command",
                  "pattern": "([", "reason": "broken"}]
        v = risk_policy.evaluate(tool="exec", command="anything",
                                 home=self.home, rules=rules)
        self.assertEqual("allow", v["decision"])

    def test_empty_subject_skipped(self):
        rules = [{"id": "x", "tier": "hard", "match": "path",
                  "pattern": ".*", "reason": "match-all"}]
        v = risk_policy.evaluate(tool="write", path="", home=self.home, rules=rules)
        self.assertEqual("allow", v["decision"])

    def test_tool_matcher_misses(self):
        rules = [{"id": "x", "tier": "hard", "match": "command",
                  "pattern": "danger", "reason": "r", "tools": ["^write$"]}]
        v = risk_policy.evaluate(tool="exec", command="danger",
                                 home=self.home, rules=rules)
        self.assertEqual("allow", v["decision"])

    def test_normalize_empty_and_home_root(self):
        self.assertEqual("", risk_policy._normalize_path("", self.home))
        self.assertEqual("~", risk_policy._normalize_path(str(self.home), self.home))


class TestSystem1Shadow(unittest.TestCase):
    """The shadow tier must never influence decisions and must degrade silently."""

    def setUp(self):
        self._saved = {k: os.environ.get(k) for k in
                       ("CIEL_SYSTEM1_DISABLED", "CIEL_SYSTEM1_URL", "CIEL_SYSTEM1_KEY")}

    def tearDown(self):
        for k, v in self._saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v

    def test_disabled_returns_none(self):
        os.environ["CIEL_SYSTEM1_DISABLED"] = "1"
        self.assertIsNone(risk_policy.system1_verdict("exec", "rm -rf /", ""))

    def test_unreachable_returns_none_fast(self):
        os.environ.pop("CIEL_SYSTEM1_DISABLED", None)
        os.environ["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:9"  # discard port
        import time
        t0 = time.monotonic()
        self.assertIsNone(risk_policy.system1_verdict("exec", "ls", ""))
        self.assertLess(time.monotonic() - t0, 5)

    def test_stubbed_server_returns_verdict(self):
        import http.server
        import threading

        class H(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                self.send_response(200)
                self.send_header("content-type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({
                    "answers": {"risk": {"choice": "dangerous", "confidence": 0.9}},
                    "routing": {"model": "english"},
                }).encode())

            def log_message(self, *a):
                pass

        srv = http.server.HTTPServer(("127.0.0.1", 0), H)
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        self.addCleanup(srv.shutdown)
        os.environ.pop("CIEL_SYSTEM1_DISABLED", None)
        os.environ["CIEL_SYSTEM1_URL"] = f"http://127.0.0.1:{srv.server_port}"
        os.environ["CIEL_SYSTEM1_KEY"] = "test-key"
        v = risk_policy.system1_verdict("exec", "rm -rf /", "")
        self.assertEqual("dangerous", v["choice"])
        self.assertEqual(0.9, v["confidence"])
        self.assertEqual("english", v["model"])

    def test_shadow_does_not_change_decision(self):
        # Even if system1 would say 'dangerous', a benign command still allows.
        os.environ["CIEL_SYSTEM1_DISABLED"] = "1"
        v = risk_policy.evaluate(tool="exec", command="ls -la", home=Path(tempfile.mkdtemp()))
        self.assertEqual("allow", v["decision"])

    def test_shadow_main_writes_log(self):
        import http.server
        import threading

        class H(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                self.send_response(200)
                self.send_header("content-type", "application/json")
                self.end_headers()
                self.wfile.write(json.dumps({
                    "answers": {"risk": {"choice": "safe", "confidence": 0.5}},
                }).encode())

            def log_message(self, *a):
                pass

        srv = http.server.HTTPServer(("127.0.0.1", 0), H)
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        self.addCleanup(srv.shutdown)

        with tempfile.TemporaryDirectory() as tmp:
            env = dict(os.environ,
                       CIEL_HOME=tmp,
                       CIEL_SYSTEM1_URL=f"http://127.0.0.1:{srv.server_port}",
                       CIEL_SYSTEM1_KEY="k")
            env.pop("CIEL_SYSTEM1_DISABLED", None)
            proc = subprocess.run(
                [sys.executable,
                 str(ROOT / "ciel.skill" / "init" / "hooks" / "lib" / "risk_policy.py"),
                 "--shadow"],
                input='{"ts":"t1","tool":"exec","command":"ls"}',
                capture_output=True, text=True, env=env, check=False,
            )
            self.assertEqual(0, proc.returncode)
            log = Path(tmp) / "system1" / "events.jsonl"
            self.assertTrue(log.is_file())
            rec = json.loads(log.read_text().splitlines()[0])
            self.assertEqual("t1", rec["ts"])
            self.assertEqual("pre_tool_risk", rec["surface"])
            self.assertEqual(
                "safe", rec["system1"]["answers"]["risk"]["choice"])


@unittest.skipUnless(POLICY_JSON.is_file(), "policy.json not compiled")
class TestPolicySync(unittest.TestCase):
    def test_json_matches_yaml(self):
        try:
            import yaml
        except ImportError:
            self.skipTest("PyYAML not installed")
        data = yaml.safe_load(POLICY_YAML.read_text(encoding="utf-8"))
        rendered = json.dumps(data, indent=2, ensure_ascii=False) + "\n"
        self.assertEqual(rendered, POLICY_JSON.read_text(encoding="utf-8"),
                         "policy.json stale; run scripts/compile_policy.py")

    def test_every_rule_has_required_fields(self):
        data = json.loads(POLICY_JSON.read_text(encoding="utf-8"))
        ids = set()
        for rule in data["rules"]:
            self.assertIn(rule["tier"], {"hard", "soft"}, rule["id"])
            self.assertIn(rule["match"], {"command", "path"}, rule["id"])
            self.assertTrue(rule["pattern"], rule["id"])
            self.assertTrue(rule["reason"], rule["id"])
            self.assertNotIn(rule["id"], ids, "duplicate rule id")
            ids.add(rule["id"])


class TestCompilePolicy(unittest.TestCase):
    SCRIPT = ROOT / "scripts" / "compile_policy.py"

    def setUp(self):
        try:
            import yaml  # noqa: F401
        except ImportError:
            self.skipTest("PyYAML not installed")

    def test_check_passes(self):
        proc = subprocess.run([sys.executable, str(self.SCRIPT), "--check"],
                              capture_output=True, text=True, check=False)
        self.assertEqual(0, proc.returncode, proc.stdout + proc.stderr)

    def test_write_is_idempotent(self):
        before = POLICY_JSON.read_text(encoding="utf-8")
        proc = subprocess.run([sys.executable, str(self.SCRIPT)],
                              capture_output=True, text=True, check=False)
        self.assertEqual(0, proc.returncode, proc.stdout + proc.stderr)
        self.assertEqual(before, POLICY_JSON.read_text(encoding="utf-8"))


if __name__ == "__main__":
    unittest.main()
