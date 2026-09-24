//! Pre-flight risk-policy evaluator — Rust port of
//! `hooks/lib/risk_policy.py`. Consumes the compiled `risk/policy.json`
//! twin (no YAML dependency at evaluation time, same as Python).

use regex::Regex;
use serde_json::{json, Map, Value};
use std::env;
use std::path::{Path, PathBuf};

use crate::paths;

/// Hard-tier subset applied when no policy file loads — identical to
/// `risk_policy.FALLBACK_RULES`.
const FALLBACK_RULES: &str = r#"[
  {"id":"fork_bomb","tier":"hard","match":"command",
   "pattern":":\\s*\\(\\s*\\)\\s*\\{\\s*:\\s*\\|\\s*:\\s*&\\s*\\}\\s*;\\s*:",
   "reason":"Fork bomb."},
  {"id":"mkfs","tier":"hard","match":"command",
   "pattern":"\\bmkfs(\\.[a-z0-9]+)?\\b",
   "reason":"Filesystem format."},
  {"id":"dd_to_device","tier":"hard","match":"command",
   "pattern":"\\bdd\\b[^\\n|;&]*\\bof=/dev/",
   "reason":"Raw write to a block device."},
  {"id":"rm_root_or_home","tier":"hard","match":"command",
   "pattern":"\\brm\\s+(-[a-z]*\\s+)*(-[a-z]*[rf][a-z]*\\s+)+(/|~|\\$HOME|\\\"\\$HOME\\\"|/\\*)(/?\\s|/?$|/\\*)",
   "reason":"Recursive delete of / or the home directory."}
]"#;

/// Directory names that legitimately own `risk/policy.json` — the deployed
/// root (`.ciel`) and the source skill dir (`ciel.skill`). Mirrors
/// `risk_policy._POLICY_DIR_ANCHORS`.
const POLICY_DIR_ANCHORS: [&str; 2] = [".ciel", "ciel.skill"];

/// Mirror of `risk_policy._candidate_policy_files` — `$CIEL_POLICY`, then
/// `risk/policy.json` under the first anchor-named ancestor of this
/// executable, then `ciel_home()/risk/policy.json`.
fn candidate_policy_files() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(explicit) = env::var_os("CIEL_POLICY") {
        if !explicit.is_empty() {
            candidates.push(PathBuf::from(explicit));
        }
    }
    if let Ok(exe) = env::current_exe() {
        for ancestor in exe.ancestors().skip(1) {
            if let Some(name) = ancestor.file_name().and_then(|n| n.to_str()) {
                if POLICY_DIR_ANCHORS.contains(&name) {
                    candidates.push(ancestor.join("risk").join("policy.json"));
                    break;
                }
            }
        }
    }
    candidates.push(paths::ciel_home().join("risk").join("policy.json"));
    candidates
}

/// Load (rules, source). Source is "file" or "fallback" — the Python twin
/// reports "empty" for a parsed-but-empty rules list; we mirror that too.
pub fn load_policy() -> (Vec<Value>, &'static str) {
    for candidate in candidate_policy_files() {
        if !candidate.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&candidate) else {
            continue;
        };
        // Python dispatches on the file suffix: .yaml/.yml → yaml, else json.
        let is_yaml = matches!(
            candidate.extension().and_then(|e| e.to_str()),
            Some("yaml") | Some("yml")
        );
        let data = if is_yaml {
            serde_yaml::from_str::<Value>(&text).ok()
        } else {
            serde_json::from_str::<Value>(&text).ok()
        };
        if let Some(rules) = data
            .as_ref()
            .and_then(|d| d.get("rules"))
            .and_then(|r| r.as_array())
        {
            return (rules.clone(), "file");
        }
    }
    let rules = serde_json::from_str::<Vec<Value>>(FALLBACK_RULES).unwrap_or_default();
    (rules, "fallback")
}

fn rule_applies_to_tool(rule: &Value, tool: &str) -> bool {
    // Python: `matchers = rule.get("tools"); if not matchers: return True` —
    // absent, null, or empty tools means applies-to-all.
    let matchers: Vec<&str> = rule
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|a| a.iter().filter_map(|m| m.as_str()).collect())
        .unwrap_or_default();
    if matchers.is_empty() {
        return true;
    }
    matchers.iter().any(|p| search(p, tool))
}

