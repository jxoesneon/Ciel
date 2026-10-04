//! `eval` — port of `scripts/system1_eval.py`. Calibration harness for the
//! System-1 decision tier: scores the configured endpoint against labeled
//! corpora and writes `ciel.skill/risk/system1_calibration.json`.

use regex::Regex;
use serde_json::{json, Map, Value};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Instant;

use crate::{jsonfmt, py, sys1};

static RAW_STATE: Mutex<bool> = Mutex::new(false);

fn expect_label(case: &Value) -> Option<String> {
    match case.get("expect").and_then(|v| v.as_str()) {
        Some("allow") => Some("safe".into()),
        Some("deny") | Some("allow_overridden") => Some("dangerous".into()),
        _ => None,
    }
}

fn risk_state(case: &Value) -> Value {
    let get = |k: &str| case.get(k).and_then(|v| v.as_str()).unwrap_or("");
    if *RAW_STATE.lock().unwrap() {
        json!({"tool": get("tool"), "command": get("command"), "path": get("path")})
    } else {
        sys1::tool_state(get("tool"), get("command"), get("path"))
    }
}

fn prescreen_state(case: &Value) -> Value {
    json!({"event": case.get("event").and_then(|v| v.as_str()).unwrap_or("")})
}

fn router_state(case: &Value) -> Value {
    json!({"task": case.get("task").and_then(|v| v.as_str()).unwrap_or("")})
}

fn router_questions(corpus: &Value) -> Value {
    json!({
        "route": {
            "type": "choice",
            "instructions": "Which skill should handle this task?",
            "criteria": corpus.get("candidates")
                .and_then(|c| c.as_object())
                .cloned()
                .unwrap_or_default(),
        }
    })
}

fn skill_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(d) = std::env::var("CIEL_SKILLS_DIR") {
        if !d.is_empty() {
            dirs.push(PathBuf::from(d));
        }
    }
    dirs.push(py::home_dir().join(".ciel").join("skills"));
    dirs.push(py::repo_root().join("skills"));
    dirs.into_iter().filter(|d| d.is_dir()).collect()
}

/// `{skill_id: description}` from the first populated skills dir.
fn registry_candidates() -> Map<String, Value> {
    let desc_re = Regex::new(r"(?m)^description:\s*(.+)$").unwrap();
    for d in skill_dirs() {
        let mut out = Map::new();
        let mut skill_mds: Vec<PathBuf> = std::fs::read_dir(&d)
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.path())
                    .filter(|p| p.is_dir() && p.join("SKILL.md").is_file())
                    .map(|p| p.join("SKILL.md"))
                    .collect()
            })
            .unwrap_or_default();
        skill_mds.sort();
        for skill_md in skill_mds {
            let Ok(text) = std::fs::read_to_string(&skill_md) else {
                continue;
            };
            let head: String = text.chars().take(2000).collect();
            let desc = if let Some(m) = desc_re.captures(&head) {
                m[1].trim().to_string()
            } else {
                let mut d = String::new();
                for line in head.lines() {
                    let line = line.trim();
                    if !line.is_empty()
                        && !line.starts_with('#')
                        && !line.starts_with("---")
                        && !line.starts_with("name:")
                    {
                        d = line.to_string();
                        break;
                    }
                }
                d
            };
            let name = skill_md
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            out.insert(name, json!(desc));
        }
        if !out.is_empty() {
            return out;
        }
    }
    Map::new()
}

static REGISTRY_CACHE: Mutex<Option<Map<String, Value>>> = Mutex::new(None);

fn router_registry_questions(case: &Value, _corpus: &Value) -> Value {
    {
        let mut cache = REGISTRY_CACHE.lock().unwrap();
        if cache.is_none() {
            *cache = Some(registry_candidates());
        }
    }
    let cache = REGISTRY_CACHE.lock().unwrap();
    let registry = cache.clone().unwrap_or_default();
    let task = case.get("task").and_then(|v| v.as_str()).unwrap_or("");
    let criteria = sys1::shortlist_options(task, &registry, 10);
    json!({
        "route": {
            "type": "choice",
            "instructions": "Which skill should handle this task?",
            "criteria": criteria,
        }
    })
}

struct SurfaceSpec {
    corpus: PathBuf,
    /// Static questions from the corpus, or per-case builder.
    questions_for_corpus: Option<fn(&Value) -> Value>,
    questions_for_case: Option<fn(&Value, &Value) -> Value>,
    state: fn(&Value) -> Value,
    truth: fn(&Value) -> Option<String>,
    positive: Option<&'static str>,
}

