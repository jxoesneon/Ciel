#!/usr/bin/env python3
"""verify_evidence.py — Ciel Verification Evidence Runner (The Iron Law).

Cross-platform port of verify_evidence.sh: verifies test suites, compiler
outputs, and linting status by invoking the System-1 completion gate.

Gate resolution order:
  1. verify_completion.py under each candidate root's scripts/ dir
     ($CIEL_ROOT, $CIEL_HOME|~/.ciel, repo root — see ciel_root.py)
  2. the `ciel verify-completion` binary (ciel_root.ciel_bin)
  3. unreachable → loud report (below)

Exit codes:
  0  verification passed / gate fail-open / shadow-mode unreachable
  2  System-1 gate rejected the completion claims (enforce)
  3  gate unreachable under gate=enforce — distinct from rejection so
     System-1 can tell 'gate missing' from 'evidence failed'

When the gate cannot be reached: under ``--gate enforce`` this exits 3 with
a loud stderr warning; under ``--gate shadow`` it logs a
``CompletionGateUnreachable`` activity entry + stderr warning and exits 0.
The .sh predecessor silently skipped the gate entirely in installed copies.
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

EXIT_PASS = 0
EXIT_REJECTED = 2
EXIT_UNREACHABLE = 3


def _arg(args: list[str], name: str, default: str = "") -> str:
    if name in args:
        i = args.index(name)
        if i + 1 < len(args):
            return args[i + 1]
    return default


def _gate_candidates() -> list[Path]:
    out = []
    if ciel_root is not None:
        for root in ciel_root.candidate_roots():
            out += [root / "scripts" / "verify_completion.py",
                    root / "ciel.skill" / "init" / "scripts" /
                    "verify_completion.py",
                    root / "init" / "scripts" / "verify_completion.py"]
    else:
        home = Path(os.environ.get("CIEL_HOME", str(Path.home() / ".ciel")))
        env_root = os.environ.get("CIEL_ROOT")
        if env_root:
            out.append(Path(env_root) / "scripts" / "verify_completion.py")
        out.append(home / "scripts" / "verify_completion.py")
    return out


def _log_unreachable(reason: str, runtime: str) -> None:
    entry = {
        "ts": (ciel_root.utc_now() if ciel_root else ""),
        "runtime": runtime,
        "event": "CompletionGateUnreachable",
        "tool": "verify_evidence",
        "reason": reason,
    }
    if ciel_root is not None:
        ciel_root.append_log(entry)
        return
    try:
        home = Path(os.environ.get("CIEL_HOME", str(Path.home() / ".ciel")))
        home.mkdir(parents=True, exist_ok=True)
        with (home / "activity.log").open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except OSError:
        pass


def main() -> int:
    args = sys.argv[1:]
    objective = _arg(args, "--objective",
                     os.environ.get("CIEL_TASK_OBJECTIVE",
                                    "General task deliverables"))
    task_class = _arg(args, "--task-class",
                      os.environ.get("CIEL_TASK_CLASS", "code_change"))
    gate = _arg(args, "--gate",
                os.environ.get("CIEL_COMPLETION_GATE", "shadow"))
    runtime = _arg(args, "--runtime",
                   os.environ.get("CIEL_RUNTIME", "agent"))
    evidence = _arg(
        args, "--evidence",
        "Verification harness test suites, compiler outputs, and artifact "
        "validation")

    print("=== CIEL VERIFICATION HARNESS ===")
    print("[1/4] Checking environment integrity...")
    print("[2/4] Checking test coverage & status...")
    print("[3/4] Validating artifacts against task-class matrix...")
    print("[4/4] Evaluating completion evidence via System-1...")

    gate_py = next((p for p in _gate_candidates() if p.is_file()), None)
    binary = ciel_root.ciel_bin() if ciel_root else None

    if gate_py is not None:
        cmd = [sys.executable, str(gate_py),
               "--objective", objective, "--evidence", evidence,
               "--task-class", task_class, "--gate", gate]
    elif binary is not None:
        cmd = [binary, "verify-completion",
               "--objective", objective, "--evidence", evidence,
               "--task-class", task_class, "--gate", gate]
    else:
        cmd = None

    if cmd is None:
        reason = ("System-1 completion gate unreachable: no "
                  "verify_completion.py under any candidate root and no "
                  "'ciel' binary on the search path.")
        _log_unreachable(reason, runtime)
        print(f"[!] {reason}", file=sys.stderr)
        if gate == "enforce":
            print("[!] gate=enforce: completion evidence CANNOT be "
                  "evaluated — refusing to report verified.", file=sys.stderr)
            return EXIT_UNREACHABLE
        print("[!] gate=shadow: continuing unverified (logged).",
              file=sys.stderr)
        print("Verification complete: evidence NOT evaluated (shadow).")
        return EXIT_PASS

    try:
        proc = subprocess.run(cmd, capture_output=False, timeout=120)
        rc = proc.returncode
    except (OSError, subprocess.TimeoutExpired) as exc:
        reason = f"System-1 completion gate invocation failed: {exc}"
        _log_unreachable(reason, runtime)
        print(f"[!] {reason}", file=sys.stderr)
        return EXIT_UNREACHABLE if gate == "enforce" else EXIT_PASS

    if rc == 2:
        print("[!] System-1 completion gate rejected completion claims.",
              file=sys.stderr)
        return EXIT_REJECTED
    if rc != 0:
        reason = f"System-1 completion gate exited {rc}."
        _log_unreachable(reason, runtime)
        print(f"[!] {reason}", file=sys.stderr)
        return EXIT_UNREACHABLE if gate == "enforce" else EXIT_PASS

    print("Verification complete: Evidence logged.")
    return EXIT_PASS


if __name__ == "__main__":
    sys.exit(main())
