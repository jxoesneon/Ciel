//! `ciel pretool --runtime {devin|antigravity}` — the full PreToolUse hook
//! body in one process: payload parse → policy evaluate → activity append →
//! detached System-1 shadow → advisory scan dispatch → decision emit.
//! Mirrors `hooks/{devin,antigravity}/pre_tool_use.sh` exactly — the .sh
//! files remain the runtime adapters and fall back to the Python body when
//! the binary is absent or fails.

use serde_json::{json, Map, Value};
use std::io::{Read, Write};

use crate::{attribution, paths, risk, shadow, system1};

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
    // Python: print(json.dumps({...})) — spaced separators, ensure_ascii.
    let _ = writeln!(std::io::stdout(), "{}", crate::jsonfmt::dumps(v));
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
            // Python `or` chain: an empty object is falsy — fall through.
            let nonempty = |v: &Value| v.as_object().is_some_and(|m| !m.is_empty());
            let args = call
                .get("args")
                .filter(|v| nonempty(v))
                .or_else(|| payload.get("toolInput"))
                .cloned()
                .unwrap_or(json!({}));
            (
                call.get("name")
                    .and_then(|n| n.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        payload
                            .get("toolName")
                            .and_then(|n| n.as_str())
                            .filter(|s| !s.is_empty())
                    })
                    .unwrap_or("unknown")
                    .to_string(),
                get_str(&args, &["CommandLine", "command"]).to_string(),
                get_str(&args, &["path", "file_path", "filePath"]).to_string(),
            )
        }
        _ => {
            let input = payload
                .get("tool_input")
                .filter(|v| v.as_object().is_some_and(|m| !m.is_empty()))
                .or_else(|| payload.get("toolInput"))
                .cloned()
                .unwrap_or(json!({}));
            (
                payload
                    .get("tool_name")
                    .and_then(|n| n.as_str())
                    .filter(|s| !s.is_empty())
                    .or_else(|| {
                        payload
                            .get("toolName")
                            .and_then(|n| n.as_str())
                            .filter(|s| !s.is_empty())
                    })
                    .unwrap_or("unknown")
                    .to_string(),
                get_str(&input, &["command", "CommandLine"]).to_string(),
                get_str(&input, &["file_path", "path", "notebook_path"]).to_string(),
            )
        }
    };

    let verdict = risk::evaluate(&tool, &command, &path, None, None);
    let denied = verdict["decision"] == "deny";
    let override_ = verdict["decision"] == "allow_overridden";
    let grant = risk::grant_state();
    let privileged = override_ || grant["active"].as_bool().unwrap_or(false);
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
        entry["session_id"] = payload.get("session_id").cloned().unwrap_or(Value::Null);
        entry["prompt_id"] = payload.get("prompt_id").cloned().unwrap_or(Value::Null);
    }
    append_activity(&entry);

    // Active System-1 pipeline integration: evaluate semantic risk in the pipeline
    let s1_verdict = system1::evaluate_risk(
        runtime,
        &ts,
        &tool,
        &command,
        &path,
        verdict["decision"].as_str().unwrap_or("allow"),
        &verdict["rule_id"],
        1.5,
    );

    // In shadow mode, dispatch detached shadow (active evaluation was skipped)
    let s1_mode = system1::mode();
    if s1_mode == "shadow" && !system1::disabled() {
        shadow::shadow_async(
            runtime,
            &ts,
            &tool,
            &command,
            &path,
            verdict["decision"].as_str().unwrap_or("allow"),
            &verdict["rule_id"],
        );
    }

    // Advisory scans: devin hook dispatches `scan: attribution`.
    if runtime != "antigravity" && verdict["scan"].as_str() == Some("attribution") && !denied {
        let report = attribution::scan(&command);
        if report["result"].as_str() == Some("flagged") {
            let mut cats: Vec<String> = report["findings"]
                .as_array()
                .map(|fs| {
                    fs.iter()
                        .filter_map(|f| f["category"].as_str().map(String::from))
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

    let tau = std::env::var("CIEL_SYSTEM1_TAU")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.20);

    let s1_intercept = if let Some((ref choice, conf, b)) = s1_verdict {
        b == "flag" && choice == "dangerous" && conf >= tau
    } else {
        false
    };

    let s1_mode = system1::mode();
    let is_failsafe = if !denied
        && !privileged
        && s1_mode == "active"
        && s1_verdict.is_none()
        && !system1::disabled()
    {
        let cmd = command.to_lowercase();
        let cmd_dest = ["rm ", "mkfs", "dd ", "sudo ", "chmod 777"]
            .iter()
            .any(|&c| cmd.contains(c))
            || (cmd.contains("curl") && cmd.contains("bash"))
            || cmd == "rm";
        let state = system1::tool_state(&tool, &command, &path);
        let sensitive = state
            .get("targets_sensitive_path")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_write = tool == "write"
            || tool == "edit"
            || tool == "notebook_edit"
            || (!path.is_empty() && command.is_empty());
        let path_dest = sensitive && is_write;
        cmd_dest || path_dest
    } else {
        false
    };

    if is_failsafe {
        let reason = "System-1 daemon offline or timed out; destructive command held under fail-safe policy. Start laya-serve or set allow_privileged.";
        if runtime == "antigravity" {
            out(&json!({
                "decision": "deny",
                "rule_id": "system1_offline_failsafe",
                "reason": reason,
            }));
        } else {
            out(&json!({
                "decision": "block",
                "rule_id": "system1_offline_failsafe",
                "reason": reason,
            }));
        }
        return 0;
    }

    if denied || s1_intercept {
        let is_s1 = !denied && s1_intercept;
        let reason = if is_s1 {
            let (ref choice, conf, _) = s1_verdict.as_ref().unwrap();
            format!(
                "System-1 (Laya) semantic risk intercept: flagged as {choice} ({:.0}% confidence)",
                conf * 100.0
            )
        } else {
            let r = verdict["reason"].as_str().unwrap_or("");
            if r.is_empty() {
                "critical risk".to_string()
            } else {
                r.to_string()
            }
        };

        if is_s1 && privileged {
            entry["event"] = json!("PreToolUse+System1Override");
            entry["system1_override"] = json!(true);
            append_activity(&entry);
            if runtime == "antigravity" {
                out(&json!({
                    "decision": "allow",
                    "reason": "Ciel: System-1 semantic warning overridden by allow_privileged.",
                }));
            }
            return 0;
        }

        if is_s1 {
            entry["risk"] = json!("critical");
            entry["event"] = json!("PreToolUse+System1Intercept");
            entry["system1"] = json!({"choice": "dangerous", "action": "intercept"});
            append_activity(&entry);
        }

        let rule_tag = if is_s1 {
            "system1_semantic_risk"
        } else {
            verdict["rule_id"].as_str().unwrap_or("")
        };
        if runtime == "antigravity" {
            out(&json!({
                "decision": "deny",
                "reason": format!("Ciel safety gate [{rule_tag}]: {reason}"),
            }));
        } else {
            out(&json!({
                "decision": "block",
                "reason": format!(
                    "Ciel safety gate [{rule_tag}]: {reason} Run it manually outside the agent or narrow the operation before retrying."
                ),
            }));
        }
        return 0;
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
