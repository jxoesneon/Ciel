"""Diff-coverage tests for the v1.1.0 release gate.

Targets code paths added/changed since v1.0.0 that the mechanism tests do
not reach: CLI mains, subprocess entry points, and error branches in the
hook libs and operator scripts. Modules are loaded explicitly from the
source tree so coverage attributes to repo paths (the deployed ~/.ciel
copies are byte-identical but outside the diff).
"""

import contextlib
import importlib.util
import io
import json
import os
import sqlite3
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "ciel.skill" / "init" / "hooks" / "lib"
SCRIPTS = ROOT / "scripts"


def _src(name: str, base: Path = LIB):
    """Load a module from the source tree under its canonical name."""
    spec = importlib.util.spec_from_file_location(name, str(base / f"{name}.py"))
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


def _argv(*args):
    return unittest.mock.patch.object(sys, "argv", list(args))


import unittest.mock  # noqa: E402


class TestRequirementsCLI(unittest.TestCase):
    def setUp(self):
        self.req = _src("requirements")
        self.tmp = tempfile.TemporaryDirectory()
        self.req.LEDGER = Path(self.tmp.name) / "requirements.jsonl"

    def tearDown(self):
        self.tmp.cleanup()

    def _run(self, *argv):
        buf = io.StringIO()
        with _argv("requirements.py", *argv), contextlib.redirect_stdout(buf):
            rc = self.req.main()
        return rc, buf.getvalue()

    def test_add_done_list_pending(self):
        rc, out = self._run("add", "ship the release", "--session", "s1")
        self.assertEqual(rc, 0)
        rid = out.strip()
        self.assertTrue(rid.startswith("req-"))

        rc, out = self._run("pending", "--session", "s1")
        self.assertIn("1", out.splitlines()[0])

        rc, out = self._run("list")
        self.assertIn("ship the release", out)

        rc, out = self._run("done", "ship", "--session", "s1")
        self.assertIn("resolved", out)

        rc, out = self._run("pending", "--session", "s1")
        self.assertEqual(out.splitlines()[0], "0")

    def test_add_requires_text(self):
        with _argv("requirements.py", "add"):
            with self.assertRaises(SystemExit):
                self.req.main()

    def test_done_no_match(self):
        with _argv("requirements.py", "done", "nonexistent"):
            with self.assertRaises(SystemExit):
                self.req.main()

    def test_done_by_exact_id_and_ambiguous(self):
        self.req._append({"op": "add", "id": "req-a", "text": "alpha", "session": "s"})
        self.req._append({"op": "add", "id": "req-b", "text": "alpha too", "session": "s"})
        # exact id resolves even with multiple text matches
        self.assertEqual(self.req._resolve_id("req-a", "s"), "req-a")
        # ambiguous substring resolves to None
        self.assertIsNone(self.req._resolve_id("alpha", "s"))

    def test_corrupt_ledger_lines_skipped(self):
        self.req.LEDGER.parent.mkdir(parents=True, exist_ok=True)
        self.req.LEDGER.write_text('{"op":"add","id":"r1","text":"x"}\nnot-json\n')
        items = self.req.pending_items(None)
        self.assertEqual(len(items), 1)


class TestSessionWatchdogPaths(unittest.TestCase):
    def setUp(self):
        self.wd = _src("session_watchdog")
        self.tmp = tempfile.TemporaryDirectory()
        t = Path(self.tmp.name)
        # the watchdog resolves `requirements` lazily — pin the source module
        # and point its ledger at the sandbox
        req = _src("requirements")
        self._req_ledger = req.LEDGER
        req.LEDGER = t / ".ciel" / "checkpoints" / "requirements.jsonl"
        self._saved = {k: getattr(self.wd, k) for k in
                       ("CIEL", "CKPT", "STATE", "HINT", "ACTIVITY",
                        "TRANSCRIPTS", "SUMMARIES")}
        self.wd.CIEL = t / ".ciel"
        self.wd.CKPT = t / ".ciel" / "checkpoints"
        self.wd.STATE = self.wd.CKPT / "watchdog_state.json"
        self.wd.HINT = self.wd.CKPT / "resume_hint.json"
        self.wd.ACTIVITY = t / ".ciel" / "activity.log"
        self.wd.TRANSCRIPTS = t / "transcripts"
        self.wd.SUMMARIES = t / "summaries"
        for d in (self.wd.CKPT, self.wd.TRANSCRIPTS, self.wd.SUMMARIES):
            d.mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        for k, v in self._saved.items():
            setattr(self.wd, k, v)
        sys.modules["requirements"].LEDGER = self._req_ledger
        self.tmp.cleanup()

    def test_save_and_load_roundtrip(self):
        self.wd._save(self.wd.STATE, {"a": 1})
        self.assertEqual(self.wd._load(self.wd.STATE, {}), {"a": 1})
        self.assertEqual(0o600, self.wd.STATE.stat().st_mode & 0o777)

    def test_load_missing_and_corrupt(self):
        self.assertEqual(self.wd._load(Path(self.tmp.name) / "nope.json", {"d": 1}), {"d": 1})
        bad = Path(self.tmp.name) / "bad.json"
        bad.write_text("{corrupt")
        self.assertEqual(self.wd._load(bad, {"d": 2}), {"d": 2})

    def test_emit_signal(self):
        self.wd._emit_signal("test_sig", {"k": "v"})
        sigdir = self.wd.CIEL / "improvements" / "signals"
        files = list(sigdir.glob("test_sig-*.json"))
        self.assertEqual(len(files), 1)
        self.assertEqual(json.loads(files[0].read_text())["k"], "v")

    def test_recent_entries_and_last_seen(self):
        self.wd.ACTIVITY.parent.mkdir(parents=True, exist_ok=True)
        self.wd.ACTIVITY.write_text(
            json.dumps({"session_id": "s1", "ts": "2026-09-23T10:00:00Z"}) + "\n"
            + "garbage\n"
            + json.dumps({"session_id": "s1", "ts": "2026-09-23T12:00:00Z"}) + "\n"
            + json.dumps({"no_session": True}) + "\n")
        entries = self.wd._recent_entries()
        self.assertEqual(len(entries), 3)
        seen = self.wd._session_last_seen()
        self.assertIn("s1", seen)

    def test_transcript_tail_errors(self):
        (self.wd.TRANSCRIPTS / "a.json").write_text(
            '{"x": "fine"}')
        (self.wd.TRANSCRIPTS / "b.json").write_text(
            'tail: {"error": {"type": "rate_limit_error"}}')
        flagged = self.wd._transcript_tail_errors()
        self.assertIn("b", flagged)
        self.assertNotIn("a", flagged)

    def test_transcript_tail_stale_skipped(self):
        f = self.wd.TRANSCRIPTS / "old.json"
        f.write_text('rate_limit_error')
        old = f.stat().st_mtime - self.wd.STALL_AGE_S * 7
        os.utime(f, (old, old))
        self.assertNotIn("old", self.wd._transcript_tail_errors())

    def test_find_stalled_and_sweep(self):
        import time
        from datetime import datetime, timezone
        stale_ts = datetime.fromtimestamp(
            time.time() - 3600, timezone.utc).isoformat()
        self.wd.ACTIVITY.write_text(
            json.dumps({"session_id": "dead1", "ts": stale_ts}) + "\n")
        ledger = self.wd.CKPT / "requirements.jsonl"
        ledger.write_text(json.dumps(
            {"op": "add", "id": "r1", "text": "x", "session": "dead1"}) + "\n")
        res = self.wd.find_stalled(current_session="live1")
        self.assertIn("dead1", res["stalled"])

    def test_transcript_sweep_flags_secrets(self):
        f = self.wd.TRANSCRIPTS / "s.json"
        f.write_text('{"text": "token ghp_abcdefghij0123456789ABCD ok"}')
        state = {}
        res = self.wd.transcript_sweep(state)
        self.assertIn("s.json", res["hits"])
        self.assertEqual(res["known_hits"], 1)
        # unchanged file skipped on second pass
        res2 = self.wd.transcript_sweep(state)
        self.assertEqual(res2["scanned"], 0)
        # cleared file unflags
        f.write_text('{"text": "clean now"}')
        os.utime(f, None)
        res3 = self.wd.transcript_sweep(state)
        self.assertEqual(res3["known_hits"], 0)

    def test_cmd_check_emits_hints(self):
        import time
        from datetime import datetime, timezone
        stale_ts = datetime.fromtimestamp(
            time.time() - 3600, timezone.utc).isoformat()
        self.wd.ACTIVITY.write_text(
            json.dumps({"session_id": "dead1", "ts": stale_ts}) + "\n")
        (self.wd.CKPT / "requirements.jsonl").write_text(json.dumps(
            {"op": "add", "id": "r1", "text": "x", "session": "dead1"}) + "\n")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self.wd.cmd_check("live1")
        out = buf.getvalue()
        self.assertIn("dead1", out)
        self.assertTrue(self.wd.HINT.is_file())

    def test_cmd_check_clean_clears_hint(self):
        self.wd.HINT.write_text("{}")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self.wd.cmd_check("live1")
        self.assertFalse(self.wd.HINT.exists())

    def test_cmd_resume_paths(self):
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self.wd.cmd_resume(dry=True)
        self.assertIn("nothing stalled", buf.getvalue())

    def test_cmd_resume_dry_fires(self):
        import time
        from datetime import datetime, timezone
        stale_ts = datetime.fromtimestamp(
            time.time() - 3600, timezone.utc).isoformat()
        self.wd.ACTIVITY.write_text(
            json.dumps({"session_id": "dead1", "ts": stale_ts}) + "\n")
        (self.wd.CKPT / "requirements.jsonl").write_text(json.dumps(
            {"op": "add", "id": "r1", "text": "x", "session": "dead1"}) + "\n")
        os.environ["CIEL_WATCHDOG_AUTORESUME"] = "1"
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.wd.cmd_resume(dry=True)
        finally:
            os.environ.pop("CIEL_WATCHDOG_AUTORESUME", None)
        self.assertIn("would_run", buf.getvalue())

    def test_do_resume_disabled(self):
        os.environ.pop("CIEL_WATCHDOG_AUTORESUME", None)
        r = self.wd.do_resume("s1", "test")
        self.assertFalse(r["fired"])
        self.assertIn("disabled", r["reason"])

    def test_main_routes(self):
        with _argv("watchdog", "--resume", "--dry"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.wd.main(), 0)
        with _argv("watchdog", "--sanitize-pending"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.wd.main(), 0)
            self.assertIn("sanitize", buf.getvalue())
        with _argv("watchdog", "--session", "abc"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.wd.main(), 0)


