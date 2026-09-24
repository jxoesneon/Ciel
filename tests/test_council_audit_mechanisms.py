"""Unit tests for the council-20260923-conversation-audit mechanisms:

- secret_scan.py (M8) — deterministic secret-in-prompt detection
- attribution_scan.py (M2) — durable-artifact attribution scan
- store_perms.py (M3) — owner-only permission self-heal
- requirements.py (M7) — session requirement ledger
- risk_policy grant_state + advisory tier (M4 / M2 trigger)
- scripts/council_verify.py (M1) — run-artifact verification
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
sys.path.insert(0, str(LIB))

import risk_policy
import secret_scan


class TestSecretScan(unittest.TestCase):
    def test_github_token(self):
        r = secret_scan.scan("my token is ghp_abcdefghij0123456789ABCD ok")
        self.assertIn("github_token", r["categories"])

    def test_password_assignment(self):
        r = secret_scan.scan("the sudo password is hunter2")
        self.assertIn("password_assignment", r["categories"])

    def test_jwt(self):
        r = secret_scan.scan("eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PLf")
        self.assertIn("jwt", r["categories"])

    def test_private_key_block(self):
        r = secret_scan.scan("-----BEGIN OPENSSH PRIVATE KEY-----")
        self.assertIn("private_key_block", r["categories"])

    def test_crates_token(self):
        r = secret_scan.scan("token: cioZZZZZZZZZZZZZZZZZZZZZZZZZZZZ")
        self.assertIn("crates_token", r["categories"])

    def test_clean_text(self):
        r = secret_scan.scan("please fix the off-by-one in paginate.py and run tests")
        self.assertEqual(0, r["hits"])

    def test_no_short_false_positive(self):
        r = secret_scan.scan("the sk variable and api_key handling in config")
        self.assertEqual(0, r["hits"])


class TestAttributionScan(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.home = Path(self.tmp.name)
        self.risk = self.home / "risk"
        self.risk.mkdir()

        import attribution_scan
        self.orig_home = attribution_scan.CIEL_HOME
        attribution_scan.CIEL_HOME = self.home
        # Hermetic: no real git state leaks into scan results
        self.orig_git = attribution_scan._git
        attribution_scan._git = lambda args: ""
        self.mod = attribution_scan

    def tearDown(self):
        self.mod.CIEL_HOME = self.orig_home
        self.mod._git = self.orig_git
        self.tmp.cleanup()

    def test_trailer_in_command(self):
        r = self.mod.scan('git commit -m "fix\n\nGenerated with AI"')
        self.assertEqual("flagged", r["result"])
        cats = {f["category"] for f in r["findings"]}
        self.assertIn("attribution_trailer", cats)

    def test_coauthored_by(self):
        r = self.mod.scan('git commit -m "x" -m "Co-Authored-By: Bot <b@x>"')
        self.assertEqual("flagged", r["result"])

    def test_council_mention(self):
        r = self.mod.scan('gh pr create --body "reviewed by the Council of Five"')
        self.assertEqual("flagged", r["result"])
        cats = {f["category"] for f in r["findings"]}
        self.assertIn("internal_identity", cats)

    def test_label_emoji(self):
        r = self.mod.scan('git commit -m "fix: «Report» done ✨"')
        self.assertEqual("flagged", r["result"])

    def test_bypass_env_in_command(self):
        r = self.mod.scan('CIEL_ATTRIBUTION_SKIP=1 git commit -m "Generated with x"')
        self.assertEqual("bypass", r["result"])

    def test_allowlist_exempts(self):
        (self.risk / "attribution_allowlist.txt").write_text("generated\\s+with\n")
        r = self.mod.scan('git commit -m "Generated with AI"')
        self.assertEqual("clean", r["result"])

    def test_enforce_mode_read(self):
        (self.risk / "attribution_gate").write_text("enforce")
        r = self.mod.scan("git commit -m test")
        self.assertEqual("enforce", r["mode"])

    def test_default_shadow(self):
        r = self.mod.scan("git commit -m test")
        self.assertEqual("shadow", r["mode"])

    def test_clean_commit(self):
        r = self.mod.scan('git commit -m "fix pagination off-by-one"')
        self.assertEqual("clean", r["result"])


class TestAdvisoryTier(unittest.TestCase):
    def test_advisory_rule_returns_scan(self):
        v = risk_policy.evaluate(tool="exec", command="git commit -m x", path="")
        self.assertEqual("allow", v["decision"])
        self.assertEqual("advisory", v.get("tier"))
        self.assertEqual("attribution", v.get("scan"))

    def test_advisory_does_not_mask_hard(self):
        v = risk_policy.evaluate(tool="exec", command="git commit -m x; mkfs.ext4 /dev/sda", path="")
        self.assertEqual("deny", v["decision"])
        self.assertEqual("hard", v.get("tier"))

    def test_advisory_does_not_mask_soft(self):
        v = risk_policy.evaluate(tool="exec", command="git commit -m x && sudo reboot", path="")
        # soft rules still deny (no allow_privileged in test env)
        if not (Path.home() / ".ciel" / "allow_privileged").exists():
            self.assertEqual("deny", v["decision"])


class TestGrantState(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.home = Path(self.tmp.name)
        self.orig = risk_policy.ciel_home
        risk_policy.ciel_home = lambda: self.home

    def tearDown(self):
        risk_policy.ciel_home = self.orig
        self.tmp.cleanup()

    def test_inactive(self):
        s = risk_policy.grant_state()
        self.assertFalse(s["active"])
        self.assertIsNone(s["sentinel_mtime"])

    def test_active_with_provenance(self):
        (self.home / "allow_privileged").touch()
        s = risk_policy.grant_state()
        self.assertTrue(s["active"])
        self.assertIsNotNone(s["sentinel_mtime"])
        self.assertIsNotNone(s["first_seen"])

    def test_transition_logged(self):
        (self.home / "allow_privileged").touch()
        risk_policy.grant_state()
        (self.home / "allow_privileged").unlink()
        risk_policy.grant_state()
        lines = (self.home / "grants.log").read_text().splitlines()
        events = [json.loads(l)["event"] for l in lines]
        self.assertEqual(["grant_first_seen", "grant_removed"], events)


class TestRequirementsLedger(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        import requirements
        self.mod = requirements
        self.orig = requirements.LEDGER
        requirements.LEDGER = Path(self.tmp.name) / "requirements.jsonl"

    def tearDown(self):
        self.mod.LEDGER = self.orig
        self.tmp.cleanup()

    def test_add_pending_done(self):
        self.mod._append({"op": "add", "id": "req-1", "text": "run tests", "session": "s1"})
        self.assertEqual(1, len(self.mod.pending_items("s1")))
        self.mod._append({"op": "done", "id": "req-1", "session": "s1"})
        self.assertEqual(0, len(self.mod.pending_items("s1")))

    def test_session_scoping(self):
        self.mod._append({"op": "add", "id": "req-1", "text": "a", "session": "s1"})
        self.mod._append({"op": "add", "id": "req-2", "text": "b", "session": "s2"})
        self.assertEqual(1, len(self.mod.pending_items("s1")))
        self.assertEqual(2, len(self.mod.pending_items(None)))

    def test_resolve_by_substring(self):
        self.mod._append({"op": "add", "id": "req-9", "text": "verify coverage", "session": None})
        self.assertEqual("req-9", self.mod._resolve_id("coverage", None))


class TestStorePerms(unittest.TestCase):
    def test_sweep_runs_and_reports(self):
        # The real sweep already ran this session — a second run must report ok
        out = subprocess.run(
            [sys.executable, str(LIB / "store_perms.py")],
            capture_output=True, text=True,
        )
        self.assertEqual(0, out.returncode)
        report = out.stdout.strip()
        self.assertTrue(report == "ok" or report.startswith("repaired:"), report)


class TestCouncilVerify(unittest.TestCase):
    def _run_dir(self, tmp: str) -> Path:
        run = Path(tmp) / "council-test" 
        (run / "members").mkdir(parents=True)
        (run / "spawn_receipts.json").write_text(json.dumps({
            "mode": "subagent",
            "members": {m: {"stage1_agent_id": "x", "stage2_agent_id": "y"}
                        for m in ("coherence", "capability", "safety", "efficiency", "evolution")},
        }))
        for m in ("coherence", "capability", "safety", "efficiency", "evolution"):
            for stage in (1, 2):
                (run / "members" / f"{m}.stage{stage}.json").write_text(json.dumps(
                    {"member": m, "stage": stage, "score": 7, "rationale": "ok"}
                ))
        (run / "verdict.json").write_text(json.dumps({
            "verdict": "pass",
            "votes": {m: 7 for m in ("coherence", "capability", "safety", "efficiency", "evolution")},
        }))
        return run

    def test_verified_run(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import council_verify
        with tempfile.TemporaryDirectory() as tmp:
            result = council_verify.verify(self._run_dir(tmp))
        self.assertTrue(result["verified"])
        self.assertEqual("subagent", result["mode"])

    def test_missing_receipts_fails(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import council_verify
        with tempfile.TemporaryDirectory() as tmp:
            run = self._run_dir(tmp)
            (run / "spawn_receipts.json").unlink()
            result = council_verify.verify(run)
        self.assertFalse(result["verified"])
        self.assertTrue(any("spawn_receipts" in p for p in result["problems"]))

    def test_inline_warns(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import council_verify
        with tempfile.TemporaryDirectory() as tmp:
            run = self._run_dir(tmp)
            (run / "spawn_receipts.json").write_text(json.dumps({"mode": "inline", "members": {}}))
            result = council_verify.verify(run)
        self.assertTrue(result["verified"])
        self.assertTrue(any("unverified_member_isolation" in w for w in result["warnings"]))

    def test_safety_veto_caught(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import council_verify
        with tempfile.TemporaryDirectory() as tmp:
            run = self._run_dir(tmp)
            (run / "verdict.json").write_text(json.dumps({
                "verdict": "pass",
                "votes": {"coherence": 7, "capability": 7, "safety": 2, "efficiency": 7, "evolution": 7},
            }))
            result = council_verify.verify(run)
        self.assertFalse(result["verified"])
        self.assertTrue(any("veto" in p for p in result["problems"]))


if __name__ == "__main__":
    unittest.main()


class TestSessionWatchdog(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.home = Path(self.tmp.name)
        sys.path.insert(0, str(LIB))
        import session_watchdog
        self.mod = session_watchdog

    def tearDown(self):
        self.tmp.cleanup()

    def test_resume_caps_session(self):
        m = self.mod
        today = __import__("datetime").datetime.now(
            __import__("datetime").timezone.utc).strftime("%Y-%m-%d")
        state = {"resume_attempts": {"_day": today, "s1": 1}}
        ok, why = m.resume_capable(state, "s1")
        self.assertFalse(ok)
        self.assertIn("session cap", why)

    def test_resume_caps_daily(self):
        m = self.mod
        today = __import__("datetime").datetime.now(
            __import__("datetime").timezone.utc).strftime("%Y-%m-%d")
        state = {"resume_attempts": {"_day": today, "a": 1, "b": 1, "c": 1}}
        ok, why = m.resume_capable(state, "d")
        self.assertFalse(ok)
        self.assertIn("daily cap", why)

    def test_resume_backoff(self):
        m = self.mod
        state = {"resume_attempts": {"_day": "x"}, "last_resume": __import__("time").time()}
        ok, why = m.resume_capable(state, "new")
        self.assertFalse(ok)
        self.assertIn("backoff", why)

    def test_autoresume_gate(self):
        m = self.mod
        self.assertFalse(os.environ.get("CIEL_WATCHDOG_AUTORESUME"))
        # do_resume without the env must refuse regardless of caps
        orig_state = m.STATE
        m.STATE = self.home / "state.json"
        try:
            r = m.do_resume("s1", "test", dry=False)
        finally:
            m.STATE = orig_state
        self.assertFalse(r["fired"])
        self.assertIn("disabled", r["reason"])

    def test_dry_resume(self):
        m = self.mod
        os.environ["CIEL_WATCHDOG_AUTORESUME"] = "1"
        orig_state = m.STATE
        m.STATE = self.home / "state.json"
        try:
            r = m.do_resume("s1", "test", dry=True)
        finally:
            m.STATE = orig_state
            os.environ.pop("CIEL_WATCHDOG_AUTORESUME")
        self.assertFalse(r["fired"])
        self.assertIn("devin -c", r.get("would_run", ""))


class TestTranscriptSanitize(unittest.TestCase):
    def test_redact_replaces_with_placeholder(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import transcript_sanitize
        with tempfile.TemporaryDirectory() as tmp:
            f = Path(tmp) / "t.json"
            f.write_text('{"text": "the sudo password is hunter2 now"}')
            r = transcript_sanitize.redact_file(f, dry=False)
            self.assertTrue(r["changed"])
            self.assertGreaterEqual(r["replacements"], 1)
            out = f.read_text()
            self.assertIn("[REDACTED:", out)
            self.assertNotIn("hunter2", out)
            self.assertTrue(f.with_suffix(".json.bak").is_file())
            self.assertEqual(0o600, f.stat().st_mode & 0o777)

    def test_clean_file_untouched(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import transcript_sanitize
        with tempfile.TemporaryDirectory() as tmp:
            f = Path(tmp) / "t.json"
            f.write_text('{"text": "fix the tests please"}')
            r = transcript_sanitize.redact_file(f, dry=False)
            self.assertFalse(r["changed"])
            self.assertFalse(f.with_suffix(".json.bak").exists())

    def test_gz_roundtrip(self):
        import gzip
        sys.path.insert(0, str(ROOT / "scripts"))
        import transcript_sanitize
        with tempfile.TemporaryDirectory() as tmp:
            f = Path(tmp) / "t.log.gz"
            f.write_bytes(gzip.compress(b"log: password is hunter2 done\n"))
            r = transcript_sanitize.redact_file(f, dry=False)
            self.assertTrue(r["changed"])
            out = gzip.decompress(f.read_bytes()).decode()
            self.assertIn("[REDACTED:", out)
            self.assertNotIn("hunter2", out)
            self.assertEqual(0o600, f.stat().st_mode & 0o777)

    def test_sessions_db_sql_redact(self):
        import sqlite3
        sys.path.insert(0, str(ROOT / "scripts"))
        import transcript_sanitize
        with tempfile.TemporaryDirectory() as tmp:
            db_path = Path(tmp) / "sessions.db"
            db = sqlite3.connect(db_path)
            db.execute("CREATE TABLE prompt_history (id INTEGER PRIMARY KEY, content TEXT)")
            db.execute("INSERT INTO prompt_history VALUES (1, 'token ghp_abcdefghij0123456789ABCD here')")
            db.execute("INSERT INTO prompt_history VALUES (2, 'clean prompt')")
            db.commit()
            db.close()
            orig = transcript_sanitize.SESSIONS_DB
            tables = transcript_sanitize.SESSIONS_TABLES
            try:
                transcript_sanitize.SESSIONS_DB = db_path
                transcript_sanitize.SESSIONS_TABLES = [
                    ("prompt_history", "content", "id", "broad", False)]
                r = transcript_sanitize.redact_sessions_db(dry=False, retries=1, wait=0.1)
            finally:
                transcript_sanitize.SESSIONS_DB = orig
                transcript_sanitize.SESSIONS_TABLES = tables
            self.assertTrue(r["changed"])
            self.assertEqual(r["tables"].get("prompt_history"), 1)
            db = sqlite3.connect(db_path)
            rows = [r[0] for r in db.execute("SELECT content FROM prompt_history ORDER BY id")]
            db.close()
            self.assertIn("[REDACTED:", rows[0])
            self.assertNotIn("ghp_", rows[0])
            self.assertEqual(rows[1], "clean prompt")

    def test_sessions_db_locked_reports(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import transcript_sanitize
        with tempfile.TemporaryDirectory() as tmp:
            missing = Path(tmp) / "absent.db"
            orig = transcript_sanitize.SESSIONS_DB
            try:
                transcript_sanitize.SESSIONS_DB = missing
                r = transcript_sanitize.redact_sessions_db(retries=1, wait=0.1)
            finally:
                transcript_sanitize.SESSIONS_DB = orig
            self.assertFalse(r["changed"])
            self.assertEqual(r.get("reason"), "absent")


class TestDeferredSanitize(unittest.TestCase):
    def test_pending_flag_consumed(self):
        import importlib
        import session_watchdog
        importlib.reload(session_watchdog)
        with tempfile.TemporaryDirectory() as tmp:
            import sqlite3
            db_path = Path(tmp) / "sessions.db"
            db = sqlite3.connect(db_path)
            db.execute("CREATE TABLE t (id INTEGER PRIMARY KEY, c TEXT)")
            db.commit()
            db.close()
            os.environ["CIEL_SESSIONS_DB"] = str(db_path)
            state = {"sessions_db_sanitize_pending": True}
            real_ciel = session_watchdog.CIEL
            src = Path(session_watchdog.CIEL) / "scripts" / "transcript_sanitize.py"
            if not src.is_file():
                src = ROOT / "scripts" / "transcript_sanitize.py"
            session_watchdog.CIEL = Path(tmp)
            (Path(tmp) / "scripts").mkdir()
            import shutil
            shutil.copy(src,
                        Path(tmp) / "scripts" / "transcript_sanitize.py")
            try:
                msg = session_watchdog._sessions_db_sanitize(state)
            finally:
                session_watchdog.CIEL = real_ciel
                os.environ.pop("CIEL_SESSIONS_DB", None)
            self.assertIsNotNone(msg)
            self.assertFalse(state["sessions_db_sanitize_pending"])

    def test_no_pending_noop(self):
        import importlib
        import session_watchdog
        importlib.reload(session_watchdog)
        state = {}
        self.assertIsNone(session_watchdog._sessions_db_sanitize(state))
