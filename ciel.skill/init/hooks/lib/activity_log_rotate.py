#!/usr/bin/env python3
"""Rotate ~/.ciel/activity.log per observability/ACTIVITY_LOG.md §Rotation.

Triggers: the log exceeds CIEL_LOG_MAX_BYTES (default 5 MiB), or its first
line is a JSON entry whose `ts` predates today's UTC date (daily rotation).
Rotated logs are renamed atomically, compressed to
~/.ciel/archive/logs/activity-YYYYMMDD-HHMMSS.log.zst (gzip fallback), and
archives older than CIEL_LOG_RETENTION_DAYS (default 90) are pruned. A
`sweep`/`log_rotate` marker line is appended to the fresh log.

Session-boundary hooks (devin session_start.sh, antigravity
pre_invocation.sh) call this once per session. All OSError are swallowed;
exit code is always 0.
"""

import contextlib
import gzip
import json
import os
import shutil
import subprocess
import sys
from datetime import date, datetime, timedelta, timezone
from pathlib import Path

try:
    from compression import zstd as _zstd
except ImportError:
    _zstd = None

MAX_BYTES = int(os.environ.get("CIEL_LOG_MAX_BYTES", str(5 * 1024 * 1024)))
RETENTION_DAYS = int(os.environ.get("CIEL_LOG_RETENTION_DAYS", "90"))
CIEL_HOME = Path(os.environ.get("CIEL_HOME", str(Path.home() / ".ciel")))

ARCHIVE_GLOB = "activity-*.log.*"


def _first_line_date(log: Path) -> date | None:
    try:
        with log.open("r", encoding="utf-8", errors="replace") as fh:
            first = fh.readline()
        entry = json.loads(first)
        ts = entry["ts"]
        if isinstance(ts, str) and ts.endswith("Z"):
            ts = ts[:-1] + "+00:00"
        parsed = datetime.fromisoformat(ts)
        if parsed.tzinfo is None:
            parsed = parsed.replace(tzinfo=timezone.utc)
        return parsed.astimezone(timezone.utc).date()
    except (OSError, ValueError, KeyError, AttributeError, TypeError):
        return None


def _compress(src: Path, dest_base: Path) -> Path | None:
    """Compress src to dest_base + '.zst' (or '.gz' fallback)."""
    if _zstd is not None:
        try:
            dest = dest_base.parent / (dest_base.name + ".zst")
            dest.write_bytes(_zstd.compress(src.read_bytes()))
            src.unlink(missing_ok=True)
            return dest
        except OSError:
            pass
    if shutil.which("zstd"):
        dest = dest_base.parent / (dest_base.name + ".zst")
        try:
            subprocess.run(
                ["zstd", "-q", "--rm", "-o", str(dest), str(src)],
                check=True,
                capture_output=True,
            )
            return dest
        except (OSError, subprocess.CalledProcessError):
            pass
    try:
        dest = dest_base.parent / (dest_base.name + ".gz")
        with src.open("rb") as fin, gzip.open(dest, "wb") as fout:
            shutil.copyfileobj(fin, fout)
        src.unlink(missing_ok=True)
        return dest
    except OSError:
        return None


def _prune(archive: Path, now: datetime) -> None:
    cutoff = now.date() - timedelta(days=RETENTION_DAYS)
    for old in archive.glob(ARCHIVE_GLOB):
        try:
            stamp = old.name.split("-", 2)[1]
            file_date = datetime.strptime(stamp, "%Y%m%d").replace(tzinfo=timezone.utc).date()
        except (IndexError, ValueError):
            continue
        if file_date < cutoff:
            with contextlib.suppress(OSError):
                old.unlink()


def rotate(ciel_home: Path, now: datetime) -> dict | None:
    """Rotate ciel_home/activity.log if a trigger fires.

    Returns the marker dict written to the fresh log, or None when no
    rotation was needed.
    """
    log = ciel_home / "activity.log"
    archive = ciel_home / "archive" / "logs"
    try:
        if not log.is_file():
            return None
        size = log.stat().st_size
        reason = None
        if size > MAX_BYTES:
            reason = "size"
        else:
            first_date = _first_line_date(log)
            if first_date is not None and first_date < now.date():
                reason = "daily"
        if reason is None:
            return None

        archive.mkdir(parents=True, exist_ok=True)
        stamp = now.strftime("%Y%m%d-%H%M%S")
        dest_base = archive / f"activity-{stamp}.log"
        log.rename(dest_base)
        compressed = _compress(dest_base, dest_base)
        if compressed is None:
            return None
        _prune(archive, now)

        marker = {
            "ts": now.isoformat(),
            "kind": "sweep",
            "op": "log_rotate",
            "archived_to": f"archive/logs/{compressed.name}",
            "reason": reason,
            "bytes": size,
        }
        with log.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(marker) + "\n")
        return marker
    except OSError:
        return None


def main() -> int:
    rotate(CIEL_HOME, datetime.now(timezone.utc))
    return 0


if __name__ == "__main__":
    sys.exit(main())
