# CONVERSATION_AUDIT — Recurring Cross-Runtime Retrospective

Docket `council-20260923-conversation-audit`, council addition: the
cross-runtime conversation audit is a repeatable growth-signal generator —
run it on a monthly cadence under the `scheduled_sweep` trigger.

## Cadence

Monthly (the weekly `scheduled_sweep` fires it once per calendar month).

## Corpus

| Runtime | Store | Notes |
| --- | --- | --- |
| devin-cli | `~/.local/share/devin/cli/transcripts/*.json` | full steps; user turns are `type: "user"` messages |
| devin-cli | `~/.local/share/devin/cli/summaries/history_*.md` | condensed session summaries; quote user requests verbatim |
| agy-cli | `~/.gemini/antigravity*/conversations/*.db` | SQLite `steps.step_payload` protobuf; `step_type=14` ≈ user prompts — **filter agent-spawned council/subagent prompts** (single-step DBs, system-prompt content) |
| windsurf / a-studio / agy-ide | `~/.gemini/antigravity-ide \| antigravity-cli/...`, cascade dirs | check for empties before trusting zero-counts |

## Method (proven 2026-09-23)

1. Extract direct user turns; separate human prompts from agent-generated

   spawns by conversation shape and content.

2. Classify recurring requests/corrections; count distinct sessions, not

   raw hits — a correction in N sessions is a mechanism candidate.

3. Scan agent-side failure patterns (error storms, repeated escalations,

   "still broken" admissions).

4. Check store file permissions — conversation stores must be owner-only.
5. Summarize findings: aggregate counts, sanitized examples, file paths,

   session ids — **never reproduce secret values**.

6. Distinguish one-off situations from durable mechanism candidates.
7. Findings go to the improvement signal store

   (`~/.ciel/improvements/signals/audit-<ts>.json`); mechanism candidates
   get a Council docket — as this audit's mechanisms did.

## Safety

- Stores are `0600`/`0700` post-2026-09-23 sanitization; the sweep in

  `hooks/lib/store_perms.py` re-enforces every session.

- Never log or emit matched secret content; categories only.
- Credentials found in transcripts are presumed compromised → recommend

  rotation, do not handle.
