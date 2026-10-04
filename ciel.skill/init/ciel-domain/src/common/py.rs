//! Python runtime surface needed by the ports: `round()` (banker's rounding),
//! `{:,}` integer commas, `datetime.utcnow().isoformat()`, `date.today()`,
//! `os.path.expanduser("~")`, and float repr details.

use std::env;
use std::path::PathBuf;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;

/// Python `round(x, ndigits)` — round-half-to-even on the decimal value.
/// (`f64::round_ties_even` needs rustc 1.77; we stay on 1.75.)
pub fn round_py(x: f64, ndigits: i32) -> f64 {
    if !x.is_finite() {
        return x;
    }
    let scale = 10f64.powi(ndigits);
    let m = x * scale;
    if !m.is_finite() {
        return x;
    }
    let floor = m.floor();
    let diff = m - floor;
    let r = if diff > 0.5 {
        floor + 1.0
    } else if diff < 0.5 {
        floor
    } else {
        // exact half → even
        if (floor as i64) % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    };
    r / scale
}

/// `f"{n:,}"` for integers.
pub fn comma_int(n: impl Into<i64>) -> String {
    let v = n.into();
    let neg = v < 0;
    let digits = v.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

/// `datetime.utcnow().isoformat() + "Z"` — microseconds precision.
pub fn utcnow_iso_z() -> String {
    let now = OffsetDateTime::now_utc();
    now.format(&Iso8601::DEFAULT)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// `datetime.now().strftime("%Y-%m-%d %H:%M:%S")` — local time.
pub fn now_local_hms() -> String {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

/// `date.today()` → (y, m, d) in local time.
pub fn today() -> (i32, u8, u8) {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    (now.year(), now.month() as u8, now.day())
}

/// `date.fromisoformat(s) <= date.today()`; false when unparseable.
pub fn valid_date_not_future(s: &str) -> bool {
    let (sy, sm, sd) = match parse_iso_date(s) {
        Some(v) => v,
        None => return false,
    };
    let (ty, tm, td) = today();
    (sy, sm, sd) <= (ty, tm, td)
}

pub fn parse_iso_date(s: &str) -> Option<(i32, u8, u8)> {
    let mut it = s.split('-');
    let y: i32 = it.next()?.parse().ok()?;
    let m: u8 = it.next()?.parse().ok()?;
    let d: u8 = it.next()?.parse().ok()?;
    if it.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some((y, m, d))
}

/// `os.path.expanduser("~")`.
pub fn home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|_| PathBuf::from("/"))
}

/// `os.path.expanduser` for `~/...` strings.
pub fn expanduser(p: &str) -> PathBuf {
    if p == "~" {
        home_dir()
    } else if let Some(rest) = p.strip_prefix("~/") {
        home_dir().join(rest)
    } else {
        PathBuf::from(p)
    }
}

/// Python `repr()` of a `serde_json::Value` — single-quoted strings,
/// `True`/`False`/`None`, insertion-ordered dicts. Used where Python renders
/// `str(dict)`/`f"{dict}"` into output text.
pub fn repr(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => "None".to_string(),
        serde_json::Value::Bool(b) => if *b { "True" } else { "False" }.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => repr_str(s),
        serde_json::Value::Array(a) => {
            let inner: Vec<String> = a.iter().map(repr).collect();
            format!("[{}]", inner.join(", "))
        }
        serde_json::Value::Object(o) => {
            let inner: Vec<String> = o
                .iter()
                .map(|(k, val)| format!("{}: {}", repr_str(k), repr(val)))
                .collect();
            format!("{{{}}}", inner.join(", "))
        }
    }
}

/// Python `repr()` of a string — prefers single quotes, switches to double
/// quotes when the value contains `'` but not `"`.
fn repr_str(s: &str) -> String {
    let use_double = s.contains('\'') && !s.contains('"');
    let quote = if use_double { '"' } else { '\'' };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || (c as u32) == 0x7f => {
                out.push_str(&format!("\\x{:02x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}

/// Python `os.path.abspath` (no symlink resolution): join cwd + normalize
/// `.`/`..` segments lexically.
pub fn abspath(p: &str) -> String {
    let path = PathBuf::from(p);
    let joined = if path.is_absolute() {
        path
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    };
    let mut parts: Vec<String> = Vec::new();
    for comp in joined.components() {
        use std::path::Component::*;
        match comp {
            RootDir => parts.push("/".to_string()),
            CurDir => {}
            ParentDir => {
                if parts.len() > 1 {
                    parts.pop();
                }
            }
            Normal(s) => parts.push(s.to_string_lossy().into_owned()),
            _ => {}
        }
    }
    if parts.is_empty() {
        "/".to_string()
    } else if parts.len() == 1 {
        parts[0].clone()
    } else {
        let mut s = parts[0].clone(); // "/"
        for p in &parts[1..] {
            if !s.ends_with('/') {
                s.push('/');
            }
            s.push_str(p);
        }
        s
    }
}

/// Python `repr`-style float formatting inside f-strings, e.g. `{x}`:
/// shortest round-trip, `4.0`-style integral floats keep `.0`.
pub fn py_float(x: f64) -> String {
    if !x.is_finite() {
        return if x.is_nan() {
            "nan".into()
        } else if x > 0.0 {
            "inf".into()
        } else {
            "-inf".into()
        };
    }
    let s = format!("{}", x);
    if !s.contains('.') && !s.contains('e') && !s.contains("inf") && !s.contains("nan") {
        format!("{}.0", s)
    } else {
        s
    }
}

/// Python `str(int)` for the `{x:,}` and numeric sites where ints must not
/// carry a trailing `.0` — used when emitting JSON with `json!` we pass
/// serde the right numeric type instead; this is for text interpolation.
pub fn py_num(x: f64) -> String {
    if x.fract() == 0.0 && x.abs() < 9.0e15 {
        format!("{}", x as i64)
    } else {
        py_float(x)
    }
}
