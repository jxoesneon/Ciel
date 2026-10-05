//! `rlcd` — port of `scripts/ciel_rlcd_pipeline.py`. Generates preference
//! pairs from System-1 flags, council signals, and dockets.

use serde_json::{json, Map, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::{jsonfmt, py, secrets::scrub_secrets};

fn obj_or_empty(v: Option<&Value>) -> Map<String, Value> {
    v.and_then(|v| v.as_object()).cloned().unwrap_or_default()
}

/// `process_events` — pairs from flagged/uncertain pre_tool_risk records,
/// context-surface flag corrections (context_select, memory_salience,
/// context_compaction, mandate_canary), and council_prescreen consensus
/// signals.
fn process_events(events_file: &Path) -> Vec<Value> {
    let mut pairs = Vec::new();
    let Ok(fh) = std::fs::File::open(events_file) else {
        return pairs;
    };
    for line in BufReader::new(fh).lines() {
        let Ok(line) = line else { continue };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(rec) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let surface = rec.get("surface").cloned().unwrap_or(Value::Null);
        let meta = obj_or_empty(rec.get("meta"));
        let answers = rec
            .get("system1")
            .filter(|s| s.is_object())
            .map(|s| obj_or_empty(s.get("answers")))
            .unwrap_or_default();
        let flag = rec.get("flag").cloned().unwrap_or(json!("pass"));
        let state = scrub_secrets(&rec.get("state").cloned().unwrap_or(json!({})));

        // 1. System-1 Flagged Command (Overriding explicit allow/pass)
        if (flag == json!("flag") || flag == json!("uncertain"))
            && surface == json!("pre_tool_risk")
        {
            for (qkey, ans) in &answers {
                if let Some(a) = ans.as_object() {
                    if a.get("choice") == Some(&json!("dangerous")) {
                        pairs.push(json!({
                            "surface": surface,
                            "question_key": qkey,
                            "state": state,
                            "chosen": "dangerous",
                            "rejected": ["safe"],
                            "source": "system1_flag",
                            "model_confidence": a.get("confidence").cloned().unwrap_or(Value::Null),
                        }));
                    }
                }
            }
        }

        // 2. Context surfaces — a flagged/uncertain shadow verdict means the
        // model's direction departs from the passive default: emit a pair
        // preferring the verdict direction over the default, the same
        // flag-overrides-pass convention as pre_tool_risk above.
        if flag == json!("flag") || flag == json!("uncertain") {
            match surface.as_str().unwrap_or("") {
                // context_select keeps everything by default; a drop verdict
                // at flag band is the correction.
                "context_select" => {
                    if let Some(ans) = answers.get("relevant").and_then(|a| a.as_object()) {
                        if ans.get("choice") == Some(&json!("drop")) {
                            pairs.push(json!({
                                "surface": surface,
                                "question_key": "relevant",
                                "state": state,
                                "chosen": "drop",
                                "rejected": ["keep"],
                                "source": "system1_flag",
                                "model_confidence": ans.get("confidence").cloned().unwrap_or(Value::Null),
                            }));
                        }
                    }
                }
                // memory_salience: the shadow fires on ledger writes (which
                // always store); a "skip" verdict contradicts the write.
                "memory_salience" => {
                    if let Some(ans) = answers.get("salience").and_then(|a| a.as_object()) {
                        if ans.get("choice") == Some(&json!("skip")) {
                            pairs.push(json!({
                                "surface": surface,
                                "question_key": "salience",
                                "state": state,
                                "chosen": "skip",
                                "rejected": ["store"],
                                "source": "system1_flag",
                                "model_confidence": ans.get("confidence").cloned().unwrap_or(Value::Null),
                            }));
                        }
                    }
                }
                // context_compaction: "continue" is the default; any other
                // action at flag band is the correction.
                "context_compaction" => {
                    if let Some(ans) = answers.get("action").and_then(|a| a.as_object()) {
                        if let Some(ch) = ans.get("choice").and_then(|c| c.as_str()) {
                            if ch != "continue" {
                                pairs.push(json!({
                                    "surface": surface,
                                    "question_key": "action",
                                    "state": state,
                                    "chosen": ch,
                                    "rejected": ["continue"],
                                    "source": "system1_flag",
                                    "model_confidence": ans.get("confidence").cloned().unwrap_or(Value::Null),
                                }));
                            }
                        }
                    }
                }
                // mandate_canary: operative is the default; a drifted flag
                // is the correction.
                "mandate_canary" => {
                    if let Some(ans) = answers.get("mandates").and_then(|a| a.as_object()) {
                        if ans.get("choice") == Some(&json!("drifted")) {
                            pairs.push(json!({
                                "surface": surface,
                                "question_key": "mandates",
                                "state": state,
                                "chosen": "drifted",
                                "rejected": ["operative"],
                                "source": "system1_flag",
                                "model_confidence": ans.get("confidence").cloned().unwrap_or(Value::Null),
                            }));
                        }
                    }
                }
                _ => {}
            }
        }

        // 3. Council Override / Prescreen
        if surface == json!("council_prescreen") {
            let cons = meta.get("council_consensus").and_then(|v| v.as_str());
            match cons {
                Some("reject") | Some("escalate") => pairs.push(json!({
                    "surface": surface,
                    "question_key": "council_review",
                    "state": state,
                    "chosen": "escalate",
                    "rejected": ["routine"],
                    "source": "council_override",
                })),
                Some("approve") => pairs.push(json!({
                    "surface": surface,
                    "question_key": "council_review",
                    "state": state,
                    "chosen": "routine",
                    "rejected": ["escalate"],
                    "source": "council_approval",
                })),
                _ => {}
            }
        }
    }
    pairs
}

/// `process_signals_and_dockets` — pairs from audit-signal JSONs and
/// council docket markdown.
fn process_signals_and_dockets(dockets_dir: &Path, signals_dir: &Path) -> Vec<Value> {
    let mut pairs = Vec::new();

    if signals_dir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(signals_dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) != Some("json") {
                    continue;
                }
                let data: Value = match std::fs::read_to_string(&p)
                    .ok()
                    .and_then(|t| serde_json::from_str(&t).ok())
                {
                    Some(d) => d,
                    None => continue,
                };
                if data.get("signal") != Some(&json!("council_verdict")) {
                    continue;
                }
                let scores = obj_or_empty(data.get("member_verdicts"));
                let mut passed = true;
                for stages in scores.values() {
                    if let Some(st) = stages.as_object() {
                        let stage2 = st.get("stage2").and_then(|v| v.as_f64()).unwrap_or(10.0);
                        if stage2 <= 3.0 {
                            passed = false;
                        }
                    }
                }
                if passed && !scores.is_empty() {
                    let sum: f64 = scores
                        .values()
                        .filter(|s| s.is_object())
                        .map(|s| s.get("stage2").and_then(|v| v.as_f64()).unwrap_or(0.0))
                        .sum();
                    let avg = sum / scores.len() as f64;
                    if avg < 6.5 {
                        passed = false;
                    }
                }
                let chosen = if passed { "routine" } else { "escalate" };
                let rejected = if passed { "escalate" } else { "routine" };
                pairs.push(json!({
                    "surface": "council_audit",
                    "question_key": "audit_verdict",
                    "state": {
                        "run_id": data.get("run_id").cloned().unwrap_or(Value::Null),
                        "mode": data.get("mode").cloned().unwrap_or(Value::Null),
                        "member_verdicts": data.get("member_verdicts").cloned().unwrap_or(Value::Null),
                    },
                    "chosen": chosen,
                    "rejected": [rejected],
                    "source": "council_signal_json",
                }));
            }
        }
    }

    // Council verdict files (~/.ciel/council/*.verdict.json) — the
    // Chairman-serialized shape: {"verdict": "pass"|"reject"|"deadlock"}.
    let council_dir = if dockets_dir
        .file_name()
        .map(|n| n == "dockets")
        .unwrap_or(false)
    {
        dockets_dir
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| dockets_dir.to_path_buf())
    } else {
        dockets_dir.to_path_buf()
    };
    if council_dir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(&council_dir) {
            for e in rd.flatten() {
                let p = e.path();
                let is_verdict = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".verdict.json"))
                    .unwrap_or(false);
                if !is_verdict {
                    continue;
                }
                let data: Value = match std::fs::read_to_string(&p)
                    .ok()
                    .and_then(|t| serde_json::from_str(&t).ok())
                {
                    Some(d) => d,
                    None => continue,
                };
                let verdict = data.get("verdict").and_then(|v| v.as_str()).unwrap_or("");
                if !matches!(verdict, "pass" | "reject" | "deadlock") {
                    continue;
                }
                let passed = verdict == "pass";
                pairs.push(json!({
                    "surface": "council_audit",
                    "question_key": "audit_verdict",
                    "state": {
                        "run_id": data.get("run_id").cloned()
                            .unwrap_or_else(|| json!(p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default())),
                        "weighted_score": data.get("weighted_score").cloned().unwrap_or(Value::Null),
                        "votes": data.get("votes").cloned().unwrap_or(Value::Null),
                    },
                    "chosen": if passed { "routine" } else { "escalate" },
                    "rejected": [if passed { "escalate" } else { "routine" }],
                    "source": "council_verdict_json",
                }));
            }
        }
    }

    if dockets_dir.is_dir() {
        if let Ok(rd) = std::fs::read_dir(dockets_dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let Ok(content) = std::fs::read_to_string(&p) else {
                    // Python propagates read errors; degrade to skip here
                    // rather than abort the whole pipeline.
                    continue;
                };
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if content.contains("VERDICT: REJECT")
                    || content.contains("Veto Condition Triggered")
                {
                    pairs.push(json!({
                        "surface": "council_audit",
                        "question_key": "audit_verdict",
                        "state": {"docket": name, "summary": "Council Vetoed or Rejected"},
                        "chosen": "escalate",
                        "rejected": ["routine"],
                        "source": "council_docket_md",
                    }));
                } else if content.contains("VERDICT: APPROVE") || content.contains("Exemplary Pass")
                {
                    pairs.push(json!({
                        "surface": "council_audit",
                        "question_key": "audit_verdict",
                        "state": {"docket": name, "summary": "Council Approved"},
                        "chosen": "routine",
                        "rejected": ["escalate"],
                        "source": "council_docket_md",
                    }));
                }
            }
        }
    }
    pairs
}

