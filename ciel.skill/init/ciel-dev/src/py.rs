//! Python-runtime surface needed by the ports: `shlex.quote`, `round()`,
//! `os.path.expanduser`, `datetime.isoformat()`, float repr details.

use std::env;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use time::OffsetDateTime;

/// `os.path.expanduser("~")`.
pub fn home_dir() -> PathBuf {
    env::var("HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("USERPROFILE").map(PathBuf::from))
        .unwrap_or_else(|_| PathBuf::from("/"))
}

/// `os.path.expanduser` for `~/...` strings.
#[allow(dead_code)]
pub fn expanduser(p: &str) -> PathBuf {
    if p == "~" {
        home_dir()
    } else if let Some(rest) = p.strip_prefix("~/") {
        home_dir().join(rest)
    } else {
        PathBuf::from(p)
    }
}

/// `Path.home() / ".ciel"` overridden by `CIEL_HOME`.
pub fn ciel_home() -> PathBuf {
    match env::var("CIEL_HOME") {
        Ok(v) if !v.is_empty() => PathBuf::from(v),
        _ => home_dir().join(".ciel"),
    }
}

/// Repo root: `scripts/../` in Python terms. The crate lives at
/// `<repo>/ciel.skill/init/ciel-dev`, so three ancestors up is the repo
/// when built in-tree; fall back to the cwd ancestor carrying `ciel.skill`.
pub fn repo_root() -> PathBuf {
    if let Ok(root) = env::var("CIEL_REPO_ROOT") {
        if !root.is_empty() {
            return PathBuf::from(root);
        }
    }
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    if let Some(root) = manifest.ancestors().nth(3) {
        if root.join("ciel.skill").is_dir() {
            return root.to_path_buf();
        }
    }
    // Walk the cwd for a directory containing ciel.skill — lets an
    // installed binary still anchor on the checkout it runs inside.
    if let Ok(cwd) = env::current_dir() {
        for anc in cwd.ancestors() {
            if anc.join("ciel.skill").is_dir() {
                return anc.to_path_buf();
            }
        }
    }
    manifest
        .ancestors()
        .nth(3)
        .unwrap_or(manifest)
        .to_path_buf()
}

/// Python `round(x, ndigits)` — round-half-to-even on the decimal value.
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
    // Python round-half-to-even.
    let r = if diff < 0.5 || (diff == 0.5 && (floor as i64) % 2 == 0) {
        floor
    } else {
        floor + 1.0
    };
    r / scale
}

/// Python `repr`-style float formatting inside f-strings, e.g. `{x}`:
/// shortest round-trip, integral floats keep `.0`.
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

/// `f"{x:.2f}"`.
#[allow(dead_code)]
pub fn py_f2(x: f64) -> String {
    format!("{:.2}", x)
}

/// `time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())`.
pub fn utc_stamp_z() -> String {
    OffsetDateTime::now_utc()
        .format(&time::macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
        ))
        .unwrap_or_default()
}

fn iso_from_parts(o: &OffsetDateTime, micros: u32) -> String {
    let base = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
        o.year(),
        o.month() as u8,
        o.day(),
        o.hour(),
        o.minute(),
        o.second()
    );
    // timespec="auto": sub-second digits appear only when nonzero.
    if micros == 0 {
        format!("{base}+00:00")
    } else {
        format!("{base}.{micros:06}+00:00")
    }
}

/// `datetime.now(timezone.utc).isoformat()` — `+00:00` suffix, microseconds
/// only when nonzero.
pub fn iso_now_utc() -> String {
    let o = OffsetDateTime::now_utc();
    iso_from_parts(&o, o.microsecond())
}

/// `datetime.fromtimestamp(ts, timezone.utc).isoformat()` for a unix-epoch
/// float (Python rounds the fraction to the nearest microsecond).
pub fn iso_from_unix(ts: f64) -> String {
    let secs = ts.floor() as i64;
    let mut micros = ((ts - secs as f64) * 1e6).round() as i64;
    let mut whole = secs;
    if micros >= 1_000_000 {
        micros -= 1_000_000;
        whole += 1;
    }
    let o = OffsetDateTime::from_unix_timestamp(whole).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    iso_from_parts(&o, micros.max(0) as u32)
}

/// Python `Path.resolve(strict=False)` approximation: cwd-join + lexical
/// `.`/`..` normalization (no symlink resolution fallback).
pub fn abspath(p: &Path) -> PathBuf {
    let joined = if p.is_absolute() {
        p.to_path_buf()
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(p)
    };
    let mut out = PathBuf::new();
    for comp in joined.components() {
        use std::path::Component::*;
        match comp {
            RootDir => out.push("/"),
            CurDir => {}
            ParentDir => {
                out.pop();
            }
            Normal(s) => out.push(s),
            Prefix(_) => {}
        }
    }
    out
}

