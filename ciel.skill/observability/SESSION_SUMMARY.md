# SESSION_SUMMARY

Per-session summary rendered at session end (when configured).

## Trigger

- `observability.config.session_summary: on` — always at session end.
- `observability.config.session_summary: on_error` — only when the session encountered errors or escalations.
- `observability.config.session_summary: off` — disabled.

## Format

Markdown, rendered to the user's native output + saved to `~/.ciel/sessions/<session_id>.md`:

```markdown
## Ciel session summary

- **Runtime**: claude-code
- **Project**: <hash>
- **Duration**: 42m
- **Requests routed**: 18 (fast: 12, reasoning: 5, acquisition: 1)
- **Council runs**: 1 (pass)
- **Acquisitions**: 1 new skill — `html_linter`
- **Risk distribution**: low 14 / mid 3 / high 1 / critical 0
- **Escalations**: 0
- **Errors**: 0
- **Cost**: $0.18
- **Notable**:
  - Added `html_linter/SKILL.md` to registry.
  - Improved fast-path trigger coverage for `docker:*` (proposal #42 applied).
  - `git push origin main` gated by judge — approved after dry-run.

Next steps:
- None pending.

Pending improvement proposals: 2 (see `/ciel-diff`).
```

## Privacy

Project names, file paths, and secrets are not included in the summary body (only hashed project id). Summaries are safe to share for debugging.

## Storage

Retained per `observability.config.session_summary_retention_days` (default 30). Older summaries compacted into monthly rollups.

## Uses

- Debugging.
- Telemetry analysis.
- Cost tracking.
- User confidence ("here's exactly what Ciel did").