fn surfaces() -> Vec<(&'static str, SurfaceSpec)> {
    let fixtures = py::repo_root().join("tests").join("fixtures");
    vec![
        (
            "pre_tool_risk",
            SurfaceSpec {
                corpus: fixtures.join("hook_redteam_cases.json"),
                questions_for_corpus: Some(|_| sys1::risk_questions()),
                questions_for_case: None,
                state: risk_state,
                truth: expect_label,
                positive: Some("dangerous"),
            },
        ),
        (
            "council_prescreen",
            SurfaceSpec {
                corpus: fixtures.join("system1_prescreen_cases.json"),
                questions_for_corpus: Some(|_| sys1::prescreen_questions()),
                questions_for_case: None,
                state: prescreen_state,
                truth: |c| c.get("expected").and_then(|v| v.as_str()).map(String::from),
                positive: Some("escalate"),
            },
        ),
        (
            "router",
            SurfaceSpec {
                corpus: fixtures.join("system1_router_cases.json"),
                questions_for_corpus: Some(router_questions),
                questions_for_case: None,
                state: router_state,
                truth: |c| c.get("expected").and_then(|v| v.as_str()).map(String::from),
                positive: None,
            },
        ),
        (
            "router_registry",
            SurfaceSpec {
                corpus: fixtures.join("system1_router_cases.json"),
                questions_for_corpus: None,
                questions_for_case: Some(router_registry_questions),
                state: router_state,
                truth: |c| c.get("expected").and_then(|v| v.as_str()).map(String::from),
                positive: None,
            },
        ),
    ]
}

/// `_predict` — first answer's choice; any "dangerous" wins.
fn predict(answers: &Map<String, Value>) -> Option<String> {
    let choices: Vec<Option<String>> = answers
        .values()
        .filter(|a| a.is_object())
        .map(|a| a.get("choice").and_then(|c| c.as_str()).map(String::from))
        .collect();
    if choices.iter().any(|c| c.as_deref() == Some("dangerous")) {
        return Some("dangerous".into());
    }
    choices.into_iter().next().flatten()
}

/// `_confidence` — max numeric confidence across answers, else 0.0.
fn confidence(answers: &Map<String, Value>) -> f64 {
    answers
        .values()
        .filter(|a| a.is_object())
        .filter_map(|a| a.get("confidence").and_then(|c| c.as_f64()))
        .fold(0.0, f64::max)
}