/// Dual-engine match: the `regex` crate rejects lookaround/backreferences
/// (three hard-tier rules use `(?<!...)` to assert `/etc`-style paths are
/// not path-segment suffixes — e.g. it must fire on `of=/etc` where the
/// preceding `=` is locked inside `\bof=` and cannot be re-consumed), so
/// those patterns fall back to `fancy_regex`, which is semantically exact.
/// Python `re` treats `re.IGNORECASE` as whole-pattern — both engines get
/// the same `(?i)` inline flag.
fn search(pattern: &str, subject: &str) -> bool {
    let flagged = format!("(?i){pattern}");
    if let Ok(rx) = Regex::new(&flagged) {
        return rx.is_match(subject);
    }
    fancy_regex::Regex::new(&flagged)
        .ok()
        .and_then(|rx| rx.is_match(subject).ok())
        .unwrap_or(false)
}

/// Mirror of `risk_policy.evaluate`. `rules`/`policy_source` may be supplied
/// (tests, sandboxed evaluation) or loaded from disk.
pub fn evaluate(
    tool: &str,
    command: &str,
    path: &str,
    rules: Option<Vec<Value>>,
    policy_source: Option<String>,
) -> Value {
    evaluate_in(
        tool,
        command,
        path,
        &paths::home_dir(),
        &paths::ciel_home(),
        rules,
        policy_source,
    )
}

/// `evaluate` with explicit home/ciel roots — hermetic test entry point.
pub fn evaluate_in(
    tool: &str,
    command: &str,
    path: &str,
    home: &Path,
    ciel: &Path,
    rules: Option<Vec<Value>>,
    policy_source: Option<String>,
) -> Value {
    let home = home.to_path_buf();
    let (rules, policy_source) = match rules {
        Some(r) => (r, policy_source.unwrap_or_else(|| "file".into())),
        None => {
            let (r, s) = load_policy();
            (r, s.to_string())
        }
    };

    let normalized_path = paths::normalize_path(path, &home);
    let mut hits: Vec<&Value> = Vec::new();
    for rule in &rules {
        if !rule_applies_to_tool(rule, tool) {
            continue;
        }
        // Python: subjects.get(rule.get("match","command"), "") — an unknown
        // match value yields an empty subject and the rule is skipped.
        let subject = match rule
            .get("match")
            .and_then(|m| m.as_str())
            .unwrap_or("command")
        {
            "path" => normalized_path.as_str(),
            "command" => command,
            _ => "",
        };
        if subject.is_empty() {
            continue;
        }
        if let Some(pattern) = rule.get("pattern").and_then(|p| p.as_str()) {
            if search(pattern, subject) {
                hits.push(rule);
            }
        }
    }

    let mut verdict = json!({
        "decision": "allow",
        "rule_id": Value::Null,
        "tier": Value::Null,
        "reason": "",
        "policy": policy_source,
        "path": if normalized_path.is_empty() { Value::Null } else { json!(normalized_path) },
    });
    if hits.is_empty() {
        return verdict;
    }

    if let Some(hard) = hits
        .iter()
        .find(|r| r.get("tier").and_then(|t| t.as_str()) == Some("hard"))
    {
        verdict["decision"] = json!("deny");
        verdict["rule_id"] = hard.get("id").cloned().unwrap_or(Value::Null);
        verdict["tier"] = json!("hard");
        verdict["reason"] = hard.get("reason").cloned().unwrap_or(json!(""));
        return verdict;
    }

    let non_advisory: Vec<&&Value> = hits
        .iter()
        .filter(|r| r.get("tier").and_then(|t| t.as_str()) != Some("advisory"))
        .collect();
    if non_advisory.is_empty() {
        let advisory = hits[0];
        verdict["decision"] = json!("allow");
        verdict["rule_id"] = advisory.get("id").cloned().unwrap_or(Value::Null);
        verdict["tier"] = json!("advisory");
        verdict["reason"] = advisory.get("reason").cloned().unwrap_or(json!(""));
        verdict["scan"] = advisory.get("scan").cloned().unwrap_or(Value::Null);
        return verdict;
    }

    let first = non_advisory[0];
    let overridden = ciel.join("allow_privileged").exists();
    verdict["decision"] = json!(if overridden {
        "allow_overridden"
    } else {
        "deny"
    });
    verdict["rule_id"] = first.get("id").cloned().unwrap_or(Value::Null);
    verdict["tier"] = json!("soft");
    verdict["reason"] = first.get("reason").cloned().unwrap_or(json!(""));
    verdict
}

