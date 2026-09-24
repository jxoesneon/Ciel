//! Owner-only permission self-heal — Rust port of `hooks/lib/store_perms.py`.
//! State-bearing paths only: dirs 0700, files 0600. Prints `ok` or
//! `repaired:N` exactly like the Python — session_start surfaces the count.

use std::fs;
use std::path::{Path, PathBuf};

use crate::paths;

fn devin_cli(home: &Path) -> PathBuf {
    home.join(".local").join("share").join("devin").join("cli")
}

fn dir_targets(home: &Path, ciel: &Path) -> Vec<PathBuf> {
    let cli = devin_cli(home);
    vec![
        ciel.to_path_buf(),
        ciel.join("system1"),
        ciel.join("system1").join("cache"),
        ciel.join("system1").join("inflight"),
        ciel.join("council"),
        ciel.join("improvements"),
        ciel.join("checkpoints"),
        ciel.join("logs"),
        ciel.join("backups"),
        ciel.join("archive"),
        ciel.join(".attic"),
        ciel.join("ciel.skill").join("memory"),
        cli.join("transcripts"),
        cli.join("summaries"),
        cli.join("logs"),
    ]
}

fn state_files(home: &Path, ciel: &Path) -> Vec<PathBuf> {
    let cli = devin_cli(home);
    vec![
        ciel.join("activity.log"),
        ciel.join("palace.db"),
        ciel.join("context.md"),
        ciel.join("bootstrap.log"),
        ciel.join("INSTALLED.json"),
        ciel.join("INTEGRITY.json"),
        ciel.join("allow_privileged"),
        ciel.join("grants.log"),
        ciel.join("system1").join("events.jsonl"),
        cli.join("sessions.db"),
        cli.join("sessions.db-wal"),
        cli.join("sessions.db-shm"),
    ]
}

/// (base, pattern) — pattern is `*` / `*.ext` / `**/*` / `**/*.ext`.
fn file_globs(home: &Path, ciel: &Path) -> Vec<(PathBuf, &'static str)> {
    let cli = devin_cli(home);
    vec![
        (cli.join("transcripts"), "*.json"),
        (cli.join("summaries"), "*.md"),
        (cli.join("logs"), "*.log"),
        (cli.join("logs"), "*.log.gz"),
        (ciel.join("council"), "**/*.json"),
        (ciel.join("system1").join("cache"), "**/*"),
        (ciel.join("system1").join("inflight"), "**/*"),
        (ciel.join("logs"), "*.log"),
        (ciel.join("backups"), "*"),
        (
            home.join(".gemini")
                .join("antigravity")
                .join("conversations"),
            "*.db",
        ),
        (
            home.join(".gemini")
                .join("antigravity-ide")
                .join("conversations"),
            "*.db",
        ),
        (
            home.join(".gemini")
                .join("antigravity-cli")
                .join("conversations"),
            "*.db",
        ),
    ]
}

fn skipped(p: &Path, ciel: &Path) -> bool {
    p.starts_with(ciel.join("system1").join("venv"))
}

/// Filename match for the two pattern shapes the sweep uses.
fn name_matches(name: &str, pattern: &str) -> bool {
    match pattern.strip_prefix('*') {
        Some("") => true,                       // "*"
        Some(suffix) => name.ends_with(suffix), // "*.ext"
        None => name == pattern,
    }
}

fn walk(base: &Path, pattern: &str, out: &mut Vec<PathBuf>) {
    let recursive = pattern.starts_with("**/");
    let tail = pattern.strip_prefix("**/").unwrap_or(pattern);
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if recursive {
                    stack.push(p);
                }
            } else if name_matches(p.file_name().and_then(|n| n.to_str()).unwrap_or(""), tail) {
                out.push(p);
            }
        }
    }
}

#[cfg(unix)]
fn mode_of(p: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    p.metadata().ok().map(|m| m.permissions().mode() & 0o777)
}

#[cfg(unix)]
fn chmod(p: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(p, fs::Permissions::from_mode(mode));
}

fn fix_file(p: &Path, ciel: &Path, repaired: &mut Vec<PathBuf>) {
    if p.is_file() && !skipped(p, ciel) && mode_of(p).is_some_and(|m| m & 0o077 != 0) {
        chmod(p, 0o600);
        repaired.push(p.to_path_buf());
    }
}

fn fix_dir(p: &Path, ciel: &Path, repaired: &mut Vec<PathBuf>) {
    if p.is_dir() && !skipped(p, ciel) && mode_of(p).is_some_and(|m| m & 0o077 != 0) {
        chmod(p, 0o700);
        repaired.push(p.to_path_buf());
    }
}

/// Run the full sweep; returns the repaired-path count (0 = all clean).
pub fn sweep(home: &Path, ciel: &Path) -> usize {
    let mut repaired = Vec::new();
    for d in dir_targets(home, ciel) {
        fix_dir(&d, ciel, &mut repaired);
    }
    for f in state_files(home, ciel) {
        fix_file(&f, ciel, &mut repaired);
    }
    for (base, pattern) in file_globs(home, ciel) {
        if !base.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        walk(&base, pattern, &mut files);
        for f in files {
            fix_file(&f, ciel, &mut repaired);
        }
    }
    repaired.len()
}

/// `ciel store-perms` — sweep + `ok`/`repaired:N`.
pub fn main_() -> i32 {
    let n = sweep(&paths::home_dir(), &paths::ciel_home());
    if n == 0 {
        println!("ok");
    } else {
        println!("repaired:{n}");
    }
    0
}