fn evaluate_surface(name: &str, spec: &SurfaceSpec, timeout: f64) -> Result<Value, String> {
    let corpus_text = std::fs::read_to_string(&spec.corpus)
        .map_err(|e| format!("{}: {e}", spec.corpus.display()))?;
    let corpus: Value =
        serde_json::from_str(&corpus_text).map_err(|e| format!("corpus parse: {e}"))?;
    let cases: Vec<Value> = match &corpus {
        Value::Object(m) => m
            .get("cases")
            .and_then(|c| c.as_array())
            .cloned()
            .unwrap_or_default(),
        Value::Array(a) => a.clone(),
        _ => Vec::new(),
    };
    let mut questions = match spec.questions_for_case {
        None => spec
            .questions_for_corpus
            .map(|f| f(&corpus))
            .unwrap_or(json!({})),
        Some(_) => json!({}),
    };

    let (mut tp, mut fp, mut tn, mut fn_) = (0u64, 0u64, 0u64, 0u64);
    let (mut errors, mut correct) = (0u64, 0u64);
    let (mut in_options_total, mut in_options_hits) = (0u64, 0u64);
    let (mut conf_correct, mut conf_wrong, mut margins, mut latencies) = (
        Vec::<f64>::new(),
        Vec::<f64>::new(),
        Vec::<f64>::new(),
        Vec::<f64>::new(),
    );
    let mut per_case: Vec<Value> = Vec::new();

    for case in &cases {
        let Some(truth) = (spec.truth)(case) else {
            continue;
        };
        if let Some(f) = spec.questions_for_case {
            questions = f(case, &corpus);
        }
        let t0 = Instant::now();
        let result = sys1::ask(&(spec.state)(case), &questions, timeout);
        latencies.push(t0.elapsed().as_secs_f64());
        let Some(result) = result else {
            errors += 1;
            per_case.push(json!({
                "id": case.get("id").cloned().unwrap_or(Value::Null),
                "truth": truth,
                "pred": Value::Null,
            }));
            continue;
        };
        let answers = result
            .get("answers")
            .and_then(|a| a.as_object())
            .cloned()
            .unwrap_or_default();
        let pred = predict(&answers);
        let conf = confidence(&answers);
        let mut probs = Map::new();
        for a in answers.values() {
            if let Some(p) = a.get("probabilities").and_then(|p| p.as_object()) {
                for (k, v) in p {
                    probs.insert(k.clone(), v.clone());
                }
            }
        }
        let mut margin = Value::Null;
        if let Some(pv) = pred
            .as_ref()
            .and_then(|p| probs.get(p))
            .and_then(|v| v.as_f64())
        {
            let others: Vec<f64> = probs
                .iter()
                .filter(|(k, _)| Some(*k) != pred.as_ref())
                .filter_map(|(_, v)| v.as_f64())
                .collect();
            let m = pv - others.iter().cloned().fold(0.0, f64::max);
            margin = json!(m);
            margins.push(m);
        }
        per_case.push(json!({
            "id": case.get("id").cloned().unwrap_or(Value::Null),
            "truth": truth,
            "pred": pred,
            "confidence": conf,
            "margin": margin,
        }));
        match spec.positive {
            None => {
                // did the truth survive candidate reduction (shortlist recall)?
                let mut offered = std::collections::HashSet::new();
                if let Some(qs) = questions.as_object() {
                    for q in qs.values() {
                        if let Some(crit) = q.get("criteria").and_then(|c| c.as_object()) {
                            for k in crit.keys() {
                                offered.insert(k.clone());
                            }
                        }
                    }
                }
                if !offered.is_empty() {
                    in_options_total += 1;
                    if offered.contains(&truth) {
                        in_options_hits += 1;
                    }
                }
                if pred.as_deref() == Some(truth.as_str()) {
                    correct += 1;
                }
            }
            Some(pos) => {
                if pred.as_deref() == Some(pos) && truth == pos {
                    tp += 1;
                } else if pred.as_deref() == Some(pos) {
                    fp += 1;
                } else if truth == pos {
                    fn_ += 1;
                } else {
                    tn += 1;
                }
            }
        }
        if pred.as_deref() == Some(truth.as_str()) {
            conf_correct.push(conf);
        } else {
            conf_wrong.push(conf);
        }
    }

    if latencies.is_empty() {
        // Python: sum(latencies)/len(latencies) → ZeroDivisionError.
        return Err("zero evaluated cases (no latency samples)".into());
    }

    let mean = |v: &[f64]| -> Value {
        if v.is_empty() {
            Value::Null
        } else {
            json!(v.iter().sum::<f64>() / v.len() as f64)
        }
    };

    let mut base = json!({
        "surface": name,
        "cases": per_case.len(),
        "errors": errors,
        "mean_confidence_correct": mean(&conf_correct),
        "mean_confidence_wrong": mean(&conf_wrong),
        "mean_latency_s": latencies.iter().sum::<f64>() / latencies.len() as f64,
        "mean_margin": mean(&margins),
        "per_case": per_case,
    });

    let obj = base.as_object_mut().unwrap();
    match spec.positive {
        None => {
            obj.insert(
                "accuracy".into(),
                if per_case_is_empty(obj) {
                    Value::Null
                } else {
                    json!(correct as f64 / cases_count(obj) as f64)
                },
            );
            if in_options_total > 0 {
                obj.insert(
                    "shortlist_recall".into(),
                    json!(in_options_hits as f64 / in_options_total as f64),
                );
            }
        }
        Some(pos) => {
            let precision = if tp + fp > 0 {
                Some(tp as f64 / (tp + fp) as f64)
            } else {
                None
            };
            let recall = if tp + fn_ > 0 {
                Some(tp as f64 / (tp + fn_) as f64)
            } else {
                None
            };
            let f1 = match (precision, recall) {
                (Some(p), Some(r)) if p != 0.0 && r != 0.0 => Some(2.0 * p * r / (p + r)),
                _ => None,
            };
            obj.insert(
                "confusion".into(),
                json!({"tp": tp, "fp": fp, "tn": tn, "fn": fn_}),
            );
            obj.insert(format!("precision_{pos}"), opt_num(precision));
            obj.insert(format!("recall_{pos}"), opt_num(recall));
            obj.insert(format!("f1_{pos}"), opt_num(f1));
            obj.insert(
                "sweep_flag_uncertain_negative".into(),
                uncertain_sweep(obj.get("per_case").and_then(|v| v.as_array()).unwrap(), pos),
            );
        }
    }
    Ok(base)
}

fn per_case_is_empty(obj: &Map<String, Value>) -> bool {
    obj.get("per_case")
        .and_then(|v| v.as_array())
        .map(|a| a.is_empty())
        .unwrap_or(true)
}

fn cases_count(obj: &Map<String, Value>) -> usize {
    obj.get("cases").and_then(|v| v.as_u64()).unwrap_or(0) as usize
}

