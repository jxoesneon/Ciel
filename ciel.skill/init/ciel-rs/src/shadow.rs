//! Detached System-1 shadow spawn — replicates `risk_policy
//! .system1_shadow_async` + `system1.ask_async` semantics parent-side: the
//! marker/inflight-cap contract stays identical while the ask itself still
//! runs via the Python `system1.py --ask` child (full port is a later phase).

use serde_json::{json, Value};
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::paths;

const MAX_INFLIGHT: usize = 2;
const INFLIGHT_STALE_S: u64 = 120;

const READ_ONLY_TOOLS: [&str; 13] = [
    "read",
    "grep",
    "find_file_by_name",
    "webfetch",
    "web_search",
    "get_output",
    "mcp_read_resource",
    "mcp_list_tools",
    "mcp_list_servers",
    "notebook_read",
    "skill",
    "read_subagent",
    "list_skills",
];
const WRITE_TOOLS: [&str; 3] = ["write", "edit", "notebook_edit"];
const SENSITIVE_MARKERS: [&str; 17] = [
    "/.ssh",
    "/.aws",
    "/.gnupg",
    "/.kube",
    "/.docker",
    "/.netrc",
    "/.npmrc",
    "/.pypirc",
    "/.ciel/hooks",
    "/.ciel/risk",
    "/.config/devin",
    "/etc/",
    "/usr/",
    "/bin/",
    "/sbin/",
    "/boot/",
    "/root/",
];

/// Mirror of `system1.tool_state` — deterministic enrichment, verdict-free.
fn tool_state(tool: &str, command: &str, path: &str) -> Value {
    let mut state = json!({"tool": tool, "command": command, "path": path});
    let mut side_effects: Vec<String> = Vec::new();

    if READ_ONLY_TOOLS.contains(&tool) {
        state["action"] = json!(format!("read data via {tool}"));
        state["reversibility"] = json!("read-only; no state change");
    } else if WRITE_TOOLS.contains(&tool) || (!path.is_empty() && command.is_empty()) {
        state["action"] = json!(format!(
            "create or modify the file at {}",
            if path.is_empty() {
                "(unknown path)"
            } else {
                path
            }
        ));
        state["reversibility"] =
            json!("reversible if the target is tracked by version control; destructive otherwise");
        side_effects.push("modifies the filesystem at the target path".into());
    } else if tool == "exec" || !command.is_empty() {
        let truncated: String = command.chars().take(200).collect();
        state["action"] = json!(format!("run shell command: {truncated}"));
        state["reversibility"] = json!(
            "depends on the command; writes, deletes, and package/system changes may be irreversible");
        side_effects
            .push("runs a subprocess that may change files, network, or system state".into());
    } else {
        state["action"] = json!(format!(
            "invoke {}",
            if tool.is_empty() { "a tool" } else { tool }
        ));
        state["reversibility"] = json!("unknown");
    }

    let haystack = format!("{path} {command}");
    if SENSITIVE_MARKERS.iter().any(|m| haystack.contains(m)) {
        state["targets_sensitive_path"] = json!(true);
        side_effects.push(
            "touches a credential store, agent configuration, or protected system path".into(),
        );
    }
    state["side_effects"] = json!(side_effects);
    state
}

fn questions() -> Value {
    json!({
        "risk": {
            "type": "choice",
            "instructions": "Is this tool call dangerous? Judge BOTH the command AND the file path it targets — a write to a sensitive path is dangerous even with no command.",
            "criteria": {
                "safe": "routine, reversible, or read-only operation on non-sensitive paths",
                "dangerous": "destructive, irreversible, privilege-escalating, credential/sensitive-path writing, or data-exfiltrating operation",
            },
        }
    })
}

fn inflight_dir() -> PathBuf {
    paths::ciel_home().join("system1").join("inflight")
}

fn inflight_count() -> usize {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut count = 0;
    if let Ok(rd) = std::fs::read_dir(inflight_dir()) {
        for e in rd.flatten() {
            if let Ok(md) = e.metadata() {
                let age = md
                    .modified()
                    .ok()
                    .and_then(|m| m.duration_since(UNIX_EPOCH).ok())
                    .map(|d| now.saturating_sub(d.as_secs()))
                    .unwrap_or(0);
                if age > INFLIGHT_STALE_S {
                    let _ = std::fs::remove_file(e.path());
                } else {
                    count += 1;
                }
            }
        }
    }
    count
}

/// Fire-and-forget shadow ask — mirrors `system1.ask_async`: disabled env,
/// saturation drop, marker file, detached `system1.py --ask` child fed the
/// payload on stdin with `CIEL_SYSTEM1_MARKER` set.
pub fn shadow_async(
    runtime: &str,
    ts: &str,
    tool: &str,
    command: &str,
    path: &str,
    regex_decision: &str,
    rule_id: &Value,
) {
    if env::var_os("CIEL_SYSTEM1_DISABLED").is_some() {
        return;
    }
    let d = inflight_dir();
    if std::fs::create_dir_all(&d).is_err() || inflight_count() >= MAX_INFLIGHT {
        return;
    }
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_nanos())
        .unwrap_or(0);
    let marker = d.join(format!("{}.{nanos}", std::process::id()));
    if std::fs::write(&marker, "").is_err() {
        return;
    }

    let lib = env::var_os("CIEL_HOOK_LIB")
        .map(PathBuf::from)
        .unwrap_or_else(|| paths::ciel_home().join("hooks").join("lib"));
    let payload = json!({
        "surface": "pre_tool_risk",
        "state": tool_state(tool, command, path),
        "questions": questions(),
        "meta": {
            "ts": ts,
            "runtime": runtime,
            "regex_decision": regex_decision,
            "rule_id": rule_id,
        },
    });
    let mut cmd = Command::new("python3");
    cmd.arg(lib.join("system1.py"))
        .arg("--ask")
        .env("CIEL_SYSTEM1_MARKER", &marker)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // Mirror Python's start_new_session=True so the child outlives the hook.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let spawned = cmd.spawn().and_then(|mut child| {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(payload.to_string().as_bytes());
        }
        Ok(())
    });
    if spawned.is_err() {
        let _ = std::fs::remove_file(&marker);
    }
    // child intentionally not waited on — detached fire-and-forget
}
