//! Activity-log rotation — Rust port of `hooks/lib/activity_log_rotate.py`.
//! Triggers: size > CIEL_LOG_MAX_BYTES (5 MiB) or first-line `ts` predates
//! today UTC. Compresses to `.zst` (zstd binary) with `.gz` fallback,
//! prunes archives older than CIEL_LOG_RETENTION_DAYS (90), appends a
//! `sweep`/`log_rotate` marker to the fresh log. All errors swallowed.

use serde_json::{json, Value};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime, PrimitiveDateTime};

use crate::paths;

fn max_bytes() -> u64 {
    std::env::var("CIEL_LOG_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5 * 1024 * 1024)
}

fn retention_days() -> i64 {
    std::env::var("CIEL_LOG_RETENTION_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(90)
}

fn parse_ts(ts: &str) -> Option<OffsetDateTime> {
    OffsetDateTime::parse(ts, &Rfc3339).ok().or_else(|| {
        PrimitiveDateTime::parse(
            ts,
            &time::macros::format_description!(
                "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]"
            ),
        )
        .ok()
        .map(|p| p.assume_utc())
    })
}

fn first_line_date(log: &Path) -> Option<time::Date> {
    let mut buf = String::new();
    std::fs::File::open(log)
        .ok()?
        .read_to_string(&mut buf)
        .ok()?;
    let first = buf.lines().next()?;
    let entry: Value = serde_json::from_str(first).ok()?;
    let ts = entry.get("ts")?.as_str()?;
    Some(parse_ts(ts).unwrap_or_else(OffsetDateTime::now_utc).date())
}

fn compress(src: &Path, dest_base: &Path) -> Option<PathBuf> {
    // zstd binary → .zst (matches the Python's preferred outputs)
    let zdest =
        dest_base.with_file_name(format!("{}.zst", dest_base.file_name()?.to_string_lossy()));
    if Command::new("zstd")
        .args(["-q", "--rm", "-o"])
        .arg(&zdest)
        .arg(src)
        .output()
        .is_ok_and(|o| o.status.success())
    {
        return Some(zdest);
    }
    // gzip fallback → .gz
    let gdest =
        dest_base.with_file_name(format!("{}.gz", dest_base.file_name()?.to_string_lossy()));
    let input = std::fs::read(src).ok()?;
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    enc.write_all(&input).ok()?;
    let bytes = enc.finish().ok()?;
    std::fs::write(&gdest, bytes).ok()?;
    let _ = std::fs::remove_file(src);
    Some(gdest)
}

fn prune(archive: &Path, now: &OffsetDateTime) {
    let cutoff = now.date() - Duration::days(retention_days());
    let Ok(rd) = std::fs::read_dir(archive) else {
        return;
    };
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !(name.starts_with("activity-") && name.contains(".log.")) {
            continue;
        }
        // activity-YYYYMMDD-HHMMSS.log.* → the date stamp
        let Some(stamp) = name.split('-').nth(1) else {
            continue;
        };
        if stamp.len() != 8 || !stamp.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let (Ok(y), Ok(m), Ok(d)) = (
            stamp[..4].parse::<i32>(),
            stamp[4..6].parse::<u8>(),
            stamp[6..8].parse::<u8>(),
        ) else {
            continue;
        };
        let Ok(file_date) = time::Date::from_calendar_date(
            y,
            time::Month::try_from(m).unwrap_or(time::Month::January),
            d,
        ) else {
            continue;
        };
        if file_date < cutoff {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

/// Mirror of `activity_log_rotate.rotate` — returns the marker or None.
pub fn rotate(ciel: &Path, now: &OffsetDateTime) -> Option<Value> {
    let log = ciel.join("activity.log");
    let archive = ciel.join("archive").join("logs");
    if !log.is_file() {
        return None;
    }
    let size = log.metadata().ok()?.len();
    let reason = if size > max_bytes() {
        Some("size")
    } else {
        first_line_date(&log).and_then(|d| (d < now.date()).then_some("daily"))
    }?;

    let _ = std::fs::create_dir_all(&archive);
    let stamp = now
        .format(&time::macros::format_description!(
            "[year][month][day]-[hour][minute][second]"
        ))
        .unwrap_or_else(|_| "00000000-000000".to_string());
    let dest_base = archive.join(format!("activity-{stamp}.log"));
    std::fs::rename(&log, &dest_base).ok()?;
    let compressed = compress(&dest_base, &dest_base)?;
    prune(&archive, now);

    let marker = json!({
        "ts": paths::utc_now_iso(),
        "kind": "sweep",
        "op": "log_rotate",
        "archived_to": format!(
            "archive/logs/{}",
            compressed.file_name().unwrap_or_default().to_string_lossy()),
        "reason": reason,
        "bytes": size,
    });
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log)
    {
        // Python writes json.dumps(marker) — default separators, ensure_ascii.
        let _ = writeln!(f, "{}", crate::jsonfmt::dumps(&marker));
    }
    Some(marker)
}

/// `ciel log-rotate` — silent like the Python (exit 0 always).
pub fn main_() -> i32 {
    let _ = rotate(&paths::ciel_home(), &OffsetDateTime::now_utc());
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc3339_parse() {
        assert!(parse_ts("2026-09-24T05:07:49.322523+00:00").is_some());
        assert!(parse_ts("2026-09-24T05:07:49Z").is_some());
    }
}
