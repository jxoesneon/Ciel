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


SCRIPTS = ROOT / "scripts"
SANITIZE_PY = SCRIPTS / "transcript_sanitize.py"
COMPILE_POLICY_PY = SCRIPTS / "compile_policy.py"
COUNCIL_VERIFY_PY = SCRIPTS / "council_verify.py"


@unittest.skipUnless(
    Path(CIEL_BIN).exists(), f"ciel binary not built at {CIEL_BIN}"
)
class TestOperatorParity(unittest.TestCase):
    """Differential parity for the Phase-3 operator ports."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="ciel-op-"))
        self.ciel = self.tmp / ".ciel"
        (self.ciel / "checkpoints").mkdir(parents=True)
        (self.ciel / "improvements" / "signals").mkdir(parents=True)
        self.env = dict(os.environ)
        self.env["HOME"] = str(self.tmp)
        self.env["CIEL_HOME"] = str(self.ciel)
        self.env.pop("CIEL_BIN", None)
        self.env.pop("CIEL_SESSIONS_DB", None)

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _py(self, script: Path, *args: str):
        proc = subprocess.run(
            ["python3", str(script), *args],
            capture_output=True, text=True, env=self.env, timeout=120,
        )
        return proc.returncode, proc.stdout

    # ------------------------------------------------------- compile-policy

    def test_compile_policy_check_parity(self):
        rc_py, out_py = self._py(COMPILE_POLICY_PY, "--check")
        rc_rs, out_rs = run_rust("compile-policy", "--check", env=self.env)
        self.assertEqual(rc_py, rc_rs)
        self.assertEqual(out_py, out_rs)

    def test_compile_policy_write_byte_exact(self):
        # python --check proves the repo twin was rendered by json.dumps;
        # rust renders the same yaml into a scratch dir — bytes must match
        src = ROOT / "ciel.skill" / "risk"
        rc_py, _ = self._py(COMPILE_POLICY_PY, "--check")
        self.assertEqual(0, rc_py)
        risk_rs = self.tmp / "risk-rs"
        shutil.copytree(src, risk_rs)
        (risk_rs / "policy.json").unlink()
        env_rs = dict(self.env, CIEL_POLICY_DIR=str(risk_rs))
        rc_rs, out_rs = run_rust("compile-policy", env=env_rs)
        self.assertEqual(0, rc_rs)
        self.assertIn("rules)", out_rs)
        self.assertEqual(
            (src / "policy.json").read_text(),
            (risk_rs / "policy.json").read_text())

    def _py_env(self, script: Path, env: dict, *args: str):
        proc = subprocess.run(
            ["python3", str(script), *args],
            capture_output=True, text=True, env=env, timeout=120,
        )
        return proc.returncode, proc.stdout

    # ------------------------------------------------------- council-verify

    def _make_run(self, verdict="pass", safety=7, mode="subagent",
                  missing=()) -> Path:
        run = self.tmp / "council" / "council-test-run"
        (run / "members").mkdir(parents=True)
        (run / "spawn_receipts.json").write_text(
            json.dumps({"mode": mode, "members": {}}))
        members = ["coherence", "capability", "safety",
                   "efficiency", "evolution"]
        for m in members:
            for st in (1, 2):
                if f"{m}.stage{st}" in missing:
                    continue
                (run / "members" / f"{m}.stage{st}.json").write_text(
                    json.dumps({"member": m, "stage": st,
                                "score": safety if m == "safety" else 7}))
        (run / "verdict.json").write_text(json.dumps({
            "verdict": verdict,
            "votes": {m: (safety if m == "safety" else 7) for m in members},
        }))
        return run

    def _verify_both(self, run: Path):
        rc_py, out_py = self._py(COUNCIL_VERIFY_PY, str(run))
        # remove the python-emitted signal so rust sees the same world
        for s in (self.ciel / "improvements" / "signals").glob("council-*"):
            s.unlink()
        rc_rs, out_rs = run_rust("council-verify", str(run), env=self.env)
        return (rc_py, out_py), (rc_rs, out_rs)

    def test_council_verify_pass(self):
        (rc_py, out_py), (rc_rs, out_rs) = self._verify_both(self._make_run())
        self.assertEqual(rc_py, rc_rs)
        self.assertEqual(out_py, out_rs)
        self.assertEqual(0, rc_rs)
        self.assertIn("VERIFIED", out_rs)

    def test_council_verify_safety_veto(self):
        (rc_py, out_py), (rc_rs, out_rs) = self._verify_both(
            self._make_run(safety=2))
        self.assertEqual(out_py, out_rs)
        self.assertEqual(1, rc_rs)
        self.assertIn("safety <= 3", out_rs)

    def test_council_verify_missing_member(self):
        (rc_py, out_py), (rc_rs, out_rs) = self._verify_both(
            self._make_run(missing=("safety.stage2",)))
        self.assertEqual(out_py, out_rs)
        self.assertEqual(1, rc_rs)

    def test_council_verify_inline_mode(self):
        (rc_py, out_py), (rc_rs, out_rs) = self._verify_both(
            self._make_run(mode="inline"))
        self.assertEqual(out_py, out_rs)
        self.assertEqual(0, rc_rs)
        self.assertIn("unverified_member_isolation", out_rs)

    # ------------------------------------------------------------- sanitize

    def _seed_stores(self, root: Path):
        """Identical store trees under two homes for mutation parity."""
        transcripts = (root / ".local" / "share" / "devin" / "cli"
                       / "transcripts")
        transcripts.mkdir(parents=True)
        (transcripts / "t1.json").write_text(
            '["msg ghp_abcdefghij0123456789ABCD end"]')
        (transcripts / "clean.json").write_text('["nothing here"]')
        logs = root / ".local" / "share" / "devin" / "cli" / "logs"
        logs.mkdir(parents=True)
        import gzip as _gz
        (logs / "s.log.gz").write_bytes(
            _gz.compress(b"log AKIAABCDEFGHIJKLMNOP end\n"))

    def test_sanitize_scan_parity(self):
        self._seed_stores(self.tmp)
        rc_py, out_py = self._py(SANITIZE_PY, "--scan")
        rc_rs, out_rs = run_rust("sanitize", "--scan", env=self.env)
        self.assertEqual(rc_py, rc_rs)
        self.assertEqual(json.loads(out_py), json.loads(out_rs))
        self.assertEqual(1, rc_rs)

    def test_sanitize_redact_dry_parity(self):
        self._seed_stores(self.tmp)
        rc_py, out_py = self._py(SANITIZE_PY, "--redact", "--dry")
        # python's pass tightened store-dir perms — restore the drift so the
        # rust run reports the same count
        for d in (self.tmp / ".local/share/devin/cli/transcripts",
                  self.tmp / ".local/share/devin/cli/logs",
                  self.ciel / "checkpoints", self.ciel):
            d.chmod(0o755)
        rc_rs, out_rs = run_rust("sanitize", "--redact", "--dry", env=self.env)
        self.assertEqual(json.loads(out_py), json.loads(out_rs))
        self.assertEqual(0, rc_rs)

    def test_sanitize_redact_bytes_identical(self):
        # two identical homes — python redacts one, rust the other; compare
        # every resulting byte + backup + perms
        home_py = self.tmp / "h-py"
        home_rs = self.tmp / "h-rs"
        for h in (home_py, home_rs):
            (h / ".ciel" / "checkpoints").mkdir(parents=True)
            self._seed_stores(h)
        env_py = dict(self.env, HOME=str(home_py),
                      CIEL_HOME=str(home_py / ".ciel"))
        env_rs = dict(self.env, HOME=str(home_rs),
                      CIEL_HOME=str(home_rs / ".ciel"))
        rc_py, out_py = self._py_env(SANITIZE_PY, env_py, "--redact")
        rc_rs, out_rs = run_rust("sanitize", "--redact", env=env_rs)
        self.assertEqual(0, rc_py)
        self.assertEqual(0, rc_rs)
        j_py, j_rs = json.loads(out_py), json.loads(out_rs)
        # same files changed, same replacement counts (paths differ by home)
        self.assertEqual(j_py["files_changed"], j_rs["files_changed"])
        self.assertEqual(j_py["perms_tightened"], j_rs["perms_tightened"])
        for r_py, r_rs in zip(j_py["results"], j_rs["results"]):
            self.assertEqual(Path(r_py["file"]).name,
                             Path(r_rs["file"]).name)
            self.assertEqual(r_py["replacements"], r_rs["replacements"])
            self.assertEqual(r_py["categories"], r_rs["categories"])
        # byte-identical outcomes under both trees
        t_py = home_py / ".local/share/devin/cli/transcripts/t1.json"
        t_rs = home_rs / ".local/share/devin/cli/transcripts/t1.json"
        self.assertEqual(t_py.read_bytes(), t_rs.read_bytes())
        self.assertIn(b"[REDACTED:github_token]", t_rs.read_bytes())
        self.assertTrue(Path(str(t_rs) + ".bak").is_file())
        self.assertEqual(0o600, t_rs.stat().st_mode & 0o777)
        import gzip as _gz
        g_py = _gz.decompress(
            (home_py / ".local/share/devin/cli/logs/s.log.gz").read_bytes())
        g_rs = _gz.decompress(
            (home_rs / ".local/share/devin/cli/logs/s.log.gz").read_bytes())
        self.assertEqual(g_py, g_rs)
        self.assertIn(b"[REDACTED:aws_access_key]", g_rs)

    def _make_sessions_db(self, path: Path):
        import sqlite3
        path.parent.mkdir(parents=True, exist_ok=True)
        db = sqlite3.connect(path)
        db.execute("CREATE TABLE prompt_history "
                   "(id INTEGER PRIMARY KEY, content TEXT)")
        db.execute("CREATE TABLE message_nodes "
                   "(row_id INTEGER PRIMARY KEY, chat_message BLOB)")
        db.execute("CREATE TABLE sessions (id TEXT PRIMARY KEY, "
                   "metadata TEXT, cogs_json TEXT, title TEXT)")
        db.execute("INSERT INTO prompt_history VALUES (1, ?)",
                   ("run it ghp_abcdefghij0123456789ABCD now",))
        db.execute("INSERT INTO message_nodes VALUES (1, ?)",
                   (sqlite3.Binary(
                       b"\x00\xffmsg AKIAABCDEFGHIJKLMNOP end"),))
        db.execute("INSERT INTO sessions VALUES ('s1', '{}', '{}', 'clean')")
        db.commit()
        db.close()

    def test_sanitize_sessions_db_scan_and_redact(self):
        for tag, runner in (("py", "py"), ("rs", "rs")):
            db = self.tmp / f"sessions-{tag}.db"
            self._make_sessions_db(db)
            env = dict(self.env, CIEL_SESSIONS_DB=str(db))
            if runner == "py":
                rc, out = self._py_env(SANITIZE_PY, env, "--scan", "--deep")
            else:
                rc, out = run_rust("sanitize", "--scan", "--deep", env=env)
            hits = json.loads(out)["hits"]
            self.assertIn("sessions.db", hits, f"{tag}: {out}")
            self.assertIn("github_token", hits["sessions.db"])
            self.assertIn("aws_access_key", hits["sessions.db"])
            self.assertEqual(1, rc)
            # redact, then confirm the rows hold no live pattern
            if runner == "py":
                rc, out = self._py_env(SANITIZE_PY, env, "--redact")
            else:
                rc, out = run_rust("sanitize", "--redact", env=env)
            self.assertEqual(0, rc)
            body = json.loads(out)
            sdb = body["sessions_db"]
            self.assertTrue(sdb["changed"], f"{tag}: {sdb}")
            self.assertGreaterEqual(
                sdb["tables"].get("prompt_history", 0), 1)
            self.assertGreaterEqual(
                sdb["tables"].get("message_nodes", 0), 1)
            import sqlite3
            chk = sqlite3.connect(db)
            row = chk.execute(
                "SELECT content FROM prompt_history WHERE id=1").fetchone()
            self.assertIn("[REDACTED:github_token]", row[0])
            blob = chk.execute(
                "SELECT chat_message FROM message_nodes WHERE row_id=1"
            ).fetchone()[0]
            self.assertIsInstance(blob, bytes)
            # 20-char match < 21-char tag → all-stars length-preserving
            # placeholder (identical for both engines); the token is gone
            # and the byte length is unchanged
            self.assertNotIn(b"AKIA", blob)
            self.assertIn(b"****", blob)
            self.assertEqual(30, len(blob))
            self.assertIn(b"\x00\xff", blob)  # invalid bytes verbatim
            chk.close()

    def test_sanitize_sessions_db_locked_flag(self):
        import sqlite3
        db = self.tmp / "sessions-locked.db"
        self._make_sessions_db(db)
        env = dict(self.env, CIEL_SESSIONS_DB=str(db))
        # hold an exclusive write lock from this process
        holder = sqlite3.connect(db)
        holder.execute("BEGIN IMMEDIATE")
        holder.execute("INSERT INTO sessions VALUES ('lock', '', '', '')")
        try:
            rc_rs, out_rs = run_rust(
                "sanitize", "--redact", env=env)
            self.assertEqual(0, rc_rs)
            sdb = json.loads(out_rs)["sessions_db"]
            self.assertTrue(sdb["locked"])
            self.assertIn("deferred", sdb)
            state = json.loads(
                (self.ciel / "checkpoints" / "watchdog_state.json")
                .read_text())
            self.assertTrue(state["sessions_db_sanitize_pending"])
        finally:
            holder.rollback()
            holder.close()

    def test_watchdog_sanitize_pending_inprocess(self):
        # flag set + unlocked db → consume in-process, clear flag, signal
        db = self.tmp / "sessions-p.db"
        self._make_sessions_db(db)
        env = dict(self.env, CIEL_SESSIONS_DB=str(db))
        state = self.ciel / "checkpoints" / "watchdog_state.json"
        state.write_text(json.dumps({"sessions_db_sanitize_pending": True}))
        rc, out = run_rust("watchdog", "--sanitize-pending", env=env)
        self.assertEqual(0, rc)
        msg = json.loads(out)["sanitize"]
        self.assertIn("sanitized", msg)
        self.assertFalse(
            json.loads(state.read_text())["sessions_db_sanitize_pending"])
        sigs = list((self.ciel / "improvements" / "signals").glob(
            "sessions_db_sanitized-*"))
        self.assertEqual(1, len(sigs))


SYSTEM1_PY = LIB / "system1.py"


@unittest.skipUnless(
    Path(CIEL_BIN).exists(), f"ciel binary not built at {CIEL_BIN}"
)
class TestSystem1Parity(unittest.TestCase):
    """Differential parity for `ciel system1 --ask|--decide`."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="ciel-s1-"))
        self.ciel = self.tmp / ".ciel"
        (self.ciel / "system1" / "inflight").mkdir(parents=True)
        self.env = dict(os.environ)
        self.env["HOME"] = str(self.tmp)
        self.env["CIEL_HOME"] = str(self.ciel)
        self.env.pop("CIEL_BIN", None)
        self.env.pop("CIEL_SYSTEM1_MARKER", None)
        self.env.pop("CIEL_SYSTEM1_DISABLED", None)
        self.payload = json.dumps({
            "surface": "pre_tool_risk",
            "state": {"tool": "exec", "command": "ls", "path": ""},
            "questions": {"risk": {"type": "choice",
                                   "instructions": "dangerous?",
                                   "criteria": {"safe": "s", "dangerous": "d"}}},
            "meta": {"ts": "2026-01-01T00:00:00Z", "runtime": "devin"},
        })

    def tearDown(self):
        shutil.rmtree(self.tmp, ignore_errors=True)

    def _py(self, *args: str, stdin: str = ""):
        proc = subprocess.run(
            ["python3", str(SYSTEM1_PY), *args],
            input=stdin, capture_output=True, text=True,
            env=self.env, timeout=120,
        )
        return proc.returncode, proc.stdout

    def _events(self):
        log = self.ciel / "system1" / "events.jsonl"
        if not log.exists():
            return []
        return [json.loads(l) for l in log.read_text().splitlines() if l.strip()]

    def _event_tail(self):
        (self.ciel / "system1" / "events.jsonl").unlink(missing_ok=True)

    def test_decide_offline(self):
        env = dict(self.env, CIEL_SYSTEM1_URL="http://127.0.0.1:1")
        for runner in ("py", "rs"):
            self._event_tail()
            if runner == "py":
                proc = subprocess.run(
                    ["python3", str(SYSTEM1_PY), "--decide"],
                    input=self.payload, capture_output=True, text=True,
                    env=env, timeout=60)
                out = proc.stdout
            else:
                _, out = run_rust("system1", "--decide",
                                  stdin=self.payload, env=env)
            self.assertEqual("null\n", out, runner)
            ev = self._events()
            self.assertEqual(1, len(ev), runner)
            rec = ev[0]
            self.assertIsNone(rec["system1"])
            self.assertEqual("pass", rec["flag"])
            self.assertFalse(rec["cache_hit"])
            self.assertIn("latency_ms", rec)
            self.assertEqual("pre_tool_risk", rec["surface"])

    def test_decide_stub_verdict(self):
        import http.server
        import threading
        body = json.dumps({
            "answers": {"risk": {"choice": "dangerous",
                                 "confidence": 0.91,
                                 "probabilities": {"safe": 0.09,
                                                   "dangerous": 0.91}}},
            "routing": {"model": "stub-1"},
        }).encode()

        class H(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                n = int(self.headers.get("content-length", 0))
                self.rfile.read(n)
                self.send_response(200)
                self.send_header("content-type", "application/json")
                self.send_header("content-length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, *a):
                pass

        srv = http.server.HTTPServer(("127.0.0.1", 0), H)
        port = srv.server_address[1]
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        try:
            env = dict(self.env,
                       CIEL_SYSTEM1_URL=f"http://127.0.0.1:{port}")
            outs = {}
            for runner in ("py", "rs"):
                self._event_tail()
                cache = self.ciel / "system1" / "cache"
                if cache.exists():
                    shutil.rmtree(cache)
                if runner == "py":
                    proc = subprocess.run(
                        ["python3", str(SYSTEM1_PY), "--decide"],
                        input=self.payload, capture_output=True,
                        text=True, env=env, timeout=60)
                    outs[runner] = proc.stdout
                else:
                    _, outs[runner] = run_rust(
                        "system1", "--decide", stdin=self.payload, env=env)
            # identical verdict text (python json.dumps default separators)
            self.assertEqual(outs["py"], outs["rs"])
            self.assertIn('"dangerous"', outs["rs"])
            # rust event record (fresh log, engine-independent shape)
            ev = self._events()
            self.assertEqual(1, len(ev))
            rec = ev[0]
            self.assertEqual("flag", rec["flag"])
            self.assertEqual("stub-1", rec["system1"]["model"])
            self.assertFalse(rec["cache_hit"])
            self.assertIn("latency_ms", rec)
            # cache file exists — the digest path proves canonical-serial
            # parity when the python side resolves it below
            caches = list((self.ciel / "system1" / "cache").glob("*.json"))
            self.assertEqual(1, len(caches))
            self.cache_digest = caches[0].stem
        finally:
            srv.shutdown()

    def test_decide_cache_digest_parity(self):
        # compute the python-side cache path for this payload, write a
        # canned result there, then rust --decide must report cache_hit
        sys.path.insert(0, str(LIB))
        home_prev = os.environ.get("HOME")
        ciel_prev = os.environ.get("CIEL_HOME")
        os.environ["HOME"] = str(self.tmp)
        os.environ["CIEL_HOME"] = str(self.ciel)
        try:
            import importlib
            import system1 as s1
            importlib.reload(s1)
            payload = json.loads(self.payload)
            cache_path = s1._cache_path(payload["state"],
                                        payload["questions"])
        finally:
            if home_prev is None:
                os.environ.pop("HOME", None)
            else:
                os.environ["HOME"] = home_prev
            if ciel_prev is None:
                os.environ.pop("CIEL_HOME", None)
            else:
                os.environ["CIEL_HOME"] = ciel_prev
        cache_path.parent.mkdir(parents=True, exist_ok=True)
        cache_path.write_text(json.dumps(
            {"answers": {"risk": {"choice": "safe", "confidence": 0.99}},
             "model": "cached"}))
        env = dict(self.env, CIEL_SYSTEM1_URL="http://127.0.0.1:1")
        _, out = run_rust("system1", "--decide", stdin=self.payload, env=env)
        # hit the python-computed digest → canonical dumps parity proven
        self.assertIn('"cached"', out)
        ev = self._events()
        self.assertTrue(ev[0]["cache_hit"])
        self.assertNotIn("latency_ms", ev[0])

    def test_ask_marker_cleanup(self):
        marker = self.ciel / "system1" / "inflight" / "m.123"
        marker.touch()
        env = dict(self.env,
                   CIEL_SYSTEM1_MARKER=str(marker),
                   CIEL_SYSTEM1_URL="http://127.0.0.1:1")
        rc, _ = run_rust("system1", "--ask", stdin=self.payload, env=env)
        self.assertEqual(0, rc)
        self.assertFalse(marker.exists())
        self.assertEqual(1, len(self._events()))

    def test_ask_disabled_records_null(self):
        # --ask itself never checks the kill-switch (it lives inside ask) —
        # a disabled run still appends the event with system1: null
        env = dict(self.env, CIEL_SYSTEM1_DISABLED="1")
        rc, _ = run_rust("system1", "--ask", stdin=self.payload, env=env)
        self.assertEqual(0, rc)
        ev = self._events()
        self.assertEqual(1, len(ev))
        self.assertIsNone(ev[0]["system1"])

    def test_ask_empty_payload(self):
        rc, _ = run_rust("system1", "--ask", stdin="{}", env=self.env)
        self.assertEqual(0, rc)
        self.assertEqual([], self._events())


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
