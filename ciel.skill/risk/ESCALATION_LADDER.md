# ESCALATION_LADDER

End-to-end decision flow.

```text
┌─────────────────────────┐
│ Operation proposed      │
└──────────┬──────────────┘
           v
┌─────────────────────────┐
│ Classify (CLASSIFICATION)│
└──────────┬──────────────┘
           v
    ┌──────┴──────┐
   low           mid / high / critical
    │               │
    │               v
    │         ┌─────────────────┐
    │         │ research first? │── research via seed_skills/research
    │         │ (if config)     │
    │         └─────────┬───────┘
    │                   v
    │         ┌─────────┴─────────┐
    │         mid              high           critical
    │          │                 │                │
    │          v                 v                v
    │    LLM_JUDGE         Council of Five    User escalation
    │          │                 │                │   (always)
    │   proceed?            pass?                 │
    │          │                 │                │
    │         yes                yes              approved?
    │          │                 │                │
    │          └────────┬────────┘                │
    │                   │                         │
    │                   v                         yes
    │              Execute + log                  │
    │                                             v
    │                                    Execute + post-mortem
    │
    v
Execute (autonomous) + log
```

## Research-First

If `autonomy.research_first: true` (default), any classification ≥ mid receives a research pass before deciding the gate. Research output informs judge / Council.

## Fall-Through Rules

- Judge `abort` → high treatment.
- Council `deadlock` → `council/ESCALATION.md`.
- User no-response within `escalation.timeout_hours` (default 72) → operation abandoned; Ciel logs and moves on.

## Project Profile Shift

`configuration/local/escalation.config.md.effective`:

- `research` — shifts the mid/high boundary higher (more autonomy).
- `development` — defaults.
- `production` — shifts boundary lower (more gating).
- `regulated` — lowest; almost everything non-trivial is at least mid.

Constitutional floor ensures critical always escalates regardless of profile.
