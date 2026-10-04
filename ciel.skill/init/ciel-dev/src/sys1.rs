//! Minimal System-1 decision client — port of the `system1.py` surface the
//! dev harnesses use: `ask`, `_resolve` (response cache), `_append_event`,
//! `_band`, `surface_tau`, `tool_state`, `shortlist_options`, and
//! `completion_check`. Fail-open throughout: None on any error.
//!
//! `http://` endpoints are served by an in-process HTTP/1.1 client;
//! `https://` goes through `curl` (no vendored TLS stack).

use regex::Regex;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::{jsonfmt, py};

const EVENTS_LOG_MAX: u64 = 4 * 1024 * 1024;
const DEFAULT_TAU: f64 = 0.05;

// ------------------------------------------------------------ constants

/// `risk_policy.SYSTEM1_QUESTIONS`.
pub fn risk_questions() -> Value {
    json!({
        "risk": {
            "type": "choice",
            "instructions": "Is this tool call dangerous? Judge BOTH the command AND the file path it targets — a write to a sensitive path is dangerous even with no command.",
            "criteria": {
                "safe": "routine, reversible, or read-only operation on non-sensitive paths",
                "dangerous": "destructive, irreversible, privilege-escalating, credential/sensitive-path writing, or data-exfiltrating operation",
            }
        }
    })
}

/// `system1.PRESCREEN_QUESTIONS`.
pub fn prescreen_questions() -> Value {
    json!({
        "scope": {
            "type": "choice",
            "instructions": "Does this event require full multi-lens deliberation? When in doubt, escalate — an unnecessary review costs little; a skipped review of a sensitive change is dangerous.",
            "criteria": {
                "routine": "only clearly low-risk, reversible, well-precedented actions",
                "escalate": "anything irreversible, security-relevant, self-modifying, trust-changing, or novel-scope — including when uncertain",
            }
        }
    })
}

/// `system1.COMPLETION_QUESTIONS` (+ `COMPLETION_SCORE_QUESTIONS` when
/// `with_score`).
pub fn completion_questions(with_score: bool) -> Value {
    let mut q = json!({
        "done": {
            "type": "choice",
            "instructions": "Evaluate whether the objective is verifiably satisfied by the provided empirical evidence. When in doubt, mark incomplete if claims lack empirical verification artifacts (tests, execution logs, diffs, live probes).",
            "criteria": {
                "complete": "objective is fully satisfied with direct empirical proof and verification artifacts",
                "incomplete": "objective is unverified, missing required artifacts, failed verification, or asserts claims without evidence",
            }
        }
    });
    if with_score {
        q["evidence_score"] = json!({
            "type": "score",
            "instructions": "Rate how well empirical evidence substantiates the completion claim on the ordered rubric (lowest level = unverified/pure claim, highest level = complete empirical proof).",
            "criteria": [
                "no evidence or contradictory evidence (pure assertion/hallucination)",
                "partial evidence with major unverified claims or failing tests",
                "indirect or ambiguous evidence without target-state verification",
                "direct empirical evidence verifying primary claims",
                "exhaustive empirical verification of all claims and task-class artifacts",
            ]
        });
    }
    q
}

// --------------------------------------------------------- env plumbing

