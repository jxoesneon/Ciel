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


def _serve(payload, counter=None, bodies=None, status=200):
    class H(http.server.BaseHTTPRequestHandler):
        def do_POST(self):
            if counter is not None:
                counter.append(1)
            if bodies is not None:
                n = int(self.headers.get("content-length", 0))
                bodies.append(self.rfile.read(n).decode())
            self.send_response(status)
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
            "CIEL_SYSTEM1_KEY", "CIEL_SYSTEM1_MODEL",
            "CIEL_SYSTEM1_HOSTED", "CIEL_SYSTEM1_HOSTED_URL",
            "CIEL_SYSTEM1_HOSTED_KEY", "CIEL_SYSTEM1_HOSTED_MODEL",
            "JEV_API_KEY")}
        os.environ["CIEL_HOME"] = self.tmp.name
        for k in ("CIEL_SYSTEM1_DISABLED", "CIEL_SYSTEM1_HOSTED_URL",
                  "CIEL_SYSTEM1_HOSTED_KEY", "CIEL_SYSTEM1_HOSTED_MODEL",
                  "JEV_API_KEY"):
            os.environ.pop(k, None)
        # Hosted failover off by default — otherwise a bare
        # CIEL_SYSTEM1_KEY in a test would promote to real egress.
        os.environ["CIEL_SYSTEM1_HOSTED"] = "off"

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


class TestEndpointResolution(System1TestCase):
    """CIEL_SYSTEM1_URL is a base URL — hosted Jev mounts under /api/v1,
    local laya-serve under /v1, and a full endpoint passes through."""

    def test_hosted_jev_gets_api_prefix(self):
        for u, want in (
                ("https://jev-agent.com",
                 "https://jev-agent.com/api/v1/systemone"),
                ("https://jev-agent.com/",
                 "https://jev-agent.com/api/v1/systemone"),
                ("https://www.jev-agent.com",
                 "https://www.jev-agent.com/api/v1/systemone"),
                ("https://jev-agent.com:443",
                 "https://jev-agent.com:443/api/v1/systemone")):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertEqual(want, system1._endpoint(), u)

    def test_local_and_oss_backends_get_v1(self):
        for u in ("http://127.0.0.1:8765", "http://localhost:8765",
                  "https://autojev.ai", "http://192.168.1.10:8765"):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertEqual(u.rstrip("/") + "/v1/systemone",
                                 system1._endpoint(), u)

    def test_full_endpoint_and_api_base_passthrough(self):
        for u, want in (
                ("https://jev-agent.com/api",
                 "https://jev-agent.com/api/v1/systemone"),
                ("http://127.0.0.1:8765/v1/systemone",
                 "http://127.0.0.1:8765/v1/systemone")):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertEqual(want, system1._endpoint(), u)

    def test_schemeless_and_userinfo_authorities(self):
        for u, want in (
                ("jev-agent.com",
                 "jev-agent.com/api/v1/systemone"),
                ("https://jev-agent.com:443@evil.example",
                 "https://jev-agent.com:443@evil.example/v1/systemone"),
                # authority ends at ? # \ — a later @ is not userinfo, so
                # the real connect-host is classified, not the spoof
                ("https://evil.example?x@jev-agent.com",
                 "https://evil.example?x@jev-agent.com/v1/systemone"),
                ("https://evil.example\\@jev-agent.com",
                 "https://evil.example\\@jev-agent.com/v1/systemone")):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertEqual(want, system1._endpoint(), u)

    def test_remote_detection(self):
        for u, want in (
                ("https://jev-agent.com", True),
                ("https://autojev.ai", True),
                ("http://127.0.0.1:8765", False),
                ("http://localhost:8765", False)):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertEqual(want, system1._remote(), u)

    def test_official_typesafe_api_gets_v1(self):
        for u in ("https://api.typesafe.ai", "https://typesafe.ai"):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertEqual(u + "/v1/systemone",
                                 system1._endpoint(), u)

    def test_hosted_detection(self):
        for u, want in (
                ("https://api.typesafe.ai", True),
                ("https://jev-agent.com", True),
                ("https://typesafe.ai", True),
                ("https://autojev.ai", False),
                ("http://127.0.0.1:8765", False),
                ("https://lan-laya.internal:8765", False)):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertEqual(want, system1._hosted(), u)

    def test_hosted_model_never_local_checkpoint(self):
        # api.typesafe.ai 400s on unknown models and 422s when absent —
        # a hosted ask must send a Jev model id, never "typed-decisions".
        with unittest.mock.patch.dict(os.environ, {
                "CIEL_SYSTEM1_URL": "https://api.typesafe.ai",
                "CIEL_SYSTEM1_MODEL": "typed-decisions"}):
            os.environ.pop("CIEL_SYSTEM1_HOSTED_MODEL", None)
            self.assertEqual("jev-latest", system1._hosted_model())
        with unittest.mock.patch.dict(os.environ, {
                "CIEL_SYSTEM1_URL": "https://api.typesafe.ai",
                "CIEL_SYSTEM1_HOSTED_MODEL": "jev-preview"}):
            self.assertEqual("jev-preview", system1._hosted_model())
        with unittest.mock.patch.dict(os.environ, {
                "CIEL_SYSTEM1_URL": "http://127.0.0.1:8765",
                "CIEL_SYSTEM1_MODEL": "typed-decisions"}):
            self.assertEqual("typed-decisions", system1._model())

    def test_hosted_spoof_shapes_not_hosted(self):
        for u, want_host in (
                ("https://evil.com\\@api.typesafe.ai", "evil.com"),
                ("https://evil.com?x@api.typesafe.ai", "evil.com"),
                ("https://evil.com#x@api.typesafe.ai", "evil.com"),
                ("https://api.typesafe.ai.evil.com",
                 "api.typesafe.ai.evil.com")):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertFalse(system1._hosted(), u)
                self.assertEqual(want_host, system1._host(), u)

    def test_hosted_requires_https_egress(self):
        # A cleartext Bearer is a credential leak — hosted asks over http
        # or schemeless URLs fail closed before any bytes leave.
        for u, want in (
                ("https://api.typesafe.ai", True),
                ("http://api.typesafe.ai", False),
                ("api.typesafe.ai", False),
                ("http://192.168.1.10:8765", True),
                ("http://127.0.0.1:8765", True)):
            with unittest.mock.patch.dict(
                    os.environ, {"CIEL_SYSTEM1_URL": u}):
                self.assertEqual(want, system1._egress_allowed(), u)

    def test_remote_ask_redacts_state_on_the_wire(self):
        bodies = []
        srv = _serve({"answers": {"risk": {"choice": "safe",
                                           "confidence": 0.7}},
                      "model": "english"}, bodies=bodies)
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{srv.server_port}")
        os.environ["CIEL_SYSTEM1_KEY"] = "k"
        # Redaction keys off the wire target's host, so simulate a remote
        # host for a loopback mock.
        with unittest.mock.patch.object(
                system1, "_host_of", return_value="example.com"):
            system1.ask({"command": "aws AKIAIOSFODNN7EXAMPLE"}, QUESTIONS)
        self.assertEqual(1, len(bodies))
        sent = json.loads(bodies[0])
        self.assertIn("[REDACTED:aws_access_key]",
                      sent["state"]["command"])
        self.assertNotIn("AKIAIOSFODNN7EXAMPLE",
                         sent["state"]["command"])


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


