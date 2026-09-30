#!/usr/bin/env python3
"""Comprehensive System-1 Verification Suite (Slot 5).

Covers:
1. Active vs Shadow modes (CIEL_SYSTEM1_MODE=active vs shadow vs off).
2. Cache hit/miss behavior & SHA-256 canonical state digest calculation parity.
3. Rapid sequential calls, thread hangs, resource and FD leak checks.
"""

import hashlib
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
HOOKS_DIR = ROOT / "ciel.skill" / "init" / "hooks"
CIEL_BIN = os.environ.get(
    "CIEL_RS_BIN",
    str(ROOT / "ciel.skill" / "init" / "ciel-rs" / "target" / "debug" / "ciel"),
)

sys.path.insert(0, str(LIB))
import system1  # noqa: E402


class MockLayaServer:
    """Mock Laya/System-1 server with controllable answers and request counting."""

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


class BaseSystem1Test(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="ciel-s1-slot5-"))
        self.ciel = self.tmp / ".ciel"
        self.ciel.mkdir(parents=True)
        self._saved = {k: os.environ.get(k) for k in (
            "CIEL_HOME", "HOME", "CIEL_BIN", "CIEL_SYSTEM1_MODE",
            "CIEL_SYSTEM1_URL", "CIEL_SYSTEM1_KEY", "CIEL_SYSTEM1_DISABLED"
        )}
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
            timeout=30,
        )
        return proc.returncode, proc.stdout, proc.stderr


