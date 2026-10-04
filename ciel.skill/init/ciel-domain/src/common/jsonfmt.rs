//! Python-`json.dumps`-compatible serialization. The Python engine's output
//! contracts depend on its defaults — `", "`/`": "` separators, ensure_ascii
//! escaping, insertion order — and several state files use `indent=` or
//! `ensure_ascii=False`. Every site that emits JSON consumed by runtimes,
//! greps, or the Python twin routes through here so the byte contract holds.
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
            c if !ascii_only || (c as u32) < 0x7f => out.push(c),
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

fn obj_keys(m: &serde_json::Map<String, Value>, sorted: bool) -> Vec<&String> {
    let mut keys: Vec<&String> = m.keys().collect();
    if sorted {
        keys.sort();
    }
    keys
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
            let items: Vec<String> = obj_keys(m, sorted)
                .into_iter()
                .map(|k| format!("\"{}\": {}", escape(k, ascii), inner(&m[k], sorted, ascii)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
    }
}

fn inner_indent(v: &Value, indent: usize, level: usize, ascii: bool) -> String {
    let pad = " ".repeat(indent * (level + 1));
    let close_pad = " ".repeat(indent * level);
    match v {
        Value::Array(a) if !a.is_empty() => {
            let items: Vec<String> = a
                .iter()
                .map(|x| format!("{pad}{}", inner_indent(x, indent, level + 1, ascii)))
                .collect();
            format!("[\n{}\n{close_pad}]", items.join(",\n"))
        }
        Value::Object(m) if !m.is_empty() => {
            let items: Vec<String> = m
                .keys()
                .map(|k| {
                    format!(
                        "{pad}\"{}\": {}",
                        escape(k, ascii),
                        inner_indent(&m[k], indent, level + 1, ascii)
                    )
                })
                .collect();
            format!("{{\n{}\n{close_pad}}}", items.join(",\n"))
        }
        other => inner(other, false, ascii),
    }
}

fn inner_compact(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!("\"{}\"", escape(s, true)),
        Value::Array(a) => {
            let items: Vec<String> = a.iter().map(inner_compact).collect();
            format!("[{}]", items.join(","))
        }
        Value::Object(m) => {
            let items: Vec<String> = m
                .keys()
                .map(|k| format!("\"{}\":{}", escape(k, true), inner_compact(&m[k])))
                .collect();
            format!("{{{}}}", items.join(","))
        }
    }
}

/// `json.dumps(v)` — Python defaults: `", "`/`": "` separators, ensure_ascii.
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

/// `json.dumps(v, indent=n, ensure_ascii=False)` — state-file form.
pub fn dumps_indent(v: &Value, indent: usize) -> String {
    inner_indent(v, indent, 0, false)
}

/// `json.dump(v, indent=n)` with Python's default `ensure_ascii=True`.
pub fn dumps_indent_ascii(v: &Value, indent: usize) -> String {
    inner_indent(v, indent, 0, true)
}

/// `json.dumps(v, separators=(',', ':'))` — token-efficient compact form.
pub fn dumps_compact(v: &Value) -> String {
    inner_compact(v)
}
