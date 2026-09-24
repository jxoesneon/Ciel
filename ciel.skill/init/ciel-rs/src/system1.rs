//! System-1 decision client — Rust port of `hooks/lib/system1.py`.
//!
//! Speaks `POST {url}/v1/systemone` with typed choice/noul/score questions.
//! Every public function is fail-open: bounded timeout, None on any error,
//! never raises into the caller. `CIEL_SYSTEM1_DISABLED=1` is the global kill
//! switch.
//!
//! `ask_async` spawns a detached `ciel system1 --ask` re-exec (zero added
//! latency for callers). The child consults the response cache, posts, and
//! appends the verdict to `system1/events.jsonl`. In-flight work is bounded
//! by MAX_INFLIGHT marker files under `system1/inflight/` — saturation drops
//! the request rather than queueing unboundedly.
//!
//! HTTP: plain `http://` (the documented local laya-serve endpoint) is
//! handled by a minimal HTTP/1.1 client in-process; `https://` hosted
//! backends go through `curl` — the one subprocess retained, since TLS is
//! genuinely not worth a vendored stack here. `system1_embed.py` stays
//! Python (sentence-transformers) and is spawned exactly as before.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;

use crate::paths;

const EVENTS_LOG_MAX: u64 = 4 * 1024 * 1024;
const MAX_INFLIGHT: usize = 2;
const INFLIGHT_STALE_S: u64 = 120;
const ASK_TIMEOUT_S: f64 = 30.0;
const DEFAULT_TAU: f64 = 0.2;

// Advisory banding per surface — mirror of SURFACE_FLAGS.
fn surface_flag(surface: &str) -> &'static [&'static str] {
    match surface {
        "pre_tool_risk" => &["dangerous"],
        "council_prescreen" => &["escalate"],
        "completion_check" => &["incomplete"],
        _ => &[],
    }
}

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

/// Mirror of `tool_state` — deterministic enrichment, verdict-free.
pub fn tool_state(tool: &str, command: &str, path: &str) -> Value {
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

fn disabled() -> bool {
    std::env::var_os("CIEL_SYSTEM1_DISABLED").is_some()
}

fn env_file_value(names: &[&str]) -> String {
    let env_file = paths::ciel_home().join("system1").join("env");
    if let Ok(text) = std::fs::read_to_string(env_file) {
        for line in text.lines() {
            for name in names {
                if let Some(rest) = line.strip_prefix(&format!("{name}=")) {
                    return rest.trim().to_string();
                }
            }
        }
    }
    String::new()
}

fn key() -> String {
    std::env::var("CIEL_SYSTEM1_KEY")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| env_file_value(&["CIEL_SYSTEM1_KEY", "LAYA_API_KEY"]))
}

fn url() -> String {
    std::env::var("CIEL_SYSTEM1_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8765".into())
        .trim_end_matches('/')
        .to_string()
}

fn model() -> String {
    let direct = std::env::var("CIEL_SYSTEM1_MODEL")
        .unwrap_or_default()
        .trim()
        .to_string();
    if !direct.is_empty() {
        return direct;
    }
    env_file_value(&["CIEL_SYSTEM1_MODEL"])
}

// -------------------------------------------------------------- HTTP layer

/// Minimal HTTP/1.1 POST for `http://` endpoints — connection: close,
/// read-to-end, chunked-decode when the server uses it. `https://` goes
/// through curl (TLS without a vendored stack). Returns the body on 2xx.
fn http_post(url: &str, body: &[u8], auth: &str, timeout: Duration) -> Option<String> {
    if url.starts_with("https://") {
        return curl_post(url, body, auth, timeout);
    }
    let rest = url.strip_prefix("http://")?;
    let (authority, path) = match rest.split_once('/') {
        Some((a, p)) => (a, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    let addr = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{authority}:80")
    };
    let sock = addr.to_socket_addrs().ok()?.next()?;
    let mut stream = TcpStream::connect_timeout(&sock, timeout).ok()?;
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {authority}\r\ncontent-type: application/json\r\nauthorization: Bearer {auth}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(req.as_bytes()).ok()?;
    stream.write_all(body).ok()?;
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).ok()?;
    let text = String::from_utf8_lossy(&resp);
    let (head, raw) = text.split_once("\r\n\r\n")?;
    let status_ok = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .is_some_and(|c| (200..300).contains(&c));
    if !status_ok {
        return None;
    }
    let chunked = head.to_lowercase().contains("transfer-encoding: chunked");
    if !chunked {
        return Some(raw.to_string());
    }
    // minimal dechunk: <hex>\r\n<data>\r\n ... 0\r\n\r\n
    let mut out = String::new();
    let mut rest = raw;
    while let Some((size_line, after)) = rest.split_once("\r\n") {
        let Ok(size) = usize::from_str_radix(size_line.trim(), 16) else {
            break;
        };
        if size == 0 {
            break;
        }
        if after.len() < size + 2 {
            out.push_str(&after[..after.len().min(size)]);
            break;
        }
        out.push_str(&after[..size]);
        rest = &after[size + 2..];
    }
    Some(out)
}

