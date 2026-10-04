//! `export` — port of `scripts/system1_export.py`. Streams the System-1
//! shadow log into RLCD preference pairs.

use serde_json::{json, Map, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use crate::{jsonfmt, py, secrets::scrub_secrets};

fn risk_label(decision: &str) -> Option<&'static str> {
    match decision {
        "deny" | "allow_overridden" => Some("dangerous"),
        "allow" => Some("safe"),
        _ => None,
    }
}

/// `_extract_truth` → (truth_label, is_weak, source).
fn extract_truth(surface: &str, meta: &Map<String, Value>) -> (Option<String>, bool, String) {
    if let Some(expected) = meta.get("expected") {
        if !expected.is_null() {
            return (
                Some(json_scalar_string(expected)),
                false,
                "meta.expected".into(),
            );
        }
    }
    match surface {
        "pre_tool_risk" => {
            if let Some(reg) = meta.get("regex_decision").and_then(|v| v.as_str()) {
                if !reg.is_empty() {
                    return (
                        risk_label(reg).map(String::from),
                        reg == "allow",
                        "regex_decision".into(),
                    );
                }
            }
        }
        "router" | "router_selection" => {
            // `meta.get("chosen_skill") or meta.get("successful_skill")`
            let skill = meta
                .get("chosen_skill")
                .filter(|v| py::json_truthy(v))
                .or_else(|| meta.get("successful_skill").filter(|v| py::json_truthy(v)));
            if let Some(skill) = skill {
                return (
                    Some(json_scalar_string(skill)),
                    false,
                    "router_outcome".into(),
                );
            }
        }
        "completion_check" => {
            if let Some(v) = meta.get("verified") {
                // `"verified" in meta` — presence check, then truthiness.
                let truth = if py::json_truthy(v) {
                    "complete"
                } else {
                    "incomplete"
                };
                return (Some(truth.into()), false, "completion_outcome".into());
            }
        }
        "council_prescreen" => {
            if let Some(cons) = meta.get("council_consensus").filter(|v| py::json_truthy(v)) {
                // `cons == "approve"` — any other truthy value escalates.
                let truth = if cons.as_str() == Some("approve") {
                    "routine"
                } else {
                    "escalate"
                };
                return (Some(truth.into()), false, "council_consensus".into());
            }
        }
        _ => {}
    }
    (None, false, "unknown".into())
}

/// `str(v)` for the truth-label path (`str(meta["expected"])`).
fn json_scalar_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        Value::Null => "None".into(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

fn compute_margin(probs: &Map<String, Value>) -> f64 {
    if probs.len() < 2 {
        return 0.0;
    }
    let mut vals: Vec<f64> = probs.values().filter_map(|v| v.as_f64()).collect();
    vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    if vals.len() < 2 {
        return 0.0;
    }
    vals[0] - vals[1]
}

const FALLBACK_REJECTED: [&str; 6] = [
    "safe",
    "dangerous",
    "routine",
    "escalate",
    "complete",
    "incomplete",
];

/// `_pairs(rec)` — preference pairs harvested from one event record.
fn pairs(rec: &Value) -> Vec<Value> {
    let surface = rec
        .get("surface")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("unknown")
        .to_string();
    let meta = rec
        .get("meta")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    let answers = rec
        .get("system1")
        .and_then(|s| s.get("answers"))
        .and_then(|a| a.as_object())
        .cloned()
        .unwrap_or_default();
    // `rec.get("state") or {}` — falsy values collapse to {}.
    let raw_state = rec
        .get("state")
        .filter(|v| py::json_truthy(v))
        .cloned()
        .unwrap_or(json!({}));
    let scrubbed_state = scrub_secrets(&raw_state);

    let (truth, weak, source) = extract_truth(&surface, &meta);
    let Some(truth) = truth else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for (qkey, answer) in &answers {
        let Some(ans) = answer.as_object() else {
            continue;
        };
        let empty = Map::new();
        let probs = ans
            .get("probabilities")
            .and_then(|p| p.as_object())
            .unwrap_or(&empty);
        // `truth not in (probs or {truth: 1})` — empty probs passes.
        if !probs.is_empty() && !probs.contains_key(&truth) {
            continue;
        }
        let rejected: Vec<Value> = {
            let mut r: Vec<Value> = probs
                .keys()
                .filter(|o| *o != &truth)
                .map(|o| json!(o))
                .collect();
            if r.is_empty() {
                r = FALLBACK_REJECTED
                    .iter()
                    .filter(|o| **o != truth.as_str())
                    .map(|o| json!(o))
                    .collect();
            }
            r
        };
        let margin = compute_margin(probs);
        out.push(json!({
            "surface": surface,
            "question_key": qkey,
            "state": scrubbed_state,
            "state_ref": meta.get("ts").cloned().unwrap_or(Value::Null),
            "chosen": truth,
            "rejected": rejected,
            "model_choice": ans.get("choice").cloned().unwrap_or(Value::Null),
            "model_confidence": ans.get("confidence").cloned().unwrap_or(Value::Null),
            "margin": margin,
            "weak": weak,
            "source": source,
        }));
    }
    out
}

struct Args {
    log: PathBuf,
    out: PathBuf,
    surface: String,
    min_confidence: f64,
    min_margin: f64,
}

fn parse_args(args: &[String]) -> Result<Args, i32> {
    let home = py::ciel_home();
    let mut a = Args {
        log: home.join("system1").join("events.jsonl"),
        out: home.join("system1").join("rlcd_pairs.jsonl"),
        surface: String::new(),
        min_confidence: 0.0,
        min_margin: 0.0,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--log" => {
                i += 1;
                a.log = args.get(i).map(PathBuf::from).ok_or(2)?;
            }
            "--out" => {
                i += 1;
                a.out = args.get(i).map(PathBuf::from).ok_or(2)?;
            }
            "--surface" => {
                i += 1;
                a.surface = args.get(i).cloned().ok_or(2)?;
            }
            "--min-confidence" => {
                i += 1;
                a.min_confidence = args.get(i).and_then(|s| s.parse().ok()).ok_or(2)?;
            }
            "--min-margin" => {
                i += 1;
                a.min_margin = args.get(i).and_then(|s| s.parse().ok()).ok_or(2)?;
            }
            "-h" | "--help" => return Err(0),
            _ => return Err(2),
        }
        i += 1;
    }
    Ok(a)
}