/// `shlex.quote(s)` — POSIX single-quote with `'"'"'` escapes.
pub fn shell_quote(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    let safe = s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "@%_+=:,./-".contains(c));
    if safe {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

/// `os.path.relpath(path, root)` for the common case where `path` is
/// lexically under `root`; falls back to the absolute path.
pub fn relpath(path: &Path, root: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => path.to_string_lossy().into_owned(),
    }
}

/// `str(v)` for the YAML/JSON scalar values the scripts stringify
/// (`str(extra.get("version"))` and friends).
pub fn py_str(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(u) = n.as_u64() {
                u.to_string()
            } else {
                n.as_f64().map(py_float).unwrap_or_default()
            }
        }
        serde_yaml::Value::Bool(b) => {
            if *b {
                "True".into()
            } else {
                "False".into()
            }
        }
        serde_yaml::Value::Null => "None".into(),
        _ => String::new(),
    }
}

/// Python truthiness for a `serde_json::Value` (`if v:` / `v or x`).
pub fn json_truthy(v: &serde_json::Value) -> bool {
    use serde_json::Value;
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64() != Some(0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Trailing `n` *characters* of `s` (Python `s[-n:]`).
pub fn tail_chars(s: &str, n: usize) -> String {
    let count = s.chars().count();
    if count <= n {
        return s.to_string();
    }
    s.chars().skip(count - n).collect()
}

/// Current unix time as f64 seconds (`time.time()`).
pub fn time_f64() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn shell_quote_safe_and_empty() {
        assert_eq!(shell_quote("abc-def_123./:"), "abc-def_123./:");
        assert_eq!(shell_quote(""), "''");
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote("it's"), "'it'\"'\"'s'");
        assert_eq!(shell_quote("$HOME"), "'$HOME'");
    }

    #[test]
    fn round_py_bankers() {
        assert_eq!(round_py(2.5, 0), 2.0);
        assert_eq!(round_py(3.5, 0), 4.0);
        assert_eq!(round_py(-2.5, 0), -2.0);
        assert_eq!(round_py(1.234, 1), 1.2);
        assert_eq!(round_py(0.0, 0), 0.0);
        assert!(round_py(f64::NAN, 0).is_nan());
    }

    #[test]
    fn py_float_repr() {
        assert_eq!(py_float(1.0), "1.0");
        assert_eq!(py_float(0.5), "0.5");
        assert_eq!(py_float(-2.0), "-2.0");
        assert_eq!(py_float(300.0), "300.0");
    }

    #[test]
    fn json_truthy_table() {
        for v in [
            json!(null),
            json!(false),
            json!(0),
            json!(0.0),
            json!(""),
            json!([]),
            json!({}),
        ] {
            assert!(!json_truthy(&v), "{v:?} should be falsy");
        }
        for v in [
            json!(true),
            json!(1),
            json!(-1.5),
            json!("x"),
            json!([0]),
            json!({"a": null}),
        ] {
            assert!(json_truthy(&v), "{v:?} should be truthy");
        }
    }

    #[test]
    fn tail_chars_unicode_safe() {
        assert_eq!(tail_chars("abcdef", 3), "def");
        assert_eq!(tail_chars("ab", 10), "ab");
        assert_eq!(tail_chars("aé漢", 2), "é漢");
        assert_eq!(tail_chars("", 5), "");
    }

    #[test]
    fn iso_from_unix_epoch_and_fraction() {
        assert_eq!(iso_from_unix(0.0), "1970-01-01T00:00:00+00:00");
        assert_eq!(iso_from_unix(1_700_000_000.0), "2023-11-14T22:13:20+00:00");
        assert_eq!(
            iso_from_unix(1_700_000_000.5),
            "2023-11-14T22:13:20.500000+00:00"
        );
    }

    #[test]
    fn relpath_child_and_outside() {
        assert_eq!(relpath(Path::new("/a/b/c.md"), Path::new("/a/b")), "c.md");
        assert_eq!(relpath(Path::new("/x/y.md"), Path::new("/a/b")), "/x/y.md");
    }

    #[test]
    fn abspath_normalizes() {
        let cwd = env::current_dir().unwrap();
        assert_eq!(abspath(Path::new("x/../y")), cwd.join("y"));
        assert_eq!(abspath(Path::new("/a/../b")), PathBuf::from("/b"));
    }
}
