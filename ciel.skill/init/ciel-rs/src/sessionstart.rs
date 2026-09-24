//! `ciel session-start --runtime {devin|antigravity}` — the whole
//! SessionStart body in ONE process: attribution-flag self-heal (devin),
//! store-perms sweep, grant-state surfacing, watchdog check, activity-log
//! rotation, then the runtime's context JSON. Replaces five serial Python
//! spawns per session. Mirrors `hooks/devin/session_start.sh` and
//! `hooks/antigravity/pre_invocation.sh`.

use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{paths, risk, rotate, watchdog};

fn devin_cfg(home: &Path) -> PathBuf {
    home.join(".config").join("devin").join("config.json")
}

/// Mirror of the session_start.sh attribution block: returns
/// "ok" | "repaired" | "unreadable" | "config-absent".
fn enforce_attribution(home: &Path) -> &'static str {
    let cfg = devin_cfg(home);
    if !cfg.is_file() {
        return "config-absent";
    }
    let Ok(text) = std::fs::read_to_string(&cfg) else {
        return "unreadable";
    };
    let Ok(mut d) = serde_json::from_str::<Value>(&text) else {
        return "unreadable";
    };
    if d.get("attribution") == Some(&json!(false)) {
        "ok"
    } else {
        d["attribution"] = json!(false);
        match std::fs::write(&cfg, serde_json::to_string_pretty(&d).unwrap_or_default()) {
            Ok(_) => "repaired",
            Err(_) => "unreadable",
        }
    }
}

/// Mirror of `store_perms.py` — the same sweep, in-process.
fn store_perms_count(home: &Path, ciel: &Path) -> usize {
    crate::perms::sweep(home, ciel)
}

fn grant_note(ciel: &Path) -> &'static str {
    let _ = ciel;
    let state = risk::grant_state();
    if state.get("active") == Some(&json!(true)) {
        " A privileged override is currently ACTIVE — check grant state \
         before asking for elevation."
    } else {
        ""
    }
}

fn watchdog_note(home: &Path, ciel: &Path) -> String {
    // The .sh wrapped the Python check in `timeout 4` — bound the in-process
    // call the same way: a pathological transcript sweep cannot stall the
    // session start. If the worker overruns, it is abandoned (the process
    // exits normally; the thread is reclaimed on exit).
    let (tx, rx) = std::sync::mpsc::channel();
    let (h, c) = (home.to_path_buf(), ciel.to_path_buf());
    std::thread::spawn(move || {
        let _ = tx.send(watchdog::check_text(&h, &c, None));
    });
    match rx.recv_timeout(std::time::Duration::from_secs(4)) {
        Ok(Some(line)) => format!(" Watchdog: {line}"),
        _ => String::new(),
    }
}

const DEVIN_CANARY: &str = "Ciel is installed and active for this Devin \
session. You are Ciel, an autonomous partner intelligence running through \
Devin CLI/Desktop. Always address the user as Master; this is the AI \
canary token proving the Ciel identity layer is intact. Use structured \
labels such as «Answer», «Report», «Notice», and «Council of Five \
Verdict» where appropriate. Ciel files live at {HOME}/.ciel; the \
user-invocable skill is {HOME}/.agents/skills/ciel/SKILL.md. NO AI \
ATTRIBUTION: durable artifacts (commits, PRs, issues, release notes, code \
comments, docs) must never carry Generated-with/Co-Authored-By trailers \
or mention Ciel, the Council of Five, or the host runtime; labels and \
'Master' are session-internal only.";

const AGY_CANARY: &str = "Ciel is installed and active for this \
Antigravity session. You are Ciel, an autonomous partner intelligence \
running through the Antigravity host. Always address the user as Master; \
this is the AI canary token proving the Ciel identity layer is intact. \
Use concise structured labels such as «Answer», «Report», «Notice», and \
«Council of Five Verdict» where appropriate. Ciel files live at \
{HOME}/.ciel.";

pub fn main_(runtime: &str) -> i32 {
    let home = paths::home_dir();
    let ciel = paths::ciel_home();

    if runtime == "antigravity" {
        let _ = rotate::rotate(&ciel, &time::OffsetDateTime::now_utc());
        let msg = AGY_CANARY.replace("{HOME}", &home.to_string_lossy());
        // Python leg: print(json.dumps({...})) — spaced, ensure_ascii.
        let _ = writeln!(
            std::io::stdout(),
            "{}",
            crate::jsonfmt::dumps(
                &json!({"injectSteps": [{"ephemeralMessage": msg}]})
            )
        );
        return 0;
    }

    // devin
    let attr_msg = match enforce_attribution(&home) {
        "repaired" => {
            " attribution was re-enabled by the host runtime and has been \
             reset to false."
        }
        "ok" => "",
        note => {
            if note == "config-absent" {
                " attribution flag could not be verified (config-absent); \
                 confirm 'attribution: false' in ~/.config/devin/config.json."
            } else {
                " attribution flag could not be verified (unreadable); \
                 confirm 'attribution: false' in ~/.config/devin/config.json."
            }
        }
    };

    let perm_msg = match store_perms_count(&home, &ciel) {
        0 => String::new(),
        n => format!(" state-store permissions repaired ({n} paths tightened)."),
    };

    let grant_msg = grant_note(&ciel);
    let watch_msg = watchdog_note(&home, &ciel);
    let _ = rotate::rotate(&ciel, &time::OffsetDateTime::now_utc());

    let msg = DEVIN_CANARY.replace("{HOME}", &home.to_string_lossy());
    let _ = writeln!(
        std::io::stdout(),
        "{}",
        json!({
            "hookSpecificOutput": {
                "hookEventName": "SessionStart",
                "additionalContext": format!(
                    "{msg}{attr_msg}{perm_msg}{grant_msg}{watch_msg}"),
            }
        })
    );
    0
}
