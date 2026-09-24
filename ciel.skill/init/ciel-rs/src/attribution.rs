//! Durable-artifact attribution scan — Rust port of
//! `hooks/lib/attribution_scan.py`. Fires on publish-ish commands via the
//! `publish_artifact_scan` advisory rule (`scan: attribution`).

use regex::Regex;
use serde_json::{json, Value};
use std::env;
use std::process::Command;
use wait_timeout::ChildExt;

use crate::paths;

const BYPASS_ENV: &str = "CIEL_ATTRIBUTION_SKIP";
const DEFAULT_MODE: &str = "shadow";

fn patterns() -> Vec<(&'static str, Regex)> {
    vec![
        (
            "attribution_trailer",
            Regex::new(r"(?i)generated\s+with|co-authored-by\s*:").unwrap(),
        ),
        (
            "internal_identity",
            Regex::new("(?i)«(?:Answer|Report|Notice|Council[^»]*)»|council\\s+of\\s+five")
                .unwrap(),
        ),
        (
            "ai_emoji",
            Regex::new("[\u{1F300}-\u{1FAFF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]").unwrap(),
        ),
        ("host_runtime", Regex::new(r"(?i)\bdevin\b").unwrap()),
    ]
}

fn gate_mode() -> String {
    if let Ok(mode) = env::var("CIEL_ATTRIBUTION_GATE") {
        let m = mode.trim().to_lowercase();
        if m == "shadow" || m == "enforce" {
            return m;
        }
    }
    if let Ok(text) =
        std::fs::read_to_string(paths::ciel_home().join("risk").join("attribution_gate"))
    {
        let m = text.trim().to_lowercase();
        if m == "shadow" || m == "enforce" {
            return m;
        }
    }
    DEFAULT_MODE.to_string()
}

fn allowlist() -> Vec<Regex> {
    let mut out = Vec::new();
    if let Ok(text) = std::fs::read_to_string(
        paths::ciel_home()
            .join("risk")
            .join("attribution_allowlist.txt"),
    ) {
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Ok(rx) = Regex::new(&format!("(?i){line}")) {
                out.push(rx);
            }
        }
    }
    out
}

fn git(args: &[&str]) -> String {
    // Python: subprocess.run(..., timeout=10) drains stdout concurrently via
    // communicate(). A piped child writing >64KiB deadlocks if we wait before
    // reading, so drain on a reader thread while wait_timeout bounds the wait.
    let mut child = match Command::new("git")
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    use std::io::Read;
    let reader = child.stdout.take().map(|mut so| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            let _ = so.read_to_end(&mut buf);
            buf
        })
    });
    match child.wait_timeout(std::time::Duration::from_secs(10)) {
        Ok(Some(s)) if s.success() => reader
            .and_then(|r| r.join().ok())
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default(),
        _ => {
            let _ = child.kill();
            let _ = child.wait();
            if let Some(r) = reader {
                let _ = r.join();
            }
            String::new()
        }
    }
}

/// Return {source: [lines]} to scan for the given publish command.
fn collect_text(command: &str) -> Vec<(String, Vec<String>)> {
    let mut sources: Vec<(String, Vec<String>)> = vec![(
        "command".to_string(),
        if command.lines().count() > 0 {
            command.lines().map(String::from).collect()
        } else {
            vec![command.to_string()]
        },
    )];

    if Regex::new(r"\bgit\s+commit\b").unwrap().is_match(command) {
        let diff = git(&["diff", "--cached", "--unified=0"]);
        let added: Vec<String> = diff
            .lines()
            .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
            .map(|l| l[1..].to_string())
            .collect();
        if !added.is_empty() {
            sources.push(("staged_diff".to_string(), added));
        }
    }

    if Regex::new(r"\bgit\s+(push|tag)\b")
        .unwrap()
        .is_match(command)
    {
        let log = git(&["log", "--format=%B%x00", "@{u}..HEAD"]);
        if !log.is_empty() {
            sources.push((
                "unpushed_messages".to_string(),
                log.split('\0')
                    .filter(|l| !l.trim().is_empty())
                    .map(String::from)
                    .collect(),
            ));
        }
    }
    sources
}

/// Mirror of `attribution_scan.scan` — {result, findings, mode}.
pub fn scan(command: &str) -> Value {
    // Python: `os.environ.get(BYPASS_ENV)` — truthy, so an empty value does
    // NOT bypass; set-but-empty must behave the same.
    let bypass_env = env::var(BYPASS_ENV).map(|v| !v.is_empty()).unwrap_or(false);
    if command.contains(BYPASS_ENV) || bypass_env {
        return json!({"result": "bypass", "findings": [], "mode": gate_mode()});
    }
    let allow = allowlist();
    let pats = patterns();
    let mut findings = Vec::new();
    for (source, lines) in collect_text(command) {
        for line in &lines {
            for (category, rx) in &pats {
                if rx.is_match(line) && !allow.iter().any(|a| a.is_match(line)) {
                    findings.push(json!({
                        "category": category,
                        "source": source,
                        "line": line.trim().chars().take(120).collect::<String>(),
                    }));
                    break;
                }
            }
        }
    }
    json!({
        "result": if findings.is_empty() { "clean" } else { "flagged" },
        "findings": findings.into_iter().take(25).collect::<Vec<_>>(),
        "mode": gate_mode(),
    })
}

/// `ciel attribution-scan` — stdin command → result JSON.
pub fn main_() -> i32 {
    let mut buf = String::new();
    use std::io::Read;
    let _ = std::io::stdin().read_to_string(&mut buf);
    // Python: print(json.dumps(scan(command), ensure_ascii=False)).
    println!("{}", crate::jsonfmt::dumps_raw(&scan(&buf)));
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailer_flagged() {
        let r = scan("git commit -m 'x' -m 'Generated with AI'");
        assert_eq!("flagged", r["result"]);
    }

    #[test]
    fn clean_commit() {
        let r = scan("git commit -m 'ordinary change'");
        assert_eq!("clean", r["result"]);
    }
}