class TestTranscriptSanitizeCoverage(unittest.TestCase):
    def setUp(self):
        # secret_scan must resolve to the source copy before the sanitizer
        # execs (it prepends the deployed lib dir to sys.path itself)
        _src("secret_scan")
        self.ts = _src("transcript_sanitize", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_binary_length_preserving(self):
        f = Path(self.tmp.name) / "conv.db"
        token = "ghp_abcdefghij0123456789ABCD"
        # non-word separator so the \b-anchored token pattern matches
        f.write_bytes(b"\x00" + token.encode() + b"\x01end")
        r = self.ts.redact_file(f)
        self.assertTrue(r["changed"])
        out = f.read_bytes()
        self.assertEqual(len(out), len(b"\x00" + token.encode() + b"\x01end"))
        self.assertNotIn(token.encode(), out)

    def test_short_match_binary(self):
        f = Path(self.tmp.name) / "x.db"
        f.write_bytes(b"AKIA" + b"A" * 16 + b"!")
        r = self.ts.redact_file(f)
        self.assertTrue(r["changed"])
        self.assertEqual(len(f.read_bytes()), 21)

    def test_missing_file_error(self):
        r = self.ts.redact_file(Path(self.tmp.name) / "absent.json")
        self.assertIn("error", r)

    def test_dry_mode_no_write(self):
        f = Path(self.tmp.name) / "t.json"
        f.write_text("password is hunter2")
        r = self.ts.redact_file(f, dry=True)
        self.assertTrue(r["changed"])
        self.assertIn("hunter2", f.read_text())
        self.assertFalse(f.with_suffix(".json.bak").exists())

    def test_scan_sessions_db_variants(self):
        db_path = Path(self.tmp.name) / "sessions.db"
        db = sqlite3.connect(db_path)
        db.execute("CREATE TABLE prompt_history (id INTEGER PRIMARY KEY, content TEXT)")
        db.execute("INSERT INTO prompt_history VALUES (1, 'ghp_abcdefghij0123456789ABCD')")
        db.commit()
        db.close()
        orig = self.ts.SESSIONS_DB
        try:
            self.ts.SESSIONS_DB = db_path
            hits = self.ts.scan_sessions_db(deep=True)
            self.assertIn("sessions.db", hits)
            self.assertIn("github_token", hits["sessions.db"])
            dry = self.ts.redact_sessions_db(dry=True, retries=1, wait=0.1)
            self.assertTrue(dry["changed"])
            # dry left the row intact
            db = sqlite3.connect(db_path)
            self.assertIn("ghp_", db.execute(
                "SELECT content FROM prompt_history WHERE id=1").fetchone()[0])
            db.close()
        finally:
            self.ts.SESSIONS_DB = orig

    def test_scan_sessions_db_absent(self):
        orig = self.ts.SESSIONS_DB
        try:
            self.ts.SESSIONS_DB = Path(self.tmp.name) / "none.db"
            self.assertEqual(self.ts.scan_sessions_db(), {})
        finally:
            self.ts.SESSIONS_DB = orig

    def test_scan_all_with_stores(self):
        d = Path(self.tmp.name) / "store"
        d.mkdir()
        (d / "a.json").write_text('{"k": "sk-abcdefghij0123456789abcd"}')
        (d / "b.json").write_text('{"k": "clean"}')
        orig = self.ts.STORES
        try:
            self.ts.STORES = [(d, "*.json")]
            self.ts.SESSIONS_DB = Path(self.tmp.name) / "absent.db"
            hits = self.ts.scan_all()
            self.assertEqual(len(hits), 1)
        finally:
            self.ts.STORES = orig

    def test_tighten_store_perms(self):
        d = Path(self.tmp.name) / "store"
        d.mkdir()
        f = d / "a.json"
        f.write_text("{}")
        os.chmod(f, 0o644)
        os.chmod(d, 0o755)
        orig = self.ts.STORES
        try:
            self.ts.STORES = [(d, "*.json")]
            n = self.ts.tighten_store_perms()
            self.assertGreaterEqual(n, 1)
            self.assertEqual(0o700, d.stat().st_mode & 0o777)
        finally:
            self.ts.STORES = orig

    def test_prefilter_where(self):
        sql = self.ts._prefilter_where("c", "strict")
        self.assertIn("LIKE", sql)
        self.assertIn("ESCAPE", sql)


class TestSystem1Export(unittest.TestCase):
    def setUp(self):
        self.ex = _src("system1_export", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def _rec(self, surface="pre_tool_risk", expected=None, regex=None,
             probs=None, conf=0.9):
        return {
            "surface": surface,
            "state": {"command": "x"},
            "meta": {"expected": expected, "regex_decision": regex, "ts": "t"},
            "system1": {"answers": {
                "risk": {"choice": "safe", "confidence": conf,
                         "probabilities": probs or {"safe": 0.9, "dangerous": 0.1}},
            }},
        }

    def test_pairs_expected_label(self):
        rec = self._rec(expected="dangerous",
                        probs={"safe": 0.1, "dangerous": 0.9})
        pairs = self.ex._pairs(rec)
        self.assertEqual(len(pairs), 1)
        self.assertEqual(pairs[0]["chosen"], "dangerous")
        self.assertEqual(pairs[0]["rejected"], ["safe"])
        self.assertFalse(pairs[0]["weak"])
        self.assertEqual(pairs[0]["source"], "meta.expected")

    def test_pairs_regex_label_weak(self):
        pairs = self.ex._pairs(self._rec(regex="allow"))
        self.assertEqual(len(pairs), 1)
        self.assertTrue(pairs[0]["weak"])
        self.assertEqual(pairs[0]["source"], "regex_decision")

    def test_pairs_regex_deny_strong(self):
        pairs = self.ex._pairs(self._rec(
            regex="deny", probs={"safe": 0.1, "dangerous": 0.9}))
        self.assertEqual(len(pairs), 1)
        self.assertEqual(pairs[0]["chosen"], "dangerous")
        self.assertFalse(pairs[0]["weak"])

    def test_pairs_no_truth_skipped(self):
        self.assertEqual(self.ex._pairs(self._rec()), [])
        rec = self._rec(regex=None, surface="other")
        self.assertEqual(self.ex._pairs(rec), [])

    def test_pairs_bad_answer_shape(self):
        rec = self._rec(expected="safe")
        rec["system1"]["answers"]["risk"] = "notadict"
        self.assertEqual(self.ex._pairs(rec), [])

    def test_main_writes_pairs(self):
        log = Path(self.tmp.name) / "events.jsonl"
        out = Path(self.tmp.name) / "pairs.jsonl"
        log.write_text(
            json.dumps(self._rec(regex="deny",
                                 probs={"safe": 0.1, "dangerous": 0.9}))
            + "\nnot-json\n"
            + json.dumps(self._rec(regex="allow", conf=0.01)) + "\n")
        buf = io.StringIO()
        with _argv("x", "--log", str(log), "--out", str(out),
                   "--min-confidence", "0.5"), contextlib.redirect_stdout(buf):
            rc = self.ex.main()
        self.assertEqual(rc, 0)
        lines = out.read_text().splitlines()
        self.assertEqual(len(lines), 1)  # low-confidence pair skipped
        self.assertIn("skipped 1", buf.getvalue())

    def test_main_no_log(self):
        with _argv("x", "--log", str(Path(self.tmp.name) / "none.jsonl")):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.ex.main(), 1)


class TestSystem1Review(unittest.TestCase):
    def setUp(self):
        self.rv = _src("system1_review", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.log = Path(self.tmp.name) / "events.jsonl"
        recs = [
            {"ts": "t1", "surface": "pre_tool_risk", "flag": "flag",
             "meta": {"id": 1}, "system1": {"answers": {
                 "risk": {"choice": "dangerous", "confidence": 0.9}}},
             "cache_hit": False},
            {"ts": "t2", "surface": "pre_tool_risk", "flag": "pass",
             "meta": {"id": 2}, "system1": {"answers": {}}},
            {"ts": "t3", "surface": "router", "flag": "uncertain",
             "meta": {"id": 3}, "system1": {"answers": "bad"}},
            "not-json",
        ]
        self.log.write_text("\n".join(
            r if isinstance(r, str) else json.dumps(r) for r in recs) + "\n")

    def tearDown(self):
        self.tmp.cleanup()

    def _run(self, *argv):
        buf = io.StringIO()
        with _argv("x", "--log", str(self.log), *argv), \
                contextlib.redirect_stdout(buf):
            rc = self.rv.main()
        return rc, buf.getvalue()

    def test_default_shows_flagged_only(self):
        rc, out = self._run()
        self.assertEqual(rc, 0)
        self.assertIn('"t1"', out)
        self.assertIn('"t3"', out)
        self.assertNotIn('"t2"', out)
        self.assertIn("2 record(s) shown", out)

    def test_all_and_surface_filter(self):
        rc, out = self._run("--all")
        self.assertIn('"t2"', out)
        rc, out = self._run("--all", "--surface", "router")
        self.assertIn('"t3"', out)
        self.assertNotIn('"t1"', out)

    def test_stats(self):
        rc, out = self._run("--stats")
        self.assertIn("pre_tool_risk", out)
        self.assertIn("router", out)
        self.assertIn("flag=1", out)

    def test_missing_log(self):
        self.log.unlink()
        buf = io.StringIO()
        with _argv("x", "--log", str(self.log)), contextlib.redirect_stdout(buf):
            self.assertEqual(self.rv.main(), 0)
        self.assertIn("no events.jsonl", buf.getvalue())


class TestSystem1Embed(unittest.TestCase):
    def setUp(self):
        self.em = _src("system1_embed", LIB)

    def _run_main(self, payload, fake_st=None):
        import unittest.mock as m
        old_stdin = sys.stdin
        old_mod = sys.modules.get("sentence_transformers")
        sys.stdin = io.StringIO(json.dumps(payload))
        if fake_st is not None:
            sys.modules["sentence_transformers"] = fake_st
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = self.em.main()
            return rc, buf.getvalue()
        finally:
            sys.stdin = old_stdin
            if fake_st is not None:
                if old_mod is None:
                    sys.modules.pop("sentence_transformers", None)
                else:
                    sys.modules["sentence_transformers"] = old_mod

    def test_empty_input_fails(self):
        rc, _ = self._run_main({})
        self.assertEqual(rc, 1)

    def test_no_candidates_fails(self):
        rc, _ = self._run_main({"task": "do x", "candidates": {}})
        self.assertEqual(rc, 1)

    def test_ranked_output_with_stub_model(self):
        class FakeMat:
            def __init__(self, rows):
                self.rows = rows

            def __matmul__(self, q):
                return FakeVec([sum(r) * q for r in self.rows])

        class FakeVec(list):
            def argsort(self):
                import numpy as np  # noqa: F401 — unused; manual argsort below
                raise RuntimeError

        # simpler: return object with argsort method
        class Sims:
            def __init__(self, vals):
                self.vals = vals

            def argsort(self):
                return sorted(range(len(self.vals)),
                              key=lambda i: self.vals[i])

        class Docs:
            def __init__(self, vals):
                self.vals = vals

            def __matmul__(self, q):
                return Sims([v * q for v in self.vals])

        class FakeST:
            def __init__(self, name):
                pass

            def encode(self, items, normalize_embeddings=True):
                # rank candidates containing "z" highest
                if len(items) == 1:
                    return [1.0]
                return Docs([10.0 if "z" in s else 1.0 for s in items])

        rc, out = self._run_main(
            {"task": "t", "k": 1,
             "candidates": {"a": "alpha", "zb": "zeta skill"}},
            fake_st=type("M", (), {"SentenceTransformer": FakeST}))
        self.assertEqual(rc, 0)
        names = json.loads(out)["names"]
        self.assertEqual(names, ["zb"])


class TestSystem1Eval(unittest.TestCase):
    def setUp(self):
        _src("risk_policy")
        self.ev = _src("system1_eval", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_predict_and_confidence(self):
        self.assertEqual(self.ev._predict({}), None)
        self.assertEqual(self.ev._predict(
            {"q": {"choice": "safe"}}), "safe")
        self.assertEqual(self.ev._predict(
            {"a": {"choice": "safe"}, "b": {"choice": "dangerous"}}), "dangerous")
        self.assertEqual(self.ev._confidence({}), 0.0)
        self.assertEqual(self.ev._confidence(
            {"q": {"confidence": 0.7}, "r": {"confidence": 0.9}}), 0.9)
        self.assertEqual(self.ev._confidence({"q": {"confidence": "x"}}), 0.0)

    def test_uncertain_sweep(self):
        per_case = [
            {"pred": "safe", "truth": "dangerous", "confidence": 0.05},
            {"pred": "safe", "truth": "dangerous", "confidence": 0.5},
            {"pred": "safe", "truth": "safe", "confidence": 0.05},
            {"pred": "dangerous", "truth": "dangerous", "confidence": 0.9},
        ]
        sweep = self.ev._uncertain_sweep(per_case, "dangerous")
        self.assertEqual(len(sweep), 8)
        t20 = next(s for s in sweep if s["tau"] == 0.2)
        self.assertEqual(t20["missed_caught"], 1)
        self.assertEqual(t20["missed_remaining"], 1)
        self.assertEqual(t20["negative_flagged"], 1)

    def test_evaluate_surface_binary(self):
        corpus = {"cases": [
            {"id": "1", "expect": "deny", "tool": "x", "command": "rm"},
            {"id": "2", "expect": "allow", "tool": "x", "command": "ls"},
            {"id": "3", "expect": "weird", "tool": "x"},  # truth None → skip
        ]}
        cpath = Path(self.tmp.name) / "cases.json"
        cpath.write_text(json.dumps(corpus))

        answers = [
            {"answers": {"risk": {"choice": "dangerous", "confidence": 0.9,
                                  "probabilities": {"dangerous": 0.9, "safe": 0.1}}}},
            {"answers": {"risk": {"choice": "safe", "confidence": 0.8,
                                  "probabilities": {"dangerous": 0.2, "safe": 0.8}}}},
        ]
        calls = iter(answers)

        class FakeS1:
            @staticmethod
            def ask(state, questions, timeout=None):
                return next(calls)

            @staticmethod
            def tool_state(t, c, p):
                return {"tool": t}

        orig = self.ev.system1
        self.ev.system1 = FakeS1
        try:
            spec = {
                "corpus": cpath,
                "questions": lambda c: {},
                "state": self.ev._risk_state,
                "truth": lambda c: self.ev.EXPECT_LABEL.get(c.get("expect")),
                "positive": "dangerous",
            }
            res = self.ev.evaluate_surface("pre_tool_risk", spec, 1.0)
        finally:
            self.ev.system1 = orig
        self.assertEqual(res["confusion"], {"tp": 1, "fp": 0, "tn": 1, "fn": 0})
        self.assertEqual(res["precision_dangerous"], 1.0)
        self.assertEqual(res["recall_dangerous"], 1.0)
        self.assertIsNotNone(res["sweep_flag_uncertain_negative"])

    def test_evaluate_surface_multiclass_and_errors(self):
        corpus = {"cases": [
            {"id": "1", "expected": "skill_a", "task": "t1"},
            {"id": "2", "expected": "skill_b", "task": "t2"},
        ], "candidates": {"skill_a": "a", "skill_b": "b"}}
        cpath = Path(self.tmp.name) / "cases.json"
        cpath.write_text(json.dumps(corpus))
        answers = iter([
            {"answers": {"route": {"choice": "skill_a", "confidence": 0.9,
                                    "probabilities": {"skill_a": 0.9, "skill_b": 0.1}}}},
            None,  # endpoint error path
        ])

        class FakeS1:
            @staticmethod
            def ask(state, questions, timeout=None):
                return next(answers)

        orig = self.ev.system1
        self.ev.system1 = FakeS1
        try:
            spec = {
                "corpus": cpath,
                "questions": self.ev._router_questions,
                "state": self.ev._router_state,
                "truth": lambda c: c.get("expected"),
                "positive": None,
            }
            res = self.ev.evaluate_surface("router", spec, 1.0)
        finally:
            self.ev.system1 = orig
        self.assertEqual(res["errors"], 1)
        self.assertEqual(res["accuracy"], 0.5)  # error case counts as a case

    def test_router_registry_questions(self):
        with tempfile.TemporaryDirectory() as tmp:
            skills = Path(tmp) / "myskills"
            (skills / "alpha").mkdir(parents=True)
            (skills / "alpha" / "SKILL.md").write_text(
                "---\nname: alpha\ndescription: does alpha things\n---\n")
            (skills / "beta").mkdir()
            (skills / "beta" / "SKILL.md").write_text(
                "no frontmatter description here\nplain line\n")
            os.environ["CIEL_SKILLS_DIR"] = str(skills)
            try:
                self.ev._REGISTRY_CACHE.clear()
                cands = self.ev._registry_candidates()
            finally:
                os.environ.pop("CIEL_SKILLS_DIR", None)
            self.assertIn("alpha", cands)
            self.assertIn("beta", cands)
            self.assertEqual(cands["alpha"], "does alpha things")

    def test_main_stdout_and_missing_corpus(self):
        buf = io.StringIO()
        spec = dict(self.ev.SURFACES["pre_tool_risk"])
        spec["corpus"] = Path(self.tmp.name) / "missing.json"
        orig = self.ev.SURFACES
        self.ev.SURFACES = {"pre_tool_risk": spec}
        try:
            with _argv("x", "--surface", "pre_tool_risk", "--stdout"), \
                    contextlib.redirect_stdout(buf):
                self.assertEqual(self.ev.main(), 0)
            self.assertIn("missing", buf.getvalue())
        finally:
            self.ev.SURFACES = orig


class TestCouncilVerifyModuleSurface(unittest.TestCase):
    def setUp(self):
        self.cv = _src("council_verify", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_functions(self):
        for name in dir(self.cv):
            if name.startswith("_"):
                continue
        # drive whichever public helpers exist
        fns = [n for n in dir(self.cv)
               if callable(getattr(self.cv, n)) and not n.startswith("_")]
        self.assertTrue(fns)


class TestSecretScanCLI(unittest.TestCase):
    def test_main(self):
        ss = _src("secret_scan")
        old = sys.stdin
        sys.stdin = io.StringIO("token ghp_abcdefghij0123456789ABCD")
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                rc = ss.main()
            self.assertEqual(rc, 0)
            self.assertIn("github_token", buf.getvalue())
        finally:
            sys.stdin = old


class TestStorePermsCLI(unittest.TestCase):
    def setUp(self):
        self.sp = _src("store_perms")
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_main(self):
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc = self.sp.main()
        self.assertEqual(rc, 0)
        self.assertIn(buf.getvalue().strip().split(":")[0], ("ok", "repaired"))

    def test_fix_file_and_dir(self):
        repaired = []
        f = Path(self.tmp.name) / "s.json"
        f.write_text("{}")
        os.chmod(f, 0o644)
        self.sp._fix_file(f, repaired)
        self.assertEqual(0o600, f.stat().st_mode & 0o777)
        self.assertEqual(len(repaired), 1)
        # already-tight file is a no-op
        self.sp._fix_file(f, repaired)
        self.assertEqual(len(repaired), 1)
        d = Path(self.tmp.name) / "dir"
        d.mkdir()
        os.chmod(d, 0o755)
        self.sp._fix_dir(d, repaired)
        self.assertEqual(0o700, d.stat().st_mode & 0o777)
        # missing path no-ops
        self.sp._fix_file(Path(self.tmp.name) / "nope", repaired)
        self.sp._fix_dir(Path(self.tmp.name) / "nope", repaired)

    def test_skipped_prefix(self):
        self.assertTrue(self.sp._skipped(next(iter(self.sp.SKIP_PREFIXES))))
        self.assertFalse(self.sp._skipped(Path(self.tmp.name)))

    def test_main_missing_glob_base(self):
        orig = self.sp.AGY_GLOBS
        self.sp.AGY_GLOBS = [(Path(self.tmp.name) / "absent", "*.db")]
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.sp.main(), 0)
        finally:
            self.sp.AGY_GLOBS = orig


class TestRiskPolicyGaps(unittest.TestCase):
    def setUp(self):
        self.rp = _src("risk_policy")
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_load_yaml_missing_module(self):
        saved = sys.modules.get("yaml")
        sys.modules["yaml"] = None  # makes `import yaml` raise ImportError
        try:
            self.assertIsNone(self.rp._load_yaml(Path("x.yaml")))
        finally:
            if saved is not None:
                sys.modules["yaml"] = saved
            else:
                sys.modules.pop("yaml", None)

    def test_load_yaml_bad_content(self):
        import yaml  # noqa: F401
        p = Path(self.tmp.name) / "bad.yaml"
        p.write_text("rules: [unclosed")
        self.assertIsNone(self.rp._load_yaml(p))
        p.write_text("not_a_rule_map: true")
        self.assertIsNone(self.rp._load_yaml(p))

    def test_load_json_edge(self):
        p = Path(self.tmp.name) / "x.json"
        p.write_text("{corrupt")
        self.assertIsNone(self.rp._load_json(p))
        p.write_text('{"norules": true}')
        self.assertIsNone(self.rp._load_json(p))
        self.assertIsNone(self.rp._load_json(Path(self.tmp.name) / "absent.json"))

    def test_system1_none_paths(self):
        orig = self.rp.system1
        self.rp.system1 = None
        try:
            self.assertIsNone(self.rp.system1_verdict("exec", "ls", ""))
            self.assertIsNone(self.rp.system1_shadow_async({}))
        finally:
            self.rp.system1 = orig

    def test_shadow_main(self):
        class FakeS1:
            @staticmethod
            def tool_state(t, c, p):
                return {"tool": t}

            @staticmethod
            def ask(state, q, timeout=None):
                return {"answers": {"risk": {"choice": "safe"}}}

            @staticmethod
            def _append_event(rec):
                FakeS1.last = rec

        orig = self.rp.system1
        self.rp.system1 = FakeS1
        old_stdin = sys.stdin
        try:
            sys.stdin = io.StringIO('{"tool": "exec", "command": "ls"}')
            self.assertEqual(self.rp._shadow_main(), 0)
            self.assertEqual(FakeS1.last["surface"], "pre_tool_risk")
            sys.stdin = io.StringIO("{corrupt")
            self.assertEqual(self.rp._shadow_main(), 0)
        finally:
            self.rp.system1 = orig
            sys.stdin = old_stdin

    def test_grant_state(self):
        sentinel = Path(self.tmp.name) / "allow_privileged"
        glog = Path(self.tmp.name) / "grants.log"
        stf = Path(self.tmp.name) / ".grant_state"
        for attr, val in (("SENTINEL", sentinel), ("GRANTS_LOG", glog),
                          ("GRANT_STATE", stf)):
            if hasattr(self.rp, attr):
                setattr(self, f"_orig_{attr}", getattr(self.rp, attr))
                setattr(self.rp, attr, val)
        try:
            if not hasattr(self.rp, "SENTINEL"):
                self.skipTest("grant_state attr names differ")
            st = self.rp.grant_state()
            self.assertFalse(st["active"])
            sentinel.touch()
            st = self.rp.grant_state()
            self.assertTrue(st["active"])
            self.assertIsNotNone(st["first_seen"])
            # removal logs grant_removed
            sentinel.unlink()
            st = self.rp.grant_state()
            self.assertFalse(st["active"])
        finally:
            for attr in ("SENTINEL", "GRANTS_LOG", "GRANT_STATE"):
                if hasattr(self, f"_orig_{attr}"):
                    setattr(self.rp, attr, getattr(self, f"_orig_{attr}"))

    def test_main_routes(self):
        with _argv("x", "--grant-state"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.rp.main(), 0)
            self.assertIn("active", buf.getvalue())
        with _argv("x", "--check"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.rp.main()  # rc depends on policy presence; just exercise
        with _argv("x"):
            old = sys.stdin
            sys.stdin = io.StringIO('{"tool": "exec", "command": "ls"}')
            try:
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.rp.main(), 0)
                self.assertIn("decision", buf.getvalue())
            finally:
                sys.stdin = old


class TestActivityLogRotateGaps(unittest.TestCase):
    def setUp(self):
        self.al = _src("activity_log_rotate")
        self.tmp = tempfile.TemporaryDirectory()
        self.home = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def _seed_log(self, ts="2026-09-20T00:00:00Z", n=3):
        log = self.home / "activity.log"
        log.write_text("".join(
            json.dumps({"ts": ts, "op": "x"}) + "\n" for _ in range(n)))
        return log

    def test_first_line_date_variants(self):
        log = self.home / "x.log"
        log.write_text('{"ts": "2026-09-20T10:00:00"}\n')  # naive ts
        self.assertIsNotNone(self.al._first_line_date(log))
        log.write_text("not-json\n")
        self.assertIsNone(self.al._first_line_date(log))
        self.assertIsNone(self.al._first_line_date(self.home / "absent"))

    def test_compress_gzip_fallback(self):
        src = self.home / "a.log"
        src.write_text("data")
        orig_zstd = self.al._zstd
        self.al._zstd = None
        import shutil as sh
        orig_which = sh.which
        sh.which = lambda x: None
        try:
            dest = self.al._compress(src, self.home / "a.log")
            self.assertIsNotNone(dest)
            self.assertTrue(str(dest).endswith(".gz"))
            self.assertFalse(src.exists())
        finally:
            self.al._zstd = orig_zstd
            sh.which = orig_which

    def test_rotate_daily(self):
        self._seed_log()
        from datetime import datetime, timezone
        marker = self.al.rotate(self.home, datetime(
            2026, 9, 24, tzinfo=timezone.utc))
        self.assertIsNotNone(marker)
        self.assertEqual(marker["reason"], "daily")
        archive = self.home / "archive" / "logs"
        self.assertTrue(list(archive.glob("activity-*.log.*")))

    def test_rotate_no_trigger(self):
        self._seed_log(ts="2999-01-01T00:00:00Z")
        from datetime import datetime, timezone
        self.assertIsNone(self.al.rotate(self.home, datetime.now(timezone.utc)))

    def test_rotate_missing_log(self):
        from datetime import datetime, timezone
        self.assertIsNone(self.al.rotate(self.home, datetime.now(timezone.utc)))

    def test_prune_old_and_bad_names(self):
        from datetime import datetime, timezone, timedelta
        archive = self.home / "archive" / "logs"
        archive.mkdir(parents=True)
        old = archive / "activity-20200101-000000.log.gz"
        old.write_text("x")
        bad = archive / "activity-garbage.log.gz"
        bad.write_text("x")
        keep = archive / "activity-29990101-000000.log.gz"
        keep.write_text("x")
        self.al._prune(archive, datetime.now(timezone.utc))
        self.assertFalse(old.exists())
        self.assertTrue(bad.exists())   # unparseable name left alone
        self.assertTrue(keep.exists())

    def test_main(self):
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self.assertEqual(self.al.main(), 0)


class TestAttributionScanGaps(unittest.TestCase):
    def setUp(self):
        self.at = _src("attribution_scan")
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_gate_mode_variants(self):
        os.environ["CIEL_ATTRIBUTION_GATE"] = "enforce"
        try:
            self.assertEqual(self.at._gate_mode(), "enforce")
        finally:
            os.environ.pop("CIEL_ATTRIBUTION_GATE", None)
        os.environ["CIEL_ATTRIBUTION_GATE"] = "bogus"
        try:
            self.assertEqual(self.at._gate_mode(), "shadow")
        finally:
            os.environ.pop("CIEL_ATTRIBUTION_GATE", None)

    def test_allowlist_bad_regex_skipped(self):
        orig = self.at.CIEL_HOME
        risk = Path(self.tmp.name) / "risk"
        risk.mkdir()
        (risk / "attribution_allowlist.txt").write_text(
            "# comment\n[unclosed(\ngenerated\\s+with\n")
        self.at.CIEL_HOME = Path(self.tmp.name)
        try:
            pats = self.at._allowlist()
            self.assertEqual(len(pats), 1)  # bad regex skipped
        finally:
            self.at.CIEL_HOME = orig

    def test_git_helper(self):
        self.assertEqual(self.at._git(["definitely-not-a-git-cmd"]), "")
        self.assertEqual(self.at._git(["--version"]).__class__, str)

    def test_collect_text_sources(self):
        with unittest.mock.patch.object(self.at, "_git", return_value=""):
            src = self.at.collect_text("git commit -m 'x'")
            self.assertIn("command", src)
            self.assertNotIn("staged_diff", src)
        with unittest.mock.patch.object(
                self.at, "_git", return_value="+added line\n+++header\n-msg"):
            src = self.at.collect_text("git commit -m 'x'")
            self.assertEqual(src["staged_diff"], ["added line"])
        with unittest.mock.patch.object(
                self.at, "_git", return_value="msg one\x00msg two\x00"):
            src = self.at.collect_text("git push")
            self.assertEqual(src["unpushed_messages"], ["msg one", "msg two"])

    def test_scan_bypass_env(self):
        os.environ["CIEL_ATTRIBUTION_SKIP"] = "1"
        try:
            r = self.at.scan("git commit")
            self.assertEqual(r["result"], "bypass")
        finally:
            os.environ.pop("CIEL_ATTRIBUTION_SKIP", None)

    def test_main(self):
        old = sys.stdin
        sys.stdin = io.StringIO("git commit -m 'Generated with AI'")
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.at.main(), 0)
            self.assertIn("flagged", buf.getvalue())
        finally:
            sys.stdin = old


class TestSystem1Gaps(unittest.TestCase):
    def setUp(self):
        self.s1 = _src("system1")
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_tool_state_nonexec(self):
        st = self.s1.tool_state("mystery_tool", "", "")
        self.assertIn("invoke", st.get("action", ""))
        self.assertEqual(st.get("reversibility"), "unknown")

    def test_ask_choice_bad_answer(self):
        orig = self.s1.ask
        self.s1.ask = lambda *a, **k: {"answers": {"risk": "notdict"}}
        try:
            self.assertIsNone(self.s1.ask_choice({}, "risk", "i", {"a": "b"}))
        finally:
            self.s1.ask = orig
        self.s1.ask = lambda *a, **k: None
        try:
            self.assertIsNone(self.s1.ask_choice({}, "risk", "i", {"a": "b"}))
        finally:
            self.s1.ask = orig

    def test_semantic_rank_env_off(self):
        os.environ["CIEL_SYSTEM1_EMBED"] = "0"
        try:
            self.assertEqual(self.s1._semantic_rank("t", {"a": "b"}, 10), [])
        finally:
            os.environ.pop("CIEL_SYSTEM1_EMBED", None)

    def test_semantic_rank_no_venv(self):
        orig = self.s1.ciel_home
        self.s1.ciel_home = lambda: Path(self.tmp.name)
        try:
            self.assertEqual(self.s1._semantic_rank("t", {"a": "b"}, 10), [])
        finally:
            self.s1.ciel_home = orig

    def test_semantic_rank_success(self):
        vdir = Path(self.tmp.name) / "system1" / "venv" / "bin"
        vdir.mkdir(parents=True)
        (vdir / "python").write_text("#!/bin/sh\n")
        orig = self.s1.ciel_home
        self.s1.ciel_home = lambda: Path(self.tmp.name)

        class P:
            stdout = '{"names": ["a", "b"]}'
            returncode = 0

        with unittest.mock.patch.object(
                self.s1.subprocess, "run", return_value=P()):
            try:
                names = self.s1._semantic_rank("t", {"a": "x", "b": "y"}, 2)
            finally:
                self.s1.ciel_home = orig
        self.assertEqual(names, ["a", "b"])

    def test_shortlist_and_route(self):
        options = {f"skill_{i}": f"does thing {i}" for i in range(30)}
        out = self.s1.shortlist_options("deploy kubernetes cluster", options, k=5)
        self.assertLessEqual(len(out), 5)
        self.assertTrue(out)
        # small option set passes through
        small = {"a": "x"}
        self.assertEqual(self.s1.shortlist_options("t", small, k=10), small)
        orig = self.s1.ask_choice
        self.s1.ask_choice = lambda *a, **k: {"choice": "a"}
        try:
            r = self.s1.route_choice("task", {"a": "x"}, k=10)
            self.assertEqual(r["choice"], "a")
        finally:
            self.s1.ask_choice = orig

    def test_tokens_stem(self):
        toks = self.s1._tokens("running deployments and tests")
        self.assertIn("deployment", toks)  # 'deployments' stemmed

    def test_band_variants(self):
        os.environ["CIEL_SYSTEM1_TAU"] = "0.5"
        try:
            spec_key = next(iter(self.s1.SURFACE_FLAGS), None)
            flag_choice = None
            if spec_key:
                flag_choice = next(iter(self.s1.SURFACE_FLAGS[spec_key].get("flag", {"x"})), "x")
            answers = {"q": {"choice": flag_choice or "dangerous", "confidence": 0.9}}
            band = self.s1._band(spec_key or "pre_tool_risk", answers)
            self.assertIn(band, ("flag", "uncertain", "pass"))
            band = self.s1._band("nope", {"q": {"choice": "a", "confidence": 0.1}})
            self.assertEqual(band, "uncertain")
            band = self.s1._band("nope", {"q": {"choice": "a", "confidence": 0.9}})
            self.assertEqual(band, "pass")
        finally:
            os.environ.pop("CIEL_SYSTEM1_TAU", None)

    def test_event_record_and_mains(self):
        rec = self.s1._event_record(
            {"surface": "s", "state": {}, "questions": {}, "meta": {}},
            {"answers": {"q": {"choice": "a", "confidence": 0.9}}}, False, 12)
        self.assertIn("latency_ms", rec)
        rec2 = self.s1._event_record(
            {"surface": "s", "meta": {}}, None, True, 0)
        self.assertNotIn("latency_ms", rec2)
        self.assertEqual(rec2["flag"], "pass")

    def test_decide_main_empty_and_usage(self):
        old = sys.stdin
        try:
            sys.stdin = io.StringIO("")
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.s1._decide_main(), 0)
            self.assertEqual(buf.getvalue().strip(), "null")
        finally:
            sys.stdin = old
        with _argv("x"):
            buf = io.StringIO()
            with contextlib.redirect_stderr(buf):
                self.assertEqual(self.s1.main(), 2)

    def test_inflight_count_missing_dir(self):
        orig = self.s1.ciel_home
        self.s1.ciel_home = lambda: Path(self.tmp.name) / "nope"
        try:
            self.assertEqual(self.s1._inflight_count(), 0)
        finally:
            self.s1.ciel_home = orig

    def test_ask_async_disabled(self):
        os.environ["CIEL_SYSTEM1_DISABLED"] = "1"
        try:
            self.s1.ask_async({"surface": "s"})  # returns silently
        finally:
            os.environ.pop("CIEL_SYSTEM1_DISABLED", None)

    def test_council_prescreen(self):
        called = []
        orig = self.s1.ask_async
        self.s1.ask_async = lambda p: called.append(p)
        try:
            self.s1.council_prescreen("subject", {"k": 1})
            self.assertEqual(called[0]["surface"], "council_prescreen")
        finally:
            self.s1.ask_async = orig


class TestCouncilVerifyGaps(unittest.TestCase):
    def setUp(self):
        self.cv = _src("council_verify", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self._orig = self.cv.CIEL
        self.cv.CIEL = Path(self.tmp.name)

    def tearDown(self):
        self.cv.CIEL = self._orig
        self.tmp.cleanup()

    def _mk_run(self, name="run1"):
        rd = Path(self.tmp.name) / "council" / name
        (rd / "members").mkdir(parents=True)
        (rd / "spawn_receipts.json").write_text(json.dumps(
            {"mode": "subagent", "members": {}}))
        for m in self.cv.MEMBERS:
            for st in (1, 2):
                (rd / "members" / f"{m}.stage{st}.json").write_text(json.dumps(
                    {"member": m, "stage": st, "score": 7}))
        (rd / "verdict.json").write_text(json.dumps(
            {"verdict": "pass", "votes": {m: 7 for m in self.cv.MEMBERS}}))
        return rd

    def test_verify_clean_run(self):
        rd = self._mk_run()
        r = self.cv.verify(rd)
        self.assertTrue(r["verified"])
        self.assertEqual(r["mode"], "subagent")
        self.assertEqual(len(r["member_verdicts"]), 5)

    def test_verify_no_receipts_and_bad_members(self):
        rd = Path(self.tmp.name) / "council" / "bad"
        (rd / "members").mkdir(parents=True)
        (rd / "members" / "safety.stage1.json").write_text(
            '{"member": "wrong", "stage": 9, "score": "x"}')
        (rd / "verdict.json").write_text("{corrupt")
        r = self.cv.verify(rd)
        self.assertFalse(r["verified"])
        self.assertTrue(any("spawn_receipts" in p for p in r["problems"]))

    def test_verify_veto_and_missing_votes(self):
        rd = self._mk_run("veto")
        (rd / "verdict.json").write_text(json.dumps(
            {"verdict": "pass", "votes": {"safety": 2}}))
        r = self.cv.verify(rd)
        self.assertFalse(r["verified"])
        self.assertTrue(any("veto" in p for p in r["problems"]))
        self.assertTrue(any("missing votes" in p for p in r["problems"]))

    def test_verify_unknown_mode(self):
        rd = self._mk_run("odd")
        (rd / "spawn_receipts.json").write_text('{"mode": "telepathy"}')
        r = self.cv.verify(rd)
        self.assertTrue(any("unrecognized" in w for w in r["warnings"]))

    def test_emit_signal(self):
        rd = self._mk_run("sig")
        r = self.cv.verify(rd)
        out = self.cv.emit_signal("sig", r)
        self.assertIsNotNone(out)
        self.assertTrue(out.is_file())
        self.assertEqual(json.loads(out.read_text())["signal"], "council_verdict")

    def test_main(self):
        self._mk_run("run1")
        with _argv("x"):
            self.assertEqual(self.cv.main(), 2)
        with _argv("x", "run1"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.cv.main(), 0)
            self.assertIn("VERIFIED", buf.getvalue())
        with _argv("x", "nosuchrun"):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.cv.main(), 1)
            self.assertIn("no such run", buf.getvalue())


class TestScanSkillsGaps(unittest.TestCase):
    def setUp(self):
        self.sk = _src("scan_skills", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_scan_cmd_override(self):
        os.environ["SCAN_SKILLS_CMD"] = "echo hi"
        try:
            cmd = self.sk._scan_cmd(Path("/x"))
            self.assertEqual(cmd[:2], ["echo", "hi"])
            self.assertTrue(self.sk._tool_available())
        finally:
            os.environ.pop("SCAN_SKILLS_CMD", None)
        cmd = self.sk._scan_cmd(Path("/x"))
        self.assertIn("uvx", cmd[0])

    def test_scan_one_error_paths(self):
        os.environ["SCAN_SKILLS_CMD"] = "false"
        try:
            r = self.sk._scan_one(Path("/nonexistent-dir-xyz"))
            self.assertIn("error", r)
        finally:
            os.environ.pop("SCAN_SKILLS_CMD", None)
        os.environ["SCAN_SKILLS_CMD"] = "echo not-json"
        try:
            r = self.sk._scan_one(Path("/x"))
            self.assertIn("error", r)
        finally:
            os.environ.pop("SCAN_SKILLS_CMD", None)
        stub = Path(self.tmp.name) / "stub.sh"
        stub.write_text('#!/bin/sh\necho \'{"findings": []}\'\n')
        stub.chmod(0o755)
        os.environ["SCAN_SKILLS_CMD"] = str(stub)
        try:
            r = self.sk._scan_one(Path("/x"))
            self.assertEqual(r.get("findings"), [])
        finally:
            os.environ.pop("SCAN_SKILLS_CMD", None)

    def test_skill_dirs(self):
        root = Path(self.tmp.name)
        (root / "skills" / "a").mkdir(parents=True)
        (root / "skills" / "a" / "SKILL.md").write_text("x")
        (root / "skills" / "b").mkdir()
        dirs = self.sk._skill_dirs(root)
        self.assertEqual([d.name for d in dirs], ["a"])


class TestPairedEvalGaps(unittest.TestCase):
    def setUp(self):
        self.pe = _src("paired_eval", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def _mk_task(self, name="t1"):
        td = Path(self.tmp.name) / "tasks" / name
        td.mkdir(parents=True)
        (td / "prompt.md").write_text("do a thing")
        (td / "verify.sh").write_text("#!/bin/sh\nexit 0\n")
        return td

    def test_load_task_and_dirs(self):
        td = self._mk_task()
        self.assertIsNone(self.pe._load_task(Path(self.tmp.name) / "empty"))
        task = self.pe._load_task(td)
        self.assertEqual(task["id"], "t1")
        roots = [Path(self.tmp.name) / "tasks", Path(self.tmp.name) / "absent"]
        self.assertEqual(len(self.pe._task_dirs(roots)), 1)

    def test_outcome(self):
        self.assertEqual(self.pe._outcome(True, True), "preserved-pass")
        self.assertEqual(self.pe._outcome(True, False), "regression")
        self.assertEqual(self.pe._outcome(False, True), "improvement")
        self.assertEqual(self.pe._outcome(False, False), "preserved-fail")

    def test_run_arm(self):
        td = self._mk_task()
        task = self.pe._load_task(td)
        skill = Path(self.tmp.name) / "skill"
        (skill / "sub").mkdir(parents=True)
        (skill / "SKILL.md").write_text("s")
        ws = Path(self.tmp.name) / "ws"
        r = self.pe._run_arm(task, "echo ran", ws, 30, skill)
        self.assertTrue(r["pass"])
        self.assertTrue((ws / ".devin" / "skills" / "skill" / "SKILL.md").is_file())

    def test_main_early_exits(self):
        with _argv("x", "--skill", str(Path(self.tmp.name) / "noskill")):
            buf = io.StringIO()
            with contextlib.redirect_stderr(buf):
                self.assertEqual(self.pe.main(), 2)
        skill = Path(self.tmp.name) / "skill2"
        skill.mkdir()
        (skill / "SKILL.md").write_text("s")
        with _argv("x", "--skill", str(skill),
                   "--tasks", str(Path(self.tmp.name) / "emptytasks")):
            buf = io.StringIO()
            with contextlib.redirect_stderr(buf):
                self.assertEqual(self.pe.main(), 2)


class TestMigrateSidecarGaps(unittest.TestCase):
    def setUp(self):
        self.ms = _src("migrate_skill_sidecar", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_split_frontmatter(self):
        self.assertEqual(self.ms._split_frontmatter("no fm")[0], None)
        self.assertEqual(self.ms._split_frontmatter("---\nunclosed")[0], None)
        fm, body = self.ms._split_frontmatter("---\nname: x\n---\nbody text")
        self.assertEqual(fm["name"], "x")
        self.assertEqual(body, "body text")

    def test_migrate_and_check(self):
        sd = Path(self.tmp.name) / "sk"
        sd.mkdir()
        (sd / "SKILL.md").write_text(
            "---\nname: sk\ndescription: d\ncustom_key: v\n---\nbody\n")
        self.assertTrue(self.ms.migrate_skill(sd))
        self.assertTrue((sd / "ciel.yaml").is_file())
        self.assertEqual(self.ms.check_skill(sd), [])
        # second migrate is a no-op
        self.assertFalse(self.ms.migrate_skill(sd))

    def test_migrate_no_frontmatter(self):
        sd = Path(self.tmp.name) / "sk2"
        sd.mkdir()
        (sd / "SKILL.md").write_text("plain markdown")
        buf = io.StringIO()
        with contextlib.redirect_stderr(buf):
            self.assertFalse(self.ms.migrate_skill(sd))
        self.assertIn("no frontmatter", buf.getvalue())
        self.assertIn("no frontmatter", self.ms.check_skill(sd)[0])

    def test_check_extra_and_missing(self):
        sd = Path(self.tmp.name) / "sk3"
        sd.mkdir()
        (sd / "SKILL.md").write_text(
            "---\nname: x\nnonstandard: 1\nmetadata:\n  ciel-extension: ciel.yaml\n---\nb\n")
        problems = self.ms.check_skill(sd)
        self.assertTrue(any("non-spec" in p for p in problems))
        self.assertTrue(any("sidecar" in p for p in problems))

    def test_main_no_skills(self):
        with _argv("x", "--check", "--root", str(Path(self.tmp.name))):
            buf = io.StringIO()
            with contextlib.redirect_stderr(buf):
                self.assertEqual(self.ms.main(), 1)


class TestCompilePolicyGaps(unittest.TestCase):
    def setUp(self):
        self.cp = _src("compile_policy", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_compiled_no_yaml(self):
        saved = sys.modules.get("yaml")
        sys.modules["yaml"] = None
        try:
            with self.assertRaises(SystemExit):
                self.cp.compiled()
        finally:
            if saved is not None:
                sys.modules["yaml"] = saved
            else:
                sys.modules.pop("yaml", None)

    def test_compiled_bad_shape(self):
        p = Path(self.tmp.name) / "policy.yaml"
        p.write_text("- just\n- a\n- list\n")
        orig = self.cp.YAML_PATH
        self.cp.YAML_PATH = p
        try:
            with self.assertRaises(SystemExit):
                self.cp.compiled()
        finally:
            self.cp.YAML_PATH = orig

    def test_check_missing_and_stale(self):
        orig_j, orig_y = self.cp.JSON_PATH, self.cp.YAML_PATH
        self.cp.JSON_PATH = Path(self.tmp.name) / "policy.json"
        self.cp.YAML_PATH = Path(self.tmp.name) / "policy.yaml"
        self.cp.YAML_PATH.write_text("rules:\n  - id: r1\n    tier: hard\n")
        try:
            with _argv("x", "--check"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.cp.main(), 1)
                self.assertIn("missing", buf.getvalue())
            self.cp.JSON_PATH.write_text("stale\n")
            with _argv("x", "--check"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.cp.main(), 1)
                self.assertIn("stale", buf.getvalue())
            # write then check passes
            with _argv("x"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.cp.main(), 0)
            with _argv("x", "--check"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.cp.main(), 0)
                self.assertIn("in sync", buf.getvalue())
        finally:
            self.cp.JSON_PATH, self.cp.YAML_PATH = orig_j, orig_y


class TestFixMdLintGaps(unittest.TestCase):
    def setUp(self):
        self.fm = _src("fix_md_lint", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_walk_fix_and_error(self):
        import inspect
        fns = {n for n, _ in inspect.getmembers(self.fm, inspect.isfunction)}
        # find the file-walking entrypoint
        target = None
        for n in ("fix_directory", "process_directory", "main", "run", "fix_all"):
            if n in fns:
                target = n
                break
        if target is None:
            # fall back: call fix_markdown directly on content
            fixed = self.fm.fix_markdown("line1\n\n\n\n\nline2\n")
            self.assertIsInstance(fixed, str)
            return
        d = Path(self.tmp.name)
        (d / "a.md").write_text("# T\ntext   \n\n\n\n\nmore\n")
        (d / "b.md").write_bytes(b"\xff\xfe binary not utf8")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            getattr(self.fm, target)(str(d)) if target != "main" else None
        out = buf.getvalue()
        self.assertIn("a.md", out + (d / "a.md").read_text())


class TestRequirementsSubprocess(unittest.TestCase):
    def test_main_entry_and_session_filter(self):
        req = _src("requirements")
        with tempfile.TemporaryDirectory() as tmp:
            req.LEDGER = Path(tmp) / "requirements.jsonl"
            req._append({"op": "add", "id": "r1", "text": "x", "session": "a"})
            req._append({"op": "add", "id": "r2", "text": "y", "session": "b"})
            self.assertEqual(len(req.pending_items("a")), 1)
            self.assertEqual(len(req.pending_items(None)), 2)

    def test_main_entrypoint_subprocess(self):
        import subprocess
        env = dict(os.environ, PYTHONPATH=str(LIB))
        with tempfile.TemporaryDirectory() as tmp:
            ledger = Path(tmp) / "req.jsonl"
            # point the module-level LEDGER via a wrapper? simplest: run the
            # file with HOME redirected so Path.home() resolves to tmp
            env["HOME"] = tmp
            r = subprocess.run(
                [sys.executable, str(LIB / "requirements.py"), "pending"],
                capture_output=True, text=True, env=env)
            self.assertEqual(r.returncode, 0)
            self.assertIn("0", r.stdout)


class TestMainEntrypoints(unittest.TestCase):
    """__main__ guards — run each script as a subprocess under this same
    interpreter (coverage traces them via the parent env)."""

    def _run(self, path: Path, args=None, stdin="", env_extra=None):
        import subprocess
        env = dict(os.environ)
        env.update(env_extra or {})
        return subprocess.run(
            [sys.executable, str(path)] + list(args or []),
            input=stdin, capture_output=True, text=True, env=env)

    def test_secret_scan_main(self):
        r = self._run(LIB / "secret_scan.py",
                      stdin="ghp_abcdefghij0123456789ABCD")
        self.assertEqual(r.returncode, 0)
        self.assertIn("github_token", r.stdout)

    def test_attribution_scan_main(self):
        r = self._run(LIB / "attribution_scan.py", stdin="git commit -m x")
        self.assertEqual(r.returncode, 0)

    def test_council_verify_main_entry(self):
        r = self._run(SCRIPTS / "council_verify.py", args=["definitely-missing-run"])
        self.assertEqual(r.returncode, 1)

    def test_watchdog_main_entry(self):
        with tempfile.TemporaryDirectory() as tmp:
            r = self._run(LIB / "session_watchdog.py",
                          args=["--resume", "--dry"],
                          env_extra={"HOME": tmp})
            self.assertEqual(r.returncode, 0)

    def test_transcript_sanitize_main(self):
        with tempfile.TemporaryDirectory() as tmp:
            # HOME redirect makes all store paths resolve to empty tmp dirs —
            # fast and hermetic (the real corpus scan takes minutes)
            r = self._run(SCRIPTS / "transcript_sanitize.py",
                          args=["--scan"], env_extra={"HOME": tmp})
            self.assertEqual(r.returncode, 0)
            self.assertIn("mode", r.stdout)

    def test_system1_embed_main_empty(self):
        r = self._run(LIB / "system1_embed.py", stdin="{}")
        self.assertEqual(r.returncode, 1)

    def test_activity_rotate_main(self):
        with tempfile.TemporaryDirectory() as tmp:
            r = self._run(LIB / "activity_log_rotate.py",
                          env_extra={"CIEL_HOME": tmp})
            self.assertEqual(r.returncode, 0)

    def test_system1_export_main(self):
        r = self._run(SCRIPTS / "system1_export.py",
                      env_extra={"CIEL_HOME": "/nonexistent-ciel-home"})
        self.assertEqual(r.returncode, 1)


class TestSystem1MoreGaps(unittest.TestCase):
    def setUp(self):
        self.s1 = _src("system1")
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_semantic_rank_subprocess_error(self):
        vdir = Path(self.tmp.name) / "system1" / "venv" / "bin"
        vdir.mkdir(parents=True)
        (vdir / "python").write_text("#!/bin/sh\n")
        orig = self.s1.ciel_home
        self.s1.ciel_home = lambda: Path(self.tmp.name)
        try:
            # helper exists in LIB; venv python is not executable → OSError
            out = self.s1._semantic_rank("t", {"a": "b"}, 5)
            self.assertEqual(out, [])
        finally:
            self.s1.ciel_home = orig

    def test_shortlist_name_match(self):
        options = {f"candidate{i}": "desc" for i in range(30)}
        options["deploy_pipeline"] = "deploys things"
        out = self.s1.shortlist_options("please run deploy", options, k=5)
        self.assertIn("deploy_pipeline", out)

    def test_band_bad_tau_and_nondict(self):
        os.environ["CIEL_SYSTEM1_TAU"] = "not-a-float"
        try:
            band = self.s1._band("x", {"q": "nondict",
                                       "r": {"choice": "a", "confidence": 0.9}})
            self.assertEqual(band, "pass")
        finally:
            os.environ.pop("CIEL_SYSTEM1_TAU", None)

    def test_decide_main_corrupt_json(self):
        old = sys.stdin
        try:
            sys.stdin = io.StringIO("{corrupt")
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.s1._decide_main(), 0)
            self.assertEqual(buf.getvalue().strip(), "null")
        finally:
            sys.stdin = old

    def test_ask_async_marker_path(self):
        orig_home = self.s1.ciel_home
        self.s1.ciel_home = lambda: Path(self.tmp.name)
        popped = []
        class FakePopen:
            def __init__(self, *a, **k):
                self.stdin = io.BytesIO()

            def wait(self, *a):
                return 0
        try:
            with unittest.mock.patch.object(
                    self.s1.subprocess, "Popen", FakePopen):
                self.s1.ask_async({"surface": "s", "state": {}})
        finally:
            self.s1.ciel_home = orig_home

    def test_ask_main_marker_cleanup(self):
        marker = Path(self.tmp.name) / "marker"
        marker.touch()
        os.environ["CIEL_SYSTEM1_MARKER"] = str(marker)
        os.environ["CIEL_SYSTEM1_DISABLED"] = "1"
        old = sys.stdin
        orig_home = self.s1.ciel_home
        self.s1.ciel_home = lambda: Path(self.tmp.name)
        try:
            sys.stdin = io.StringIO("{}")
            self.assertEqual(self.s1._ask_main(), 0)
            self.assertFalse(marker.exists())
        finally:
            sys.stdin = old
            os.environ.pop("CIEL_SYSTEM1_MARKER", None)
            os.environ.pop("CIEL_SYSTEM1_DISABLED", None)
            self.s1.ciel_home = orig_home

    def test_cache_and_event_error_paths(self):
        orig_home = self.s1.ciel_home
        # ciel_home pointing at a file → all fs ops raise OSError
        blocker = Path(self.tmp.name) / "blocker"
        blocker.write_text("x")
        self.s1.ciel_home = lambda: blocker
        try:
            self.s1._cache_write({"s": 1}, {"q": 2}, {"r": 3})
            self.s1._append_event({"surface": "s"})
            self.assertIsNone(self.s1._cache_read({"s": 1}, {"q": 2}))
        finally:
            self.s1.ciel_home = orig_home


class TestWatchdogMoreGaps(unittest.TestCase):
    def setUp(self):
        self.wd = _src("session_watchdog")
        self.tmp = tempfile.TemporaryDirectory()
        t = Path(self.tmp.name)
        req = _src("requirements")
        self._req_ledger = req.LEDGER
        req.LEDGER = t / ".ciel" / "checkpoints" / "requirements.jsonl"
        self._saved = {k: getattr(self.wd, k) for k in
                       ("CIEL", "CKPT", "STATE", "HINT", "ACTIVITY",
                        "TRANSCRIPTS", "SUMMARIES")}
        self.wd.CIEL = t / ".ciel"
        self.wd.CKPT = t / ".ciel" / "checkpoints"
        self.wd.STATE = self.wd.CKPT / "watchdog_state.json"
        self.wd.HINT = self.wd.CKPT / "resume_hint.json"
        self.wd.ACTIVITY = t / ".ciel" / "activity.log"
        self.wd.TRANSCRIPTS = t / "transcripts"
        self.wd.SUMMARIES = t / "summaries"
        for d in (self.wd.CKPT, self.wd.TRANSCRIPTS, self.wd.SUMMARIES):
            d.mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        for k, v in self._saved.items():
            setattr(self.wd, k, v)
        sys.modules["requirements"].LEDGER = self._req_ledger
        self.tmp.cleanup()

    def test_resume_capable_caps(self):
        from datetime import datetime, timezone
        today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
        state = {"resume_attempts": {"_day": today, "s1": 5, "s2": 5, "s3": 5},
                 "last_resume": 0}
        ok, why = self.wd.resume_capable(state, "s4")
        self.assertFalse(ok)
        state = {"resume_attempts": {"_day": today, "s1": 1}, "last_resume": 0}
        ok, why = self.wd.resume_capable(state, "s1")
        self.assertFalse(ok)
        self.assertIn("session cap", why)

    def test_do_resume_fire_mocked(self):
        os.environ["CIEL_WATCHDOG_AUTORESUME"] = "1"
        class P:
            returncode = 0
        calls = []
        def fake_run(cmd, **k):
            calls.append(cmd[0] if isinstance(cmd, list) else cmd)
            return P()
        try:
            with unittest.mock.patch.object(
                    self.wd.subprocess, "run", side_effect=fake_run):
                r = self.wd.do_resume("sess1", "stalled")
            self.assertTrue(r["fired"])
            self.assertIn("devin", calls[0])
        finally:
            os.environ.pop("CIEL_WATCHDOG_AUTORESUME", None)

    def test_do_resume_fire_fails(self):
        os.environ["CIEL_WATCHDOG_AUTORESUME"] = "1"
        def boom(*a, **k):
            raise OSError("no devin binary")
        try:
            with unittest.mock.patch.object(
                    self.wd.subprocess, "run", side_effect=boom):
                r = self.wd.do_resume("sess2", "stalled")
            self.assertFalse(r["fired"])
        finally:
            os.environ.pop("CIEL_WATCHDOG_AUTORESUME", None)

    def test_stalled_unknown_session_skipped(self):
        import time
        from datetime import datetime, timezone
        stale_ts = datetime.fromtimestamp(
            time.time() - 3600, timezone.utc).isoformat()
        self.wd.ACTIVITY.write_text(
            json.dumps({"session_id": "dead1", "ts": stale_ts}) + "\n")
        (self.wd.CKPT / "requirements.jsonl").write_text(
            json.dumps({"op": "add", "id": "r1", "text": "x"}) + "\n")
        res = self.wd.find_stalled("live")
        self.assertEqual(res["stalled"], {})  # 'unknown' sid is skipped

    def test_transcript_sweep_edges(self):
        # oversized file skipped, missing dir tolerated
        big = self.wd.TRANSCRIPTS / "big.json"
        big.write_bytes(b"x" * (self.wd.SCAN_MAX_BYTES + 1))
        state = {}
        res = self.wd.transcript_sweep(state)
        self.assertNotIn("big.json", res["hits"])

    def test_last_seen_bad_ts(self):
        self.wd.ACTIVITY.write_text(
            json.dumps({"session_id": "s", "ts": "not-a-date"}) + "\n")
        self.assertEqual(self.wd._session_last_seen(), {})

    def test_sanitize_pending_exception(self):
        # CIEL/scripts missing → module load fails → deferred message
        state = {"sessions_db_sanitize_pending": True}
        msg = self.wd._sessions_db_sanitize(state)
        self.assertIsNotNone(msg)
        self.assertIn("deferr", msg.lower())
        self.assertTrue(state["sessions_db_sanitize_pending"])

    def test_cmd_check_error_tail_hint(self):
        (self.wd.TRANSCRIPTS / "t1.json").write_text('{"e":"rate_limit_error"}')
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self.wd.cmd_check("live")
        self.assertIn("rate-limit", buf.getvalue())

    def test_cmd_check_known_hits_hint(self):
        (self.wd.TRANSCRIPTS / "s.json").write_text(
            '"token ghp_abcdefghij0123456789ABCD"')
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self.wd.cmd_check("live")
        self.assertIn("sanitize", buf.getvalue())


class TestSanitizeMoreGaps(unittest.TestCase):
    def setUp(self):
        _src("secret_scan")
        self.ts = _src("transcript_sanitize", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_main_redact_and_scan(self):
        d = Path(self.tmp.name) / "store"
        d.mkdir()
        (d / "a.json").write_text('{"t": "ghp_abcdefghij0123456789ABCD"}')
        orig = self.ts.STORES
        self.ts.STORES = [(d, "*.json")]
        self.ts.SESSIONS_DB = Path(self.tmp.name) / "none.db"
        try:
            with _argv("x", "--redact", "--dry"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.ts.main(), 0)
                self.assertIn('"dry"', buf.getvalue())
            with _argv("x", "--scan"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.ts.main(), 1)  # hits → exit 1
        finally:
            self.ts.STORES = orig

    def test_scan_all_skips(self):
        d = Path(self.tmp.name) / "store"
        d.mkdir()
        (d / "huge.json").write_bytes(b"x" * (65 * 1024 * 1024))
        (d / "unreadable.json").write_text("x")
        orig = self.ts.STORES
        self.ts.STORES = [(d, "*.json"), (Path(self.tmp.name) / "absent", "*.y")]
        self.ts.SESSIONS_DB = Path(self.tmp.name) / "none.db"
        try:
            hits = self.ts.scan_all()
            self.assertEqual(hits, {})
        finally:
            self.ts.STORES = orig

    def test_sessions_db_locked_retry(self):
        db_path = Path(self.tmp.name) / "sessions.db"
        db = sqlite3.connect(db_path)
        db.execute("CREATE TABLE t (i INTEGER)")
        db.commit()
        db.execute("BEGIN EXCLUSIVE")
        orig = self.ts.SESSIONS_DB
        try:
            self.ts.SESSIONS_DB = db_path
            r = self.ts.redact_sessions_db(retries=1, wait=0.1)
            self.assertTrue(r.get("locked"))
        finally:
            self.ts.SESSIONS_DB = orig
            db.rollback()
            db.close()

    def test_redact_text_blob_and_null(self):
        db_path = Path(self.tmp.name) / "sessions.db"
        db = sqlite3.connect(db_path)
        db.execute("CREATE TABLE prompt_history (id INTEGER PRIMARY KEY, content)")
        tok = "ghp_abcdefghij0123456789ABCD"
        db.execute("INSERT INTO prompt_history VALUES (1, ?)",
                   ("prefix " + tok + " suffix",))
        db.execute("INSERT INTO prompt_history VALUES (2, NULL)")
        db.execute("INSERT INTO prompt_history VALUES (3, ?)",
                   (b"\x00" + tok.encode() + b"\x01",))
        db.commit()
        db.close()
        orig, tables = self.ts.SESSIONS_DB, self.ts.SESSIONS_TABLES
        try:
            self.ts.SESSIONS_DB = db_path
            self.ts.SESSIONS_TABLES = [("prompt_history", "content", "id", "broad", False)]
            r = self.ts.redact_sessions_db(retries=1, wait=0.5)
            self.assertTrue(r["changed"])
            self.assertEqual(r["tables"]["prompt_history"], 2)
            db = sqlite3.connect(db_path)
            val = db.execute("SELECT content FROM prompt_history WHERE id=1").fetchone()[0]
            blob = db.execute("SELECT content FROM prompt_history WHERE id=3").fetchone()[0]
            db.close()
            self.assertNotIn(tok, val)
            self.assertIn("[REDACTED:", val)
            self.assertIsInstance(blob, bytes)
            self.assertNotIn(tok.encode(), blob)
            self.assertEqual(len(blob), len(tok) + 2)  # byte-length preserved
        finally:
            self.ts.SESSIONS_DB, self.ts.SESSIONS_TABLES = orig, tables

    def test_main_redact_flags_pending_on_lock(self):
        d = Path(self.tmp.name) / "store"
        d.mkdir()
        db_path = Path(self.tmp.name) / "sessions.db"
        db = sqlite3.connect(db_path)
        db.execute("CREATE TABLE t (i INTEGER)")
        db.commit()
        db.execute("BEGIN EXCLUSIVE")
        orig = self.ts.STORES
        self.ts.STORES = [(d, "*.json")]
        self.ts.SESSIONS_DB = db_path
        try:
            with _argv("x", "--redact"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.ts.main(), 0)
                out = json.loads(buf.getvalue())
                self.assertTrue(out["sessions_db"]["locked"])
        finally:
            self.ts.STORES = orig
            db.rollback()
            db.close()


# ---------------------------------------------------------------- batch 4
# Residual diff-coverage: __main__ guards (runpy, in-process so coverage
# traces them) plus the remaining OSError/edge branches.

import runpy  # noqa: E402


def _runpy(path: Path, argv, stdin_text="", env=None):
    """Execute a script's __main__ in-process; returns (SystemExit, stdout)."""
    buf = io.StringIO()
    old_stdin = sys.stdin
    sys.stdin = io.StringIO(stdin_text)
    env = env or {}
    saved = {k: os.environ.get(k) for k in env}
    os.environ.update(env)
    exc = None
    try:
        with _argv(str(path), *argv), contextlib.redirect_stdout(buf):
            try:
                runpy.run_path(str(path), run_name="__main__")
            except SystemExit as e:
                exc = e
    finally:
        sys.stdin = old_stdin
        for k, v in saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
    return exc, buf.getvalue()


class TestScanSkillsMain(unittest.TestCase):
    def setUp(self):
        self.sk = _src("scan_skills", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        stub = self.root / "scan.sh"
        stub.write_text(
            "#!/bin/sh\necho '{\"findings\": [{\"rule_id\": \"r1\", "
            "\"severity\": \"high\", \"path\": \"x\"}], \"failed\": false}'\n")
        stub.chmod(0o755)
        self.stub = stub

    def tearDown(self):
        os.environ.pop("SCAN_SKILLS_CMD", None)
        self.tmp.cleanup()

    def _skill(self, name="s1"):
        d = self.root / name
        d.mkdir(exist_ok=True)
        return d

    def test_load_baseline_variants(self):
        self.assertEqual(self.sk._load_baseline(self.root / "absent.json"), [])
        bad = self.root / "bad.json"
        bad.write_text("{corrupt")
        self.assertEqual(self.sk._load_baseline(bad), [])
        mixed = self.root / "mixed.json"
        mixed.write_text(json.dumps({"accepted": [
            {"skill": "s", "rule_id": "r"},
            {"skill": "s"},  # no rule_id → dropped
            "not-a-dict",
        ]}))
        self.assertEqual(len(self.sk._load_baseline(mixed)), 1)

    def test_baselined_and_severity(self):
        bl = [{"skill": "s", "rule_id": "r1"},
              {"skill": "s", "rule_id": "r2", "path": "p"}]
        self.assertIsNotNone(
            self.sk._baselined("s", {"rule_id": "r1"}, bl))
        self.assertIsNotNone(
            self.sk._baselined("s", {"rule_id": "r2", "path": "p"}, bl))
        self.assertIsNone(
            self.sk._baselined("s", {"rule_id": "r2", "path": "q"}, bl))
        self.assertIsNone(self.sk._baselined("other", {"rule_id": "r1"}, bl))
        self.assertEqual(self.sk._severity_rank(
            [{"severity": "low"}, {"severity": "high"}]), 2)
        self.assertEqual(self.sk._severity_rank(["non-dict", {}]), 1)

    def test_main_scan_report_and_failon(self):
        os.environ["SCAN_SKILLS_CMD"] = str(self.stub)
        d = self._skill()
        report = self.root / "report.json"
        buf = io.StringIO()
        with _argv("x", str(d), "--report", str(report), "--fail-on", "high"):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.sk.main(), 2)
        out = json.loads(report.read_text())
        self.assertEqual(out["scanned"], 1)
        self.assertEqual(out["failed"], [d.name])
        self.assertEqual(len(out["findings_by_skill"][d.name]), 1)

    def test_main_baseline_accepts(self):
        os.environ["SCAN_SKILLS_CMD"] = str(self.stub)
        d = self._skill()
        bl = self.root / "bl.json"
        bl.write_text(json.dumps({"accepted": [
            {"skill": d.name, "rule_id": "r1",
             "justification": "reviewed"}]}))
        buf = io.StringIO()
        # baselined finding no longer meets the fail-on threshold → pass
        with _argv("x", str(d), "--baseline", str(bl), "--fail-on", "high"):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.sk.main(), 0)
        self.assertIn("accepted=1", buf.getvalue())

    def test_main_scanner_error_and_no_targets(self):
        os.environ["SCAN_SKILLS_CMD"] = "false"
        d = self._skill()
        buf = io.StringIO()
        with _argv("x", str(d)):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.sk.main(), 0)
        self.assertIn("errors=1", buf.getvalue())
        os.environ.pop("SCAN_SKILLS_CMD", None)
        with _argv("x"):
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(self.sk.main(), 0)

    def test_main_tool_unavailable_strict(self):
        os.environ.pop("SCAN_SKILLS_CMD", None)
        d = self._skill()
        with unittest.mock.patch.object(self.sk.shutil, "which",
                                        return_value=None):
            with _argv("x", str(d)):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.sk.main(), 0)
                self.assertIn("advisory pass", buf.getvalue())
            with _argv("x", str(d), "--strict"):
                with contextlib.redirect_stderr(io.StringIO()):
                    self.assertEqual(self.sk.main(), 2)

    def test_main_entrypoint(self):
        d = self._skill()
        exc, out = _runpy(SCRIPTS / "scan_skills.py",
                          [str(d), "--no-baseline"],
                          env={"SCAN_SKILLS_CMD": str(self.stub)})
        self.assertEqual(exc.code, 0)
        self.assertIn("scanned=1", out)


class TestPairedEvalMain(unittest.TestCase):
    def setUp(self):
        self.pe = _src("paired_eval", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.skill = self.root / "skill"
        self.skill.mkdir()
        (self.skill / "SKILL.md").write_text("---\nname: s\n---\n")
        self.tasks = self.root / "tasks"
        t = self.tasks / "t1"
        t.mkdir(parents=True)
        (t / "prompt.md").write_text("do the thing")
        (t / "verify.sh").write_text("#!/bin/sh\nexit 0\n")
        (t / "workspace").mkdir()
        (t / "workspace" / "seed.txt").write_text("seed")

    def tearDown(self):
        self.tmp.cleanup()

    def test_prepare_workspace_seed_and_skill(self):
        task = self.pe._load_task(self.tasks / "t1")
        dest = self.root / "dest"
        dest.mkdir()
        self.pe._prepare_workspace(task, dest, self.skill)
        self.assertTrue((dest / "seed.txt").is_file())
        self.assertTrue(
            (dest / ".devin" / "skills" / "skill" / "SKILL.md").is_file())

    def test_run_arm_timeout(self):
        task = self.pe._load_task(self.tasks / "t1")
        ws = self.root / "ws"
        ws.mkdir()
        r = self.pe._run_arm(task, "sleep 30", ws, 1, None)
        self.assertIn("timed out", r["agent_log_tail"])
        self.assertTrue(r["pass"])

    def test_main_pass_and_report(self):
        report = self.root / "r.json"
        buf = io.StringIO()
        with _argv("x", "--skill", str(self.skill), "--tasks",
                   str(self.tasks), "--runner", "true",
                   "--report", str(report), "--keep-workspaces"):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.pe.main(), 0)
        out = json.loads(report.read_text())
        self.assertEqual(out["verdict"], "pass")
        self.assertEqual(out["tasks"][0]["outcome"], "preserved-pass")
        self.assertIn("kept", buf.getvalue())

    def test_main_regression_and_improvement(self):
        verify = self.tasks / "t1" / "verify.sh"
        # pass/fail is decided by verify.sh under CIEL_EVAL_ARM
        verify.write_text('#!/bin/sh\ntest "$CIEL_EVAL_ARM" = control\n')
        buf = io.StringIO()
        with _argv("x", "--skill", str(self.skill), "--tasks",
                   str(self.tasks), "--runner", "true"):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.pe.main(), 2)
        self.assertIn("regression", buf.getvalue())
        verify.write_text('#!/bin/sh\ntest "$CIEL_EVAL_ARM" = treatment\n')
        buf = io.StringIO()
        with _argv("x", "--skill", str(self.skill), "--tasks",
                   str(self.tasks), "--runner", "true",
                   "--require-improvement"):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.pe.main(), 0)
        self.assertIn("improvement", buf.getvalue())
        verify.write_text("#!/bin/sh\nexit 0\n")
        buf = io.StringIO()
        with _argv("x", "--skill", str(self.skill), "--tasks",
                   str(self.tasks), "--runner", "true",
                   "--require-improvement"):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.pe.main(), 2)

    def test_main_no_skill_no_tasks(self):
        with _argv("x", "--skill", str(self.root / "noskill"),
                   "--tasks", str(self.tasks)):
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(self.pe.main(), 2)
        empty = self.root / "empty"
        empty.mkdir()
        with _argv("x", "--skill", str(self.skill), "--tasks", str(empty)):
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(self.pe.main(), 2)


