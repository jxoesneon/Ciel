//! Transcript secret sanitizer — Rust port of `scripts/transcript_sanitize.py`.
//!
//! Modes:
//!   ciel sanitize --scan [--deep]   report files + categories (never content)
//!   ciel sanitize --redact [--dry]  replace matches with [REDACTED:<category>]
//!                                   in place; writes <file>.bak first
//!
//! Stores scanned: devin transcripts + summaries, antigravity conversation DBs
//! (binary-safe: redaction only applied to decodable UTF-8 spans — equivalent
//! to Python's surrogateescape, since no pattern can match a surrogate).
//! sessions.db is a live SQLite WAL store and gets SQL-level redaction with
//! busy-retry; when locked, the `sessions_db_sanitize_pending` flag is set in
//! the watchdog state for the next SessionStart to consume.

use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{paths, secretscan};

const BINARY_SUFFIXES: [&str; 3] = [".db", ".sqlite", ".sqlite3"];
const SCAN_SIZE_CAP: u64 = 64 * 1024 * 1024;

// (table, column, row-key, tier, deep_only) — mirror of SESSIONS_TABLES.
const SESSIONS_TABLES: [(&str, &str, &str, &str, bool); 5] = [
    ("prompt_history", "content", "id", "broad", false),
    ("message_nodes", "chat_message", "row_id", "big", true),
    ("sessions", "metadata", "id", "broad", false),
    ("sessions", "cogs_json", "id", "broad", false),
    ("sessions", "title", "id", "broad", false),
];

const PREFILTER_STRICT: [&str; 19] = [
    r"%ghp\_%",
    r"%gho\_%",
    r"%ghu\_%",
    r"%ghs\_%",
    r"%ghr\_%",
    r"%github\_pat\_%",
    r"%AKIA%",
    r"%PRIVATE KEY%",
    r"%xox%",
    r"%AIza%",
    r"%eyJ%.%.%",
    r"%npm\_%",
    r"%cio_________________________%",
    r"%sk\_%____________________%",
    r"%pk\_%____________________%",
    r"%key\_%____________________%",
    r"%api\_%____________________%",
    r"%tok\_%____________________%",
    r"%sk-____________________%",
];
const PREFILTER_ASSIGNMENT: [&str; 19] = [
    r"%password%",
    r"%passwd%",
    r"%passphrase%",
    r"%api\_key%",
    r"%api-key%",
    r"%access\_token%",
    r"%access-token%",
    r"%auth\_token%",
    r"%auth-token%",
    r"%secret\_key%",
    r"%secret-key%",
    r"%client\_secret%",
    r"%client-secret%",
    r"%token is%",
    r"%secret is%",
    r"%token=%",
    r"%secret=%",
    r"%token:%",
    r"%secret:%",
];

fn prefilter_broad() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = PREFILTER_STRICT.to_vec();
    v.extend(PREFILTER_ASSIGNMENT);
    v.extend(["%secret%", "%token%"]);
    v
}
fn prefilter_big() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = PREFILTER_STRICT.to_vec();
    v.extend(PREFILTER_ASSIGNMENT);
    v
}
fn terms(tier: &str) -> Vec<&'static str> {
    match tier {
        "strict" => PREFILTER_STRICT.to_vec(),
        "big" => prefilter_big(),
        _ => prefilter_broad(),
    }
}

fn sessions_db_path(home: &Path) -> PathBuf {
    std::env::var_os("CIEL_SESSIONS_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            home.join(".local")
                .join("share")
                .join("devin")
                .join("cli")
                .join("sessions.db")
        })
}

fn stores(home: &Path, ciel: &Path) -> Vec<(PathBuf, &'static str)> {
    let devin = home.join(".local").join("share").join("devin").join("cli");
    vec![
        (devin.join("transcripts"), "*.json"),
        (devin.join("summaries"), "*.md"),
        (devin.join("logs"), "*.log"),
        (devin.join("logs"), "*.log.gz"),
        (
            home.join(".gemini")
                .join("antigravity")
                .join("conversations"),
            "*.db",
        ),
        (
            home.join(".gemini")
                .join("antigravity-ide")
                .join("conversations"),
            "*.db",
        ),
        (
            home.join(".gemini")
                .join("antigravity-cli")
                .join("conversations"),
            "*.db",
        ),
        (ciel.join("system1"), "*.jsonl"),
        (ciel.join("checkpoints"), "*.jsonl"),
        (ciel.to_path_buf(), "activity.log"),
        (ciel.to_path_buf(), "grants.log"),
    ]
}

