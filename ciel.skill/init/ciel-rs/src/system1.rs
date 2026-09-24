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

use serde_json::{json, Map, Value};
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
    loop {
        let Some((size_line, after)) = rest.split_once("\r\n") else {
            break;
        };
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

/// One `choice` question — mirror of `ask_choice`.
#[allow(dead_code)]
pub fn ask_choice(
    state: &Value,
    key: &str,
    instructions: &str,
    options: &Value,
    timeout_s: f64,
) -> Option<Value> {
    let result = ask(
        state,
        &json!({key: {"type": "choice", "instructions": instructions,
                      "criteria": options}}),
        timeout_s,
    )?;
    let answer = result["answers"].get(key)?;
    if !answer.is_object() {
        return None;
    }
    Some(json!({
        "choice": answer.get("choice").cloned().unwrap_or(Value::Null),
        "confidence": answer.get("confidence").cloned().unwrap_or(Value::Null),
        "probabilities": answer.get("probabilities").cloned().unwrap_or(Value::Null),
        "model": result.get("model").cloned().unwrap_or(Value::Null),
    }))
}

// ------------------------------------------------------ shortlist (lexical)

#[allow(dead_code)]
fn stopwords() -> &'static std::collections::HashSet<&'static str> {
    use std::sync::OnceLock;
    static SW: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    SW.get_or_init(|| {
        "in my and the a an to for of on is it me we i or be this that with out \
         up do how what which should can could would your our at by from as \
         into about before after just need want help please use using make get \
         set new all any some no not if when then so than too very will are was \
         were been has have had does did over again once here there where why \
         who these those each few more most other own same only also now like \
         through between both per via whether while during without within \
         across upon off down along around among against step run write create \
         add check see look take give go put let keep work thing things \
         something anything lot kind type part way"
            .split_whitespace()
            .collect()
    })
}

#[allow(dead_code)]
fn alnum_tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            cur.push(c);
        } else if !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[allow(dead_code)]
fn tokens(text: &str) -> std::collections::HashSet<String> {
    let sw = stopwords();
    let mut out = std::collections::HashSet::new();
    for tok in alnum_tokens(text) {
        if sw.contains(tok.as_str()) {
            continue;
        }
        out.insert(tok.clone());
        if tok.len() > 3 && tok.ends_with('s') && !tok.ends_with("ss") {
            out.insert(tok[..tok.len() - 1].to_string());
        }
    }
    out
}

/// IDF-weighted token overlap — mirror of `_lexical_rank`.
#[allow(dead_code)]
fn lexical_rank(text: &str, options: &Map<String, Value>) -> Vec<(f64, String)> {
    let query = tokens(text);
    let docs: Vec<(String, std::collections::HashSet<String>)> = options
        .iter()
        .map(|(name, crit)| {
            (
                name.clone(),
                tokens(&format!("{name} {}", crit.as_str().unwrap_or(""))),
            )
        })
        .collect();
    let mut df: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (_, toks) in &docs {
        for tok in toks {
            *df.entry(tok.clone()).or_insert(0) += 1;
        }
    }
    let n = docs.len().max(1) as f64;
    let mut ranked: Vec<(f64, String)> = docs
        .into_iter()
        .map(|(name, toks)| {
            let score: f64 = query
                .intersection(&toks)
                .map(|t| (n / (1 + df[t.as_str()]) as f64).ln() + 1.0)
                .sum();
            (score, name)
        })
        .collect();
    ranked.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
    });
    ranked
}

/// Bi-encoder shortlist via the laya-venv helper subprocess —
/// `system1_embed.py` stays Python (sentence-transformers). Same spawn
/// contract as the Python caller.
#[allow(dead_code)]
fn semantic_rank(text: &str, options: &Value, k: usize) -> Vec<String> {
    if std::env::var("CIEL_SYSTEM1_EMBED").ok().as_deref() == Some("0") {
        return Vec::new();
    }
    let venv_py = paths::ciel_home()
        .join("system1")
        .join("venv")
        .join("bin")
        .join("python");
    let lib = std::env::var_os("CIEL_HOOK_LIB")
        .map(PathBuf::from)
        .unwrap_or_else(|| paths::ciel_home().join("hooks").join("lib"));
    let helper = lib.join("system1_embed.py");
    if !(venv_py.is_file() && helper.is_file()) {
        return Vec::new();
    }
    let payload = json!({"task": text, "candidates": options, "k": k});
    let Ok(mut child) = Command::new(venv_py)
        .arg(helper)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    else {
        return Vec::new();
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(payload.to_string().as_bytes());
    }
    // mirror subprocess.run(timeout=60): kill on expiry, [] on any error
    use wait_timeout::ChildExt;
    let out = match child.wait_timeout(Duration::from_secs(60)) {
        Ok(Some(_)) => {
            let mut buf = Vec::new();
            match child.stdout.take() {
                Some(mut so) => {
                    let _ = so.read_to_end(&mut buf);
                    buf
                }
                None => return Vec::new(),
            }
        }
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            return Vec::new();
        }
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&out);
    serde_json::from_str::<Value>(&text)
        .ok()
        .and_then(|v| v.get("names").cloned())
        .and_then(|n| n.as_array().cloned())
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Coarse-to-fine candidate reduction — mirror of `shortlist_options`.
#[allow(dead_code)]
pub fn shortlist_options(text: &str, options: &Value, k: usize) -> Value {
    let obj = options.as_object().cloned().unwrap_or_default();
    if obj.len() <= k {
        return Value::Object(obj);
    }
    let mut keep: Vec<String> = Vec::new();
    for name in semantic_rank(text, options, k) {
        if obj.contains_key(&name) && !keep.contains(&name) {
            keep.push(name);
        }
    }
    for (_, name) in lexical_rank(text, &obj).into_iter().take(5) {
        if !keep.contains(&name) {
            keep.push(name);
        }
    }
    let task_tokens: std::collections::HashSet<String> = alnum_tokens(text).into_iter().collect();
    for name in obj.keys() {
        if !keep.contains(name) {
            let name_tokens: std::collections::HashSet<String> =
                alnum_tokens(name).into_iter().collect();
            if !name_tokens.is_disjoint(&task_tokens) {
                keep.push(name.clone());
            }
        }
    }
    let mut out = Map::new();
    for name in keep.into_iter().take(k) {
        if let Some(v) = obj.get(&name) {
            out.insert(name, v.clone());
        }
    }
    Value::Object(out)
}

