//! `review` — port of `scripts/system1_review.py`. Prints shadow-log
//! records banded `flag`/`uncertain` (the advisory review queue).

use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::{jsonfmt, py};

struct Args {
    all: bool,
    surface: Option<String>,
    stats: bool,
    log: PathBuf,
}

fn parse_args(args: &[String]) -> Result<Args, i32> {
    let mut a = Args {
        all: false,
        surface: None,
        stats: false,
        log: py::ciel_home().join("system1").join("events.jsonl"),
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--all" => a.all = true,
            "--stats" => a.stats = true,
            "--surface" => {
                i += 1;
                a.surface = Some(args.get(i).cloned().ok_or(2)?);
            }
            "--log" => {
                i += 1;
                a.log = args.get(i).map(PathBuf::from).ok_or(2)?;
            }
            "-h" | "--help" => return Err(0),
            _ => return Err(2),
        }
        i += 1;
    }
    Ok(a)
}

/// Python `x or default`: falsy JSON values (null, false, 0, "", [], {})
/// fall through to the default; everything else is kept verbatim.
fn or_default(v: Option<&Value>, default: &str) -> Value {
    match v {
        None | Some(Value::Null) | Some(Value::Bool(false)) => json!(default),
        Some(Value::Number(n)) if n.as_f64() == Some(0.0) => json!(default),
        Some(Value::String(s)) if s.is_empty() => json!(default),
        Some(Value::Array(a)) if a.is_empty() => json!(default),
        Some(Value::Object(o)) if o.is_empty() => json!(default),
        Some(other) => other.clone(),
    }
}

fn band_key(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => jsonfmt::dumps_raw(other),
    }
}

const USAGE: &str = "usage: ciel-dev review [--all] [--surface NAME] [--stats] [--log FILE]";

pub fn main_(args: &[String]) -> i32 {
    let a = match parse_args(args) {
        Ok(a) => a,
        Err(code) => {
            if code == 0 {
                println!("{USAGE}");
            } else {
                eprintln!("{USAGE}");
            }
            return code;
        }
    };

    if !a.log.is_file() {
        println!("[system1_review] no events.jsonl — no shadow traffic yet");
        return 0;
    }

    let Ok(text) = std::fs::read_to_string(&a.log) else {
        println!("[system1_review] no events.jsonl — no shadow traffic yet");
        return 0;
    };

    let mut shown = 0u64;
    let mut stats: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for line in text.lines() {
        let Ok(rec) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let surface = or_default(rec.get("surface"), "unknown");
        let band = or_default(rec.get("flag"), "pass");
        *stats
            .entry(band_key(&surface))
            .or_default()
            .entry(band_key(&band))
            .or_insert(0) += 1;
        if let Some(s) = &a.surface {
            if surface.as_str() != Some(s.as_str()) {
                continue;
            }
        }
        if !a.all && band == Value::String("pass".into()) {
            continue;
        }
        let answers = rec
            .get("system1")
            .and_then(|s| s.get("answers"))
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(Map::new);
        let mut detail = Map::new();
        for (k, v) in &answers {
            if let Some(ans) = v.as_object() {
                detail.insert(
                    k.clone(),
                    json!({
                        "choice": ans.get("choice").cloned().unwrap_or(Value::Null),
                        "confidence": ans.get("confidence").cloned().unwrap_or(Value::Null),
                    }),
                );
            }
        }
        let out = json!({
            "ts": rec.get("ts").cloned().unwrap_or(Value::Null),
            "surface": surface,
            "flag": band,
            "meta": rec.get("meta").cloned().unwrap_or(Value::Null),
            "answers": Value::Object(detail),
            "cache_hit": rec.get("cache_hit").cloned().unwrap_or(Value::Null),
        });
        println!("{}", jsonfmt::dumps_raw(&out));
        shown += 1;
    }

    if a.stats || shown == 0 {
        for (surface, bands) in &stats {
            let parts: Vec<String> = bands.iter().map(|(b, n)| format!("{b}={n}")).collect();
            println!("[stats] {surface}: {}", parts.join(" "));
        }
    }
    if !a.stats {
        println!("[system1_review] {shown} record(s) shown");
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn or_default_falsy_table() {
        for v in [
            None,
            Some(&json!(null)),
            Some(&json!(false)),
            Some(&json!(0)),
            Some(&json!(0.0)),
            Some(&json!("")),
            Some(&json!([])),
            Some(&json!({})),
        ] {
            assert_eq!(or_default(v, "X"), json!("X"), "{v:?}");
        }
        for v in [json!("surf"), json!(1), json!({"a": 1}), json!([0])] {
            assert_eq!(or_default(Some(&v), "X"), v, "{v:?}");
        }
    }

    #[test]
    fn band_key_handles_nonstring() {
        assert_eq!(band_key(&json!("flag")), "flag");
        assert_eq!(band_key(&json!(null)), "null");
        assert_eq!(band_key(&json!(3)), "3");
    }
}