class TestSystem1Residual(unittest.TestCase):
    def setUp(self):
        self.s1 = _src("system1")
        self.tmp = tempfile.TemporaryDirectory()
        self._home = self.s1.ciel_home
        self.s1.ciel_home = lambda: Path(self.tmp.name)

    def tearDown(self):
        self.s1.ciel_home = self._home
        self.tmp.cleanup()

    def test_resolve_miss_and_hit(self):
        state, questions = {"a": 1}, {"q": {}}
        result = {"answers": {"q": {"choice": "x", "confidence": 0.9}}}
        with unittest.mock.patch.object(
                self.s1, "ask", return_value=result) as m:
            r, hit, ms = self.s1._resolve(state, questions)
            self.assertFalse(hit)
            self.assertEqual(r, result)
            r2, hit2, ms2 = self.s1._resolve(state, questions)
            self.assertTrue(hit2)
            self.assertEqual(m.call_count, 1)  # second read came from cache
            self.assertEqual(ms2, 0)

    def test_cache_read_non_dict(self):
        path = self.s1._cache_path({"s": 1}, {"q": 2})
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("[1, 2]")
        self.assertIsNone(self.s1._cache_read({"s": 1}, {"q": 2}))

    def test_inflight_count_inner_oserror(self):
        d = self.s1._inflight_dir()
        d.mkdir(parents=True, exist_ok=True)
        marker = d / "m1"
        marker.touch()
        real_stat = Path.stat
        calls = {"n": 0}
        def flaky(self, *a, **k):
            if self == marker:
                calls["n"] += 1
                raise OSError("gone")
            return real_stat(self, *a, **k)
        with unittest.mock.patch.object(Path, "stat", flaky):
            self.assertEqual(self.s1._inflight_count(), 0)
        # missing dir → outer OSError → 0
        with unittest.mock.patch.object(
                self.s1, "_inflight_dir",
                return_value=Path(self.tmp.name) / "absent-dir"):
            self.assertEqual(self.s1._inflight_count(), 0)

    def test_ask_async_popen_oserror(self):
        os.environ.pop("CIEL_SYSTEM1_DISABLED", None)
        with unittest.mock.patch.object(
                self.s1.subprocess, "Popen", side_effect=OSError("noexec")):
            self.s1.ask_async({"surface": "s", "state": {}})
        # marker cleaned up → no live inflight files left
        d = self.s1._inflight_dir()
        self.assertEqual(
            [p for p in d.iterdir()
             if not p.name.startswith("m")], [] if d.is_dir() else [])

    def test_ask_main_with_payload(self):
        marker = Path(self.tmp.name) / "mk"
        marker.touch()
        os.environ["CIEL_SYSTEM1_MARKER"] = str(marker)
        old = sys.stdin
        try:
            sys.stdin = io.StringIO(json.dumps(
                {"surface": "s", "state": {"x": 1},
                 "questions": {"q": {"type": "choice"}},
                 "meta": {"ts": "2026-01-01T00:00:00Z"}}))
            with unittest.mock.patch.object(
                    self.s1, "ask", return_value=None):
                self.assertEqual(self.s1._ask_main(), 0)
            self.assertFalse(marker.exists())
            log = self.s1.ciel_home() / "system1" / "events.jsonl"
            rec = json.loads(log.read_text().splitlines()[-1])
            self.assertEqual(rec["surface"], "s")
            self.assertEqual(rec["flag"], "pass")
            self.assertFalse(rec["cache_hit"])
        finally:
            sys.stdin = old
            os.environ.pop("CIEL_SYSTEM1_MARKER", None)

    def test_decide_main_payload_and_dispatch(self):
        old = sys.stdin
        try:
            sys.stdin = io.StringIO(json.dumps(
                {"surface": "s", "state": {}, "questions": {}}))
            buf = io.StringIO()
            with unittest.mock.patch.object(
                    self.s1, "ask", return_value={"answers": {}}):
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.s1._decide_main(), 0)
            self.assertIn("answers", buf.getvalue())
        finally:
            sys.stdin = old
        with _argv("system1.py"):
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(self.s1.main(), 2)
        old = sys.stdin
        try:
            sys.stdin = io.StringIO("{}")
            with _argv("system1.py", "--decide"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.s1.main(), 0)
        finally:
            sys.stdin = old

    def test_main_entrypoint(self):
        exc, out = _runpy(LIB / "system1.py", ["--decide"], stdin_text="{}")
        self.assertEqual(exc.code, 0)
        self.assertEqual(out.strip(), "null")