fn env_file_pairs() -> &'static Vec<(String, String)> {
    static PAIRS: OnceLock<Vec<(String, String)>> = OnceLock::new();
    PAIRS.get_or_init(|| {
        let env_file = py::ciel_home().join("system1").join("env");
        std::fs::read_to_string(&env_file)
            .map(|text| {
                text.lines()
                    .filter_map(|line| {
                        let t = line.trim();
                        if t.starts_with('#') || !t.contains('=') {
                            return None;
                        }
                        let (k, v) = t.split_once('=')?;
                        Some((
                            k.trim().to_string(),
                            v.trim().trim_matches('"').trim_matches('\'').to_string(),
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

fn env_file_value(names: &[&str]) -> String {
    for name in names {
        for (k, v) in env_file_pairs() {
            if k == name {
                return v.clone();
            }
        }
    }
    String::new()
}

/// `system1._mode()` — env var, then env file, default "active".
fn mode() -> String {
    let v = std::env::var("CIEL_SYSTEM1_MODE")
        .unwrap_or_default()
        .trim()
        .to_string();
    if !v.is_empty() {
        return v;
    }
    let v = env_file_value(&["CIEL_SYSTEM1_MODE"]);
    if !v.is_empty() {
        return v;
    }
    "active".into()
}

pub fn disabled() -> bool {
    std::env::var_os("CIEL_SYSTEM1_DISABLED").is_some() || mode() == "off"
}

fn key() -> String {
    if let Ok(k) = std::env::var("CIEL_SYSTEM1_KEY") {
        let k = k.trim();
        if !k.is_empty() {
            return k.to_string();
        }
    }
    if hosted() {
        // Hosted never receives the local laya credential.
        return env_file_value(&["CIEL_SYSTEM1_KEY"]).pipe(|s| {
            if s.is_empty() {
                env_file_value(&["JEV_API_KEY"])
            } else {
                s
            }
        });
    }
    env_file_value(&["CIEL_SYSTEM1_KEY", "LAYA_API_KEY"])
}

/// `system1._url()` — env var only, default loopback laya-serve.
pub fn url() -> String {
    std::env::var("CIEL_SYSTEM1_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8765".into())
        .trim_end_matches('/')
        .to_string()
}

pub fn model() -> String {
    let m = std::env::var("CIEL_SYSTEM1_MODEL")
        .unwrap_or_default()
        .trim()
        .to_string();
    if !m.is_empty() {
        return m;
    }
    env_file_value(&["CIEL_SYSTEM1_MODEL"])
}

const HOSTED_API_HOSTS: [&str; 2] = ["jev-agent.com", "www.jev-agent.com"];
const HOSTED_MODEL_HOSTS: [&str; 5] = [
    "jev-agent.com",
    "www.jev-agent.com",
    "api.typesafe.ai",
    "typesafe.ai",
    "www.typesafe.ai",
];
const DEFAULT_HOSTED_MODEL: &str = "jev-latest";
const HOSTED_URL_DEFAULT: &str = "https://api.typesafe.ai";
const HOSTED_TRIP_AUTH_S: f64 = 300.0;
const HOSTED_TRIP_RATE_S: f64 = 60.0;
const HOSTED_TRIP_ERR_S: f64 = 30.0;

pub fn host_of(base: &str) -> String {
    let rest = base.split("://").nth(1).unwrap_or(base);
    let authority = rest.split(['/', '\\', '?', '#']).next().unwrap_or("");
    authority
        .rsplit('@')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn host() -> String {
    host_of(&url())
}

fn endpoint_of(base: &str) -> String {
    if base.contains("/systemone") {
        return base.to_string();
    }
    let rest = base.split("://").nth(1).unwrap_or(base);
    let path = rest.split_once('/').map(|x| x.1).unwrap_or("");
    if HOSTED_API_HOSTS.contains(&host_of(base).as_str()) && path.is_empty() {
        return format!("{base}/api/v1/systemone");
    }
    format!("{base}/v1/systemone")
}

fn endpoint_egress_ok(base: &str) -> bool {
    let h = host_of(base);
    if matches!(h.as_str(), "127.0.0.1" | "localhost" | "::1" | "[::1]") {
        return true;
    }
    if HOSTED_MODEL_HOSTS.contains(&h.as_str()) {
        // Cleartext Bearer to a hosted endpoint is a key leak.
        return base
            .split("://")
            .next()
            .unwrap_or("")
            .eq_ignore_ascii_case("https");
    }
    // secret_scan is statically available in this port.
    true
}

fn remote_of(base: &str) -> bool {
    !matches!(
        host_of(base).as_str(),
        "127.0.0.1" | "localhost" | "::1" | "[::1]"
    )
}

fn remote() -> bool {
    remote_of(&url())
}

fn hosted() -> bool {
    HOSTED_MODEL_HOSTS.contains(&host().as_str())
}

fn hosted_url() -> String {
    let u = std::env::var("CIEL_SYSTEM1_HOSTED_URL")
        .unwrap_or_default()
        .trim()
        .to_string();
    if !u.is_empty() {
        return u.trim_end_matches('/').to_string();
    }
    let u = env_file_value(&["CIEL_SYSTEM1_HOSTED_URL"]);
    if !u.is_empty() {
        return u.trim_end_matches('/').to_string();
    }
    HOSTED_URL_DEFAULT.into()
}

fn hosted_key() -> String {
    for name in ["CIEL_SYSTEM1_HOSTED_KEY", "JEV_API_KEY", "CIEL_SYSTEM1_KEY"] {
        if let Ok(k) = std::env::var(name) {
            let k = k.trim();
            if !k.is_empty() {
                return k.to_string();
            }
        }
    }
    env_file_value(&["CIEL_SYSTEM1_HOSTED_KEY", "JEV_API_KEY", "CIEL_SYSTEM1_KEY"])
}

fn hosted_enabled() -> bool {
    let v = std::env::var("CIEL_SYSTEM1_HOSTED")
        .unwrap_or_default()
        .trim()
        .to_string();
    let v = if v.is_empty() {
        env_file_value(&["CIEL_SYSTEM1_HOSTED"])
    } else {
        v
    };
    !matches!(
        v.to_ascii_lowercase().as_str(),
        "off" | "0" | "false" | "no"
    )
}

fn hosted_state_path() -> PathBuf {
    py::ciel_home().join("system1").join("hosted_state.json")
}

fn hosted_down() -> bool {
    let Ok(text) = std::fs::read_to_string(hosted_state_path()) else {
        return false;
    };
    let Ok(d) = serde_json::from_str::<Value>(&text) else {
        return false;
    };
    let until = d.get("down_until").and_then(|v| v.as_f64()).unwrap_or(0.0);
    py::time_f64() < until
}

fn hosted_trip(status: Option<u16>) {
    let backoff = match status {
        Some(401) | Some(402) | Some(403) => HOSTED_TRIP_AUTH_S,
        Some(429) => HOSTED_TRIP_RATE_S,
        None => HOSTED_TRIP_ERR_S,
        Some(s) if s >= 500 => HOSTED_TRIP_ERR_S,
        _ => return,
    };
    let rec = json!({"down_until": py::time_f64() + backoff, "status": status});
    let path = hosted_state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, rec.to_string());
}

fn hosted_active() -> bool {
    let u = hosted_url();
    let scheme_ok = u
        .split("://")
        .next()
        .unwrap_or("")
        .eq_ignore_ascii_case("https")
        || matches!(
            host_of(&u).as_str(),
            "127.0.0.1" | "localhost" | "::1" | "[::1]"
        );
    hosted_enabled() && !hosted_key().is_empty() && scheme_ok && !hosted_down()
}

fn hosted_model() -> String {
    let m = std::env::var("CIEL_SYSTEM1_HOSTED_MODEL")
        .unwrap_or_default()
        .trim()
        .to_string();
    if !m.is_empty() {
        return m;
    }
    let m = env_file_value(&["CIEL_SYSTEM1_HOSTED_MODEL"]);
    if !m.is_empty() {
        return m;
    }
    DEFAULT_HOSTED_MODEL.into()
}

fn local_base() -> String {
    if hosted() {
        "http://127.0.0.1:8765".into()
    } else {
        url()
    }
}

// tiny pipe helper
trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl Pipe for String {}

// --------------------------------------------------------------- redact

fn redactors() -> &'static Vec<(String, Regex)> {
    static REDACTORS: OnceLock<Vec<(String, Regex)>> = OnceLock::new();
    REDACTORS.get_or_init(|| {
        let raw: Vec<(&str, &str)> = vec![
            ("github_token", r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b|\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
            ("aws_access_key", r"\bAKIA[0-9A-Z]{16}\b"),
            ("api_key_prefixed", r"\b(?:sk|pk|key|api|tok)_[A-Za-z0-9_-]{20,}\b|\bsk-[A-Za-z0-9_-]{20,}\b"),
            ("private_key_block", r"-----BEGIN [A-Z ]*PRIVATE KEY"),
            ("slack_token", r"\bxox[baprs]-[0-9A-Za-z-]{10,}\b"),
            ("gcp_api_key", r"\bAIza[0-9A-Za-z_-]{35}\b"),
            ("jwt", r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b"),
            ("npm_token", r"\bnpm_[A-Za-z0-9]{36}\b"),
            ("crates_token", r"\bcio[0-9A-Za-z]{25,}\b"),
            ("password_assignment", r#"(?i)\b(?:sudo\s+)?(?:password|passwd|passphrase)\s*(?:is|:|=)\s*['"]?[^\s'"]{4,}"#),
            ("secret_assignment", r#"(?i)\b(?:api[_-]?key|access[_-]?token|auth[_-]?token|secret[_-]?key|client[_-]?secret)\s*[:=]\s*['"]?[A-Za-z0-9_\-]{8,}"#),
            ("generic_secret_kv", r#"(?i)\b(?:token|secret)\s+is\s+['"]?[A-Za-z0-9_\-]{8,}"#),
        ];
        raw.into_iter()
            .filter_map(|(n, p)| Regex::new(p).ok().map(|r| (n.to_string(), r)))
            .collect()
    })
}

fn redact_string(s: &str) -> String {
    let mut out = s.to_string();
    for (cat, re) in redactors() {
        out = re
            .replace_all(&out, format!("[REDACTED:{cat}]").as_str())
            .into_owned();
    }
    out
}

fn redact_value(val: &mut Value) {
    match val {
        Value::String(s) => *s = redact_string(s),
        Value::Array(a) => a.iter_mut().for_each(redact_value),
        Value::Object(o) => o.values_mut().for_each(redact_value),
        _ => {}
    }
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

/// Minimal HTTP/1.1 POST for `http://` endpoints; `https://` via curl.
/// Returns (http_status, body); status None on transport failure.
fn http_post_sc(
    url: &str,
    body: &[u8],
    auth: &str,
    timeout: Duration,
) -> (Option<u16>, Option<String>) {
    if url.starts_with("https://") {
        return curl_post_sc(url, body, auth, timeout);
    }
    let Some(rest) = url.strip_prefix("http://") else {
        return (None, None);
    };
    let (authority, path) = match rest.split_once('/') {
        Some((a, p)) => (a, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    let addr = if authority.contains(':') {
        authority.to_string()
    } else {
        format!("{authority}:80")
    };
    let Ok(addrs) = addr.to_socket_addrs() else {
        return (None, None);
    };
    let connect_timeout =
        if authority.starts_with("127.0.0.1") || authority.starts_with("localhost") {
            Duration::from_millis(10).min(timeout)
        } else {
            Duration::from_secs(1).min(timeout)
        };
    let mut stream = None;
    for sock in addrs {
        if let Ok(s) = TcpStream::connect_timeout(&sock, connect_timeout) {
            stream = Some(s);
            break;
        }
    }
    let Some(mut stream) = stream else {
        return (None, None);
    };
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
    if stream.write_all(req.as_bytes()).is_err() || stream.write_all(body).is_err() {
        return (None, None);
    }

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
        return (None, None);
    }
    let Some((hend, bstart)) = header_body_split.or_else(|| find_header_split(&resp)) else {
        return (None, None);
    };
    let head = String::from_utf8_lossy(&resp[..hend]);
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse::<u16>().ok());
    let chunked = is_chunked
        || head.lines().any(|l| {
            let lower = l.to_ascii_lowercase();
            lower.starts_with("transfer-encoding:") && lower.contains("chunked")
        });
    if chunked {
        let raw = String::from_utf8_lossy(&resp[bstart..]);
        let decoded = dechunk(&raw);
        if decoded.is_empty() && raw.trim() != "0" {
            return (status, None);
        }
        return (status, Some(decoded));
    }
    let body_bytes = if let Some(clen) = content_len {
        if resp.len() < bstart + clen {
            return (status, None);
        }
        &resp[bstart..bstart + clen]
    } else {
        &resp[bstart..]
    };
    (
        status,
        Some(String::from_utf8_lossy(body_bytes).to_string()),
    )
}

fn curl_post_sc(
    url: &str,
    body: &[u8],
    auth: &str,
    timeout: Duration,
) -> (Option<u16>, Option<String>) {
    let connect_timeout = if matches!(
        host_of(url).as_str(),
        "127.0.0.1" | "localhost" | "::1" | "[::1]"
    ) {
        "0.05".to_string()
    } else {
        format!("{:.3}", timeout.as_secs_f64().min(3.0))
    };
    let mut cmd = Command::new("curl");
    cmd.args([
        "-sS",
        "--connect-timeout",
        &connect_timeout,
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
        "-w",
        "\n%{http_code}",
        "--data-binary",
        "@-",
        url,
    ]);
    let out = match cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut c| {
            if let Some(mut stdin) = c.stdin.take() {
                let _ = stdin.write_all(body);
            }
            c.wait_with_output()
        }) {
        Ok(o) => o,
        Err(_) => return (None, None),
    };
    if !out.status.success() {
        return (None, None);
    }
    let text = match String::from_utf8(out.stdout) {
        Ok(t) => t,
        Err(_) => return (None, None),
    };
    let Some((body_part, code_part)) = text.rsplit_once('\n') else {
        return (None, Some(text));
    };
    (
        code_part.trim().parse::<u16>().ok(),
        Some(body_part.to_string()),
    )
}

// --------------------------------------------------------------- ask layer

/// One POST to a fully-resolved endpoint — (result, http_status).
fn do_ask(
    endpoint: &str,
    key: &str,
    model: &str,
    state: &Value,
    questions: &Value,
    timeout_s: f64,
    redact: bool,
) -> (Option<Value>, Option<u16>) {
    let mut state_v = state.clone();
    if redact {
        redact_value(&mut state_v);
    }
    let mut body = json!({"state": state_v, "questions": questions});
    if !model.is_empty() {
        body["model"] = json!(model);
    }
    let (status, resp) = http_post_sc(
        endpoint,
        body.to_string().as_bytes(),
        key,
        Duration::from_secs_f64(timeout_s.max(0.05)),
    );
    let Some(resp) = resp else {
        return (None, status);
    };
    let Ok(data) = serde_json::from_str::<Value>(&resp) else {
        return (None, status);
    };
    let Some(answers) = data.get("answers") else {
        return (None, status);
    };
    if !answers.is_object() {
        return (None, status);
    }
    let routing_model = data
        .get("routing")
        .and_then(|r| r.get("model"))
        .cloned()
        .filter(|v| !(v.is_null() || v == &json!("") || v == &json!(false) || v == &json!(0)));
    let model = routing_model.or_else(|| data.get("model").cloned());
    (
        Some(json!({"answers": answers.clone(), "model": model.unwrap_or(Value::Null)})),
        status,
    )
}

/// `system1.ask` — POST state+questions; hosted Jev primary when
/// configured, local laya fallback / primary while the breaker is tripped.
pub fn ask(state: &Value, questions: &Value, timeout_s: f64) -> Option<Value> {
    if disabled() {
        return None;
    }
    let deadline = Instant::now() + Duration::from_secs_f64(timeout_s.max(0.05));
    let mut hosted_base = None;
    if hosted() {
        hosted_base = Some(url());
    } else if !remote() && hosted_active() {
        hosted_base = Some(hosted_url());
    }
    if let Some(base) = hosted_base {
        if !hosted_down() && endpoint_egress_ok(&base) {
            let (result, status) = do_ask(
                &endpoint_of(&base),
                &hosted_key(),
                &hosted_model(),
                state,
                questions,
                (timeout_s * 0.5).min(2.0),
                true,
            );
            if result.is_some() {
                return result;
            }
            hosted_trip(status);
        }
    }
    let local = local_base();
    if !endpoint_egress_ok(&local) {
        return None;
    }
    let remaining = deadline
        .saturating_duration_since(Instant::now())
        .as_secs_f64()
        .max(0.05);
    do_ask(
        &endpoint_of(&local),
        &key(),
        &model(),
        state,
        questions,
        remaining,
        remote_of(&local),
    )
    .0
}

// -------------------------------------------------------- cache + events

fn cache_path(state: &Value, questions: &Value) -> PathBuf {
    let canonical = jsonfmt::dumps_sorted(&json!({"s": state, "q": questions}));
    let digest: String = Sha256::digest(canonical.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    py::ciel_home()
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
    let _ = std::fs::write(&path, jsonfmt::dumps_raw(result));
}

fn append_event(record: &Value) {
    let mut record = record.clone();
    redact_value(&mut record);
    let log = py::ciel_home().join("system1").join("events.jsonl");
    if let Some(parent) = log.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Rotate over the 4 MiB cap before appending.
    if let Ok(meta) = std::fs::metadata(&log) {
        if meta.len() > EVENTS_LOG_MAX {
            let _ = std::fs::rename(&log, log.with_extension("jsonl.1"));
        }
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log)
    {
        let _ = writeln!(f, "{}", jsonfmt::dumps_raw(&record));
    }
}

// ------------------------------------------------------------- band/tau

fn surface_flag(surface: &str) -> &'static [&'static str] {
    match surface {
        "pre_tool_risk" => &["dangerous"],
        "council_prescreen" => &["escalate"],
        "completion_check" => &["incomplete"],
        "context_compaction" => &["compress", "drop_stale", "escalate"],
        "mandate_canary" => &["drifted"],
        _ => &[],
    }
}

fn policy_thresholds() -> HashMap<String, f64> {
    static CACHE: OnceLock<HashMap<String, f64>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let mut map = HashMap::new();
            let repo = py::repo_root();
            for cand in [
                py::ciel_home().join("risk").join("policy.json"),
                repo.join("ciel.skill").join("risk").join("policy.json"),
                py::ciel_home()
                    .join("risk")
                    .join("system1_calibration.json"),
                repo.join("ciel.skill")
                    .join("risk")
                    .join("system1_calibration.json"),
            ] {
                let Ok(text) = std::fs::read_to_string(&cand) else {
                    continue;
                };
                let Ok(val) = serde_json::from_str::<Value>(&text) else {
                    continue;
                };
                let Some(obj) = val
                    .get("system1_thresholds")
                    .or_else(|| val.get("threshold_lattice"))
                    .and_then(|v| v.as_object())
                else {
                    continue;
                };
                for (k, v) in obj {
                    if let Some(num) = v.as_f64() {
                        map.insert(k.clone(), num);
                    }
                }
                return map;
            }
            map
        })
        .clone()
}

pub fn surface_tau(surface: &str) -> f64 {
    let env_surf = format!("CIEL_SYSTEM1_TAU_{}", surface.to_ascii_uppercase());
    if let Ok(v) = std::env::var(&env_surf) {
        if let Ok(f) = v.parse::<f64>() {
            return f;
        }
    }
    if let Ok(v) = std::env::var("CIEL_SYSTEM1_TAU") {
        if let Ok(f) = v.parse::<f64>() {
            return f;
        }
    }
    if let Some(&v) = policy_thresholds().get(surface) {
        return v;
    }
    match surface {
        "pre_tool_risk" => 0.025,
        "router" => 0.47,
        "router_selection" => 0.33,
        "completion_check" => 0.10,
        "council_prescreen" => 0.025,
        "context_select" => 0.05,
        "memory_salience" => 0.05,
        "context_compaction" => 0.02,
        "mandate_canary" => 0.15,
        _ => DEFAULT_TAU,
    }
}

fn band(surface: &str, answers: &Value) -> &'static str {
    let flag_set = surface_flag(surface);
    let tau = surface_tau(surface);
    let mut worst = "pass";
    if let Some(obj) = answers.as_object() {
        for (_qname, answer) in obj {
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
            if surface == "completion_check" {
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

// ------------------------------------------------------------ tool_state

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

/// `system1.tool_state` — deterministic enrichment, verdict-free.
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

// ------------------------------------------------------------ shortlist

fn stopwords() -> &'static std::collections::HashSet<&'static str> {
    static SW: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    SW.get_or_init(|| {
        [
            "in",
            "my",
            "and",
            "the",
            "a",
            "an",
            "to",
            "for",
            "of",
            "on",
            "is",
            "it",
            "me",
            "we",
            "i",
            "or",
            "be",
            "this",
            "that",
            "with",
            "out",
            "up",
            "do",
            "how",
            "what",
            "which",
            "should",
            "can",
            "could",
            "would",
            "your",
            "our",
            "at",
            "by",
            "from",
            "as",
            "into",
            "about",
            "before",
            "after",
            "just",
            "need",
            "want",
            "help",
            "please",
            "use",
            "using",
            "make",
            "get",
            "set",
            "new",
            "all",
            "any",
            "some",
            "no",
            "not",
            "if",
            "when",
            "then",
            "so",
            "than",
            "too",
            "very",
            "will",
            "are",
            "was",
            "were",
            "been",
            "has",
            "have",
            "had",
            "does",
            "did",
            "over",
            "again",
            "once",
            "here",
            "there",
            "where",
            "why",
            "who",
            "these",
            "those",
            "each",
            "few",
            "more",
            "most",
            "other",
            "own",
            "same",
            "only",
            "also",
            "now",
            "like",
            "through",
            "between",
            "both",
            "per",
            "via",
            "whether",
            "while",
            "during",
            "without",
            "within",
            "across",
            "upon",
            "off",
            "down",
            "along",
            "around",
            "among",
            "against",
            "step",
            "run",
            "write",
            "create",
            "add",
            "check",
            "see",
            "look",
            "take",
            "give",
            "go",
            "put",
            "let",
            "keep",
            "work",
            "thing",
            "things",
            "something",
            "anything",
            "lot",
            "kind",
            "type",
            "part",
            "way",
        ]
        .into_iter()
        .collect()
    })
}

fn tokens(text: &str) -> std::collections::HashSet<String> {
    let re = Regex::new(r"[a-z0-9]+").unwrap();
    let mut out = std::collections::HashSet::new();
    for m in re.find_iter(&text.to_lowercase()) {
        let tok = m.as_str();
        if stopwords().contains(tok) {
            continue;
        }
        out.insert(tok.to_string());
        if tok.len() > 3 && tok.ends_with('s') && !tok.ends_with("ss") {
            out.insert(tok[..tok.len() - 1].to_string());
        }
    }
    out
}

/// `_lexical_rank` — candidates ranked by IDF-weighted token overlap.
fn lexical_rank(text: &str, options: &serde_json::Map<String, Value>) -> Vec<(f64, String)> {
    let query = tokens(text);
    let docs: Vec<(String, std::collections::HashSet<String>)> = options
        .iter()
        .map(|(name, crit)| {
            let crit_s = crit.as_str().unwrap_or("");
            (name.clone(), tokens(&format!("{name} {crit_s}")))
        })
        .collect();
    let mut df: HashMap<String, usize> = HashMap::new();
    for (_, toks) in &docs {
        for tok in toks {
            *df.entry(tok.clone()).or_insert(0) += 1;
        }
    }
    let n = docs.len().max(1) as f64;
    let mut scored: Vec<(f64, String)> = docs
        .into_iter()
        .map(|(name, toks)| {
            let score: f64 = query
                .intersection(&toks)
                .map(|t| (n / (1.0 + *df.get(t).unwrap_or(&0) as f64)).ln() + 1.0)
                .sum();
            (score, name)
        })
        .collect();
    // Python sorted(key=(-score, name)) — score desc, name asc.
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.cmp(&b.1))
    });
    scored
}

/// `_semantic_rank` — bi-encoder top-k via the in-crate `embed` engine
/// (the Rust twin of the `system1_embed.py` venv helper). Empty when the
/// model snapshot is unavailable or `CIEL_SYSTEM1_EMBED=0`.
fn semantic_rank(text: &str, options: &serde_json::Map<String, Value>, k: usize) -> Vec<String> {
    if std::env::var("CIEL_SYSTEM1_EMBED").ok().as_deref() == Some("0") {
        return Vec::new();
    }
    #[cfg(feature = "embed")]
    {
        let candidates: Vec<(String, String)> = options
            .iter()
            .map(|(n, c)| (n.clone(), c.as_str().unwrap_or("").to_string()))
            .collect();
        crate::embed::top_k(text, &candidates, k).unwrap_or_default()
    }
    #[cfg(not(feature = "embed"))]
    {
        let _ = (text, options, k);
        Vec::new()
    }
}

/// `shortlist_options` — semantic top-k ∪ lexical top-5 ∪ exact name-token
/// matches, capped at k, order preserved.
pub fn shortlist_options(
    text: &str,
    options: &serde_json::Map<String, Value>,
    k: usize,
) -> serde_json::Map<String, Value> {
    if options.len() <= k {
        return options.clone();
    }
    let mut keep: Vec<String> = Vec::new();
    for name in semantic_rank(text, options, k) {
        if options.contains_key(&name) && !keep.contains(&name) {
            keep.push(name);
        }
    }
    for (_, name) in lexical_rank(text, options).into_iter().take(5) {
        if !keep.contains(&name) {
            keep.push(name);
        }
    }
    let task_tokens = tokens_raw(text);
    for name in options.keys() {
        let name_tokens = tokens_raw(name);
        if !name_tokens.is_disjoint(&task_tokens) && !keep.contains(name) {
            keep.push(name.clone());
        }
    }
    let mut out = serde_json::Map::new();
    for name in keep.into_iter().take(k) {
        if let Some(v) = options.get(&name) {
            out.insert(name, v.clone());
        }
    }
    out
}

/// Raw `[a-z0-9]+` token set (no stopword/stem — used for the exact-name
/// overlap pass, which does not stem).
fn tokens_raw(text: &str) -> std::collections::HashSet<String> {
    let re = Regex::new(r"[a-z0-9]+").unwrap();
    re.find_iter(&text.to_lowercase())
        .map(|m| m.as_str().to_string())
        .collect()
}

// -------------------------------------------------------- resolve + event

fn resolve(state: &Value, questions: &Value, timeout_s: f64) -> (Option<Value>, bool, u64) {
    if let Some(cached) = cache_read(state, questions) {
        return (Some(cached), true, 0);
    }
    let started = Instant::now();
    let result = ask(state, questions, timeout_s);
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
    let ts = meta_ts.unwrap_or_else(py::utc_stamp_z);
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

/// `system1.completion_check` — evaluate objective vs empirical evidence on
/// the completion_check surface. Returns the Python dict shape
/// {choice, confidence, band, score, answers, model} or None (fail-open).
pub fn completion_check(
    objective: &str,
    evidence: &str,
    task_class: &str,
    with_score: bool,
    timeout_s: f64,
) -> Option<Value> {
    if disabled() {
        return None;
    }
    let mut state = json!({"objective": objective, "evidence": evidence});
    if !task_class.is_empty() {
        state["task_class"] = json!(task_class);
    }
    let questions = completion_questions(with_score);
    let payload = json!({
        "surface": "completion_check",
        "state": state,
        "questions": questions,
        "meta": {"pipeline": "completion_verification"},
    });
    let (result, hit, latency_ms) = resolve(&state, &questions, timeout_s);
    append_event(&event_record(&payload, result.as_ref(), hit, latency_ms));
    let result = result?;
    let answers = result.get("answers")?.clone();
    let done_ans = answers.get("done")?;
    if !done_ans.is_object() {
        return None;
    }
    let choice = done_ans.get("choice").cloned();
    let confidence = done_ans.get("confidence").cloned().unwrap_or(json!(0.0));
    let b = band("completion_check", &answers);
    let score = if with_score {
        answers
            .get("evidence_score")
            .and_then(|s| s.get("score"))
            .cloned()
            .unwrap_or(Value::Null)
    } else {
        Value::Null
    };
    Some(json!({
        "choice": choice,
        "confidence": confidence,
        "band": b,
        "score": score,
        "answers": answers,
        "model": result.get("model").cloned().unwrap_or(Value::Null),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Map;

    #[test]
    fn host_of_parses_authority() {
        assert_eq!(host_of("http://127.0.0.1:8765"), "127.0.0.1");
        assert_eq!(host_of("https://user@jev-agent.com:443/x"), "jev-agent.com");
        assert_eq!(host_of("https://EXAMPLE.com/p?y"), "example.com");
        assert_eq!(host_of("localhost"), "localhost");
    }

    #[test]
    fn endpoint_of_paths() {
        assert_eq!(
            endpoint_of("http://127.0.0.1:8765"),
            "http://127.0.0.1:8765/v1/systemone"
        );
        assert_eq!(
            endpoint_of("https://jev-agent.com"),
            "https://jev-agent.com/api/v1/systemone"
        );
        assert_eq!(endpoint_of("http://x/systemone"), "http://x/systemone");
    }

    #[test]
    fn tool_state_read_exec_sensitive() {
        let s = tool_state("read", "", "/tmp/x");
        assert_eq!(s["action"], "read data via read");
        assert_eq!(s["reversibility"], "read-only; no state change");
        assert_eq!(s["side_effects"], json!([]));

        let s = tool_state("exec", "rm -rf /tmp/z", "");
        assert_eq!(s["action"], "run shell command: rm -rf /tmp/z");
        assert!(!s["side_effects"].as_array().unwrap().is_empty());

        let s = tool_state("write", "", "/home/u/.ssh/id");
        assert_eq!(s["targets_sensitive_path"], true);

        let s = tool_state("write", "", "/tmp/plain");
        assert!(s.get("targets_sensitive_path").is_none());
        assert_eq!(s["action"], "create or modify the file at /tmp/plain");
    }

    #[test]
    fn band_flag_uncertain_pass() {
        // "incomplete" flags on completion_check.
        let a = json!({"done": {"choice": "incomplete", "confidence": 0.9}});
        assert_eq!(band("completion_check", &a), "flag");
        let a = json!({"done": {"choice": "complete", "confidence": 0.9}});
        assert_eq!(band("completion_check", &a), "pass");
        // below tau → uncertain
        let a = json!({"done": {"choice": "complete", "confidence": 0.01}});
        assert_eq!(band("completion_check", &a), "uncertain");
        // dangerous on risk surface → flag
        let a = json!({"risk": {"choice": "dangerous", "confidence": 0.9}});
        assert_eq!(band("pre_tool_risk", &a), "flag");
        let a = json!({"risk": {"choice": "safe", "confidence": 0.9}});
        assert_eq!(band("pre_tool_risk", &a), "pass");
    }

    #[test]
    fn shortlist_passthrough_when_small() {
        let mut options = Map::new();
        options.insert("a".into(), json!("alpha"));
        options.insert("b".into(), json!("beta"));
        let out = shortlist_options("anything", &options, 10);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn shortlist_lexical_and_name_hits() {
        std::env::set_var("CIEL_SYSTEM1_EMBED", "0"); // skip model load
        let mut options = Map::new();
        for i in 0..20 {
            options.insert(format!("skill-{i:02}"), json!("unrelated filler words"));
        }
        options.insert("git-ops".into(), json!("git status branches commits"));
        options.insert("sql-runner".into(), json!("database query sql"));
        let out = shortlist_options("check my git branches", &options, 5);
        assert!(out.contains_key("git-ops"));
        assert!(out.len() <= 5);
        std::env::remove_var("CIEL_SYSTEM1_EMBED");
    }

    #[test]
    fn surface_tau_defaults() {
        // env vars are process-global; assert defaults on surfaces where
        // CIEL_SYSTEM1_TAU_* is not set in this test environment.
        std::env::remove_var("CIEL_SYSTEM1_TAU_PRE_TOOL_RISK");
        std::env::remove_var("CIEL_SYSTEM1_TAU");
        // policy.json may override defaults — only check unknown surface.
        assert_eq!(surface_tau("nonexistent_surface"), DEFAULT_TAU);
    }
}