fn opt_num(v: Option<f64>) -> Value {
    v.map(|f| json!(f)).unwrap_or(Value::Null)
}

/// `_uncertain_sweep` — flag low-confidence negative-class predictions.
fn uncertain_sweep(per_case: &[Value], positive: &str) -> Value {
    let taus = [0.01, 0.05, 0.1, 0.15, 0.2, 0.25, 0.3, 0.4];
    let mut sweep = Vec::new();
    for tau in taus {
        let (mut caught, mut flagged, mut missed) = (0u64, 0u64, 0u64);
        for c in per_case {
            let conf = c.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let pred = c.get("pred").and_then(|v| v.as_str());
            if pred.is_none() || pred == Some(positive) {
                continue;
            }
            let truth = c.get("truth").and_then(|v| v.as_str());
            if truth == Some(positive) {
                if conf < tau {
                    caught += 1;
                } else {
                    missed += 1;
                }
            } else if conf < tau {
                flagged += 1;
            }
        }
        sweep.push(json!({
            "tau": tau,
            "missed_caught": caught,
            "missed_remaining": missed,
            "negative_flagged": flagged,
        }));
    }
    json!(sweep)
}

struct Args {
    surface: Option<String>,
    timeout: f64,
    stdout: bool,
    raw_state: bool,
}

fn parse_args(args: &[String]) -> Result<Args, i32> {
    let mut a = Args {
        surface: None,
        timeout: 30.0,
        stdout: false,
        raw_state: false,
    };
    let valid: Vec<String> = surfaces().iter().map(|(n, _)| n.to_string()).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--surface" => {
                i += 1;
                let v = args.get(i).ok_or(2)?;
                if !valid.contains(v) {
                    eprintln!(
                        "ciel-dev eval: invalid choice: '{v}' (choose from {})",
                        valid.join(", ")
                    );
                    return Err(2);
                }
                a.surface = Some(v.clone());
            }
            "--timeout" => {
                i += 1;
                a.timeout = args.get(i).and_then(|s| s.parse().ok()).ok_or(2)?;
            }
            "--stdout" => a.stdout = true,
            "--raw-state" => a.raw_state = true,
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
                eprintln!("usage: ciel-dev eval [--surface NAME] [--timeout SECS] [--stdout] [--raw-state]");
            } else {
                println!("usage: ciel-dev eval [--surface NAME] [--timeout SECS] [--stdout] [--raw-state]");
            }
            return code;
        }
    };
    *RAW_STATE.lock().unwrap() = a.raw_state;

    let all = surfaces();
    let specs: Vec<(&str, &SurfaceSpec)> = match &a.surface {
        Some(name) => all
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(n, s)| (*n, s))
            .collect(),
        None => all.iter().map(|(n, s)| (*n, s)).collect(),
    };

    let model = sys1::model();
    let mut report = json!({
        "generated": py::utc_stamp_z(),
        "endpoint": sys1::url(),
        "model": if model.is_empty() { "(endpoint default)".to_string() } else { model },
        "surfaces": {},
    });
    let surfaces_obj = report["surfaces"].as_object_mut().unwrap();
    for (name, spec) in &specs {
        if !spec.corpus.is_file() {
            surfaces_obj.insert(
                name.to_string(),
                json!({"error": format!("missing {}", spec.corpus.display())}),
            );
            continue;
        }
        match evaluate_surface(name, spec, a.timeout) {
            Ok(v) => surfaces_obj.insert(name.to_string(), v),
            Err(e) => {
                eprintln!("[system1_eval] {name}: {e}");
                return 1;
            }
        };
    }
    let rendered = format!("{}\n", jsonfmt::dumps_indent(&report, 2));
    if a.stdout {
        print!("{rendered}");
    } else {
        let out = py::repo_root()
            .join("ciel.skill")
            .join("risk")
            .join("system1_calibration.json");
        if let Err(e) = std::fs::write(&out, &rendered) {
            eprintln!("[system1_eval] cannot write {}: {e}", out.display());
            return 1;
        }
        for (name, s) in report["surfaces"].as_object().unwrap() {
            if let Some(err) = s.get("error") {
                println!("[system1_eval] {name}: {}", err.as_str().unwrap_or(""));
                continue;
            }
            let metric = if s.get("accuracy").is_some() {
                format!("accuracy={:.2}", s["accuracy"].as_f64().unwrap_or(f64::NAN))
            } else {
                // `or` semantics: falsy (0.0/None) falls to the escalate key.
                let p = s
                    .get("precision_dangerous")
                    .filter(|v| v.as_f64().map(|f| f != 0.0).unwrap_or(false))
                    .or_else(|| s.get("precision_escalate"));
                let r = s
                    .get("recall_dangerous")
                    .filter(|v| v.as_f64().map(|f| f != 0.0).unwrap_or(false))
                    .or_else(|| s.get("recall_escalate"));
                format!("P={} R={}", fmt_opt(p), fmt_opt(r))
            };
            println!(
                "[system1_eval] {name}: cases={} errors={} {metric}",
                s.get("cases").and_then(|v| v.as_u64()).unwrap_or(0),
                s.get("errors").and_then(|v| v.as_u64()).unwrap_or(0),
            );
        }
        println!("[system1_eval] -> {}", out.display());
    }
    0
}

