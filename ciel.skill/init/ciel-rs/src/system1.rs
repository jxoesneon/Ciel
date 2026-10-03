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

use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;

struct Redactor {
    cat: &'static str,
    re: Regex,
}

fn redactors() -> &'static [Redactor] {
    static REDACTORS: OnceLock<Vec<Redactor>> = OnceLock::new();
    REDACTORS.get_or_init(|| {
        vec![
            Redactor { cat: "github_token", re: Regex::new(r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b|\bgithub_pat_[A-Za-z0-9_]{20,}\b").unwrap() },
            Redactor { cat: "aws_access_key", re: Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap() },
            Redactor { cat: "api_key_prefixed", re: Regex::new(r"\b(?:sk|pk|key|api|tok)_[A-Za-z0-9_-]{20,}\b|\bsk-[A-Za-z0-9_-]{20,}\b").unwrap() },
            Redactor { cat: "private_key_block", re: Regex::new(r"-----BEGIN [A-Z ]*PRIVATE KEY").unwrap() },
            Redactor { cat: "slack_token", re: Regex::new(r"\bxox[baprs]-[0-9A-Za-z-]{10,}\b").unwrap() },
            Redactor { cat: "gcp_api_key", re: Regex::new(r"\bAIza[0-9A-Za-z_-]{35}\b").unwrap() },
            Redactor { cat: "jwt", re: Regex::new(r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b").unwrap() },
            Redactor { cat: "npm_token", re: Regex::new(r"\bnpm_[A-Za-z0-9]{36}\b").unwrap() },
            Redactor { cat: "crates_token", re: Regex::new(r"\bcio[0-9A-Za-z]{25,}\b").unwrap() },
            Redactor { cat: "password_assignment", re: Regex::new(r#"(?i)\b(?:sudo\s+)?(?:password|passwd|passphrase)\s*(?:is|:|=)\s*['"]?[^\s'"]{4,}"#).unwrap() },
            Redactor { cat: "secret_assignment", re: Regex::new(r#"(?i)\b(?:api[_-]?key|access[_-]?token|auth[_-]?token|secret[_-]?key|client[_-]?secret)\s*[:=]\s*['"]?[A-Za-z0-9_\-]{8,}"#).unwrap() },
            Redactor { cat: "generic_secret_kv", re: Regex::new(r#"(?i)\b(?:token|secret)\s+is\s+['"]?[A-Za-z0-9_\-]{8,}"#).unwrap() },
        ]
    })
}

use crate::paths;

const EVENTS_LOG_MAX: u64 = 4 * 1024 * 1024;
const MAX_INFLIGHT: usize = 2;
const INFLIGHT_STALE_S: u64 = 120;
const ASK_TIMEOUT_S: f64 = 30.0;
const DEFAULT_TAU: f64 = 0.2;

static THRESHOLDS_CACHE: std::sync::RwLock<Option<HashMap<String, f64>>> =
    std::sync::RwLock::new(None);

fn load_policy_thresholds() -> HashMap<String, f64> {
    {
        if let Ok(lock) = THRESHOLDS_CACHE.read() {
            if let Some(ref m) = *lock {
                return m.clone();
            }
        }
    }
    let mut map = HashMap::new();
    let candidates = [
        paths::ciel_home().join("risk").join("policy.json"),
        paths::ciel_home()
            .join("risk")
            .join("system1_calibration.json"),
    ];
    for p in &candidates {
        if let Ok(text) = std::fs::read_to_string(p) {
            if let Ok(val) = serde_json::from_str::<Value>(&text) {
                if let Some(obj) = val
                    .get("system1_thresholds")
                    .or_else(|| val.get("threshold_lattice"))
                    .and_then(|v| v.as_object())
                {
                    for (k, v) in obj {
                        if let Some(num) = v.as_f64() {
                            map.insert(k.clone(), num);
                        }
                    }
                    if !map.is_empty() {
                        break;
                    }
                }
            }
        }
    }
    if let Ok(mut lock) = THRESHOLDS_CACHE.write() {
        *lock = Some(map.clone());
    }
    map
}

pub fn surface_tau(surface: &str) -> f64 {
    // 1. Surface-specific override: CIEL_SYSTEM1_TAU_<SURFACE>
    let env_name = format!("CIEL_SYSTEM1_TAU_{}", surface.to_ascii_uppercase());
    if let Ok(val) = std::env::var(&env_name) {
        if let Ok(f) = val.parse::<f64>() {
            return f;
        }
    }
    // 2. Global override: CIEL_SYSTEM1_TAU
    if let Ok(val) = std::env::var("CIEL_SYSTEM1_TAU") {
        if let Ok(f) = val.parse::<f64>() {
            return f;
        }
    }
    // 3. Declarative policy.json / system1_calibration.json single source of truth
    let thresh = load_policy_thresholds();
    if let Some(&val) = thresh.get(surface) {
        return val;
    }
    // 4. Calibrated default lattice constants
    match surface {
        "pre_tool_risk" => 0.65,
        "router" | "router_selection" => 0.82,
        "completion_check" => 0.75,
        "council_prescreen" => 0.70,
        _ => DEFAULT_TAU,
    }
}

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

pub fn disabled() -> bool {
    if std::env::var_os("CIEL_SYSTEM1_DISABLED").is_some() {
        return true;
    }
    mode() == "off"
}

struct EnvCacheEntry {
    path: PathBuf,
    mtime: Option<SystemTime>,
    pairs: Vec<(String, String)>,
    last_check: Instant,
}

static ENV_CACHE: std::sync::RwLock<Option<EnvCacheEntry>> = std::sync::RwLock::new(None);

fn env_file_value(names: &[&str]) -> String {
    let env_file = paths::ciel_home().join("system1").join("env");
    let now = Instant::now();

    // Fast path: if checked within the last 1000ms, reuse cache without touching filesystem metadata
    {
        if let Ok(lock) = ENV_CACHE.read() {
            if let Some(ref entry) = *lock {
                if entry.path == env_file
                    && now.duration_since(entry.last_check) < Duration::from_millis(1000)
                {
                    for name in names {
                        for (k, v) in &entry.pairs {
                            if k == *name {
                                return v.clone();
                            }
                        }
                    }
                    return String::new();
                }
            }
        }
    }

    let current_mtime = env_file.metadata().ok().and_then(|m| m.modified().ok());

    {
        if let Ok(lock) = ENV_CACHE.read() {
            if let Some(ref entry) = *lock {
                if entry.path == env_file && entry.mtime == current_mtime {
                    for name in names {
                        for (k, v) in &entry.pairs {
                            if k == *name {
                                return v.clone();
                            }
                        }
                    }
                    return String::new();
                }
            }
        }
    }

    let mut lock = ENV_CACHE.write().unwrap_or_else(|e| e.into_inner());
    if let Some(ref entry) = *lock {
        if entry.path == env_file && entry.mtime == current_mtime {
            for name in names {
                for (k, v) in &entry.pairs {
                    if k == *name {
                        return v.clone();
                    }
                }
            }
            return String::new();
        }
    }

    let new_pairs = if let Ok(text) = std::fs::read_to_string(&env_file) {
        text.lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.is_empty() {
                    return None;
                }
                let (k, v) = trimmed.split_once('=')?;
                Some((
                    k.trim().to_string(),
                    v.trim().trim_matches('"').trim_matches('\'').to_string(),
                ))
            })
            .collect()
    } else {
        Vec::new()
    };
    *lock = Some(EnvCacheEntry {
        path: env_file,
        mtime: current_mtime,
        pairs: new_pairs.clone(),
        last_check: now,
    });

    for name in names {
        for (k, v) in &new_pairs {
            if k == *name {
                return v.clone();
            }
        }
    }
    String::new()
}

pub fn mode() -> String {
    std::env::var("CIEL_SYSTEM1_MODE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            let m = env_file_value(&["CIEL_SYSTEM1_MODE"]);
            if m.is_empty() {
                "active".into()
            } else {
                m
            }
        })
}

fn key() -> String {
    std::env::var("CIEL_SYSTEM1_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| env_file_value(&["CIEL_SYSTEM1_KEY", "LAYA_API_KEY"]))
}

fn url() -> String {
    let direct = std::env::var("CIEL_SYSTEM1_URL")
        .ok()
        .filter(|s| !s.trim().is_empty());
    if let Some(u) = direct {
        return u.trim_end_matches('/').to_string();
    }
    let file_url = env_file_value(&["CIEL_SYSTEM1_URL"]);
    if !file_url.is_empty() {
        return file_url.trim_end_matches('/').to_string();
    }
    let host = env_file_value(&["LAYA_HOST"]);
    let port = env_file_value(&["LAYA_PORT"]);
    if !host.is_empty() || !port.is_empty() {
        let h = if host.is_empty() { "127.0.0.1" } else { &host };
        let p = if port.is_empty() { "8765" } else { &port };
        return format!("http://{h}:{p}");
    }
    "http://127.0.0.1:8765".into()
}

fn model() -> String {
    let direct = std::env::var("CIEL_SYSTEM1_MODEL")
        .ok()
        .filter(|s| !s.trim().is_empty());
    if let Some(m) = direct {
        return m;
    }
    env_file_value(&["CIEL_SYSTEM1_MODEL", "LAYA_MODEL"])
}

// -------------------------------------------------------------- HTTP layer

fn find_header_split(bytes: &[u8]) -> Option<(usize, usize)> {
    if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
        return Some((pos, pos + 4));
    }
    if let Some(pos) = bytes.windows(2).position(|w| w == b"\n\n") {
        return Some((pos, pos + 2));
    }
    None
}

fn dechunk(raw: &str) -> String {
    let mut out = String::new();
    let mut rest = raw;
    let mut saw_zero = false;
    while let Some((size_line, after)) = rest.split_once("\r\n").or_else(|| rest.split_once('\n')) {
        let hex_part = size_line.split(';').next().unwrap_or("").trim();
        let Ok(size) = usize::from_str_radix(hex_part, 16) else {
            break;
        };
        if size == 0 {
            saw_zero = true;
            break;
        }
        if after.len() < size {
            break;
        }
        out.push_str(&after[..size]);
        let remaining = &after[size..];
        if let Some(stripped) = remaining.strip_prefix("\r\n") {
            rest = stripped;
        } else if let Some(stripped) = remaining.strip_prefix('\n') {
            rest = stripped;
        } else {
            rest = remaining;
        }
    }
    if !saw_zero {
        return String::new();
    }
    out
}

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
    let addrs = addr.to_socket_addrs().ok()?;
    let connect_timeout =
        if authority.starts_with("127.0.0.1") || authority.starts_with("localhost") {
            Duration::from_millis(10).min(timeout)
        } else {
            Duration::from_millis(50).min(timeout)
        };
    let mut stream = None;
    for sock in addrs {
        if let Ok(s) = TcpStream::connect_timeout(&sock, connect_timeout) {
            stream = Some(s);
            break;
        }
    }
    let mut stream = stream?;
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));

    let auth_header = if !auth.trim().is_empty() {
        format!("authorization: Bearer {}\r\n", auth.trim())
    } else {
        String::new()
    };

    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {authority}\r\ncontent-type: application/json\r\n{auth_header}content-length: {}\r\nconnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(req.as_bytes()).ok()?;
    stream.write_all(body).ok()?;

    let mut resp = Vec::new();
    let mut buf = [0u8; 4096];
    let mut header_body_split: Option<(usize, usize)> = None;
    let mut content_len: Option<usize> = None;
    let mut is_chunked = false;

    while let Ok(n) = stream.read(&mut buf) {
        if n == 0 {
            break;
        }
        resp.extend_from_slice(&buf[..n]);

        if header_body_split.is_none() {
            if let Some((hend, bstart)) = find_header_split(&resp) {
                header_body_split = Some((hend, bstart));
                let head_str = String::from_utf8_lossy(&resp[..hend]);
                for line in head_str.lines() {
                    let l = line.to_ascii_lowercase();
                    if l.starts_with("content-length:") {
                        if let Some(val) = line
                            .split_once(':')
                            .and_then(|(_, v)| v.trim().parse::<usize>().ok())
                        {
                            content_len = Some(val);
                        }
                    }
                    if l.starts_with("transfer-encoding:") && l.contains("chunked") {
                        is_chunked = true;
                    }
                }
            }
        }

        if let Some((_, bstart)) = header_body_split {
            if let Some(clen) = content_len {
                if resp.len() >= bstart + clen {
                    break;
                }
            } else if is_chunked {
                let body_slice = &resp[bstart..];
                if body_slice.ends_with(b"\r\n0\r\n\r\n")
                    || body_slice.ends_with(b"\n0\n\n")
                    || body_slice.ends_with(b"\r\n0\n\n")
                    || body_slice == b"0\r\n\r\n"
                    || body_slice == b"0\n\n"
                {
                    break;
                }
            }
        }
    }

    if resp.is_empty() {
        return None;
    }

    let (hend, bstart) = header_body_split.or_else(|| find_header_split(&resp))?;
    let head = String::from_utf8_lossy(&resp[..hend]);

    let status_ok = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok())
        .is_some_and(|c| (200..300).contains(&c));
    if !status_ok {
        return None;
    }

    let chunked = is_chunked
        || head.lines().any(|l| {
            let lower = l.to_ascii_lowercase();
            lower.starts_with("transfer-encoding:") && lower.contains("chunked")
        });

    if chunked {
        let raw = String::from_utf8_lossy(&resp[bstart..]);
        let decoded = dechunk(&raw);
        if decoded.is_empty() && raw.trim() != "0" {
            return None;
        }
        return Some(decoded);
    }

    let body_bytes = if let Some(clen) = content_len {
        if resp.len() < bstart + clen {
            // Premature EOF before full body received
            return None;
        }
        &resp[bstart..bstart + clen]
    } else {
        &resp[bstart..]
    };

    Some(String::from_utf8_lossy(body_bytes).to_string())
}

fn curl_post(url: &str, body: &[u8], auth: &str, timeout: Duration) -> Option<String> {
    let mut cmd = Command::new("curl");
    cmd.args([
        "-sS",
        "--fail",
        "--connect-timeout",
        "0.05",
        "-X",
        "POST",
        "-H",
        "content-type: application/json",
    ]);
    if !auth.trim().is_empty() {
        cmd.args(["-H", &format!("authorization: Bearer {}", auth.trim())]);
    }
    cmd.args([
        "--max-time",
        &format!("{:.3}", timeout.as_secs_f64().max(0.05)),
        "--data-binary",
        "@-",
        url,
    ]);
    let out = cmd
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

fn redact_string(s: &str) -> String {
    let mut out = s.to_string();
    for r in redactors() {
        out =
            r.re.replace_all(&out, format!("[REDACTED:{}]", r.cat).as_str())
                .into_owned();
    }
    out
}

fn redact_value(val: &mut Value) {
    match val {
        Value::String(s) => *s = redact_string(s),
        Value::Array(arr) => arr.iter_mut().for_each(redact_value),
        Value::Object(obj) => obj.values_mut().for_each(redact_value),
        _ => {}
    }
}

fn append_event(record: &Value) {
    let mut record = record.clone();
    redact_value(&mut record);
    let log = paths::ciel_home().join("system1").join("events.jsonl");

    if let Some(parent) = log.parent() {
        if !parent.is_dir() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log)
    {
        if let Ok(meta) = f.metadata() {
            if meta.len() > EVENTS_LOG_MAX {
                drop(f);
                let _ = std::fs::rename(&log, log.with_extension("jsonl.1"));
                if let Ok(mut f2) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log)
                {
                    let _ = writeln!(f2, "{}", crate::jsonfmt::dumps_raw(&record));
                }
                return;
            }
        }
        let _ = writeln!(f, "{}", crate::jsonfmt::dumps_raw(&record));
    }
}

fn band(surface: &str, answers: &Value) -> &'static str {
    let flag_set = surface_flag(surface);
    let tau = surface_tau(surface);
    let mut worst = "pass";
    if let Some(obj) = answers.as_object() {
        for (qname, answer) in obj {
            let Some(a) = answer.as_object() else {
                continue;
            };
            if let Some(choice) = a.get("choice").and_then(|c| c.as_str()) {
                if flag_set.contains(&choice) {
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
            if surface == "completion_check" && qname == "evidence_score" {
                // score is a probability-weighted mean over rubric indices —
                // a float (e.g. 2.85), not an integer.
                if let Some(score) = a.get("score").and_then(|s| s.as_f64()) {
                    if score < 4.0 {
                        worst = "uncertain";
                    }
                }
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

/// Synchronous pipeline evaluation of tool risk — active intercept tier.
/// Returns Some((choice, confidence, band)) if System-1 returned an answer,
/// or None if disabled, offline, or timed out (fail-open).
#[allow(clippy::too_many_arguments)]
pub fn evaluate_risk(
    runtime: &str,
    ts: &str,
    tool: &str,
    command: &str,
    path: &str,
    regex_decision: &str,
    rule_id: &Value,
    timeout_s: f64,
) -> Option<(String, f64, &'static str)> {
    if disabled() {
        return None;
    }
    let m = mode();
    if m == "shadow" || m == "off" {
        return None;
    }

    let state = tool_state(tool, command, path);
    let questions = crate::shadow::questions();

    // Fast-path: consult response cache first
    let (result, hit, latency_ms) = if let Some(cached) = cache_read(&state, &questions) {
        (Some(cached), true, 0)
    } else {
        let started = Instant::now();
        let r = ask(&state, &questions, timeout_s);
        let elapsed = started.elapsed().as_millis() as u64;
        if let Some(ref val) = r {
            cache_write(&state, &questions, val);
        }
        (r, false, elapsed)
    };

    let payload = json!({
        "surface": "pre_tool_risk",
        "state": state,
        "questions": questions,
        "meta": {
            "ts": ts,
            "runtime": runtime,
            "regex_decision": regex_decision,
            "rule_id": rule_id,
            "pipeline": "active",
        }
    });
    append_event(&event_record(&payload, result.as_ref(), hit, latency_ms));

    let r = result?;
    let risk_answer = r.get("answers")?.get("risk")?;
    let choice = risk_answer.get("choice")?.as_str()?.to_string();
    let conf = risk_answer.get("confidence")?.as_f64().unwrap_or(0.0);
    let b = band("pre_tool_risk", &r["answers"]);
    Some((choice, conf, b))
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompletionVerdict {
    pub choice: String,
    pub confidence: f64,
    pub band: &'static str,
    // score is the protocol's probability-weighted mean over rubric indices —
    // keep it f64: the fraction indicates which way the probability mass leans.
    pub score: Option<f64>,
}

/// Typed questions for the completion_check surface.
#[allow(dead_code)]
pub fn completion_questions() -> Value {
    completion_questions_with_score(true)
}

/// Typed questions with optional quality score for completion_check.
pub fn completion_questions_with_score(with_score: bool) -> Value {
    let mut q = json!({
        "done": {
            "type": "choice",
            "instructions": "Evaluate whether the objective is verifiably satisfied by the provided empirical evidence. When in doubt, mark incomplete if claims lack empirical verification artifacts (tests, execution logs, diffs, live probes).",
            "criteria": {
                "complete": "objective is fully satisfied with direct empirical proof and verification artifacts",
                "incomplete": "objective is unverified, missing required artifacts, failed verification, or asserts claims without evidence"
            }
        }
    });
    if with_score {
        q["evidence_score"] = json!({
            "type": "score",
            "instructions": "Rate how well empirical evidence substantiates the completion claim on the ordered rubric (lowest level = unverified/pure claim, highest level = complete empirical proof).",
            // /v1/systemone protocol: score questions take `criteria` as a
            // list of level descriptions, index 0 first — a keyed rubric dict
            // is rejected with a per-question schema error.
            "criteria": [
                "no evidence or contradictory evidence (pure assertion/hallucination)",
                "partial evidence with major unverified claims or failing tests",
                "indirect or ambiguous evidence without target-state verification",
                "direct empirical evidence verifying primary claims",
                "exhaustive empirical verification of all claims and task-class artifacts"
            ]
        });
    }
    q
}

/// Synchronous pipeline evaluation of completion evidence — verification tier.
/// Returns Some(CompletionVerdict) if System-1 returned an answer,
/// or None if disabled, offline, or timed out (fail-open).
pub fn evaluate_completion(
    objective: &str,
    evidence: &str,
    task_class: &str,
    timeout_s: f64,
) -> Option<CompletionVerdict> {
    if disabled() {
        return None;
    }
    let m = mode();
    if m == "shadow" || m == "off" {
        return None;
    }

    let mut state = json!({
        "objective": objective,
        "evidence": evidence,
    });
    if !task_class.is_empty() {
        state["task_class"] = json!(task_class);
    }
    let questions = completion_questions_with_score(true);

    // Fast-path: consult response cache first
    let (result, hit, latency_ms) = if let Some(cached) = cache_read(&state, &questions) {
        (Some(cached), true, 0)
    } else {
        let started = Instant::now();
        let r = ask(&state, &questions, timeout_s);
        let elapsed = started.elapsed().as_millis() as u64;
        if let Some(ref val) = r {
            cache_write(&state, &questions, val);
        }
        (r, false, elapsed)
    };

    let payload = json!({
        "surface": "completion_check",
        "state": state,
        "questions": questions,
        "meta": {
            "pipeline": "completion_verification",
        }
    });
    append_event(&event_record(&payload, result.as_ref(), hit, latency_ms));

    let r = result?;
    let done_answer = r.get("answers")?.get("done")?;
    let choice = done_answer.get("choice")?.as_str()?.to_string();
    let conf = done_answer.get("confidence")?.as_f64().unwrap_or(0.0);
    let b = band("completion_check", &r["answers"]);
    let score = r
        .get("answers")
        .and_then(|a| a.get("evidence_score"))
        .and_then(|s| s.get("score"))
        .and_then(|s| s.as_f64());

    Some(CompletionVerdict {
        choice,
        confidence: conf,
        band: b,
        score,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteVerdict {
    pub status: String,
    pub degraded: bool,
    pub choice: Option<String>,
    pub confidence: Option<f64>,
    pub margin: Option<f64>,
    pub shortlist: Vec<String>,
    pub model: Option<String>,
}

impl RouteVerdict {
    pub fn to_json(&self) -> Value {
        json!({
            "status": self.status,
            "degraded": self.degraded,
            "choice": self.choice,
            "confidence": self.confidence,
            "margin": self.margin,
            "shortlist": self.shortlist,
            "model": self.model,
        })
    }
}

pub fn route_choice(
    task: &str,
    options: &serde_json::Map<String, Value>,
    k: usize,
    timeout_s: f64,
) -> RouteVerdict {
    let mut candidate_keys: Vec<String> = options.keys().cloned().collect();
    candidate_keys.sort();

    let shortlist_keys: Vec<String> = if candidate_keys.len() <= k {
        candidate_keys
    } else {
        let task_lower = task.to_ascii_lowercase();
        let task_words: std::collections::HashSet<&str> = task_lower.split_whitespace().collect();
        let mut matched = Vec::new();
        for key in &candidate_keys {
            if task_words.contains(key.to_ascii_lowercase().as_str())
                || task_lower.contains(&key.to_ascii_lowercase())
            {
                matched.push(key.clone());
            }
        }
        for key in &candidate_keys {
            if !matched.contains(key) && matched.len() < k {
                matched.push(key.clone());
            }
        }
        matched
    };

    let mut criteria_obj = serde_json::Map::new();
    for key in &shortlist_keys {
        if let Some(val) = options.get(key) {
            criteria_obj.insert(key.clone(), val.clone());
        }
    }

    let questions = json!({
        "route": {
            "type": "choice",
            "instructions": "Which candidate best fits the task?",
            "criteria": criteria_obj
        }
    });

    let state = json!({
        "task": task,
        "candidates": shortlist_keys
    });

    let payload = json!({
        "surface": "router",
        "state": state,
        "questions": questions,
        "meta": {
            "pipeline": "router_selection"
        }
    });

    let (result, hit, latency_ms) = if let Some(cached) = cache_read(&state, &questions) {
        (Some(cached), true, 0)
    } else {
        let started = Instant::now();
        let r = ask(&state, &questions, timeout_s);
        let elapsed = started.elapsed().as_millis() as u64;
        if let Some(ref val) = r {
            cache_write(&state, &questions, val);
        }
        (r, false, elapsed)
    };
    append_event(&event_record(&payload, result.as_ref(), hit, latency_ms));

    if let Some(r) = result {
        if let Some(answer) = r.get("answers").and_then(|a| a.get("route")) {
            let choice = answer
                .get("choice")
                .and_then(|c| c.as_str())
                .map(ToString::to_string);
            let confidence = answer.get("confidence").and_then(|c| c.as_f64());
            let mut margin = None;
            if let Some(probs) = answer.get("probabilities").and_then(|p| p.as_object()) {
                let mut vals: Vec<f64> = probs.values().filter_map(|v| v.as_f64()).collect();
                vals.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
                if vals.len() >= 2 {
                    margin = Some(vals[0] - vals[1]);
                }
            }
            let model = r
                .get("model")
                .and_then(|m| m.as_str())
                .map(ToString::to_string);
            return RouteVerdict {
                status: "evaluated".into(),
                degraded: false,
                choice,
                confidence,
                margin,
                shortlist: shortlist_keys,
                model,
            };
        }
    }

    RouteVerdict {
        status: "fail_open".into(),
        degraded: true,
        choice: None,
        confidence: None,
        margin: None,
        shortlist: shortlist_keys,
        model: None,
    }
}

pub fn route_choice_main() -> i32 {
    let payload = read_stdin_payload();
    let task = payload
        .get("task")
        .or_else(|| payload.get("prompt"))
        .or_else(|| payload.get("objective"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let empty_map = serde_json::Map::new();
    let options = payload
        .get("options")
        .or_else(|| payload.get("candidates"))
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);
    let k = payload.get("k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    let timeout = payload
        .get("timeout")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.9);

    let verdict = route_choice(task, options, k, timeout);
    println!("{}", crate::jsonfmt::dumps_raw(&verdict.to_json()));
    0
}

pub fn verify_completion_main(args: &[String]) -> i32 {
    let mut objective = String::new();
    let mut evidence = String::new();
    let mut task_class = "code_change".to_string();
    let mut gate = "enforce".to_string();
    let mut timeout = 5.0;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--objective" if i + 1 < args.len() => {
                objective = args[i + 1].clone();
                i += 2;
            }
            "--evidence" if i + 1 < args.len() => {
                evidence = args[i + 1].clone();
                i += 2;
            }
            "--task-class" if i + 1 < args.len() => {
                task_class = args[i + 1].clone();
                i += 2;
            }
            "--gate" if i + 1 < args.len() => {
                gate = args[i + 1].clone();
                i += 2;
            }
            "--timeout" if i + 1 < args.len() => {
                if let Ok(t) = args[i + 1].parse::<f64>() {
                    timeout = t;
                }
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    if objective.is_empty() {
        let payload = read_stdin_payload();
        if let Some(obj) = payload.get("objective").and_then(|v| v.as_str()) {
            objective = obj.to_string();
        }
        if let Some(ev) = payload.get("evidence").and_then(|v| v.as_str()) {
            evidence = ev.to_string();
        }
        if let Some(tc) = payload.get("task_class").and_then(|v| v.as_str()) {
            task_class = tc.to_string();
        }
        if let Some(g) = payload.get("gate").and_then(|v| v.as_str()) {
            gate = g.to_string();
        }
        if let Some(t) = payload.get("timeout").and_then(|v| v.as_f64()) {
            timeout = t;
        }
    }

    let verdict = evaluate_completion(&objective, &evidence, &task_class, timeout);
    match verdict {
        None => {
            let out = json!({
                "status": "fail_open_pass",
                "degraded": true,
                "decision": "allow",
                "verified": true,
                "choice": Value::Null,
                "score": Value::Null,
                "band": "pass",
                "confidence": Value::Null,
                "reason": "System-1 decision tier is offline or disabled; failing open."
            });
            println!("{}", crate::jsonfmt::dumps_raw(&out));
            0
        }
        Some(v) => {
            let tau = surface_tau("completion_check");
            let score_ok = v.score.map(|s| s >= 4.0).unwrap_or(true);
            let conf_ok = v.confidence >= tau;
            let is_complete = v.choice == "complete" && v.band == "pass" && score_ok && conf_ok;
            let is_false_pass = !is_complete
                || v.choice == "incomplete"
                || v.band == "flag"
                || v.score.map(|s| s < 4.0).unwrap_or(false);

            let (decision, verified, reason, exit_code) = if is_false_pass {
                let dec = if gate == "enforce" { "deny" } else { "allow" };
                let r = format!(
                    "System-1 flagged completion as incomplete (confidence {:.2}, score {:?}/5)",
                    v.confidence,
                    v.score.unwrap_or(0.0)
                );
                let code = if gate == "enforce" { 2 } else { 0 };
                (dec, false, r, code)
            } else {
                let r = format!(
                    "System-1 verified completion (confidence {:.2}, score {:?}/5)",
                    v.confidence,
                    v.score.unwrap_or(0.0)
                );
                ("allow", true, r, 0)
            };

            let out = json!({
                "status": "evaluated",
                "degraded": false,
                "decision": decision,
                "verified": verified,
                "choice": v.choice,
                "score": v.score,
                "band": v.band,
                "confidence": v.confidence,
                "reason": reason
            });
            println!("{}", crate::jsonfmt::dumps_raw(&out));
            exit_code
        }
    }
}

fn read_stdin_payload() -> Value {
    let mut buf = String::new();
    let _ = std::io::stdin().read_to_string(&mut buf);
    serde_json::from_str(&buf).unwrap_or_else(|_| json!({}))
}

fn ask_main(surface_override: Option<&str>) -> i32 {
    let marker = std::env::var("CIEL_SYSTEM1_MARKER").ok();
    let mut payload = read_stdin_payload();
    if payload.is_object() && !payload.as_object().unwrap().is_empty() {
        if let Some(surf) = surface_override {
            payload["surface"] = json!(surf);
        }
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

fn decide_main(surface_override: Option<&str>) -> i32 {
    let mut payload = read_stdin_payload();
    if !payload.is_object() || payload.as_object().unwrap().is_empty() {
        println!("null");
        return 0;
    }
    if let Some(surf) = surface_override {
        payload["surface"] = json!(surf);
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

/// `ciel system1 [--surface <name>] --ask | --decide`.
pub fn main_(args: &[String]) -> i32 {
    let mut surface_override = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--surface" && i + 1 < args.len() {
            surface_override = Some(args[i + 1].as_str());
            i += 2;
            continue;
        }
        i += 1;
    }

    if args.iter().any(|a| a == "--ask") {
        return ask_main(surface_override);
    }
    if args.iter().any(|a| a == "--decide") {
        return decide_main(surface_override);
    }
    eprintln!(
        "usage: ciel system1 [--surface <name>] --ask | --decide  (reads JSON payload on stdin; \
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

    #[test]
    fn header_split_crlf_and_lf() {
        let crlf = b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello";
        let (hend, bstart) = find_header_split(crlf).expect("crlf split");
        assert_eq!(&crlf[..hend], b"HTTP/1.1 200 OK\r\nContent-Length: 5");
        assert_eq!(&crlf[bstart..], b"hello");

        let lf = b"HTTP/1.1 200 OK\nContent-Length: 5\n\nhello";
        let (hend2, bstart2) = find_header_split(lf).expect("lf split");
        assert_eq!(&lf[..hend2], b"HTTP/1.1 200 OK\nContent-Length: 5");
        assert_eq!(&lf[bstart2..], b"hello");

        let malformed = b"HTTP/1.1 200 OK incomplete";
        assert!(find_header_split(malformed).is_none());
    }

    #[test]
    fn dechunk_standard_and_extensions() {
        let standard = "5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";
        assert_eq!("hello world", dechunk(standard));

        let with_ext = "5;ext=val\r\nhello\r\n6;another=1\r\n world\r\n0\r\n\r\n";
        assert_eq!("hello world", dechunk(with_ext));

        let lf_only = "5\nhello\n6\n world\n0\n\n";
        assert_eq!("hello world", dechunk(lf_only));

        let zero_only = "0\r\n\r\n";
        assert_eq!("", dechunk(zero_only));

        let malformed = "xyz\r\ncorrupt";
        assert_eq!("", dechunk(malformed));
    }

    #[test]
    fn env_variable_handling() {
        // Test default url
        assert!(url().starts_with("http://"));

        // Mode defaults to active if unset
        assert_eq!(mode(), "active");

        unsafe {
            // URL override
            std::env::set_var("CIEL_SYSTEM1_URL", "http://custom-host:9999/");
            assert_eq!(url(), "http://custom-host:9999");
            std::env::remove_var("CIEL_SYSTEM1_URL");

            // Key override
            std::env::set_var("CIEL_SYSTEM1_KEY", "test-secret-key-123");
            assert_eq!(key(), "test-secret-key-123");
            std::env::remove_var("CIEL_SYSTEM1_KEY");

            // Model override
            std::env::set_var("CIEL_SYSTEM1_MODEL", "typed-decisions");
            assert_eq!(model(), "typed-decisions");
            std::env::remove_var("CIEL_SYSTEM1_MODEL");

            // Mode override (shadow, off, active)
            std::env::set_var("CIEL_SYSTEM1_MODE", "shadow");
            assert_eq!(mode(), "shadow");
            std::env::set_var("CIEL_SYSTEM1_MODE", "off");
            assert_eq!(mode(), "off");
            assert!(disabled());
            std::env::remove_var("CIEL_SYSTEM1_MODE");

            // Disabled flag
            std::env::set_var("CIEL_SYSTEM1_DISABLED", "1");
            assert!(disabled());
            std::env::remove_var("CIEL_SYSTEM1_DISABLED");
        }
    }

    #[test]
    fn completion_check_questions_and_fail_open() {
        let q = completion_questions();
        assert!(q.get("done").is_some());
        let criteria = q["done"].get("criteria").unwrap();
        assert!(criteria.get("complete").is_some());
        assert!(criteria.get("incomplete").is_some());

        // When disabled, evaluate_completion returns None (fail-open)
        unsafe {
            std::env::set_var("CIEL_SYSTEM1_DISABLED", "1");
        }
        let res = evaluate_completion("fix bug", "test passed", "code_change", 0.1);
        assert!(res.is_none());
        unsafe {
            std::env::remove_var("CIEL_SYSTEM1_DISABLED");
        }
    }

    #[test]
    fn http_post_mock_tcp_roundtrip() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"status\":\"ok\"}";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
            let _ = stream.shutdown(std::net::Shutdown::Write);
            let _ = stream.read(&mut buf);
        });

        let url = format!("http://127.0.0.1:{port}/v1/test");
        let resp = http_post(&url, b"{}", "", Duration::from_secs(2));
        handle.join().unwrap();
        assert_eq!(resp, Some("{\"status\":\"ok\"}".to_string()));
    }

    #[test]
    fn http_post_mock_chunked_roundtrip() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n9\r\n{\"status\"\r\n6\r\n:\"ok\"}\r\n0\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
            let _ = stream.shutdown(std::net::Shutdown::Write);
            let _ = stream.read(&mut buf);
        });

        let url = format!("http://127.0.0.1:{port}/v1/chunked");
        let resp = http_post(&url, b"{}", "", Duration::from_secs(2));
        handle.join().unwrap();
        assert_eq!(resp, Some("{\"status\":\"ok\"}".to_string()));
    }

    #[test]
    fn http_post_mock_500_error_returns_none() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 5\r\nConnection: close\r\n\r\nerror";
            let _ = stream.write_all(resp.as_bytes());
        });

        let url = format!("http://127.0.0.1:{port}/v1/err");
        let resp = http_post(&url, b"{}", "", Duration::from_secs(2));
        handle.join().unwrap();
        assert!(resp.is_none());
    }

    #[test]
    fn http_post_mock_truncated_content_length_returns_none() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // Declares 50 bytes but only sends 5 before closing connection
            let resp = "HTTP/1.1 200 OK\r\nContent-Length: 50\r\nConnection: close\r\n\r\nshort";
            let _ = stream.write_all(resp.as_bytes());
        });

        let url = format!("http://127.0.0.1:{port}/v1/truncated");
        let resp = http_post(&url, b"{}", "", Duration::from_secs(2));
        handle.join().unwrap();
        assert!(resp.is_none());
    }

    #[test]
    fn http_post_mock_truncated_chunked_returns_none() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // Missing terminal zero chunk "0\r\n\r\n"
            let resp = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n5\r\nhello\r\n";
            let _ = stream.write_all(resp.as_bytes());
        });

        let url = format!("http://127.0.0.1:{port}/v1/bad-chunk");
        let resp = http_post(&url, b"{}", "", Duration::from_secs(2));
        handle.join().unwrap();
        assert!(resp.is_none());
    }

    #[test]
    fn http_post_mock_malformed_header_returns_none() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // Non-HTTP garbage
            let resp = "NOT-HTTP GARBAGE DATA WITH NO DELIMITER";
            let _ = stream.write_all(resp.as_bytes());
        });

        let url = format!("http://127.0.0.1:{port}/v1/garbage");
        let resp = http_post(&url, b"{}", "", Duration::from_secs(2));
        handle.join().unwrap();
        assert!(resp.is_none());
    }

    #[test]
    fn http_post_closed_port_returns_none() {
        // Pick an unbound port by binding and immediately dropping listener
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        };
        let url = format!("http://127.0.0.1:{port}/v1/unbound");
        let resp = http_post(&url, b"{}", "", Duration::from_millis(100));
        assert!(resp.is_none());
    }

    #[test]
    fn curl_post_invalid_https_returns_none() {
        // Tests curl HTTPS fallback on an unreachable/invalid endpoint
        let resp = curl_post(
            "https://127.0.0.1:49999/v1/systemone",
            b"{}",
            "",
            Duration::from_millis(200),
        );
        assert!(resp.is_none());
    }

    #[test]
    fn evaluate_risk_offline_latency_under_5ms() {
        std::env::set_var("CIEL_SYSTEM1_URL", "http://127.0.0.1:54321");
        let mut min_elapsed = Duration::from_secs(10);
        let mut res = None;
        for _ in 0..5 {
            let t0 = Instant::now();
            res = evaluate_risk(
                "antigravity",
                "2026-09-30T00:00:00Z",
                "exec",
                "ls -la",
                "",
                "allow",
                &serde_json::Value::Null,
                0.05,
            );
            let elapsed = t0.elapsed();
            if elapsed < min_elapsed {
                min_elapsed = elapsed;
            }
        }
        std::env::remove_var("CIEL_SYSTEM1_URL");
        println!("evaluate_risk offline min latency: {:?}", min_elapsed);
        assert!(res.is_none());
        assert!(
            min_elapsed < Duration::from_millis(5),
            "evaluate_risk offline took {:?}",
            min_elapsed
        );
    }

    #[test]
    fn surface_tau_lattice_and_overrides() {
        // Defaults from lattice
        assert!((surface_tau("pre_tool_risk") - 0.65).abs() < 1e-4);
        assert!((surface_tau("router") - 0.82).abs() < 1e-4);
        assert!((surface_tau("completion_check") - 0.75).abs() < 1e-4);
        assert!((surface_tau("council_prescreen") - 0.70).abs() < 1e-4);
        assert!((surface_tau("unknown_surface") - 0.20).abs() < 1e-4);

        // Global override
        unsafe {
            std::env::set_var("CIEL_SYSTEM1_TAU", "0.42");
        }
        assert!((surface_tau("pre_tool_risk") - 0.42).abs() < 1e-4);
        assert!((surface_tau("router") - 0.42).abs() < 1e-4);

        // Surface-specific override takes precedence
        unsafe {
            std::env::set_var("CIEL_SYSTEM1_TAU_PRE_TOOL_RISK", "0.99");
        }
        assert!((surface_tau("pre_tool_risk") - 0.99).abs() < 1e-4);
        assert!((surface_tau("router") - 0.42).abs() < 1e-4);

        unsafe {
            std::env::remove_var("CIEL_SYSTEM1_TAU_PRE_TOOL_RISK");
            std::env::remove_var("CIEL_SYSTEM1_TAU");
        }
    }

    #[test]
    fn route_choice_fail_open_offline() {
        let mut options = serde_json::Map::new();
        options.insert("git".into(), json!("git operations"));
        options.insert("docker".into(), json!("container operations"));
        let verdict = route_choice("commit my changes", &options, 10, 0.05);
        assert_eq!(verdict.status, "fail_open");
        assert!(verdict.degraded);
        assert_eq!(verdict.shortlist, vec!["docker", "git"]);
    }
}