fn name_matches(name: &str, pattern: &str) -> bool {
    if let Some(suffix) = pattern.strip_prefix('*') {
        name.ends_with(suffix)
    } else {
        name == pattern
    }
}

fn iter_files(home: &Path, ciel: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for (base, pattern) in stores(home, ciel) {
        if !base.is_dir() {
            continue;
        }
        let mut files: Vec<PathBuf> = std::fs::read_dir(&base)
            .map(|rd| {
                rd.flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.is_file()
                            && p.file_name()
                                .and_then(|n| n.to_str())
                                .is_some_and(|n| name_matches(n, pattern))
                    })
                    .collect()
            })
            .unwrap_or_default();
        files.sort();
        out.extend(files);
    }
    out
}

/// `[REDACTED:<cat[:10]>]` padded with `*` to the match's char length —
/// mirror of `_length_preserving` (char count, not bytes, same as Python).
fn length_preserving(match_text: &str, category: &str) -> String {
    let n = match_text.chars().count();
    let cat10: String = category.chars().take(10).collect();
    let tag = format!("[REDACTED:{cat10}]");
    let t = tag.chars().count();
    if n >= t {
        format!("{tag}{}", "*".repeat(n - t))
    } else {
        "*".repeat(n)
    }
}

fn compiled() -> Vec<(&'static str, Regex)> {
    secretscan::PATTERNS
        .iter()
        .filter_map(|(n, p)| Regex::new(p).ok().map(|r| (*n, r)))
        .collect()
}

/// Apply every category substitution to one UTF-8 span.
fn sub_str(
    s: &str,
    binary: bool,
    rxs: &[(&str, Regex)],
    replacements: &mut usize,
    cats: &mut BTreeSet<String>,
) -> String {
    let mut cur = s.to_string();
    for (name, rx) in rxs {
        cur = rx
            .replace_all(&cur, |m: &regex::Captures| {
                *replacements += 1;
                cats.insert((*name).to_string());
                if binary {
                    length_preserving(m.get(0).unwrap().as_str(), name)
                } else {
                    format!("[REDACTED:{name}]")
                }
            })
            .into_owned();
    }
    cur
}

/// Surrogateescape-equivalent: substitute within each valid-UTF-8 span and
/// copy invalid bytes verbatim — byte-faithful for protobuf-in-BLOB stores.
fn sub_bytes(
    raw: &[u8],
    binary: bool,
    rxs: &[(&str, Regex)],
    replacements: &mut usize,
    cats: &mut BTreeSet<String>,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        match std::str::from_utf8(&raw[i..]) {
            Ok(s) => {
                out.extend_from_slice(sub_str(s, binary, rxs, replacements, cats).as_bytes());
                break;
            }
            Err(e) => {
                let v = e.valid_up_to();
                if v > 0 {
                    let s = std::str::from_utf8(&raw[i..i + v]).unwrap();
                    out.extend_from_slice(sub_str(s, binary, rxs, replacements, cats).as_bytes());
                }
                i += v;
                let skip = e.error_len().unwrap_or(raw.len() - i);
                out.extend_from_slice(&raw[i..i + skip]);
                i += skip;
            }
        }
    }
    out
}

fn gunzip(raw: &[u8]) -> Option<Vec<u8>> {
    let mut dec = flate2::read::GzDecoder::new(raw);
    let mut out = Vec::new();
    dec.read_to_end(&mut out).ok()?;
    Some(out)
}

fn read_text(path: &Path) -> Option<String> {
    let raw = std::fs::read(path).ok()?;
    let bytes = if path.extension().and_then(|e| e.to_str()) == Some("gz") {
        gunzip(&raw)?
    } else {
        raw
    };
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn is_binary(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| BINARY_SUFFIXES.contains(&format!(".{}", e.to_lowercase()).as_str()))
        .unwrap_or(false)
}