pub fn main_(args: &[String]) -> i32 {
    let a = match parse_args(args) {
        Ok(a) => a,
        Err(code) => {
            if code != 0 {
                eprintln!("usage: ciel-dev export [--log FILE] [--out FILE] [--surface NAME] [--min-confidence F] [--min-margin F]");
            } else {
                println!("usage: ciel-dev export [--log FILE] [--out FILE] [--surface NAME] [--min-confidence F] [--min-margin F]");
            }
            return code;
        }
    };

    if !a.log.is_file() {
        println!("[system1_export] no events.jsonl");
        return 1;
    }
    let out_path: &Path = &a.out;
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let Ok(in_fh) = std::fs::File::open(&a.log) else {
        println!("[system1_export] no events.jsonl");
        return 1;
    };
    let mut out_fh = match std::fs::File::create(&a.out) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[system1_export] cannot write {}: {e}", a.out.display());
            return 1;
        }
    };

    let mut written = 0u64;
    let mut skipped = 0u64;
    for line in BufReader::new(in_fh).lines() {
        let Ok(line) = line else { continue };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(rec) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if !a.surface.is_empty()
            && rec.get("surface").and_then(|v| v.as_str()) != Some(a.surface.as_str())
        {
            continue;
        }
        for pair in pairs(&rec) {
            let conf = pair.get("model_confidence").and_then(|v| v.as_f64());
            if let Some(c) = conf {
                if c < a.min_confidence {
                    skipped += 1;
                    continue;
                }
            }
            let margin = pair.get("margin").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if margin < a.min_margin {
                skipped += 1;
                continue;
            }
            let _ = writeln!(out_fh, "{}", jsonfmt::dumps_raw(&pair));
            written += 1;
        }
    }
    println!(
        "[system1_export] wrote {written} pairs (skipped {skipped}) -> {}",
        a.out.display()
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(surface: &str, meta: Value, answers: Value) -> Value {
        json!({
            "surface": surface,
            "meta": meta,
            "state": {"cmd": "x"},
            "system1": {"answers": answers},
        })
    }

    #[test]
    fn truth_meta_expected_wins() {
        let meta = json!({"expected": "safe", "regex_decision": "deny"});
        let (t, weak, src) = extract_truth("pre_tool_risk", meta.as_object().unwrap());
        assert_eq!(
            (t.as_deref(), weak, src.as_str()),
            (Some("safe"), false, "meta.expected")
        );
    }

    #[test]
    fn truth_regex_decision_labels() {
        let meta = json!({"regex_decision": "deny"});
        let (t, weak, src) = extract_truth("pre_tool_risk", meta.as_object().unwrap());
        assert_eq!(
            (t.as_deref(), weak, src.as_str()),
            (Some("dangerous"), false, "regex_decision")
        );
        let meta = json!({"regex_decision": "allow"});
        let (t, weak, _) = extract_truth("pre_tool_risk", meta.as_object().unwrap());
        assert_eq!((t.as_deref(), weak), (Some("safe"), true));
        let meta = json!({"regex_decision": "allow_overridden"});
        let (t, _, _) = extract_truth("pre_tool_risk", meta.as_object().unwrap());
        assert_eq!(t.as_deref(), Some("dangerous"));
    }

    #[test]
    fn truth_router_and_completion_and_council() {
        let meta = json!({"chosen_skill": "git-ops"});
        let (t, _, s) = extract_truth("router", meta.as_object().unwrap());
        assert_eq!(
            (t.as_deref(), s.as_str()),
            (Some("git-ops"), "router_outcome")
        );

        let meta = json!({"verified": false});
        let (t, _, s) = extract_truth("completion_check", meta.as_object().unwrap());
        assert_eq!(
            (t.as_deref(), s.as_str()),
            (Some("incomplete"), "completion_outcome")
        );

        let meta = json!({"council_consensus": "approve"});
        let (t, _, s) = extract_truth("council_prescreen", meta.as_object().unwrap());
        assert_eq!(
            (t.as_deref(), s.as_str()),
            (Some("routine"), "council_consensus")
        );
        let meta = json!({"council_consensus": "reject"});
        let (t, _, _) = extract_truth("council_prescreen", meta.as_object().unwrap());
        assert_eq!(t.as_deref(), Some("escalate"));

        let meta = json!({});
        let (t, weak, s) = extract_truth("pre_tool_risk", meta.as_object().unwrap());
        assert_eq!((t, weak, s.as_str()), (None, false, "unknown"));
    }

    #[test]
    fn compute_margin_top1_minus_top2() {
        let probs = json!({"a": 0.7, "b": 0.2, "c": 0.1});
        let m = compute_margin(probs.as_object().unwrap());
        assert!((m - 0.5).abs() < 1e-9);
        assert_eq!(compute_margin(&Map::new()), 0.0);
        let one = json!({"a": 1.0});
        assert_eq!(compute_margin(one.as_object().unwrap()), 0.0);
    }

    #[test]
    fn pairs_shape_and_truth_filter() {
        let r = rec(
            "pre_tool_risk",
            json!({"regex_decision": "deny", "ts": "t0"}),
            json!({"risk_q": {"choice": "deny", "confidence": 0.9,
                              "probabilities": {"safe": 0.1, "dangerous": 0.9}}}),
        );
        let out = pairs(&r);
        assert_eq!(out.len(), 1);
        let p = &out[0];
        assert_eq!(p["surface"], "pre_tool_risk");
        assert_eq!(p["question_key"], "risk_q");
        assert_eq!(p["chosen"], "dangerous");
        assert_eq!(p["rejected"], json!(["safe"]));
        assert_eq!(p["state_ref"], "t0");
        assert_eq!(p["weak"], false);
        assert_eq!(p["source"], "regex_decision");

        // truth missing from a non-empty probs map → no pair.
        let r = rec(
            "pre_tool_risk",
            json!({"regex_decision": "allow"}),
            json!({"q": {"choice": "x", "probabilities": {"dangerous": 1.0}}}),
        );
        assert!(pairs(&r).is_empty());
    }

    #[test]
    fn pairs_scrubs_state_secrets() {
        let r = rec(
            "pre_tool_risk",
            json!({"regex_decision": "deny"}),
            json!({"q": {"choice": "deny", "probabilities": {}}}),
        );
        let mut r = r;
        r["state"] = json!({"key": "sk-abcdefghijklmnopqrstuvwxyz1"});
        let out = pairs(&r);
        assert_eq!(out[0]["state"]["key"], "[REDACTED_SECRET:API_KEY_PREFIXED]");
    }

    #[test]
    fn fallback_rejected_options() {
        // Empty probs → fallback rejected list minus the truth.
        let r = rec(
            "pre_tool_risk",
            json!({"regex_decision": "deny"}),
            json!({"q": {"choice": "dangerous", "probabilities": {}}}),
        );
        let out = pairs(&r);
        let rejected = out[0]["rejected"].as_array().unwrap();
        assert!(rejected.contains(&json!("safe")));
        assert!(!rejected.contains(&json!("dangerous")));
    }
}
