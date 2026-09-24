#!/usr/bin/env python3
"""Rust↔Python differential parity tests.

Feeds identical inputs through `hooks/lib/risk_policy.py` and the `ciel`
Rust binary, asserting byte-identical verdicts/results. Skips cleanly when
the binary is absent so the Python suite stays green on hosts without Rust.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "ciel.skill" / "init" / "hooks" / "lib"
CIEL_BIN = os.environ.get(
    "CIEL_RS_BIN",
    str(ROOT / "ciel.skill" / "init" / "ciel-rs" / "target" / "debug" / "ciel"),
)

sys.path.insert(0, str(LIB))
import risk_policy  # noqa: E402
import secret_scan  # noqa: E402
import attribution_scan  # noqa: E402

FIXTURES = json.loads(
    (ROOT / "tests" / "fixtures" / "hook_redteam_cases.json").read_text()
)["cases"]


def run_rust(*args: str, stdin: str = "", env: dict | None = None):
    proc = subprocess.run(
        [CIEL_BIN, *args], input=stdin, capture_output=True, text=True,
        env=env,
    )
    return proc.returncode, proc.stdout


@unittest.skipUnless(
    Path(CIEL_BIN).exists(), f"ciel binary not built at {CIEL_BIN}"
)
class TestRiskEvalParity(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="ciel-parity-"))
        self.ciel = self.tmp / ".ciel"
        self.ciel.mkdir(parents=True)
        self._saved = {k: os.environ.get(k) for k in ("CIEL_HOME", "HOME")}
        os.environ["CIEL_HOME"] = str(self.ciel)
        os.environ["HOME"] = str(self.tmp)
        self.env = dict(os.environ)

    def tearDown(self):
        for k, v in self._saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _pair(self, case, grant: bool):
        sentinel = self.ciel / "allow_privileged"
        if grant:
            sentinel.touch()
        elif sentinel.exists():
            sentinel.unlink()
        py = risk_policy.evaluate(
            case.get("tool", "exec"),
            case.get("command", ""),
            case.get("path", ""),
        )
        rc, out = run_rust(
            "risk-eval",
            stdin=json.dumps({
                "tool": case.get("tool", "exec"),
                "command": case.get("command", ""),
                "path": case.get("path", ""),
            }),
            env=self.env,
        )
        self.assertEqual(0, rc, case["id"])
        rs = json.loads(out)
        return py, rs

    def test_redteam_corpus_no_override(self):
        for case in FIXTURES:
            with self.subTest(case=case["id"]):
                py, rs = self._pair(case, grant=False)
                self.assertEqual(py, rs)

    def test_redteam_corpus_with_override(self):
        for case in FIXTURES:
            with self.subTest(case=case["id"]):
                py, rs = self._pair(case, grant=True)
                self.assertEqual(py, rs)

    def test_expected_decisions_also_hold(self):
        """Parity is necessary but not sufficient — the Rust verdict must
        also satisfy the corpus's expected outcome (python already does)."""
        for case in FIXTURES:
            with self.subTest(case=case["id"]):
                _, rs = self._pair(case, grant=bool(case.get("override")))
                expect = case["expect"]
                if expect == "deny":
                    self.assertEqual("deny", rs["decision"], case["id"])
                elif expect == "allow":
                    self.assertIn(
                        rs["decision"], ("allow", "allow_overridden"),
                        case["id"],
                    )

    def test_path_normalization_pairs(self):
        home = str(self.tmp)
        forms = [
            f"{home}/.ciel//risk//policy.json",
            f"{home}/.ciel/./allow_privileged",
            "~/./.ciel/risk",
            f"{home}/x/../.ciel/hooks",
            f"{home}//.ssh//id_rsa",
        ]
        for raw in forms:
            with self.subTest(path=raw):
                py = risk_policy.evaluate("write", "", raw)
                rc, out = run_rust(
                    "risk-eval",
                    stdin=json.dumps(
                        {"tool": "write", "command": "", "path": raw}),
                    env=self.env,
                )
                self.assertEqual(py, json.loads(out), raw)

    def test_malformed_json_stdin(self):
        rc, out = run_rust("risk-eval", stdin="{not json", env=self.env)
        self.assertEqual(0, rc)
        self.assertEqual("allow", json.loads(out)["decision"])

    def test_fallback_policy_source(self):
        env = dict(self.env)
        env["CIEL_POLICY"] = str(self.tmp / "nonexistent.json")
        env["HOME"] = str(self.tmp / "no-such-home")
        py = risk_policy.evaluate("exec", "mkfs /dev/sda", "")
        rc, out = run_rust(
            "risk-eval",
            stdin=json.dumps(
                {"tool": "exec", "command": "mkfs /dev/sda", "path": ""}),
            env=env,
        )
        rs = json.loads(out)
        self.assertEqual(py["decision"], rs["decision"])
        self.assertEqual(py["policy"], rs["policy"])


