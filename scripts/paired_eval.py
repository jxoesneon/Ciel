#!/usr/bin/env python3
"""Paired-evaluation commit gate for skill mutations and promotions.

Runs a small deterministic task set twice per task: once in a clean workspace
(control arm) and once with the candidate skill installed into the
workspace's .devin/skills/ directory (treatment arm). Each arm is scored by
the task's own verify.sh — execution evidence, not self-evaluation.

Outcome per task:
  improvement     control fail -> treatment pass
  regression      control pass -> treatment fail   (gate FAILS on any of these)
  preserved-pass  both pass
  preserved-fail  both fail

Usage:
    paired_eval.py --skill PATH [--tasks DIR ...] [--runner CMD]
                   [--timeout SECS] [--report PATH] [--require-improvement]
                   [--keep-workspaces]

--runner is a shell template executed inside the task workspace; the literal
token {prompt} is replaced by the shell-quoted task prompt, and {skill_dir}
by the installed skill dir (treatment only). Default runner:
    devin -p --permission-mode accept-edits --respect-workspace-trust false -- {prompt}

Task layout (evals/tasks/<id>/):
    prompt.md     the task prompt given to the runner
    verify.sh     exit 0 = pass; run in the workspace after the runner
    workspace/    optional seed files copied into the eval workspace
"""

import argparse
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_RUNNER = "devin -p --permission-mode accept-edits --respect-workspace-trust false -- {prompt}"
REPO_TASKS = Path(__file__).resolve().parent.parent / "evals" / "tasks"
LIB_DIR = Path(__file__).resolve().parent.parent / "ciel.skill" / "init" / "hooks" / "lib"


def _load_task(task_dir: Path) -> dict | None:
    prompt_file = task_dir / "prompt.md"
    verify = task_dir / "verify.sh"
    if not prompt_file.is_file() or not verify.is_file():
        return None
    return {
        "id": task_dir.name,
        "dir": task_dir,
        "prompt": prompt_file.read_text(encoding="utf-8").strip(),
        "verify": verify,
        "seed": task_dir / "workspace",
    }


def _task_dirs(roots: list[Path]) -> list[Path]:
    dirs = []
    for root in roots:
        if not root.is_dir():
            continue
        dirs.extend(
            sorted(p for p in root.iterdir() if p.is_dir() and (p / "prompt.md").is_file())
        )
    return dirs


def _prepare_workspace(task: dict, dest: Path, skill: Path | None) -> None:
    if task["seed"].is_dir():
        shutil.copytree(task["seed"], dest, dirs_exist_ok=True)
    if skill is not None:
        target = dest / ".devin" / "skills" / skill.name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copytree(skill, target)


def _run_arm(task: dict, runner: str, workspace: Path, timeout: int,
             skill: Path | None, completion_gate: str = "off") -> dict:
    _prepare_workspace(task, workspace, skill)
    env = dict(os.environ)
    env["CIEL_EVAL_ARM"] = "treatment" if skill else "control"
    skill_dir = workspace / ".devin" / "skills" / (skill.name if skill else "")
    cmd = runner.replace("{prompt}", shlex.quote(task["prompt"]))
    cmd = cmd.replace("{skill_dir}", shlex.quote(str(skill_dir)))
    started = time.monotonic()
    try:
        proc = subprocess.run(
            cmd, shell=True, cwd=workspace, env=env,
            capture_output=True, text=True, timeout=timeout, check=False,
        )
        agent_log = (proc.stdout + proc.stderr)[-4000:]
    except subprocess.TimeoutExpired:
        agent_log = f"runner timed out after {timeout}s"
    agent_secs = round(time.monotonic() - started, 1)

    verify = subprocess.run(
        ["bash", str(task["verify"])], cwd=workspace, env=env,
        capture_output=True, text=True, timeout=120, check=False,
    )
    verify_pass = (verify.returncode == 0)
    verify_log = (verify.stdout + verify.stderr)[-1000:]

    completion_info = None
    false_pass = False
    arm_pass = verify_pass

    if completion_gate in ("shadow", "enforce"):
        evidence = (
            f"exit_code: {verify.returncode}\n"
            f"verify_output:\n{verify_log}\n"
            f"agent_output_tail:\n{agent_log}\n"
        )
        try:
            if str(LIB_DIR) not in sys.path:
                sys.path.insert(0, str(LIB_DIR))
            import system1
            completion_info = system1.completion_check(
                objective=task["prompt"],
                evidence=evidence,
                task_class="code_change",
                with_score=True,
                timeout=float(os.environ.get("CIEL_SYSTEM1_TIMEOUT", "2.0")),
            )
        except Exception:  # noqa: BLE001 - completion check is best-effort
            completion_info = None

        # False pass: verify passed deterministically, but System-1 flags incomplete
        if completion_info and verify_pass and completion_info.get("band") == "flag":
            false_pass = True
            if completion_gate == "enforce":
                arm_pass = False
        # Fail-open: if completion_info is None, arm_pass remains verify_pass unchanged

    arm_result = {
        "pass": arm_pass,
        "verify_pass": verify_pass,
        "agent_secs": agent_secs,
        "verify_log": verify_log,
        "agent_log_tail": agent_log,
    }
    if completion_info is not None:
        arm_result["completion_check"] = completion_info
    if false_pass:
        arm_result["false_pass_detected"] = True

    return arm_result


