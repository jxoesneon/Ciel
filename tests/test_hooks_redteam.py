"""End-to-end red-team harness for the PreToolUse gates.

Fires every case in fixtures/hook_redteam_cases.json as a real subprocess
against both runtime hooks (devin + antigravity), in a sandboxed HOME so
allow_privileged state and activity.log stay hermetic.
"""

import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HOOKS_DIR = ROOT / "ciel.skill" / "init" / "hooks"
FIXTURE = Path(__file__).resolve().parent / "fixtures" / "hook_redteam_cases.json"


def _devin_payload(case: dict) -> dict:
    return {
        "tool_name": case["tool"],
        "tool_input": {
            "command": case.get("command", ""),
            "file_path": case.get("path", ""),
        },
        "session_id": "redteam",
        "prompt_id": case["id"],
    }


def _antigravity_payload(case: dict) -> dict:
    return {
        "toolCall": {
            "name": case["tool"],
            "args": {
                "CommandLine": case.get("command", ""),
                "path": case.get("path", ""),
            },
        },
        "conversationId": "redteam",
    }


RUNTIMES = {
    "devin": (HOOKS_DIR / "devin" / "pre_tool_use.sh", _devin_payload),
    "antigravity": (HOOKS_DIR / "antigravity" / "pre_tool_use.sh", _antigravity_payload),
}


def _run_hook(script: Path, payload: dict, home: Path) -> tuple[str, bool]:
    """Return (decision, overridden) for one firing of the hook."""
    env = dict(os.environ, HOME=str(home))
    env.pop("CIEL_HOME", None)
    env.pop("CIEL_POLICY", None)
    proc = subprocess.run(
        ["bash", str(script)],
        input=json.dumps(payload),
        capture_output=True,
        text=True,
        env=env,
        timeout=30,
        check=False,
    )
    assert proc.returncode == 0, f"hook exited {proc.returncode}: {proc.stderr}"

    decision = "allow"
    out = proc.stdout.strip()
    if out:
        verdict = json.loads(out.splitlines()[-1])
        if verdict.get("decision") in {"deny", "block"}:
            decision = "deny"

    overridden = False
    log = home / ".ciel" / "activity.log"
    if log.is_file():
        last = json.loads(log.read_text(encoding="utf-8").splitlines()[-1])
        overridden = bool(last.get("overridden"))
    return decision, overridden


@unittest.skipUnless(shutil.which("bash"), "bash required for hook red-team")
class TestHookRedTeam(unittest.TestCase):
    maxDiff = None

    def _fire(self, runtime: str, case: dict) -> str:
        script, payload_fn = RUNTIMES[runtime]
        with tempfile.TemporaryDirectory() as tmp:
            home = Path(tmp)
            (home / ".ciel").mkdir()
            if case.get("override"):
                (home / ".ciel" / "allow_privileged").touch()
            decision, overridden = _run_hook(script, payload_fn(case), home)
        if decision == "deny":
            return "deny"
        return "allow_overridden" if overridden else "allow"

    def test_corpus(self):
        cases = json.loads(FIXTURE.read_text(encoding="utf-8"))["cases"]
        mismatches = []
        for runtime in RUNTIMES:
            for case in cases:
                got = self._fire(runtime, case)
                if got != case["expect"]:
                    mismatches.append(
                        f"{runtime}/{case['id']}: expected {case['expect']}, got {got}"
                    )
        self.assertEqual([], mismatches)


if __name__ == "__main__":
    unittest.main()