class TestActiveVsShadowModes(BaseSystem1Test):
    """Test active and shadow modes (CIEL_SYSTEM1_MODE=active vs shadow vs off)."""

    def test_active_mode_semantic_intercept_blocks_execution(self):
        """In active mode, high-confidence dangerous verdict actively blocks PreToolUse."""
        server = MockLayaServer({
            "answers": {
                "risk": {
                    "choice": "dangerous",
                    "confidence": 0.95,
                    "probabilities": {"dangerous": 0.95, "safe": 0.05},
                }
            },
            "routing": {"model": "laya-flash"},
        })
        self.addCleanup(server.shutdown)

        self.env["CIEL_SYSTEM1_MODE"] = "active"
        self.env["CIEL_SYSTEM1_URL"] = server.url
        self.env["CIEL_SYSTEM1_KEY"] = "test-key"

        # A command that is benign under deterministic regex policy but deemed dangerous by System-1
        payload = json.dumps({
            "toolCall": {"name": "exec", "args": {"CommandLine": "python3 train_exploit.py"}},
            "conversationId": "conv-s1-active",
        })

        rc, stdout, _ = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        self.assertEqual(rc, 0)
        data = json.loads(stdout)
        self.assertEqual(data["decision"], "deny")
        self.assertIn("System-1 (Laya) semantic risk intercept", data["reason"])
        self.assertIn("flagged as dangerous", data["reason"])

        # Verify activity.log entry recorded the intercept
        log = (self.ciel / "activity.log").read_text(encoding="utf-8")
        self.assertIn("PreToolUse+System1Intercept", log)
        self.assertIn("dangerous", log)

        # Verify system1 event log recorded active pipeline event
        s1_log = (self.ciel / "system1" / "events.jsonl").read_text(encoding="utf-8")
        rec = json.loads(s1_log.splitlines()[-1])
        self.assertEqual(rec["surface"], "pre_tool_risk")
        self.assertEqual(rec["flag"], "flag")
        self.assertEqual(rec["meta"]["pipeline"], "active")

    def test_active_mode_override_allows_with_warning(self):
        """In active mode, allow_privileged converts intercept into override."""
        server = MockLayaServer({
            "answers": {
                "risk": {
                    "choice": "dangerous",
                    "confidence": 0.95,
                }
            },
            "model": "laya-flash",
        })
        self.addCleanup(server.shutdown)

        self.env["CIEL_SYSTEM1_MODE"] = "active"
        self.env["CIEL_SYSTEM1_URL"] = server.url
        self.env["CIEL_SYSTEM1_KEY"] = "test-key"
        (self.ciel / "allow_privileged").touch()

        payload = json.dumps({
            "toolCall": {"name": "exec", "args": {"CommandLine": "python3 train_exploit.py"}},
            "conversationId": "conv-s1-override",
        })

        rc, stdout, _ = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        self.assertEqual(rc, 0)
        data = json.loads(stdout)
        self.assertEqual(data["decision"], "allow")
        self.assertIn("overridden by allow_privileged", data["reason"])

        # Check activity log recorded PreToolUse+System1Override
        log = (self.ciel / "activity.log").read_text(encoding="utf-8")
        self.assertIn("PreToolUse+System1Override", log)

    def test_shadow_mode_non_blocking_and_dispatches_async(self):
        """In shadow mode, pretool allows immediately and detached shadow logs verdict."""
        server = MockLayaServer({
            "answers": {
                "risk": {
                    "choice": "dangerous",
                    "confidence": 0.95,
                }
            },
            "model": "laya-flash",
        })
        self.addCleanup(server.shutdown)

        self.env["CIEL_SYSTEM1_MODE"] = "shadow"
        self.env["CIEL_SYSTEM1_URL"] = server.url
        self.env["CIEL_SYSTEM1_KEY"] = "test-key"

        payload = json.dumps({
            "toolCall": {"name": "exec", "args": {"CommandLine": "python3 analyze_model.py"}},
            "conversationId": "conv-s1-shadow",
        })

        t0 = time.monotonic()
        rc, stdout, _ = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        elapsed = time.monotonic() - t0
        self.assertEqual(rc, 0)
        data = json.loads(stdout)
        self.assertEqual(data["decision"], "allow")
        # Ensure zero added latency on the fast path
        self.assertLess(elapsed, 0.5)

        # Wait for detached shadow child to finish writing to events.jsonl
        s1_log_file = self.ciel / "system1" / "events.jsonl"
        for _ in range(50):
            if s1_log_file.exists() and s1_log_file.stat().st_size > 0:
                break
            time.sleep(0.05)

        self.assertTrue(s1_log_file.exists())
        lines = s1_log_file.read_text(encoding="utf-8").splitlines()
        self.assertGreaterEqual(len(lines), 1)
        rec = json.loads(lines[-1])
        self.assertEqual(rec["surface"], "pre_tool_risk")
        self.assertEqual(rec["flag"], "flag")

    def test_off_mode_bypasses_all_system1_activity(self):
        """When CIEL_SYSTEM1_MODE=off, no network requests or shadow dispatches occur."""
        server = MockLayaServer({"answers": {}})
        self.addCleanup(server.shutdown)

        self.env["CIEL_SYSTEM1_MODE"] = "off"
        self.env["CIEL_SYSTEM1_URL"] = server.url

        payload = json.dumps({
            "toolCall": {"name": "read", "args": {"path": "/tmp/test.txt"}},
            "conversationId": "conv-off",
        })

        rc, stdout, _ = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        self.assertEqual(rc, 0)
        time.sleep(0.2)
        self.assertEqual(server.count(), 0)
        self.assertFalse((self.ciel / "system1" / "events.jsonl").exists())


