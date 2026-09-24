//! Session watchdog — Rust port of `hooks/lib/session_watchdog.py`.
//! Stall detection + resume hints + incremental transcript secret sweep.
//! The deferred sessions.db sanitize runs in-process via `sanitize` —
//! SessionStart spawns `watchdog --sanitize-pending` detached so its
//! bounded check window is never spent inside a SQLite redaction.

use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::{ledger, paths, sanitize, secretscan};

const MAX_RESUME_PER_SESSION: usize = 1;
const MAX_RESUME_PER_DAY: usize = 3;
const MIN_RESUME_INTERVAL: f64 = 1800.0;
const STALL_AGE_S: f64 = 600.0;
const SCAN_MAX_BYTES: u64 = 8 * 1024 * 1024;

fn error_tail() -> Regex {
    Regex::new(
        r#"(?i)rate_limit_error|"status"\s*:\s*429|HTTP 429|429 Too Many|overloaded_error|insufficient_quota|"error"\s*:\s*\{[^}]{0,200}rate"#,
    )
    .unwrap()
}

fn state_path(ciel: &Path) -> PathBuf {
    ciel.join("checkpoints").join("watchdog_state.json")
}
fn hint_path(ciel: &Path) -> PathBuf {
    ciel.join("checkpoints").join("resume_hint.json")
}
fn transcripts(home: &Path) -> PathBuf {
    home.join(".local")
        .join("share")
        .join("devin")
        .join("cli")
        .join("transcripts")
}
fn summaries(home: &Path) -> PathBuf {
    home.join(".local")
        .join("share")
        .join("devin")
        .join("cli")
        .join("summaries")
}

fn now_s() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

fn load(path: &Path) -> Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}))
}

fn save(path: &Path, data: &Value) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Python _save: json.dumps(data, ensure_ascii=False, indent=1).
    if std::fs::write(path, crate::jsonfmt::dumps_indent(data, 1)).is_ok() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
    }
}

fn emit_signal(ciel: &Path, name: &str, payload: &Value) {
    let dir = ciel.join("improvements").join("signals");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let stamp = OffsetDateTime::now_utc()
        .format(&time::macros::format_description!(
            "[year][month][day]T[hour][minute][second]Z"
        ))
        .unwrap_or_else(|_| "00000000T000000Z".into());
    // Python _emit_signal: json.dumps(payload, ensure_ascii=False, indent=1).
    let _ = std::fs::write(
        dir.join(format!("{name}-{stamp}.json")),
        format!("{}\n", crate::jsonfmt::dumps_indent(payload, 1)),
    );
}

// ------------------------------------------------------------- stall find
fn recent_entries(ciel: &Path, n: usize) -> Vec<Value> {
    let Ok(text) = std::fs::read_to_string(ciel.join("activity.log")) else {
        return Vec::new();
    };
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(n)..]
        .iter()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .collect()
}

fn session_last_seen(ciel: &Path) -> BTreeMap<String, f64> {
    let mut seen: BTreeMap<String, f64> = BTreeMap::new();
    for e in recent_entries(ciel, 400) {
        let (Some(sid), Some(ts)) = (
            e.get("session_id").and_then(|v| v.as_str()),
            e.get("ts").and_then(|v| v.as_str()),
        ) else {
            continue;
        };
        let t = OffsetDateTime::parse(&ts.replace('Z', "+00:00"), &Rfc3339)
            .ok()
            .map(|d| d.unix_timestamp() as f64);
        let Some(t) = t else { continue };
        let cur = seen.entry(sid.to_string()).or_insert(0.0);
        if t > *cur {
            *cur = t;
        }
    }
    seen
}

fn transcript_tail_errors(home: &Path) -> Vec<String> {
    let dir = transcripts(home);
    let mut files: Vec<(u64, PathBuf)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            if let Ok(md) = p.metadata() {
                files.push((md.mtime() as u64, p));
            }
        }
    }
    files.sort_by_key(|(m, _)| *m);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let rx = error_tail();
    let mut flagged = Vec::new();
    for (_, f) in files.iter().skip(files.len().saturating_sub(5)) {
        let Ok(md) = f.metadata() else { continue };
        if now.saturating_sub(md.mtime() as u64) > (STALL_AGE_S * 6.0) as u64 {
            continue;
        }
        let Ok(bytes) = std::fs::read(f) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        let tail: String = text
            .chars()
            .rev()
            .take(200_000)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if rx.is_match(&tail) {
            flagged.push(
                f.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );
            if flagged.len() >= 3 {
                break;
            }
        }
    }
    flagged
}