def _outcome(control: bool, treatment: bool) -> str:
    if control and treatment:
        return "preserved-pass"
    if control:
        return "regression"
    if treatment:
        return "improvement"
    return "preserved-fail"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skill", type=Path, required=True,
                        help="candidate skill directory (contains SKILL.md)")
    parser.add_argument("--tasks", type=Path, nargs="+", default=[REPO_TASKS],
                        help="task-set directories (default: evals/tasks/)")
    parser.add_argument("--runner", default=DEFAULT_RUNNER,
                        help="shell template; {prompt} and {skill_dir} are substituted")
    parser.add_argument("--timeout", type=int, default=300,
                        help="per-arm runner timeout in seconds")
    parser.add_argument("--completion-gate", choices=["off", "shadow", "enforce"],
                        default=os.environ.get("CIEL_COMPLETION_GATE", "off"),
                        help="System-1 completion verification gate (off|shadow|enforce)")
    parser.add_argument("--report", type=Path, help="write JSON evidence report")
    parser.add_argument("--require-improvement", action="store_true",
                        help="fail unless at least one task shows improvement")
    parser.add_argument("--keep-workspaces", action="store_true",
                        help="keep temp workspaces for debugging")
    args = parser.parse_args()

    skill = args.skill.resolve()
    if not (skill / "SKILL.md").is_file():
        print(f"[eval] {skill} has no SKILL.md", file=sys.stderr)
        return 2

    tasks = [t for t in (_load_task(d) for d in _task_dirs(args.tasks)) if t]
    if not tasks:
        print("[eval] no tasks found (need <tasks>/<id>/prompt.md + verify.sh)",
              file=sys.stderr)
        return 2

    results = []
    for task in tasks:
        with tempfile.TemporaryDirectory(prefix="ciel-eval-") as tmp:
            control_ws = Path(tmp) / "control"
            treatment_ws = Path(tmp) / "treatment"
            control_ws.mkdir()
            treatment_ws.mkdir()
            control = _run_arm(task, args.runner, control_ws, args.timeout, None,
                               completion_gate=args.completion_gate)
            treatment = _run_arm(task, args.runner, treatment_ws, args.timeout, skill,
                                 completion_gate=args.completion_gate)
            outcome = _outcome(control["pass"], treatment["pass"])
            results.append({
                "task": task["id"],
                "outcome": outcome,
                "control": control,
                "treatment": treatment,
            })
            fp_flag = ""
            if control.get("false_pass_detected") or treatment.get("false_pass_detected"):
                fp_flag = " [FALSE PASS INTERCEPTED BY SYSTEM-1]" if args.completion_gate == "enforce" else " [SYSTEM-1 FLAGGED INCOMPLETE]"
            print(f"[eval] {task['id']}: {outcome} "
                  f"(control {'pass' if control['pass'] else 'fail'} "
                  f"{control['agent_secs']}s, "
                  f"treatment {'pass' if treatment['pass'] else 'fail'} "
                  f"{treatment['agent_secs']}s){fp_flag}")
            if args.keep_workspaces:
                keep = Path(tempfile.gettempdir()) / f"ciel-eval-{task['id']}"
                shutil.rmtree(keep, ignore_errors=True)
                shutil.copytree(tmp, keep)
                print(f"[eval]   workspaces kept at {keep}")

    regressions = [r["task"] for r in results if r["outcome"] == "regression"]
    improvements = [r["task"] for r in results if r["outcome"] == "improvement"]
    false_passes = [r["task"] for r in results if r["control"].get("false_pass_detected") or r["treatment"].get("false_pass_detected")]
    verdict = "fail" if (regressions or (args.completion_gate == "enforce" and false_passes)) else "pass"
    if args.require_improvement and not improvements:
        verdict = "fail"

    report = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "skill": str(skill),
        "runner": args.runner,
        "completion_gate": args.completion_gate,
        "verdict": verdict,
        "regressions": regressions,
        "improvements": improvements,
        "false_passes": false_passes,
        "tasks": results,
    }
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(f"[eval] report -> {args.report}")

    print(f"[eval] verdict={verdict} tasks={len(results)} "
          f"improvements={len(improvements)} regressions={len(regressions)}")
    return 0 if verdict == "pass" else 2


if __name__ == "__main__":
    sys.exit(main())