fn curl_post(url: &str, body: &[u8], auth: &str, timeout: Duration) -> Option<String> {
    let out = Command::new("curl")
        .args([
            "-sS",
            "-X",
            "POST",
            "-H",
            "content-type: application/json",
            "-H",
            &format!("authorization: Bearer {auth}"),
            "--max-time",
            &format!("{}", timeout.as_secs_f64()),
            "--data-binary",
            "@-",
            url,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut c| {
            if let Some(mut stdin) = c.stdin.take() {
                let _ = stdin.write_all(body);
            }
            c.wait_with_output()
        })
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8(out.stdout).ok()
}

// --------------------------------------------------------------- ask layer

/// Mirror of `ask` — POST state+questions, return {"answers","model"} or
/// None on any failure.
pub fn ask(state: &Value, questions: &Value, timeout_s: f64) -> Option<Value> {
    if disabled() {
        return None;
    }
    let mut body = json!({"state": state, "questions": questions});
    let m = model();
    if !m.is_empty() {
        body["model"] = json!(m);
    }
    let resp = http_post(
        &format!("{}/v1/systemone", url()),
        body.to_string().as_bytes(),
        &key(),
        Duration::from_secs_f64(timeout_s.max(0.05)),
    )?;
    let data: Value = serde_json::from_str(&resp).ok()?;
    let answers = data.get("answers")?.clone();
    if !answers.is_object() {
        return None;
    }
    // python `or` semantics: any falsy routing.model falls through
    let routing_model = data
        .get("routing")
        .and_then(|r| r.get("model"))
        .cloned()
        .filter(|v| !(v.is_null() || v == &json!("") || v == &json!(false) || v == &json!(0)));
    let model = routing_model.or_else(|| data.get("model").cloned());
    Some(json!({"answers": answers, "model": model.unwrap_or(Value::Null)}))
}

// The library-only helper surfaces (`ask_choice`, `route_choice`,
// `shortlist_options`, `council_prescreen`, lexical/semantic ranking) have no
// caller through the binary's subcommands — the Python fallback retains them
// for in-process importers such as `risk_policy.py` and `system1_eval.py`.

// -------------------------------------------------------- cache + event log

fn cache_path(state: &Value, questions: &Value) -> PathBuf {
    let canonical = crate::jsonfmt::dumps_sorted(&json!({"s": state, "q": questions}));
    let digest: String = Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    paths::ciel_home()
        .join("system1")
        .join("cache")
        .join(format!("{digest}.json"))
}

fn cache_read(state: &Value, questions: &Value) -> Option<Value> {
    let data: Value =
        serde_json::from_str(&std::fs::read_to_string(cache_path(state, questions)).ok()?).ok()?;
    data.is_object().then_some(data)
}

fn cache_write(state: &Value, questions: &Value, result: &Value) {
    let path = cache_path(state, questions);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, crate::jsonfmt::dumps_raw(result));
}

fn append_event(record: &Value) {
    let log = paths::ciel_home().join("system1").join("events.jsonl");
    if let Some(parent) = log.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if log
        .metadata()
        .map(|m| m.len() > EVENTS_LOG_MAX)
        .unwrap_or(false)
    {
        let _ = std::fs::rename(&log, log.with_extension("jsonl.1"));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log)
    {
        let _ = writeln!(f, "{}", crate::jsonfmt::dumps_raw(record));
    }
}

fn band(surface: &str, answers: &Value) -> &'static str {
    let flag_set = surface_flag(surface);
    let tau = std::env::var("CIEL_SYSTEM1_TAU")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(DEFAULT_TAU);
    let mut worst = "pass";
    if let Some(obj) = answers.as_object() {
        for answer in obj.values() {
            let Some(a) = answer.as_object() else {
                continue;
            };
            if a.get("choice")
                .and_then(|c| c.as_str())
                .is_some_and(|c| flag_set.contains(&c))
            {
                return "flag";
            }
            let conf_ok = a
                .get("confidence")
                .and_then(|c| c.as_f64())
                .is_some_and(|c| c >= tau);
            if !conf_ok {
                worst = "uncertain";
            }
        }
    }
    worst
}

// ------------------------------------------------- detached ask (markers)

fn inflight_dir() -> PathBuf {
    paths::ciel_home().join("system1").join("inflight")
}

/// Live in-flight markers; reaps stale ones (>INFLIGHT_STALE_S).
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

/// Detached ask — mirror of `ask_async`: kill-switch, saturation drop,
/// marker file, detached `ciel system1 --ask` re-exec fed the payload on
/// stdin with `CIEL_SYSTEM1_MARKER` set.
pub fn ask_async(payload: &Value) {
    if disabled() {
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
    let exe = std::env::var_os("CIEL_BIN")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok());
    let Some(exe) = exe else {
        let _ = std::fs::remove_file(&marker);
        return;
    };
    let mut cmd = Command::new(exe);
    cmd.arg("system1")
        .arg("--ask")
        .env("CIEL_SYSTEM1_MARKER", &marker)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let spawned = cmd.spawn().map(|mut child| {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(payload.to_string().as_bytes());
        }
    });
    if spawned.is_err() {
        let _ = std::fs::remove_file(&marker);
    }
}

