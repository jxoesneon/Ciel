# ACTIVITY_LOG

Append-only structured log of every meaningful Ciel action.

## Location

- Global: `~/.ciel/activity.log`
- Local mirror (project-scoped entries only): `<project>/.ciel/activity.log`

## Format

One JSON object per line (JSONL). Schema:

```json
{
  "ts": "2026-01-15T09:24:00.123Z",
  "session": "<session_id>",
  "runtime": "claude_code|gemini_cli|generic",
  "project": "<hash>|null",
  "kind": "route|acquisition|council|mutation|escalation|health|merge|permission|error|improvement|sweep|backup",
  "op": "<short-op>",
  "risk": "low|mid|high|critical",
  "path": "fast|reasoning|acquisition|n/a",
  "skill": "<skill_id>|null",
  "decision": "allow|deny|ask|defer|pass|reject|proceed|revise|abort|n/a",
  "duration_ms": 123,
  "tokens": {"in": 0, "out": 0, "cached": 0},
  "cost_usd": 0.0,
  "commit": "<sha>|null",
  "council_run": "<id>|null",
  "notes": "short prose",
  "redacted": ["secret-like fields removed"]
}
```

Template at `templates/activity_log_entry.template.md`.

## Rotation

- Rotation is performed by `init/hooks/lib/activity_log_rotate.py`, invoked once per session from the session-boundary hooks (`hooks/devin/session_start.sh`, `hooks/antigravity/pre_invocation.sh`).
- Triggers: the log crosses the UTC day boundary (first entry's `ts` is earlier than today UTC) **or** it exceeds 5 MiB (`CIEL_LOG_MAX_BYTES`).
- Rotated files: `~/.ciel/archive/logs/activity-YYYYMMDD-HHMMSS.log.zst` (UTC rotation timestamp; `.log.gz` when no zstd backend is available). The log is renamed atomically, then compressed.
- Retention: archives older than 90 days (`CIEL_LOG_RETENTION_DAYS`) are pruned on each rotation.
- After rotation a `{"kind": "sweep", "op": "log_rotate"}` marker line is appended to the fresh log recording the archive path, trigger reason, and pre-rotation size.

## Redaction

Values detected as secrets (env-var shape, high-entropy tokens, matching known patterns) are replaced with `"<redacted>"` and the field name appears in `redacted[]`. Redaction is at write time.

## Querying

`seed_skills/log_analyzer/SKILL.md`:

- `logs.recent(n)`
- `logs.by_kind(kind)`
- `logs.failures(since)`
- `logs.cost(since)`
- `logs.search(pattern)`

## Never

- No secrets written raw.
- No project-specific paths crossing partition boundaries in the log body.
- No silent events. If Ciel acted, it's here.
