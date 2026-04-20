# TEMPLATES — Index

Instantiation scaffolds Ciel uses to create new artifacts.

## Index

| Template | When |
| --- | --- |
| `skill.template.md` | Any new `.skill` bundle |
| `subagent.template.md` | Council member, acquisition worker, domain-specialist subagent |
| `council_vote.template.md` | Each Council member's scored output per stage |
| `activity_log_entry.template.md` | Every `activity.log` line |
| `adapter.template.md` | New runtime adapter scaffold |
| `config.template.md` | New config field or config file scaffold |
| `risk_assessment.template.md` | Risk classification + judge output record |
| `improvement_proposal.template.md` | Self-improvement proposal |

## Instantiation Protocol

1. Load template.
2. Fill placeholders (`{{name}}`, `{{version}}`, ...).
3. Validate against its schema (where applicable).
4. Commit via the appropriate flow (Council-gated when required).

## Versioning

Templates have their own versions. Changes flow through `self_improvement/` and Council.
