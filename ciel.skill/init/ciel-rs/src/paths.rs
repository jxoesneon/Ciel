//! Home/path resolution shared by every subcommand. Mirrors the Python
//! helpers in `hooks/lib/risk_policy.py` exactly — the verdicts the binary
//! emits must be byte-for-byte compatible with the Python fallback.

use std::env;
use std::path::{Path, PathBuf};

/// `CIEL_HOME` override else `~/.ciel` (mirrors `risk_policy.ciel_home`).
pub fn ciel_home() -> PathBuf {
    match env::var_os("CIEL_HOME") {
        Some(v) if !v.is_empty() => PathBuf::from(v),
        _ => home_dir().join(".ciel"),
    }
}

/// User home — mirrors `Path.home()` (HOME env, then passwd entry).
pub fn home_dir() -> PathBuf {
    if let Some(h) = env::var_os("HOME") {
        if !h.is_empty() {
            return PathBuf::from(h);
        }
    }
    dirs_fallback()
}

fn dirs_fallback() -> PathBuf {
    env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| {
            let drive = env::var_os("HOMEDRIVE")?;
            let path = env::var_os("HOMEPATH")?;
            Some(PathBuf::from(format!(
                "{}{}",
                drive.to_string_lossy(),
                path.to_string_lossy()
            )))
        })
        .unwrap_or_else(|| PathBuf::from("/"))
}

/// `~name` home lookup — mirrors `os.path.expanduser` which uses the
/// password database (getpwnam). Unknown users stay literal, like Python.
#[cfg(unix)]
fn user_home(name: &str) -> Option<PathBuf> {
    let c = std::ffi::CString::new(name).ok()?;
    unsafe {
        let pw = libc::getpwnam(c.as_ptr());
        if pw.is_null() {
            return None;
        }
        let dir = std::ffi::CStr::from_ptr((*pw).pw_dir);
        Some(PathBuf::from(dir.to_string_lossy().into_owned()))
    }
}

#[cfg(not(unix))]
fn user_home(_name: &str) -> Option<PathBuf> {
    None
}

/// Mirror of `expanduser(expandvars(raw))`: environment references resolve
/// FIRST so a `$VAR` holding `~/..` still gets tilde-expanded; then `~`,
/// `~/x`, and `~user/x` expand (unknown users stay literal, like Python).
fn expand(raw: &str, home: &Path) -> String {
    // expandvars: $NAME and ${NAME}; os.path.expandvars leaves unset vars
    // literal ($FOO stays "$FOO") — mirror that exactly.
    let mut s = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                if let Some(end) = raw[i + 2..].find('}') {
                    let name = &raw[i + 2..i + 2 + end];
                    match env::var(name) {
                        Ok(v) => s.push_str(&v),
                        Err(_) => s.push_str(&raw[i..i + 2 + end + 1]),
                    }
                    i += 2 + end + 1;
                    continue;
                }
            } else {
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                    j += 1;
                }
                if j > start {
                    match env::var(&raw[start..j]) {
                        Ok(v) => s.push_str(&v),
                        Err(_) => s.push_str(&raw[i..j]),
                    }
                    i = j;
                    continue;
                }
            }
        }
        s.push(bytes[i] as char);
        i += 1;
    }
    // expanduser second — a var may inject a leading `~`.
    if s == "~" {
        return home.to_string_lossy().into_owned();
    }
    if let Some(rest) = s.strip_prefix("~/") {
        return format!("{}/{}", home.to_string_lossy(), rest);
    }
    if let Some(user_part) = s.strip_prefix('~') {
        if let Some(slash) = user_part.find('/') {
            let name = &user_part[..slash];
            if let Some(h) = user_home(name) {
                return format!("{}{}", h.to_string_lossy(), &user_part[slash..]);
            }
        } else if !user_part.is_empty() {
            if let Some(h) = user_home(user_part) {
                return h.to_string_lossy().into_owned();
            }
        }
    }
    s
}

/// Lexical normpath: collapse `//`, `/./`, resolve `/../` without touching
/// the filesystem (mirrors `os.path.normpath` semantics closely enough for
/// the policy surface — trailing-slash and root handling included).
fn normpath(p: &str) -> String {
    let absolute = p.starts_with('/');
    let mut parts: Vec<&str> = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => continue,
            ".." => {
                if parts.last().is_some_and(|l| *l != "..") {
                    parts.pop();
                } else if !absolute {
                    parts.push("..");
                }
            }
            s => parts.push(s),
        }
    }
    let joined = parts.join("/");
    if absolute {
        format!("/{joined}")
    } else if joined.is_empty() {
        ".".to_string()
    } else {
        joined
    }
}

/// Mirror of `risk_policy._normalize_path`: expand, normpath, then collapse
/// the home prefix to `~` so policy patterns see a canonical subject.
pub fn normalize_path(raw: &str, home: &Path) -> String {
    if raw.is_empty() {
        return String::new();
    }
    let expanded = normpath(&expand(raw, home));
    for base in [home.to_path_buf(), home_dir()] {
        let b = base.to_string_lossy().replace('\\', "/");
        if expanded == b {
            return "~".to_string();
        }
        if let Some(rest) = expanded.strip_prefix(&format!("{b}/")) {
            return format!("~/{rest}");
        }
    }
    expanded
}

/// RFC-3339 timestamp in the exact shape `datetime.now(timezone.utc)
/// .isoformat()` produces: `YYYY-MM-DDTHH:MM:SS.ffffff+00:00` — activity.log
/// consumers (`session_watchdog`, `activity_log_rotate`) parse with
/// `fromisoformat`, so the shape must match.
pub fn utc_now_iso() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}+00:00",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        now.microsecond()
    )
}

/// Append one JSON line to `~/.ciel/activity.log`; errors are swallowed the
/// same way the Python hooks swallow OSError.
pub fn activity_log(entry: &serde_json::Value) {
    let path = ciel_home().join("activity.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        use std::io::Write;
        // Python: json.dumps(entry, ensure_ascii=False) — spaced, raw UTF-8.
        let _ = writeln!(f, "{}", crate::jsonfmt::dumps_raw(entry));
    }
}