class TestCacheAndDigestParity(BaseSystem1Test):
    """Verify cache hit/miss behavior and SHA-256 canonical state digest parity."""

    def test_canonical_digest_parity_across_edge_cases(self):
        """Verify Rust and Python calculate identical SHA-256 digests across complex inputs."""
        test_cases = [
            (
                {"tool": "exec", "command": "echo 'Hello World'", "path": ""},
                {"risk": {"type": "choice", "instructions": "dangerous?"}},
            ),
            (
                # Multi-byte UTF-8, accented chars, emojis
                {"tool": "write", "command": "", "path": "/data/é/üñíçødé/🚀.txt"},
                {"risk": {"criteria": {"safe": "sâfe", "dangerous": "dàngerous ⚠️"}}},
            ),
            (
                # Differently ordered keys in state and questions
                {"z": 100, "a": "first", "nested": {"k2": False, "k1": None}},
                {"q2": [1, 2, 3], "q1": {"inner": True}},
            ),
            (
                # Empty fields and boolean values
                {"tool": "", "command": "", "path": "", "empty_dict": {}, "empty_list": []},
                {},
            ),
        ]

        for state, questions in test_cases:
            # 1. Compute in Python
            py_canonical = json.dumps({"s": state, "q": questions}, sort_keys=True)
            py_digest = hashlib.sha256(py_canonical.encode("utf-8")).hexdigest()

            # 2. Write a unique cached result at the Python-computed cache path
            canned_answer = {
                "answers": {"test": {"choice": "cached_val", "confidence": 0.99}},
                "model": f"model-for-{py_digest[:8]}",
            }
            cache_file = self.ciel / "system1" / "cache" / f"{py_digest}.json"
            cache_file.parent.mkdir(parents=True, exist_ok=True)
            cache_file.write_text(json.dumps(canned_answer), encoding="utf-8")

            # 3. Query Rust via `system1 --decide` with dummy URL (must NOT hit network)
            payload = json.dumps({
                "surface": "pre_tool_risk",
                "state": state,
                "questions": questions,
            })
            env = dict(self.env, CIEL_SYSTEM1_URL="http://127.0.0.1:1")
            rc, stdout, stderr = self.run_rust("system1", "--decide", stdin=payload, env=env)
            self.assertEqual(rc, 0, f"Failed for digest {py_digest}: {stderr}")

            # 4. Assert Rust found the Python-cached file!
            rs_res = json.loads(stdout)
            self.assertEqual(
                rs_res["answers"]["test"]["choice"],
                "cached_val",
                f"Rust did not hit Python cache for digest {py_digest}",
            )

            # 5. Verify the event log records cache_hit=True
            log_lines = (self.ciel / "system1" / "events.jsonl").read_text().splitlines()
            rec = json.loads(log_lines[-1])
            self.assertTrue(rec["cache_hit"])

    def test_cache_miss_writes_cache_and_hit_reads_it(self):
        """End-to-end verification: Miss -> write -> Hit -> no network call."""
        server = MockLayaServer({
            "answers": {
                "risk": {"choice": "safe", "confidence": 0.88}
            },
            "model": "laya-cache-test",
        })
        self.addCleanup(server.shutdown)
        self.env["CIEL_SYSTEM1_URL"] = server.url

        state = {"tool": "read", "path": "/etc/hosts"}
        questions = {"risk": {"type": "choice"}}
        payload = json.dumps({
            "surface": "pre_tool_risk",
            "state": state,
            "questions": questions,
        })

        # Call 1: Cache Miss
        rc1, out1, _ = self.run_rust("system1", "--decide", stdin=payload)
        self.assertEqual(rc1, 0)
        self.assertEqual(server.count(), 1)
        res1 = json.loads(out1)
        self.assertEqual(res1["model"], "laya-cache-test")

        # Verify cache file was created
        caches = list((self.ciel / "system1" / "cache").glob("*.json"))
        self.assertEqual(len(caches), 1)

        # Call 2: Cache Hit
        rc2, out2, _ = self.run_rust("system1", "--decide", stdin=payload)
        self.assertEqual(rc2, 0)
        # Server request count MUST remain 1
        self.assertEqual(server.count(), 1)
        res2 = json.loads(out2)
        self.assertEqual(res1, res2)

        # Python can also read the cache written by Rust
        py_read = system1._cache_read(state, questions)
        self.assertIsNotNone(py_read)
        self.assertEqual(py_read["model"], "laya-cache-test")


