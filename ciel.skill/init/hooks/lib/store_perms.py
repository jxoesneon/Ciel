#!/usr/bin/env python3
"""Owner-only permission self-heal for Ciel and host-runtime state stores.

Conversation stores and Ciel state hold prompt content, shell commands,
council deliberations, and cached model state. They must stay owner-only
(dir 0700, file 0600). This sweep is idempotent and cheap: it only touches
paths whose permissions have drifted, and prints ``ok`` or ``repaired:N``
so session_start can surface a one-line note when it had to act.

Scope is deliberately narrow — state-bearing paths only. Source trees,
venvs, and skill payloads keep normal executable perms.
"""

import os
import stat
import sys
from pathlib import Path

HOME = Path.home()
CIEL = HOME / ".ciel"
DEVIN_CLI = HOME / ".local" / "share" / "devin" / "cli"

# (path, dir_mode, file_mode) — files get file_mode, dirs get dir_mode.
# Globs are expanded per entry. Missing paths are skipped silently.
DIR_TARGETS = [
    CIEL,
    CIEL / "system1",
    CIEL / "system1" / "cache",
    CIEL / "system1" / "inflight",
    CIEL / "council",
    CIEL / "improvements",
    CIEL / "checkpoints",
    CIEL / "logs",
    CIEL / "backups",
    CIEL / "archive",
    CIEL / ".attic",
    CIEL / "ciel.skill" / "memory",
    DEVIN_CLI / "transcripts",
    DEVIN_CLI / "summaries",
    DEVIN_CLI / "logs",
]
# Subtrees whose internals are code, not state — never swept.
SKIP_PREFIXES = [CIEL / "system1" / "venv"]

STATE_FILES = [
    CIEL / "activity.log",
    CIEL / "palace.db",
    CIEL / "context.md",
    CIEL / "bootstrap.log",
    CIEL / "INSTALLED.json",
    CIEL / "INTEGRITY.json",
    CIEL / "allow_privileged",
    CIEL / "grants.log",
    CIEL / "system1" / "events.jsonl",
    DEVIN_CLI / "sessions.db",
    DEVIN_CLI / "sessions.db-wal",
    DEVIN_CLI / "sessions.db-shm",
]

FILE_GLOBS = [
    (DEVIN_CLI / "transcripts", "*.json"),
    (DEVIN_CLI / "summaries", "*.md"),
    (DEVIN_CLI / "logs", "*.log"),
    (DEVIN_CLI / "logs", "*.log.gz"),
    (CIEL / "council", "**/*.json"),
    (CIEL / "system1" / "cache", "**/*"),
    (CIEL / "system1" / "inflight", "**/*"),
    (CIEL / "logs", "*.log"),
    (CIEL / "backups", "*"),
]

# Antigravity conversation DBs — already 0600 by default, swept anyway so a
# runtime regression self-heals on the next Devin session.
AGY_GLOBS = [
    (HOME / ".gemini" / "antigravity" / "conversations", "*.db"),
    (HOME / ".gemini" / "antigravity-ide" / "conversations", "*.db"),
    (HOME / ".gemini" / "antigravity-cli" / "conversations", "*.db"),
]


def _skipped(p: Path) -> bool:
    return any(p == s or s in p.parents for s in SKIP_PREFIXES)


def _fix_file(p: Path, repaired: list) -> None:
    try:
        if p.is_file() and not _skipped(p) and p.stat().st_mode & 0o077:
            os.chmod(p, 0o600)
            repaired.append(str(p))
    except OSError:
        pass


def _fix_dir(p: Path, repaired: list) -> None:
    try:
        if p.is_dir() and not _skipped(p) and p.stat().st_mode & 0o077:
            os.chmod(p, 0o700)
            repaired.append(str(p))
    except OSError:
        pass


def main() -> int:
    repaired: list = []
    for d in DIR_TARGETS:
        _fix_dir(d, repaired)
    for f in STATE_FILES:
        _fix_file(f, repaired)
    for base, pattern in FILE_GLOBS + AGY_GLOBS:
        if not base.is_dir():
            continue
        try:
            for p in base.glob(pattern):
                _fix_file(p, repaired)
        except OSError:
            pass
    print("ok" if not repaired else f"repaired:{len(repaired)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
