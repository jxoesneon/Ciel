//! `ciel council-verify <run_id|run_dir>` — Rust port of
//! `scripts/council_verify.py`. Validates a council run's artifact
//! completeness under `ciel_home()/council/<run_id>/` and emits the run's
//! structured member verdicts to `improvements/signals/council-<run_id>.json`.
//!
//! Contract (adapters/devin/COUNCIL_INVOCATION.md):
//!   spawn_receipts.json          {"mode": "subagent"|"inline", "members": {...}}
//!   members/<member>.stage{1,2}.json   {"member","stage":N,"score":num,...}
//!   verdict.json                 chairman docket with "votes" + "verdict"
//!
//! Exit: 0 verified (warnings allowed), 1 failed, 2 usage.

use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

use crate::paths;

const MEMBERS: [&str; 5] = [
    "coherence",
    "capability",
    "safety",
    "efficiency",
    "evolution",
];

fn load_json(path: &Path) -> Option<Value> {
    serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()
}

pub fn verify(run_dir: &Path) -> Value {
    let mut problems: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    let receipts = load_json(&run_dir.join("spawn_receipts.json"));
    let mode = match &receipts {
        None => {
            problems.push("missing spawn_receipts.json — member isolation unproven".into());
            "unknown".to_string()
        }
        Some(r) => {
            let m = r
                .get("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            if m == "inline" {
                warnings.push(
                    "unverified_member_isolation: run was inline, not subagent-dispatched".into(),
                );
            } else if m != "subagent" {
                warnings.push(format!("unrecognized spawn mode '{m}'"));
            }
            m
        }
    };

    let mut member_verdicts = Map::new();
    for member in MEMBERS {
        for stage in [1, 2] {
            let path = run_dir
                .join("members")
                .join(format!("{member}.stage{stage}.json"));
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let Some(data) = load_json(&path) else {
                problems.push(format!("missing/invalid {name}"));
                continue;
            };
            if data.get("member").and_then(|v| v.as_str()) != Some(member) {
                problems.push(format!(
                    "{name}: member field is '{}'",
                    data.get("member")
                        .map(|v| v.as_str().unwrap_or("null").to_string())
                        .unwrap_or_else(|| "null".into())
                ));
            }
            if data.get("stage").and_then(|v| v.as_i64()) != Some(stage as i64) {
                problems.push(format!(
                    "{name}: stage field is {}",
                    data.get("stage")
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "null".into())
                ));
            }
            match data.get("score") {
                // Python isinstance(x, (int,float)) — bool counts as int
                Some(s) if s.is_number() || s.is_boolean() => {
                    member_verdicts
                        .entry(member.to_string())
                        .or_insert_with(|| json!({}))
                        .as_object_mut()
                        .unwrap()
                        .insert(format!("stage{stage}"), s.clone());
                }
                other => problems.push(format!(
                    "{name}: non-numeric score {}",
                    other
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "null".into())
                )),
            }
        }
    }

    match load_json(&run_dir.join("verdict.json")) {
        None => problems.push("missing/invalid verdict.json".into()),
        Some(verdict) => {
            let votes = verdict.get("votes").cloned().unwrap_or_else(|| json!({}));
            let missing: Vec<&str> = MEMBERS
                .iter()
                .filter(|m| votes.get(**m).is_none())
                .copied()
                .collect();
            if !missing.is_empty() {
                problems.push(format!(
                    "verdict.json missing votes for: {}",
                    missing.join(", ")
                ));
            }
            let safety = votes.get("safety");
            let safety_num = safety.and_then(|s| {
                s.as_f64()
                    .or_else(|| s.as_bool().map(|b| if b { 1.0 } else { 0.0 }))
            });
            if verdict.get("verdict").and_then(|v| v.as_str()) == Some("pass")
                && safety_num.is_some_and(|n| n <= 3.0)
            {
                problems.push("verdict=pass but safety <= 3 (veto condition)".into());
            }
        }
    }

    json!({
        "run_dir": run_dir.to_string_lossy(),
        "mode": mode,
        "verified": problems.is_empty(),
        "problems": problems,
        "warnings": warnings,
        "member_verdicts": Value::Object(member_verdicts),
    })
}

fn emit_signal(ciel: &Path, run_id: &str, result: &Value) -> Option<PathBuf> {
    let signals = ciel.join("improvements").join("signals");
    std::fs::create_dir_all(&signals).ok()?;
    let slug = if run_id.starts_with("council-") {
        run_id.to_string()
    } else {
        format!("council-{run_id}")
    };
    let out = signals.join(format!("{slug}.json"));
    let body = json!({
        "signal": "council_verdict",
        "run_id": run_id,
        "mode": result["mode"],
        "verified": result["verified"],
        "member_verdicts": result["member_verdicts"],
        "warnings": result["warnings"],
    });
    let mut text = serde_json::to_string_pretty(&body).ok()?;
    text.push('\n');
    std::fs::write(&out, text).ok()?;
    Some(out)
}

/// `ciel council-verify <run_id|run_dir>`.
pub fn main_(args: &[String]) -> i32 {
    let ciel = paths::ciel_home();
    if args.len() != 1 {
        eprintln!("Council run verifier — member-as-subagent enforcement (M1).");
        return 2;
    }
    let arg = PathBuf::from(&args[0]);
    let run_dir = if arg.is_dir() {
        arg
    } else {
        ciel.join("council").join(&args[0])
    };
    if !run_dir.is_dir() {
        println!("[council_verify] no such run: {}", run_dir.display());
        return 1;
    }
    let result = verify(&run_dir);
    let run_id = run_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let signal_path = emit_signal(&ciel, &run_id, &result);

    let status = if result["verified"].as_bool().unwrap_or(false) {
        "VERIFIED"
    } else {
        "FAILED"
    };
    println!(
        "[council_verify] {run_id}: {status} (mode={})",
        result["mode"].as_str().unwrap_or("unknown")
    );
    if let Some(ws) = result["warnings"].as_array() {
        for w in ws.iter().filter_map(|w| w.as_str()) {
            println!("  warning: {w}");
        }
    }
    if let Some(ps) = result["problems"].as_array() {
        for p in ps.iter().filter_map(|p| p.as_str()) {
            println!("  problem: {p}");
        }
    }
    if let Some(sp) = signal_path {
        println!("  signal: {}", sp.display());
    }
    if result["verified"].as_bool().unwrap_or(false) {
        0
    } else {
        1
    }
}