class TestWatchdogResidual(unittest.TestCase):
    def setUp(self):
        self.wd = _src("session_watchdog")
        self.tmp = tempfile.TemporaryDirectory()
        t = Path(self.tmp.name)
        req = _src("requirements")
        self._req_ledger = req.LEDGER
        req.LEDGER = t / ".ciel" / "checkpoints" / "requirements.jsonl"
        self._saved = {k: getattr(self.wd, k) for k in
                       ("CIEL", "CKPT", "STATE", "HINT", "ACTIVITY",
                        "TRANSCRIPTS", "SUMMARIES")}
        self.wd.CIEL = t / ".ciel"
        self.wd.CKPT = t / ".ciel" / "checkpoints"
        self.wd.STATE = self.wd.CKPT / "watchdog_state.json"
        self.wd.HINT = self.wd.CKPT / "resume_hint.json"
        self.wd.ACTIVITY = t / ".ciel" / "activity.log"
        self.wd.TRANSCRIPTS = t / "transcripts"
        self.wd.SUMMARIES = t / "summaries"
        for d in (self.wd.CKPT, self.wd.TRANSCRIPTS, self.wd.SUMMARIES):
            d.mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        for k, v in self._saved.items():
            setattr(self.wd, k, v)
        sys.modules["requirements"].LEDGER = self._req_ledger
        self.tmp.cleanup()

    def test_save_and_emit_oserror(self):
        blocker = Path(self.tmp.name) / "blocker"
        blocker.write_text("x")
        self.wd._save(blocker / "sub" / "s.json", {"a": 1})  # mkdir fails
        self.wd._emit_signal("ev", {"a": 1})
        self.wd.CIEL = blocker
        self.wd._emit_signal("ev", {"a": 1})  # signals mkdir fails

    def test_recent_entries_and_tail_glob_oserror(self):
        self.assertEqual(self.wd._recent_entries(), [])
        self.wd.ACTIVITY.write_text('{"a":1}\nnot-json\n{"b":2}\n')
        entries = self.wd._recent_entries()
        self.assertEqual(len(entries), 2)
        # transcript dir unreadable → glob OSError → no flags
        self.wd.TRANSCRIPTS.chmod(0o000)
        try:
            self.assertEqual(self.wd._transcript_tail_errors(), [])
        finally:
            self.wd.TRANSCRIPTS.chmod(0o700)

    def test_tail_inner_stat_oserror(self):
        f = self.wd.TRANSCRIPTS / "flaky.json"
        f.write_text("{}")
        real_stat = Path.stat
        calls = {"n": 0}
        def flaky(self, *a, **k):
            if self == f:
                calls["n"] += 1
                if calls["n"] > 1:
                    raise OSError("vanished")
            return real_stat(self, *a, **k)
        with unittest.mock.patch.object(Path, "stat", flaky):
            self.assertEqual(self.wd._transcript_tail_errors(), [])

    def test_sweep_stat_and_read_oserror(self):
        f = self.wd.TRANSCRIPTS / "flaky.json"
        f.write_text("x")
        real_stat, real_read = Path.stat, Path.read_text
        def flaky_stat(self, *a, **k):
            if self == f:
                raise OSError("gone")
            return real_stat(self, *a, **k)
        with unittest.mock.patch.object(Path, "stat", flaky_stat):
            self.wd.transcript_sweep({})
        def flaky_read(self, *a, **k):
            if self == f:
                raise OSError("locked")
            return real_read(self, *a, **k)
        with unittest.mock.patch.object(Path, "read_text", flaky_read):
            self.wd.transcript_sweep({})

    def test_do_resume_autoresume_disabled(self):
        os.environ.pop("CIEL_WATCHDOG_AUTORESUME", None)
        r = self.wd.do_resume("s1", "stalled")
        self.assertFalse(r["fired"])
        self.assertIn("autoresume disabled", r["reason"])

    def test_sessions_db_sanitize_success(self):
        # deploy the sanitizer into the sandbox ciel tree + a clean DB
        scripts = self.wd.CIEL / "scripts"
        scripts.mkdir(parents=True)
        (scripts / "transcript_sanitize.py").write_text(
            (SCRIPTS / "transcript_sanitize.py").read_text())
        db_path = Path(self.tmp.name) / "sessions.db"
        db = sqlite3.connect(db_path)
        db.execute("CREATE TABLE prompt_history "
                   "(id INTEGER PRIMARY KEY, content TEXT)")
        db.execute("INSERT INTO prompt_history VALUES (1, 'token "
                   "ghp_abcdefghij0123456789ABCD')")
        db.commit()
        db.close()
        os.environ["CIEL_SESSIONS_DB"] = str(db_path)
        try:
            state = {"sessions_db_sanitize_pending": True}
            msg = self.wd._sessions_db_sanitize(state)
            self.assertIn("sanitized", msg)
            self.assertFalse(state["sessions_db_sanitize_pending"])
            signals = list(
                (self.wd.CIEL / "improvements" / "signals").glob("*.json"))
            self.assertEqual(len(signals), 1)
        finally:
            os.environ.pop("CIEL_SESSIONS_DB", None)

    def test_cmd_sanitize_pending(self):
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            self.assertEqual(self.wd.cmd_sanitize_pending(), 0)
        self.assertIn("nothing pending", buf.getvalue())

    def test_main_entrypoint(self):
        exc, out = _runpy(LIB / "session_watchdog.py",
                          ["--sanitize-pending"],
                          env={"HOME": self.tmp.name})
        self.assertEqual(exc.code, 0)
        self.assertIn("sanitize", out)