class TestCompletionCheck(System1TestCase):
    def test_completion_check_complete_with_score(self):
        srv = _serve({
            "answers": {
                "done": {
                    "type": "choice",
                    "choice": "complete",
                    "confidence": 0.95,
                    "probabilities": {"complete": 0.95, "incomplete": 0.05},
                },
                "evidence_score": {
                    "type": "score",
                    "score": 5,
                    "probabilities": {"5": 0.9, "4": 0.1},
                },
            },
            "model": "typed-decisions",
        })
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = f"http://127.0.0.1:{srv.server_port}"
        os.environ["CIEL_SYSTEM1_KEY"] = "k"

        res = system1.completion_check(
            objective="fix bug and run full test suite",
            evidence="pytest passed: 100% tests green",
            task_class="code_change",
            with_score=True,
            timeout=5.0,
        )
        self.assertIsNotNone(res)
        self.assertEqual("complete", res["choice"])
        self.assertEqual("pass", res["band"])
        self.assertEqual(5, res["score"])
        self.assertGreaterEqual(res["confidence"], 0.9)
        self.assertEqual("typed-decisions", res["model"])

        # Check logged to events.jsonl
        log = Path(self.tmp.name) / "system1" / "events.jsonl"
        events = [json.loads(l) for l in log.read_text().splitlines()]
        self.assertEqual("completion_check", events[-1]["surface"])
        self.assertEqual("pass", events[-1]["flag"])

    def test_completion_check_incomplete_flags(self):
        srv = _serve({
            "answers": {
                "done": {
                    "type": "choice",
                    "choice": "incomplete",
                    "confidence": 0.88,
                },
            },
            "model": "typed-decisions",
        })
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = f"http://127.0.0.1:{srv.server_port}"
        os.environ["CIEL_SYSTEM1_KEY"] = "k"

        res = system1.completion_check(
            objective="implement auth token validation",
            evidence="claim: should work now, no test run",
            task_class="code_change",
            timeout=5.0,
        )
        self.assertIsNotNone(res)
        self.assertEqual("incomplete", res["choice"])
        self.assertEqual("flag", res["band"])

    def test_completion_check_disabled_fail_open(self):
        os.environ["CIEL_SYSTEM1_DISABLED"] = "1"
        res = system1.completion_check(
            objective="obj",
            evidence="ev",
        )
        self.assertIsNone(res)

    def test_completion_check_offline_fail_open(self):
        os.environ["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:9"
        res = system1.completion_check(
            objective="obj",
            evidence="ev",
            timeout=0.2,
        )
        self.assertIsNone(res)


def _serve_batch(per_state, calls=None):
    """Stub /v1/systemone/batch — ``per_state`` maps a state index to the
    answer block the server returns for it."""
    class H(http.server.BaseHTTPRequestHandler):
        def do_POST(self):
            if calls is not None:
                calls.append(1)
            n = int(self.headers.get("content-length", 0))
            body = json.loads(self.rfile.read(n).decode() or "{}")
            results = []
            for i, _ in enumerate(body.get("states") or []):
                results.append({"answers": per_state(i),
                                "model": "english"})
            self.send_response(200)
            self.send_header("content-type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"results": results}).encode())

        def log_message(self, *a):
            pass

    srv = http.server.HTTPServer(("127.0.0.1", 0), H)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


class TestAskBatch(System1TestCase):
    """/v1/systemone/batch client — shared question set over many states."""

    def test_roundtrip_positional_results(self):
        srv = _serve_batch(lambda i: {
            "risk": {"choice": "safe" if i == 0 else "dangerous",
                     "confidence": 0.8}})
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{srv.server_port}")
        os.environ["CIEL_SYSTEM1_KEY"] = "k"
        out = system1.ask_batch(
            [{"command": "ls"}, {"command": "rm -rf /"}], QUESTIONS)
        self.assertEqual(2, len(out))
        self.assertEqual("safe", out[0]["answers"]["risk"]["choice"])
        self.assertEqual("dangerous", out[1]["answers"]["risk"]["choice"])

    def test_chunks_at_server_cap(self):
        calls = []
        srv = _serve_batch(
            lambda i: {"q": {"choice": "keep", "confidence": 0.9}},
            calls=calls)
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{srv.server_port}")
        os.environ["CIEL_SYSTEM1_KEY"] = "k"
        out = system1.ask_batch(
            [{"i": i} for i in range(130)],
            {"q": {"type": "choice", "instructions": "k?",
                   "criteria": {"keep": "k", "drop": "d"}}})
        self.assertEqual(3, len(calls))  # 64 + 64 + 2
        self.assertEqual(130, len(out))

    def test_offline_fail_open_and_empty(self):
        os.environ["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:9"
        self.assertIsNone(system1.ask_batch(
            [{"a": 1}], QUESTIONS, timeout=0.2))
        self.assertIsNone(system1.ask_batch([], QUESTIONS, timeout=0.2))


class TestContextSurfaces(System1TestCase):
    """context_select / memory_salience / context_compaction /
    mandate_canary surfaces."""

    def _batch_url(self, srv):
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{srv.server_port}")
        os.environ["CIEL_SYSTEM1_KEY"] = "k"

    def test_context_select_keep_drop_tau(self):
        # i=0 drops confidently, i=1 drops below tau, i=2 keeps
        def ans(i):
            return {"relevant": {
                "choice": "drop" if i < 2 else "keep",
                "confidence": 0.9 if i == 0 else 0.4}}
        srv = _serve_batch(ans)
        self.addCleanup(srv.shutdown)
        self._batch_url(srv)
        out = system1.context_select(
            "fix the login bug",
            {"a.md": "login notes", "b.md": "weather notes",
             "c.md": "auth spec"}, k=10)
        self.assertFalse(out["a.md"]["keep"])
        self.assertTrue(out["b.md"]["keep"])   # uncertain drop -> kept
        self.assertTrue(out["c.md"]["keep"])
        log = Path(self.tmp.name) / "system1" / "events.jsonl"
        events = [json.loads(l) for l in log.read_text().splitlines()]
        self.assertEqual("context_select", events[-1]["surface"])
        self.assertEqual(3, events[-1]["meta"]["batch"])

    def test_context_select_fail_open_keeps_everything(self):
        os.environ["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:9"
        out = system1.context_select("task", {"a": "d"},
                                     timeout=0.2)
        self.assertIsNone(out)  # caller treats None as keep-all
        self.assertEqual({}, system1.context_select(
            "task", {}, timeout=0.2))

    def test_memory_salience_verdict_and_event(self):
        srv = _serve({"answers": {"salience": {
            "choice": "store", "confidence": 0.9}}, "model": "english"})
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{srv.server_port}")
        os.environ["CIEL_SYSTEM1_KEY"] = "k"
        res = system1.memory_salience({"kind": "decision",
                                     "text": "user prefers tabs"})
        self.assertEqual("store", res["choice"])
        self.assertEqual("pass", res["band"])
        log = Path(self.tmp.name) / "system1" / "events.jsonl"
        events = [json.loads(l) for l in log.read_text().splitlines()]
        self.assertEqual("memory_salience", events[-1]["surface"])

    def test_compaction_decision_flags_pressure(self):
        srv = _serve({"answers": {
            "action": {"choice": "escalate", "confidence": 0.91},
            "pressure": {"score": 4.6, "confidence": 0.8}},
            "model": "english"})
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{srv.server_port}")
        os.environ["CIEL_SYSTEM1_KEY"] = "k"
        res = system1.compaction_decision(
            {"tokens_used": 30000, "budget": 32000})
        self.assertEqual("escalate", res["action"])
        self.assertEqual("flag", res["band"])
        self.assertEqual(4.6, res["pressure_score"])
        log = Path(self.tmp.name) / "system1" / "events.jsonl"
        events = [json.loads(l) for l in log.read_text().splitlines()]
        self.assertEqual("flag", events[-1]["flag"])

    def test_mandate_canary_drift_flags(self):
        srv = _serve({"answers": {"mandates": {
            "choice": "drifted", "confidence": 0.9}},
            "model": "english"})
        self.addCleanup(srv.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{srv.server_port}")
        os.environ["CIEL_SYSTEM1_KEY"] = "k"
        res = system1.mandate_canary(
            ["address user as Master"], "context without mandates")
        self.assertEqual("drifted", res["choice"])
        self.assertEqual("flag", res["band"])

    def test_surfaces_offline_fail_open(self):
        os.environ["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:9"
        self.assertIsNone(system1.memory_salience(
            {"kind": "x"}, timeout=0.2))
        self.assertIsNone(system1.compaction_decision(
            {"tokens_used": 1}, timeout=0.2))
        self.assertIsNone(system1.mandate_canary(
            ["m"], "ctx", timeout=0.2))


class TestHostedFailover(System1TestCase):
    """Hosted Jev primary + laya fallback: per-call failover, the persisted
    circuit breaker, and the hosted-off/no-key bypasses."""

    LOCAL_PAYLOAD = {"answers": {"risk": {"choice": "safe",
                                          "confidence": 0.9}},
                     "model": "laya-local"}

    def _pair(self, hosted_status, hosted_payload=None):
        h_counter, l_counter = [], []
        hosted = _serve(hosted_payload or {"error": "x"},
                        counter=h_counter, status=hosted_status)
        local = _serve(self.LOCAL_PAYLOAD, counter=l_counter)
        self.addCleanup(hosted.shutdown)
        self.addCleanup(local.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{local.server_port}")
        os.environ["CIEL_SYSTEM1_HOSTED_URL"] = (
            f"http://127.0.0.1:{hosted.server_port}")
        os.environ["CIEL_SYSTEM1_HOSTED_KEY"] = "test-hosted-key"
        os.environ["CIEL_SYSTEM1_HOSTED"] = "on"
        return h_counter, l_counter

    def test_hosted_primary_serves_and_skips_local(self):
        h_counter, l_counter = self._pair(
            200, {"answers": {"risk": {"choice": "dangerous",
                                       "confidence": 0.99}},
                  "model": "jev-1.13.0"})
        v = system1.ask({"a": 1}, QUESTIONS)
        self.assertEqual("jev-1.13.0", v["model"])
        self.assertEqual("dangerous", v["answers"]["risk"]["choice"])
        self.assertEqual(1, len(h_counter))
        self.assertEqual(0, len(l_counter), "hosted success must not "
                                            "touch the local daemon")

    def test_hosted_401_falls_back_and_trips_breaker(self):
        h_counter, l_counter = self._pair(401)
        v = system1.ask({"a": 1}, QUESTIONS)
        self.assertEqual("laya-local", v["model"])
        self.assertTrue(system1._hosted_down(), "401 must trip the breaker")
        # Breaker open: a second ask must not pay another hosted attempt.
        v2 = system1.ask({"a": 2}, QUESTIONS)
        self.assertEqual("laya-local", v2["model"])
        self.assertEqual(1, len(h_counter))
        self.assertEqual(2, len(l_counter))

    def test_hosted_429_and_5xx_trip(self):
        state = Path(self.tmp.name) / "system1" / "hosted_state.json"
        for status in (429, 503):
            state.unlink(missing_ok=True)  # reset the breaker per class
            h_counter, l_counter = self._pair(status)
            v = system1.ask({"a": 1}, QUESTIONS)
            self.assertEqual("laya-local", v["model"], f"status {status}")
            self.assertTrue(system1._hosted_down(), f"status {status}")
            self.assertEqual(1, len(h_counter), f"status {status}")

    def test_hosted_transport_failure_trips(self):
        l_counter = []
        local = _serve(self.LOCAL_PAYLOAD, counter=l_counter)
        self.addCleanup(local.shutdown)
        os.environ["CIEL_SYSTEM1_URL"] = (
            f"http://127.0.0.1:{local.server_port}")
        # Dead hosted port → connect refused → error-class trip.
        os.environ["CIEL_SYSTEM1_HOSTED_URL"] = "http://127.0.0.1:9"
        os.environ["CIEL_SYSTEM1_HOSTED_KEY"] = "test-hosted-key"
        os.environ["CIEL_SYSTEM1_HOSTED"] = "on"
        v = system1.ask({"a": 1}, QUESTIONS)
        self.assertEqual("laya-local", v["model"])
        self.assertTrue(system1._hosted_down())
        self.assertEqual(1, len(l_counter))

    def test_request_shape_4xx_does_not_trip(self):
        h_counter, l_counter = self._pair(400)
        v = system1.ask({"a": 1}, QUESTIONS)
        self.assertEqual("laya-local", v["model"])
        self.assertFalse(system1._hosted_down(),
                         "a 400 is a request bug, not an outage")

    def test_hosted_off_goes_straight_local(self):
        h_counter, l_counter = self._pair(200, self.LOCAL_PAYLOAD)
        os.environ["CIEL_SYSTEM1_HOSTED"] = "off"
        v = system1.ask({"a": 1}, QUESTIONS)
        self.assertEqual("laya-local", v["model"])
        self.assertEqual(0, len(h_counter))
        self.assertEqual(1, len(l_counter))

    def test_no_hosted_key_goes_straight_local(self):
        h_counter, l_counter = self._pair(200, self.LOCAL_PAYLOAD)
        os.environ.pop("CIEL_SYSTEM1_HOSTED_KEY", None)
        v = system1.ask({"a": 1}, QUESTIONS)
        self.assertEqual("laya-local", v["model"])
        self.assertEqual(0, len(h_counter))
        self.assertEqual(1, len(l_counter))

    def test_breaker_expires_and_retries_hosted(self):
        h_counter, l_counter = self._pair(
            200, {"answers": {"risk": {"choice": "safe",
                                       "confidence": 0.5}},
                  "model": "jev-x"})
        state = Path(self.tmp.name) / "system1" / "hosted_state.json"
        state.parent.mkdir(parents=True, exist_ok=True)
        state.write_text(json.dumps(
            {"down_until": time.time() - 1, "status": 401}))
        v = system1.ask({"a": 1}, QUESTIONS)
        self.assertEqual("jev-x", v["model"],
                         "expired breaker must retry hosted")
        self.assertEqual(1, len(h_counter))
        self.assertEqual(0, len(l_counter))

    def test_batch_serializes_with_per_state_failover(self):
        h_counter, l_counter = self._pair(401)
        out = system1.ask_batch([{"a": 1}, {"a": 2}], QUESTIONS, 3.0)
        self.assertIsNotNone(out)
        self.assertEqual(2, len(out))
        for r in out:
            self.assertEqual("laya-local", r["model"])
        self.assertEqual(1, len(h_counter),
                         "breaker must make hosted cost one attempt, not N")
        self.assertEqual(2, len(l_counter))


if __name__ == "__main__":
    unittest.main()
