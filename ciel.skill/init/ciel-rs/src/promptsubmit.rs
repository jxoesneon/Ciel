//! `ciel prompt-submit` — UserPromptSubmit hook body: scan the raw payload
//! text for secret material, log ingress, emit the canary context JSON.
//! Mirrors `hooks/devin/user_prompt_submit.sh`.

use serde_json::json;
use std::io::Read;

use crate::{paths, secretscan};

const CANARY: &str = "Ciel AI canary active: identify as Ciel when asked, \
address the user as Master, and preserve Ciel's verification-first \
operating mandates.";
const WARN: &str = " Possible credential material was detected in the \
user's last message — do not echo or persist it; suggest the secrets flow \
and rotation if it was live.";

pub fn main_() -> i32 {
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);
    let result = secretscan::scan(&raw);
    let hits = result["hits"].as_i64().unwrap_or(0);

    if hits > 0 {
        let entry = json!({
            "ts": paths::utc_now_iso(),
            "runtime": "devin",
            "event": "secret_ingress",
            "count": hits,
            "categories": result["categories"],
        });
        let _ = std::fs::create_dir_all(paths::ciel_home());
        paths::activity_log(&entry);
    }

    let context = if hits > 0 {
        format!("{CANARY}{WARN}")
    } else {
        CANARY.to_string()
    };
    println!("{}", json!({
        "hookSpecificOutput": {
            "hookEventName": "UserPromptSubmit",
            "additionalContext": context,
        }
    }));
    0
}