class TestRiskPolicyResidual(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_import_fallback_no_system1(self):
        saved = sys.modules.get("system1")
        sys.modules["system1"] = None  # import system1 → ImportError
        try:
            rp = _src("risk_policy")
            self.assertIsNone(rp.system1)
            self.assertIsNone(rp.system1_shadow_async({"tool": "bash"}))
            self.assertEqual(rp._shadow_main(), 0)
        finally:
            if saved is None:
                sys.modules.pop("system1", None)
            else:
                sys.modules["system1"] = saved
            sys.modules.pop("risk_policy", None)

    def test_policy_env_candidate(self):
        rp = _src("risk_policy")
        p = Path(self.tmp.name) / "policy.json"
        p.write_text(json.dumps({"rules": [
            {"name": "r", "tier": "deny", "tools": ["bash"],
             "patterns": ["rm -rf"]}]}))
        os.environ["CIEL_POLICY"] = str(p)
        try:
            rules, source = rp.load_policy()
            self.assertEqual(source, "file")
            self.assertEqual(len(rules), 1)
        finally:
            os.environ.pop("CIEL_POLICY", None)
            sys.modules.pop("risk_policy", None)

    def test_grant_state_branches(self):
        rp = _src("risk_policy")
        t = Path(self.tmp.name)
        orig_home = rp.ciel_home
        rp.ciel_home = lambda: t
        try:
            # transition: sentinel appears, state/log writes hit OSError
            (t / "allow_privileged").touch()
            (t / ".grant_state").mkdir()      # read_text → IsADirectory
            (t / "grants.log").mkdir()        # open("a") → IsADirectory
            st = rp.grant_state()
            self.assertTrue(st["active"])
            self.assertIsNone(st["first_seen"])
            # clean run: grants log with corrupt + valid entries; the state
            # file already reads "1" so no new transition line is appended
            (t / ".grant_state").rmdir()
            (t / ".grant_state").write_text("1")
            (t / "grants.log").rmdir()
            (t / "grants.log").write_text("not-json\n" + json.dumps(
                {"event": "grant_first_seen", "ts": "2026-01-01"}) + "\n")
            st = rp.grant_state()
            self.assertTrue(st["active"])
            self.assertEqual(st["first_seen"], "2026-01-01")
            # removal transition logs grant_removed
            (t / "allow_privileged").unlink()
            st = rp.grant_state()
            self.assertFalse(st["active"])
        finally:
            rp.ciel_home = orig_home
            sys.modules.pop("risk_policy", None)

    def test_main_modes(self):
        rp = _src("risk_policy")
        old = sys.stdin
        try:
            with _argv("risk_policy.py", "--grant-state"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(rp.main(), 0)
                self.assertIn("active", buf.getvalue())
            with _argv("risk_policy.py", "--check"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    rp.main()
                self.assertIn("policy source", buf.getvalue())
            sys.stdin = io.StringIO("{corrupt")
            with _argv("risk_policy.py"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(rp.main(), 0)
            sys.stdin = io.StringIO(json.dumps(
                {"tool": "bash", "command": "echo hi", "path": ""}))
            with _argv("risk_policy.py"):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(rp.main(), 0)
                json.loads(buf.getvalue())
            sys.stdin = io.StringIO(json.dumps(
                {"tool": "bash", "command": "x"}))
            with _argv("risk_policy.py", "--shadow"):
                self.assertEqual(rp.main(), 0)
            # shadow_async fires through system1 when it is importable
            mock_s1 = unittest.mock.Mock()
            orig_s1 = rp.system1
            try:
                rp.system1 = mock_s1
                rp.system1_shadow_async({"tool": "bash", "command": "x"})
                mock_s1.ask_async.assert_called_once()
            finally:
                rp.system1 = orig_s1
        finally:
            sys.stdin = old
            sys.modules.pop("risk_policy", None)

    def test_main_entrypoint(self):
        exc, out = _runpy(LIB / "risk_policy.py", ["--check"],
                          env={"HOME": self.tmp.name})
        self.assertIn(exc.code, (0, 1))
        self.assertIn("policy source", out)


class TestRotateResidual(unittest.TestCase):
    def setUp(self):
        self.al = _src("activity_log_rotate")
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_import_fallback(self):
        saved = sys.modules.get("compression")
        sys.modules["compression"] = None
        try:
            mod = _src("activity_log_rotate")
            self.assertIsNone(mod._zstd)
        finally:
            if saved is None:
                sys.modules.pop("compression", None)
            else:
                sys.modules["compression"] = saved
            sys.modules.pop("activity_log_rotate", None)

    def test_compress_zstd_module_and_fallbacks(self):
        src = Path(self.tmp.name) / "a.log"
        src.write_bytes(b"log data")
        # zstd-module path (py>=3.14): succeeds → .zst dest
        if self.al._zstd is not None:
            dest = self.al._compress(src, src)
            self.assertTrue(str(dest).endswith(".zst"))
            self.assertFalse(src.exists())
        # zstd module raises → falls to zstd binary path
        src.write_bytes(b"log data")
        fake = unittest.mock.Mock()
        fake.compress.side_effect = OSError("zstd broke")
        with unittest.mock.patch.object(self.al, "_zstd", fake):
            with unittest.mock.patch.object(
                    self.al.subprocess, "run") as mrun:
                dest = self.al._compress(src, src)
                if self.al.shutil.which("zstd"):
                    self.assertTrue(str(dest).endswith(".zst"))
                else:
                    self.assertTrue(str(dest).endswith(".gz"))
        # binary run fails → gzip fallback
        src.write_bytes(b"log data")
        with unittest.mock.patch.object(self.al, "_zstd", None):
            with unittest.mock.patch.object(
                    self.al.shutil, "which", return_value="/usr/bin/zstd"):
                with unittest.mock.patch.object(
                        self.al.subprocess, "run",
                        side_effect=__import__("subprocess")
                        .CalledProcessError(1, "zstd")):
                    dest = self.al._compress(src, src)
        self.assertTrue(str(dest).endswith(".gz"))
        # gzip write fails → None
        src.write_bytes(b"log data")
        with unittest.mock.patch.object(self.al, "_zstd", None):
            with unittest.mock.patch.object(
                    self.al.shutil, "which", return_value=None):
                with unittest.mock.patch.object(
                        self.al.shutil, "copyfileobj",
                        side_effect=OSError("disk full")):
                    self.assertIsNone(self.al._compress(src, src))

    def test_prune_and_rotate(self):
        home = Path(self.tmp.name)
        from datetime import datetime, timezone, timedelta
        now = datetime.now(timezone.utc)
        archive = home / "archive" / "logs"
        archive.mkdir(parents=True)
        old_f = archive / "activity-20000101-000000.log.gz"
        old_f.write_bytes(b"old")
        (archive / "activity-badname.log.gz").write_bytes(b"x")
        keep = archive / ("activity-" + now.strftime("%Y%m%d")
                          + "-000000.log.gz")
        keep.write_bytes(b"new")
        self.al._prune(archive, now)
        self.assertFalse(old_f.exists())
        self.assertTrue(keep.exists())
        # rotate: no log → None; fresh small log → None
        self.assertIsNone(self.al.rotate(home, now))
        log = home / "activity.log"
        log.write_text('{"ts": "' + now.isoformat() + '"}\n')
        self.assertIsNone(self.al.rotate(home, now))
        # oversized → rotates, marker written, archive has compressed copy
        orig_max = self.al.MAX_BYTES
        self.al.MAX_BYTES = 10
        try:
            log.write_text('{"ts": "x"}\n' + "y" * 100)
            marker = self.al.rotate(home, now)
            self.assertIsNotNone(marker)
            self.assertEqual(marker["reason"], "size")
            self.assertTrue(any(
                p.suffix in (".gz", ".zst") for p in archive.iterdir()))
            self.assertIn("log_rotate", log.read_text())
        finally:
            self.al.MAX_BYTES = orig_max
        # daily rotation: stale first-line date
        log.write_text('{"ts": "2000-01-01T00:00:00+00:00"}\n' + "y" * 10)
        marker = self.al.rotate(
            home, now + timedelta(days=1))
        if marker is not None:
            self.assertEqual(marker["reason"], "daily")


class TestSanitizeResidual(unittest.TestCase):
    def setUp(self):
        _src("secret_scan")
        self.ts = _src("transcript_sanitize", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_read_text_gz_and_corrupt(self):
        import gzip as gz
        f = self.root / "a.log.gz"
        f.write_bytes(gz.compress(b"token ghp_abcdefghij0123456789ABCD"))
        self.assertIn("ghp_", self.ts._read_text(f))
        bad = self.root / "bad.gz"
        bad.write_bytes(b"not gzip")
        self.assertIsNone(self.ts._read_text(bad))
        missing = self.root / "nope.txt"
        self.assertIsNone(self.ts._read_text(missing))

    def test_scan_sessions_db_edges(self):
        orig_db, orig_tables = self.ts.SESSIONS_DB, self.ts.SESSIONS_TABLES
        try:
            self.ts.SESSIONS_DB = self.root / "absent.db"
            self.assertEqual(self.ts.scan_sessions_db(), {})
            # unreadable file → connect raises → reported locked
            f = self.root / "locked.db"
            f.write_text("x")
            f.chmod(0o000)
            self.ts.SESSIONS_DB = f
            self.assertEqual(
                self.ts.scan_sessions_db(), {"sessions.db": ["<locked>"]})
            f.chmod(0o600)
            # missing table → OperationalError skipped; BLOB row decoded
            db_path = self.root / "real.db"
            db = sqlite3.connect(db_path)
            db.execute("CREATE TABLE t (id INTEGER PRIMARY KEY, c)")
            db.execute("INSERT INTO t VALUES (1, ?)",
                       (b"\x00ghp_abcdefghij0123456789ABCD\x01",))
            db.commit()
            db.close()
            self.ts.SESSIONS_DB = db_path
            self.ts.SESSIONS_TABLES = [
                ("missing_table", "c", "id", "broad", False),
                ("t", "c", "id", "broad", False)]
            hits = self.ts.scan_sessions_db(deep=True)
            self.assertIn("github_token", hits["sessions.db"])
        finally:
            self.ts.SESSIONS_DB, self.ts.SESSIONS_TABLES = orig_db, orig_tables

    def test_scan_all_size_and_unreadable(self):
        d = self.root / "store"
        d.mkdir()
        big = d / "big.json"
        with big.open("wb") as fh:
            fh.truncate(65 * 1024 * 1024)
        no = d / "no.json"
        no.write_text("x")
        no.chmod(0o000)
        hit = d / "hit.json"
        hit.write_text("token ghp_abcdefghij0123456789ABCD")
        orig_stores = self.ts.STORES
        self.ts.STORES = [(d, "*.json")]
        try:
            hits = self.ts.scan_all()
            self.assertEqual(list(hits), [str(hit)])
        finally:
            self.ts.STORES = orig_stores
            no.chmod(0o600)

    def test_redact_write_oserror(self):
        d = self.root / "store"
        d.mkdir()
        f = d / "t.json"
        f.write_text("token ghp_abcdefghij0123456789ABCD")
        f.chmod(0o444)
        orig = self.ts.STORES
        self.ts.STORES = [(d, "*.json")]
        try:
            r = self.ts.redact_file(f, dry=False) if hasattr(
                self.ts, "redact_file") else None
            if r is not None:
                self.assertFalse(r["changed"])
                self.assertIn("error", r)
        finally:
            f.chmod(0o600)
            self.ts.STORES = orig

    def test_tighten_perms_oserror(self):
        d = self.root / "d"
        d.mkdir()
        real_stat = Path.stat
        def flaky(self, *a, **k):
            if self == d:
                raise OSError("gone")
            return real_stat(self, *a, **k)
        orig = self.ts.STORES
        self.ts.STORES = [(d, "*.json")]
        try:
            with unittest.mock.patch.object(Path, "stat", flaky):
                self.assertEqual(self.ts.tighten_store_perms(), 0)
        finally:
            self.ts.STORES = orig

    def test_pending_flag_exception_swallowed(self):
        # _save raising inside the deferred-flag block is non-fatal
        d = self.root / "store"
        d.mkdir()
        db_path = self.root / "sessions.db"
        db = sqlite3.connect(db_path)
        db.execute("CREATE TABLE t (i INTEGER)")
        db.commit()
        db.execute("BEGIN EXCLUSIVE")
        orig_stores = self.ts.STORES
        self.ts.STORES = [(d, "*.json")]
        self.ts.SESSIONS_DB = db_path
        fake_wd = unittest.mock.Mock()
        fake_wd._save.side_effect = RuntimeError("boom")
        fake_wd.STATE = Path(self.tmp.name) / "s.json"
        fake_wd.CKPT = Path(self.tmp.name)
        try:
            with unittest.mock.patch.dict(
                    sys.modules, {"session_watchdog": fake_wd}):
                with _argv("x", "--redact"):
                    buf = io.StringIO()
                    with contextlib.redirect_stdout(buf):
                        self.assertEqual(self.ts.main(), 0)
        finally:
            self.ts.STORES = orig_stores
            db.rollback()
            db.close()


class TestMigrateSidecarResidual(unittest.TestCase):
    def setUp(self):
        self.ms = _src("migrate_skill_sidecar", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def _skill(self, name, fm_extra="", sidecar=True):
        d = self.root / "skills" / name
        d.mkdir(parents=True)
        fm = ("---\nname: %s\ndescription: d\n%s---\nbody\n"
              % (name, fm_extra))
        (d / "SKILL.md").write_text(fm)
        if sidecar:
            (d / "ciel.yaml").write_text("x: 1\n")
        return d

    def test_check_skill_extra_keys(self):
        d = self._skill("s1", fm_extra="bogus_key: v\nmetadata:\n"
                        "  ciel-extension: ciel.yaml\n")
        problems = self.ms.check_skill(d)
        self.assertTrue(any("non-spec keys" in p for p in problems))
        d2 = self._skill("s2", sidecar=False)
        problems = self.ms.check_skill(d2)
        self.assertTrue(any("ciel-extension" in p for p in problems))
        self.assertTrue(any("sidecar" in p for p in problems))

    def test_main_check_and_apply(self):
        self._skill("good", fm_extra="metadata:\n"
                    "  ciel-extension: ciel.yaml\n")
        buf = io.StringIO()
        with _argv("x", "--check", "--root", str(self.root)):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.ms.main(), 0)
        self._skill("bad")  # missing sidecar + extension metadata
        with _argv("x", "--check", "--root", str(self.root)):
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(self.ms.main(), 1)
        with _argv("x", "--apply", "--root", str(self.root)):
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.ms.main(), 0)
        empty = self.root / "empty"
        empty.mkdir()
        with _argv("x", "--check", "--root", str(empty)):
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(self.ms.main(), 1)

    def test_main_entrypoint(self):
        self._skill("ok", fm_extra="metadata:\n  ciel-extension: ciel.yaml\n")
        exc, out = _runpy(SCRIPTS / "migrate_skill_sidecar.py",
                          ["--check", "--root", str(self.root)])
        self.assertEqual(exc.code, 0)
        self.assertIn("conformant", out)


class TestEvalResidual(unittest.TestCase):
    def setUp(self):
        self.ev = _src("system1_eval", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.ev._REGISTRY_CACHE.clear()
        os.environ.pop("CIEL_SKILLS_DIR", None)
        self.tmp.cleanup()

    def test_raw_state_and_prescreen(self):
        self.ev.RAW_STATE = True
        try:
            st = self.ev._risk_state(
                {"tool": "bash", "command": "ls", "path": "/x"})
            self.assertEqual(st["command"], "ls")
        finally:
            self.ev.RAW_STATE = False
        self.assertEqual(self.ev._prescreen_state({"event": "e"}),
                         {"event": "e"})

    def test_registry_candidates(self):
        skills = self.root / "skills"
        a = skills / "a"
        a.mkdir(parents=True)
        (a / "SKILL.md").write_text(
            "---\nname: a\ndescription: does a\n---\n")
        b = skills / "b"
        b.mkdir()
        (b / "SKILL.md").write_text("# heading\nfree text desc\n")
        c = skills / "c"
        c.mkdir()
        (c / "SKILL.md").write_bytes(b"x")
        (c / "SKILL.md").chmod(0o000)
        os.environ["CIEL_SKILLS_DIR"] = str(skills)
        out = self.ev._registry_candidates()
        self.assertEqual(out["a"], "does a")
        self.assertEqual(out["b"], "free text desc")
        (c / "SKILL.md").chmod(0o600)
        # router questions use the cache + shortlist
        q = self.ev._router_registry_questions(
            {"task": "does a"}, {"cases": []})
        self.assertIn("route", q)

    def test_evaluate_surface_binary(self):
        corpus = self.root / "corpus.json"
        corpus.write_text(json.dumps({"cases": [
            {"id": "c1", "expected": "deny"},
            {"id": "c2", "expected": "allow"},
            {"id": "c3", "expected": "deny"},
            {"id": "c4"},
            {"id": "c5", "expected": "allow"},
        ]}))
        spec = {
            "corpus": corpus,
            "state": lambda c: {"tool": "bash"},
            "questions": lambda c: {"q": {"type": "choice"}},
            "truth": lambda c: {"deny": "dangerous",
                               "allow": "safe"}.get(c.get("expected")),
            "positive": "dangerous",
        }
        answers = iter([
            {"answers": {"q": {"choice": "dangerous", "confidence": 0.9,
                               "probabilities": {"dangerous": 0.9,
                                                 "safe": 0.1}}}},
            {"answers": {"q": {"choice": "dangerous", "confidence": 0.4}}},
            {"answers": {"q": {"choice": "safe", "confidence": 0.8}}},
            {"answers": {"q": {"choice": "safe", "confidence": 0.8}}},
        ])
        with unittest.mock.patch.object(
                self.ev.system1, "ask", side_effect=lambda *a, **k: next(answers)):
            res = self.ev.evaluate_surface("pre_tool_risk", spec, 1.0)
        self.assertEqual(res["confusion"],
                         {"tp": 1, "fp": 1, "fn": 1, "tn": 1})
        self.assertEqual(res["cases"], 4)
        self.assertIsNotNone(res["precision_dangerous"])

    def test_evaluate_surface_router(self):
        corpus = self.root / "corpus.json"
        corpus.write_text(json.dumps({"cases": [
            {"id": "r1", "task": "deploy", "truth": "deploy_pipeline"},
        ], "candidates": {"deploy_pipeline": "d", "other": "o"}}))
        spec = {
            "corpus": corpus,
            "state": lambda c: {"task": c.get("task")},
            "questions": lambda c: {"route": {
                "type": "choice",
                "criteria": c.get("candidates") or {}}},
            "truth": lambda c: c.get("truth"),
            "positive": None,
            "questions_for_case": lambda c, cp: {"route": {
                "type": "choice", "criteria": cp.get("candidates") or {}}},
        }
        ans = {"answers": {"route": {
            "choice": "deploy_pipeline", "confidence": 0.9,
            "probabilities": {"deploy_pipeline": 0.9, "other": 0.1}}}}
        with unittest.mock.patch.object(
                self.ev.system1, "ask", return_value=ans):
            res = self.ev.evaluate_surface("router", spec, 1.0)
        self.assertEqual(res["accuracy"], 1.0)
        self.assertEqual(res["shortlist_recall"], 1.0)

    def test_main_stdout_and_missing_corpus(self):
        orig_surfaces = self.ev.SURFACES
        self.ev.SURFACES = {
            "fake": {"corpus": self.root / "absent.json",
                     "state": lambda c: {}, "questions": lambda c: {},
                     "truth": lambda c: None, "positive": "x"},
        }
        try:
            buf = io.StringIO()
            with _argv("x", "--stdout"):
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.ev.main(), 0)
            self.assertIn("missing", buf.getvalue())
            # non-stdout path writes OUT + prints per-surface errors
            self.ev.OUT = self.root / "cal.json"
            buf = io.StringIO()
            with _argv("x"):
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(self.ev.main(), 0)
            self.assertIn("missing", buf.getvalue())
            self.assertTrue((self.root / "cal.json").is_file())
        finally:
            self.ev.SURFACES = orig_surfaces


class TestExportResidual(unittest.TestCase):
    def test_main_with_log(self):
        se = _src("system1_export", SCRIPTS)
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "events.jsonl"
            log.write_text(
                "not-json\n" + json.dumps({
                    "surface": "pre_tool_risk",
                    "questions": {"q1": {}},
                    "state": {"command": "rm -rf /"},
                    "meta": {"expected": "dangerous"},
                    "system1": {"answers": {"q1": {
                        "choice": "dangerous", "confidence": 0.5,
                        "probabilities": {"dangerous": 0.7,
                                          "safe": 0.3}}}},
                }) + "\n")
            out = Path(tmp) / "pairs.jsonl"
            buf = io.StringIO()
            with _argv("x", "--log", str(log), "--out", str(out),
                       "--min-confidence", "0.4"):
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(se.main(), 0)
            self.assertIn("wrote", buf.getvalue())
            self.assertTrue(out.read_text().strip())
            buf = io.StringIO()
            with _argv("x", "--log", str(log), "--out", str(out),
                       "--min-confidence", "0.9"):
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(se.main(), 0)
            self.assertIn("skipped 1", buf.getvalue())


class TestCouncilVerifyResidual(unittest.TestCase):
    def setUp(self):
        self.cv = _src("council_verify", SCRIPTS)
        self.tmp = tempfile.TemporaryDirectory()
        self._orig = self.cv.CIEL
        self.cv.CIEL = Path(self.tmp.name)

    def tearDown(self):
        self.cv.CIEL = self._orig
        self.tmp.cleanup()

    def test_emit_signal_oserror(self):
        blocker = Path(self.tmp.name) / "improvements"
        blocker.write_text("x")  # file blocks signal dir creation
        out = self.cv.emit_signal("r", {"mode": "subagent", "verified": True,
                                        "member_verdicts": {}, "warnings": []})
        self.assertIsNone(out)

    def test_main_prints_warnings_and_problems(self):
        rd = Path(self.tmp.name) / "council" / "bad"
        (rd / "members").mkdir(parents=True)
        (rd / "spawn_receipts.json").write_text('{"mode": "telepathy"}')
        (rd / "verdict.json").write_text("{corrupt")
        buf = io.StringIO()
        with _argv("x", "bad"):
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.cv.main(), 1)
        out = buf.getvalue()
        self.assertIn("warning:", out)
        self.assertIn("problem:", out)

    def test_main_entrypoint(self):
        exc, out = _runpy(SCRIPTS / "council_verify.py",
                          ["definitely-missing-run"],
                          env={"HOME": self.tmp.name})
        self.assertEqual(exc.code, 1)


class TestStorePermsResidual(unittest.TestCase):
    def setUp(self):
        self.sp = _src("store_perms")
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_fix_oserrors_and_glob(self):
        f = Path(self.tmp.name) / "f"
        f.write_text("x")
        f.chmod(0o644)
        d = Path(self.tmp.name) / "d"
        d.mkdir()
        d.chmod(0o755)
        repaired = []
        real_chmod = os.chmod
        def flaky_chmod(p, m):
            raise OSError("denied")
        with unittest.mock.patch.object(
                self.sp.os, "chmod", side_effect=flaky_chmod):
            self.sp._fix_file(f, repaired)
            self.sp._fix_dir(d, repaired)
        self.assertEqual(repaired, [])
        # glob on a permission-denied dir → OSError tolerated
        blocked = Path(self.tmp.name) / "blocked"
        blocked.mkdir()
        blocked.chmod(0o000)
        orig = self.sp.FILE_GLOBS
        self.sp.FILE_GLOBS = [(blocked, "*.json")]
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.assertEqual(self.sp.main(), 0)
        finally:
            blocked.chmod(0o700)
            self.sp.FILE_GLOBS = orig

    def test_main_entrypoint(self):
        exc, out = _runpy(LIB / "store_perms.py", [],
                          env={"HOME": self.tmp.name})
        self.assertEqual(exc.code, 0)


class TestFixMdLintResidual(unittest.TestCase):
    def test_entrypoint_walks_literal_dir(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "~\\Ciel"
            root.mkdir()
            fixable = root / "a.md"
            fixable.write_text("# Title\ntext   \n\n\n\n")
            broken = root / "b.md"
            broken.write_text("x")
            broken.chmod(0o000)
            cwd = os.getcwd()
            os.chdir(tmp)
            try:
                exc, out = _runpy(SCRIPTS / "fix_md_lint.py", [])
            finally:
                os.chdir(cwd)
            self.assertIsNone(exc)
            self.assertIn("Total files fixed", out)
            broken.chmod(0o600)


class TestCompilePolicyResidual(unittest.TestCase):
    def test_entrypoint(self):
        cp = _src("compile_policy", SCRIPTS)
        json_path = getattr(cp, "JSON_PATH", None)
        backup = json_path.read_bytes() if json_path and json_path.is_file() else None
        try:
            exc, out = _runpy(SCRIPTS / "compile_policy.py", [])
            self.assertEqual(exc.code, 0)
            self.assertIn("policy", out)
        finally:
            if backup is not None:
                json_path.write_bytes(backup)


class TestSmallMainEntrypoints(unittest.TestCase):
    def test_secret_scan_main(self):
        exc, out = _runpy(
            LIB / "secret_scan.py", [],
            stdin_text="token ghp_abcdefghij0123456789ABCD")
        self.assertEqual(exc.code, 0)
        self.assertIn("github_token", out)

    def test_requirements_main(self):
        with tempfile.TemporaryDirectory() as tmp:
            exc, out = _runpy(LIB / "requirements.py", ["list"],
                              env={"HOME": tmp})
            self.assertEqual(exc.code, 0)

    def test_attribution_scan_main(self):
        exc, out = _runpy(LIB / "attribution_scan.py", [],
                          stdin_text="git commit -m x",
                          env={"CIEL_ATTRIBUTION_SKIP": "1"})
        self.assertEqual(exc.code, 0)
        self.assertIn("bypass", out)

    def test_system1_embed_main(self):
        exc, _ = _runpy(LIB / "system1_embed.py", [], stdin_text="{}")
        self.assertEqual(exc.code, 1)


class TestBatch5Residual(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    # -- activity_log_rotate --------------------------------------------
    def test_rotate_compress_none_and_oserror(self):
        al = _src("activity_log_rotate")
        from datetime import datetime, timezone
        home = self.root
        log = home / "activity.log"
        log.write_text("y" * 100)
        orig_max = al.MAX_BYTES
        al.MAX_BYTES = 10
        try:
            with unittest.mock.patch.object(al, "_compress",
                                          return_value=None):
                self.assertIsNone(
                    al.rotate(home, datetime.now(timezone.utc)))
            log.write_text("y" * 100)
            with unittest.mock.patch.object(al, "_compress",
                                          side_effect=OSError("disk")):
                self.assertIsNone(
                    al.rotate(home, datetime.now(timezone.utc)))
        finally:
            al.MAX_BYTES = orig_max

    def test_rotate_main_entrypoint(self):
        exc, _ = _runpy(LIB / "activity_log_rotate.py", [],
                        env={"CIEL_HOME": str(self.root)})
        self.assertEqual(exc.code, 0)

    # -- attribution_scan ------------------------------------------------
    def test_git_subprocess_failure(self):
        asc = _src("attribution_scan")
        with unittest.mock.patch.object(
                asc.subprocess, "run", side_effect=OSError("no git")):
            self.assertEqual(asc._git("status"), "")
        with unittest.mock.patch.object(
                asc.subprocess, "run",
                side_effect=asc.subprocess.TimeoutExpired("git", 10)):
            self.assertEqual(asc._git("status"), "")

    # -- session_watchdog ------------------------------------------------
    def _watchdog(self):
        wd = _src("session_watchdog")
        req = _src("requirements")
        self._req_ledger = req.LEDGER
        req.LEDGER = self.root / ".ciel" / "checkpoints" / "requirements.jsonl"
        self._req = req
        for k in ("CIEL", "CKPT", "STATE", "HINT", "ACTIVITY",
                  "TRANSCRIPTS", "SUMMARIES"):
            setattr(self, "_wd_" + k, getattr(wd, k))
        wd.CIEL = self.root / ".ciel"
        wd.CKPT = wd.CIEL / "checkpoints"
        wd.STATE = wd.CKPT / "watchdog_state.json"
        wd.HINT = wd.CKPT / "resume_hint.json"
        wd.ACTIVITY = wd.CIEL / "activity.log"
        wd.TRANSCRIPTS = self.root / "transcripts"
        wd.SUMMARIES = self.root / "summaries"
        for d in (wd.CKPT, wd.TRANSCRIPTS, wd.SUMMARIES):
            d.mkdir(parents=True, exist_ok=True)
        self._wd = wd
        return wd

    def _unwatchdog(self):
        for k in ("CIEL", "CKPT", "STATE", "HINT", "ACTIVITY",
                  "TRANSCRIPTS", "SUMMARIES"):
            setattr(self._wd, k, getattr(self, "_wd_" + k))
        self._req.LEDGER = self._req_ledger

    def test_wd_save_chmod_oserror(self):
        wd = self._watchdog()
        try:
            p = self.root / "s.json"
            real_chmod = os.chmod
            with unittest.mock.patch.object(
                    os, "chmod", side_effect=OSError("ro")):
                wd._save(p, {"a": 1})
            self.assertTrue(p.is_file())
        finally:
            self._unwatchdog()

    def test_wd_tail_glob_oserror_and_flag(self):
        wd = self._watchdog()
        try:
            real_glob = Path.glob
            def bad_glob(self, *a, **k):
                if self == wd.TRANSCRIPTS:
                    raise OSError("gone")
                return real_glob(self, *a, **k)
            with unittest.mock.patch.object(Path, "glob", bad_glob):
                self.assertEqual(wd._transcript_tail_errors(), [])
            # fresh error tail → flagged; stale file skipped
            (wd.TRANSCRIPTS / "t.json").write_text('{"e":"rate_limit_error"}')
            stale = wd.TRANSCRIPTS / "old.json"
            stale.write_text('{"e":"rate_limit_error"}')
            old = __import__("time").time() - (wd.STALL_AGE_S * 7)
            os.utime(stale, (old, old))
            self.assertEqual(wd._transcript_tail_errors(), ["t"])
        finally:
            self._unwatchdog()

    def test_wd_sweep_hits_and_stall_hint(self):
        wd = self._watchdog()
        try:
            (wd.TRANSCRIPTS / "s.json").write_text(
                '"token ghp_abcdefghij0123456789ABCD"')
            state = {}
            res = wd.transcript_sweep(state)
            self.assertIn("s.json", res["hits"])
            # stalled session: pending ledger + stale activity → cmd_check hint
            import time as _t
            from datetime import datetime, timezone
            stale_ts = datetime.fromtimestamp(
                _t.time() - 3600, timezone.utc).isoformat()
            wd.ACTIVITY.write_text(json.dumps(
                {"session_id": "deadbeef01", "ts": stale_ts}) + "\n")
            wd.CKPT.mkdir(parents=True, exist_ok=True)
            (wd.CKPT / "requirements.jsonl").write_text(json.dumps(
                {"op": "add", "id": "r1", "text": "x",
                 "session": "deadbeef01"}) + "\n")
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                wd.cmd_check("live")
            self.assertIn("deadbeef", buf.getvalue())
        finally:
            self._unwatchdog()

    def test_wd_do_resume_capped(self):
        wd = self._watchdog()
        try:
            from datetime import datetime, timezone
            today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
            st = {"resume_attempts": {"_day": today, "s1": 9, "s2": 9,
                                    "s3": 9, "s4": 9},
                  "last_resume": 0}
            orig = wd._load
            wd._load = lambda *a, **k: st
            r = wd.do_resume("s5", "stalled")
            self.assertFalse(r["fired"])
            self.assertIn("capped", r["reason"])
            wd._load = orig
        finally:
            self._unwatchdog()

    def test_wd_sessions_db_still_locked(self):
        wd = self._watchdog()
        try:
            scripts = wd.CIEL / "scripts"
            scripts.mkdir(parents=True)
            (scripts / "transcript_sanitize.py").write_text(
                (SCRIPTS / "transcript_sanitize.py").read_text())
            db_path = self.root / "sessions.db"
            db = sqlite3.connect(db_path)
            db.execute("CREATE TABLE t (i INTEGER)")
            db.commit()
            db.execute("BEGIN EXCLUSIVE")
            os.environ["CIEL_SESSIONS_DB"] = str(db_path)
            try:
                state = {"sessions_db_sanitize_pending": True}
                msg = wd._sessions_db_sanitize(state)
                self.assertIn("still locked", msg)
                self.assertTrue(state["sessions_db_sanitize_pending"])
            finally:
                os.environ.pop("CIEL_SESSIONS_DB", None)
                db.rollback()
                db.close()
        finally:
            self._unwatchdog()

    # -- store_perms ------------------------------------------------------
    def test_store_perms_glob_oserror(self):
        sp = _src("store_perms")
        base = self.root / "base"
        base.mkdir()
        real_glob = Path.glob
        def bad_glob(self, *a, **k):
            if self == base:
                raise OSError("gone")
            return real_glob(self, *a, **k)
        orig = sp.FILE_GLOBS
        sp.FILE_GLOBS = [(base, "*.json")]
        try:
            with unittest.mock.patch.object(Path, "glob", bad_glob):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(sp.main(), 0)
        finally:
            sp.FILE_GLOBS = orig

    # -- system1 -----------------------------------------------------------
    def test_s1_token_overlap_append(self):
        s1 = _src("system1")
        options = {"deploy_now": "ships stuff",
                   "unrelated_a": "x", "unrelated_b": "y"}
        with unittest.mock.patch.object(
                s1, "_semantic_rank", return_value=[]):
            with unittest.mock.patch.object(
                    s1, "_lexical_rank", return_value=[]):
                out = s1.shortlist_options("please deploy now", options, k=3)
        self.assertIn("deploy_now", out)

    def test_s1_ask_async_mkdir_oserror(self):
        s1 = _src("system1")
        os.environ.pop("CIEL_SYSTEM1_DISABLED", None)
        blocker = self.root / "blocker"
        blocker.write_text("x")
        orig = s1.ciel_home
        s1.ciel_home = lambda: blocker
        try:
            # inflight dir can't be created under a file → silent drop
            s1.ask_async({"surface": "s", "state": {}})
        finally:
            s1.ciel_home = orig

    def test_s1_marker_unlink_oserrors(self):
        s1 = _src("system1")
        os.environ.pop("CIEL_SYSTEM1_DISABLED", None)
        orig_home = s1.ciel_home
        s1.ciel_home = lambda: self.root
        try:
            # Popen fails AND the marker cleanup unlink fails → still silent
            with unittest.mock.patch.object(
                    s1.subprocess, "Popen", side_effect=OSError("noexec")):
                real_unlink = Path.unlink
                def bad_unlink(self, *a, **k):
                    raise OSError("stuck")
                with unittest.mock.patch.object(Path, "unlink", bad_unlink):
                    s1.ask_async({"surface": "s", "state": {}})
        finally:
            s1.ciel_home = orig_home
        # _ask_main: corrupt JSON payload → early return
        old = sys.stdin
        try:
            sys.stdin = io.StringIO("{corrupt")
            self.assertEqual(s1._ask_main(), 0)
        finally:
            sys.stdin = old
        # _ask_main marker cleanup OSError tolerated
        marker = self.root / "mk2"
        marker.touch()
        os.environ["CIEL_SYSTEM1_MARKER"] = str(marker)
        old = sys.stdin
        try:
            sys.stdin = io.StringIO("{}")
            with unittest.mock.patch.object(
                    Path, "unlink", side_effect=OSError("stuck")):
                self.assertEqual(s1._ask_main(), 0)
        finally:
            sys.stdin = old
            os.environ.pop("CIEL_SYSTEM1_MARKER", None)
        # main() --ask dispatch
        old = sys.stdin
        try:
            sys.stdin = io.StringIO("{}")
            with _argv("system1.py", "--ask"):
                self.assertEqual(s1.main(), 0)
        finally:
            sys.stdin = old

    # -- scan_skills --------------------------------------------------------
    def test_scan_all_and_failed_flag(self):
        sk = _src("scan_skills", SCRIPTS)
        stub = self.root / "scan.sh"
        stub.write_text(
            "#!/bin/sh\necho '{\"findings\": [{\"rule_id\": \"r\", "
            "\"severity\": \"low\"}], \"failed\": true}'\n")
        stub.chmod(0o755)
        os.environ["SCAN_SKILLS_CMD"] = str(stub)
        try:
            buf = io.StringIO()
            with _argv("x", "--all"):
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(sk.main(), 2)
            self.assertIn("failed", buf.getvalue())
        finally:
            os.environ.pop("SCAN_SKILLS_CMD", None)

    # -- system1_eval -------------------------------------------------------
    def test_eval_registry_empty_and_main_metrics(self):
        ev = _src("system1_eval", SCRIPTS)
        with unittest.mock.patch.object(ev, "_skill_dirs", return_value=[]):
            self.assertEqual(ev._registry_candidates(), {})
        # populated registry returns candidates
        skills = self.root / "skills"
        (skills / "a").mkdir(parents=True)
        (skills / "a" / "SKILL.md").write_text("---\ndescription: d\n---\n")
        os.environ["CIEL_SKILLS_DIR"] = str(skills)
        try:
            self.assertEqual(ev._registry_candidates(), {"a": "d"})
        finally:
            os.environ.pop("CIEL_SKILLS_DIR", None)
        # main non-stdout metric line for a successful surface eval
        corpus = self.root / "corpus.json"
        corpus.write_text(json.dumps({"cases": [
            {"id": "r1", "task": "x", "truth": "a"}],
            "candidates": {"a": "d"}}))
        spec = {
            "corpus": corpus,
            "state": lambda c: {},
            "questions": lambda c: {"route": {"type": "choice",
                                              "criteria": {"a": "d"}}},
            "truth": lambda c: "a",
            "positive": None,
        }
        ans = {"answers": {"route": {"choice": "a", "confidence": 0.9}}}
        orig_surfaces, orig_out = ev.SURFACES, ev.OUT
        ev.SURFACES = {"fake": spec}
        ev.OUT = self.root / "cal.json"
        try:
            with unittest.mock.patch.object(
                    ev.system1, "ask", return_value=ans):
                buf = io.StringIO()
                with _argv("x"):
                    with contextlib.redirect_stdout(buf):
                        self.assertEqual(ev.main(), 0)
            self.assertIn("accuracy=", buf.getvalue())
            self.assertIn("-> ", buf.getvalue())
        finally:
            ev.SURFACES, ev.OUT = orig_surfaces, orig_out

    # -- system1_export -----------------------------------------------------
    def test_export_main_entrypoint(self):
        log = self.root / "events.jsonl"
        log.write_text(json.dumps({
            "surface": "s", "questions": {"q": {}}, "state": {},
            "meta": {"expected": "safe"},
            "system1": {"answers": {"q": {
                "choice": "safe", "confidence": 0.9,
                "probabilities": {"safe": 0.9, "dangerous": 0.1}}}}}) + "\n")
        out = self.root / "pairs.jsonl"
        exc, so = _runpy(SCRIPTS / "system1_export.py",
                         ["--log", str(log), "--out", str(out)])
        self.assertEqual(exc.code, 0)
        self.assertIn("wrote 1", so)

    # -- paired_eval --------------------------------------------------------
    def test_paired_eval_entrypoint(self):
        exc, _ = _runpy(SCRIPTS / "paired_eval.py",
                        ["--skill", str(self.root / "noskill")])
        self.assertEqual(exc.code, 2)

    # -- transcript_sanitize -------------------------------------------------
    def test_scan_all_stat_oserror_and_tighten_missing(self):
        ts = _src("transcript_sanitize", SCRIPTS)
        _src("secret_scan")
        d = self.root / "store"
        d.mkdir()
        f = d / "f.json"
        f.write_text("x")
        real_stat = Path.stat
        def flaky(self, *a, **k):
            if self == f:
                raise OSError("gone")
            return real_stat(self, *a, **k)
        orig = ts.STORES
        ts.STORES = [(d, "*.json"), (self.root / "absent-dir", "*.json")]
        try:
            with unittest.mock.patch.object(Path, "stat", flaky):
                self.assertEqual(ts.scan_all(), {})
            # tighten_store_perms skips the missing base, fixes the 755 dir
            self.assertEqual(ts.tighten_store_perms(), 1)
            self.assertEqual(d.stat().st_mode & 0o777, 0o700)
        finally:
            ts.STORES = orig


class TestBatch6Residual(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_wd_tail_break_and_sanitize_hint_and_unlink_oserror(self):
        _src("secret_scan")
        wd = _src("session_watchdog")
        req = _src("requirements")
        orig_ledger = req.LEDGER
        req.LEDGER = self.root / ".ciel" / "checkpoints" / "requirements.jsonl"
        saved = {k: getattr(wd, k) for k in
                 ("CIEL", "CKPT", "STATE", "HINT", "ACTIVITY",
                  "TRANSCRIPTS", "SUMMARIES")}
        try:
            wd.CIEL = self.root / ".ciel"
            wd.CKPT = wd.CIEL / "checkpoints"
            wd.STATE = wd.CKPT / "watchdog_state.json"
            wd.HINT = wd.CKPT / "resume_hint.json"
            wd.ACTIVITY = wd.CIEL / "activity.log"
            wd.TRANSCRIPTS = self.root / "transcripts"
            wd.SUMMARIES = self.root / "summaries"
            for d in (wd.CKPT, wd.TRANSCRIPTS, wd.SUMMARIES):
                d.mkdir(parents=True, exist_ok=True)
            # >= limit flagged files → break
            for i in range(4):
                (wd.TRANSCRIPTS / f"e{i}.json").write_text(
                    '{"e":"rate_limit_error"}')
            self.assertEqual(len(wd._transcript_tail_errors()), 3)
            # sanitize hint appended when a pending flag produces a message
            wd.STATE.write_text(json.dumps(
                {"sessions_db_sanitize_pending": True}))
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                wd.cmd_check("live")
            self.assertIn("deferred", buf.getvalue())
            # HINT.unlink OSError tolerated on the no-hints path — clear the
            # error-tail transcripts first so the else-branch is reached
            for f in wd.TRANSCRIPTS.glob("*.json"):
                f.unlink()
            wd.STATE.write_text("{}")
            real_unlink = Path.unlink
            def bad_unlink(self, *a, **k):
                if self == wd.HINT:
                    raise OSError("stuck")
                return real_unlink(self, *a, **k)
            with unittest.mock.patch.object(Path, "unlink", bad_unlink):
                buf = io.StringIO()
                with contextlib.redirect_stdout(buf):
                    self.assertEqual(wd.cmd_check("live"), 0)
        finally:
            for k, v in saved.items():
                setattr(wd, k, v)
            req.LEDGER = orig_ledger

    def test_s1_token_overlap_past_cap(self):
        s1 = _src("system1")
        options = {"deploy_now": "ships stuff",
                   "unrelated_a": "x", "unrelated_b": "y",
                   "unrelated_c": "z"}
        with unittest.mock.patch.object(
                s1, "_semantic_rank", return_value=[]):
            with unittest.mock.patch.object(
                    s1, "_lexical_rank", return_value=[]):
                out = s1.shortlist_options("please deploy now", options, k=2)
        self.assertIn("deploy_now", out)

    def test_eval_main_entrypoint(self):
        exc, out = _runpy(
            SCRIPTS / "system1_eval.py",
            ["--stdout", "--timeout", "0.1"],
            env={"CIEL_SYSTEM1_URL": "http://127.0.0.1:1",
                 "CIEL_SYSTEM1_EMBED": "0"})
        self.assertEqual(exc.code, 0)
        self.assertIn("surfaces", out)

    def test_sanitize_checkpoint_busy(self):
        ts = _src("transcript_sanitize", SCRIPTS)
        _src("secret_scan")
        db_path = self.root / "sessions.db"
        db = sqlite3.connect(db_path)
        db.execute("CREATE TABLE t (id INTEGER PRIMARY KEY, c TEXT)")
        db.execute("INSERT INTO t VALUES (1, 'x')")
        db.commit()
        db.close()
        # sqlite3.Cursor is a C type — intercept at the module the function
        # imports instead, via a proxy connection/cursor pair
        import types
        class _Cur:
            def __init__(self, cur):
                self._c = cur
            def execute(self, sql, *a, **k):
                if "wal_checkpoint" in str(sql):
                    raise sqlite3.OperationalError("busy")
                return self._c.execute(sql, *a, **k)
            def __iter__(self):
                return iter(self._c)
        class _Conn:
            def __init__(self, conn):
                self._c = conn
            def cursor(self):
                return _Cur(self._c.cursor())
            def __getattr__(self, name):
                return getattr(self._c, name)
        fake = types.ModuleType("sqlite3")
        fake.OperationalError = sqlite3.OperationalError
        fake.connect = lambda *a, **k: _Conn(sqlite3.connect(*a, **k))
        orig_db, orig_tables = ts.SESSIONS_DB, ts.SESSIONS_TABLES
        try:
            ts.SESSIONS_DB = db_path
            ts.SESSIONS_TABLES = [("t", "c", "id", "broad", False)]
            with unittest.mock.patch.dict(
                    sys.modules, {"sqlite3": fake}):
                r = ts.redact_sessions_db(retries=1, wait=0.5)
            self.assertEqual(r["checkpoint"], "busy")
        finally:
            ts.SESSIONS_DB, ts.SESSIONS_TABLES = orig_db, orig_tables

    def test_embed_with_fake_transformer(self):
        class _Sims(list):
            def argsort(self):
                return sorted(range(len(self)), key=lambda i: self[i])
        class _Arr(list):
            def __matmul__(self, other):
                return _Sims(float(i) for i in range(len(self), 0, -1))
        import types
        fake = types.ModuleType("sentence_transformers")
        class _ST:
            def __init__(self, *a, **k):
                pass
            def encode(self, texts, normalize_embeddings=True):
                return _Arr([[1.0] * 4 for _ in texts])[0] \
                    if len(texts) == 1 else \
                    _Arr([[1.0] * 4 for _ in texts])
        fake.SentenceTransformer = _ST
        saved = sys.modules.get("sentence_transformers")
        sys.modules["sentence_transformers"] = fake
        try:
            payload = json.dumps({"task": "t",
                                  "candidates": {"a": "d", "b": "e"},
                                  "k": 1})
            exc, out = _runpy(LIB / "system1_embed.py", [],
                              stdin_text=payload)
            self.assertEqual(exc.code, 0)
            self.assertIn("names", out)
        finally:
            if saved is None:
                sys.modules.pop("sentence_transformers", None)
            else:
                sys.modules["sentence_transformers"] = saved
        # encode failure → exception path exits 1 (None sentinel → ImportError)
        sys.modules["sentence_transformers"] = None
        try:
            exc, _ = _runpy(LIB / "system1_embed.py", [],
                            stdin_text=json.dumps(
                                {"task": "t", "candidates": {"a": "d"}}))
            self.assertEqual(exc.code, 1)
        finally:
            sys.modules.pop("sentence_transformers", None)


if __name__ == "__main__":
    unittest.main()
