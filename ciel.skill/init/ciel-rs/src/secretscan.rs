//! Deterministic secret-in-text scanner — Rust port of
//! `hooks/lib/secret_scan.py`. Local only; reports categories, never content.

use regex::Regex;
use serde_json::{json, Value};

pub const PATTERNS: [(&str, &str); 12] = [
    (
        "github_token",
        r"\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}\b|\bgithub_pat_[A-Za-z0-9_]{20,}\b",
    ),
    ("aws_access_key", r"\bAKIA[0-9A-Z]{16}\b"),
    (
        "api_key_prefixed",
        r"\b(?:sk|pk|key|api|tok)_[A-Za-z0-9_-]{20,}\b|\bsk-[A-Za-z0-9_-]{20,}\b",
    ),
    ("private_key_block", r"-----BEGIN [A-Z ]*PRIVATE KEY"),
    ("slack_token", r"\bxox[baprs]-[0-9A-Za-z-]{10,}\b"),
    ("gcp_api_key", r"\bAIza[0-9A-Za-z_-]{35}\b"),
    (
        "jwt",
        r"\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b",
    ),
    ("npm_token", r"\bnpm_[A-Za-z0-9]{36}\b"),
    ("crates_token", r"\bcio[0-9A-Za-z]{25,}\b"),
    (
        "password_assignment",
        r#"(?i)\b(?:sudo\s+)?(?:password|passwd|passphrase)\s*(?:is|:|=)\s*['\"]?[^\s'\"]{4,}"#,
    ),
    (
        "secret_assignment",
        r#"(?i)\b(?:api[_-]?key|access[_-]?token|auth[_-]?token|secret[_-]?key|client[_-]?secret)\s*[:=]\s*['\"]?[A-Za-z0-9_\-]{8,}"#,
    ),
    (
        "generic_secret_kv",
        r#"(?i)\b(?:token|secret)\s+is\s+['\"]?[A-Za-z0-9_\-]{8,}"#,
    ),
];

/// `{"hits": N, "categories": [...]}` — categories sorted like the Python.
pub fn scan(text: &str) -> Value {
    let mut categories: Vec<&str> = PATTERNS
        .iter()
        .filter(|(_, p)| Regex::new(p).is_ok_and(|rx| rx.is_match(text)))
        .map(|(name, _)| *name)
        .collect();
    categories.sort_unstable();
    json!({"hits": categories.len(), "categories": categories})
}

/// `ciel secret-scan` — stdin raw text → result JSON.
pub fn main_() -> i32 {
    let mut buf = String::new();
    use std::io::Read;
    let _ = std::io::stdin().read_to_string(&mut buf);
    // Python: print(json.dumps(scan(text))) — spaced, ensure_ascii.
    println!("{}", crate::jsonfmt::dumps(&scan(&buf)));
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_categories_not_content() {
        let r = scan("my token is ghp_abcdefghij0123456789ABCD ok");
        assert!(r["categories"]
            .as_array()
            .unwrap()
            .contains(&json!("github_token")));
        let r = scan("nothing here");
        assert_eq!(0, r["hits"]);
    }
}
