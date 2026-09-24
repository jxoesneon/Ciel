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

LIB_DIR = LIB
STORE_PERMS = LIB / "store_perms.py"
WATCHDOG = LIB / "session_watchdog.py"
REQUIREMENTS = LIB / "requirements.py"
LOG_ROTATE = LIB / "activity_log_rotate.py"

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
class TestSessionOpsParity(unittest.TestCase):
    """Parity for the consolidated session-start batch: perms sweep,
    ledger, watchdog check, log rotation."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="ciel-sess-"))
        self.ciel = self.tmp / ".ciel"
        for d in (self.ciel, self.ciel / "checkpoints",
                  self.ciel / "system1", self.ciel / "improvements",
                  self.ciel / "logs", self.ciel / "backups",
                  self.ciel / "archive"):
            d.mkdir(parents=True, exist_ok=True)
            d.chmod(0o700)
        self.env = dict(os.environ)
        self.env["HOME"] = str(self.tmp)
        self.env["CIEL_HOME"] = str(self.ciel)
        self.env.pop("CIEL_BIN", None)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _py(self, script: Path, *args: str):
        proc = subprocess.run(
            ["python3", str(script), *args],
            capture_output=True, text=True, env=self.env, timeout=60,
        )
        return proc.returncode, proc.stdout

    def test_store_perms_clean(self):
        rc_py, out_py = self._py(STORE_PERMS)
        rc_rs, out_rs = run_rust("store-perms", env=self.env)
        self.assertEqual((rc_py, out_py), (rc_rs, out_rs))

    def _drift_perms(self):
        """Loosen every dir/file under the sandbox so a sweep has work."""
        for p in [self.ciel, *self.ciel.rglob("*")]:
            p.chmod(0o755 if p.is_dir() else 0o644)

    def test_store_perms_repairs(self):
        loose = self.ciel / "activity.log"
        loose.write_text("{}\n")
        self._drift_perms()
        rc_py, out_py = self._py(STORE_PERMS)
        # reset identical drift so the rust run sees the same world
        self._drift_perms()
        rc_rs, out_rs = run_rust("store-perms", env=self.env)
        self.assertEqual(rc_py, rc_rs)
        self.assertEqual(out_py, out_rs)
        self.assertTrue(out_rs.startswith("repaired:"))
        self.assertEqual(0o600, loose.stat().st_mode & 0o777)
        self.assertEqual(0o700, (self.ciel / "logs").stat().st_mode & 0o777)

    def test_ledger_add_pending_done(self):
        # shared-ledger sequence: both engines see identical event flow
        _, rid_py = self._py(REQUIREMENTS, "add", "write tests", "--session", "s1")
        rid_py = rid_py.strip()
        _, rid_rs = run_rust("ledger", "add", "deploy it", "--session", "s2",
                             env=self.env)
        rid_rs = rid_rs.strip()
        _, pend_py = self._py(REQUIREMENTS, "pending")
        _, pend_rs = run_rust("ledger", "pending", env=self.env)
        self.assertEqual(pend_py, pend_rs)
        _, done_py = self._py(REQUIREMENTS, "done", rid_py)
        _, pend_rs2 = run_rust("ledger", "pending", env=self.env)
        _, pend_py2 = self._py(REQUIREMENTS, "pending")
        self.assertEqual(pend_py2, pend_rs2)
        self.assertIn("resolved", done_py)
        _, list_rs = run_rust("ledger", "list", "--session", "s2", env=self.env)
        self.assertIn(rid_rs, list_rs)

    def test_watchdog_check_clean(self):
        # no ledger items, no transcripts → both engines print nothing
        rc_py, out_py = self._py(WATCHDOG, "--check")
        rc_rs, out_rs = run_rust("watchdog", env=self.env)
        self.assertEqual(rc_py, rc_rs)
        self.assertEqual(out_py, out_rs)

    def test_watchdog_check_stalled_hint(self):
        # seed a pending ledger item from a dead session + activity line
        ledger = self.ciel / "checkpoints" / "requirements.jsonl"
        ledger.write_text(json.dumps({
            "op": "add", "id": "req-1", "text": "finish the thing",
            "session": "deadbeef-dead-session", "ts": "2020-01-01T00:00:00+00:00",
        }) + "\n")
        rc_py, out_py = self._py(WATCHDOG, "--check")
        # rust sees the same world (state file now written by python —
        # reset so both evaluate identical inputs)
        (self.ciel / "checkpoints" / "watchdog_state.json").unlink(
            missing_ok=True)
        (self.ciel / "checkpoints" / "resume_hint.json").unlink(
            missing_ok=True)
        rc_rs, out_rs = run_rust("watchdog", env=self.env)
        self.assertEqual(rc_py, rc_rs)
        self.assertEqual(out_py, out_rs)
        self.assertIn("deadbeef", out_rs)
        self.assertIn("unresolved ledger item", out_rs)

    def test_watchdog_check_secret_sweep(self):
        transcripts = (self.tmp / ".local" / "share" / "devin" / "cli"
                       / "transcripts")
        transcripts.mkdir(parents=True)
        (transcripts / "t1.json").write_text(
            '["msg ghp_abcdefghij0123456789ABCD end"]')
        rc_py, out_py = self._py(WATCHDOG, "--check")
        for f in (self.ciel / "checkpoints" / "watchdog_state.json",
                  self.ciel / "checkpoints" / "resume_hint.json"):
            f.unlink(missing_ok=True)
        rc_rs, out_rs = run_rust("watchdog", env=self.env)
        self.assertEqual(out_py, out_rs)
        self.assertIn("secret", out_rs.lower())

    def test_log_rotate_daily(self):
        log = self.ciel / "activity.log"
        log.write_text(
            json.dumps({"ts": "2020-01-01T00:00:00+00:00",
                        "event": "old"}) + "\n")
        env_py = dict(self.env)
        rc_py, out_py = self._py(LOG_ROTATE)
        archives = list((self.ciel / "archive" / "logs").glob("activity-*"))
        self.assertEqual(1, len(archives))
        # reset identical world for rust
        for a in archives:
            a.unlink()
        log.write_text(
            json.dumps({"ts": "2020-01-01T00:00:00+00:00",
                        "event": "old"}) + "\n")
        rc_rs, out_rs = run_rust("log-rotate", env=self.env)
        archives_rs = list((self.ciel / "archive" / "logs").glob("activity-*"))
        self.assertEqual(1, len(archives_rs))
        self.assertTrue(
            archives_rs[0].name.endswith((".zst", ".gz")),
            archives_rs[0].name)
        marker = json.loads(log.read_text().strip().splitlines()[-1])
        self.assertEqual("log_rotate", marker["op"])
        self.assertEqual("daily", marker["reason"])

    def test_log_rotate_none_needed(self):
        log = self.ciel / "activity.log"
        log.write_text(
            json.dumps({"ts": "2999-01-01T00:00:00+00:00"}) + "\n")
        rc_rs, out_rs = run_rust("log-rotate", env=self.env)
        self.assertEqual(0, rc_rs)
        self.assertFalse((self.ciel / "archive" / "logs").exists())

    def test_session_start_json_shape(self):
        rc, out = run_rust("session-start", "--runtime", "devin",
                           env=self.env)
        self.assertEqual(0, rc)
        payload = json.loads(out)
        ctx = payload["hookSpecificOutput"]["additionalContext"]
        self.assertIn("Ciel is installed and active", ctx)
        self.assertIn("Master", ctx)
        self.assertIn("NO AI ATTRIBUTION", ctx)
        # no config in the sandbox → the could-not-verify note
        self.assertIn("config-absent", ctx)
        # and repaired when present-but-wrong
        cfg_dir = self.tmp / ".config" / "devin"
        cfg_dir.mkdir(parents=True)
        (cfg_dir / "config.json").write_text('{"attribution": true}')
        rc, out = run_rust("session-start", "--runtime", "devin",
                           env=self.env)
        payload = json.loads(out)
        self.assertIn(
            "reset to false",
            payload["hookSpecificOutput"]["additionalContext"])
        self.assertEqual(
            {"attribution": False},
            json.loads((cfg_dir / "config.json").read_text()))

    def test_session_start_antigravity(self):
        rc, out = run_rust("session-start", "--runtime", "antigravity",
                           env=self.env)
        self.assertEqual(0, rc)
        payload = json.loads(out)
        self.assertIn("Antigravity",
                      payload["injectSteps"][0]["ephemeralMessage"])

    def test_grant_state_parity(self):
        (self.ciel / "allow_privileged").touch()
        rc, out = run_rust("grant-state", env=self.env)
        rs = json.loads(out)
        self.assertTrue(rs["active"])
        self.assertIn(str(self.ciel / "allow_privileged"), rs["sentinel"])


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
