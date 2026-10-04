//! `paired-eval` — port of `scripts/paired_eval.py`. Runs each task twice
//! (control vs treatment workspace with the candidate skill installed) and
//! scores each arm with the task's own `verify.sh`.

use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::{jsonfmt, proc, py, sys1};

const DEFAULT_RUNNER: &str =
    "devin -p --permission-mode accept-edits --respect-workspace-trust false -- {prompt}";

struct Task {
    id: String,
    prompt: String,
    verify: PathBuf,
    seed: PathBuf,
}

fn load_task(task_dir: &Path) -> Option<Task> {
    let prompt_file = task_dir.join("prompt.md");
    let verify = task_dir.join("verify.sh");
    if !prompt_file.is_file() || !verify.is_file() {
        return None;
    }
    Some(Task {
        id: task_dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        prompt: std::fs::read_to_string(&prompt_file)
            .unwrap_or_default()
            .trim()
            .to_string(),
        verify,
        seed: task_dir.join("workspace"),
    })
}

fn task_dirs(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        let mut entries: Vec<PathBuf> = std::fs::read_dir(root)
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_dir() && p.join("prompt.md").is_file())
                    .collect()
            })
            .unwrap_or_default();
        entries.sort();
        dirs.extend(entries);
    }
    dirs
}