fn fmt_opt(v: Option<&Value>) -> String {
    match v {
        Some(Value::Number(n)) => py::py_float(n.as_f64().unwrap_or(0.0)),
        Some(other) => jsonfmt::dumps_raw(other),
        None => "None".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expect_label_mapping() {
        assert_eq!(
            expect_label(&json!({"expect": "allow"})).as_deref(),
            Some("safe")
        );
        assert_eq!(
            expect_label(&json!({"expect": "deny"})).as_deref(),
            Some("dangerous")
        );
        assert_eq!(
            expect_label(&json!({"expect": "allow_overridden"})).as_deref(),
            Some("dangerous")
        );
        assert_eq!(expect_label(&json!({"expect": "x"})), None);
        assert_eq!(expect_label(&json!({})), None);
    }

    #[test]
    fn predict_dangerous_wins() {
        let answers = json!({
            "a": {"choice": "safe"},
            "b": {"choice": "dangerous"},
        });
        assert_eq!(
            predict(answers.as_object().unwrap()).as_deref(),
            Some("dangerous")
        );
        let answers = json!({"a": {"choice": "safe"}, "b": {"choice": "x"}});
        assert_eq!(
            predict(answers.as_object().unwrap()).as_deref(),
            Some("safe")
        );
        assert_eq!(predict(&Map::new()), None);
    }

    #[test]
    fn confidence_max_of_numerics() {
        let answers = json!({
            "a": {"confidence": 0.3},
            "b": {"confidence": 0.9},
            "c": {"confidence": "high"},
        });
        assert_eq!(confidence(answers.as_object().unwrap()), 0.9);
        assert_eq!(confidence(&Map::new()), 0.0);
    }

    #[test]
    fn uncertain_sweep_counts() {
        let per_case = vec![
            json!({"pred": "safe", "truth": "dangerous", "confidence": 0.02}), // missed, caught below tau
            json!({"pred": "safe", "truth": "dangerous", "confidence": 0.9}),  // missed, stays
            json!({"pred": "safe", "truth": "safe", "confidence": 0.01}),      // negative flagged
            json!({"pred": "dangerous", "truth": "safe", "confidence": 0.5}),  // skipped (pos pred)
            json!({"pred": null, "truth": "safe", "confidence": 0.0}),         // skipped
        ];
        let sweep = uncertain_sweep(&per_case, "dangerous");
        let tau03 = &sweep.as_array().unwrap()[5]; // tau = 0.25
        assert_eq!(tau03["missed_caught"], 1);
        assert_eq!(tau03["missed_remaining"], 1);
        assert_eq!(tau03["negative_flagged"], 1);
        let tau001 = &sweep.as_array().unwrap()[0]; // tau = 0.01 — nothing under
        assert_eq!(tau001["missed_caught"], 0);
        assert_eq!(tau001["missed_remaining"], 2);
        assert_eq!(tau001["negative_flagged"], 0);
    }

    #[test]
    fn risk_state_raw_and_enriched() {
        *RAW_STATE.lock().unwrap() = false;
        let case = json!({"tool": "exec", "command": "ls", "path": ""});
        let s = risk_state(&case);
        assert_eq!(s["tool"], "exec");
        assert_eq!(s["action"], "run shell command: ls");
        *RAW_STATE.lock().unwrap() = true;
        let s = risk_state(&case);
        assert_eq!(s, json!({"tool": "exec", "command": "ls", "path": ""}));
        *RAW_STATE.lock().unwrap() = false;
    }

    #[test]
    fn router_questions_wraps_candidates() {
        let q = router_questions(&json!({"candidates": {"a": "alpha"}}));
        assert_eq!(q["route"]["criteria"]["a"], "alpha");
        assert_eq!(q["route"]["type"], "choice");
        let q = router_questions(&json!({}));
        assert_eq!(q["route"]["criteria"], json!({}));
    }
}