// ------------------------------------------------------------ ask mains

fn resolve(state: &Value, questions: &Value) -> (Option<Value>, bool, u64) {
    if let Some(cached) = cache_read(state, questions) {
        return (Some(cached), true, 0);
    }
    let started = Instant::now();
    let result = ask(state, questions, ASK_TIMEOUT_S);
    let latency_ms = started.elapsed().as_millis() as u64;
    if let Some(r) = &result {
        cache_write(state, questions, r);
    }
    (result, false, latency_ms)
}

fn event_record(payload: &Value, result: Option<&Value>, hit: bool, latency_ms: u64) -> Value {
    let surface = payload
        .get("surface")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown");
    let meta_ts = payload
        .get("meta")
        .and_then(|m| m.get("ts"))
        .and_then(|t| t.as_str())
        .map(String::from);
    let ts = meta_ts.unwrap_or_else(|| {
        OffsetDateTime::now_utc()
            .format(&time::macros::format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
            ))
            .unwrap_or_default()
    });
    let flag = result
        .map(|r| band(surface, &r["answers"]))
        .unwrap_or("pass");
    let mut record = json!({
        "ts": ts,
        "surface": surface,
        "questions": payload.get("questions").cloned().unwrap_or(json!({})),
        "state": payload.get("state").cloned().unwrap_or(json!({})),
        "meta": payload.get("meta").cloned().unwrap_or(json!({})),
        "system1": result.cloned().unwrap_or(Value::Null),
        "flag": flag,
        "cache_hit": hit,
    });
    if !hit {
        record["latency_ms"] = json!(latency_ms);
    }
    record
}

fn read_stdin_payload() -> Value {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    serde_json::from_str(&buf).unwrap_or_else(|_| json!({}))
}

fn ask_main() -> i32 {
    let marker = std::env::var("CIEL_SYSTEM1_MARKER").ok();
    let payload = read_stdin_payload();
    if payload.is_object() && !payload.as_object().unwrap().is_empty() {
        let empty = json!({});
        let (result, hit, latency_ms) = resolve(
            payload.get("state").unwrap_or(&empty),
            payload.get("questions").unwrap_or(&empty),
        );
        append_event(&event_record(&payload, result.as_ref(), hit, latency_ms));
    }
    if let Some(m) = marker {
        let _ = std::fs::remove_file(Path::new(&m));
    }
    0
}

fn decide_main() -> i32 {
    let payload = read_stdin_payload();
    if !payload.is_object() || payload.as_object().unwrap().is_empty() {
        println!("null");
        return 0;
    }
    let empty = json!({});
    let (result, hit, latency_ms) = resolve(
        payload.get("state").unwrap_or(&empty),
        payload.get("questions").unwrap_or(&empty),
    );
    append_event(&event_record(&payload, result.as_ref(), hit, latency_ms));
    println!(
        "{}",
        result
            .map(|r| crate::jsonfmt::dumps_raw(&r))
            .unwrap_or_else(|| "null".into())
    );
    0
}

/// `ciel system1 --ask | --decide`.
pub fn main_(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--ask") {
        return ask_main();
    }
    if args.iter().any(|a| a == "--decide") {
        return decide_main();
    }
    eprintln!(
        "usage: ciel system1 --ask | --decide  (reads JSON payload on stdin; \
         --decide prints the verdict)"
    );
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_dumps_matches_python() {
        // python: json.dumps({"s":..., "q":...}, sort_keys=True)
        let v = json!({"b": 1, "a": {"y": [true, null], "x": "héllo"}});
        assert_eq!(
            r#"{"a": {"x": "h\u00e9llo", "y": [true, null]}, "b": 1}"#,
            crate::jsonfmt::dumps_sorted(&v)
        );
        // default separators, insertion order, raw unicode
        let v2 = json!({"b": "héllo", "a": 1});
        assert_eq!(r#"{"b": "héllo", "a": 1}"#, crate::jsonfmt::dumps_raw(&v2));
    }

    #[test]
    fn band_flag_and_uncertain() {
        let answers = json!({"a": {"choice": "dangerous", "confidence": 0.9}});
        assert_eq!("flag", band("pre_tool_risk", &answers));
        let low = json!({"a": {"choice": "safe", "confidence": 0.1}});
        assert_eq!("uncertain", band("pre_tool_risk", &low));
        let ok = json!({"a": {"choice": "safe", "confidence": 0.9}});
        assert_eq!("pass", band("pre_tool_risk", &ok));
    }

    #[test]
    fn event_record_shape() {
        let p = json!({"surface": "pre_tool_risk", "state": {"a": 1},
                       "questions": {}, "meta": {"ts": "2026-01-01T00:00:00Z"}});
        let rec = event_record(&p, None, false, 42);
        assert_eq!("2026-01-01T00:00:00Z", rec["ts"]);
        assert_eq!("pass", rec["flag"]);
        assert_eq!(42, rec["latency_ms"]);
        assert!(rec["system1"].is_null());
        let cached = event_record(&p, None, true, 0);
        assert!(cached.get("latency_ms").is_none());
    }
}