/// `shutil.copytree(src, dst, dirs_exist_ok=True)` — merge contents.
fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest = dst.join(entry.file_name());
        let ft = entry.file_type()?;
        if ft.is_dir() {
            copy_tree(&entry.path(), &dest)?;
        } else {
            // fs::copy follows symlinks, matching copytree(symlinks=False).
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

fn prepare_workspace(task: &Task, dest: &Path, skill: Option<&Path>) -> std::io::Result<()> {
    if task.seed.is_dir() {
        copy_tree(&task.seed, dest)?;
    }
    if let Some(skill) = skill {
        let name = skill
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let target = dest.join(".devin").join("skills").join(name);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        copy_tree(skill, &target)?;
    }
    Ok(())
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> std::io::Result<Self> {
        let base = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        for i in 0..100u32 {
            let cand = base.join(format!(
                "{prefix}{}-{}",
                std::process::id(),
                nanos + i as u128
            ));
            match std::fs::create_dir(&cand) {
                Ok(()) => return Ok(TempDir { path: cand }),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "cannot allocate temp dir",
        ))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn run_arm(
    task: &Task,
    runner: &str,
    workspace: &Path,
    timeout: u64,
    skill: Option<&Path>,
    completion_gate: &str,
) -> Result<Value, String> {
    prepare_workspace(task, workspace, skill).map_err(|e| format!("workspace prep: {e}"))?;
    let mut env: HashMap<String, String> = std::env::vars().collect();
    env.insert(
        "CIEL_EVAL_ARM".into(),
        if skill.is_some() {
            "treatment".into()
        } else {
            "control".into()
        },
    );
    let skill_dir = workspace.join(".devin").join("skills").join(
        skill
            .map(|s| {
                s.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap_or_default(),
    );
    let cmd = runner.replace("{prompt}", &py::shell_quote(&task.prompt));
    let cmd = cmd.replace(
        "{skill_dir}",
        &py::shell_quote(&skill_dir.to_string_lossy()),
    );
    let started = Instant::now();
    let agent_log = match proc::run_shell(&cmd, workspace, &env, Some(Duration::from_secs(timeout)))
    {
        Ok(p) if p.timed_out => format!("runner timed out after {timeout}s"),
        Ok(p) => py::tail_chars(&(p.stdout + &p.stderr), 4000),
        Err(e) => format!("runner spawn failed: {e}"),
    };
    let agent_secs = py::round_py(started.elapsed().as_secs_f64(), 1);

    let mut verify_cmd = std::process::Command::new("bash");
    verify_cmd
        .arg(&task.verify)
        .current_dir(workspace)
        .envs(&env)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let verify = proc::run_cmd(&mut verify_cmd, Some(Duration::from_secs(120)))
        .map_err(|e| format!("verify spawn: {e}"))?;
    if verify.timed_out {
        // subprocess.run raises TimeoutExpired uncaught → harness dies.
        return Err(format!("verify.sh timed out after 120s for {}", task.id));
    }
    let verify_rc = verify.returncode();
    let verify_pass = verify_rc == 0;
    let verify_log = py::tail_chars(&(verify.stdout + &verify.stderr), 1000);

    let mut completion_info: Option<Value> = None;
    let mut false_pass = false;
    let mut arm_pass = verify_pass;

    if completion_gate == "shadow" || completion_gate == "enforce" {
        let evidence = format!(
            "exit_code: {}\nverify_output:\n{}\nagent_output_tail:\n{}\n",
            verify_rc, verify_log, agent_log
        );
        let timeout_s: f64 = std::env::var("CIEL_SYSTEM1_TIMEOUT")
            .unwrap_or_else(|_| "2.0".into())
            .parse()
            .map_err(|_| "invalid CIEL_SYSTEM1_TIMEOUT".to_string())?;
        // Best-effort: any failure → None, mirroring `except Exception`.
        let info = std::panic::catch_unwind(|| {
            sys1::completion_check(&task.prompt, &evidence, "code_change", true, timeout_s)
        })
        .unwrap_or(None);
        completion_info = info;
        if let Some(ci) = &completion_info {
            if verify_pass && ci.get("band").and_then(|b| b.as_str()) == Some("flag") {
                false_pass = true;
                if completion_gate == "enforce" {
                    arm_pass = false;
                }
            }
        }
    }

    let mut arm_result = json!({
        "pass": arm_pass,
        "verify_pass": verify_pass,
        "agent_secs": agent_secs,
        "verify_log": verify_log,
        "agent_log_tail": agent_log,
    });
    if let Some(ci) = completion_info {
        arm_result["completion_check"] = ci;
    }
    if false_pass {
        arm_result["false_pass_detected"] = json!(true);
    }
    Ok(arm_result)
}

fn outcome(control: bool, treatment: bool) -> &'static str {
    if control && treatment {
        "preserved-pass"
    } else if control {
        "regression"
    } else if treatment {
        "improvement"
    } else {
        "preserved-fail"
    }
}

struct Args {
    skill: Option<PathBuf>,
    tasks: Vec<PathBuf>,
    runner: String,
    timeout: u64,
    completion_gate: String,
    report: Option<PathBuf>,
    require_improvement: bool,
    keep_workspaces: bool,
}

fn parse_args(args: &[String]) -> Result<Args, i32> {
    let mut a = Args {
        skill: None,
        tasks: Vec::new(),
        runner: DEFAULT_RUNNER.to_string(),
        timeout: 300,
        completion_gate: std::env::var("CIEL_COMPLETION_GATE").unwrap_or_else(|_| "off".into()),
        report: None,
        require_improvement: false,
        keep_workspaces: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--skill" => {
                i += 1;
                a.skill = args.get(i).map(PathBuf::from);
            }
            "--tasks" => {
                // argparse nargs="+" — consume until the next flag.
                i += 1;
                let mut any = false;
                while let Some(v) = args.get(i) {
                    if v.starts_with('-') {
                        break;
                    }
                    a.tasks.push(PathBuf::from(v));
                    any = true;
                    i += 1;
                }
                if !any {
                    return Err(2);
                }
                i = i.saturating_sub(1);
            }
            "--runner" => {
                i += 1;
                a.runner = args.get(i).cloned().ok_or(2)?;
            }
            "--timeout" => {
                i += 1;
                a.timeout = args.get(i).and_then(|s| s.parse().ok()).ok_or(2)?;
            }
            "--completion-gate" => {
                i += 1;
                let v = args.get(i).cloned().ok_or(2)?;
                if !["off", "shadow", "enforce"].contains(&v.as_str()) {
                    eprintln!("ciel-dev paired-eval: invalid --completion-gate '{v}'");
                    return Err(2);
                }
                a.completion_gate = v;
            }
            "--report" => {
                i += 1;
                a.report = args.get(i).map(PathBuf::from);
            }
            "--require-improvement" => a.require_improvement = true,
            "--keep-workspaces" => a.keep_workspaces = true,
            "-h" | "--help" => return Err(0),
            _ => return Err(2),
        }
        i += 1;
    }
    if a.completion_gate.as_str() != "off"
        && !["shadow", "enforce"].contains(&a.completion_gate.as_str())
    {
        eprintln!(
            "ciel-dev paired-eval: invalid --completion-gate '{}'",
            a.completion_gate
        );
        return Err(2);
    }
    if a.tasks.is_empty() {
        a.tasks = vec![py::repo_root().join("evals").join("tasks")];
    }
    if a.skill.is_none() {
        eprintln!("ciel-dev paired-eval: the following arguments are required: --skill");
        return Err(2);
    }
    Ok(a)
}

pub fn main_(args: &[String]) -> i32 {
    let a = match parse_args(args) {
        Ok(a) => a,
        Err(code) => {
            if code != 0 {
                eprintln!("usage: ciel-dev paired-eval --skill PATH [--tasks DIR ...] [--runner CMD] [--timeout SECS] [--completion-gate off|shadow|enforce] [--report PATH] [--require-improvement] [--keep-workspaces]");
            } else {
                println!("usage: ciel-dev paired-eval --skill PATH [--tasks DIR ...] [--runner CMD] [--timeout SECS] [--completion-gate off|shadow|enforce] [--report PATH] [--require-improvement] [--keep-workspaces]");
            }
            return code;
        }
    };

    let skill = a.skill.unwrap();
    let skill = skill.canonicalize().unwrap_or_else(|_| py::abspath(&skill));
    if !skill.join("SKILL.md").is_file() {
        eprintln!("[eval] {} has no SKILL.md", skill.display());
        return 2;
    }

    let tasks: Vec<Task> = task_dirs(&a.tasks)
        .iter()
        .filter_map(|d| load_task(d))
        .collect();
    if tasks.is_empty() {
        eprintln!("[eval] no tasks found (need <tasks>/<id>/prompt.md + verify.sh)");
        return 2;
    }

    let mut results: Vec<Value> = Vec::new();
    for task in &tasks {
        let tmp = match TempDir::new("ciel-eval-") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[eval] cannot create workspace: {e}");
                return 1;
            }
        };
        let control_ws = tmp.path.join("control");
        let treatment_ws = tmp.path.join("treatment");
        let _ = std::fs::create_dir(&control_ws);
        let _ = std::fs::create_dir(&treatment_ws);
        let control = match run_arm(
            task,
            &a.runner,
            &control_ws,
            a.timeout,
            None,
            &a.completion_gate,
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[eval] {}: {e}", task.id);
                return 1;
            }
        };
        let treatment = match run_arm(
            task,
            &a.runner,
            &treatment_ws,
            a.timeout,
            Some(&skill),
            &a.completion_gate,
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[eval] {}: {e}", task.id);
                return 1;
            }
        };
        let outcome = outcome(
            control["pass"].as_bool().unwrap_or(false),
            treatment["pass"].as_bool().unwrap_or(false),
        );
        results.push(json!({
            "task": task.id,
            "outcome": outcome,
            "control": control,
            "treatment": treatment,
        }));
        let r = results.last().unwrap();
        let fp_flag = if r["control"].get("false_pass_detected").is_some()
            || r["treatment"].get("false_pass_detected").is_some()
        {
            if a.completion_gate == "enforce" {
                " [FALSE PASS INTERCEPTED BY SYSTEM-1]"
            } else {
                " [SYSTEM-1 FLAGGED INCOMPLETE]"
            }
        } else {
            ""
        };
        println!(
            "[eval] {}: {} (control {} {}s, treatment {} {}s){}",
            task.id,
            outcome,
            if r["control"]["pass"].as_bool().unwrap_or(false) {
                "pass"
            } else {
                "fail"
            },
            py::py_float(r["control"]["agent_secs"].as_f64().unwrap_or(0.0)),
            if r["treatment"]["pass"].as_bool().unwrap_or(false) {
                "pass"
            } else {
                "fail"
            },
            py::py_float(r["treatment"]["agent_secs"].as_f64().unwrap_or(0.0)),
            fp_flag,
        );
        if a.keep_workspaces {
            let keep = std::env::temp_dir().join(format!("ciel-eval-{}", task.id));
            let _ = std::fs::remove_dir_all(&keep);
            if copy_tree(&tmp.path, &keep).is_ok() {
                println!("[eval]   workspaces kept at {}", keep.display());
            }
        }
    }

    let regressions: Vec<String> = results
        .iter()
        .filter(|r| r["outcome"] == "regression")
        .map(|r| r["task"].as_str().unwrap_or("").to_string())
        .collect();
    let improvements: Vec<String> = results
        .iter()
        .filter(|r| r["outcome"] == "improvement")
        .map(|r| r["task"].as_str().unwrap_or("").to_string())
        .collect();
    let false_passes: Vec<String> = results
        .iter()
        .filter(|r| {
            r["control"].get("false_pass_detected").is_some()
                || r["treatment"].get("false_pass_detected").is_some()
        })
        .map(|r| r["task"].as_str().unwrap_or("").to_string())
        .collect();
    let mut verdict = "pass";
    if !regressions.is_empty() || (a.completion_gate == "enforce" && !false_passes.is_empty()) {
        verdict = "fail";
    }
    if a.require_improvement && improvements.is_empty() {
        verdict = "fail";
    }

    let report = json!({
        "ts": py::iso_now_utc(),
        "skill": skill.to_string_lossy(),
        "runner": a.runner,
        "completion_gate": a.completion_gate,
        "verdict": verdict,
        "regressions": regressions,
        "improvements": improvements,
        "false_passes": false_passes,
        "tasks": results,
    });
    if let Some(report_path) = &a.report {
        match std::fs::write(
            report_path,
            format!("{}\n", jsonfmt::dumps_indent_ascii(&report, 2)),
        ) {
            Ok(()) => println!("[eval] report -> {}", report_path.display()),
            Err(e) => {
                eprintln!("[eval] cannot write report: {e}");
                return 1;
            }
        }
    }
    println!(
        "[eval] verdict={} tasks={} improvements={} regressions={}",
        verdict,
        report["tasks"].as_array().map(|t| t.len()).unwrap_or(0),
        report["improvements"]
            .as_array()
            .map(|t| t.len())
            .unwrap_or(0),
        report["regressions"]
            .as_array()
            .map(|t| t.len())
            .unwrap_or(0),
    );
    if verdict == "pass" {
        0
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_truth_table() {
        assert_eq!(outcome(true, true), "preserved-pass");
        assert_eq!(outcome(true, false), "regression");
        assert_eq!(outcome(false, true), "improvement");
        assert_eq!(outcome(false, false), "preserved-fail");
    }

    fn tmp_task(with_prompt: bool, with_verify: bool) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ciel-dev-task-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        if with_prompt {
            std::fs::write(dir.join("prompt.md"), "  do the thing\n\n").unwrap();
        }
        if with_verify {
            std::fs::write(dir.join("verify.sh"), "#!/bin/sh\nexit 0\n").unwrap();
        }
        dir
    }

    #[test]
    fn load_task_needs_prompt_and_verify() {
        let d = tmp_task(true, true);
        let t = load_task(&d).unwrap();
        assert_eq!(t.id, d.file_name().unwrap().to_string_lossy());
        assert_eq!(t.prompt, "do the thing"); // .strip()
        assert_eq!(t.seed, d.join("workspace"));
        let _ = std::fs::remove_dir_all(&d);

        let d = tmp_task(true, false);
        assert!(load_task(&d).is_none());
        let _ = std::fs::remove_dir_all(&d);

        let d = tmp_task(false, true);
        assert!(load_task(&d).is_none());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn prepare_workspace_installs_skill_only() {
        let dir = std::env::temp_dir().join(format!(
            "ciel-dev-ws-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let task_dir = dir.join("task");
        let skill = dir.join("cand-skill");
        let dest = dir.join("ws");
        std::fs::create_dir_all(task_dir.join("workspace")).unwrap();
        std::fs::write(task_dir.join("prompt.md"), "p").unwrap();
        std::fs::write(task_dir.join("verify.sh"), "x").unwrap();
        std::fs::write(task_dir.join("workspace").join("seed.txt"), "s").unwrap();
        std::fs::create_dir_all(&skill).unwrap();
        std::fs::write(skill.join("SKILL.md"), "---\nname: cand-skill\n---\n").unwrap();
        let task = load_task(&task_dir).unwrap();
        std::fs::create_dir_all(&dest).unwrap();
        prepare_workspace(&task, &dest, Some(&skill)).unwrap();
        assert!(dest.join("seed.txt").is_file());
        assert!(dest.join(".devin/skills/cand-skill/SKILL.md").is_file());
        // control arm: no skill
        let dest2 = dir.join("ws2");
        std::fs::create_dir_all(&dest2).unwrap();
        prepare_workspace(&task, &dest2, None).unwrap();
        assert!(!dest2.join(".devin").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn prompt_shell_quoting_in_runner() {
        // {prompt} must be shlex-quoted when substituted.
        let quoted = py::shell_quote("do 'risky' things");
        assert_eq!(quoted, "'do '\"'\"'risky'\"'\"' things'");
    }
}