class TestStressAndResourceSafety(BaseSystem1Test):
    """Rapid sequential calls, concurrency bounds, FD and memory leak checks."""

    def test_rapid_sequential_calls_stability(self):
        """Run 60 rapid sequential pretool and decide invocations to ensure zero hangs/leaks."""
        server = MockLayaServer({
            "answers": {
                "risk": {"choice": "safe", "confidence": 0.90}
            },
            "model": "laya-fast",
        })
        self.addCleanup(server.shutdown)
        self.env["CIEL_SYSTEM1_URL"] = server.url
        self.env["CIEL_SYSTEM1_MODE"] = "active"

        payload = json.dumps({
            "toolCall": {"name": "read", "args": {"path": "/tmp/bench.txt"}},
            "conversationId": "bench-seq",
        })

        latencies = []
        for i in range(60):
            t0 = time.monotonic()
            rc, out, _ = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
            lat = time.monotonic() - t0
            latencies.append(lat)
            self.assertEqual(rc, 0)

        # Cache should kick in after 1st call; subsequent calls should be ultra fast (< 100ms avg on mobile)
        cached_latencies = latencies[1:]
        avg_cached = sum(cached_latencies) / len(cached_latencies)
        self.assertLess(avg_cached, 0.10, f"Average cached latency too high: {avg_cached:.4f}s")
        # Only 1 network request made
        self.assertEqual(server.count(), 1)

    def test_max_inflight_concurrency_bounding(self):
        """Ensure saturated inflight drops async ask gracefully without blocking."""
        inflight_dir = self.ciel / "system1" / "inflight"
        inflight_dir.mkdir(parents=True)
        # Create 2 live markers (MAX_INFLIGHT = 2)
        (inflight_dir / "live1").touch()
        (inflight_dir / "live2").touch()

        server = MockLayaServer({"answers": {}})
        self.addCleanup(server.shutdown)
        self.env["CIEL_SYSTEM1_URL"] = server.url
        self.env["CIEL_SYSTEM1_MODE"] = "shadow"

        payload = json.dumps({
            "toolCall": {"name": "exec", "args": {"CommandLine": "ls /"}},
            "conversationId": "saturated",
        })

        rc, _, _ = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        self.assertEqual(rc, 0)
        time.sleep(0.2)

        # Because inflight was saturated, no additional requests dispatched
        self.assertEqual(server.count(), 0)

    def test_stale_marker_reaping(self):
        """Stale markers (>120s) must be reaped so pipeline recovers from crashes."""
        inflight_dir = self.ciel / "system1" / "inflight"
        inflight_dir.mkdir(parents=True)
        stale = inflight_dir / "stale_marker"
        stale.touch()
        old_time = time.time() - 300  # 5 minutes ago
        os.utime(stale, (old_time, old_time))

        server = MockLayaServer({
            "answers": {"risk": {"choice": "safe", "confidence": 0.99}},
            "model": "laya-reap",
        })
        self.addCleanup(server.shutdown)
        self.env["CIEL_SYSTEM1_URL"] = server.url
        self.env["CIEL_SYSTEM1_MODE"] = "shadow"

        payload = json.dumps({
            "toolCall": {"name": "read", "args": {"path": "/tmp/reap.txt"}},
            "conversationId": "reap-test",
        })

        rc, _, _ = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
        self.assertEqual(rc, 0)

        # Wait for stale marker to be reaped
        for _ in range(30):
            if not stale.exists():
                break
            time.sleep(0.05)

        self.assertFalse(stale.exists(), "Stale marker was not reaped!")

    def test_memory_and_fd_leak_under_stress(self):
        """Stress test: 100 rapid sequential calls ensuring zero FD leaks and flat memory."""
        import psutil

        server = MockLayaServer({
            "answers": {"risk": {"choice": "safe", "confidence": 0.99}},
            "model": "laya-stress",
        })
        self.addCleanup(server.shutdown)
        self.env["CIEL_SYSTEM1_URL"] = server.url
        self.env["CIEL_SYSTEM1_MODE"] = "active"

        proc = psutil.Process()
        initial_fds = proc.num_fds() if hasattr(proc, "num_fds") else len(os.listdir("/proc/self/fd"))
        initial_rss = proc.memory_info().rss

        payload = json.dumps({
            "toolCall": {"name": "read", "args": {"path": "/tmp/stress.txt"}},
            "conversationId": "stress-run",
        })

        for i in range(100):
            rc, stdout, stderr = self.run_rust("pretool", "--runtime", "antigravity", stdin=payload)
            self.assertEqual(rc, 0)

        final_fds = proc.num_fds() if hasattr(proc, "num_fds") else len(os.listdir("/proc/self/fd"))
        final_rss = proc.memory_info().rss

        # FD count must not grow (zero FD leaks)
        self.assertLessEqual(final_fds - initial_fds, 1, f"FD leak detected: {initial_fds} -> {final_fds}")
        # Process RSS delta must be within tight bound (< 20 MB)
        rss_growth_mb = (final_rss - initial_rss) / (1024 * 1024)
        self.assertLess(rss_growth_mb, 20.0, f"Excessive memory growth: {rss_growth_mb:.2f} MB")


if __name__ == "__main__":
    unittest.main()
