//! Port of `skills/devin-conversation-recovery/scripts/find_devin_convo.py` —
//! locate and extract Devin (ACP) conversations from local Windsurf/Devin
//! storage. Two-phase search: titles live in
//! `User/globalStorage/state.vscdb`, message bodies in
//! `User/acp-messages/<uuid>.db`.

use crate::common::cli::{parse, ArgSpec, Args};
use crate::common::jsonfmt;
use crate::common::py;
use rusqlite::Connection;
use serde_json::Value;
use std::path::PathBuf;

fn base_dir() -> PathBuf {
    py::expanduser("~/Library/Application Support/Devin")
}

fn global_db() -> PathBuf {
    base_dir().join("User/globalStorage/state.vscdb")
}

fn msgs_dir() -> PathBuf {
    base_dir().join("User/acp-messages")
}

pub struct Session {
    pub session: String,
    pub uuid: String,
    pub title: String,
    pub updated: String,
    pub db: String,
}

pub fn list_sessions() -> Result<Vec<Session>, String> {
    let con = Connection::open(global_db()).map_err(|e| e.to_string())?;
    let mut stmt = con
        .prepare(
            "select key,value from ItemTable where key like 'windsurf.acp.sessioninfo.session.%'",
        )
        .map_err(|e| e.to_string())?;
    let rows: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let idx_raw: String = con
        .query_row(
            "select value from ItemTable where key='windsurf.acp.messageStore.index'",
            [],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;
    let idx: Value = serde_json::from_str(&idx_raw).map_err(|e| e.to_string())?;

    let mut out: Vec<Session> = Vec::new();
    for (_, v) in rows {
        let parsed: Value = match serde_json::from_str(&v) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let info = match parsed.get("info").and_then(Value::as_object) {
            Some(i) => i,
            None => continue,
        };
        let session_id = info.get("sessionId").and_then(Value::as_str).unwrap_or("");
        let sid = session_id.rsplit('/').next().unwrap_or("").to_string();
        let uuid = idx
            .get(session_id)
            .and_then(|e| e.get("uuid"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        out.push(Session {
            session: sid,
            db: if uuid.is_empty() {
                String::new()
            } else {
                msgs_dir()
                    .join(format!("{}.db", uuid))
                    .to_string_lossy()
                    .into_owned()
            },
            uuid,
            title: info
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            updated: info
                .get("updatedAt")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        });
    }
    out.sort_by(|a, b| b.updated.cmp(&a.updated));
    Ok(out)
}

fn take_chars(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

pub fn matches(sess: &Session, needle: &str, deep: bool) -> bool {
    let n = needle.to_lowercase();
    if sess.title.to_lowercase().contains(&n) {
        return true;
    }
    if sess.db.is_empty() || !PathBuf::from(&sess.db).exists() {
        return false;
    }
    let con = match Connection::open(&sess.db) {
        Ok(c) => c,
        Err(_) => return false,
    };
    if deep {
        if let Ok(mut stmt) = con.prepare("select payload from messages") {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                for payload in rows.flatten() {
                    if payload.to_lowercase().contains(&n) {
                        return true;
                    }
                }
            }
        }
        false
    } else {
        // cheap phase: first 5 payloads only
        if let Ok(mut stmt) = con.prepare("select payload from messages order by position limit 5")
        {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                for payload in rows.flatten() {
                    if payload.to_lowercase().contains(&n) {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub fn extract(uuid: &str) -> Result<String, String> {
    let db = msgs_dir().join(format!("{}.db", uuid));
    let con = Connection::open(&db).map_err(|e| e.to_string())?;
    let mut stmt = con
        .prepare("select position,kind,payload from messages order by position")
        .map_err(|e| e.to_string())?;
    let rows: Vec<(i64, String, String)> = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    let mut out: Vec<String> = Vec::new();
    for (p, k, payload) in rows {
        let t = text_of(&payload);
        let t = t.trim();
        if !t.is_empty() {
            out.push(format!("=== [{}] {}\n{}", p, k, t));
        }
    }
    Ok(out.join("\n\n"))
}

/// `text_of(payload)` — merge streaming text chunks with no separator,
/// render rawInput commands as `$ cmd  (exit N)` lines.
fn text_of(payload: &str) -> String {
    let d: Value = match serde_json::from_str(payload) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    let mut texts: Vec<String> = Vec::new();
    let mut lines: Vec<String> = Vec::new();
    let content = d.get("content").cloned().unwrap_or(Value::Null);
    let items: Vec<Value> = match &content {
        Value::Array(a) => a.clone(),
        other => vec![other.clone()],
    };
    for item in &items {
        let obj = match item.as_object() {
            Some(o) => o,
            None => continue,
        };
        let inner = obj.get("content").cloned().unwrap_or(Value::Null);
        if let Some(map) = inner.as_object() {
            if let Some(t) = map.get("text").and_then(Value::as_str) {
                texts.push(t.to_string());
            }
        } else if let Some(s) = inner.as_str() {
            lines.push(s.to_string());
        }
        if let Some(ri) = obj.get("rawInput") {
            // `ri.get("command") or ri.get("file_path") or json.dumps(ri)[:200]`
            // — Python `or` picks the first *truthy* value.
            fn truthy(v: Option<&Value>) -> Option<&Value> {
                v.filter(|x| match x {
                    Value::Null => false,
                    Value::Bool(b) => *b,
                    Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
                    Value::String(s) => !s.is_empty(),
                    Value::Array(a) => !a.is_empty(),
                    Value::Object(o) => !o.is_empty(),
                })
            }
            let render = |v: &Value| -> String {
                match v {
                    Value::String(s) => s.clone(),
                    other => py::repr(other),
                }
            };
            let cmd = truthy(ri.get("command"))
                .or_else(|| truthy(ri.get("file_path")))
                .map(render)
                .unwrap_or_else(|| take_chars(&jsonfmt::dumps(ri), 200));
            let exit_code = obj
                .get("_meta")
                .and_then(|m| m.get("terminal_exit"))
                .and_then(|t| t.get("exit_code"))
                .filter(|v| !v.is_null());
            let mut line = format!("$ {}", cmd);
            if let Some(ec) = exit_code {
                // Python f"{ec}" — raw value, no quoting.
                let ec_str = match ec {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                line.push_str(&format!("  (exit {})", ec_str));
            }
            if let Some(output) = obj.get("output").and_then(Value::as_str) {
                line.push_str(&format!("\nOUT: {}", take_chars(output, 400)));
            }
            lines.push(line);
        }
    }
    let merged: String = texts.iter().filter(|t| !t.is_empty()).cloned().collect();
    let mut joined = vec![merged];
    joined.extend(lines.iter().filter(|l| !l.trim().is_empty()).cloned());
    joined.join("\n").trim().to_string()
}

/// `find_devin_convo.py __main__` — returns the process exit code.
pub fn run(argv: &[String]) -> i32 {
    let args: Args = parse(
        "ciel-convo",
        argv,
        &[
            ArgSpec::positional("needle"),
            ArgSpec::flag("list", None, "list"),
            ArgSpec::flag("deep", None, "deep"),
            ArgSpec::value("extract", None, "extract"),
            ArgSpec::value("out", Some('o'), "out"),
        ],
    );

    if let Some(uuid) = args.get("extract") {
        match extract(uuid) {
            Ok(txt) => {
                if let Some(out) = args.get("out") {
                    match std::fs::write(out, &txt) {
                        Ok(()) => println!("wrote {} chars -> {}", txt.chars().count(), out),
                        Err(e) => {
                            eprintln!("{}", e);
                            return 1;
                        }
                    }
                } else {
                    println!("{}", take_chars(&txt, 20000));
                }
                return 0;
            }
            Err(e) => {
                eprintln!("{}", e);
                return 1;
            }
        }
    }

    let sessions = match list_sessions() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    let needle = args.get("needle").unwrap_or("");
    if args.flag("list") || needle.is_empty() {
        for s in &sessions {
            let uuid8 = take_chars(&s.uuid, 8);
            let uuid_disp = if uuid8.is_empty() { "-" } else { &uuid8 };
            // f'{updated[:10]}  {session:22s} {uuid8 or "-":9s} {title[:90]}'
            println!(
                "{}  {:<22} {:<9} {}",
                take_chars(&s.updated, 10),
                s.session,
                uuid_disp,
                take_chars(&s.title, 90)
            );
        }
        return 0;
    }

    let hits: Vec<&Session> = sessions
        .iter()
        .filter(|s| matches(s, needle, args.flag("deep")))
        .collect();
    for s in &hits {
        // f'{updated[:10]}  {session:22s} uuid={uuid}  {title[:80]}'
        println!(
            "{}  {:<22} uuid={}  {}",
            take_chars(&s.updated, 10),
            s.session,
            s.uuid,
            take_chars(&s.title, 80)
        );
    }
    if hits.is_empty() {
        eprintln!("no title/early-message hits; retry with --deep");
    }
    0
}
