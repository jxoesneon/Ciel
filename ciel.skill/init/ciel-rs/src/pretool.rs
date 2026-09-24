//! `ciel pretool --runtime {devin|antigravity}` — the full PreToolUse hook
//! body in one process: payload parse → policy evaluate → activity append →
//! detached System-1 shadow → advisory scan dispatch → decision emit.
//! Mirrors `hooks/{devin,antigravity}/pre_tool_use.sh` exactly — the .sh
//! files remain the runtime adapters and fall back to the Python body when
//! the binary is absent or fails.

use serde_json::{json, Map, Value};
use std::io::{Read, Write};

use crate::{attribution, paths, risk, shadow};

fn get_str<'a>(v: &'a Value, keys: &[&str]) -> &'a str {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(|x| x.as_str()) {
            if !s.is_empty() {
                return s;
            }
        }
    }
    ""
}

fn append_activity(entry: &Value) {
    let _ = std::fs::create_dir_all(paths::ciel_home());
    paths::activity_log(entry);
}

fn out(v: &Value) {
    let _ = writeln!(std::io::stdout(), "{v}");
}

pub fn main_(runtime: &str) -> i32 {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    let payload: Map<String, Value> = serde_json::from_str::<Value>(&buf)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();

    // Field extraction — mirrors each .sh's payload shape.
    let (tool, command, path) = match runtime {
        "antigravity" => {
            let call = payload.get("toolCall").cloned().unwrap_or(json!({}));
            let args = call
                .get("args")
                .or_else(|| payload.get("toolInput"))
                .cloned()
                .unwrap_or(json!({}));
            (
                call.get("name")
                    .and_then(|n| n.as_str())
                    .or_else(|| payload.get("toolName").and_then(|n| n.as_str()))
                    .unwrap_or("unknown")
                    .to_string(),
                get_str(&args, &["CommandLine", "command"]).to_string(),
                get_str(&args, &["path", "file_path", "filePath"]).to_string(),
            )
        }
        _ => {
            let input = payload
                .get("tool_input")
                .or_else(|| payload.get("toolInput"))
                .cloned()
                .unwrap_or(json!({}));
            (
                payload
                    .get("tool_name")
                    .and_then(|n| n.as_str())
                    .or_else(|| payload.get("toolName").and_then(|n| n.as_str()))
                    .unwrap_or("unknown")
                    .to_string(),
                get_str(&input, &["command", "CommandLine"]).to_string(),
                get_str(&input, &["file_path", "path", "notebook_path"])
                    .to_string(),
            )
        }
    };

    let verdict = risk::evaluate(&tool, &command, &path, None, None);
    let denied = verdict["decision"] == "deny";
    let override_ = verdict["decision"] == "allow_overridden";
    let ts = paths::utc_now_iso();

    let mut entry = json!({
        "ts": ts,
        "runtime": runtime,
        "event": "PreToolUse",
        "tool": tool,
        "risk": if denied { "critical" } else { "standard" },
        "rule_id": verdict["rule_id"],
        "tier": verdict["tier"],
        "policy": verdict["policy"],
        "overridden": override_,
    });
    if runtime == "antigravity" {
        entry["conversationId"] = payload
            .get("conversationId")
            .cloned()
            .unwrap_or(Value::Null);
    } else {
        entry["session_id"] = payload
            .get("session_id")
            .cloned()
            .unwrap_or(Value::Null);
        entry["prompt_id"] = payload
            .get("prompt_id")
            .cloned()
            .unwrap_or(Value::Null);
    }
    append_activity(&entry);

    // Shadow tier: detached semantic check — zero added latency.
    shadow::shadow_async(
        runtime,
        &ts,
        &tool,
        &command,
        &path,
        verdict["decision"].as_str().unwrap_or("allow"),
        &verdict["rule_id"],
    );

    // Advisory scans: devin hook dispatches `scan: attribution`.
    if runtime != "antigravity"
        && verdict["scan"].as_str() == Some("attribution")
        && !denied
    {
        let report = attribution::scan(&command);
        if report["result"].as_str() == Some("flagged") {
            let mut cats: Vec<String> = report["findings"]
                .as_array()
                .map(|fs| {
                    fs.iter()
                        .filter_map(|f| {
                            f["category"].as_str().map(String::from)
                        })
                        .collect()
                })
                .unwrap_or_default();
            cats.sort();
            cats.dedup();
            let joined = cats.join(", ");
            let mode = report["mode"].as_str().unwrap_or("shadow");
            entry["event"] = json!("PreToolUse+AttributionScan");
            entry["attribution"] = json!({"categories": cats, "mode": mode});
            append_activity(&entry);
            if mode == "enforce" {
                out(&json!({
                    "decision": "block",
                    "reason": format!(
                        "Ciel attribution gate: durable artifact carries \
                         forbidden patterns ({joined}). Remove them or set \
                         CIEL_ATTRIBUTION_SKIP=1 to bypass."),
                }));
            } else {
                out(&json!({
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "additionalContext": format!(
                            "Attribution scan flagged patterns ({joined}) in \
                             the staged artifact — verify no AI-attribution \
                             before publishing."),
                    }
                }));
            }
        }
    }

    if denied {
        let reason = verdict["reason"].as_str().unwrap_or("");
        let reason = if reason.is_empty() { "critical risk" } else { reason };
        if runtime == "antigravity" {
            out(&json!({
                "decision": "deny",
                "reason": format!(
                    "Ciel safety gate [{}]: {reason}",
                    verdict["rule_id"].as_str().unwrap_or("")),
            }));
        } else {
            out(&json!({
                "decision": "block",
                "reason": format!(
                    "Ciel safety gate [{}]: {reason} Run it manually outside \
                     the agent or narrow the operation before retrying.",
                    verdict["rule_id"].as_str().unwrap_or("")),
            }));
        }
    } else if runtime == "antigravity" {
        if override_ {
            out(&json!({
                "decision": "allow",
                "reason": format!(
                    "Ciel safety gate [{}]: permitted by local \
                     allow_privileged override.",
                    verdict["rule_id"].as_str().unwrap_or("")),
            }));
        } else {
            out(&json!({
                "decision": "allow",
                "reason": "Ciel pre-flight check passed.",
            }));
        }
    }
    // Devin: silent on allow/override (log only) — matches the .sh body.
    0
}
