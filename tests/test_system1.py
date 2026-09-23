"""Tests for ciel.skill/init/hooks/lib/system1.py — the shared System-1
decision client. Stub HTTP servers stand in for laya-serve; CIEL_HOME is a
tempdir so cache/inflight/events stay hermetic."""

import http.server
import json
import os
import subprocess
import sys
import tempfile
import threading
import time
import unittest
import unittest.mock
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LIB = ROOT / "ciel.skill" / "init" / "hooks" / "lib"
sys.path.insert(0, str(LIB))

import system1

QUESTIONS = {
    "risk": {
        "type": "choice",
        "instructions": "dangerous?",
        "criteria": {"safe": "benign", "dangerous": "harmful"},
    }
}


def _serve(payload, counter=None):
    class H(http.server.BaseHTTPRequestHandler):
        def do_POST(self):
            if counter is not None:
                counter.append(1)
            self.send_response(200)
            self.send_header("content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps(payload).encode())

        def log_message(self, *a):
            pass

    srv = http.server.HTTPServer(("127.0.0.1", 0), H)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


class System1TestCase(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self._saved = {k: os.environ.get(k) for k in (
            "CIEL_HOME", "CIEL_SYSTEM1_DISABLED", "CIEL_SYSTEM1_URL",
            "CIEL_SYSTEM1_KEY")}
        os.environ["CIEL_HOME"] = self.tmp.name
        os.environ.pop("CIEL_SYSTEM1_DISABLED", None)

    def tearDown(self):
        for k, v in self._saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v


class TestAsk(System1TestCase):
    def test_choice_roundtrip(self):
        srv = _serve({"answers": {"risk": {"choice": "dangerous",
                                           "confidence": 0.8,
                                           "probabilities": {"dangerous": 0.8}}},
                      "routing": {"model": "english"}})
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = f"http://127.0.0.1:{srv.server_port}"
        os.environ["CIEL_SYSTEM1_KEY"] = "k"
        v = system1.ask_choice({"command": "rm -rf /"}, "risk",
                               QUESTIONS["risk"]["instructions"],
                               QUESTIONS["risk"]["criteria"])
        self.assertEqual("dangerous", v["choice"])
        self.assertEqual(0.8, v["confidence"])
        self.assertEqual("english", v["model"])

    def test_disabled_returns_none(self):
        os.environ["CIEL_SYSTEM1_DISABLED"] = "1"
        self.assertIsNone(system1.ask({"a": 1}, QUESTIONS))

    def test_unreachable_returns_none_fast(self):
        os.environ["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:9"
        t0 = time.monotonic()
        self.assertIsNone(system1.ask({"a": 1}, QUESTIONS))
        self.assertLess(time.monotonic() - t0, 5)

    def test_bad_answer_shape_returns_none(self):
        srv = _serve({"answers": "oops"})
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = f"http://127.0.0.1:{srv.server_port}"
        self.assertIsNone(system1.ask({"a": 1}, QUESTIONS))


class TestAskMain(System1TestCase):
    """End-to-end --ask subprocess: cache, events log, marker cleanup."""

    def _run_ask(self, payload, env_extra=None):
        env = dict(os.environ)
        env.update(env_extra or {})
        return subprocess.run(
            [sys.executable, str(LIB / "system1.py"), "--ask"],
            input=json.dumps(payload), capture_output=True, text=True,
            env=env, check=False,
        )

    def _payload(self, surface="test_surface", command="ls"):
        return {"surface": surface, "state": {"command": command},
                "questions": QUESTIONS,
                "meta": {"ts": "t1", "runtime": "test"}}

    def test_ask_writes_event_and_cleans_marker(self):
        counter = []
        srv = _serve({"answers": {"risk": {"choice": "safe",
                                           "confidence": 0.7}},
                      "model": "english"}, counter)
        self.addCleanup(srv.shutdown)
        marker = Path(self.tmp.name) / "system1" / "inflight" / "m1"
        marker.parent.mkdir(parents=True)
        marker.touch()
        proc = self._run_ask(self._payload(), {
            "CIEL_SYSTEM1_URL": f"http://127.0.0.1:{srv.server_port}",
            "CIEL_SYSTEM1_KEY": "k",
            "CIEL_SYSTEM1_MARKER": str(marker),
        })
        self.assertEqual(0, proc.returncode)
        self.assertFalse(marker.exists())
        log = Path(self.tmp.name) / "system1" / "events.jsonl"
        rec = json.loads(log.read_text().splitlines()[0])
        self.assertEqual("test_surface", rec["surface"])
        self.assertEqual("t1", rec["ts"])
        self.assertFalse(rec["cache_hit"])
        self.assertEqual("safe", rec["system1"]["answers"]["risk"]["choice"])
        self.assertEqual(1, len(counter))

        # Second identical ask hits the cache: no HTTP call, cache_hit=true.
        proc = self._run_ask(self._payload(), {
            "CIEL_SYSTEM1_URL": f"http://127.0.0.1:{srv.server_port}",
            "CIEL_SYSTEM1_KEY": "k",
        })
        self.assertEqual(0, proc.returncode)
        rec = json.loads(log.read_text().splitlines()[1])
        self.assertTrue(rec["cache_hit"])
        self.assertEqual(1, len(counter))

    def test_ask_offline_logs_null_verdict(self):
        proc = self._run_ask(self._payload(), {
            "CIEL_SYSTEM1_URL": "http://127.0.0.1:9",
            "CIEL_SYSTEM1_KEY": "k",
        })
        self.assertEqual(0, proc.returncode)
        log = Path(self.tmp.name) / "system1" / "events.jsonl"
        rec = json.loads(log.read_text().splitlines()[0])
        self.assertIsNone(rec["system1"])

    def test_ask_bad_stdin_exits_clean(self):
        proc = subprocess.run(
            [sys.executable, str(LIB / "system1.py"), "--ask"],
            input="not json", capture_output=True, text=True,
            env=dict(os.environ), check=False)
        self.assertEqual(0, proc.returncode)


class TestConcurrencyBound(System1TestCase):
    def test_saturated_inflight_drops_request(self):
        d = Path(self.tmp.name) / "system1" / "inflight"
        d.mkdir(parents=True)
        for i in range(system1.MAX_INFLIGHT):
            (d / f"live{i}").touch()
        spawned = []
        original = subprocess.Popen
        system1.subprocess.Popen = lambda *a, **k: spawned.append(1) or original(*a, **k)
        try:
            system1.ask_async({"surface": "s", "state": {}, "questions": {}})
        finally:
            system1.subprocess.Popen = original
        self.assertEqual([], spawned)

    def test_stale_markers_are_reaped(self):
        d = Path(self.tmp.name) / "system1" / "inflight"
        d.mkdir(parents=True)
        stale = d / "old"
        stale.touch()
        old = time.time() - system1.INFLIGHT_STALE_S - 10
        os.utime(stale, (old, old))
        self.assertEqual(0, system1._inflight_count())
        self.assertFalse(stale.exists())

    def test_disabled_ask_async_noops(self):
        os.environ["CIEL_SYSTEM1_DISABLED"] = "1"
        d = Path(self.tmp.name) / "system1" / "inflight"
        system1.ask_async({"surface": "s", "state": {}, "questions": {}})
        self.assertFalse(d.exists())


class TestBanding(System1TestCase):
    def test_flag_choice_dominates(self):
        band = system1._band("pre_tool_risk",
                             {"risk": {"choice": "dangerous",
                                       "confidence": 0.9}})
        self.assertEqual("flag", band)

    def test_low_confidence_safe_is_uncertain(self):
        band = system1._band("pre_tool_risk",
                             {"risk": {"choice": "safe",
                                       "confidence": 0.05}})
        self.assertEqual("uncertain", band)

    def test_confident_safe_passes(self):
        band = system1._band("pre_tool_risk",
                             {"risk": {"choice": "safe",
                                       "confidence": 0.9}})
        self.assertEqual("pass", band)

    def test_tau_env_override(self):
        os.environ["CIEL_SYSTEM1_TAU"] = "0.9"
        self.addCleanup(os.environ.pop, "CIEL_SYSTEM1_TAU")
        band = system1._band("pre_tool_risk",
                             {"risk": {"choice": "safe",
                                       "confidence": 0.5}})
        self.assertEqual("uncertain", band)

    def test_prescreen_surface_flags_escalate(self):
        band = system1._band("council_prescreen",
                             {"scope": {"choice": "escalate",
                                        "confidence": 0.9}})
        self.assertEqual("flag", band)

    def test_unknown_surface_band(self):
        band = system1._band("never_seen",
                             {"q": {"choice": "x", "confidence": 0.9}})
        self.assertEqual("pass", band)


class TestEventLog(System1TestCase):
    def test_rotation(self):
        log = Path(self.tmp.name) / "system1" / "events.jsonl"
        log.parent.mkdir(parents=True)
        log.write_text("x" * (system1.EVENTS_LOG_MAX + 1))
        system1._append_event({"ts": "t", "surface": "s"})
        rotated = Path(self.tmp.name) / "system1" / "events.jsonl.1"
        self.assertTrue(rotated.is_file())
        self.assertLess(log.stat().st_size, 1024)


class TestToolState(unittest.TestCase):
    def test_read_only_tool(self):
        s = system1.tool_state("read", "", "/tmp/x.txt")
        self.assertEqual(s["reversibility"], "read-only; no state change")
        self.assertNotIn("targets_sensitive_path", s)

    def test_write_tool_sensitive_path(self):
        s = system1.tool_state("write", "", "$HOME/.ssh/authorized_keys")
        self.assertTrue(s["targets_sensitive_path"])
        self.assertTrue(any("credential" in e for e in s["side_effects"]))

    def test_exec_command(self):
        s = system1.tool_state("exec", "rm -rf /tmp/x", "")
        self.assertIn("rm -rf /tmp/x", s["action"])
        self.assertIn("irreversible", s["reversibility"])

    def test_no_policy_leak(self):
        s = system1.tool_state("exec", "sudo apt install x", "")
        self.assertNotIn("deny", json.dumps(s))
        self.assertNotIn("policy", json.dumps(s))


class TestShortlist(unittest.TestCase):
    OPTIONS = {n: f"desc for {n}" for n in
               ["docker", "git", "article-writing", "security-review",
                "python-development"] + [f"skill-{i}" for i in range(40)]}

    def setUp(self):
        # hermetic: never spawn the real embedder venv in unit tests
        self._p = unittest.mock.patch.dict(
            os.environ, {"CIEL_SYSTEM1_EMBED": "0"})
        self._p.start()
        self.addCleanup(self._p.stop)

    def test_small_set_passthrough(self):
        opts = {"a": "x", "b": "y"}
        self.assertEqual(system1.shortlist_options("t", opts), opts)

    def test_name_token_match_survives(self):
        out = system1.shortlist_options("run docker compose up", self.OPTIONS,
                                        k=5)
        self.assertIn("docker", out)

    def test_lexical_relevance(self):
        out = system1.shortlist_options(
            "audit the PR for credential leaks", self.OPTIONS, k=5)
        self.assertIn("security-review", out)

    def test_semantic_fallback_when_no_venv(self):
        with unittest.mock.patch.object(system1, "ciel_home",
                                        return_value=Path("/nonexistent")):
            out = system1.shortlist_options("anything", self.OPTIONS, k=5)
        self.assertEqual(len(out), 5)

    def test_embed_disabled_env(self):
        with unittest.mock.patch.dict(os.environ,
                                      {"CIEL_SYSTEM1_EMBED": "0"}):
            self.assertEqual(
                system1._semantic_rank("t", self.OPTIONS, 5), [])


class TestDecideMain(System1TestCase):
    def test_decide_prints_verdict_and_logs(self):
        srv = _serve({"answers": {"done": {"type": "choice", "choice": "yes",
                                          "confidence": 0.9,
                                          "probabilities": {"yes": 0.9,
                                                            "no": 0.1}}},
                      "model": "stub"})
        port = srv.server_address[1]
        try:
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": f"http://127.0.0.1:{port}"}):
                proc = subprocess.run(
                    [sys.executable, system1.__file__, "--decide"],
                    input=json.dumps({
                        "surface": "completion_check",
                        "state": {"objective": "x"},
                        "questions": {"done": {"type": "choice",
                                               "instructions": "done?",
                                               "criteria": {"yes": "y",
                                                            "no": "n"}}}}),
                    capture_output=True, text=True, timeout=30)
            self.assertEqual(proc.returncode, 0)
            out = json.loads(proc.stdout)
            self.assertIn("answers", out)
            log = Path(self.tmp.name) / "system1" / "events.jsonl"
            events = [json.loads(l) for l in log.read_text().splitlines()]
            self.assertEqual(events[-1]["surface"], "completion_check")
        finally:
            srv.shutdown()

    def test_decide_offline_prints_null(self):
        with unittest.mock.patch.dict(
                os.environ, {"CIEL_SYSTEM1_URL": "http://127.0.0.1:9"}):
            proc = subprocess.run(
                [sys.executable, system1.__file__, "--decide"],
                input=json.dumps({"surface": "s", "state": {},
                                  "questions": {}}),
                capture_output=True, text=True, timeout=30)
        self.assertEqual(proc.returncode, 0)
        self.assertEqual(proc.stdout.strip(), "null")


if __name__ == "__main__":
    unittest.main()
