//! Deterministic secret scrubbing — port of `system1_export.scrub_secrets`
//! (which wraps `secret_scan.PATTERNS`). Shared by `export` and `rlcd`.

use regex::Regex;
use serde_json::Value;
use std::sync::OnceLock;

fn patterns() -> &'static Vec<(String, Regex)> {
    static PATTERNS: OnceLock<Vec<(String, Regex)>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
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

/// Recursively redact secrets from state fields — `[REDACTED_SECRET:CAT]`.
pub fn scrub_secrets(val: &Value) -> Value {
    match val {
        Value::String(s) => {
            let mut out = s.clone();
            for (cat, rx) in patterns() {
                out = rx
                    .replace_all(&out, format!("[REDACTED_SECRET:{}]", cat.to_uppercase()))
                    .into_owned();
            }
            Value::String(out)
        }
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), scrub_secrets(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(scrub_secrets).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn scrubs_prefixed_api_key() {
        let v = json!("key sk-abcdefghijklmnopqrstuvwxyz1234 tail");
        let out = scrub_secrets(&v);
        assert_eq!(out, json!("key [REDACTED_SECRET:API_KEY_PREFIXED] tail"));
    }

    #[test]
    fn scrubs_github_and_aws() {
        let gh = format!("token ghp_{}", "a".repeat(20));
        assert!(scrub_secrets(&json!(gh))
            .as_str()
            .unwrap()
            .contains("[REDACTED_SECRET:GITHUB_TOKEN]"));
        let aws = format!("key AKIA{}", "1".repeat(16));
        assert!(scrub_secrets(&json!(aws))
            .as_str()
            .unwrap()
            .contains("[REDACTED_SECRET:AWS_ACCESS_KEY]"));
    }

    #[test]
    fn scrubs_nested_maps_and_lists() {
        let v = json!({
            "deep": {"cmd": "password = hunter2ok"},
            "list": ["xoxb-1234567890ab", "clean"]
        });
        let out = scrub_secrets(&v);
        let s = serde_json::to_string(&out).unwrap();
        assert!(s.contains("REDACTED_SECRET"));
        assert!(!s.contains("hunter2"));
        assert!(s.contains("clean"));
    }

    #[test]
    fn non_secrets_pass_through() {
        let v = json!({"n": 42, "s": "rm -rf /tmp/x", "b": true});
        assert_eq!(scrub_secrets(&v), v);
        assert_eq!(scrub_secrets(&json!(null)), json!(null));
    }
}