fn find_stalled(home: &Path, ciel: &Path, current: Option<&str>) -> Value {
    let last_seen = session_last_seen(ciel);
    let pending = ledger::pending_by_session();
    let now = now_s();
    let mut stalled = Map::new();
    for (sid, count) in pending {
        if Some(sid.as_str()) == current || sid == "unknown" {
            continue;
        }
        let last = last_seen.get(&sid).copied().unwrap_or(0.0);
        if now - last > STALL_AGE_S {
            stalled.insert(
                sid,
                json!({"pending": count, "idle_s": (now - last) as i64}),
            );
        }
    }
    json!({
        "stalled": stalled,
        "error_tails": transcript_tail_errors(home),
    })
}

// -------------------------------------------------- transcript sweep (M3c)
fn transcript_sweep(home: &Path, ciel: &Path, state: &mut Value) -> Value {
    let devin_cli = home.join(".local").join("share").join("devin").join("cli");
    let stores: Vec<(PathBuf, &str)> = vec![
        (transcripts(home), "*.json"),
        (summaries(home), "*.md"),
        (devin_cli.join("logs"), "*.log"),
        (devin_cli.clone(), "sessions.db*"),
        (ciel.join("system1"), "*.jsonl"),
        (ciel.join("checkpoints"), "*.jsonl"),
        (ciel.to_path_buf(), "activity.log"),
        (ciel.to_path_buf(), "grants.log"),
    ];

    let mut prev: Map<String, Value> = state
        .get("transcript_mtimes")
        .and_then(|m| m.as_object().cloned())
        .unwrap_or_default();
    let mut hits: Map<String, Value> = Map::new();
    let mut scanned = 0usize;

    for (base, pattern) in &stores {
        if !base.is_dir() {
            continue;
        }
        let Ok(rd) = std::fs::read_dir(base) else {
            continue;
        };
        for e in rd.flatten() {
            let f = e.path();
            if !f.is_file() {
                continue;
            }
            let name = f.file_name().unwrap_or_default().to_string_lossy();
            if !name_matches(&name, pattern) {
                continue;
            }
            let Ok(md) = f.metadata() else { continue };
            let mtime = md.mtime() as f64 + md.mtime_nsec() as f64 / 1e9;
            let key = f.to_string_lossy().into_owned();
            if prev.get(&key).and_then(|v| v.as_f64()) == Some(mtime) {
                continue;
            }
            if md.len() > SCAN_MAX_BYTES {
                continue;
            }
            let Ok(bytes) = std::fs::read(&f) else {
                continue;
            };
            scanned += 1;
            prev.insert(key.clone(), json!(mtime));
            let text = String::from_utf8_lossy(&bytes);
            let r = secretscan::scan(&text);
            if r["hits"].as_i64().unwrap_or(0) > 0 {
                hits.insert(name.into_owned(), r["categories"].clone());
                state["transcript_hits"][&key] = r["categories"].clone();
            } else {
                state["transcript_hits"][&key] = Value::Null;
            }
        }
    }
    state["transcript_mtimes"] = Value::Object(prev);
    // prune cleared entries — files stay flagged until a rescan clears them
    let known: Map<String, Value> = state
        .get("transcript_hits")
        .and_then(|m| m.as_object().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter(|(_, v)| !v.is_null())
        .collect();
    let known_count = known.len();
    state["transcript_hits"] = Value::Object(known);
    json!({"scanned": scanned, "hits": hits, "known_hits": known_count})
}

/// Glob-lite for the sweep patterns: `*` any, `*suffix`, `prefix*`, and
/// multi-segment `a*b` (first/last anchors + in-order contains).
fn name_matches(name: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut rest = name;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !rest.starts_with(part) {
                return false;
            }
            rest = &rest[part.len()..];
        } else if i == parts.len() - 1 {
            return rest.ends_with(part);
        } else if let Some(j) = rest.find(part) {
            rest = &rest[j + part.len()..];
        } else {
            return false;
        }
    }
    // Reaching the end means every non-* segment matched (a trailing '*'
    // swallows whatever remains).
    true
}

// ------------------------------------------------------------------ resume
fn resume_capable(state: &Value, session_hint: &str) -> (bool, String) {
    let now = now_s();
    let today = OffsetDateTime::now_utc()
        .format(&time::macros::format_description!("[year]-[month]-[day]"))
        .unwrap_or_default();
    let mut attempts: Map<String, Value> = state
        .get("resume_attempts")
        .and_then(|m| m.as_object().cloned())
        .unwrap_or_default();
    if attempts.get("_day").and_then(|v| v.as_str()) != Some(today.as_str()) {
        attempts = Map::new();
        attempts.insert("_day".into(), json!(today));
    }
    let per_session = attempts
        .get(session_hint)
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as usize;
    if per_session >= MAX_RESUME_PER_SESSION {
        return (false, "session cap reached".into());
    }
    let daily: usize = attempts
        .iter()
        .filter(|(k, _)| *k != "_day")
        .filter_map(|(_, v)| v.as_i64())
        .map(|v| v as usize)
        .sum();
    if daily >= MAX_RESUME_PER_DAY {
        return (false, "daily cap reached".into());
    }
    let last = state
        .get("last_resume")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    if now - last < MIN_RESUME_INTERVAL {
        return (false, "backoff window".into());
    }
    (true, "ok".into())
}