@unittest.skipUnless(
    Path(CIEL_BIN).exists(), f"ciel binary not built at {CIEL_BIN}"
)
class TestSecretScanParity(unittest.TestCase):
    SAMPLES = [
        "nothing sensitive here",
        "token is ghp_abcdefghij0123456789ABCD done",
        "aws key AKIAIOSFODNN7EXAMPLE in text",
        "-----BEGIN RSA PRIVATE KEY-----\nMII...",
        "xoxb-1234567890-abcdefghijkl",
        "npm_0123456789abcdefghijklmnopqrstuvwxyz",
        "cio3dhjRANDOMSTRINGFORTESTING12345",
        'password = "hunter2"',
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U",
        "multi\nline\ntext with sk-proj-abcdefghijklmnopqrstuvwxyz",
        "my api_key = AbCdEfGh12345678 ok",
        "the secret is Sup3rSecretValue99",
    ]

    def test_samples(self):
        for text in self.SAMPLES:
            with self.subTest(text=text[:40]):
                py = secret_scan.scan(text)
                rc, out = run_rust("secret-scan", stdin=text)
                self.assertEqual(0, rc)
                self.assertEqual(py, json.loads(out))


@unittest.skipUnless(
    Path(CIEL_BIN).exists(), f"ciel binary not built at {CIEL_BIN}"
)
class TestAttributionScanParity(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="ciel-attr-"))
        self._saved = os.environ.get("CIEL_HOME")
        os.environ["CIEL_HOME"] = str(self.tmp / ".ciel")
        self.env = dict(os.environ)

    def tearDown(self):
        if self._saved is None:
            os.environ.pop("CIEL_HOME", None)
        else:
            os.environ["CIEL_HOME"] = self._saved
        shutil.rmtree(self.tmp, ignore_errors=True)

    def test_command_surface(self):
        samples = [
            "git commit -m 'Generated with Foo'",
            "git commit -m 'Co-Authored-By: bot <b@c>'",
            "git commit -m 'ordinary message'",
            "echo «Report» shipped",
            "deploy 🚀 now",
            "devin did this",
            "ls -la",
        ]
        for cmd in samples:
            with self.subTest(cmd=cmd[:40]):
                py = attribution_scan.scan(cmd)
                rc, out = run_rust("attribution-scan", stdin=cmd, env=self.env)
                self.assertEqual(0, rc)
                rs = json.loads(out)
                self.assertEqual(py["result"], rs["result"], cmd)
                self.assertEqual(
                    [f["category"] for f in py["findings"]],
                    [f["category"] for f in rs["findings"]],
                    cmd,
                )

    def test_bypass_env(self):
        env = dict(self.env)
        env["CIEL_ATTRIBUTION_SKIP"] = "1"
        os.environ["CIEL_ATTRIBUTION_SKIP"] = "1"
        try:
            py = attribution_scan.scan("git commit -m 'Generated with x'")
        finally:
            os.environ.pop("CIEL_ATTRIBUTION_SKIP", None)
        rc, out = run_rust(
            "attribution-scan", stdin="git commit -m 'Generated with x'",
            env=env)
        self.assertEqual(py["result"], json.loads(out)["result"])


@unittest.skipUnless(
    Path(CIEL_BIN).exists(), f"ciel binary not built at {CIEL_BIN}"
)
class TestCliContract(unittest.TestCase):
    def test_unknown_subcommand_exits_2(self):
        rc, _ = run_rust("nonsense")
        self.assertEqual(2, rc)

    def test_no_args_prints_usage_ok(self):
        proc = subprocess.run([CIEL_BIN], capture_output=True, text=True)
        self.assertEqual(0, proc.returncode)
        self.assertIn("USAGE", proc.stdout)


if __name__ == "__main__":
    unittest.main()
