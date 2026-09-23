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

import risk_policy  # noqa: E402

POLICY_YAML = ROOT / "ciel.skill" / "risk" / "policy.yaml"
POLICY_JSON = ROOT / "ciel.skill" / "risk" / "policy.json"


class TestPolicyLoading(unittest.TestCase):
    def test_policy_file_resolves_via_ancestors(self):
        rules, source = risk_policy.load_policy()
        self.assertEqual("file", source)
        self.assertGreaterEqual(len(rules), 10)

    def test_fallback_rules_when_policy_missing(self):
        original = risk_policy._candidate_policy_files
        risk_policy._candidate_policy_files = lambda: []
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


if __name__ == "__main__":
    unittest.main()
