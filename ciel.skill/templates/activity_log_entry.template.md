# Template — activity.log entry

One JSON object per line. Must validate against this shape.

```json
{
  "ts": "{{iso8601}}",
  "session": "{{session_id}}",
  "runtime": "claude_code|gemini_cli|generic",
  "project": "{{hash}}|null",
  "kind": "route|acquisition|council|mutation|escalation|health|merge|permission|error|improvement|sweep|backup|trace",
  "op": "{{short-op}}",
  "risk": "low|mid|high|critical",
  "path": "fast|reasoning|acquisition|n/a",
  "skill": "{{id}}|null",
  "decision": "allow|deny|ask|defer|pass|reject|proceed|revise|abort|n/a",
  "duration_ms": 0,
  "tokens": {"in": 0, "out": 0, "cached": 0},
  "cost_usd": 0.0,
  "commit": "{{sha}}|null",
  "council_run": "{{id}}|null",
  "notes": "<=120 chars",
  "redacted": []
}
```

## Required Fields

`ts`, `session`, `kind`, `op`.

## Redaction

Any field value matching secret patterns is replaced with `"<redacted>"` and the field name appended to `redacted[]` before write.