pub fn main_(args: &[String]) -> i32 {
    // The Python pipeline has no argparse surface; --help/-h still short-
    // circuits here so a stray flag can never trigger a write.
    for a in args {
        match a.as_str() {
            "-h" | "--help" => {
                println!("usage: ciel-dev rlcd");
                return 0;
            }
            _ => {}
        }
    }
    let home = py::ciel_home();
    let events_file = home.join("system1").join("events.jsonl");
    let dockets_dir = home.join("council").join("dockets");
    let signals_dir = home.join("improvements").join("signals");
    let out_file = home.join("system1").join("rlcd_pairs.jsonl");

    let mut all_pairs = process_events(&events_file);
    all_pairs.extend(process_signals_and_dockets(&dockets_dir, &signals_dir));

    if let Some(parent) = out_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut f = match std::fs::File::create(&out_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[RLCD Pipeline] cannot write {}: {e}", out_file.display());
            return 1;
        }
    };
    for p in &all_pairs {
        let _ = writeln!(f, "{}", jsonfmt::dumps_raw(p));
    }

    println!("[RLCD Pipeline] Processed events, dockets, and signals.");
    println!(
        "[RLCD Pipeline] Generated {} calibration pairs.",
        all_pairs.len()
    );
    println!("[RLCD Pipeline] Output saved to: {}", out_file.display());
    0
}
