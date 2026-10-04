//! Python-`json.dumps`-compatible serialization (minimal copy of the
//! ciel-rs helper — same byte contract: `", "`/`": "` separators,
//! ensure_ascii escaping, insertion order, `indent=` support).
use serde_json::Value;

fn escape(s: &str, ascii_only: bool) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if !ascii_only || (c as u32) <= 0x7e => out.push(c),
            c => {
                let n = c as u32;
                if n > 0xffff {
                    let v = n - 0x10000;
                    out.push_str(&format!(
                        "\\u{:04x}\\u{:04x}",
                        0xd800 + (v >> 10),
                        0xdc00 + (v & 0x3ff)
                    ));
                } else {
                    out.push_str(&format!("\\u{:04x}", n));
                }
            }
        }
    }
    out
}

fn inner(v: &Value, sorted: bool, ascii: bool) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", escape(s, ascii)),
        Value::Array(a) => {
            let items: Vec<String> = a.iter().map(|x| inner(x, sorted, ascii)).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            if sorted {
                keys.sort();
            }
            let items: Vec<String> = keys
                .into_iter()
                .map(|k| format!("\"{}\": {}", escape(k, ascii), inner(&m[k], sorted, ascii)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

fn inner_indent(v: &Value, indent: usize, level: usize, ascii: bool, sorted: bool) -> String {
    let pad = " ".repeat(indent * (level + 1));
    let close_pad = " ".repeat(indent * level);
    match v {
        Value::Array(a) if !a.is_empty() => {
            let items: Vec<String> = a
                .iter()
                .map(|x| format!("{pad}{}", inner_indent(x, indent, level + 1, ascii, sorted)))
                .collect();
            format!("[\n{}\n{close_pad}]", items.join(",\n"))
        }
        Value::Object(m) if !m.is_empty() => {
            let mut keys: Vec<&String> = m.keys().collect();
            if sorted {
                keys.sort();
            }
            let items: Vec<String> = keys
                .into_iter()
                .map(|k| {
                    format!(
                        "{pad}\"{}\": {}",
                        escape(k, ascii),
                        inner_indent(&m[k], indent, level + 1, ascii, sorted)
                    )
                })
                .collect();
            format!("{{\n{}\n{close_pad}}}", items.join(",\n"))
        }
        other => inner(other, sorted, ascii),
    }
}

/// `json.dumps(v)` — Python defaults: `", "`/`": "` separators, ensure_ascii.
#[allow(dead_code)]
pub fn dumps(v: &Value) -> String {
    inner(v, false, true)
}

/// `json.dumps(v, ensure_ascii=False)` — raw UTF-8, spaced separators.
pub fn dumps_raw(v: &Value) -> String {
    inner(v, false, false)
}

/// `json.dumps(v, sort_keys=True)` — canonical digest form.
pub fn dumps_sorted(v: &Value) -> String {
    inner(v, true, true)
}

/// `json.dumps(v, indent=n, ensure_ascii=False)` — artifact form.
#[allow(dead_code)]
pub fn dumps_indent(v: &Value, indent: usize) -> String {
    inner_indent(v, indent, 0, false, false)
}

/// `json.dump(v, indent=n)` with Python's default `ensure_ascii=True`.
pub fn dumps_indent_ascii(v: &Value, indent: usize) -> String {
    inner_indent(v, indent, 0, true, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dumps_python_separators() {
        let v = json!({"a": 1, "b": [true, null], "c": {"d": "x"}});
        assert_eq!(
            dumps(&v),
            "{\"a\": 1, \"b\": [true, null], \"c\": {\"d\": \"x\"}}"
        );
    }

    #[test]
    fn dumps_ensure_ascii_escapes() {
        // é, DEL (0x7f is outside Python's [\x20-\x7e] printable range),
        // and an astral char → surrogate pair.
        let v = json!({"k": "é\u{7f}😀"});
        assert_eq!(dumps(&v), "{\"k\": \"\\u00e9\\u007f\\ud83d\\ude00\"}");
        assert_eq!(dumps_raw(&v), "{\"k\": \"é\u{7f}😀\"}");
    }

    #[test]
    fn dumps_escapes_controls_and_quotes() {
        let v = json!("a\"b\\c\nd\te\u{1}");
        assert_eq!(dumps(&v), "\"a\\\"b\\\\c\\nd\\te\\u0001\"");
    }

    #[test]
    fn dumps_sorted_orders_keys() {
        let v = json!({"b": 1, "a": 2});
        assert_eq!(dumps_sorted(&v), "{\"a\": 2, \"b\": 1}");
    }

    #[test]
    fn dumps_indent_shape() {
        let v = json!({"x": [1, 2], "y": {}});
        assert_eq!(
            dumps_indent_ascii(&v, 1),
            "{\n \"x\": [\n  1,\n  2\n ],\n \"y\": {}\n}"
        );
    }

    #[test]
    fn dumps_scalar_passthrough() {
        assert_eq!(dumps(&json!(null)), "null");
        assert_eq!(dumps(&json!(2.5)), "2.5");
        assert_eq!(dumps(&json!("")), "\"\"");
    }
}
