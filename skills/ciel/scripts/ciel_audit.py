#!/usr/bin/env python3
"""ciel_audit.py — agent-invoked post-tool / post-failure audit writer.

Transitional fallback shim (see ADR_20261003_PLATFORM_AGNOSTIC_AGENT_SCRIPTS):
single implementation for the agent lifecycle audit surface, replacing the
hollow post_tool_hook.sh / post_failure_hook.sh and the divergent
ciel_post_tool.ps1.

Contract:
  stdin : optional JSON {event, tool, error, conversationId, session_id,
          prompt_id, exit_code} — fields absent from argv are taken from it.
  argv  : --event NAME (default PostToolUse)  --tool NAME
          [--error TEXT]  [--exit-code N]  [--conversation-id ID]
          [--runtime R]  (default "agent" or $CIEL_RUNTIME)
  stdout: "{}" (the PostToolUse hook contract)
  exit  : always 0 — audit must never block a tool call.

Emits the hooks' byte-compatible activity.log schema ({ts, runtime, event,
tool, ...}) through the shared ciel_root.append_log writer + rotation.
"""

import json
import os
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import _bootstrap  # noqa: E402

_bootstrap.load()
try:
    import ciel_root
except ImportError:
    ciel_root = None  # noqa: E402

_FIELDS = ("event", "tool", "error", "conversationId", "session_id",
           "prompt_id", "exit_code", "status")


def _arg(args: list[str], name: str) -> str | None:
    if name in args:
        i = args.index(name)
        if i + 1 < len(args):
            return args[i + 1]
    return None


def _native_audit(args: list[str]) -> bool:
    """Delegate to `ciel audit` when the binary has the subcommand. The
    native contract prints "{}" and exits 0; an older binary exits 2 with
    usage on stderr, so treat non-zero or non-"{}" output as unsupported."""
    binary = ciel_root.ciel_bin() if ciel_root else None
    if not binary:
        return False
    try:
        proc = subprocess.run(
            [binary, "audit", *args],
            stdin=subprocess.DEVNULL if sys.stdin.isatty() else sys.stdin,
            capture_output=True, text=True, timeout=15)
    except (OSError, subprocess.TimeoutExpired):
        return False
    return proc.returncode == 0 and proc.stdout.strip() == "{}"


def main() -> int:
    args = sys.argv[1:]

    if _native_audit(args):
        print("{}")
        return 0

    try:
        payload = {} if sys.stdin.isatty() else json.loads(sys.stdin.read() or "{}")
        if not isinstance(payload, dict):
            payload = {}
    except json.JSONDecodeError:
        payload = {}

    event = _arg(args, "--event") or payload.get("event") or "PostToolUse"
    tool = _arg(args, "--tool") or payload.get("tool") or payload.get("toolName") \
        or "unknown"
    runtime = _arg(args, "--runtime") or os.environ.get("CIEL_RUNTIME", "agent")

    from datetime import datetime, timezone
    ts = (ciel_root.utc_now() if ciel_root is not None
          else datetime.now(timezone.utc).isoformat())
    entry = {
        "ts": ts,
        "runtime": runtime,
        "event": event,
        "tool": tool,
    }
    error = _arg(args, "--error") or payload.get("error")
    if error:
        entry["error"] = error
        entry["status"] = "failed"
    for field in _FIELDS:
        if field in ("event", "tool", "error", "status"):
            continue
        cli = _arg(args, "--" + field.replace("_", "-"))
        val = cli if cli is not None else payload.get(field)
        if val is not None:
            entry[field] = val

    if ciel_root is not None:
        ciel_root.append_log(entry)
    print("{}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