/// Mirror of `_literal_needle` — longest leading literal in a LIKE pattern,
/// used as a byte-level `instr()` needle so BLOBs (invisible to LIKE, which
/// also stops at embedded NULs) are still found.
fn literal_needle(term: &str) -> String {
    let b: Vec<char> = term.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        let ch = b[i];
        if ch == '\\' && i + 1 < b.len() {
            out.push(b[i + 1]);
            i += 2;
            continue;
        }
        if ch == '%' || ch == '_' {
            if !out.is_empty() {
                break;
            }
            i += 1;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

fn prefilter_where(col: &str, tier: &str) -> String {
    let t = terms(tier);
    let mut parts: Vec<String> = t
        .iter()
        .map(|p| format!("CAST({col} AS TEXT) LIKE '{p}' ESCAPE '\\'"))
        .collect();
    let mut needles: BTreeSet<String> = t.iter().map(|p| literal_needle(p)).collect();
    needles.remove("");
    for n in needles {
        let hex: String = n.bytes().map(|b| format!("{b:02x}")).collect();
        parts.push(format!("instr({col}, X'{hex}') > 0"));
    }
    parts.join(" OR ")
}

fn scan_sessions_db(home: &Path, deep: bool) -> Map<String, Value> {
    let mut out = Map::new();
    let db_path = sessions_db_path(home);
    if !db_path.is_file() {
        return out;
    }
    let db = match rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ) {
        Ok(c) => c,
        Err(_) => {
            out.insert("sessions.db".into(), json!(["<locked>"]));
            return out;
        }
    };
    let _ = db.busy_timeout(Duration::from_secs(10));
    let mut cats: BTreeSet<String> = BTreeSet::new();
    for (table, col, _key, tier, deep_only) in SESSIONS_TABLES {
        if deep_only && !deep {
            continue;
        }
        let sql = format!(
            "SELECT {col} FROM {table} WHERE {}",
            prefilter_where(col, tier)
        );
        let Ok(mut stmt) = db.prepare(&sql) else {
            continue;
        };
        let Ok(rows) = stmt.query_map([], |r| r.get::<_, rusqlite::types::Value>(0)) else {
            continue;
        };
        for val in rows.flatten() {
            let text = match val {
                rusqlite::types::Value::Text(s) => s,
                rusqlite::types::Value::Blob(b) => String::from_utf8_lossy(&b).into_owned(),
                _ => continue,
            };
            if text.is_empty() {
                continue;
            }
            let r = secretscan::scan(&text);
            if let Some(arr) = r["categories"].as_array() {
                for c in arr.iter().filter_map(|c| c.as_str()) {
                    cats.insert(c.to_string());
                }
            }
        }
    }
    if !cats.is_empty() {
        out.insert(
            "sessions.db".into(),
            json!(cats.into_iter().collect::<Vec<_>>()),
        );
    }
    out
}

fn scan_all(home: &Path, ciel: &Path, deep: bool) -> Map<String, Value> {
    let mut hits = Map::new();
    for f in iter_files(home, ciel) {
        let Ok(meta) = f.metadata() else { continue };
        if meta.len() > SCAN_SIZE_CAP {
            continue;
        }
        let Some(text) = read_text(&f) else { continue };
        let r = secretscan::scan(&text);
        if r["hits"].as_u64().unwrap_or(0) > 0 {
            hits.insert(f.to_string_lossy().into_owned(), r["categories"].clone());
        }
    }
    for (k, v) in scan_sessions_db(home, deep) {
        hits.insert(k, v);
    }
    hits
}

fn redact_file(path: &Path, dry: bool) -> Value {
    let gz = path.extension().and_then(|e| e.to_str()) == Some("gz");
    let raw = match std::fs::read(path) {
        Ok(r) => r,
        Err(e) => {
            return json!({"file": path.to_string_lossy(), "changed": false,
                          "error": e.to_string()})
        }
    };
    let work = if gz {
        match gunzip(&raw) {
            Some(w) => w,
            None => {
                return json!({"file": path.to_string_lossy(), "changed": false,
                              "error": "gzip decompress failed"})
            }
        }
    } else {
        raw.clone()
    };
    let binary = is_binary(path);
    let rxs = compiled();
    let mut replacements = 0usize;
    let mut cats: BTreeSet<String> = BTreeSet::new();
    let text = sub_bytes(&work, binary, &rxs, &mut replacements, &mut cats);

    if replacements == 0 {
        return json!({"file": path.to_string_lossy(), "changed": false,
                      "replacements": 0});
    }
    let cat_list: Vec<String> = cats.into_iter().collect();
    if dry {
        return json!({"file": path.to_string_lossy(), "changed": true,
                      "dry": true, "replacements": replacements,
                      "categories": cat_list});
    }
    // backup → write → chmod, mirroring the Python order and failure shape
    let bak = PathBuf::from(format!("{}.bak", path.to_string_lossy()));
    let result = (|| -> Result<(), std::io::Error> {
        std::fs::write(&bak, &raw)?;
        chmod(&bak, 0o600)?;
        let payload = if gz {
            let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            use std::io::Write;
            enc.write_all(&text)?;
            enc.finish()?
        } else {
            text
        };
        std::fs::write(path, payload)?;
        chmod(path, 0o600)
    })();
    match result {
        Ok(()) => json!({"file": path.to_string_lossy(), "changed": true,
                        "replacements": replacements, "categories": cat_list}),
        Err(e) => json!({"file": path.to_string_lossy(), "changed": false,
                        "error": e.to_string()}),
    }
}

#[cfg(unix)]
fn chmod(p: &Path, mode: u32) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(mode))
}
#[cfg(not(unix))]
fn chmod(_p: &Path, _mode: u32) -> Result<(), std::io::Error> {
    Ok(())
}