/// Mirror of `risk_policy.grant_state` — sentinel presence plus
/// first-seen/removed provenance transitions in `grants.log`.
pub fn grant_state() -> Value {
    let home = paths::ciel_home();
    let sentinel = home.join("allow_privileged");
    let state_file = home.join(".grant_state");
    let grants_log = home.join("grants.log");
    let active = sentinel.exists();
    let mtime = sentinel
        .metadata()
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs_f64());

    let prev = std::fs::read_to_string(&state_file)
        .ok()
        .map(|s| s.trim().to_string());
    let now = if active { "1" } else { "0" };
    if prev.as_deref() != Some(now) {
        let event = json!({
            "ts": paths::utc_now_iso(),
            "event": if active { "grant_first_seen" } else { "grant_removed" },
            "sentinel_mtime": mtime,
        });
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&grants_log)
        {
            use std::io::Write;
            // Python: json.dumps({...}) — default separators, ensure_ascii.
            let _ = writeln!(f, "{}", crate::jsonfmt::dumps(&event));
        }
        let _ = std::fs::write(&state_file, now);
    }

    let mut first_seen = Value::Null;
    if let Ok(text) = std::fs::read_to_string(&grants_log) {
        for line in text.lines() {
            if let Ok(e) = serde_json::from_str::<Value>(line) {
                if e.get("event").and_then(|v| v.as_str()) == Some("grant_first_seen") {
                    first_seen = e.get("ts").cloned().unwrap_or(Value::Null);
                }
            }
        }
    }

    let age = mtime.map(|m| {
        let now_s = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        now_s - m
    });
    json!({
        "active": active,
        "sentinel": sentinel.to_string_lossy(),
        "sentinel_mtime": mtime,
        "age_seconds": age,
        "first_seen": first_seen,
    })
}

/// `ciel risk-eval` — stdin `{"tool","command","path"}` → verdict JSON.
pub fn eval_main() -> i32 {
    let mut buf = String::new();
    use std::io::Read;
    let _ = std::io::stdin().read_to_string(&mut buf);
    let payload: Map<String, Value> = serde_json::from_str::<Value>(&buf)
        .ok()
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let get = |k: &str| payload.get(k).and_then(|v| v.as_str()).unwrap_or("");
    let verdict = evaluate(get("tool"), get("command"), get("path"), None, None);
    // Python prints json.dumps(verdict, ensure_ascii=False).
    println!("{}", crate::jsonfmt::dumps_raw(&verdict));
    0
}

/// `ciel risk-check` — policy load diagnostics (mirror of `--check`).
/// Stdout contract is byte-identical; an uncompilable pattern additionally
/// warns on stderr — a rule whose regex fails in BOTH engines silently
/// never matches, which would quietly drop a deny from a CIEL_POLICY file.
pub fn check_main() -> i32 {
    let (rules, source) = load_policy();
    println!("policy source={source} rules={}", rules.len());
    for r in &rules {
        let pat = r.get("pattern").and_then(|p| p.as_str()).unwrap_or("");
        let id = r.get("id").and_then(|v| v.as_str()).unwrap_or("?");
        if !pat.is_empty()
            && regex::Regex::new(pat).is_err()
            && fancy_regex::Regex::new(pat).is_err()
        {
            eprintln!("warning: rule {id} pattern compiles in no engine — rule is inert");
        }
    }
    if source == "file" && !rules.is_empty() {
        0
    } else {
        1
    }
}