fn do_resume(ciel: &Path, session_hint: &str, reason: &str, dry: bool) -> Value {
    let sp = state_path(ciel);
    let mut state = load(&sp);
    let (ok, why) = resume_capable(&state, session_hint);
    if !ok {
        return json!({"fired": false, "reason": format!("capped: {why}")});
    }
    if std::env::var_os("CIEL_WATCHDOG_AUTORESUME").is_none() {
        return json!({"fired": false,
            "reason": "autoresume disabled (CIEL_WATCHDOG_AUTORESUME unset)"});
    }
    let prompt = "A previous session stalled with unfinished work. Read \
        ~/.ciel/checkpoints/requirements.jsonl for pending items and resume \
        them; verify progress before declaring completion.";
    if dry {
        return json!({"fired": false, "reason": "dry-run",
            "would_run": "devin -c -p <resume prompt>"});
    }
    // Python's subprocess.run(timeout=1800) — kill on expiry, fired=false.
    // wait_timeout keeps this portable (GNU `timeout` is absent on macOS).
    use wait_timeout::ChildExt;
    let fired = Command::new("devin")
        .args(["-c", "-p", prompt])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()
        .and_then(
            |mut child| match child.wait_timeout(Duration::from_secs(1800)) {
                Ok(Some(status)) => Some(status.success()),
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    Some(false)
                }
                Err(_) => None,
            },
        )
        .unwrap_or(false);

    let mut attempts = state
        .get("resume_attempts")
        .and_then(|m| m.as_object().cloned())
        .unwrap_or_default();
    let cur = attempts
        .get(session_hint)
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    attempts.insert(session_hint.to_string(), json!(cur + 1));
    state["resume_attempts"] = Value::Object(attempts);
    state["last_resume"] = json!(now_s());
    save(&sp, &state);
    emit_signal(
        ciel,
        "watchdog_resume",
        &json!({"session": session_hint, "reason": reason, "fired": fired}),
    );
    // Python's subprocess.run(notify-send, timeout=5), portable form.
    if let Ok(mut notify) = Command::new("notify-send")
        .args([
            "Ciel watchdog",
            &format!("Auto-resumed stalled session ({reason})"),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if matches!(notify.wait_timeout(Duration::from_secs(5)), Ok(None)) {
            let _ = notify.kill();
            let _ = notify.wait();
        }
    }
    json!({"fired": fired, "reason": reason})
}

// ------------------------------------------------- deferred sanitize
/// Consume the pending flag in-process — mirror of `_sessions_db_sanitize`:
/// redact with a short retry budget, keep the flag while locked, clear it
/// and emit the `sessions_db_sanitized` signal on success.
fn sessions_db_sanitize(home: &Path, ciel: &Path, state: &mut Value) -> Option<String> {
    if state.get("sessions_db_sanitize_pending") != Some(&json!(true)) {
        return None;
    }
    let r = sanitize::redact_sessions_db(home, false, 2, 3);
    if r["locked"].as_bool().unwrap_or(false) {
        return Some("sessions.db still locked — sanitize stays pending".into());
    }
    state["sessions_db_sanitize_pending"] = json!(false);
    let tables = &r["tables"];
    if r["changed"].as_bool().unwrap_or(false) {
        emit_signal(ciel, "sessions_db_sanitized", &json!({"tables": tables}));
        let n: u64 = tables
            .as_object()
            .map(|m| m.values().filter_map(|v| v.as_u64()).sum())
            .unwrap_or(0);
        return Some(format!("sessions.db sanitized ({n} rows redacted)"));
    }
    Some("sessions.db sanitize ran clean (no hits)".into())
}

/// Kick off the deferred sanitize as a detached child — identical spawn
/// contract to the Python (`/dev/null` stdio, new session, never waited).
/// Re-execs this binary (`watchdog --sanitize-pending`), which consumes the
/// flag in-process with no retry cap from SessionStart's timeout window.
fn detached_sanitize(_ciel: &Path, state: &Value) -> Option<String> {
    if state.get("sessions_db_sanitize_pending") != Some(&json!(true)) {
        return None;
    }
    let exe = std::env::var_os("CIEL_BIN")
        .map(PathBuf::from)
        .or_else(|| std::env::current_exe().ok());
    let Some(exe) = exe else {
        return Some("sessions.db sanitize deferred (no binary)".into());
    };
    let mut cmd = Command::new(exe);
    cmd.arg("watchdog")
        .arg("--sanitize-pending")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    match cmd.spawn() {
        Ok(_) => Some("sessions.db sanitize pending — detached run started".into()),
        Err(e) => Some(format!("sessions.db sanitize deferred ({e})")),
    }
}

