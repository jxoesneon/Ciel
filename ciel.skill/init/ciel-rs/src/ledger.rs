//! Session requirement ledger — Rust port of `hooks/lib/requirements.py`.
//! Append-only JSONL events under `~/.ciel/checkpoints/requirements.jsonl`.

use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::paths;

pub fn ledger_path() -> PathBuf {
    paths::ciel_home()
        .join("checkpoints")
        .join("requirements.jsonl")
}

fn events() -> Vec<Value> {
    std::fs::read_to_string(ledger_path())
        .map(|text| {
            text.lines()
                .filter_map(|l| serde_json::from_str::<Value>(l).ok())
                .collect()
        })
        .unwrap_or_default()
}

fn append(event: &mut Value) {
    let path = ledger_path();
    let _ = std::fs::create_dir_all(path.parent().unwrap_or(&path));
    event["ts"] = json!(paths::utc_now_iso());
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        // Python: json.dumps(event, ensure_ascii=False).
        let _ = writeln!(f, "{}", crate::jsonfmt::dumps_raw(event));
    }
}

/// Unresolved `add` events (minus `done`s), optionally session-scoped.
/// Mirrors `requirements.pending_items`.
pub fn pending_items(session: Option<&str>) -> Vec<Value> {
    let mut added: Vec<(String, Value)> = Vec::new();
    let mut done: Vec<String> = Vec::new();
    for e in events() {
        if let Some(s) = session {
            let es = e.get("session").and_then(|v| v.as_str());
            if es.is_some() && es != Some(s) {
                continue;
            }
        }
        match e.get("op").and_then(|v| v.as_str()) {
            Some("add") => {
                if let Some(id) = e.get("id").and_then(|v| v.as_str()) {
                    // Python dict semantics: added[id] = e — last write wins,
                    // first insertion position kept.
                    match added.iter_mut().find(|(k, _)| k == id) {
                        Some(slot) => slot.1 = e.clone(),
                        None => added.push((id.to_string(), e.clone())),
                    }
                }
            }
            Some("done") => {
                if let Some(id) = e.get("id").and_then(|v| v.as_str()) {
                    done.push(id.to_string());
                }
            }
            _ => {}
        }
    }
    added
        .into_iter()
        .filter(|(id, _)| !done.contains(id))
        .map(|(_, e)| e)
        .collect()
}

/// Pending items grouped by session — mirrors
/// `session_watchdog.pending_ledger_by_session`.
pub fn pending_by_session() -> Vec<(String, usize)> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for e in pending_items(None) {
        let sid = e
            .get("session")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        match counts.iter_mut().find(|(k, _)| *k == sid) {
            Some((_, n)) => *n += 1,
            None => counts.push((sid, 1)),
        }
    }
    counts
}

fn resolve_id(needle: &str, session: Option<&str>) -> Option<String> {
    let items = pending_items(session);
    for e in &items {
        if e.get("id").and_then(|v| v.as_str()) == Some(needle) {
            return Some(needle.to_string());
        }
    }
    let needle_l = needle.to_lowercase();
    let matches: Vec<String> = items
        .iter()
        .filter(|e| {
            e.get("text")
                .and_then(|t| t.as_str())
                .is_some_and(|t| t.to_lowercase().contains(&needle_l))
        })
        .filter_map(|e| e.get("id").and_then(|v| v.as_str()).map(String::from))
        .collect();
    if matches.len() == 1 {
        Some(matches[0].clone())
    } else {
        None
    }
}

/// `ciel ledger {add|done|list|pending} [arg] [--session ID]`.
pub fn main_(args: &[String]) -> i32 {
    let op = args.first().map(String::as_str).unwrap_or("");
    let mut positional = String::new();
    let mut session: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--session" {
            session = args.get(i + 1).cloned();
            i += 2;
        } else {
            positional = args[i].clone();
            i += 1;
        }
    }

    match op {
        "add" => {
            if positional.is_empty() {
                eprintln!("add requires requirement text");
                return 1;
            }
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() % 100_000_000)
                .unwrap_or(0);
            let rid = format!("req-{ms}");
            // Python writes "session": args.session — null when unsupplied.
            let mut ev = json!({"op": "add", "id": rid, "text": positional,
                                "session": session.as_deref()});
            append(&mut ev);
            println!("{rid}");
            0
        }
        "done" => match resolve_id(&positional, session.as_deref()) {
            Some(rid) => {
                let mut ev = json!({"op": "done", "id": rid,
                                    "session": session.as_deref()});
                append(&mut ev);
                println!("resolved {rid}");
                0
            }
            None => {
                eprintln!("no pending item matching '{positional}'");
                1
            }
        },
        "pending" => {
            let items = pending_items(session.as_deref());
            println!("{}", items.len());
            for e in items {
                println!(
                    "  {}: {}",
                    e["id"].as_str().unwrap_or(""),
                    e["text"].as_str().unwrap_or("")
                );
            }
            0
        }
        "list" => {
            for e in pending_items(session.as_deref()) {
                println!(
                    "{} [{}] {}",
                    e["id"].as_str().unwrap_or(""),
                    e["session"].as_str().unwrap_or("-"),
                    e["text"].as_str().unwrap_or("")
                );
            }
            0
        }
        _ => {
            eprintln!("usage: ciel ledger {{add|done|list|pending}} [arg] [--session ID]");
            2
        }
    }
}