/// Mirror of `redact_sessions_db` — BEGIN IMMEDIATE busy-retry, per-table
/// prefiltered rows, length-preserving subs for BLOB values, updates applied
/// after the read cursor finishes, wal_checkpoint(TRUNCATE) on commit.
pub fn redact_sessions_db(home: &Path, dry: bool, retries: usize, wait_s: u64) -> Value {
    let db_path = sessions_db_path(home);
    if !db_path.is_file() {
        return json!({"file": db_path.to_string_lossy(), "changed": false,
                      "reason": "absent"});
    }
    let mut db: Option<rusqlite::Connection> = None;
    for attempt in 0..retries {
        let attempt_ok = rusqlite::Connection::open(&db_path).and_then(|c| {
            c.busy_timeout(Duration::from_secs(wait_s))?;
            c.execute_batch("BEGIN IMMEDIATE")?;
            Ok(c)
        });
        match attempt_ok {
            Ok(c) => {
                db = Some(c);
                break;
            }
            Err(_) => {
                if attempt + 1 < retries {
                    std::thread::sleep(Duration::from_secs(wait_s));
                }
            }
        }
    }
    let Some(db) = db else {
        return json!({"file": db_path.to_string_lossy(), "changed": false,
                      "locked": true,
                      "reason": "database is locked — rerun when no devin session is active"});
    };

    let mut results = json!({"file": db_path.to_string_lossy(),
                            "changed": false, "tables": {}});
    let rxs = compiled();
    for (table, col, key, tier, _deep) in SESSIONS_TABLES {
        let sql = format!(
            "SELECT {key}, {col} FROM {table} WHERE {}",
            prefilter_where(col, tier)
        );
        let Ok(mut stmt) = db.prepare(&sql) else {
            continue;
        };
        let rows: Vec<(rusqlite::types::Value, rusqlite::types::Value)> =
            match stmt.query_map([], |r| {
                Ok((
                    r.get::<_, rusqlite::types::Value>(0)?,
                    r.get::<_, rusqlite::types::Value>(1)?,
                ))
            }) {
                Ok(m) => m.flatten().collect(),
                Err(_) => continue,
            };
        let mut n = 0usize;
        let mut pending: Vec<(rusqlite::types::Value, rusqlite::types::Value)> = Vec::new();
        for (keyv, val) in rows {
            let (work, is_bytes) = match &val {
                rusqlite::types::Value::Text(s) => (s.clone().into_bytes(), false),
                rusqlite::types::Value::Blob(b) => (b.clone(), true),
                _ => continue,
            };
            let mut reps = 0usize;
            let mut cats = BTreeSet::new();
            let new = sub_bytes(&work, is_bytes, &rxs, &mut reps, &mut cats);
            if new != work {
                n += 1;
                if !dry {
                    let out = if is_bytes {
                        rusqlite::types::Value::Blob(new)
                    } else {
                        rusqlite::types::Value::Text(String::from_utf8_lossy(&new).into_owned())
                    };
                    pending.push((out, keyv));
                }
            }
        }
        // apply after the read pass — updating a table while its SELECT is
        // iterating can skip rows (same contract as the Python)
        for (out, keyv) in pending {
            let _ = db.execute(
                &format!("UPDATE {table} SET {col}=? WHERE {key}=?"),
                rusqlite::params![out, keyv],
            );
        }
        if n > 0 {
            results["tables"][table] = json!(n);
            results["changed"] = json!(true);
        }
    }
    if dry {
        let _ = db.execute_batch("ROLLBACK");
    } else {
        let _ = db.execute_batch("COMMIT");
        if db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)").is_err() {
            results["checkpoint"] = json!("busy");
        }
    }
    results
}

