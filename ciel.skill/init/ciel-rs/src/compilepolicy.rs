//! `ciel compile-policy [--check]` — Rust port of `scripts/compile_policy.py`.
//!
//! Compiles `risk/policy.yaml` → `risk/policy.json` (the stdlib-only twin the
//! PreToolUse hooks load). Policy dir resolution mirrors the runtime rule:
//! first anchor-named ancestor (`.ciel`/`ciel.skill`) of this executable that
//! has `risk/policy.yaml`, else `ciel_home()/risk`.
//!
//! `--check` verifies the JSON twin is byte-identical to a fresh compile —
//! the same contract `json.dumps(data, indent=2, ensure_ascii=False) + "\n"`
//! gives in Python (insertion-ordered keys, minimal escapes).

use serde_json::{Map, Value};
use std::path::PathBuf;

use crate::paths;

const ANCHORS: [&str; 2] = [".ciel", "ciel.skill"];

fn policy_dir() -> Option<PathBuf> {
    if let Some(explicit) = std::env::var_os("CIEL_POLICY_DIR") {
        let d = PathBuf::from(explicit);
        if d.join("policy.yaml").is_file() {
            return Some(d);
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors() {
            if anc
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| ANCHORS.contains(&n))
            {
                let d = anc.join("risk");
                if d.join("policy.yaml").is_file() {
                    return Some(d);
                }
            }
        }
    }
    let d = paths::ciel_home().join("risk");
    d.join("policy.yaml").is_file().then_some(d)
}

/// serde_yaml::Value → serde_json::Value preserving mapping order — the
/// Python path is `yaml.safe_load` → `json.dumps`, and dicts preserve the
/// document order, so the JSON twin must too for `--check` to be byte-exact.
fn yaml_to_json(v: serde_yaml::Value) -> Value {
    match v {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => Value::Bool(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i)
            } else if let Some(u) = n.as_u64() {
                Value::from(u)
            } else {
                n.as_f64().map(Value::from).unwrap_or(Value::Null)
            }
        }
        serde_yaml::Value::String(s) => Value::String(s),
        serde_yaml::Value::Sequence(seq) => {
            Value::Array(seq.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(m) => {
            let mut obj = Map::new();
            for (k, val) in m {
                let key = match k {
                    serde_yaml::Value::String(s) => s,
                    serde_yaml::Value::Number(n) => n.to_string(),
                    serde_yaml::Value::Bool(b) => b.to_string(),
                    _ => continue,
                };
                obj.insert(key, yaml_to_json(val));
            }
            Value::Object(obj)
        }
        serde_yaml::Value::Tagged(t) => yaml_to_json(t.value),
    }
}

fn compiled(dir: &std::path::Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(dir.join("policy.yaml"))
        .map_err(|e| format!("policy.yaml: {e}"))?;
    let data: serde_yaml::Value =
        serde_yaml::from_str(&text).map_err(|e| format!("policy.yaml: {e}"))?;
    let data = yaml_to_json(data);
    match data.get("rules").and_then(|r| r.as_array()) {
        Some(_) => Ok(data),
        None => Err("policy.yaml: expected a mapping with a 'rules' list".into()),
    }
}

/// `json.dumps(data, indent=2, ensure_ascii=False) + "\n"` — serde_json's
/// pretty printer emits the same separators/indents; ensure_ascii=False is
/// serde_json's only mode (non-ASCII is never escaped).
fn render(data: &Value) -> String {
    let mut s = serde_json::to_string_pretty(data).unwrap_or_default();
    s.push('\n');
    s
}

/// `ciel compile-policy [--check]`.
pub fn main_(args: &[String]) -> i32 {
    let check = args.iter().any(|a| a == "--check");
    let Some(dir) = policy_dir() else {
        eprintln!("[policy] no risk/policy.yaml resolvable");
        return 2;
    };
    let data = match compiled(&dir) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let rendered = render(&data);
    let n = data["rules"].as_array().map(|a| a.len()).unwrap_or(0);
    let json_path = dir.join("policy.json");

    if check {
        if !json_path.is_file() {
            println!("[policy] policy.json missing; run scripts/compile_policy.py");
            return 1;
        }
        match std::fs::read_to_string(&json_path) {
            Ok(existing) if existing == rendered => {
                println!("[policy] policy.json in sync ({n} rules)");
                0
            }
            _ => {
                println!("[policy] policy.json is stale; run scripts/compile_policy.py");
                1
            }
        }
    } else {
        match std::fs::write(&json_path, rendered) {
            Ok(()) => {
                println!("[policy] wrote {} ({n} rules)", json_path.display());
                0
            }
            Err(e) => {
                eprintln!("[policy] write failed: {e}");
                1
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yaml_order_preserved() {
        let y: serde_yaml::Value = serde_yaml::from_str("b: 1\na: 2\n").unwrap();
        let j = yaml_to_json(y);
        let keys: Vec<&String> = j.as_object().unwrap().keys().collect();
        assert_eq!(vec!["b", "a"], keys);
    }
}