// -------------------------------------------------------------------- main
/// Full check side-effects (state save, hint write, signals, detached
/// sanitize spawn); returns the joined hint line when there is anything
/// to surface, None when clean.
pub fn check_text(home: &Path, ciel: &Path, current: Option<&str>) -> Option<String> {
    let sp = state_path(ciel);
    let mut state = load(&sp);
    let stall = find_stalled(home, ciel, current);
    let sweep = transcript_sweep(home, ciel, &mut state);
    let sanitize_msg = detached_sanitize(ciel, &state);
    save(&sp, &state);

    let mut hints: Vec<String> = Vec::new();
    if let Some(m) = sanitize_msg {
        hints.push(m);
    }
    if let Some(stalled) = stall["stalled"].as_object() {
        for (sid, info) in stalled {
            hints.push(format!(
                "session {}… ended with {} unresolved ledger item(s) (idle \
                 {}m) — resume via `devin -c` or reconcile the ledger",
                &sid[..sid.len().min(8)],
                info["pending"],
                info["idle_s"].as_i64().unwrap_or(0) / 60,
            ));
        }
    }
    if let Some(tails) = stall["error_tails"].as_array() {
        if !tails.is_empty() {
            hints.push(format!(
                "recent transcript(s) end on rate-limit/error patterns: {}",
                tails
                    .iter()
                    .filter_map(|t| t.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    let hits = sweep["hits"].as_object().cloned().unwrap_or_default();
    if !hits.is_empty() {
        emit_signal(ciel, "transcript_secret_hits", &json!({"files": hits}));
    }
    let known = sweep["known_hits"].as_i64().unwrap_or(0);
    if known > 0 {
        let detail = if hits.is_empty() {
            String::new()
        } else {
            format!(
                " — new: {}",
                hits.iter()
                    .take(5)
                    .map(|(n, c)| format!(
                        "{n}({})",
                        c.as_array()
                            .map(|a| a
                                .iter()
                                .filter_map(|x| x.as_str())
                                .collect::<Vec<_>>()
                                .join("+"))
                            .unwrap_or_default()
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        hints.push(format!(
            "{known} transcript-store file(s) carry secret patterns — \
             sanitize via transcript_sanitize.py --redact and rotate{detail}"
        ));
    }

    let hp = hint_path(ciel);
    if hints.is_empty() {
        let _ = std::fs::remove_file(&hp);
        None
    } else {
        save(&hp, &json!({"ts": paths::utc_now_iso(), "hints": hints}));
        Some(hints.join("; "))
    }
}

pub fn cmd_check(home: &Path, ciel: &Path, current: Option<&str>) -> i32 {
    if let Some(line) = check_text(home, ciel, current) {
        println!("{line}");
    }
    0
}

pub fn cmd_resume(home: &Path, ciel: &Path, dry: bool) -> i32 {
    let stall = find_stalled(home, ciel, None);
    let stalled = stall["stalled"].as_object().cloned().unwrap_or_default();
    let tails = stall["error_tails"].as_array().cloned().unwrap_or_default();
    if stalled.is_empty() && tails.is_empty() {
        println!(
            "{}",
            crate::jsonfmt::dumps(&json!({"fired": false, "reason": "nothing stalled"}))
        );
        return 0;
    }
    let hint = stalled
        .keys()
        .next()
        .cloned()
        .or_else(|| tails.first().and_then(|t| t.as_str()).map(String::from))
        .unwrap_or_else(|| "unknown".into());
    let reason = format!(
        "{} stalled session(s), {} error-tail transcript(s)",
        stalled.len(),
        tails.len()
    );
    println!(
        "{}",
        crate::jsonfmt::dumps(&do_resume(ciel, &hint, &reason, dry))
    );
    0
}

/// `ciel watchdog [check|resume [--dry]]` — `--check` is the default.
pub fn main_(args: &[String]) -> i32 {
    let home = paths::home_dir();
    let ciel = paths::ciel_home();
    if args.iter().any(|a| a == "--sanitize-pending") {
        let sp = state_path(&ciel);
        let mut state = load(&sp);
        let msg = sessions_db_sanitize(&home, &ciel, &mut state);
        save(&sp, &state);
        println!(
            "{}",
            crate::jsonfmt::dumps(
                &json!({"sanitize": msg.unwrap_or_else(|| "nothing pending".into())})
            )
        );
        return 0;
    }
    if args.iter().any(|a| a == "--resume") {
        return cmd_resume(&home, &ciel, args.iter().any(|a| a == "--dry"));
    }
    let session = args
        .iter()
        .position(|a| a == "--session")
        .and_then(|i| args.get(i + 1))
        .map(String::as_str);
    cmd_check(&home, &ciel, session)
}