/// Mirror of `route_choice`.
#[allow(dead_code)]
pub fn route_choice(task: &str, options: &Value, k: usize, timeout_s: f64) -> Option<Value> {
    let obj = options.as_object().cloned().unwrap_or_default();
    let candidates = if obj.len() <= k {
        Value::Object(obj)
    } else {
        shortlist_options(task, options, k)
    };
    let mut sorted: Vec<String> = candidates
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    sorted.sort();
    ask_choice(
        &json!({"task": task, "candidates": sorted}),
        "route",
        "Which candidate best fits the task?",
        &candidates,
        timeout_s,
    )
}

// -------------------------------------------------------- cache + event log

/// `json.dumps(obj, sort_keys=True)` — canonical form for the cache digest:
/// recursively sorted keys, ", "/": " separators, ensure_ascii escaping
/// (including surrogate pairs above BMP). Must match Python byte-exactly so
/// both engines share one cache namespace.
fn py_escape(s: &str, ascii_only: bool) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if !ascii_only || (c as u32) < 0x7f => out.push(c),
            c => {
                let n = c as u32;
                if n > 0xffff {
                    let v = n - 0x10000;
                    out.push_str(&format!(
                        "\\u{:04x}\\u{:04x}",
                        0xd800 + (v >> 10),
                        0xdc00 + (v & 0x3ff)
                    ));
                } else {
                    out.push_str(&format!("\\u{:04x}", n));
                }
            }
        }
    }
    out
}

/// `json.dumps` with Python's default `(', ', ': ')` separators.
/// `sorted` → sort_keys=True (cache digest); `ascii` → ensure_ascii=True.
fn py_dumps(v: &Value, sorted: bool, ascii: bool) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", py_escape(s, ascii)),
        Value::Array(a) => {
            let items: Vec<String> = a.iter().map(|x| py_dumps(x, sorted, ascii)).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(m) => {
            let keys: Vec<&String> = if sorted {
                let mut k: Vec<&String> = m.keys().collect();
                k.sort();
                k
            } else {
                m.keys().collect()
            };
            let items: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    format!(
                        "\"{}\": {}",
                        py_escape(k, ascii),
                        py_dumps(&m[k], sorted, ascii)
                    )
                })
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

fn py_dumps_sorted(v: &Value) -> String {
    py_dumps(v, true, true)
}

fn cache_path(state: &Value, questions: &Value) -> PathBuf {
    let canonical = py_dumps_sorted(&json!({"s": state, "q": questions}));
    let digest = format!("{:x}", Sha256::digest(canonical.as_bytes()));
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
    let _ = std::fs::write(&path, py_dumps(result, false, false));
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
        let _ = writeln!(f, "{}", py_dumps(record, false, false));
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
    let spawned = cmd.spawn().and_then(|mut child| {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(payload.to_string().as_bytes());
        }
        Ok(())
    });
    if spawned.is_err() {
        let _ = std::fs::remove_file(&marker);
    }
}

/// Detached council_prescreen shadow — mirror of `council_prescreen`.
#[allow(dead_code)]
pub fn council_prescreen(subject: &str, meta: &Value) {
    ask_async(&json!({
        "surface": "council_prescreen",
        "state": {"event": subject},
        "questions": {
            "scope": {
                "type": "choice",
                "instructions": "Does this event require full multi-lens deliberation? When in doubt, escalate — an unnecessary review costs little; a skipped review of a sensitive change is dangerous.",
                "criteria": {
                    "routine": "only clearly low-risk, reversible, well-precedented actions",
                    "escalate": "anything irreversible, security-relevant, self-modifying, trust-changing, or novel-scope — including when uncertain",
                },
            }
        },
        "meta": meta,
    }));
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
            .map(|r| py_dumps(&r, false, false))
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
            py_dumps_sorted(&v)
        );
        // default separators, insertion order, raw unicode
        let v2 = json!({"b": "héllo", "a": 1});
        assert_eq!(r#"{"b": "héllo", "a": 1}"#, py_dumps(&v2, false, false));
    }

    #[test]
    fn tokens_plural_strip() {
        let t = tokens("run the tests on files");
        assert!(t.contains("tests"));
        assert!(t.contains("test"));
        assert!(t.contains("files"));
        assert!(t.contains("file"));
        assert!(!t.contains("the"));
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
