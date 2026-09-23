#!/usr/bin/env python3
"""Rotate ~/.ciel/activity.log when it exceeds MAX_BYTES.

Every lifecycle hook (PreToolUse, PostToolUse, PermissionRequest, Stop,
SessionEnd, and the Antigravity equivalents) appends one JSON line per
invocation, so without rotation the log grows without bound — observed
34 MiB / ~160k lines over 9 days.

Session-boundary hooks (devin session_start.sh, antigravity
pre_invocation.sh) call this once per session. When the log is over the
threshold it is moved to ~/.ciel/archive/activity-<UTC>.log, the newest
KEEP archives are retained, and a fresh log is opened with a
`log_rotated` marker line.
"""

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

MAX_BYTES = 5 * 1024 * 1024
KEEP_ARCHIVES = 3


def main() -> int:
    home = Path.home()
    log = home / ".ciel" / "activity.log"
    archive = home / ".ciel" / "archive"

    try:
        if not log.exists() or log.stat().st_size <= MAX_BYTES:
            return 0
        archive.mkdir(parents=True, exist_ok=True)
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        dest = archive / f"activity-{stamp}.log"
        log.rename(dest)
        for old in sorted(archive.glob("activity-*.log"))[:-KEEP_ARCHIVES]:
            old.unlink(missing_ok=True)
        entry = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "event": "log_rotated",
            "archived_to": f"archive/{dest.name}",
            "max_bytes": MAX_BYTES,
        }
        with log.open("a", encoding="utf-8") as fh:
            fh.write(json.dumps(entry) + "\n")
    except OSError:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