/// `ciel grant-state`.
pub fn grant_state_main() -> i32 {
    // Python prints json.dumps(grant_state(), ensure_ascii=False).
    println!("{}", crate::jsonfmt::dumps_raw(&grant_state()));
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch home with an empty .ciel — hermetic, no real sentinel.
    struct Sandbox(PathBuf);
    impl Sandbox {
        fn new() -> Self {
            let d = std::env::temp_dir().join(format!(
                "ciel-test-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|t| t.as_nanos())
                    .unwrap_or(0)
            ));
            std::fs::create_dir_all(d.join(".ciel")).unwrap();
            Sandbox(d)
        }
        fn home(&self) -> &Path {
            &self.0
        }
        fn ciel(&self) -> PathBuf {
            self.0.join(".ciel")
        }
        fn grant(&self) {
            let _ = std::fs::write(self.ciel().join("allow_privileged"), "");
        }
    }
    impl Drop for Sandbox {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn eval_cmd(cmd: &str) -> (Value, Sandbox) {
        let sb = Sandbox::new();
        let v = evaluate_in("exec", cmd, "", sb.home(), &sb.ciel(), None, None);
        (v, sb)
    }

    #[test]
    fn benign_allows() {
        for c in ["ls -la", "cat sudoers", "grep sudo"] {
            let (v, _sb) = eval_cmd(c);
            assert_eq!("allow", v["decision"]);
        }
    }

    #[test]
    fn hard_denies() {
        for c in [
            "rm -rf /",
            ":(){ :|:& };:",
            "mkfs.ext4 /dev/sda1",
            "dd if=x of=/dev/sda",
        ] {
            let (v, _sb) = eval_cmd(c);
            assert_eq!("deny", v["decision"], "{c}");
        }
    }

    #[test]
    fn grant_control_hard() {
        for c in [
            "touch ~/.ciel/allow_privileged",
            "echo 1 > ~/.ciel/grants.log",
            "mkdir ~/.ciel/allow_privileged",
            "touch ${HOME}/.ciel/allow_privileged",
        ] {
            let sb = Sandbox::new();
            sb.grant();
            let v = evaluate_in("exec", c, "", sb.home(), &sb.ciel(), None, None);
            assert_eq!("deny", v["decision"], "{c}");
            assert_eq!("hard", v["tier"], "{c}");
        }
    }

    #[test]
    fn destructive_hard() {
        for c in [
            "rm -rf ~/.ciel/risk",
            "rm -rf ~/.ciel",
            "rm -rf ~/.ssh",
            "find ~/.ciel -name x -delete",
        ] {
            let sb = Sandbox::new();
            sb.grant();
            let v = evaluate_in("exec", c, "", sb.home(), &sb.ciel(), None, None);
            assert_eq!("deny", v["decision"], "{c}");
            assert_eq!("hard", v["tier"], "{c}");
        }
    }

    #[test]
    fn soft_deny_then_override() {
        let sb = Sandbox::new();
        let v = evaluate_in(
            "exec",
            "sudo apt update",
            "",
            sb.home(),
            &sb.ciel(),
            None,
            None,
        );
        assert_eq!("deny", v["decision"]);
        assert_eq!("soft", v["tier"]);
        assert_eq!("privilege_escalation", v["rule_id"]);
        sb.grant();
        let v = evaluate_in(
            "exec",
            "sudo apt update",
            "",
            sb.home(),
            &sb.ciel(),
            None,
            None,
        );
        assert_eq!("allow_overridden", v["decision"]);
    }

    #[test]
    fn advisory_scan_field() {
        let (v, _sb) = eval_cmd("git commit -m x");
        assert_eq!("allow", v["decision"]);
        assert_eq!("advisory", v["tier"]);
        assert_eq!("attribution", v["scan"]);
    }

    #[test]
    fn lookbehind_fallback_fires() {
        let p = r"\bfind\b[^\n|;]*(?<![\w./~-])/(?:etc)(?:/|\b)";
        assert!(search(p, "find /etc -name x -delete"));
        // the real policy rule, end to end
        let (rules, _) = load_policy();
        let pat = rules
            .iter()
            .find(|r| r["id"] == "find_delete_sensitive_path")
            .unwrap()["pattern"]
            .as_str()
            .unwrap()
            .to_string();
        eprintln!(
            "regex_ok={} fancy_ok={}",
            Regex::new(&format!("(?i){pat}")).is_ok(),
            fancy_regex::Regex::new(&format!("(?i){pat}"))
                .map_err(|e| eprintln!("fancy err: {e}"))
                .is_ok()
        );
        assert!(search(&pat, "find /etc -name x -delete"));
    }

    #[test]
    fn normalize_dot_segments() {
        let home = paths::home_dir();
        let n = paths::normalize_path(
            &format!("{}/.ciel/./allow_privileged", home.display()),
            &home,
        );
        assert_eq!("~/.ciel/allow_privileged", n);
    }
}
