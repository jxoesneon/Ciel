#!/usr/bin/env python3
"""Agent-facing script contract tests (skills/ciel/scripts).

Covers the Council-amended contracts from DOCKET_20261003:
  - ciel_preflight.py: exit 0/2/3, fail-closed on unavailable or malformed
    input, python-path verdict == production hook verdict
    (evaluate + system1_failsafe), binary path via `ciel pretool`.
  - ciel_audit.py: byte-compatible activity.log schema incl. runtime field.
  - verify_evidence.py: 0 pass / 2 rejected / 3 gate-unreachable (enforce),
    loud + logged under shadow.
  - read_agy_conversation.js: .db prefix resolution ignores -shm/-wal.
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
SCRIPTS = ROOT / "skills" / "ciel" / "scripts"
CIEL_BIN = os.environ.get(
    "CIEL_RS_BIN",
    str(ROOT / "ciel.skill" / "init" / "ciel-rs" / "target" / "debug" / "ciel"),
)
NODE = shutil.which("node")

sys.path.insert(0, str(LIB))
import risk_policy  # noqa: E402

PREFLIGHT = SCRIPTS / "ciel_preflight.py"
AUDIT = SCRIPTS / "ciel_audit.py"
VERIFY = SCRIPTS / "verify_evidence.py"
BOOTSTRAP = SCRIPTS / "_bootstrap.py"


def run(script: Path, stdin: str = "", env: dict | None = None,
        args: list[str] | None = None, cwd: Path | None = None):
    return subprocess.run(
        [sys.executable, str(script), *(args or [])],
        input=stdin, capture_output=True, text=True, env=env, cwd=cwd,
        timeout=60,
    )


class _TmpHome(unittest.TestCase):
    """Isolated HOME/CIEL_HOME so logs and sentinels land in tmp."""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="ciel-agents-"))
        self.ciel = self.tmp / ".ciel"
        self.ciel.mkdir(parents=True)
        # Isolate the live host's ~/.ciel from in-process calls too — the
        # allow_privileged sentinel and grant state must not leak in.
        self._saved_env = {k: os.environ.get(k) for k in
                           ("HOME", "CIEL_HOME", "CIEL_SYSTEM1_MODE")}
        os.environ["HOME"] = str(self.tmp)
        os.environ["CIEL_HOME"] = str(self.ciel)
        os.environ["CIEL_SYSTEM1_MODE"] = "shadow"
        self.env = dict(
            os.environ,
            HOME=str(self.tmp),
            CIEL_HOME=str(self.ciel),
            CIEL_SYSTEM1_MODE="shadow",  # never block on a live daemon in tests
        )

    def tearDown(self):
        for k, v in self._saved_env.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v
        shutil.rmtree(self.tmp, ignore_errors=True)


class TestPreflightContract(_TmpHome):
    def _run(self, payload, env_extra=None):
        env = dict(self.env)
        if env_extra:
            env.update(env_extra)
        return run(PREFLIGHT, stdin=json.dumps(payload), env=env)

    def test_allow_exit_0(self):
        p = self._run({"tool": "exec", "command": "ls -la /tmp", "path": ""})
        self.assertEqual(0, p.returncode, p.stderr)
        v = json.loads(p.stdout)
        self.assertEqual("allow", v["decision"])

    def test_deny_exit_2(self):
        p = self._run({"tool": "exec", "command": "mkfs /dev/sda", "path": ""})
        self.assertEqual(2, p.returncode, p.stderr)
        v = json.loads(p.stdout)
        self.assertEqual("deny", v["decision"])
        self.assertEqual("mkfs", v["rule_id"])

    def test_malformed_stdin_fails_closed(self):
        env = dict(self.env)
        p = run(PREFLIGHT, stdin="{not json", env=env)
        self.assertEqual(3, p.returncode)
        v = json.loads(p.stdout)
        self.assertEqual("deny", v["decision"])
        self.assertEqual("preflight_malformed_input", v["rule_id"])

    def test_python_path_matches_hook_verdict(self):
        """Preflight verdict must equal evaluate + system1_failsafe — the
        exact pair the production pre_tool_use hooks apply."""
        for cmd in ("ls -la", "mkfs /dev/sda", "git push --force origin main"):
            expected = risk_policy.evaluate("exec", cmd, "")
            expected = risk_policy.system1_failsafe(expected, "exec", cmd, "")
            # PATH="" disables the shutil.which rescue so the python leg
            # is genuinely exercised.
            p = self._run({"tool": "exec", "command": cmd, "path": ""},
                          env_extra={"CIEL_BIN": "", "PATH": ""})
            v = json.loads(p.stdout)
            self.assertEqual(expected["decision"], v["decision"], cmd)
            self.assertEqual(expected.get("rule_id"), v.get("rule_id"), cmd)
            self.assertEqual("risk_policy", v["engine"], cmd)

    def test_logs_preflight_entry(self):
        env = dict(self.env, CIEL_BIN="", PATH="")
        run(PREFLIGHT, stdin=json.dumps(
            {"tool": "exec", "command": "ls", "path": ""}), env=env)
        line = (self.ciel / "activity.log").read_text().strip().splitlines()[-1]
        entry = json.loads(line)
        self.assertEqual("Preflight", entry["event"])
        self.assertEqual("agent", entry["runtime"])
        self.assertEqual("exec", entry["tool"])

    def test_unavailable_fails_closed(self):
        """No binary, no reachable risk_policy → deny exit 3, never allow."""
        iso = self.tmp / "iso"
        (iso / "scripts").mkdir(parents=True)
        shutil.copy(PREFLIGHT, iso / "scripts")
        shutil.copy(BOOTSTRAP, iso / "scripts")
        env = dict(os.environ)
        env.update({
            "PATH": "", "HOME": str(self.tmp),
            "CIEL_HOME": str(self.ciel),
            "CIEL_ROOT": str(self.tmp / "no-such-root"),
            "CIEL_HOOK_LIB": str(self.tmp / "no-such-lib"),
            "CIEL_BIN": str(self.tmp / "no-such-bin"),
        })
        p = run(iso / "scripts" / "ciel_preflight.py",
                stdin=json.dumps({"tool": "exec", "command": "ls"}), env=env)
        self.assertEqual(3, p.returncode, p.stderr)
        v = json.loads(p.stdout)
        self.assertEqual("deny", v["decision"])
        self.assertEqual("preflight_unavailable", v["rule_id"])

    def test_system1_offline_failsafe_python(self):
        """Destructive command + active mode + offline daemon →
        system1_offline_failsafe deny (exit 2), same as the hooks."""
        env = dict(self.env)
        env["CIEL_SYSTEM1_MODE"] = "active"
        env["CIEL_BIN"] = ""
        env["PATH"] = ""
        env["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:1"  # dead endpoint
        p = run(PREFLIGHT, stdin=json.dumps(
            {"tool": "exec", "command": "rm -rf /tmp/x", "path": ""}), env=env)
        v = json.loads(p.stdout)
        self.assertEqual("deny", v["decision"])
        self.assertEqual("system1_offline_failsafe", v["rule_id"])
        self.assertEqual(2, p.returncode)


@unittest.skipUnless(Path(CIEL_BIN).exists(), f"no ciel binary at {CIEL_BIN}")
class TestPreflightBinaryPath(_TmpHome):
    def setUp(self):
        super().setUp()
        self.env["CIEL_BIN"] = CIEL_BIN

    def test_binary_allow(self):
        env = dict(self.env, CIEL_PREFER_BINARY="1")
        p = run(PREFLIGHT, stdin=json.dumps(
            {"tool": "exec", "command": "ls -la", "path": ""}), env=env)
        self.assertEqual(0, p.returncode, p.stderr)
        v = json.loads(p.stdout)
        self.assertEqual("allow", v["decision"])
        # Newer binaries answer via the native `ciel preflight` subcommand
        # ("ciel rust"); older ones go through `ciel pretool` normalization.
        self.assertIn(v["engine"], ("ciel pretool", "ciel rust"))

    def test_binary_deny(self):
        env = dict(self.env, CIEL_PREFER_BINARY="1")
        p = run(PREFLIGHT, stdin=json.dumps(
            {"tool": "exec", "command": "mkfs /dev/sda", "path": ""}), env=env)
        self.assertEqual(2, p.returncode, p.stderr)
        v = json.loads(p.stdout)
        self.assertEqual("deny", v["decision"])

    def test_binary_python_parity(self):
        """Both engines must agree on the decision incl. the failsafe leg."""
        for cmd, mode in (("ls -la", "shadow"), ("mkfs /dev/sda", "shadow"),
                          ("rm -rf /tmp/x", "active")):
            env = dict(self.env)
            env["CIEL_SYSTEM1_MODE"] = mode
            env["CIEL_SYSTEM1_URL"] = "http://127.0.0.1:1"  # dead endpoint
            pb = run(PREFLIGHT, stdin=json.dumps(
                {"tool": "exec", "command": cmd, "path": ""}),
                env=dict(env, CIEL_PREFER_BINARY="1"))
            pp = run(PREFLIGHT, stdin=json.dumps(
                {"tool": "exec", "command": cmd, "path": ""}),
                env=dict(env, CIEL_BIN="", PATH=""))
            vb, vp = json.loads(pb.stdout), json.loads(pp.stdout)
            self.assertEqual(vb["decision"], vp["decision"], cmd)


class TestAuditContract(_TmpHome):
    def test_post_tool_entry(self):
        p = run(AUDIT, args=["--event", "PostToolUse", "--tool", "exec"],
                env=self.env)
        self.assertEqual(0, p.returncode, p.stderr)
        self.assertEqual("{}", p.stdout.strip())
        entry = json.loads(
            (self.ciel / "activity.log").read_text().strip().splitlines()[-1])
        for k in ("ts", "runtime", "event", "tool"):
            self.assertIn(k, entry)
        self.assertEqual("agent", entry["runtime"])
        self.assertEqual("PostToolUse", entry["event"])
        self.assertEqual("exec", entry["tool"])

    def test_failure_entry(self):
        p = run(AUDIT, args=["--event", "PostFailure", "--tool", "exec",
                             "--error", "boom", "--exit-code", "1"],
                env=self.env)
        self.assertEqual(0, p.returncode)
        entry = json.loads(
            (self.ciel / "activity.log").read_text().strip().splitlines()[-1])
        self.assertEqual("failed", entry["status"])
        self.assertEqual("boom", entry["error"])

    def test_stdin_payload(self):
        p = run(AUDIT, stdin=json.dumps(
            {"event": "PostToolUse", "tool": "write",
             "conversationId": "abc"}), env=self.env)
        self.assertEqual(0, p.returncode)
        entry = json.loads(
            (self.ciel / "activity.log").read_text().strip().splitlines()[-1])
        self.assertEqual("abc", entry["conversationId"])


class TestVerifyEvidenceContract(_TmpHome):
    def test_reachable_gate_passthrough(self):
        """Repo layout resolves the real bundled gate; offline System-1
        fails open to exit 0."""
        p = run(VERIFY, env=self.env)
        self.assertEqual(0, p.returncode, p.stderr)
        self.assertIn("VERIFICATION HARNESS", p.stdout)

    def _isolated(self):
        iso = self.tmp / "iso"
        (iso / "scripts").mkdir(parents=True)
        shutil.copy(VERIFY, iso / "scripts")
        shutil.copy(BOOTSTRAP, iso / "scripts")
        env = dict(os.environ)
        env.update({
            "PATH": "", "HOME": str(self.tmp),
            "CIEL_HOME": str(self.ciel),
            "CIEL_ROOT": str(self.tmp / "no-such-root"),
            "CIEL_HOOK_LIB": str(self.tmp / "no-such-lib"),
            "CIEL_BIN": str(self.tmp / "no-such-bin"),
        })
        return iso / "scripts" / "verify_evidence.py", env

    def test_unreachable_enforce_exit_3(self):
        script, env = self._isolated()
        env["CIEL_COMPLETION_GATE"] = "enforce"
        p = run(script, env=env)
        self.assertEqual(3, p.returncode)
        self.assertIn("unreachable", p.stderr)
        log = self.ciel / "activity.log"
        self.assertIn("CompletionGateUnreachable", log.read_text())

    def test_unreachable_shadow_warns_exit_0(self):
        script, env = self._isolated()
        env["CIEL_COMPLETION_GATE"] = "shadow"
        p = run(script, env=env)
        self.assertEqual(0, p.returncode)
        self.assertIn("unreachable", p.stderr)
        self.assertIn("CompletionGateUnreachable",
                      (self.ciel / "activity.log").read_text())

    def test_gate_rejection_exit_2(self):
        root = self.tmp / "fakeroot"
        gate = root / "scripts"
        gate.mkdir(parents=True)
        (gate / "verify_completion.py").write_text(
            "import sys; sys.exit(2)\n")
        env = dict(self.env, CIEL_ROOT=str(root), CIEL_BIN="/nonexistent",
                   CIEL_COMPLETION_GATE="enforce")
        p = run(VERIFY, env=env)
        self.assertEqual(2, p.returncode)
        self.assertIn("rejected", p.stderr)


class TestShimInvariant(unittest.TestCase):
    """Platform-agnostic invariant: any future OS shim in this directory may
    only be a thin dispatcher forwarding to a .py sibling — never a second
    policy implementation (DOCKET_20261003 amendment)."""

    def test_shims_are_thin_dispatchers(self):
        shims = [p for p in SCRIPTS.iterdir()
                 if p.suffix in (".sh", ".ps1", ".bat", ".cmd")]
        for shim in shims:
            body = [ln.strip() for ln in shim.read_text().splitlines()
                    if ln.strip()
                    and not ln.strip().startswith(("#", "rem", "REM"))]
            forwarders = [ln for ln in body
                          if "python" in ln.lower() and ".py" in ln]
            self.assertTrue(
                forwarders,
                f"{shim.name}: OS shim must forward to a .py sibling")
            self.assertLessEqual(
                len(body), 10,
                f"{shim.name}: shim has {len(body)} code lines — policy "
                "logic must live in the shared .py, not the shim")
            for token in ("allow_overridden", "system1_failsafe",
                          "risk-eval", "decision"):
                for ln in body:
                    self.assertNotIn(
                        token, ln,
                        f"{shim.name}: '{token}' in shim — divergent policy")


@unittest.skipUnless(NODE, "node not installed")
class TestReadAgy(_TmpHome):
    def _seed(self):
        conv = self.tmp / ".gemini" / "antigravity-cli" / "conversations"
        conv.mkdir(parents=True)
        cid = "abc12345-dead-beef-0000-000000000000"
        for suffix in (".db", ".db-wal", ".db-shm"):
            (conv / (cid + suffix)).write_text("")
        return cid

    def test_prefix_ignores_wal_shm(self):
        cid = self._seed()
        # USERPROFILE: os.homedir() prefers it over HOME on Windows runners.
        env = dict(self.env, USERPROFILE=str(self.tmp))
        p = subprocess.run(
            [NODE, str(SCRIPTS / "read_agy_conversation.js"), cid[:8]],
            capture_output=True, text=True, env=env)
        self.assertEqual(0, p.returncode, p.stderr)
        self.assertIn(f"conversation={cid}", p.stdout)

    def test_raw_without_transcript(self):
        cid = self._seed()
        env = dict(self.env, USERPROFILE=str(self.tmp))
        p = subprocess.run(
            [NODE, str(SCRIPTS / "read_agy_conversation.js"), cid, "--raw"],
            capture_output=True, text=True, env=env)
        self.assertEqual(1, p.returncode)
        self.assertIn("No JSONL transcript", p.stderr)


if __name__ == "__main__":
    unittest.main()