fn tighten_store_perms(home: &Path, ciel: &Path) -> usize {
    let mut n = 0;
    for (base, _) in stores(home, ciel) {
        if !base.is_dir() {
            continue;
        }
        if let Ok(meta) = base.metadata() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if meta.permissions().mode() & 0o077 != 0 && chmod(&base, 0o700).is_ok() {
                    n += 1;
                }
            }
        }
    }
    n
}

/// `json.dumps(obj, indent=1)` equivalent — Python's indent is a space
/// count, serde_json's pretty formatter is fixed at two; emit it manually.
/// Set `sessions_db_sanitize_pending` in the watchdog state — same file the
/// Python `session_watchdog._save` writes.
fn flag_pending(ciel: &Path) {
    let sp = ciel.join("checkpoints").join("watchdog_state.json");
    let mut state: Value = std::fs::read_to_string(&sp)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| json!({}));
    state["sessions_db_sanitize_pending"] = json!(true);
    if let Some(parent) = sp.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Mirror session_watchdog._save: indent=1, ensure_ascii=False, chmod 0600.
    if std::fs::write(&sp, crate::jsonfmt::dumps_indent(&state, 1)).is_ok() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&sp, std::fs::Permissions::from_mode(0o600));
        }
    }
}

/// `ciel sanitize [--scan|--redact] [--dry] [--deep]`.
pub fn main_(args: &[String]) -> i32 {
    let home = paths::home_dir();
    let ciel = paths::ciel_home();
    let redact = args.iter().any(|a| a == "--redact");
    let dry = args.iter().any(|a| a == "--dry");
    let deep = args.iter().any(|a| a == "--deep");

    if redact {
        let mut results: Vec<Value> = Vec::new();
        for f in iter_files(&home, &ciel) {
            let r = redact_file(&f, dry);
            if r["changed"].as_bool().unwrap_or(false) || r.get("error").is_some() {
                results.push(r);
            }
        }
        let mut sdb = redact_sessions_db(&home, dry, if dry { 1 } else { 6 }, 5);
        if sdb["locked"].as_bool().unwrap_or(false) {
            flag_pending(&ciel);
            sdb["deferred"] = json!("flagged for next SessionStart");
        }
        let tightened = tighten_store_perms(&home, &ciel);
        println!(
            "{}",
            crate::jsonfmt::dumps_indent(
                &json!({
                    "mode": if dry { "dry" } else { "redact" },
                    "files_changed": results.len(),
                    "perms_tightened": tightened,
                    "sessions_db": sdb,
                    "results": results,
                }),
                1,
            )
        );
        return 0;
    }

    let hits = scan_all(&home, &ciel, deep);
    let mut names = Map::new();
    for (k, v) in &hits {
        let base = Path::new(k)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(k)
            .to_string();
        names.insert(base, v.clone());
    }
    println!(
        "{}",
        crate::jsonfmt::dumps_indent(
            &json!({
                "mode": "scan",
                "files_with_hits": hits.len(),
                "hits": Value::Object(names),
            }),
            1,
        )
    );
    if hits.is_empty() {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_needle_extracts_prefix() {
        assert_eq!("ghp_", literal_needle(r"%ghp\_%"));
        assert_eq!("AKIA", literal_needle(r"%AKIA%"));
        assert_eq!("secret", literal_needle(r"%secret%"));
        assert_eq!("sk_", literal_needle(r"%sk\_%____________________%"));
        assert_eq!("sk-", literal_needle(r"%sk-____________________%"));
    }

    #[test]
    fn length_preserving_pads() {
        let r = length_preserving("ghp_abcdefghij0123456789ABCD", "github_token");
        assert_eq!(28, r.chars().count());
        assert!(r.starts_with("[REDACTED:github_tok]"));
    }

    #[test]
    fn sub_bytes_preserves_invalid_utf8() {
        let mut raw = b"tok ghp_abcdefghij0123456789ABCD \x00\xff end".to_vec();
        let rxs = compiled();
        let mut n = 0;
        let mut c = BTreeSet::new();
        let out = sub_bytes(&raw, true, &rxs, &mut n, &mut c);
        assert_eq!(1, n);
        // invalid bytes verbatim, length preserved
        assert!(out.windows(2).any(|w| w == b"\x00\xff"));
        assert_eq!(raw.len(), out.len());
        raw.clear();
    }

}
